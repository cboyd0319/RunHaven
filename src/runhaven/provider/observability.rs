use std::io::Write;

use anyhow::Result;
use serde_json::json;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::auth_broker::BrokerDecision;
use crate::egress::{ProxyDecision, is_ip_literal};
use crate::paths::{auth_broker_log_path, egress_policy_log_path, open_private_append};
use crate::plans::AgentRunPlan;
use crate::provider_endpoints::match_provider_endpoints;

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
            "path": decision.path,
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
    run_id: &str,
) {
    let denials = decisions
        .iter()
        .filter(|decision| decision.decision == "denied")
        .collect::<Vec<_>>();
    if denials.is_empty() {
        return;
    }
    let total = denials.iter().map(|decision| decision.count).sum::<usize>();
    let plural = if total == 1 { "request" } else { "requests" };
    eprintln!(
        "RunHaven provider proxy blocked {total} CONNECT {plural} across {} target(s).",
        denials.len()
    );
    eprintln!("Run id: {run_id}");
    eprintln!("Review:");
    for decision in denials {
        let matched_rule = if decision.matched_rule.is_empty() {
            "-"
        } else {
            &decision.matched_rule
        };
        eprintln!(
            "  - {}:{}  count={}  reason={}  rule={}",
            decision.host, decision.port, decision.count, decision.reason, matched_rule
        );
        eprintln!(
            "    Next action: {}",
            provider_denial_next_action(plan, decision)
        );
    }
    eprintln!("Recent policy log: runhaven egress log --limit 20");
}

pub fn provider_denial_next_action(plan: &AgentRunPlan, decision: &ProxyDecision) -> String {
    let host = &decision.host;
    if is_ip_literal(host) {
        return "IP literal targets cannot be allowed; use a reviewed provider hostname."
            .to_string();
    }
    match decision.reason.as_str() {
        "port-not-allowed" => {
            return "provider mode only allows HTTPS CONNECT on port 443.".to_string();
        }
        "unsafe-resolved-address" => {
            return "do not add an override; the allowed host resolved to a non-public address."
                .to_string();
        }
        "dns-resolution-failed" => {
            return "check DNS or provider availability before changing the allowlist.".to_string();
        }
        _ => {}
    }
    let explanation = format!("runhaven why host {host} --agent {}", plan.profile_name);
    if !match_provider_endpoints(host, Some(&plan.profile_name)).is_empty() {
        format!(
            "{explanation}; rerun with --provider-host {host} only if the documented purpose matches."
        )
    } else {
        format!("{explanation}; add --provider-host {host} only after source review.")
    }
}
