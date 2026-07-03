mod network;
mod selection;

use super::broker_secret::require_api_key_broker_secret_with_sources;
use super::decision_log::{DecisionCounts, provider_decision_deltas};
use super::*;
use anyhow::anyhow;
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroizing;

use crate::provider::egress::ProxyDecision;
use crate::runtime::launch::{AgentCommandContext, AgentCommandRunner};
use crate::support::paths::override_cache_root_for_tests;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

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
    let broker = create_api_key_broker(
        crate::provider::auth_broker::CODEX_BROKER,
        SecretString::from("test-key"),
        &info,
    )
    .expect("wildcard broker");
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

fn broker_plan(profile_name: &str, agent: &str) -> AgentRunPlan {
    let image = "runhaven/base:0.1.0";
    AgentRunPlan {
        command: vec![
            "container".to_string(),
            "run".to_string(),
            image.to_string(),
            agent.to_string(),
        ],
        preflight: Vec::new(),
        workspace: std::path::PathBuf::from("/tmp/ws"),
        state_volume: "vol".to_string(),
        session: "default".to_string(),
        container_name: "runhaven-test-run".to_string(),
        profile_name: profile_name.to_string(),
        workspace_scope: crate::runtime::plans::WorkspaceScope::Current,
        workspace_scope_note: None,
        auth_scope: crate::runtime::plans::AuthScope::Agent,
        worktree: None,
        run_id: None,
        network_name: Some("runhaven-test-internal".to_string()),
        network_mode: crate::runtime::plans::NetworkMode::Provider,
        egress_summary: String::new(),
        image: image.to_string(),
        provider_allowed_hosts: vec!["example.com".to_string()],
        claude_auth_source: Some(crate::runtime::plans::ClaudeAuthSource::ApiKeyBrokerEnv(
            "PROVIDER_KEY".to_string(),
        )),
        security_notices: Vec::new(),
    }
}

use crate::runtime::plans::ClaudeAuthSource;

/// Keychain reader stub for the env-var broker tests: the API-key path MUST NOT
/// read the Keychain, so calling this is a test failure.
fn keychain_never() -> Result<Option<Zeroizing<Vec<u8>>>> {
    panic!("the API-key broker path must never read the Keychain");
}

#[test]
fn api_key_broker_secret_requires_fixed_host_env() {
    let mut plan = broker_plan("codex", "codex");
    plan.claude_auth_source = Some(ClaudeAuthSource::ApiKeyBrokerEnv(
        "OPENAI_API_KEY".to_string(),
    ));

    let secret = require_api_key_broker_secret_with_sources(
        &plan,
        |name| (name == "OPENAI_API_KEY").then(|| "test-openai-key".to_string()),
        keychain_never,
    )
    .expect("broker secret")
    .expect("broker enabled");

    assert_eq!(secret.0.host_env, "OPENAI_API_KEY");
    // The env key is carried in a `SecretString`; compare through the one
    // `expose_secret()` accessor.
    assert_eq!(secret.1.expose_secret(), "test-openai-key");

    plan.claude_auth_source = Some(ClaudeAuthSource::ApiKeyBrokerEnv(
        "AWS_SECRET_ACCESS_KEY".to_string(),
    ));
    let wrong_env = require_api_key_broker_secret_with_sources(
        &plan,
        |_| Some("must-not-be-read".to_string()),
        keychain_never,
    )
    .expect_err("unrelated env rejected");
    assert!(wrong_env.to_string().contains("OPENAI_API_KEY"));

    plan.claude_auth_source = Some(ClaudeAuthSource::ApiKeyBrokerEnv(
        "OPENAI_API_KEY".to_string(),
    ));
    let missing = require_api_key_broker_secret_with_sources(&plan, |_| None, keychain_never)
        .expect_err("missing fixed env rejected");
    assert!(missing.to_string().contains("OPENAI_API_KEY"));
    assert!(missing.to_string().contains("not set on the host"));
}

#[test]
fn keychain_oauth_source_reads_token_from_the_keychain_seam_not_env() {
    // The Claude OAuth run selects the Bearer OAuth profile and takes its secret
    // from the injected Keychain reader, never from an env var.
    let mut plan = broker_plan("claude", "claude");
    plan.claude_auth_source = Some(ClaudeAuthSource::KeychainOAuth);

    let (profile, token) = require_api_key_broker_secret_with_sources(
        &plan,
        |_| panic!("the Keychain OAuth path must never read an env var"),
        || {
            Ok(Some(Zeroizing::new(
                b"  sk-ant-oat01-FROM-KEYCHAIN  ".to_vec(),
            )))
        },
    )
    .expect("keychain secret")
    .expect("broker enabled");

    assert_eq!(
        profile,
        crate::provider::auth_broker::CLAUDE_OAUTH_BROKER,
        "the OAuth source must select the Bearer OAuth broker, not the x-api-key broker"
    );
    assert_eq!(
        profile.injection.injected_header_names(),
        vec!["authorization"]
    );
    // The token stays a `SecretString` end to end; compare through the one
    // `expose_secret()` accessor.
    assert_eq!(
        token.expose_secret(),
        "sk-ant-oat01-FROM-KEYCHAIN",
        "the token is trimmed"
    );
}

