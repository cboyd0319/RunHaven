use std::time::Duration;

use anyhow::{Context, Result};

use super::headers;
use super::{CODEX_BROKER_REQUEST_TIMEOUT_SECONDS, CODEX_BROKER_UPSTREAM_HOST};

#[derive(Clone, Debug)]
pub struct BrokerUpstreamResponse {
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
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
    pub(super) fn for_host(host: &str) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_upstream_uses_global_request_timeout() {
        let upstream = HttpsUpstream::default();
        assert_eq!(
            upstream.agent.config().timeouts().global,
            Some(Duration::from_secs(CODEX_BROKER_REQUEST_TIMEOUT_SECONDS))
        );
    }
}
