use std::process::Command;
use std::thread;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::active::{
    active_run_terminal_status, remove_active_run_record, write_active_run_record,
};
use crate::auth_broker::{
    CODEX_BROKER_PLACEHOLDER_ENV, CODEX_BROKER_PLACEHOLDER_VALUE, CODEX_BROKER_PROVIDER_ID,
    CodexApiKeyBrokerProxy,
};
use crate::egress::{EgressPolicy, ThreadedAllowlistProxy};
use crate::git::{capture_git_snapshot, summarize_git_change};
use crate::plans::AgentRunPlan;
use crate::provider_observability::{
    print_provider_blocked_host_review, utc_timestamp, write_auth_broker_log,
    write_provider_policy_log,
};
use crate::records::{RunRecordInput, write_run_record};

#[derive(Clone, Debug)]
pub struct InternalNetworkInfo {
    pub ipv4_gateway: String,
    pub ipv4_subnet: String,
}

pub fn run_provider_agent(plan: &AgentRunPlan) -> Result<i32> {
    let network_name = plan
        .network_name
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("provider network plan is missing an internal network"))?;
    if plan.provider_allowed_hosts.is_empty() {
        bail!("provider network plan is missing provider hosts");
    }
    let codex_api_key = require_codex_api_key_broker_secret(plan)?;
    let mut provider_network_created = false;
    let mut proxy: Option<ThreadedAllowlistProxy> = None;
    let mut proxy_thread = None;
    let mut broker: Option<CodexApiKeyBrokerProxy> = None;
    let mut broker_thread = None;
    let run_id = plan
        .run_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    let mut cleanup =
        json!({"provider_network": "not-created", "provider_network_name": network_name});
    let mut return_code = None;
    let mut started_at = None;
    let mut finished_at = None;
    let mut terminal_status = None;
    let mut git = None;
    let mut active_recorded = false;

    let result = (|| -> Result<i32> {
        for command in &plan.preflight {
            run_preflight(command)?;
            if command_starts_with(command, &["container", "network", "create", "--internal"])
                && command.last() == Some(network_name)
            {
                provider_network_created = true;
            }
        }
        let network_info = inspect_internal_network(network_name)?;
        let policy = EgressPolicy::new(&plan.provider_allowed_hosts)?;
        let provider_proxy = create_provider_proxy(policy, &network_info)?;
        let proxy_url = format!(
            "http://{}:{}",
            network_info.ipv4_gateway,
            provider_proxy.server_addr()?.port()
        );
        let proxy_clone = provider_proxy.clone();
        proxy_thread = Some(thread::spawn(move || proxy_clone.serve_forever()));
        proxy = Some(provider_proxy);

        let command = if let Some(api_key) = codex_api_key {
            let codex_broker = create_codex_api_key_broker(api_key, &network_info)?;
            let broker_url = format!(
                "http://{}:{}/v1",
                network_info.ipv4_gateway,
                codex_broker.server_addr()?.port()
            );
            let broker_clone = codex_broker.clone();
            broker_thread = Some(thread::spawn(move || broker_clone.serve_forever()));
            broker = Some(codex_broker);
            let command = with_provider_proxy_environment(
                plan,
                &proxy_url,
                &[network_info.ipv4_gateway.as_str()],
            );
            with_codex_api_key_broker_config(&command, plan, &broker_url)?
        } else {
            with_provider_proxy_environment(plan, &proxy_url, &[])
        };

        let before = capture_git_snapshot(&plan.workspace);
        let started = utc_timestamp();
        eprintln!("Run id: {run_id}");
        write_active_run_record(plan, &run_id, &started)?;
        active_recorded = true;
        let status = Command::new(&command[0]).args(&command[1..]).status()?;
        terminal_status = active_run_terminal_status(&run_id);
        let finished = utc_timestamp();
        git = Some(summarize_git_change(
            before,
            capture_git_snapshot(&plan.workspace),
        ));
        started_at = Some(started);
        finished_at = Some(finished);
        let code = status.code().unwrap_or(1);
        return_code = Some(code);
        Ok(code)
    })();

    if let Some(broker) = &broker {
        broker.shutdown();
    }
    if let Some(handle) = broker_thread {
        let _ = handle.join();
    }
    if let Some(proxy) = &proxy {
        proxy.shutdown();
    }
    if let Some(handle) = proxy_thread {
        let _ = handle.join();
    }
    if provider_network_created {
        cleanup = cleanup_provider_network(plan)?;
    }

    let provider_decisions = proxy
        .as_ref()
        .map(ThreadedAllowlistProxy::policy_decisions)
        .unwrap_or_default();
    let auth_decisions = broker
        .as_ref()
        .map(CodexApiKeyBrokerProxy::broker_decisions);
    if let Some(code) = return_code
        && let (Some(started), Some(finished), Some(git)) =
            (started_at.as_deref(), finished_at.as_deref(), git)
    {
        write_provider_policy_log(plan, &provider_decisions, &run_id)?;
        if let Some(decisions) = auth_decisions.as_ref() {
            write_auth_broker_log(plan, decisions, &run_id, code)?;
        }
        print_provider_blocked_host_review(plan, &provider_decisions, &run_id);
        write_run_record(RunRecordInput {
            plan,
            run_id: &run_id,
            started_at: started,
            finished_at: finished,
            return_code: code,
            status: terminal_status.as_deref(),
            provider_decisions: &provider_decisions,
            auth_decisions: auth_decisions.as_deref(),
            cleanup,
            git,
        })?;
    }
    if active_recorded {
        let _ = remove_active_run_record(&run_id);
    }
    result
}

