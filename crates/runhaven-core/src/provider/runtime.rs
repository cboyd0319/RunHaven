use std::collections::BTreeMap;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::provider::auth_broker::{
    ApiKeyBrokerProxy, GuestRedirect, ProviderBrokerProfile, broker_profile_for_agent,
};
use crate::provider::egress::{EgressPolicy, ProxyDecision, ThreadedAllowlistProxy};
use crate::provider::observability::{
    print_provider_blocked_host_review, utc_timestamp, write_auth_broker_log,
    write_provider_policy_log,
};
use crate::records::{RunRecordInput, write_run_record};
use crate::runtime::active::{
    active_run_terminal_status, remove_active_run_record, write_active_run_record,
};
use crate::runtime::plans::AgentRunPlan;
use crate::support::git::{capture_git_snapshot, summarize_git_change};

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
    let broker_secret = require_api_key_broker_secret(plan)?;
    let mut provider_network_created = false;
    let mut proxy: Option<ThreadedAllowlistProxy> = None;
    let mut proxy_thread = None;
    let mut broker: Option<ApiKeyBrokerProxy> = None;
    let mut broker_thread = None;
    let mut decision_log_stream = None;
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
        decision_log_stream = Some(start_provider_decision_log_stream(
            plan,
            &run_id,
            provider_proxy.clone(),
        ));
        proxy = Some(provider_proxy);

        let command = if let Some((broker_profile, api_key)) = broker_secret {
            let key_broker = create_api_key_broker(broker_profile, api_key, &network_info)?;
            // Bare base URL; the guest-config step appends any provider-specific
            // path segment (Codex expects a /v1 suffix; Claude and Gemini append
            // the full path themselves).
            let broker_base_url = format!(
                "http://{}:{}",
                network_info.ipv4_gateway,
                key_broker.server_addr()?.port()
            );
            let broker_clone = key_broker.clone();
            broker_thread = Some(thread::spawn(move || broker_clone.serve_forever()));
            broker = Some(key_broker);
            let command = with_provider_proxy_environment(
                plan,
                &proxy_url,
                &[network_info.ipv4_gateway.as_str()],
            );
            with_api_key_broker_config(&command, plan, broker_profile, &broker_base_url)?
        } else {
            with_provider_proxy_environment(plan, &proxy_url, &[])
        };

        let before = capture_git_snapshot(&plan.workspace);
        let started = utc_timestamp();
        eprintln!("Run id: {run_id}");
        write_active_run_record(plan, &run_id, &started)?;
        active_recorded = true;
        let token_injection = crate::runtime::login::run_token_injection(plan);
        let command = match &token_injection {
            Some((env, _)) => crate::runtime::login::with_token_env(&command, &plan.image, env),
            None => command,
        };
        let mut agent_command = Command::new(&command[0]);
        agent_command.args(&command[1..]);
        if let Some((env, value)) = &token_injection {
            eprintln!(
                "Using your stored {} login (injected into the sandbox env).",
                plan.profile_name
            );
            agent_command.env(env, value);
        }
        let status = agent_command.status();
        terminal_status = active_run_terminal_status(&run_id);
        let finished = utc_timestamp();
        git = Some(summarize_git_change(
            before,
            capture_git_snapshot(&plan.workspace),
        ));
        started_at = Some(started);
        finished_at = Some(finished);
        match status {
            Ok(status) => {
                let code = status.code().unwrap_or(1);
                return_code = Some(code);
                Ok(code)
            }
            Err(error) => {
                return_code = Some(1);
                Err(anyhow::anyhow!(
                    "could not launch provider agent command {:?}: {error}",
                    command[0]
                ))
            }
        }
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
    let decision_log_result = decision_log_stream
        .map(ProviderDecisionLogStream::stop)
        .unwrap_or(Ok(()));
    if provider_network_created {
        cleanup = cleanup_provider_network(plan).unwrap_or_else(|error| {
            json!({
                "provider_network": "cleanup-error",
                "provider_network_name": network_name,
                "error": error.to_string(),
            })
        });
    }

    let provider_decisions = proxy
        .as_ref()
        .map(ThreadedAllowlistProxy::policy_decisions)
        .unwrap_or_default();
    let auth_decisions = broker.as_ref().map(ApiKeyBrokerProxy::broker_decisions);
    let record_result = if let Some(code) = return_code
        && let (Some(started), Some(finished), Some(git)) =
            (started_at.as_deref(), finished_at.as_deref(), git)
    {
        (|| -> Result<()> {
            decision_log_result?;
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
            })
        })()
    } else {
        decision_log_result
    };
    if active_recorded {
        let _ = remove_active_run_record(&run_id);
    }
    match (result, record_result) {
        (Ok(code), Ok(())) => Ok(code),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), _) => Err(error),
    }
}

