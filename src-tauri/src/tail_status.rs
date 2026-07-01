//! Sanitized, bounded status tracking for background jobs that stream output.
//!
//! Agent-image rebuilds and in-app agent sign-ins both keep a bounded, in-memory
//! map of job progress plus a redacted, tail-bounded slice of the job's output
//! that a read-only command surfaces to the WebView. The shared mechanics live
//! in `TailStatusRegistry`: bound the map, prune the oldest jobs, redact and
//! truncate each output line, cap the tail, strip control characters from
//! terminal detail, and fail closed on a poisoned lock. Two thin per-domain
//! wrappers keep the distinct response types, states, labels, and messages the
//! command layer and WebView depend on.
//!
//! Image builds carry an image `tag`; sign-ins do not, and a sign-in flips from
//! `starting` to `needs_browser` on its first line of output (when the login URL
//! and short-lived device code appear). Nothing here reads, stores, or returns
//! an agent token: sign-in credentials stay owned by RunHaven core.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use runhaven_core::provider::observability::utc_timestamp;
use runhaven_core::support::redact::redact_run_output;

use crate::contracts::{AgentSignInStatusResponse, ImageBuildStatusResponse};
use crate::registry_support::{
    BoundedMap, lock_or, strip_control_chars, truncate_on_char_boundary, validate_registry_id,
};

const MAX_IMAGE_BUILD_STATUSES: usize = 20;
const MAX_SIGN_IN_STATUSES: usize = 20;
const MAX_OUTPUT_TAIL_LINES: usize = 200;
const MAX_OUTPUT_LINE_LEN: usize = 500;

/// One tracked job: its domain fields, current sanitized state, and a bounded,
/// redacted tail of output lines. `tag` is `Some` for image builds and `None`
/// for sign-ins.
struct TailEntry {
    agent: String,
    tag: Option<String>,
    state: String,
    label: String,
    detail: String,
    tail: VecDeque<String>,
    updated_at: String,
}

impl TailEntry {
    fn output_tail(&self) -> String {
        self.tail.iter().cloned().collect::<Vec<_>>().join("\n")
    }
}

/// The shared, bounded, fail-closed mechanics behind both tail registries. Each
/// wrapper configures its map cap and its lock/not-found messages so the
/// security-relevant bounding and error behavior has one implementation.
#[derive(Clone)]
struct TailStatusRegistry {
    entries: Arc<Mutex<BoundedMap<TailEntry>>>,
    max_entries: usize,
    lock_message: &'static str,
    not_found_message: &'static str,
}

impl TailStatusRegistry {
    fn new(
        max_entries: usize,
        lock_message: &'static str,
        not_found_message: &'static str,
    ) -> Self {
        Self {
            entries: Arc::new(Mutex::new(BoundedMap::default())),
            max_entries,
            lock_message,
            not_found_message,
        }
    }

    /// Record a new job in its initial state and prune the oldest jobs.
    fn insert_starting(
        &self,
        id: &str,
        agent: &str,
        tag: Option<String>,
        state: &str,
        label: &str,
        detail: String,
    ) -> Result<(), String> {
        validate_registry_id(id)?;
        let mut entries = lock_or(self.entries.lock(), self.lock_message)?;
        entries.insert_tracked(
            id,
            TailEntry {
                agent: agent.to_string(),
                tag,
                state: state.to_string(),
                label: label.to_string(),
                detail,
                tail: VecDeque::new(),
                updated_at: utc_timestamp(),
            },
        );
        entries.prune(self.max_entries);
        Ok(())
    }

    /// Append one redacted, length-capped line of output, keeping the tail
    /// bounded. `on_first_output`, when set, runs against the entry on its first
    /// line of output (used to flip a sign-in to `needs_browser`).
    fn append(
        &self,
        id: &str,
        line: &str,
        on_first_output: Option<fn(&mut TailEntry)>,
    ) -> Result<(), String> {
        validate_registry_id(id)?;
        let mut cleaned = redact_run_output(line.trim_end());
        truncate_on_char_boundary(&mut cleaned, MAX_OUTPUT_LINE_LEN);
        let mut entries = lock_or(self.entries.lock(), self.lock_message)?;
        let Some(entry) = entries.get_mut(id) else {
            return Ok(());
        };
        if entry.tail.is_empty()
            && let Some(transition) = on_first_output
        {
            transition(entry);
        }
        entry.tail.push_back(cleaned);
        while entry.tail.len() > MAX_OUTPUT_TAIL_LINES {
            entry.tail.pop_front();
        }
        entry.updated_at = utc_timestamp();
        Ok(())
    }

    /// Move a job to a terminal state, stripping control characters from the
    /// terminal detail line.
    fn set_terminal(
        &self,
        id: &str,
        state: &str,
        label: &str,
        detail: String,
    ) -> Result<(), String> {
        validate_registry_id(id)?;
        let mut entries = lock_or(self.entries.lock(), self.lock_message)?;
        let Some(entry) = entries.get_mut(id) else {
            return Ok(());
        };
        entry.state = state.to_string();
        entry.label = label.to_string();
        entry.detail = strip_control_chars(&detail);
        entry.updated_at = utc_timestamp();
        Ok(())
    }

