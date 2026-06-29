use std::io::Write;

use anyhow::Result;
use serde_json::json;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::provider::auth_broker::{BrokerDecision, sanitize_broker_request_path};
use crate::provider::egress::ProxyDecision;
use crate::runtime::plans::AgentRunPlan;
use crate::support::paths::{auth_broker_log_path, egress_policy_log_path, open_private_append};

pub fn utc_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub fn write_provider_policy_log(
    plan: &AgentRunPlan,
    decisions: &[ProxyDecision],
    run_id: &str,
) -> Result<()> {
    if decisions.is_empty() {
        return Ok(());
    }
    let path = egress_policy_log_path();
    let timestamp = utc_timestamp();
    let mut file = open_private_append(&path)?;
    for decision in decisions {
        let payload = json!({
            "timestamp": timestamp,
            "run_id": run_id,
            "profile": plan.profile_name,
            "workspace": plan.workspace.display().to_string(),
            "network": plan.network_mode.as_str(),
            "host": decision.host,
            "port": decision.port,
            "decision": decision.decision,
            "reason": decision.reason,
            "matched_rule": decision.matched_rule,
            "count": decision.count,
        });
        writeln!(file, "{}", serde_json::to_string(&payload)?)?;
    }
    Ok(())
}

pub fn write_auth_broker_log(
    plan: &AgentRunPlan,
    decisions: &[BrokerDecision],
    run_id: &str,
    return_code: i32,
) -> Result<()> {
    let path = auth_broker_log_path();
    let timestamp = utc_timestamp();
    let mut file = open_private_append(&path)?;
    if decisions.is_empty() {
        let payload = auth_payload(plan, run_id, &timestamp, None, return_code);
        writeln!(file, "{}", serde_json::to_string(&payload)?)?;
        return Ok(());
    }
    for decision in decisions {
        let payload = auth_payload(plan, run_id, &timestamp, Some(decision), return_code);
        writeln!(file, "{}", serde_json::to_string(&payload)?)?;
    }
    Ok(())
}

fn auth_payload(
    plan: &AgentRunPlan,
    run_id: &str,
    timestamp: &str,
    decision: Option<&BrokerDecision>,
    return_code: i32,
) -> serde_json::Value {
    match decision {
        Some(decision) => json!({
            "timestamp": timestamp,
            "run_id": run_id,
            "profile": plan.profile_name,
            "workspace": plan.workspace.display().to_string(),
            "network": plan.network_mode.as_str(),
            "broker": "codex-api-key",
            "method": decision.method,
            "path": sanitize_broker_request_path(&decision.path),
            "decision": decision.decision,
            "reason": decision.reason,
            "upstream_status": decision.upstream_status,
            "count": decision.count,
            "return_code": return_code,
        }),
        None => json!({
            "timestamp": timestamp,
            "run_id": run_id,
            "profile": plan.profile_name,
            "workspace": plan.workspace.display().to_string(),
            "network": plan.network_mode.as_str(),
            "broker": "codex-api-key",
            "method": "-",
            "path": "-",
            "decision": "no-requests",
            "reason": "run-complete",
            "upstream_status": null,
            "count": 0,
            "return_code": return_code,
        }),
    }
}

pub fn print_provider_blocked_host_review(
    plan: &AgentRunPlan,
    decisions: &[ProxyDecision],
    _run_id: &str,
) {
    let denials = decisions
        .iter()
        .filter(|decision| decision.decision == "denied")
        .collect::<Vec<_>>();
    if denials.is_empty() {
        return;
    }
    // A calm, plain-language notice. RunHaven is primarily for less-technical
    // people, so the default does not dump hostnames or a per-host "review"; the
    // full per-host detail stays available in `runhaven egress log`.
    let total = denials.iter().map(|decision| decision.count).sum::<usize>();
    let destinations = denials.len();
    let attempts = if total == 1 { "attempt" } else { "attempts" };
    let places = if destinations == 1 {
        "destination"
    } else {
        "destinations"
    };
    let agent = &plan.profile_name;
    eprintln!(
        "RunHaven kept {agent} inside its provider's network and blocked {destinations} other {places} ({total} {attempts}) to protect your data."
    );
    eprintln!(
        "If {agent} seemed to miss something, run `runhaven egress log` to see what was blocked."
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::plans::{
        AuthScope, NetworkMode, RunOptions, WorkspaceScope, build_run_plan,
    };
    use crate::runtime::profiles::get_profile;
    use crate::support::paths::{auth_broker_log_path, override_cache_root_for_tests};

    #[test]
    fn auth_broker_writer_strips_query_and_fragment_from_paths() {
        let cache = tempfile::tempdir().expect("cache");
        let workspace = tempfile::tempdir().expect("workspace");
        let _cache_home = override_cache_root_for_tests(cache.path());
        let plan = build_run_plan(RunOptions {
            profile: get_profile("codex").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: Vec::new(),
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Provider,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Agent,
            read_only_workspace: false,
            ssh: false,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: true,
            tty: true,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: Vec::new(),
            api_key_broker_env: None,
            worktree: None,
            run_id: None,
        })
        .expect("run plan");
        let decision = BrokerDecision {
            method: "POST".to_string(),
            path: "/v1/responses?token=secret#fragment".to_string(),
            decision: "allowed".to_string(),
            reason: "-".to_string(),
            upstream_status: Some(200),
            count: 1,
        };

        write_auth_broker_log(&plan, &[decision], "run-123", 0).expect("write auth log");

        let raw = std::fs::read_to_string(auth_broker_log_path()).expect("read auth log");
        assert!(raw.contains("/v1/responses"));
        assert!(!raw.contains("token=secret"));
        assert!(!raw.contains("#fragment"));
    }
}
