//! Per-agent sign-in tracking that RunHaven can truthfully know.
//!
//! A marker records that an agent completed its sign-in flow INSIDE ITS
//! ISOLATED SANDBOX and that RunHaven observed that flow exit successfully.
//! RunHaven never reads the credential: the marker is secret-free and holds only
//! `{agent, scope, signed_in_at}`. It never carries a token, key, env value, or
//! path.
//!
//! The marker means "the agent signed in in its sandbox", NOT "signed in via
//! RunHaven" and NOT "the token is still valid". RunHaven does not store or
//! manage the credential; it only witnessed the sign-in flow finish. Readiness
//! labels describe the agent's state only.

use std::io::Write;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::provider::auth_profiles::{AgentSignIn, agent_sign_in_mode};
use crate::provider::observability::utc_timestamp;
use crate::records::read_jsonl;
use crate::support::paths::{agent_sign_ins_path, open_private_append};
use crate::support::validators::validate_run_id;

/// The shared-agent sign-in scope: a device-code sign-in signs the agent in for
/// its shared agent state, not a per-workspace scope.
pub const AGENT_SIGN_IN_SCOPE: &str = "agent";

/// A secret-free record that an agent completed sign-in in its sandbox. The
/// three fields are the ONLY thing ever written; a token, key, env value, or
/// path in this struct would be a hard security failure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentSignInMarker {
    pub agent: String,
    pub scope: String,
    pub signed_in_at: String,
}

/// The per-agent readiness the Home "Agent setup" panel shows. Every field is a
/// plain, non-secret string describing the AGENT's state, never RunHaven's.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReadiness {
    pub name: String,
    /// One of "ready", "signed-in", or "needs-sign-in".
    pub category: String,
    pub label: String,
}

/// Append one secret-free sign-in marker for `agent` at `scope`.
///
/// The agent id and scope are validated with the same id validator the run log
/// uses, so a control or terminal-escape byte cannot ride into the record and
/// back out into a rendered surface. Only `{agent, scope, signed_in_at}` is
/// written; the timestamp comes from the shared UTC helper, never a forbidden
/// wall-clock call.
pub fn record_agent_sign_in(agent: &str, scope: &str) -> Result<()> {
    validate_run_id(agent)?;
    validate_run_id(scope)?;
    let marker = AgentSignInMarker {
        agent: agent.to_string(),
        scope: scope.to_string(),
        signed_in_at: utc_timestamp(),
    };
    let path = agent_sign_ins_path();
    let mut file = open_private_append(&path)?;
    writeln!(file, "{}", serde_json::to_string(&marker)?)?;
    Ok(())
}

/// Read every sign-in marker. Malformed lines are skipped, matching the run
/// log's tolerant read. Cheap: a single local file read, no container,
/// Keychain, or subprocess.
pub fn agent_sign_in_markers() -> Result<Vec<AgentSignInMarker>> {
    let records = read_jsonl(&agent_sign_ins_path(), 0)?;
    Ok(records
        .into_iter()
        .filter_map(|value| serde_json::from_value(value).ok())
        .collect())
}

/// Whether any marker exists for `agent` at the shared agent scope. Never errors
/// the caller out: a read failure reads as "not signed in" so a broken or
/// missing marker file degrades to "needs sign-in" instead of a hard error.
pub fn agent_signed_in(agent: &str) -> bool {
    agent_sign_in_markers()
        .map(|markers| {
            markers
                .iter()
                .any(|marker| marker.agent == agent && marker.scope == AGENT_SIGN_IN_SCOPE)
        })
        .unwrap_or(false)
}

