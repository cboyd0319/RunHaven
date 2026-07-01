//! Sanitized status tracking for background agent-image rebuilds.
//!
//! Mirrors `LaunchStatusRegistry`: a bounded, in-memory map of build progress
//! that the read-only `get_image_build_status` command surfaces to the WebView.
//! Builder output is redacted with the shared run-output scrubber and bounded to
//! a fixed number of tail lines so a long or secret-echoing build cannot leak
//! secrets or grow without limit.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use runhaven_core::provider::observability::utc_timestamp;
use runhaven_core::support::redact::redact_run_output;
use runhaven_core::support::validators::validate_run_id;

use crate::contracts::ImageBuildStatusResponse;

const MAX_IMAGE_BUILD_STATUSES: usize = 20;
const MAX_OUTPUT_TAIL_LINES: usize = 200;
const MAX_OUTPUT_LINE_LEN: usize = 500;

#[derive(Clone, Default)]
pub(crate) struct ImageBuildStatusRegistry {
    state: Arc<Mutex<ImageBuildState>>,
}

#[derive(Default)]
struct ImageBuildState {
    builds: HashMap<String, BuildEntry>,
    order: VecDeque<String>,
}

struct BuildEntry {
    agent: String,
    tag: String,
    state: String,
    label: String,
    detail: String,
    tail: VecDeque<String>,
    updated_at: String,
}

impl ImageBuildStatusRegistry {
    pub(crate) fn mark_building(
        &self,
        build_id: &str,
        agent: &str,
        tag: &str,
    ) -> Result<(), String> {
        validate_build_id(build_id)?;
        let mut state = self.lock()?;
        if !state.builds.contains_key(build_id) {
            state.order.push_back(build_id.to_string());
        }
        state.builds.insert(
            build_id.to_string(),
            BuildEntry {
                agent: agent.to_string(),
                tag: tag.to_string(),
                state: "building".to_string(),
                label: "Building".to_string(),
                detail: format!("RunHaven is rebuilding the {agent} image."),
                tail: VecDeque::new(),
                updated_at: utc_timestamp(),
            },
        );
        prune_builds(&mut state);
        Ok(())
    }

    /// Append one line of builder output. The line is scrubbed of terminal
    /// control sequences and well-known secret shapes, truncated on a char
    /// boundary, and the tail is capped so old lines fall off the front.
    pub(crate) fn append_output(&self, build_id: &str, line: &str) -> Result<(), String> {
        validate_build_id(build_id)?;
        let mut cleaned = redact_run_output(line.trim_end());
        truncate_on_char_boundary(&mut cleaned, MAX_OUTPUT_LINE_LEN);
        let mut state = self.lock()?;
        let Some(entry) = state.builds.get_mut(build_id) else {
            return Ok(());
        };
        entry.tail.push_back(cleaned);
        while entry.tail.len() > MAX_OUTPUT_TAIL_LINES {
            entry.tail.pop_front();
        }
        entry.updated_at = utc_timestamp();
        Ok(())
    }

    pub(crate) fn mark_succeeded(&self, build_id: &str) -> Result<(), String> {
        self.set_terminal(
            build_id,
            "succeeded",
            "Succeeded",
            "The agent image finished rebuilding.".to_string(),
        )
    }

    pub(crate) fn mark_failed(&self, build_id: &str, detail: String) -> Result<(), String> {
        self.set_terminal(build_id, "failed", "Failed", detail)
    }

    pub(crate) fn get(&self, build_id: &str) -> Result<ImageBuildStatusResponse, String> {
        validate_build_id(build_id)?;
        let state = self.lock()?;
        let entry = state
            .builds
            .get(build_id)
            .ok_or_else(|| "image build status not found".to_string())?;
        Ok(ImageBuildStatusResponse {
            build_id: build_id.to_string(),
            agent: entry.agent.clone(),
            tag: entry.tag.clone(),
            state: entry.state.clone(),
            label: entry.label.clone(),
            detail: entry.detail.clone(),
            output_tail: entry.tail.iter().cloned().collect::<Vec<_>>().join("\n"),
            updated_at: entry.updated_at.clone(),
        })
    }

    fn set_terminal(
        &self,
        build_id: &str,
        state_name: &str,
        label: &str,
        detail: String,
    ) -> Result<(), String> {
        validate_build_id(build_id)?;
        let mut state = self.lock()?;
        let Some(entry) = state.builds.get_mut(build_id) else {
            return Ok(());
        };
        entry.state = state_name.to_string();
        entry.label = label.to_string();
        entry.detail = strip_control_chars(&detail);
        entry.updated_at = utc_timestamp();
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ImageBuildState>, String> {
        self.state
            .lock()
            .map_err(|_| "image build status registry is unavailable".to_string())
    }
}

fn validate_build_id(build_id: &str) -> Result<(), String> {
    validate_run_id(build_id).map_err(|error| error.to_string())
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

fn prune_builds(state: &mut ImageBuildState) {
    while state.builds.len() > MAX_IMAGE_BUILD_STATUSES {
        let Some(build_id) = state.order.pop_front() else {
            return;
        };
        state.builds.remove(&build_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_build_registry_redacts_and_bounds_output() {
        let registry = ImageBuildStatusRegistry::default();
        registry
            .mark_building("image-build-run", "codex", "runhaven-codex")
            .expect("building");

        registry
            .append_output(
                "image-build-run",
                "step: sk-ant-api03-abcdefghij_KLmnopqrstuvwxyz1234 leaked\u{1b}[0m",
            )
            .expect("append");
        for line in 0..(MAX_OUTPUT_TAIL_LINES + 50) {
            registry
                .append_output("image-build-run", &format!("layer {line}"))
                .expect("append");
        }

        let status = registry.get("image-build-run").expect("status");
        assert_eq!(status.state, "building");
        assert!(!status.output_tail.contains("sk-ant-api03"));
        assert!(!status.output_tail.contains('\u{1b}'));
        assert_eq!(
            status.output_tail.lines().count(),
            MAX_OUTPUT_TAIL_LINES,
            "tail must stay bounded"
        );

        registry.mark_succeeded("image-build-run").expect("done");
        assert_eq!(
            registry.get("image-build-run").expect("status").state,
            "succeeded"
        );
    }

    #[test]
    fn image_build_registry_reports_missing_build() {
        let registry = ImageBuildStatusRegistry::default();
        let error = registry.get("missing-build-run").expect_err("missing");
        assert!(error.contains("not found"));
    }
}
