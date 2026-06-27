use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};

pub const MAX_HEADER_BYTES: usize = 64 * 1024;
pub const RELAY_BUFFER_BYTES: usize = 64 * 1024;
pub const MAX_DENIED_CONNECT_TARGETS: usize = 50;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProxyDecision {
    pub host: String,
    pub port: u16,
    pub decision: String,
    pub reason: String,
    pub matched_rule: String,
    pub count: usize,
}

#[derive(Clone, Debug)]
pub struct EgressPolicy {
    allowed_hosts: Vec<String>,
    allowed_ports: Vec<u16>,
}

impl EgressPolicy {
    pub fn new(hosts: &[String]) -> Result<Self> {
        Self::with_ports(hosts, &[443])
    }

    pub fn with_ports(hosts: &[String], ports: &[u16]) -> Result<Self> {
        if hosts.is_empty() {
            bail!("provider egress policy needs at least one allowed host");
        }
        if ports.contains(&0) {
            bail!("provider egress policy ports must be between 1 and 65535");
        }
        let mut allowed_hosts = Vec::with_capacity(hosts.len());
        for host in hosts {
            allowed_hosts.push(normalize_allowed_entry(host)?);
        }
        Ok(Self {
            allowed_hosts,
            allowed_ports: ports.to_vec(),
        })
    }

    pub fn allowed_hosts(&self) -> &[String] {
        &self.allowed_hosts
    }

    pub fn allows(&self, host: &str, port: u16) -> bool {
        self.match_rule(host, port).is_some()
    }

    pub fn match_rule(&self, host: &str, port: u16) -> Option<String> {
        let normalized = normalize_host(host).ok()?;
        if !self.allowed_ports.contains(&port) || is_ip_literal(&normalized) {
            return None;
        }
        self.allowed_hosts
            .iter()
            .find(|allowed| allowed_entry_matches(allowed, &normalized))
            .cloned()
    }

    pub fn denial_reason(&self, host: &str, port: u16) -> &'static str {
        let Ok(normalized) = normalize_host(host) else {
            return "invalid-host";
        };
        if !self.allowed_ports.contains(&port) {
            return "port-not-allowed";
        }
        if is_ip_literal(&normalized) {
            return "ip-literal";
        }
        "not-in-allowlist"
    }
}

/// Validate and normalize one allowlist entry. A leading `*` marks a wildcard
/// family pattern (for example `*-cloudcode-pa.googleapis.com`): it matches any
/// host ending in the literal text after the `*`. The pattern is anchored so the
/// wildcard can only expand a subdomain label inside one registrable domain: it
/// must start with `-` or `.` and the tail must carry a registrable domain (at
/// least two dots). That keeps `*-cloudcode-pa.googleapis.com` Google-only and
/// rejects cross-registrable-domain patterns like `*-foo.com`. Wildcard patterns
/// are maintainer-curated for the bundled allowlist, never required from users.
fn normalize_allowed_entry(entry: &str) -> Result<String> {
    let Some(suffix) = entry.strip_prefix('*') else {
        return normalize_host(entry);
    };
    if !(suffix.starts_with('-') || suffix.starts_with('.')) || suffix.matches('.').count() < 2 {
        bail!(
            "invalid wildcard host pattern {entry:?}: use *-name.domain.tld or *.domain.tld so it stays inside one registrable domain"
        );
    }
    // The domain tail must itself be a valid host.
    normalize_host(suffix.trim_start_matches(['-', '.']))?;
    Ok(entry.to_ascii_lowercase())
}

/// Whether a normalized host matches one allowlist entry. Plain entries match
/// the host itself or any subdomain; `*`-prefixed entries match any host ending
/// in the literal suffix after the `*`.
fn allowed_entry_matches(allowed: &str, normalized: &str) -> bool {
    match allowed.strip_prefix('*') {
        Some(suffix) => normalized.ends_with(suffix),
        None => normalized == allowed || normalized.ends_with(&format!(".{allowed}")),
    }
}

#[derive(Debug)]
struct ProxyState {
    policy: EgressPolicy,
    connect_timeout: Duration,
    relay_timeout: Duration,
    allowed_client_networks: Vec<IpNet>,
    denied_connect_targets: Mutex<Vec<(String, u16)>>,
    policy_decisions: Mutex<BTreeMap<ProxyDecisionKey, usize>>,
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
    *decisions.entry(key).or_insert(0) += 1;
}