/// Compute per-agent setup readiness for the Home panel. Secret-free and
/// Keychain-free: the caller passes `claude_keychain_present` (the Keychain
/// presence check lives in the desktop app, not core).
///
/// Rules:
/// - A profile with no sign-in (the generic shell) is always "ready".
/// - `claude` is driven by the RunHaven-managed Keychain token presence.
/// - Any other agent that signs in is "signed-in" once a marker exists.
///
/// Known gap (documented, not worked around): an agent signed in only through a
/// terminal, not the device-code runner and not the Claude Keychain, leaves no
/// marker and reads "needs-sign-in". RunHaven will not read the sandbox
/// credential to close this gap; a truthful "needs-sign-in" is preferred over
/// inspecting an isolated token.
pub fn agent_setup_readiness(
    agents: &[&str],
    claude_keychain_present: bool,
) -> Vec<AgentReadiness> {
    agents
        .iter()
        .map(|&name| {
            let (category, label) = if agent_sign_in_mode(name) == AgentSignIn::NotApplicable {
                ("ready", "Ready")
            } else if name == "claude" {
                if claude_keychain_present {
                    ("signed-in", "Signed in")
                } else {
                    ("needs-sign-in", "Needs sign-in")
                }
            } else if agent_signed_in(name) {
                ("signed-in", "Signed in")
            } else {
                ("needs-sign-in", "Needs sign-in")
            };
            AgentReadiness {
                name: name.to_string(),
                category: category.to_string(),
                label: label.to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::paths::{ensure_private_parent, override_cache_root_for_tests};

    fn seed_cache() -> tempfile::TempDir {
        tempfile::tempdir().expect("cache")
    }

    #[test]
    fn record_then_read_round_trips() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        record_agent_sign_in("codex", AGENT_SIGN_IN_SCOPE).expect("record");

        let markers = agent_sign_in_markers().expect("markers");
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].agent, "codex");
        assert_eq!(markers[0].scope, AGENT_SIGN_IN_SCOPE);
        assert!(!markers[0].signed_in_at.is_empty());
    }

    #[test]
    fn agent_signed_in_reflects_recorded_markers() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        assert!(!agent_signed_in("codex"));

        record_agent_sign_in("codex", AGENT_SIGN_IN_SCOPE).expect("record");

        assert!(agent_signed_in("codex"));
        assert!(!agent_signed_in("gemini"));
    }

    #[test]
    fn agent_signed_in_is_false_when_no_marker_file_exists() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        // No file has been written yet: a missing marker file reads as false, not
        // an error.
        assert!(!agent_signed_in("codex"));
    }

    #[test]
    fn markers_read_tolerates_a_malformed_line() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        let path = agent_sign_ins_path();
        ensure_private_parent(&path).expect("parent");
        std::fs::write(
            &path,
            "{ not valid json\n{\"agent\":\"codex\",\"scope\":\"agent\",\"signed_in_at\":\"2026-07-01T00:00:00Z\"}\n",
        )
        .expect("seed markers");

        let markers = agent_sign_in_markers().expect("markers");
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].agent, "codex");
        assert!(agent_signed_in("codex"));
    }

    #[test]
    fn readiness_marks_shell_ready() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        let readiness = agent_setup_readiness(&["shell"], false);
        assert_eq!(readiness[0].category, "ready");
        assert_eq!(readiness[0].label, "Ready");
    }

    #[test]
    fn readiness_follows_claude_keychain_flag() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        let present = agent_setup_readiness(&["claude"], true);
        assert_eq!(present[0].category, "signed-in");
        assert_eq!(present[0].label, "Signed in");

        let absent = agent_setup_readiness(&["claude"], false);
        assert_eq!(absent[0].category, "needs-sign-in");
        assert_eq!(absent[0].label, "Needs sign-in");
    }

    #[test]
    fn readiness_follows_marker_for_a_brokered_agent() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        // Claude's flag must not leak into other agents: a true keychain flag
        // still leaves codex needing sign-in until it has its own marker.
        let before = agent_setup_readiness(&["codex"], true);
        assert_eq!(before[0].category, "needs-sign-in");

        record_agent_sign_in("codex", AGENT_SIGN_IN_SCOPE).expect("record");

        let after = agent_setup_readiness(&["codex"], false);
        assert_eq!(after[0].category, "signed-in");
        assert_eq!(after[0].label, "Signed in");
    }

    #[test]
    fn recorded_marker_is_secret_free() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        record_agent_sign_in("codex", AGENT_SIGN_IN_SCOPE).expect("record");

        let marker = &agent_sign_in_markers().expect("markers")[0];
        let serialized = serde_json::to_string(marker).expect("serialize");

        // Exactly the three secret-free fields, and no token-shaped text.
        let value: serde_json::Value = serde_json::from_str(&serialized).expect("json");
        let object = value.as_object().expect("object");
        assert_eq!(object.len(), 3);
        assert!(object.contains_key("agent"));
        assert!(object.contains_key("scope"));
        assert!(object.contains_key("signed_in_at"));

        let lowered = serialized.to_ascii_lowercase();
        for needle in [
            "token", "secret", "key", "sk-", "bearer", "password", "auth",
        ] {
            assert!(
                !lowered.contains(needle),
                "marker leaked token-shaped text: {needle}"
            );
        }
    }
}