type DecisionKey = (String, u16, String, String, String);
type DecisionCounts = BTreeMap<DecisionKey, usize>;

struct ProviderDecisionLogStream {
    shutdown: Arc<AtomicBool>,
    handle: thread::JoinHandle<Result<()>>,
}

impl ProviderDecisionLogStream {
    fn stop(self) -> Result<()> {
        self.shutdown.store(true, Ordering::Relaxed);
        self.handle
            .join()
            .map_err(|_| anyhow::anyhow!("provider decision log stream panicked"))?
    }
}

fn start_provider_decision_log_stream(
    plan: &AgentRunPlan,
    run_id: &str,
    proxy: ThreadedAllowlistProxy,
) -> ProviderDecisionLogStream {
    let plan = plan.clone();
    let run_id = run_id.to_string();
    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = Arc::clone(&shutdown);
    let handle = thread::spawn(move || {
        let mut seen = DecisionCounts::new();
        while !thread_shutdown.load(Ordering::Relaxed) {
            flush_provider_decision_deltas(&plan, &run_id, &proxy, &mut seen)?;
            thread::sleep(Duration::from_millis(250));
        }
        flush_provider_decision_deltas(&plan, &run_id, &proxy, &mut seen)
    });
    ProviderDecisionLogStream { shutdown, handle }
}

fn flush_provider_decision_deltas(
    plan: &AgentRunPlan,
    run_id: &str,
    proxy: &ThreadedAllowlistProxy,
    seen: &mut DecisionCounts,
) -> Result<()> {
    let deltas = provider_decision_deltas(&proxy.policy_decisions(), seen);
    write_provider_policy_log(plan, &deltas, run_id)
}

fn provider_decision_deltas(
    decisions: &[ProxyDecision],
    seen: &mut DecisionCounts,
) -> Vec<ProxyDecision> {
    decisions
        .iter()
        .filter_map(|decision| {
            let key = (
                decision.host.clone(),
                decision.port,
                decision.decision.clone(),
                decision.reason.clone(),
                decision.matched_rule.clone(),
            );
            let previous = seen.get(&key).copied().unwrap_or_default();
            if decision.count <= previous {
                return None;
            }
            seen.insert(key, decision.count);
            let mut delta = decision.clone();
            delta.count -= previous;
            Some(delta)
        })
        .collect()
}

pub fn require_api_key_broker_secret(
    plan: &AgentRunPlan,
) -> Result<Option<(ProviderBrokerProfile, String)>> {
    let Some(name) = &plan.api_key_broker_env else {
        return Ok(None);
    };
    let Some(profile) = broker_profile_for_agent(&plan.profile_name) else {
        bail!(
            "the {} profile has no API-key broker; remove --api-key-broker-env",
            plan.profile_name
        );
    };
    let value = std::env::var(name).unwrap_or_default();
    if value.trim().is_empty() {
        bail!("{name} is not set on the host; export it before using --api-key-broker-env");
    }
    Ok(Some((profile, value)))
}