#[test]
fn keychain_oauth_source_fails_closed_on_read_error_or_missing_item() {
    let mut plan = broker_plan("claude", "claude");
    plan.claude_auth_source = Some(ClaudeAuthSource::KeychainOAuth);

    // Read failure / user cancel: the read seam returns an error and the run
    // fails closed with a fixed, path-free message.
    let read_error = require_api_key_broker_secret_with_sources(
        &plan,
        |_| None,
        || Err(anyhow!("could not read the Claude token from the Keychain")),
    )
    .expect_err("read error must fail the run");
    assert!(
        read_error
            .to_string()
            .contains("could not read the Claude token")
    );

    // No stored item: fail closed with a plain sign-in prompt.
    let missing = require_api_key_broker_secret_with_sources(&plan, |_| None, || Ok(None))
        .expect_err("missing item must fail the run");
    assert!(missing.to_string().contains("no Claude token is saved"));

    // Empty stored token: fail closed.
    let empty = require_api_key_broker_secret_with_sources(
        &plan,
        |_| None,
        || Ok(Some(Zeroizing::new(b"   ".to_vec()))),
    )
    .expect_err("empty token must fail the run");
    assert!(empty.to_string().contains("empty"));
}

#[test]
fn keychain_oauth_source_is_claude_only() {
    // A non-Claude profile cannot use the Keychain OAuth broker.
    let mut plan = broker_plan("codex", "codex");
    plan.claude_auth_source = Some(ClaudeAuthSource::KeychainOAuth);
    let error = require_api_key_broker_secret_with_sources(
        &plan,
        |_| None,
        || Ok(Some(Zeroizing::new(b"sk-ant-oat01-x".to_vec()))),
    )
    .expect_err("non-claude oauth rejected");
    assert!(
        error
            .to_string()
            .contains("only available for the claude profile")
    );
}

#[test]
fn claude_broker_redirects_base_url_and_gives_guest_only_placeholder() {
    let plan = broker_plan("claude", "claude");
    let result = with_api_key_broker_config(
        &plan.command,
        &plan,
        crate::provider::auth_broker::CLAUDE_BROKER,
        "http://192.0.2.1:8080",
    )
    .expect("claude broker config");
    let joined = result.join(" ");
    assert!(joined.contains("ANTHROPIC_BASE_URL=http://192.0.2.1:8080"));
    assert!(joined.contains("ANTHROPIC_API_KEY=runhaven-broker-placeholder"));
    // The host key env var name never enters the guest command; the broker reads
    // it host-side and the guest sees only the placeholder.
    assert!(!joined.contains("PROVIDER_KEY"));
}

#[test]
fn gemini_broker_redirects_base_url() {
    let plan = broker_plan("gemini", "gemini");
    let result = with_api_key_broker_config(
        &plan.command,
        &plan,
        crate::provider::auth_broker::GEMINI_BROKER,
        "http://192.0.2.1:8080",
    )
    .expect("gemini broker config");
    let joined = result.join(" ");
    assert!(joined.contains("GOOGLE_GEMINI_BASE_URL=http://192.0.2.1:8080"));
    assert!(joined.contains("GEMINI_API_KEY=runhaven-broker-placeholder"));
}

#[test]
fn codex_broker_injects_custom_provider_with_v1_base() {
    let plan = broker_plan("codex", "codex");
    let result = with_api_key_broker_config(
        &plan.command,
        &plan,
        crate::provider::auth_broker::CODEX_BROKER,
        "http://192.0.2.1:8080",
    )
    .expect("codex broker config");
    let joined = result.join(" ");
    assert!(joined.contains("base_url=\"http://192.0.2.1:8080/v1\""));
    assert!(joined.contains("RUNHAVEN_CODEX_BROKER_TOKEN=runhaven-broker-placeholder"));
    assert!(joined.contains("wire_api=\"responses\""));
}

#[test]
fn provider_decision_deltas_only_emit_new_counts() {
    let mut seen = DecisionCounts::new();
    let first = vec![ProxyDecision {
        host: "api.example.com".to_string(),
        port: 443,
        decision: "allowed".to_string(),
        reason: "allowed".to_string(),
        matched_rule: "api.example.com".to_string(),
        count: 2,
    }];

    let deltas = provider_decision_deltas(&first, &mut seen);
    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].count, 2);

    assert!(provider_decision_deltas(&first, &mut seen).is_empty());

    let next = vec![ProxyDecision {
        count: 5,
        ..first[0].clone()
    }];
    let deltas = provider_decision_deltas(&next, &mut seen);
    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].count, 3);
}

struct RecordingProviderRunner {
    expected_run_id: String,
    command: Option<Vec<String>>,
    saw_active_marker: bool,
}

impl AgentCommandRunner for RecordingProviderRunner {
    fn run_agent_command(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        context: AgentCommandContext<'_>,
    ) -> Result<i32> {
        assert_eq!(context.run_id, self.expected_run_id);
        assert_eq!(
            context.plan.run_id.as_deref(),
            Some(self.expected_run_id.as_str())
        );
        assert!(env.is_none());
        self.command = Some(command.to_vec());
        self.saw_active_marker =
            crate::runtime::active::find_active_run_record(context.run_id).is_ok();
        Ok(0)
    }
}

#[test]
fn provider_agent_command_can_use_backend_owned_runner() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = broker_plan("codex", "codex");
    plan.workspace = workspace.path().to_path_buf();
    plan.run_id = Some("provider-runner-seam".to_string());
    let command = with_provider_proxy_environment(&plan, "http://192.0.2.1:8080", &["192.0.2.1"]);
    assert_ne!(command, plan.command);
    let mut runner = RecordingProviderRunner {
        expected_run_id: "provider-runner-seam".to_string(),
        command: None,
        saw_active_marker: false,
    };

    let execution = run_provider_agent_command_with_runner(
        &plan,
        "provider-runner-seam",
        command.clone(),
        &mut runner,
    )
    .expect("provider command runner");

    assert_eq!(execution.return_code, 0);
    assert_eq!(runner.command.as_deref(), Some(command.as_slice()));
    assert!(runner.saw_active_marker);
    assert!(crate::runtime::active::find_active_run_record("provider-runner-seam").is_err());
}
