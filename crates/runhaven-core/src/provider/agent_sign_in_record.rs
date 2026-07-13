//! Per-agent sign-in tracking that RunHaven can truthfully know.
//!
//! A record tracks whether an agent completed or cleared its sign-in flow INSIDE
//! ITS ISOLATED SANDBOX. RunHaven never reads the credential: each record is
//! secret-free and carries only an agent, scope, timestamp, and active state.
//!
//! The marker means "the agent signed in in its sandbox", NOT "signed in via
//! RunHaven" and NOT "the token is still valid". RunHaven does not store or
//! manage the credential; it only witnessed the sign-in flow finish. Readiness
//! labels describe the agent's state only.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::provider::auth_profiles::{AgentSignIn, agent_sign_in_mode};
use crate::provider::observability::utc_timestamp;
use crate::records::{append_private_jsonl, read_jsonl};
use crate::runtime::session_state::shared_state_volume_name;
use crate::support::paths::agent_sign_ins_path;
use crate::support::validators::validate_run_id;

/// The shared-agent sign-in scope: a device-code sign-in signs the agent in for
/// its shared agent state, not a per-workspace scope.
pub const AGENT_SIGN_IN_SCOPE: &str = "agent";

/// A secret-free sign-in state record. `updated_at` accepts the historical
/// `signed_in_at` field, and a missing `active` field means signed in, so
/// existing records remain valid after the append-only sign-out state was
/// introduced.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentSignInMarker {
    pub agent: String,
    pub scope: String,
    #[serde(alias = "signed_in_at")]
    pub updated_at: String,
    #[serde(default = "active_by_default")]
    pub active: bool,
}

const fn active_by_default() -> bool {
    true
}

/// The per-agent readiness the Home "Agent setup" panel shows. Every field is a
/// plain, non-secret string describing the AGENT's state, never RunHaven's.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReadiness {
    pub name: String,
    /// One of "ready", "sign-in-saved", or "needs-sign-in".
    pub category: String,
    pub label: String,
}

/// Append one secret-free signed-in state for `agent` at `scope`.
///
/// The agent id and scope are validated with the same id validator the run log
/// uses, so a control or terminal-escape byte cannot ride into the record and
/// back out into a rendered surface. The timestamp comes from the shared UTC
/// helper, never a forbidden wall-clock call.
pub fn record_agent_sign_in(agent: &str, scope: &str) -> Result<()> {
    record_agent_sign_in_state(agent, scope, true)
}

/// Append one secret-free signed-out state after the isolated login volume was
/// deleted. This does not inspect or handle the provider credential.
pub fn record_agent_sign_out(agent: &str, scope: &str) -> Result<()> {
    record_agent_sign_in_state(agent, scope, false)
}

fn record_agent_sign_in_state(agent: &str, scope: &str, active: bool) -> Result<()> {
    validate_run_id(agent)?;
    validate_run_id(scope)?;
    let marker = AgentSignInMarker {
        agent: agent.to_string(),
        scope: scope.to_string(),
        updated_at: utc_timestamp(),
        active,
    };
    let path = agent_sign_ins_path();
    append_private_jsonl(&path, &[marker])
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

/// Whether the latest record for `agent` at the shared agent scope is active.
/// Never errors the caller out: a read failure reads as "not signed in" so a
/// broken or missing marker file degrades to "needs sign-in" instead of a hard
/// error.
pub fn agent_signed_in(agent: &str) -> bool {
    agent_sign_in_markers()
        .map(|markers| {
            markers
                .iter()
                .rev()
                .find(|marker| marker.agent == agent && marker.scope == AGENT_SIGN_IN_SCOPE)
                .is_some_and(|marker| marker.active)
        })
        .unwrap_or(false)
}

/// Compute per-agent setup readiness for the Home panel. Secret-free and
/// Credential-free: readiness is based on the secret-free completion marker
/// RunHaven writes after a successful native sign-in terminal plus the
/// continued presence of that agent's isolated login volume.
///
/// Rules:
/// - A profile with no sign-in (the generic shell) is always "ready".
/// - Every supported agent has "sign-in-saved" once its marker and volume exist.
///
/// A marker means RunHaven witnessed a successful sign-in, not that a token is
/// still valid. RunHaven does not inspect the isolated credential.
pub fn agent_setup_readiness(
    agents: &[&str],
    existing_state_volumes: &[String],
) -> Vec<AgentReadiness> {
    agents
        .iter()
        .map(|&name| {
            let (category, label) = if agent_sign_in_mode(name) == AgentSignIn::NotApplicable {
                ("ready", "Ready")
            } else if agent_signed_in(name)
                && existing_state_volumes
                    .iter()
                    .any(|volume| volume == &shared_state_volume_name(name))
            {
                ("sign-in-saved", "Sign-in saved")
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
        assert!(!markers[0].updated_at.is_empty());
        assert!(markers[0].active);
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
    fn latest_sign_out_record_clears_readiness_and_allows_sign_in_again() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        record_agent_sign_in("codex", AGENT_SIGN_IN_SCOPE).expect("sign in");
        record_agent_sign_out("codex", AGENT_SIGN_IN_SCOPE).expect("sign out");
        assert!(!agent_signed_in("codex"));

        record_agent_sign_in("codex", AGENT_SIGN_IN_SCOPE).expect("sign in again");
        assert!(agent_signed_in("codex"));
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
        assert!(markers[0].active, "legacy record defaults to active");
        assert!(agent_signed_in("codex"));
    }

    #[test]
    fn readiness_marks_shell_ready() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        let readiness = agent_setup_readiness(&["shell"], &[]);
        assert_eq!(readiness[0].category, "ready");
        assert_eq!(readiness[0].label, "Ready");
    }

    #[test]
    fn readiness_describes_saved_sign_in_without_claiming_current_validity() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        for agent in ["claude", "codex", "antigravity", "copilot"] {
            let volumes = vec![shared_state_volume_name(agent)];
            let before = agent_setup_readiness(&[agent], &volumes);
            assert_eq!(before[0].category, "needs-sign-in");

            record_agent_sign_in(agent, AGENT_SIGN_IN_SCOPE).expect("record");

            let after = agent_setup_readiness(&[agent], &volumes);
            assert_eq!(
                (after[0].category.as_str(), after[0].label.as_str()),
                ("sign-in-saved", "Sign-in saved")
            );
        }
    }

    #[test]
    fn readiness_fails_closed_when_the_login_volume_is_missing() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        record_agent_sign_in("codex", AGENT_SIGN_IN_SCOPE).expect("record");

        let readiness = agent_setup_readiness(&["codex"], &[]);

        assert_eq!(readiness[0].category, "needs-sign-in");
    }

    #[test]
    fn recorded_marker_is_secret_free() {
        let cache = seed_cache();
        let _override = override_cache_root_for_tests(cache.path());
        record_agent_sign_in("codex", AGENT_SIGN_IN_SCOPE).expect("record");

        let marker = &agent_sign_in_markers().expect("markers")[0];
        let serialized = serde_json::to_string(marker).expect("serialize");

        // Exactly the four secret-free fields, and no token-shaped text.
        let value: serde_json::Value = serde_json::from_str(&serialized).expect("json");
        let object = value.as_object().expect("object");
        assert_eq!(object.len(), 4);
        assert!(object.contains_key("agent"));
        assert!(object.contains_key("scope"));
        assert!(object.contains_key("updated_at"));
        assert!(object.contains_key("active"));

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