pub fn validate_runtime_auth_broker_environment(plan: &AgentRunPlan) -> Result<()> {
    require_api_key_broker_secret(plan).map(|_| ())
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

pub fn with_api_key_broker_config(
    command: &[String],
    plan: &AgentRunPlan,
    profile: ProviderBrokerProfile,
    broker_base_url: &str,
) -> Result<Vec<String>> {
    let image_index = command
        .iter()
        .position(|arg| arg == &plan.image)
        .expect("image in command");
    // The guest always receives only the placeholder key value; the real key
    // stays host-side in the broker.
    let placeholder_env = format!("{}={}", profile.placeholder_env, profile.placeholder_value);
    match profile.guest_redirect {
        GuestRedirect::CodexCustomProvider {
            provider_id,
            wire_api,
        } => {
            if command.get(image_index + 1).map(String::as_str) != Some("codex") {
                bail!("the Codex API key broker requires the agent command to start with codex");
            }
            let broker_environment = ["--env".to_string(), placeholder_env];
            let mut command_with_env = command[..image_index].to_vec();
            command_with_env.extend(broker_environment.clone());
            command_with_env.extend(command[image_index..].to_vec());
            let codex_index = image_index + broker_environment.len() + 1;
            // Codex's base_url convention expects the API-version segment.
            let base_url = format!("{broker_base_url}/v1");
            let config = vec![
                "-c".to_string(),
                format!("model_provider=\"{provider_id}\""),
                "-c".to_string(),
                format!("model_providers.{provider_id}.name=\"{}\"", profile.label),
                "-c".to_string(),
                format!("model_providers.{provider_id}.base_url=\"{base_url}\""),
                "-c".to_string(),
                format!(
                    "model_providers.{provider_id}.env_key=\"{}\"",
                    profile.placeholder_env
                ),
                "-c".to_string(),
                format!("model_providers.{provider_id}.wire_api=\"{wire_api}\""),
            ];
            let mut result = command_with_env[..=codex_index].to_vec();
            result.extend(config);
            result.extend(command_with_env[codex_index + 1..].to_vec());
            Ok(result)
        }
        GuestRedirect::EnvRedirect { base_url_env } => {
            // Claude and Gemini honor a base-URL env var; point it at the broker
            // and hand the guest only the placeholder key.
            let injected = [
                "--env".to_string(),
                format!("{base_url_env}={broker_base_url}"),
                "--env".to_string(),
                placeholder_env,
            ];
            let mut result = command[..image_index].to_vec();
            result.extend(injected);
            result.extend(command[image_index..].to_vec());
            Ok(result)
        }
    }
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
    match ThreadedAllowlistProxy::bind((&network_info.ipv4_gateway, 0), policy.clone(), &subnets) {
        Ok(proxy) => Ok(proxy),
        Err(_) => {
            // The gateway IP is not bindable at proxy-bind time on Apple
            // container 1.0.0 (the vmnet interface only comes up once a container
            // attaches), so this 0.0.0.0 fallback is the expected path here, not
            // an error worth warning on each run. The off-subnet client filter
            // still rejects connections from outside the container subnet; see
            // docs/SECURITY_MODEL.md.
            ThreadedAllowlistProxy::bind(("0.0.0.0", 0), policy, &subnets).with_context(|| {
                format!(
                    "could not bind provider allowlist proxy for Apple container gateway {}",
                    network_info.ipv4_gateway
                )
            })
        }
    }
}

pub fn create_api_key_broker(
    profile: ProviderBrokerProfile,
    api_key: String,
    network_info: &InternalNetworkInfo,
) -> Result<ApiKeyBrokerProxy> {
    let subnets = vec![network_info.ipv4_subnet.clone()];
    match ApiKeyBrokerProxy::bind_for_profile(
        (&network_info.ipv4_gateway, 0),
        profile,
        api_key.clone(),
        &subnets,
    ) {
        Ok(broker) => Ok(broker),
        Err(_) => {
            // Same expected 0.0.0.0 fallback as the provider proxy (the gateway
            // is not bindable at bind time on this runtime); the off-subnet
            // client filter still applies. See docs/SECURITY_MODEL.md.
            ApiKeyBrokerProxy::bind_for_profile(("0.0.0.0", 0), profile, api_key, &subnets)
                .with_context(|| {
                    format!(
                        "could not bind {} for Apple container gateway {}",
                        profile.label, network_info.ipv4_gateway
                    )
                })
        }
    }
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
mod tests;
