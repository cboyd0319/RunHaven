use super::*;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

const NETWORK_INSPECT_HOSTONLY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/apple_container/network-inspect-hostonly.json"
));

#[test]
fn parses_current_apple_network_inspect_shape() {
    let info =
        parse_internal_network_info("runhaven-volume-prep-internal", NETWORK_INSPECT_HOSTONLY)
            .expect("network inspect");

    assert_eq!(info.ipv4_gateway, "192.168.130.1");
    assert_eq!(info.ipv4_subnet, "192.168.130.0/24");
}

#[test]
fn rejects_non_host_only_network_inspect_shape() {
    let error = parse_internal_network_info(
        "runhaven-default",
        br#"[{"configuration":{"mode":"nat"},"status":{"ipv4Gateway":"192.168.64.1","ipv4Subnet":"192.168.64.0/24"}}]"#,
    )
    .expect_err("nat network");

    assert!(error.to_string().contains("not host-only"));
}

#[test]
fn rejects_network_inspect_missing_ipv4_fields() {
    let error = parse_internal_network_info(
        "runhaven-missing-ipv4",
        br#"[{"configuration":{"mode":"hostOnly"},"status":{}}]"#,
    )
    .expect_err("missing ipv4");

    assert!(error.to_string().contains("missing IPv4 gateway or subnet"));
}

fn connect_address(proxy_addr: SocketAddr) -> SocketAddr {
    if proxy_addr.ip().is_unspecified() {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), proxy_addr.port())
    } else {
        proxy_addr
    }
}

fn read_status_line(stream: &mut TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout");
    let mut response = Vec::new();
    let mut buffer = [0; 64];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                response.extend_from_slice(&buffer[..read]);
                if response.windows(2).any(|window| window == b"\r\n") {
                    break;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => break,
            Err(error) => panic!("read response: {error}"),
        }
    }
    String::from_utf8(response).expect("utf8 response")
}

#[test]
fn provider_proxy_wildcard_bind_rejects_clients_outside_container_subnet() {
    let info = InternalNetworkInfo {
        ipv4_gateway: "192.0.2.1".to_string(),
        ipv4_subnet: "192.0.2.0/24".to_string(),
    };
    let proxy = create_provider_proxy(
        EgressPolicy::new(&["example.com".to_string()]).unwrap(),
        &info,
    )
    .expect("wildcard proxy");
    assert!(proxy.server_addr().unwrap().ip().is_unspecified());
    let proxy_thread = {
        let proxy = proxy.clone();
        thread::spawn(move || proxy.serve_forever())
    };

    let mut stream = TcpStream::connect(connect_address(proxy.server_addr().unwrap()))
        .expect("connect to proxy");
    stream
        .write_all(b"CONNECT example.com:443 HTTP/1.1\r\nHost: example.com:443\r\n\r\n")
        .expect("proxy request");
    let response = read_status_line(&mut stream);
    assert!(
        response.starts_with("HTTP/1.1 403 Forbidden"),
        "unexpected proxy response: {response:?}"
    );

    proxy.shutdown();
    proxy_thread.join().expect("proxy thread");
}

#[test]
fn codex_broker_wildcard_bind_rejects_clients_outside_container_subnet() {
    let info = InternalNetworkInfo {
        ipv4_gateway: "192.0.2.1".to_string(),
        ipv4_subnet: "192.0.2.0/24".to_string(),
    };
    let broker =
        create_codex_api_key_broker("test-key".to_string(), &info).expect("wildcard broker");
    assert!(broker.server_addr().unwrap().ip().is_unspecified());
    let broker_thread = {
        let broker = broker.clone();
        thread::spawn(move || broker.serve_forever())
    };

    let mut stream = TcpStream::connect(connect_address(broker.server_addr().unwrap()))
        .expect("connect to broker");
    stream
        .write_all(b"POST /v1/responses HTTP/1.1\r\nHost: broker\r\nContent-Length: 2\r\n\r\n{}")
        .expect("broker request");
    let response = read_status_line(&mut stream);
    assert!(
        response.starts_with("HTTP/1.1 403 Forbidden"),
        "unexpected broker response: {response:?}"
    );

    broker.shutdown();
    broker_thread.join().expect("broker thread");
}
