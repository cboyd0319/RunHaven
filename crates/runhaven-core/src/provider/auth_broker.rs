use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};

mod headers;
mod profiles;

pub use headers::broker_request_headers;
pub use profiles::{
    BROKER_PLACEHOLDER_VALUE, CLAUDE_BROKER, CODEX_BROKER, CredentialInjection, GEMINI_BROKER,
    GuestRedirect, PathRule, ProviderBrokerProfile, broker_profile_for_agent,
};

pub use crate::provider::auth_profiles::{
    AUTH_BROKER_RUNTIME, AUTH_BROKER_STATUS, CODEX_API_KEY_BROKER_STATUS,
    CODEX_BROKER_PLACEHOLDER_ENV, DESIGN_ONLY_AUTH_BROKER_STATUS,
};

pub const CODEX_BROKER_PROVIDER_ID: &str = "runhaven_openai";
pub const CODEX_BROKER_PLACEHOLDER_VALUE: &str = "runhaven-broker-placeholder";
pub const CODEX_BROKER_UPSTREAM_HOST: &str = "api.openai.com";
pub const CODEX_BROKER_REQUEST_TIMEOUT_SECONDS: u64 = 120;
pub const MAX_CODEX_BROKER_REQUEST_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct BrokerUpstreamResponse {
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BrokerDecision {
    pub method: String,
    pub path: String,
    pub decision: String,
    pub reason: String,
    pub upstream_status: Option<u16>,
    pub count: usize,
}

pub trait BrokerUpstream: Send + Sync + 'static {
    fn send(
        &self,
        method: &str,
        path: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Result<BrokerUpstreamResponse>;
}

#[derive(Clone)]
pub struct HttpsUpstream {
    host: String,
    agent: ureq::Agent,
}

impl Default for HttpsUpstream {
    fn default() -> Self {
        Self::for_host(CODEX_BROKER_UPSTREAM_HOST)
    }
}

impl HttpsUpstream {
    /// HTTPS upstream pinned to a provider host. The sender is host-agnostic
    /// (`https://{host}{path}`); only the broker profile decides the host.
    fn for_host(host: &str) -> Self {
        Self {
            host: host.to_string(),
            agent: broker_upstream_agent(),
        }
    }
}

impl BrokerUpstream for HttpsUpstream {
    fn send(
        &self,
        _method: &str,
        path: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Result<BrokerUpstreamResponse> {
        let url = format!("https://{}{}", self.host, path);
        let mut request = self.agent.post(&url);
        for (name, value) in headers {
            request = request.header(name.as_str(), value.as_str());
        }
        let mut response = request
            .send(body)
            .context("broker upstream request failed")?;
        let status = response.status().as_u16();
        let reason = response
            .status()
            .canonical_reason()
            .unwrap_or("")
            .to_string();
        let headers = response
            .headers()
            .iter()
            .filter(|(name, _)| !headers::response_header_is_hop_by_hop(name.as_str()))
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| (name.to_string(), value.to_string()))
            })
            .collect();
        let body = response.body_mut().read_to_vec()?;
        Ok(BrokerUpstreamResponse {
            status,
            reason,
            headers,
            body,
        })
    }
}

fn broker_upstream_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(
            CODEX_BROKER_REQUEST_TIMEOUT_SECONDS,
        )))
        .build()
        .into()
}

#[derive(Clone)]
pub struct ApiKeyBrokerProxy {
    listener: Arc<TcpListener>,
    state: Arc<BrokerState>,
    shutdown: Arc<AtomicBool>,
}

struct BrokerState {
    api_key: String,
    profile: ProviderBrokerProfile,
    upstream: Arc<dyn BrokerUpstream>,
    allowed_client_networks: Vec<IpNet>,
    decisions: Mutex<BTreeMap<BrokerDecisionKey, usize>>,
}

type BrokerDecisionKey = (String, String, String, String, Option<u16>);

impl ApiKeyBrokerProxy {
    pub fn bind_for_profile(
        address: (&str, u16),
        profile: ProviderBrokerProfile,
        api_key: String,
        allowed_client_subnets: &[String],
    ) -> Result<Self> {
        let upstream = Arc::new(HttpsUpstream::for_host(profile.upstream_host));
        Self::bind_with_upstream(address, profile, api_key, allowed_client_subnets, upstream)
    }