pub fn normalize_host(host: &str) -> Result<String> {
    let mut value = host.trim().to_ascii_lowercase();
    while value.ends_with('.') {
        value.pop();
    }
    if value.starts_with('[') && value.ends_with(']') {
        value = value[1..value.len() - 1].to_string();
    }
    if value.is_empty()
        || value.chars().any(char::is_whitespace)
        || value.contains('/')
        || value.contains('\\')
    {
        bail!("invalid host: {host:?}");
    }
    if let Ok(address) = value.parse::<IpAddr>() {
        return Ok(address.to_string());
    }
    if value.contains(':') {
        bail!("invalid host: {host:?}");
    }
    let ascii_host =
        idna::domain_to_ascii(&value).map_err(|_| anyhow::anyhow!("invalid host: {host:?}"))?;
    if ascii_host.is_empty() || ascii_host.starts_with('-') || ascii_host.ends_with('-') {
        bail!("invalid host: {host:?}");
    }
    Ok(ascii_host)
}

pub fn is_ip_literal(host: &str) -> bool {
    host.parse::<IpAddr>().is_ok()
}

#[derive(Debug)]
enum SafeAddrError {
    Resolve,
    Unsafe,
}

fn safe_addrinfos(host: &str, port: u16) -> Result<Vec<SocketAddr>, SafeAddrError> {
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|_| SafeAddrError::Resolve)?
        .collect::<Vec<_>>();
    if addrs.is_empty() {
        return Err(SafeAddrError::Resolve);
    }
    if addrs
        .iter()
        .any(|addr| !is_safe_upstream_address(addr.ip()))
    {
        return Err(SafeAddrError::Unsafe);
    }
    Ok(addrs)
}

pub fn is_safe_upstream_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            let [a, b, c, d] = address.octets();
            !(a == 0
                || a == 10
                || a == 127
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192 && b == 168)
                || (a == 192 && b == 0 && c == 0)
                || (a == 192 && b == 0 && c == 2)
                || (a == 198 && (b == 18 || b == 19))
                || (a == 198 && b == 51 && c == 100)
                || (a == 203 && b == 0 && c == 113)
                || a >= 224
                || [a, b, c, d] == [255, 255, 255, 255])
        }
        IpAddr::V6(address) => address.to_ipv4_mapped().map_or_else(
            || {
                let segments = address.segments();
                !(address.is_unspecified()
                    || address.is_loopback()
                    || (segments[0] & 0xff00) == 0xff00
                    || (segments[0] & 0xfe00) == 0xfc00
                    || (segments[0] & 0xffc0) == 0xfe80
                    || (segments[0] == 0x2001 && segments[1] == 0x0db8))
            },
            |mapped| is_safe_upstream_address(IpAddr::V4(mapped)),
        ),
    }
}

fn connect_any(addrs: &[SocketAddr], timeout: Duration) -> std::io::Result<TcpStream> {
    let mut last_error = None;
    for addr in addrs {
        match TcpStream::connect_timeout(addr, timeout) {
            Ok(stream) => return Ok(stream),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| std::io::Error::other("no upstream addresses to connect")))
}

pub fn parse_connect_target(target: &str) -> Result<(String, u16)> {
    let (host, port_text) = if target.starts_with('[') {
        let Some(index) = target.rfind("]:") else {
            bail!("invalid CONNECT target: {target:?}");
        };
        (&target[..=index], &target[index + 2..])
    } else {
        let Some(index) = target.rfind(':') else {
            bail!("invalid CONNECT target: {target:?}");
        };
        (&target[..index], &target[index + 1..])
    };
    let port = port_text
        .parse::<u16>()
        .map_err(|_| anyhow::anyhow!("invalid CONNECT target port: {target:?}"))?;
    if port == 0 {
        bail!("invalid CONNECT target port: {target:?}");
    }
    Ok((normalize_host(host)?, port))
}

fn relay(client: TcpStream, upstream: TcpStream, timeout: Duration) {
    let _ = client.set_nonblocking(false);
    let _ = upstream.set_nonblocking(false);
    let _ = client.set_read_timeout(Some(timeout));
    let _ = client.set_write_timeout(Some(timeout));
    let _ = upstream.set_read_timeout(Some(timeout));
    let _ = upstream.set_write_timeout(Some(timeout));

    let mut client_read = match client.try_clone() {
        Ok(stream) => stream,
        Err(_) => return,
    };
    let mut upstream_write = match upstream.try_clone() {
        Ok(stream) => stream,
        Err(_) => return,
    };
    let mut upstream_read = upstream;
    let mut client_write = client;

    let client_to_upstream = thread::spawn(move || {
        let _ = std::io::copy(&mut client_read, &mut upstream_write);
        let _ = upstream_write.shutdown(Shutdown::Write);
    });
    let upstream_to_client = thread::spawn(move || {
        let _ = std::io::copy(&mut upstream_read, &mut client_write);
        let _ = client_write.shutdown(Shutdown::Write);
    });

    let _ = client_to_upstream.join();
    let _ = upstream_to_client.join();
}

#[cfg(test)]
mod tests;
