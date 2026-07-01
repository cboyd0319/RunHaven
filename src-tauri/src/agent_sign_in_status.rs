//! Sanitized status tracking for background agent sign-ins.
//!
//! Mirrors `ImageBuildStatusRegistry`: a bounded, in-memory map of sign-in
//! progress that the read-only `get_agent_sign_in_status` command surfaces to
//! the WebView. The agent's login output is redacted with the shared run-output
//! scrubber and bounded to a fixed number of tail lines so the sign-in URL and
//! short-lived device code stay visible while any token or key that leaks into
//! the stream is scrubbed, and a long or noisy login cannot grow without limit.
//!
//! The desktop app only ever triggers sign-in and watches this progress. The
//! resulting credential is owned by RunHaven core (unchanged `0600` path); no
//! token or host path is stored or returned here.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use runhaven_core::provider::observability::utc_timestamp;
use runhaven_core::support::redact::redact_run_output;
use runhaven_core::support::validators::validate_run_id;

use crate::contracts::AgentSignInStatusResponse;

const MAX_SIGN_IN_STATUSES: usize = 20;
const MAX_OUTPUT_TAIL_LINES: usize = 200;
const MAX_OUTPUT_LINE_LEN: usize = 500;

#[derive(Clone, Default)]
pub(crate) struct AgentSignInStatusRegistry {
    state: Arc<Mutex<SignInState>>,
}

#[derive(Default)]
struct SignInState {
    sign_ins: HashMap<String, SignInEntry>,
    order: VecDeque<String>,
}

struct SignInEntry {
    agent: String,
    state: String,
    label: String,
    detail: String,
    tail: VecDeque<String>,
    updated_at: String,
}

impl AgentSignInStatusRegistry {
    pub(crate) fn mark_starting(&self, sign_in_id: &str, agent: &str) -> Result<(), String> {
        validate_sign_in_id(sign_in_id)?;
        let mut state = self.lock()?;
        if !state.sign_ins.contains_key(sign_in_id) {
            state.order.push_back(sign_in_id.to_string());
        }
        state.sign_ins.insert(
            sign_in_id.to_string(),
            SignInEntry {
                agent: agent.to_string(),
                state: "starting".to_string(),
                label: "Starting".to_string(),
                detail: format!("RunHaven is starting sign-in for {agent}."),
                tail: VecDeque::new(),
                updated_at: utc_timestamp(),
            },
        );
        prune_sign_ins(&mut state);
        Ok(())
    }

    /// Append one line of login output. The line is scrubbed of terminal control
    /// sequences and well-known secret shapes, truncated on a char boundary, and
    /// the tail is capped so old lines fall off the front. The first line of
    /// output flips a still-starting sign-in to `needs_browser`, since that is
    /// when the login URL and device code appear.
    pub(crate) fn append_output(&self, sign_in_id: &str, line: &str) -> Result<(), String> {
        validate_sign_in_id(sign_in_id)?;
        let mut cleaned = redact_run_output(line.trim_end());
        truncate_on_char_boundary(&mut cleaned, MAX_OUTPUT_LINE_LEN);
        let mut state = self.lock()?;
        let Some(entry) = state.sign_ins.get_mut(sign_in_id) else {
            return Ok(());
        };
        if entry.state == "starting" {
            entry.state = "needs_browser".to_string();
            entry.label = "Open your browser".to_string();
            entry.detail =
                "Open the sign-in link shown below and finish in your browser. RunHaven is waiting."
                    .to_string();
        }
        entry.tail.push_back(cleaned);
        while entry.tail.len() > MAX_OUTPUT_TAIL_LINES {
            entry.tail.pop_front();
        }
        entry.updated_at = utc_timestamp();
        Ok(())
    }

    pub(crate) fn mark_succeeded(&self, sign_in_id: &str, agent: &str) -> Result<(), String> {
        self.set_terminal(
            sign_in_id,
            "succeeded",
            "Signed in",
            format!("You are signed in to {agent}."),
        )
    }

    pub(crate) fn mark_failed(&self, sign_in_id: &str, detail: String) -> Result<(), String> {
        self.set_terminal(sign_in_id, "failed", "Sign-in failed", detail)
    }

