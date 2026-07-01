use std::net::IpAddr;

use anyhow::{Result, bail};

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
/// family pattern, for example `*-cloudcode-pa.googleapis.com`.
fn normalize_allowed_entry(entry: &str) -> Result<String> {
    let Some(suffix) = entry.strip_prefix('*') else {
        return normalize_host(entry);
    };
    if !(suffix.starts_with('-') || suffix.starts_with('.')) || suffix.matches('.').count() < 2 {
        bail!(
            "invalid wildcard host pattern {entry:?}: use *-name.domain.tld or *.domain.tld so it stays inside one registrable domain"
        );
    }
    normalize_host(suffix.trim_start_matches(['-', '.']))?;
    Ok(entry.to_ascii_lowercase())
}

fn allowed_entry_matches(allowed: &str, normalized: &str) -> bool {
    match allowed.strip_prefix('*') {
        Some(suffix) => normalized.ends_with(suffix),
        None => normalized == allowed || normalized.ends_with(&format!(".{allowed}")),
    }
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