    pub fn bind_with_upstream(
        address: (&str, u16),
        profile: ProviderBrokerProfile,
        api_key: String,
        allowed_client_subnets: &[String],
        upstream: Arc<dyn BrokerUpstream>,
    ) -> Result<Self> {
        if api_key.trim().is_empty() {
            bail!("{} requires a host API key", profile.label);
        }
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        let mut networks = Vec::new();
        for subnet in allowed_client_subnets {
            networks.push(
                IpNet::from_str(subnet)
                    .with_context(|| format!("invalid client subnet: {subnet}"))?,
            );
        }
        Ok(Self {
            listener: Arc::new(listener),
            state: Arc::new(BrokerState {
                api_key,
                profile,
                upstream,
                allowed_client_networks: networks,
                decisions: Mutex::new(BTreeMap::new()),
            }),
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn server_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    pub fn serve_forever(&self) {
        while !self.shutdown.load(Ordering::Relaxed) {
            match self.listener.accept() {
                Ok((stream, peer)) => {
                    let state = Arc::clone(&self.state);
                    thread::spawn(move || {
                        let _ = handle_broker_client(stream, peer, state);
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Ok(addr) = self.server_addr() {
            let _ = TcpStream::connect_timeout(&addr, Duration::from_millis(100));
        }
    }

    pub fn broker_decisions(&self) -> Vec<BrokerDecision> {
        self.state
            .decisions
            .lock()
            .expect("broker decision lock")
            .iter()
            .map(
                |((method, path, decision, reason, upstream_status), count)| BrokerDecision {
                    method: method.clone(),
                    path: path.clone(),
                    decision: decision.clone(),
                    reason: reason.clone(),
                    upstream_status: *upstream_status,
                    count: *count,
                },
            )
            .collect()
    }
}

fn handle_broker_client(
    mut stream: TcpStream,
    peer: SocketAddr,
    state: Arc<BrokerState>,
) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(
        CODEX_BROKER_REQUEST_TIMEOUT_SECONDS,
    )))?;
    stream.set_write_timeout(Some(Duration::from_secs(
        CODEX_BROKER_REQUEST_TIMEOUT_SECONDS,
    )))?;
    if !allows_client(&state, peer.ip()) {
        record_decision(
            &state,
            "POST",
            "<client-denied>",
            "denied",
            "client-not-allowed",
            None,
        );
        send_error(&mut stream, 403, "Forbidden")?;
        return Ok(());
    }

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        send_error(&mut stream, 400, "Bad Request")?;
        return Ok(());
    }
    let parts = request_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 {
        send_error(&mut stream, 400, "Bad Request")?;
        return Ok(());
    }
    let method = parts[0].to_ascii_uppercase();
    let target = parts[1];
    if method != "POST" {
        record_decision(
            &state,
            &method,
            "<unsupported>",
            "denied",
            "method-not-allowed",
            None,
        );
        send_error(&mut stream, 405, "Method Not Allowed")?;
        return Ok(());
    }
    let upstream_path = match broker_upstream_path(target, &state.profile.path_rule) {
        Ok(path) => path,
        Err(_) => {
            record_decision(
                &state,
                "POST",
                "<unsupported>",
                "denied",
                "unsupported-path",
                None,
            );
            send_error(&mut stream, 403, "Forbidden")?;
            return Ok(());
        }
    };
    let headers = read_headers(&mut reader)?;
    let length = match parse_content_length(header_value(&headers, "content-length")) {
        Ok(Some(length)) => length,
        Ok(None) => {
            record_decision(
                &state,
                "POST",
                &upstream_path,
                "denied",
                "length-required",
                None,
            );
            send_error(&mut stream, 411, "Length Required")?;
            return Ok(());
        }
        Err(_) => {
            record_decision(
                &state,
                "POST",
                &upstream_path,
                "denied",
                "bad-content-length",
                None,
            );
            send_error(&mut stream, 400, "Bad Request")?;
            return Ok(());
        }
    };
    if length > MAX_CODEX_BROKER_REQUEST_BYTES {
        record_decision(
            &state,
            "POST",
            &upstream_path,
            "denied",
            "payload-too-large",
            None,
        );
        send_error(&mut stream, 413, "Payload Too Large")?;
        return Ok(());
    }
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    let request_headers = broker_request_headers(
        &headers,
        state.profile.upstream_host,
        &state.api_key,
        &state.profile.injection,
        body.len(),
    );
    match state
        .upstream
        .send("POST", &upstream_path, &request_headers, &body)
    {
        Ok(response) => {
            record_decision(
                &state,
                "POST",
                &upstream_path,
                "allowed",
                "upstream-response",
                Some(response.status),
            );
            send_upstream_response(&mut stream, response)?;
        }
        Err(_) => {
            record_decision(
                &state,
                "POST",
                &upstream_path,
                "allowed",
                "upstream-error",
                None,
            );
            send_error(&mut stream, 502, "Bad Gateway")?;
        }
    }
    Ok(())
}

fn allows_client(state: &BrokerState, address: IpAddr) -> bool {
    state.allowed_client_networks.is_empty()
        || state
            .allowed_client_networks
            .iter()
            .any(|network| network.contains(&address))
}

fn read_headers(reader: &mut BufReader<TcpStream>) -> Result<Vec<(String, String)>> {
    let mut headers = Vec::new();
    let mut consumed = 0usize;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        consumed += read;
        if consumed > crate::provider::egress::MAX_HEADER_BYTES {
            bail!("headers too large");
        }
        if read == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    Ok(headers)
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn record_decision(
    state: &BrokerState,
    method: &str,
    path: &str,
    decision: &str,
    reason: &str,
    upstream_status: Option<u16>,
) {
    let mut decisions = state.decisions.lock().expect("broker decision lock");
    let key = (
        method.to_string(),
        path.to_string(),
        decision.to_string(),
        reason.to_string(),
        upstream_status,
    );
    *decisions.entry(key).or_insert(0) += 1;
}

fn send_error(stream: &mut TcpStream, status: u16, reason: &str) -> Result<()> {
    let body = format!("{status} {reason}\n");
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nConnection: close\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn send_upstream_response(stream: &mut TcpStream, response: BrokerUpstreamResponse) -> Result<()> {
    let reason = if response.reason.is_empty() {
        "OK"
    } else {
        &response.reason
    };
    write!(stream, "HTTP/1.1 {} {}\r\n", response.status, reason)?;
    let has_content_length = response
        .headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("content-length"));
    for (name, value) in response.headers {
        if !headers::response_header_is_hop_by_hop(&name) {
            write!(stream, "{name}: {value}\r\n")?;
        }
    }
    if !has_content_length {
        write!(stream, "Content-Length: {}\r\n", response.body.len())?;
    }
    stream.write_all(b"\r\n")?;
    stream.write_all(&response.body)?;
    Ok(())
}

pub fn broker_upstream_path(target: &str, path_rule: &PathRule) -> Result<String> {
    let (path, query) = target
        .split_once('?')
        .map_or((target, None), |(path, query)| (path, Some(query)));
    if !path_rule.allows(path) {
        bail!("broker does not allow request path {path}");
    }
    Ok(query.map_or_else(|| path.to_string(), |query| format!("{path}?{query}")))
}

pub fn sanitize_broker_request_path(path: &str) -> String {
    let path = path
        .split(['?', '#'])
        .next()
        .filter(|path| !path.is_empty())
        .unwrap_or("-");
    path.to_string()
}

pub fn parse_content_length(value: Option<&str>) -> Result<Option<usize>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let length = value
        .parse::<isize>()
        .map_err(|_| anyhow::anyhow!("invalid Content-Length"))?;
    if length < 0 {
        bail!("invalid Content-Length");
    }
    Ok(Some(length as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn openai_upstream_uses_global_request_timeout() {
        let upstream = HttpsUpstream::default();
        assert_eq!(
            upstream.agent.config().timeouts().global,
            Some(Duration::from_secs(CODEX_BROKER_REQUEST_TIMEOUT_SECONDS))
        );
    }
}
