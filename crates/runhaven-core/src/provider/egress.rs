use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};

mod policy;
mod tunnel;

pub use policy::{EgressPolicy, is_ip_literal, normalize_host};
use tunnel::{SafeAddrError, connect_any, relay, safe_addrinfos};
pub use tunnel::{is_safe_upstream_address, parse_connect_target};

pub const MAX_HEADER_BYTES: usize = 64 * 1024;
pub const RELAY_BUFFER_BYTES: usize = 64 * 1024;
pub const MAX_DENIED_CONNECT_TARGETS: usize = 50;
pub const MAX_CONCURRENT_EGRESS_CLIENTS: usize = 32;
pub const MAX_PROXY_DECISION_KEYS: usize = 256;

#[derive(Debug)]
pub(crate) struct ClientAdmission {
    active: AtomicUsize,
    limit: usize,
}

impl ClientAdmission {
    pub(crate) fn new(limit: usize) -> Arc<Self> {
        Arc::new(Self {
            active: AtomicUsize::new(0),
            limit,
        })
    }

    pub(crate) fn try_acquire(self: &Arc<Self>) -> Option<ClientPermit> {
        self.active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < self.limit).then_some(active + 1)
            })
            .ok()?;
        Some(ClientPermit {
            admission: Arc::clone(self),
        })
    }

    #[cfg(test)]
    pub(crate) fn active(&self) -> usize {
        self.active.load(Ordering::Acquire)
    }
}

pub(crate) struct ClientPermit {
    admission: Arc<ClientAdmission>,
}

