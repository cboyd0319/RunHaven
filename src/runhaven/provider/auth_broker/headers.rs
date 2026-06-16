const HOP_BY_HOP_REQUEST_HEADERS: &[&str] = &[
    "authorization",
    "connection",
    "content-length",
    "host",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "proxy-connection",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

const HOP_BY_HOP_RESPONSE_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

pub fn broker_request_headers(
    headers: &[(String, String)],
    upstream_host: &str,
    api_key: &str,
    body_length: usize,
) -> Vec<(String, String)> {
    let mut forwarded = headers
        .iter()
        .filter(|(name, _)| !header_is_hop_by_hop(name, HOP_BY_HOP_REQUEST_HEADERS))
        .cloned()
        .collect::<Vec<_>>();
    forwarded.push(("Host".to_string(), upstream_host.to_string()));
    forwarded.push(("Authorization".to_string(), format!("Bearer {api_key}")));
    forwarded.push(("Content-Length".to_string(), body_length.to_string()));
    forwarded
}

pub(super) fn response_header_is_hop_by_hop(name: &str) -> bool {
    header_is_hop_by_hop(name, HOP_BY_HOP_RESPONSE_HEADERS)
}

fn header_is_hop_by_hop(name: &str, list: &[&str]) -> bool {
    list.iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}