pub fn require_codex_api_key_broker_secret(plan: &AgentRunPlan) -> Result<Option<String>> {
    let Some(name) = &plan.codex_api_key_broker_env else {
        return Ok(None);
    };
    let value = std::env::var(name).unwrap_or_default();
    if value.trim().is_empty() {
        bail!("{name} is not set on the host; export it before using --codex-api-key-broker-env");
    }
    Ok(Some(value))
}

pub fn validate_runtime_auth_broker_environment(plan: &AgentRunPlan) -> Result<()> {
    require_codex_api_key_broker_secret(plan).map(|_| ())
}

pub fn with_provider_proxy_environment(
    plan: &AgentRunPlan,
    proxy_url: &str,
    no_proxy_hosts: &[&str],
) -> Vec<String> {
    let image_index = plan
        .command
        .iter()
        .position(|arg| arg == &plan.image)
        .expect("image in command");
    let no_proxy = std::iter::once("localhost")
        .chain(["127.0.0.1", "::1"])
        .chain(no_proxy_hosts.iter().copied())
        .collect::<Vec<_>>()
        .join(",");
    let proxy_environment = [
        ("HTTPS_PROXY", proxy_url),
        ("HTTP_PROXY", proxy_url),
        ("ALL_PROXY", proxy_url),
        ("https_proxy", proxy_url),
        ("http_proxy", proxy_url),
        ("all_proxy", proxy_url),
        ("NO_PROXY", &no_proxy),
        ("no_proxy", &no_proxy),
    ];
    let mut injected = Vec::new();
    for (name, value) in proxy_environment {
        injected.extend(["--env".to_string(), format!("{name}={value}")]);
    }
    let mut command = plan.command[..image_index].to_vec();
    command.extend(injected);
    command.extend(plan.command[image_index..].to_vec());
    command
}

pub fn with_codex_api_key_broker_config(
    command: &[String],
    plan: &AgentRunPlan,
    broker_base_url: &str,
) -> Result<Vec<String>> {
    let image_index = command
        .iter()
        .position(|arg| arg == &plan.image)
        .expect("image in command");
    if command.get(image_index + 1).map(String::as_str) != Some("codex") {
        bail!("Codex API key broker requires the agent command to start with codex");
    }
    let broker_environment = [
        "--env".to_string(),
        format!("{CODEX_BROKER_PLACEHOLDER_ENV}={CODEX_BROKER_PLACEHOLDER_VALUE}"),
    ];
    let mut command_with_env = command[..image_index].to_vec();
    command_with_env.extend(broker_environment.clone());
    command_with_env.extend(command[image_index..].to_vec());
    let codex_index = image_index + broker_environment.len() + 1;
    let config = vec![
        "-c".to_string(),
        format!("model_provider=\"{CODEX_BROKER_PROVIDER_ID}\""),
        "-c".to_string(),
        format!(
            "model_providers.{CODEX_BROKER_PROVIDER_ID}.name=\"RunHaven OpenAI API-key broker\""
        ),
        "-c".to_string(),
        format!("model_providers.{CODEX_BROKER_PROVIDER_ID}.base_url=\"{broker_base_url}\""),
        "-c".to_string(),
        format!(
            "model_providers.{CODEX_BROKER_PROVIDER_ID}.env_key=\"{CODEX_BROKER_PLACEHOLDER_ENV}\""
        ),
        "-c".to_string(),
        format!("model_providers.{CODEX_BROKER_PROVIDER_ID}.wire_api=\"responses\""),
    ];
    let mut result = command_with_env[..=codex_index].to_vec();
    result.extend(config);
    result.extend(command_with_env[codex_index + 1..].to_vec());
    Ok(result)
}