    /// Read a tracked job, building the caller's response type directly from the
    /// entry so no intermediate snapshot type is needed.
    fn read<R>(&self, id: &str, build: impl FnOnce(&TailEntry) -> R) -> Result<R, String> {
        validate_registry_id(id)?;
        let entries = lock_or(self.entries.lock(), self.lock_message)?;
        let entry = entries
            .get(id)
            .ok_or_else(|| self.not_found_message.to_string())?;
        Ok(build(entry))
    }
}

/// Sanitized status tracking for background agent-image rebuilds. Surfaced by the
/// read-only `get_image_build_status` command.
#[derive(Clone)]
pub(crate) struct ImageBuildStatusRegistry {
    inner: TailStatusRegistry,
}

impl Default for ImageBuildStatusRegistry {
    fn default() -> Self {
        Self {
            inner: TailStatusRegistry::new(
                MAX_IMAGE_BUILD_STATUSES,
                "image build status registry is unavailable",
                "image build status not found",
            ),
        }
    }
}

impl ImageBuildStatusRegistry {
    pub(crate) fn mark_building(
        &self,
        build_id: &str,
        agent: &str,
        tag: &str,
    ) -> Result<(), String> {
        self.inner.insert_starting(
            build_id,
            agent,
            Some(tag.to_string()),
            "building",
            "Building",
            format!("RunHaven is rebuilding the {agent} image."),
        )
    }

    pub(crate) fn append_output(&self, build_id: &str, line: &str) -> Result<(), String> {
        self.inner.append(build_id, line, None)
    }

    pub(crate) fn mark_succeeded(&self, build_id: &str) -> Result<(), String> {
        self.inner.set_terminal(
            build_id,
            "succeeded",
            "Succeeded",
            "The agent image finished rebuilding.".to_string(),
        )
    }

    pub(crate) fn mark_failed(&self, build_id: &str, detail: String) -> Result<(), String> {
        self.inner
            .set_terminal(build_id, "failed", "Failed", detail)
    }

    pub(crate) fn get(&self, build_id: &str) -> Result<ImageBuildStatusResponse, String> {
        self.inner.read(build_id, |entry| ImageBuildStatusResponse {
            build_id: build_id.to_string(),
            agent: entry.agent.clone(),
            tag: entry.tag.clone().unwrap_or_default(),
            state: entry.state.clone(),
            label: entry.label.clone(),
            detail: entry.detail.clone(),
            output_tail: entry.output_tail(),
            updated_at: entry.updated_at.clone(),
        })
    }
}

/// Sanitized status tracking for background in-app agent sign-ins. Surfaced by
/// the read-only `get_agent_sign_in_status` command.
#[derive(Clone)]
pub(crate) struct AgentSignInStatusRegistry {
    inner: TailStatusRegistry,
}

impl Default for AgentSignInStatusRegistry {
    fn default() -> Self {
        Self {
            inner: TailStatusRegistry::new(
                MAX_SIGN_IN_STATUSES,
                "sign-in status registry is unavailable",
                "sign-in status not found",
            ),
        }
    }
}

impl AgentSignInStatusRegistry {
    pub(crate) fn mark_starting(&self, sign_in_id: &str, agent: &str) -> Result<(), String> {
        self.inner.insert_starting(
            sign_in_id,
            agent,
            None,
            "starting",
            "Starting",
            format!("RunHaven is starting sign-in for {agent}."),
        )
    }

    pub(crate) fn append_output(&self, sign_in_id: &str, line: &str) -> Result<(), String> {
        self.inner
            .append(sign_in_id, line, Some(flip_to_needs_browser))
    }

    pub(crate) fn mark_succeeded(&self, sign_in_id: &str, agent: &str) -> Result<(), String> {
        self.inner.set_terminal(
            sign_in_id,
            "succeeded",
            "Signed in",
            format!("You are signed in to {agent}."),
        )
    }

    pub(crate) fn mark_failed(&self, sign_in_id: &str, detail: String) -> Result<(), String> {
        self.inner
            .set_terminal(sign_in_id, "failed", "Sign-in failed", detail)
    }

    pub(crate) fn get(&self, sign_in_id: &str) -> Result<AgentSignInStatusResponse, String> {
        self.inner
            .read(sign_in_id, |entry| AgentSignInStatusResponse {
                sign_in_id: sign_in_id.to_string(),
                agent: entry.agent.clone(),
                state: entry.state.clone(),
                label: entry.label.clone(),
                detail: entry.detail.clone(),
                output_tail: entry.output_tail(),
                updated_at: entry.updated_at.clone(),
            })
    }
}

/// The sign-in first-output transition: the first login line is where the URL
/// and short-lived device code appear, so the UI is told to open the browser.
fn flip_to_needs_browser(entry: &mut TailEntry) {
    entry.state = "needs_browser".to_string();
    entry.label = "Open your browser".to_string();
    entry.detail =
        "Open the sign-in link shown below and finish in your browser. RunHaven is waiting."
            .to_string();
}

#[cfg(test)]
mod tests;