    pub(crate) fn get(&self, sign_in_id: &str) -> Result<AgentSignInStatusResponse, String> {
        validate_sign_in_id(sign_in_id)?;
        let state = self.lock()?;
        let entry = state
            .sign_ins
            .get(sign_in_id)
            .ok_or_else(|| "sign-in status not found".to_string())?;
        Ok(AgentSignInStatusResponse {
            sign_in_id: sign_in_id.to_string(),
            agent: entry.agent.clone(),
            state: entry.state.clone(),
            label: entry.label.clone(),
            detail: entry.detail.clone(),
            output_tail: entry.tail.iter().cloned().collect::<Vec<_>>().join("\n"),
            updated_at: entry.updated_at.clone(),
        })
    }

    fn set_terminal(
        &self,
        sign_in_id: &str,
        state_name: &str,
        label: &str,
        detail: String,
    ) -> Result<(), String> {
        validate_sign_in_id(sign_in_id)?;
        let mut state = self.lock()?;
        let Some(entry) = state.sign_ins.get_mut(sign_in_id) else {
            return Ok(());
        };
        entry.state = state_name.to_string();
        entry.label = label.to_string();
        entry.detail = strip_control_chars(&detail);
        entry.updated_at = utc_timestamp();
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, SignInState>, String> {
        self.state
            .lock()
            .map_err(|_| "sign-in status registry is unavailable".to_string())
    }
}

fn validate_sign_in_id(sign_in_id: &str) -> Result<(), String> {
    validate_run_id(sign_in_id).map_err(|error| error.to_string())
}

fn strip_control_chars(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

fn truncate_on_char_boundary(text: &mut String, max_len: usize) {
    if text.len() <= max_len {
        return;
    }
    let mut end = max_len;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
}

fn prune_sign_ins(state: &mut SignInState) {
    while state.sign_ins.len() > MAX_SIGN_IN_STATUSES {
        let Some(sign_in_id) = state.order.pop_front() else {
            return;
        };
        state.sign_ins.remove(&sign_in_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_in_registry_redacts_and_bounds_output_and_flags_browser_step() {
        let registry = AgentSignInStatusRegistry::default();
        registry
            .mark_starting("agent-sign-in-run", "codex")
            .expect("starting");
        assert_eq!(
            registry.get("agent-sign-in-run").expect("status").state,
            "starting"
        );

        registry
            .append_output(
                "agent-sign-in-run",
                "open https://example.com/device and enter ABCD-1234; token sk-ant-api03-abcdefghij_KLmnopqrstuvwxyz1234\u{1b}[0m",
            )
            .expect("append");

        let status = registry.get("agent-sign-in-run").expect("status");
        // First output flipped the state to needs_browser so the UI tells the
        // user to open the link.
        assert_eq!(status.state, "needs_browser");
        // The short-lived device code stays visible so the user can complete
        // sign-in, but a leaked token and control sequences are scrubbed.
        assert!(status.output_tail.contains("ABCD-1234"));
        assert!(!status.output_tail.contains("sk-ant-api03"));
        assert!(!status.output_tail.contains('\u{1b}'));

        // A long or noisy login cannot grow the tail without bound.
        for line in 0..(MAX_OUTPUT_TAIL_LINES + 50) {
            registry
                .append_output("agent-sign-in-run", &format!("waiting {line}"))
                .expect("append");
        }
        assert_eq!(
            registry
                .get("agent-sign-in-run")
                .expect("status")
                .output_tail
                .lines()
                .count(),
            MAX_OUTPUT_TAIL_LINES,
            "tail must stay bounded"
        );

        registry
            .mark_succeeded("agent-sign-in-run", "codex")
            .expect("done");
        let done = registry.get("agent-sign-in-run").expect("status");
        assert_eq!(done.state, "succeeded");
        assert!(done.detail.contains("signed in"));
    }

    #[test]
    fn sign_in_registry_reports_missing_sign_in() {
        let registry = AgentSignInStatusRegistry::default();
        let error = registry.get("missing-sign-in-run").expect_err("missing");
        assert!(error.contains("not found"));
    }

    #[test]
    fn sign_in_registry_prunes_old_entries() {
        let registry = AgentSignInStatusRegistry::default();
        for idx in 0..=MAX_SIGN_IN_STATUSES {
            let sign_in_id = format!("sign-in-prune-{idx}");
            registry
                .mark_starting(&sign_in_id, "codex")
                .expect("starting");
        }
        assert!(
            registry.get("sign-in-prune-0").is_err(),
            "oldest sign-in must be pruned"
        );
        assert!(
            registry
                .get(&format!("sign-in-prune-{MAX_SIGN_IN_STATUSES}"))
                .is_ok(),
            "newest sign-in must be retained"
        );
    }
}
