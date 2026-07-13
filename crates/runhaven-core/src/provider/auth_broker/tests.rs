use super::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

struct NoopUpstream;

impl BrokerUpstream for NoopUpstream {
    fn send(
        &self,
        _method: &str,
        _path: &str,
        _headers: &[(String, String)],
        _body: &[u8],
    ) -> Result<BrokerUpstreamResponse> {
        panic!("oversized request must not reach upstream")
    }
}

fn test_broker() -> ApiKeyBrokerProxy {
    ApiKeyBrokerProxy::bind_with_upstream(
        ("127.0.0.1", 0),
        CODEX_BROKER,
        SecretString::from("test-key"),
        &[],
        Arc::new(NoopUpstream),
    )
    .expect("test broker")
}

fn read_status_line(stream: &mut TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout");
    let mut response = Vec::new();
    let mut byte = [0_u8; 1];
    while stream.read(&mut byte).expect("response byte") != 0 {
        response.push(byte[0]);
        if response.ends_with(b"\r\n") {
            break;
        }
    }
    String::from_utf8(response).expect("response utf8")
}

#[test]
fn sanitize_broker_request_path_strips_query_and_fragment() {
    assert_eq!(
        sanitize_broker_request_path("/v1/responses?token=secret#frag"),
        "/v1/responses"
    );
    assert_eq!(
        sanitize_broker_request_path("/v1/responses#fragment"),
        "/v1/responses"
    );
    assert_eq!(sanitize_broker_request_path("?token=secret"), "-");
}

/// Type-enforced no-leak guarantee for the held broker credential. The token is
/// held for the run as `BrokerState.api_key: SecretString` (see this module's
/// parent). This proves that formatting that exact type with `{:?}` renders
/// `[REDACTED]` and never the token, so a stray debug format of `BrokerState` or
/// this field cannot leak the credential. `SecretString` also has no `Display`
/// impl, so `{}` on the token would fail to compile; the only path to the
/// plaintext is the single audited `expose_secret()` call at header injection.
#[test]
fn held_broker_token_type_is_redacted_in_debug_output() {
    let token = "sk-ant-oat01-SECRETTOKENVALUE";
    let held: SecretString = SecretString::from(token);
    // Sanity: the value is really present behind the audited accessor.
    assert_eq!(held.expose_secret(), token);

    let debug = format!("{held:?}");
    assert!(
        !debug.contains(token) && !debug.contains("sk-ant"),
        "the held credential type must never render its plaintext in Debug, got {debug:?}"
    );
    assert!(
        debug.contains("REDACTED"),
        "Debug of the held credential type must be redacted, got {debug:?}"
    );
}

#[test]
fn broker_request_body_limit_is_small_enough_for_host_memory_safety() {
    assert_eq!(MAX_CODEX_BROKER_REQUEST_BYTES, 8 * 1024 * 1024);
    assert_eq!(BROKER_CLIENT_IO_TIMEOUT_SECONDS, 10);
    assert_eq!(MAX_CONCURRENT_BROKER_CLIENTS, 8);
}

#[test]
fn broker_rejects_an_oversized_request_line_before_parsing_headers() {
    let broker = test_broker();
    let server = {
        let broker = broker.clone();
        thread::spawn(move || broker.serve_forever())
    };
    let mut stream = TcpStream::connect(broker.server_addr().expect("broker address"))
        .expect("broker connection");
    let request = format!("POST /v1/{} HTTP/1.1\r\n\r\n", "a".repeat(8 * 1024));

    stream
        .write_all(request.as_bytes())
        .expect("oversized request line");

    assert!(read_status_line(&mut stream).starts_with("HTTP/1.1 414"));
    broker.shutdown();
    server.join().expect("broker server");
}

#[test]
fn broker_closes_an_excess_client_before_spawning_a_worker() {
    let mut broker = test_broker();
    Arc::get_mut(&mut broker.state)
        .expect("unshared broker state")
        .client_admission = ClientAdmission::new(1);
    let (server_started_tx, server_started_rx) = mpsc::channel();
    let server = {
        let broker = broker.clone();
        thread::spawn(move || {
            server_started_tx.send(()).expect("server started");
            broker.serve_forever();
        })
    };
    server_started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("broker server thread did not start");
    let address = broker.server_addr().expect("broker address");
    let first = TcpStream::connect(address).expect("first client");
    let deadline = Instant::now() + Duration::from_secs(5);
    while broker.state.client_admission.active() != 1 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        broker.state.client_admission.active(),
        1,
        "broker did not accept the first client before the deadline"
    );
    let mut excess = TcpStream::connect(address).expect("excess client");
    excess
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("excess timeout");
    let mut byte = [0_u8; 1];

    let closed = match excess.read(&mut byte) {
        Ok(0) => true,
        Err(error) => matches!(
            error.kind(),
            std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
        ),
        _ => false,
    };

    assert!(closed, "excess broker client must be closed");
    drop(first);
    broker.shutdown();
    server.join().expect("broker server");
}

#[test]
fn broker_decision_keys_have_a_fixed_cardinality_limit() {
    let broker = test_broker();
    for index in 0..300 {
        record_decision(
            &broker.state,
            "POST",
            &format!("/v1/messages/{index}"),
            "denied",
            "test",
            None,
        );
    }

    let decisions = broker.broker_decisions();
    assert!(decisions.len() <= MAX_BROKER_DECISION_KEYS);
    assert_eq!(
        decisions
            .iter()
            .map(|decision| decision.count)
            .sum::<usize>(),
        300
    );
    assert!(
        decisions
            .iter()
            .any(|decision| decision.reason == "decision-key-limit")
    );
}
