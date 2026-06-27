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
    injection: &super::profiles::CredentialInjection,
    body_length: usize,
) -> Vec<(String, String)> {
    use super::profiles::CredentialInjection;
    // Strip hop-by-hop headers and any guest-sent copy of the headers we inject
    // (e.g. a placeholder x-api-key), so the guest can never set or leak the
    // real credential headers.
    let injected = injection.injected_header_names();
    let mut forwarded = headers
        .iter()
        .filter(|(name, _)| {
            !header_is_hop_by_hop(name, HOP_BY_HOP_REQUEST_HEADERS)
                && !injected.iter().any(|n| name.eq_ignore_ascii_case(n))
        })
        .cloned()
        .collect::<Vec<_>>();
    forwarded.push(("Host".to_string(), upstream_host.to_string()));
    match injection {
        CredentialInjection::BearerAuth => {
            forwarded.push(("Authorization".to_string(), format!("Bearer {api_key}")));
        }
        CredentialInjection::ApiKeyHeader { name, extra } => {
            forwarded.push(((*name).to_string(), api_key.to_string()));
            for (key, value) in *extra {
                forwarded.push(((*key).to_string(), (*value).to_string()));
            }
        }
    }
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
