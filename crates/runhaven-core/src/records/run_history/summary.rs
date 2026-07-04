//! One compact, greppable "what happened" log line per persisted run record.
//!
//! Kept beside the JSONL writer but in its own module so `run_history.rs` stays
//! within its file-size ceiling. The line carries only the non-secret fields the
//! record already persists, and materializes wherever a `log` sink is installed
//! (the desktop app's runhaven.log). In the CLI, with no logger, the `log`
//! facade drops it. Prefixed with `run <id>:` so a single `grep <run_id>`
//! returns a run's whole lifecycle.

use crate::provider::auth_broker::BrokerDecision;
use crate::provider::egress::ProxyDecision;

use super::RunRecordInput;

/// Emit the run summary at INFO. `status` is the already-resolved record status
/// so the log and the JSONL record never disagree.
pub(super) fn log_run_summary(input: &RunRecordInput<'_>, status: &str) {
    let provider_allowed = allowed_count(input.provider_decisions.iter().map(decision_pair));
    let provider_denied = denied_count(input.provider_decisions.iter().map(decision_pair));
    let (auth_allowed, auth_denied) = match input.auth_decisions {
        Some(decisions) => (
            allowed_count(decisions.iter().map(broker_pair)),
            denied_count(decisions.iter().map(broker_pair)),
        ),
        None => (0, 0),
    };
    log::info!(
        "run {run_id}: summary profile={profile} status={status} return_code={code} \
         duration={duration} broker={broker} \
         provider(allowed={provider_allowed},denied={provider_denied}) \
         auth(allowed={auth_allowed},denied={auth_denied})",
        run_id = input.run_id,
        profile = input.plan.profile_name,
        code = input.return_code,
        duration = run_duration_label(input.started_at, input.finished_at),
        broker = input.plan.broker_label(),
    );
}

fn decision_pair(decision: &ProxyDecision) -> (&str, usize) {
    (decision.decision.as_str(), decision.count)
}

fn broker_pair(decision: &BrokerDecision) -> (&str, usize) {
    (decision.decision.as_str(), decision.count)
}

fn allowed_count<'a>(pairs: impl Iterator<Item = (&'a str, usize)>) -> usize {
    pairs
        .filter(|(decision, _)| *decision == "allowed")
        .map(|(_, count)| count)
        .sum()
}

fn denied_count<'a>(pairs: impl Iterator<Item = (&'a str, usize)>) -> usize {
    pairs
        .filter(|(decision, _)| *decision == "denied")
        .map(|(_, count)| count)
        .sum()
}

/// Whole-second run duration from the RFC 3339 record timestamps, or `unknown`
/// when either is missing or unparseable. Reuses the crate's existing `time`
/// dependency; no wall clock is read here.
fn run_duration_label(started_at: &str, finished_at: &str) -> String {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    match (
        OffsetDateTime::parse(started_at, &Rfc3339),
        OffsetDateTime::parse(finished_at, &Rfc3339),
    ) {
        (Ok(started), Ok(finished)) if finished >= started => {
            format!("{}s", (finished - started).whole_seconds())
        }
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_duration_label_reports_seconds_or_unknown() {
        assert_eq!(
            run_duration_label("2026-07-01T00:00:00Z", "2026-07-01T00:00:15Z"),
            "15s"
        );
        // Missing/unparseable timestamps and reversed order fall back to unknown.
        assert_eq!(run_duration_label("", "2026-07-01T00:00:15Z"), "unknown");
        assert_eq!(
            run_duration_label("2026-07-01T00:00:15Z", "2026-07-01T00:00:00Z"),
            "unknown"
        );
    }
}
