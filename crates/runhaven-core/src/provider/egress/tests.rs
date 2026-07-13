use std::io::{Cursor, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use super::*;

#[test]
fn policy_allows_exact_and_subdomain_hosts_only() {
    let policy = EgressPolicy::new(&["api.example.com".to_string()]).expect("policy");

    assert!(policy.allows("api.example.com", 443));
    assert!(policy.allows("chat.api.example.com", 443));
    assert!(!policy.allows("example.com", 443));
    assert!(!policy.allows("api.example.com", 80));
    assert!(!policy.allows("127.0.0.1", 443));
}

#[test]
fn policy_wildcard_family_matches_prefixes_within_one_domain() {
    let policy = EgressPolicy::new(&["*-cloudcode-pa.googleapis.com".to_string()]).expect("policy");

    // Any channel/region prefix of the family is allowed.
    assert!(policy.allows("daily-cloudcode-pa.googleapis.com", 443));
    assert!(policy.allows("us-cloudcode-pa.googleapis.com", 443));
    // The pattern stays inside googleapis.com: other Google services, the bare
    // host (no prefix), and a look-alike under another domain are all denied.
    assert!(!policy.allows("storage.googleapis.com", 443));
    assert!(!policy.allows("cloudcode-pa.googleapis.com", 443));
    assert!(!policy.allows("x-cloudcode-pa.googleapis.com.attacker.com", 443));
}

#[test]
fn wildcard_pattern_must_anchor_to_one_registrable_domain() {
    // A tail that is a single registrable domain would match other domains an
    // attacker could register (attacker-foo.com), so it is rejected.
    assert!(EgressPolicy::new(&["*-foo.com".to_string()]).is_err());
    assert!(EgressPolicy::new(&["*foo".to_string()]).is_err());
    // A properly anchored family pattern is accepted.
    assert!(EgressPolicy::new(&["*-cloudcode-pa.googleapis.com".to_string()]).is_ok());
}

#[test]
fn upstream_address_safety_rejects_private_and_documentation_ranges() {
    assert!(!is_safe_upstream_address(IpAddr::V4(Ipv4Addr::new(
        10, 0, 0, 1
    ))));
    assert!(!is_safe_upstream_address(IpAddr::V4(Ipv4Addr::new(
        192, 168, 1, 1
    ))));
    assert!(!is_safe_upstream_address(IpAddr::V4(Ipv4Addr::new(
        203, 0, 113, 10
    ))));
    assert!(!is_safe_upstream_address(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert!(is_safe_upstream_address(IpAddr::V4(Ipv4Addr::new(
        93, 184, 216, 34
    ))));
}

#[test]
fn parse_connect_target_normalizes_ipv6_brackets() {
    assert_eq!(
        parse_connect_target("[2001:db8::1]:443").expect("target"),
        ("2001:db8::1".to_string(), 443)
    );
}

#[test]
fn proxy_request_reader_does_not_consume_tunneled_bytes() {
    let mut request = Cursor::new(
        b"CONNECT example.com:443 HTTP/1.1\r\nHost: example.com:443\r\n\r\nTLS_CLIENT_HELLO",
    );

    assert_eq!(
        read_proxy_request_line(&mut request).expect("request line"),
        Some("CONNECT example.com:443 HTTP/1.1".to_string())
    );

    let mut remaining = Vec::new();
    request
        .read_to_end(&mut remaining)
        .expect("remaining bytes");
    assert_eq!(remaining, b"TLS_CLIENT_HELLO");
}

#[test]
fn relay_moves_bytes_in_both_directions() {
    let upstream_listener = TcpListener::bind(("127.0.0.1", 0)).expect("upstream listener");
    let upstream_address = upstream_listener.local_addr().expect("upstream address");
    let upstream_server = thread::spawn(move || {
        let (mut stream, _) = upstream_listener.accept().expect("upstream accept");
        let mut request = [0u8; 4];
        stream.read_exact(&mut request).expect("upstream read");
        assert_eq!(&request, b"ping");
        stream.write_all(b"pong").expect("upstream write");
        stream.shutdown(Shutdown::Write).expect("upstream shutdown");
    });

    let client_listener = TcpListener::bind(("127.0.0.1", 0)).expect("client listener");
    let client_address = client_listener.local_addr().expect("client address");
    let relay_thread = thread::spawn(move || {
        let (proxy_client, _) = client_listener.accept().expect("proxy accept");
        let upstream = TcpStream::connect(upstream_address).expect("upstream connect");
        proxy_client
            .set_nonblocking(true)
            .expect("proxy nonblocking");
        upstream
            .set_nonblocking(true)
            .expect("upstream nonblocking");
        relay(proxy_client, upstream, Duration::from_secs(2));
    });
    let mut client = TcpStream::connect(client_address).expect("client connect");

    client.write_all(b"ping").expect("client write");
    client.shutdown(Shutdown::Write).expect("client shutdown");
    let mut response = Vec::new();
    client.read_to_end(&mut response).expect("client read");
    assert_eq!(response, b"pong");
    relay_thread.join().expect("relay");
    upstream_server.join().expect("upstream server");
}

#[test]
fn proxy_and_broker_accept_loops_admit_before_spawning_workers() {
    for source in [
        include_str!("../egress.rs"),
        include_str!("../auth_broker.rs"),
    ] {
        let accept_loop = source
            .split("pub fn serve_forever")
            .nth(1)
            .expect("serve loop");
        let before_handler = accept_loop.split("handle_").next().expect("accept branch");
        assert!(
            before_handler.contains("try_acquire"),
            "client admission must happen before a worker thread is spawned"
        );
    }
}

#[test]
fn client_admission_releases_capacity_when_a_worker_finishes() {
    assert_eq!(MAX_CONCURRENT_EGRESS_CLIENTS, 32);
    let admission = ClientAdmission::new(2);
    let first = admission.try_acquire().expect("first client");
    let _second = admission.try_acquire().expect("second client");

    assert!(admission.try_acquire().is_none(), "excess client rejected");
    drop(first);
    assert!(
        admission.try_acquire().is_some(),
        "finished worker releases slot"
    );
}

#[test]
fn proxy_closes_an_excess_client_before_spawning_a_worker() {
    let mut proxy = ThreadedAllowlistProxy::bind(
        ("127.0.0.1", 0),
        EgressPolicy::new(&["example.com".to_string()]).expect("policy"),
        &[],
    )
    .expect("proxy");
    Arc::get_mut(&mut proxy.state)
        .expect("unshared proxy state")
        .client_admission = ClientAdmission::new(1);
    let server = {
        let proxy = proxy.clone();
        thread::spawn(move || proxy.serve_forever())
    };
    let address = proxy.server_addr().expect("proxy address");
    let first = TcpStream::connect(address).expect("first client");
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while proxy.state.client_admission.active() != 1 && std::time::Instant::now() < deadline {
        thread::yield_now();
    }
    assert_eq!(proxy.state.client_admission.active(), 1);
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

    assert!(closed, "excess proxy client must be closed");
    drop(first);
    proxy.shutdown();
    server.join().expect("proxy server");
}

#[test]
fn proxy_decision_keys_have_a_fixed_cardinality_limit() {
    let proxy = ThreadedAllowlistProxy::bind(
        ("127.0.0.1", 0),
        EgressPolicy::new(&["example.com".to_string()]).expect("policy"),
        &[],
    )
    .expect("proxy");
    for index in 0..300 {
        record_decision(
            &proxy.state,
            &format!("denied-{index}.example"),
            443,
            "denied",
            "test",
            "",
        );
    }

    let decisions = proxy.policy_decisions();
    assert!(decisions.len() <= MAX_PROXY_DECISION_KEYS);
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
