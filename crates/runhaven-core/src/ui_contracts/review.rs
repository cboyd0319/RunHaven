use serde::{Deserialize, Serialize};

use super::helpers::{
    i32_field, string_field, string_pointer_field, strip_terminal_controls,
    strip_terminal_controls_preserving_lines, u64_pointer_field,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDiffData {
    pub run_id: String,
    pub text: String,
    pub returned_lines: usize,
    pub truncated: bool,
    pub source: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunControlResultData {
    pub action: String,
    pub run_id: String,
    pub container_name: String,
    pub return_code: Option<i32>,
    pub status: String,
    pub marker_removed: Option<bool>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunHistoryListData {
    pub runs: Vec<RunHistorySummaryData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunHistorySummaryData {
    pub run_id: String,
    pub profile: String,
    pub network: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
    pub return_code: Option<i32>,
    pub workspace_scope: String,
    pub session: String,
    pub state_volume: String,
    pub provider_allowed: u64,
    pub provider_denied: u64,
    pub auth_allowed: u64,
    pub auth_denied: u64,
    pub cleanup_provider_network: String,
    pub git_summary: String,
    pub worktree_branch: Option<String>,
    pub review_command: String,
}

impl RunHistoryListData {
    pub fn from_run_records(records: impl IntoIterator<Item = serde_json::Value>) -> Self {
        let mut runs = records
            .into_iter()
            .map(RunHistorySummaryData::from_run_record)
            .collect::<Vec<_>>();
        runs.sort_by(|left, right| {
            right
                .finished_at
                .cmp(&left.finished_at)
                .then_with(|| right.started_at.cmp(&left.started_at))
                .then_with(|| right.run_id.cmp(&left.run_id))
        });
        Self { runs }
    }
}

impl RunHistorySummaryData {
    pub fn from_run_record(record: serde_json::Value) -> Self {
        let run_id = string_field(&record, "run_id");
        let worktree_branch = record
            .get("worktree")
            .and_then(|worktree| worktree.get("branch"))
            .and_then(serde_json::Value::as_str)
            .map(strip_terminal_controls);
        Self {
            review_command: format!("runhaven runs show {run_id}"),
            run_id,
            profile: string_field(&record, "profile"),
            network: string_field(&record, "network"),
            status: string_field(&record, "status"),
            started_at: string_field(&record, "started_at"),
            finished_at: string_field(&record, "finished_at"),
            return_code: i32_field(&record, "return_code"),
            workspace_scope: string_field(&record, "workspace_scope"),
            session: string_field(&record, "session"),
            state_volume: string_field(&record, "state_volume"),
            provider_allowed: u64_pointer_field(&record, "/provider_policy/allowed"),
            provider_denied: u64_pointer_field(&record, "/provider_policy/denied"),
            auth_allowed: u64_pointer_field(&record, "/auth_broker/allowed"),
            auth_denied: u64_pointer_field(&record, "/auth_broker/denied"),
            cleanup_provider_network: string_pointer_field(&record, "/cleanup/provider_network"),
            git_summary: strip_terminal_controls(&crate::records::format_git_summary(
                record.get("git").unwrap_or(&serde_json::Value::Null),
            )),
            worktree_branch,
        }
    }
}

impl RunDiffData {
    pub fn from_run_diff_text(run_id: &str, text: &str, max_lines: usize) -> Self {
        let sanitized = strip_terminal_controls_preserving_lines(text);
        let mut returned = Vec::new();
        let mut total_lines = 0usize;
        for line in sanitized.lines() {
            total_lines += 1;
            if returned.len() < max_lines {
                returned.push(line.to_string());
            }
        }
        let mut text = returned.join("\n");
        if !text.is_empty() {
            text.push('\n');
        }
        let truncated = total_lines > returned.len();
        let mut warnings = vec!["Diff can include workspace file contents.".to_string()];
        if truncated {
            warnings.push("Diff output was truncated to the bounded preview size.".to_string());
        }

        Self {
            run_id: strip_terminal_controls(run_id),
            text,
            returned_lines: returned.len(),
            truncated,
            source: "git diff".to_string(),
            warnings,
        }
    }
}

impl RunControlResultData {
    pub fn from_stop_payload(value: serde_json::Value) -> serde_json::Result<Self> {
        Self::from_stop_or_kill_payload("stop", value)
    }

    pub fn from_kill_payload(value: serde_json::Value) -> serde_json::Result<Self> {
        Self::from_stop_or_kill_payload("kill", value)
    }

    pub fn from_repair_payload(value: serde_json::Value) -> serde_json::Result<Self> {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        struct RawRepairResult {
            run_id: String,
            container_name: String,
            inspect_return_code: i32,
            marker_removed: bool,
            status: String,
        }

        let raw: RawRepairResult = serde_json::from_value(value)?;
        let run_id = strip_terminal_controls(&raw.run_id);
        let container_name = strip_terminal_controls(&raw.container_name);
        let status = strip_terminal_controls(&raw.status);
        Ok(Self {
            action: "repair".to_string(),
            return_code: Some(raw.inspect_return_code),
            marker_removed: Some(raw.marker_removed),
            message: repair_message(&run_id, &container_name, &status),
            run_id,
            container_name,
            status,
        })
    }

    fn from_stop_or_kill_payload(
        action: &'static str,
        value: serde_json::Value,
    ) -> serde_json::Result<Self> {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        struct RawRunControlResult {
            run_id: String,
            container_name: String,
            return_code: i32,
        }

        let raw: RawRunControlResult = serde_json::from_value(value)?;
        let run_id = strip_terminal_controls(&raw.run_id);
        let container_name = strip_terminal_controls(&raw.container_name);
        let status = if raw.return_code == 0 {
            "requested"
        } else {
            "failed"
        }
        .to_string();
        Ok(Self {
            action: action.to_string(),
            return_code: Some(raw.return_code),
            marker_removed: None,
            message: stop_or_kill_message(action, &run_id, &container_name, raw.return_code),
            run_id,
            container_name,
            status,
        })
    }
}

fn stop_or_kill_message(
    action: &str,
    run_id: &str,
    container_name: &str,
    return_code: i32,
) -> String {
    let label = if action == "kill" {
        "Hard stop"
    } else {
        "Stop"
    };
    if return_code == 0 {
        format!("{label} requested for run {run_id} ({container_name}).")
    } else {
        format!("{label} failed for run {run_id} ({container_name}) with exit code {return_code}.")
    }
}

fn repair_message(run_id: &str, container_name: &str, status: &str) -> String {
    match status {
        "removed" => format!("Removed stale active marker for run {run_id} ({container_name})."),
        "kept" => format!("Kept active marker for run {run_id}; the container still exists."),
        _ => format!("Could not verify active marker for run {run_id} ({container_name})."),
    }
}