pub fn cleanup_provider_network(plan: &AgentRunPlan) -> Result<Value> {
    let Some(name) = &plan.network_name else {
        return Ok(json!({"provider_network": "not-created", "provider_network_name": null}));
    };
    let code = delete_container_network(name)?;
    Ok(json!({
        "provider_network": if code == 0 { "deleted" } else { "delete-failed" },
        "provider_network_name": name,
        "delete_return_code": code,
    }))
}

pub fn create_provider_proxy(
    policy: EgressPolicy,
    network_info: &InternalNetworkInfo,
) -> Result<ThreadedAllowlistProxy> {
    let subnets = vec![network_info.ipv4_subnet.clone()];
    ThreadedAllowlistProxy::bind((&network_info.ipv4_gateway, 0), policy.clone(), &subnets)
        .or_else(|_| ThreadedAllowlistProxy::bind(("0.0.0.0", 0), policy, &subnets))
}

pub fn create_codex_api_key_broker(
    api_key: String,
    network_info: &InternalNetworkInfo,
) -> Result<CodexApiKeyBrokerProxy> {
    let subnets = vec![network_info.ipv4_subnet.clone()];
    CodexApiKeyBrokerProxy::bind((&network_info.ipv4_gateway, 0), api_key.clone(), &subnets)
        .or_else(|_| CodexApiKeyBrokerProxy::bind(("0.0.0.0", 0), api_key, &subnets))
}

pub fn inspect_internal_network(name: &str) -> Result<InternalNetworkInfo> {
    let output = Command::new("container")
        .args(["network", "inspect", name])
        .output()?;
    if !output.status.success() {
        bail!("container network inspect failed: {name}");
    }
    parse_internal_network_info(name, &output.stdout)
}

fn parse_internal_network_info(name: &str, stdout: &[u8]) -> Result<InternalNetworkInfo> {
    let payload: Value = serde_json::from_slice(stdout)
        .with_context(|| format!("could not inspect provider network {name:?}"))?;
    let item = payload
        .as_array()
        .and_then(|items| items.first())
        .ok_or_else(|| anyhow::anyhow!("could not inspect provider network {name:?}"))?;
    if item.pointer("/configuration/mode").and_then(Value::as_str) != Some("hostOnly") {
        bail!("provider network {name:?} is not host-only");
    }
    let gateway = item
        .pointer("/status/ipv4Gateway")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!("provider network {name:?} is missing IPv4 gateway or subnet")
        })?;
    let subnet = item
        .pointer("/status/ipv4Subnet")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!("provider network {name:?} is missing IPv4 gateway or subnet")
        })?;
    Ok(InternalNetworkInfo {
        ipv4_gateway: gateway.to_string(),
        ipv4_subnet: subnet.to_string(),
    })
}

pub fn delete_container_network(name: &str) -> Result<i32> {
    Ok(Command::new("container")
        .args(["network", "delete", name])
        .status()?
        .code()
        .unwrap_or(1))
}

pub fn ensure_internal_network(name: &str) -> Result<()> {
    let existing = Command::new("container")
        .args(["network", "inspect", name])
        .output()?;
    if existing.status.success() {
        let mode = inspect_network_mode(&String::from_utf8_lossy(&existing.stdout));
        if mode.as_deref() == Some("hostOnly") {
            return Ok(());
        }
        bail!(
            "existing container network {name:?} is {}, not host-only",
            mode.unwrap_or_else(|| "unknown".to_string())
        );
    }
    let status = Command::new("container")
        .args(["network", "create", "--internal", name])
        .status()?;
    if !status.success() {
        bail!("container network create failed: {name}");
    }
    Ok(())
}

pub fn inspect_network_mode(output: &str) -> Option<String> {
    let payload = serde_json::from_str::<Value>(output).ok()?;
    payload
        .as_array()?
        .first()?
        .pointer("/configuration/mode")?
        .as_str()
        .map(str::to_string)
}

pub fn run_preflight(command: &[String]) -> Result<()> {
    if command_starts_with(command, &["container", "network", "create", "--internal"])
        && let Some(name) = command.last()
    {
        return ensure_internal_network(name);
    }
    let status = Command::new(&command[0]).args(&command[1..]).status()?;
    if !status.success() {
        bail!("preflight command failed: {status}");
    }
    Ok(())
}

fn command_starts_with(command: &[String], prefix: &[&str]) -> bool {
    command.len() >= prefix.len()
        && command
            .iter()
            .zip(prefix.iter())
            .all(|(left, right)| left == right)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
