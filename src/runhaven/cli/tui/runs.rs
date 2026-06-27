use std::collections::BTreeMap;

use anyhow::{Result, bail};
use serde_json::Value;

use crate::runhaven::diagnostics::read_egress_policy_log;
use crate::runhaven::runtime::active::{
    DEFAULT_LOG_SNAPSHOT_LINES, active_run_log_snapshot_payload, active_run_status_payload,
    kill_active_run, read_active_run_records, repair_active_run, stop_active_run,
};

const EGRESS_LOG_LIMIT: usize = 250;
const LOG_RENDER_MAX_COLUMNS: u16 = 240;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunControlAction {
    Stop,
    Kill,
    Repair,
}

impl RunControlAction {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Stop => "Stop",
            Self::Kill => "Hard stop",
            Self::Repair => "Repair marker",
        }
    }

    pub(crate) fn phrase(self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::Kill => "kill",
            Self::Repair => "repair",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Stop => "asks the container to stop cleanly",
            Self::Kill => "forces the container to stop",
            Self::Repair => "removes a stale marker only if the container is missing",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunControlDialog {
    pub(crate) action: RunControlAction,
    pub(crate) run_id: String,
    pub(crate) container_name: String,
    pub(crate) input: String,
}

impl RunControlDialog {
    pub(crate) fn ready(&self) -> bool {
        self.input == self.action.phrase()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunSummary {
    pub(crate) run_id: String,
    pub(crate) profile: String,
    pub(crate) workspace: String,
    pub(crate) network: String,
    pub(crate) marker_status: String,
    pub(crate) container_name: String,
    pub(crate) state_volume: String,
    pub(crate) timestamp: String,
}

impl RunSummary {
    fn from_record(record: &Value) -> Option<Self> {
        Some(Self {
            run_id: require_str(record, "run_id")?.to_string(),
            container_name: require_str(record, "container_name")?.to_string(),
            profile: value_str(record, "profile"),
            workspace: value_str(record, "workspace"),
            network: value_str(record, "network"),
            marker_status: value_str(record, "status"),
            state_volume: value_str(record, "state_volume"),
            timestamp: value_str(record, "timestamp"),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunStatus {
    pub(crate) marker_status: String,
    pub(crate) container_state: String,
    pub(crate) started_at: String,
    pub(crate) image: String,
    pub(crate) resources: String,
    pub(crate) networks: Vec<String>,
}

impl RunStatus {
    pub(crate) fn from_payload(payload: &Value) -> Self {
        let active_run = &payload["active_run"];
        let container = &payload["container"];
        Self {
            marker_status: value_str(active_run, "status"),
            container_state: value_str(container, "state"),
            started_at: value_str(container, "started_at"),
            image: value_str(container, "image"),
            resources: format_resources(container.get("resources")),
            networks: format_networks(container.get("networks")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EgressDecision {
    pub(crate) timestamp: String,
    pub(crate) decision: String,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) reason: String,
    pub(crate) matched_rule: String,
    pub(crate) count: String,
}

impl EgressDecision {
    fn from_record(record: &Value) -> Option<Self> {
        let host = require_str(record, "host")?.to_string();
        Some(Self {
            timestamp: value_str(record, "timestamp"),
            decision: value_str(record, "decision"),
            host,
            port: record
                .get("port")
                .map(Value::to_string)
                .unwrap_or_else(|| "?".to_string()),
            reason: value_str(record, "reason"),
            matched_rule: value_str(record, "matched_rule"),
            count: record
                .get("count")
                .map(Value::to_string)
                .unwrap_or_else(|| "1".to_string()),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LogSnapshot {
    pub(crate) text: String,
    pub(crate) returned_lines: usize,
    pub(crate) requested_lines: u32,
    pub(crate) truncated: bool,
    pub(crate) warnings: Vec<String>,
}

impl LogSnapshot {
    pub(crate) fn from_payload(payload: &Value) -> Self {
        Self {
            text: value_str(payload, "text"),
            returned_lines: payload
                .get("returned_lines")
                .and_then(Value::as_u64)
                .unwrap_or_default() as usize,
            requested_lines: payload
                .get("requested_lines")
                .and_then(Value::as_u64)
                .unwrap_or(u64::from(DEFAULT_LOG_SNAPSHOT_LINES))
                as u32,
            truncated: payload
                .get("truncated")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            warnings: payload
                .get("warnings")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VisibleLogLine {
    pub(crate) text: String,
    pub(crate) matched: bool,
}

#[derive(Debug)]
pub(crate) struct LogViewerState {
    pub(crate) requested_lines: u32,
    pub(crate) snapshot: Option<LogSnapshot>,
    pub(crate) error: Option<String>,
    pub(crate) search: String,
    pub(crate) search_editing: bool,
    pub(crate) scroll: usize,
    pub(crate) tail: bool,
}

impl Default for LogViewerState {
    fn default() -> Self {
        Self {
            requested_lines: DEFAULT_LOG_SNAPSHOT_LINES,
            snapshot: None,
            error: None,
            search: String::new(),
            search_editing: false,
            scroll: 0,
            tail: true,
        }
    }
}

impl LogViewerState {
    pub(crate) fn set_snapshot(&mut self, snapshot: LogSnapshot) {
        self.snapshot = Some(snapshot);
        self.error = None;
        self.tail = true;
    }

    pub(crate) fn set_error(&mut self, error: impl ToString) {
        self.error = Some(error.to_string());
    }

    pub(crate) fn begin_search(&mut self) {
        self.clear_search();
        self.search_editing = true;
    }

    pub(crate) fn finish_search(&mut self) {
        self.search_editing = false;
    }

    pub(crate) fn push_search_char(&mut self, ch: char) {
        if !ch.is_control() {
            self.search.push(ch);
        }
    }

    pub(crate) fn pop_search_char(&mut self) {
        self.search.pop();
    }

    pub(crate) fn clear_search(&mut self) {
        self.search.clear();
    }

    pub(crate) fn scroll_up(&mut self) {
        self.tail = false;
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub(crate) fn scroll_down(&mut self) {
        self.tail = false;
        self.scroll = self.scroll.saturating_add(1);
    }

    pub(crate) fn follow_tail(&mut self) {
        self.tail = true;
    }

    pub(crate) fn visible_lines(&self, width: u16, height: u16) -> Vec<VisibleLogLine> {
        let Some(snapshot) = &self.snapshot else {
            return Vec::new();
        };
        let rendered = render_log_text_lines(&snapshot.text, width);
        let height = usize::from(height.max(1));
        let max_start = rendered.len().saturating_sub(height);
        let start = if self.tail {
            max_start
        } else {
            self.scroll.min(max_start)
        };
        let query = self.search.to_ascii_lowercase();
        rendered
            .into_iter()
            .skip(start)
            .take(height)
            .map(|text| {
                let matched = !query.is_empty() && text.to_ascii_lowercase().contains(&query);
                VisibleLogLine { text, matched }
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub(crate) struct RunManagerState {
    pub(crate) runs: Vec<RunSummary>,
    pub(crate) selected: usize,
    pub(crate) status: Option<RunStatus>,
    pub(crate) status_error: Option<String>,
    pub(crate) egress: Vec<EgressDecision>,
    pub(crate) egress_error: Option<String>,
    pub(crate) logs: LogViewerState,
    pub(crate) control: Option<RunControlDialog>,
    pub(crate) message: Option<String>,
}

impl RunManagerState {
    pub(crate) fn refresh_dashboard(&mut self) {
        self.refresh_runs();
        self.refresh_selected_status();
        self.refresh_selected_egress();
    }

    pub(crate) fn refresh_runs(&mut self) {
        let previous = self.selected_run_id().map(ToOwned::to_owned);
        self.runs = read_active_run_records()
            .into_iter()
            .filter_map(|record| RunSummary::from_record(&record))
            .collect();
        if let Some(previous) = previous
            && let Some(index) = self.runs.iter().position(|run| run.run_id == previous)
        {
            self.selected = index;
            return;
        }
        self.selected = self.selected.min(self.runs.len().saturating_sub(1));
    }

    pub(crate) fn select_next(&mut self) {
        if !self.runs.is_empty() {
            self.selected = (self.selected + 1).min(self.runs.len() - 1);
            self.refresh_selected_status();
            self.refresh_selected_egress();
        }
    }

    pub(crate) fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.refresh_selected_status();
        self.refresh_selected_egress();
    }

    pub(crate) fn selected_run(&self) -> Option<&RunSummary> {
        self.runs.get(self.selected)
    }

    pub(crate) fn selected_run_id(&self) -> Option<&str> {
        self.selected_run().map(|run| run.run_id.as_str())
    }

    pub(crate) fn refresh_selected_status(&mut self) {
        let Some(run_id) = self.selected_run_id().map(ToOwned::to_owned) else {
            self.status = None;
            self.status_error = None;
            return;
        };
        match active_run_status_payload(&run_id) {
            Ok(payload) => {
                self.status = Some(RunStatus::from_payload(&payload));
                self.status_error = None;
            }
            Err(error) => {
                self.status = None;
                self.status_error = Some(error.to_string());
            }
        }
    }

    pub(crate) fn refresh_selected_egress(&mut self) {
        let Some(run_id) = self.selected_run_id().map(ToOwned::to_owned) else {
            self.egress.clear();
            self.egress_error = None;
            return;
        };
        match read_egress_policy_log(EGRESS_LOG_LIMIT) {
            Ok(entries) => {
                self.egress = egress_decisions_for_run(&entries, &run_id);
                self.egress_error = None;
            }
            Err(error) => {
                self.egress.clear();
                self.egress_error = Some(error.to_string());
            }
        }
    }

    pub(crate) fn refresh_logs(&mut self) {
        let Some(run_id) = self.selected_run_id().map(ToOwned::to_owned) else {
            self.logs.set_error("No active run selected.");
            return;
        };
        match active_run_log_snapshot_payload(&run_id, self.logs.requested_lines) {
            Ok(payload) => self.logs.set_snapshot(LogSnapshot::from_payload(&payload)),
            Err(error) => self.logs.set_error(error),
        }
    }

    pub(crate) fn begin_control(&mut self, action: RunControlAction) -> Result<()> {
        let run = self
            .selected_run()
            .ok_or_else(|| anyhow::anyhow!("No active run selected."))?;
        self.control = Some(RunControlDialog {
            action,
            run_id: run.run_id.clone(),
            container_name: run.container_name.clone(),
            input: String::new(),
        });
        Ok(())
    }

    pub(crate) fn execute_control(&mut self) -> Result<()> {
        let dialog = self
            .control
            .take()
            .ok_or_else(|| anyhow::anyhow!("No run control is active."))?;
        if !dialog.ready() {
            bail!("type {} to confirm", dialog.action.phrase());
        }
        let payload = match dialog.action {
            RunControlAction::Stop => stop_active_run(&dialog.run_id)?,
            RunControlAction::Kill => kill_active_run(&dialog.run_id)?,
            RunControlAction::Repair => repair_active_run(&dialog.run_id)?,
        };
        self.message = Some(control_message(dialog.action, &payload));
        self.refresh_dashboard();
        Ok(())
    }

    pub(crate) fn notifications(&self) -> Vec<String> {
        let mut notifications = Vec::new();
        if let Some(error) = &self.status_error {
            notifications.push(format!("Status needs attention: {error}"));
        }
        if let Some(status) = &self.status {
            let marker = status.marker_status.to_ascii_lowercase();
            let container = status.container_state.to_ascii_lowercase();
            if marker.contains("stop-requested") || marker.contains("kill-requested") {
                notifications.push(
                    "Run control is in progress; refresh status before sending another control."
                        .to_string(),
                );
            }
            if container.contains("exited")
                || container.contains("stopped")
                || container.contains("not found")
            {
                notifications.push(
                    "Run may be done or stale; review logs/history, then repair the marker if needed."
                        .to_string(),
                );
            }
            if marker.contains("repair") || marker.contains("stale") {
                notifications.push(
                    "Active marker may be stale; use repair after confirming the container is gone."
                        .to_string(),
                );
            }
        }
        if let Some(snapshot) = &self.logs.snapshot {
            let lower = snapshot.text.to_ascii_lowercase();
            if lower.contains("waiting for input")
                || lower.contains("press enter")
                || lower.contains("device code")
            {
                notifications.push(
                    "Output may be waiting for input; use the CLI attach path for interactive input."
                        .to_string(),
                );
            }
        }
        notifications
    }
}

pub(crate) fn egress_decisions_for_run(entries: &[Value], run_id: &str) -> Vec<EgressDecision> {
    entries
        .iter()
        .filter(|entry| entry.get("run_id").and_then(Value::as_str) == Some(run_id))
        .filter_map(EgressDecision::from_record)
        .collect()
}

pub(crate) fn render_log_text_lines(text: &str, width: u16) -> Vec<String> {
    let columns = width.clamp(1, LOG_RENDER_MAX_COLUMNS);
    let estimated_wrap_rows = text.len().saturating_div(usize::from(columns));
    let rows = text
        .lines()
        .count()
        .saturating_add(estimated_wrap_rows)
        .saturating_add(1)
        .clamp(1, 500) as u16;
    let mut parser = vt100::Parser::new(rows, columns, 0);
    parser.process(text.as_bytes());
    let contents = parser.screen().contents();
    if contents.is_empty() {
        return Vec::new();
    }
    contents.lines().map(|line| line.to_string()).collect()
}

pub(crate) fn summarize_egress(decisions: &[EgressDecision]) -> String {
    let mut totals = BTreeMap::<&str, usize>::new();
    for decision in decisions {
        let count = decision.count.parse::<usize>().unwrap_or(1);
        *totals.entry(decision.decision.as_str()).or_default() += count;
    }
    if totals.is_empty() {
        return "no provider egress decisions logged yet".to_string();
    }
    totals
        .into_iter()
        .map(|(decision, count)| format!("{decision}={count}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn control_message(action: RunControlAction, payload: &Value) -> String {
    let run_id = payload.get("run_id").and_then(Value::as_str).unwrap_or("-");
    let code = payload.get("return_code").and_then(Value::as_i64);
    let status = payload.get("status").and_then(Value::as_str);
    match (action, code, status) {
        (RunControlAction::Repair, _, Some(status)) => {
            format!("Repair {status} for run {run_id}.")
        }
        (_, Some(code), _) => format!(
            "{} requested for run {run_id} (return={code}).",
            action.label()
        ),
        _ => format!("{} requested for run {run_id}.", action.label()),
    }
}

fn require_str<'a>(record: &'a Value, key: &str) -> Option<&'a str> {
    record.get(key).and_then(Value::as_str)
}

fn value_str(record: &Value, key: &str) -> String {
    record
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string()
}

fn format_resources(resources: Option<&Value>) -> String {
    let Some(resources) = resources.and_then(Value::as_object) else {
        return "-".to_string();
    };
    let cpus = resources
        .get("cpus")
        .and_then(Value::as_f64)
        .map(format_cpu_count);
    let memory = resources
        .get("memory_in_bytes")
        .and_then(Value::as_u64)
        .map(format_bytes);
    [cpus, memory]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" / ")
}

fn format_cpu_count(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{} cpu", value as u64)
    } else {
        format!("{value:.1} cpu")
    }
}

fn format_networks(networks: Option<&Value>) -> Vec<String> {
    networks
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|network| {
            let name = network
                .get("network")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let ipv4 = network
                .get("ipv4_address")
                .and_then(Value::as_str)
                .unwrap_or("-");
            let hostname = network
                .get("hostname")
                .and_then(Value::as_str)
                .unwrap_or("-");
            format!("{name} ipv4={ipv4} hostname={hostname}")
        })
        .collect()
}

fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;
    if bytes >= GIB {
        format!("{} GiB", bytes / GIB)
    } else if bytes >= MIB {
        format!("{} MiB", bytes / MIB)
    } else {
        format!("{bytes} bytes")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn run_summary_requires_safe_identity_fields() {
        let summary = RunSummary::from_record(&json!({
            "run_id": "abc",
            "container_name": "runhaven-abc-run",
            "profile": "codex",
            "workspace": "/workspace",
            "network": "provider",
            "status": "running",
            "state_volume": "runhaven-codex-shared-home",
            "timestamp": "2026-06-27T00:00:00Z"
        }))
        .expect("summary");

        assert_eq!(summary.run_id, "abc");
        assert_eq!(summary.profile, "codex");
        assert!(RunSummary::from_record(&json!({"run_id": "abc"})).is_none());
    }

    #[test]
    fn status_payload_is_sanitized_summary() {
        let status = RunStatus::from_payload(&json!({
            "active_run": {"status": "running"},
            "container": {
                "state": "running",
                "started_at": "2026-06-27T00:00:01Z",
                "image": "runhaven/codex:0.1.0",
                "resources": {"cpus": 4, "memory_in_bytes": 4294967296u64},
                "networks": [{"network": "runhaven-codex", "ipv4_address": "192.0.2.2/24", "hostname": "runhaven"}]
            }
        }));

        assert_eq!(status.marker_status, "running");
        assert_eq!(status.resources, "4 cpu / 4 GiB");
        assert_eq!(
            status.networks[0],
            "runhaven-codex ipv4=192.0.2.2/24 hostname=runhaven"
        );
    }

    #[test]
    fn egress_decisions_are_filtered_by_run_id() {
        let entries = vec![
            json!({"run_id": "one", "host": "api.example.com", "port": 443, "decision": "allowed", "reason": "allowed", "matched_rule": "api.example.com", "count": 2}),
            json!({"run_id": "two", "host": "other.example.com", "port": 443, "decision": "denied"}),
        ];

        let decisions = egress_decisions_for_run(&entries, "one");
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].host, "api.example.com");
        assert_eq!(summarize_egress(&decisions), "allowed=2");
    }

    #[test]
    fn log_renderer_interprets_ansi_without_replaying_escapes() {
        let lines = render_log_text_lines("\x1b[31mred\x1b[0m\nplain\n", 20);

        assert!(lines.iter().any(|line| line.contains("red")));
        assert!(lines.iter().all(|line| !line.contains('\x1b')));
    }

    #[test]
    fn log_renderer_allocates_rows_for_wrapped_lines() {
        let lines = render_log_text_lines("abcdefghij", 4);

        assert!(lines.iter().any(|line| line.contains("abcd")));
        assert!(lines.iter().any(|line| line.contains("efgh")));
        assert!(lines.iter().any(|line| line.contains("ij")));
    }

    #[test]
    fn log_viewer_tails_and_marks_search_matches() {
        let mut viewer = LogViewerState::default();
        viewer.set_snapshot(LogSnapshot {
            text: "alpha\nbeta\ngamma\n".to_string(),
            returned_lines: 3,
            requested_lines: 3,
            truncated: false,
            warnings: Vec::new(),
        });
        viewer.search = "amm".to_string();

        let visible = viewer.visible_lines(20, 2);
        assert_eq!(visible.len(), 2);
        assert!(visible[1].matched);
    }

    #[test]
    fn notifications_detect_done_and_waiting_states() {
        let mut manager = RunManagerState {
            status: Some(RunStatus {
                marker_status: "running".to_string(),
                container_state: "exited".to_string(),
                started_at: "-".to_string(),
                image: "-".to_string(),
                resources: "-".to_string(),
                networks: Vec::new(),
            }),
            ..RunManagerState::default()
        };
        manager.logs.set_snapshot(LogSnapshot {
            text: "Device code login is waiting for input. Press enter.".to_string(),
            returned_lines: 1,
            requested_lines: 1,
            truncated: false,
            warnings: Vec::new(),
        });

        let notifications = manager.notifications();

        assert!(
            notifications
                .iter()
                .any(|message| message.contains("done or stale"))
        );
        assert!(
            notifications
                .iter()
                .any(|message| message.contains("waiting for input"))
        );
    }
}