impl Drop for ClientPermit {
    fn drop(&mut self) {
        self.admission.active.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProxyDecision {
    pub host: String,
    pub port: u16,
    pub decision: String,
    pub reason: String,
    pub matched_rule: String,
    pub count: usize,
}

#[derive(Debug)]
struct ProxyState {
    policy: EgressPolicy,
    connect_timeout: Duration,
    relay_timeout: Duration,
    allowed_client_networks: Vec<IpNet>,
    denied_connect_targets: Mutex<Vec<(String, u16)>>,
    policy_decisions: Mutex<BTreeMap<ProxyDecisionKey, usize>>,
    client_admission: Arc<ClientAdmission>,
}

type ProxyDecisionKey = (String, u16, String, String, String);

#[derive(Clone, Debug)]
pub struct ThreadedAllowlistProxy {
    listener: Arc<TcpListener>,
    state: Arc<ProxyState>,
    shutdown: Arc<AtomicBool>,
}

impl ThreadedAllowlistProxy {
    pub fn bind(
        server_address: (&str, u16),
        policy: EgressPolicy,
        allowed_client_subnets: &[String],
    ) -> Result<Self> {
        let listener = TcpListener::bind(server_address)?;
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
            state: Arc::new(ProxyState {
                policy,
                connect_timeout: Duration::from_secs(10),
                relay_timeout: Duration::from_secs(30),
                allowed_client_networks: networks,
                denied_connect_targets: Mutex::new(Vec::new()),
                policy_decisions: Mutex::new(BTreeMap::new()),
                client_admission: ClientAdmission::new(MAX_CONCURRENT_EGRESS_CLIENTS),
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
                    let Some(permit) = self.state.client_admission.try_acquire() else {
                        drop(stream);
                        continue;
                    };
                    let state = Arc::clone(&self.state);
                    let _ = thread::Builder::new()
                        .name("runhaven-egress-client".to_string())
                        .spawn(move || {
                            let _permit = permit;
                            let _ = handle_proxy_client(stream, peer, state);
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

    pub fn policy_decisions(&self) -> Vec<ProxyDecision> {
        self.state
            .policy_decisions
            .lock()
            .expect("policy lock")
            .iter()
            .map(
                |((host, port, decision, reason, matched_rule), count)| ProxyDecision {
                    host: host.clone(),
                    port: *port,
                    decision: decision.clone(),
                    reason: reason.clone(),
                    matched_rule: matched_rule.clone(),
                    count: *count,
                },
            )
            .collect()
    }

    pub fn denied_connect_targets(&self) -> Vec<(String, u16)> {
        self.state
            .denied_connect_targets
            .lock()
            .expect("denied lock")
            .clone()
    }
}

fn handle_proxy_client(
    mut stream: TcpStream,
    peer: SocketAddr,
    state: Arc<ProxyState>,
) -> Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(state.connect_timeout))?;
    stream.set_write_timeout(Some(state.connect_timeout))?;
    if !allows_client(&state, peer.ip()) {
        send_proxy_response(&mut stream, 403, "Forbidden")?;
        return Ok(());
    }

    let Some(request_line) = read_proxy_request_line(&mut stream)? else {
        send_proxy_response(&mut stream, 400, "Bad Request")?;
        return Ok(());
    };
    let parts = request_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 {
        send_proxy_response(&mut stream, 400, "Bad Request")?;
        return Ok(());
    }
    if !parts[0].eq_ignore_ascii_case("CONNECT") {
        send_proxy_response(&mut stream, 405, "Method Not Allowed")?;
        return Ok(());
    }
    let Ok((host, port)) = parse_connect_target(parts[1]) else {
        send_proxy_response(&mut stream, 400, "Bad Request")?;
        return Ok(());
    };
    let Some(matched_rule) = state.policy.match_rule(&host, port) else {
        let reason = state.policy.denial_reason(&host, port).to_string();
        record_decision(&state, &host, port, "denied", &reason, "");
        record_denied_target(&state, &host, port);
        send_proxy_response(&mut stream, 403, "Forbidden")?;
        return Ok(());
    };

    let addrinfos = match safe_addrinfos(&host, port) {
        Ok(value) => value,
        Err(SafeAddrError::Unsafe) => {
            record_decision(
                &state,
                &host,
                port,
                "denied",
                "unsafe-resolved-address",
                &matched_rule,
            );
            send_proxy_response(&mut stream, 403, "Forbidden")?;
            return Ok(());
        }
        Err(SafeAddrError::Resolve) => {
            record_decision(
                &state,
                &host,
                port,
                "denied",
                "dns-resolution-failed",
                &matched_rule,
            );
            send_proxy_response(&mut stream, 502, "Bad Gateway")?;
            return Ok(());
        }
    };
    let upstream = match connect_any(&addrinfos, state.connect_timeout) {
        Ok(stream) => stream,
        Err(_) => {
            record_decision(&state, &host, port, "allowed", "allowed", &matched_rule);
            send_proxy_response(&mut stream, 502, "Bad Gateway")?;
            return Ok(());
        }
    };
    record_decision(&state, &host, port, "allowed", "allowed", &matched_rule);
    stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
    stream.flush()?;
    relay(stream, upstream, state.relay_timeout);
    Ok(())
}

fn allows_client(state: &ProxyState, address: IpAddr) -> bool {
    state.allowed_client_networks.is_empty()
        || state
            .allowed_client_networks
            .iter()
            .any(|network| network.contains(&address))
}

fn read_proxy_request_line(reader: &mut impl Read) -> Result<Option<String>> {
    let mut header = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        if header.len() >= MAX_HEADER_BYTES {
            return Ok(None);
        }
        let read = reader.read(&mut byte)?;
        if read == 0 {
            return Ok(None);
        }
        header.push(byte[0]);
        if header.ends_with(b"\r\n\r\n") || header.ends_with(b"\n\n") {
            let request = std::str::from_utf8(&header).context("invalid proxy request header")?;
            return Ok(request.lines().next().map(str::to_string));
        }
    }
}

fn send_proxy_response(stream: &mut TcpStream, status: u16, reason: &str) -> Result<()> {
    let body = format!("{status} {reason}\n");
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn record_denied_target(state: &ProxyState, host: &str, port: u16) {
    let mut targets = state.denied_connect_targets.lock().expect("denied lock");
    let target = (host.to_string(), port);
    if targets.len() < MAX_DENIED_CONNECT_TARGETS && !targets.contains(&target) {
        targets.push(target);
    }
}

fn record_decision(
    state: &ProxyState,
    host: &str,
    port: u16,
    decision: &str,
    reason: &str,
    matched_rule: &str,
) {
    let mut decisions = state.policy_decisions.lock().expect("policy lock");
    let key = (
        host.to_string(),
        port,
        decision.to_string(),
        reason.to_string(),
        matched_rule.to_string(),
    );
    if let Some(count) = decisions.get_mut(&key) {
        *count = count.saturating_add(1);
        return;
    }
    if decisions.len() < MAX_PROXY_DECISION_KEYS.saturating_sub(2) {
        decisions.insert(key, 1);
        return;
    }
    let overflow_decision = if decision == "allowed" {
        "allowed"
    } else {
        "denied"
    };
    let overflow = (
        "<other>".to_string(),
        0,
        overflow_decision.to_string(),
        "decision-key-limit".to_string(),
        String::new(),
    );
    let count = decisions.entry(overflow).or_insert(0);
    *count = count.saturating_add(1);
}

#[cfg(test)]
mod tests;
