use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

mod headers;
mod http;
mod profiles;
mod upstream;

pub use headers::broker_request_headers;
pub use http::{broker_upstream_path, parse_content_length, sanitize_broker_request_path};
use http::{header_value, read_headers, send_error, send_upstream_response};
pub use profiles::{
    BROKER_PLACEHOLDER_VALUE, CLAUDE_BROKER, CLAUDE_OAUTH_BROKER, CODEX_BROKER,
    CredentialInjection, GEMINI_BROKER, GuestRedirect, PathRule, ProviderBrokerProfile,
    broker_profile_for_agent,
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
pub use upstream::{BrokerUpstream, BrokerUpstreamResponse, HttpsUpstream};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BrokerDecision {
    pub method: String,
    pub path: String,
    pub decision: String,
    pub reason: String,
    pub upstream_status: Option<u16>,
    pub count: usize,
}

#[derive(Clone)]
pub struct ApiKeyBrokerProxy {
    listener: Arc<TcpListener>,
    state: Arc<BrokerState>,
    shutdown: Arc<AtomicBool>,
}

struct BrokerState {
    // Held in memory for the run so injection can add it per request. Zeroizing
    // overwrites this copy on drop (Hardening 3) so it is not left in freed heap
    // after the proxy stops; the residual note in the design covers what this
    // cannot reach (pipe buffers, prior realloc, the child process).
    api_key: Zeroizing<String>,
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
        api_key: Zeroizing<String>,
        allowed_client_subnets: &[String],
    ) -> Result<Self> {
        let upstream = Arc::new(HttpsUpstream::for_host(profile.upstream_host));
        Self::bind_with_upstream(address, profile, api_key, allowed_client_subnets, upstream)
    }

    pub fn bind_with_upstream(
        address: (&str, u16),
        profile: ProviderBrokerProfile,
        api_key: Zeroizing<String>,
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
                // Already `Zeroizing` from the secret seam (Keychain read or host
                // env var), so it lands here without a plain-`String` hop.
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

#[cfg(test)]
mod tests;
