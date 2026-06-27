use serde_json::Value;

use crate::runhaven::diagnostics::{
    auth_status_payload, read_auth_broker_log, read_egress_policy_log,
};
use crate::runhaven::doctor::{Check, collect_checks};
use crate::runhaven::records::{format_git_summary, read_run_records, run_diff_text};

use super::codex::image_protocol::ImageProtocol;
use super::theme::{MotionMode, TuiSettings};

const HISTORY_LIMIT: usize = 100;
const DIAGNOSTIC_LOG_LIMIT: usize = 80;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunHistorySummary {
    pub(crate) run_id: String,
    pub(crate) profile: String,
    pub(crate) workspace: String,
    pub(crate) network: String,
    pub(crate) status: String,
    pub(crate) return_code: String,
    pub(crate) timestamp: String,
    pub(crate) provider_denied: u64,
    pub(crate) auth_denied: u64,
    pub(crate) cleanup: String,
    pub(crate) git_summary: String,
}

impl RunHistorySummary {
    fn from_record(record: &Value) -> Option<Self> {
        let run_id = record.get("run_id").and_then(Value::as_str)?.to_string();
        Some(Self {
            run_id,
            profile: value_str(record, "profile"),
            workspace: value_str(record, "workspace"),
            network: value_str(record, "network"),
            status: value_str(record, "status"),
            return_code: record
                .get("return_code")
                .map(Value::to_string)
                .unwrap_or_else(|| "-".to_string()),
            timestamp: value_str(record, "timestamp"),
            provider_denied: record
                .pointer("/provider_policy/denied")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            auth_denied: record
                .pointer("/auth_broker/denied")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            cleanup: record
                .pointer("/cleanup/provider_network")
                .and_then(Value::as_str)
                .unwrap_or("-")
                .to_string(),
            git_summary: record
                .get("git")
                .map(format_git_summary)
                .unwrap_or_else(|| "Git: unavailable (missing)".to_string()),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiagnosticEgressEntry {
    pub(crate) timestamp: String,
    pub(crate) profile: String,
    pub(crate) decision: String,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) count: String,
    pub(crate) reason: String,
    pub(crate) run_id: String,
}

impl DiagnosticEgressEntry {
    fn from_record(record: &Value) -> Option<Self> {
        Some(Self {
            timestamp: value_str(record, "timestamp"),
            profile: value_str(record, "profile"),
            decision: value_str(record, "decision"),
            host: record.get("host").and_then(Value::as_str)?.to_string(),
            port: value_json_string(record, "port", "?"),
            count: value_json_string(record, "count", "1"),
            reason: value_str(record, "reason"),
            run_id: value_str(record, "run_id"),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiagnosticAuthEntry {
    pub(crate) timestamp: String,
    pub(crate) profile: String,
    pub(crate) broker: String,
    pub(crate) decision: String,
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) upstream_status: String,
    pub(crate) count: String,
    pub(crate) reason: String,
    pub(crate) run_id: String,
}

impl DiagnosticAuthEntry {
    fn from_record(record: &Value) -> Self {
        Self {
            timestamp: value_str(record, "timestamp"),
            profile: value_str(record, "profile"),
            broker: value_str(record, "broker"),
            decision: value_str(record, "decision"),
            method: value_str(record, "method"),
            path: value_str(record, "path"),
            upstream_status: value_json_string(record, "upstream_status", "-"),
            count: value_json_string(record, "count", "1"),
            reason: value_str(record, "reason"),
            run_id: value_str(record, "run_id"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthProfileSummary {
    pub(crate) agent: String,
    pub(crate) broker: String,
    pub(crate) status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthStatusSummary {
    pub(crate) status: String,
    pub(crate) runtime: String,
    pub(crate) credential_stores_inspected: bool,
    pub(crate) environment_values_inspected: bool,
    pub(crate) secrets_printed: bool,
    pub(crate) profiles: Vec<AuthProfileSummary>,
}

impl AuthStatusSummary {
    fn from_payload(payload: &Value) -> Self {
        let profiles = payload
            .get("profiles")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|profile| AuthProfileSummary {
                agent: value_str(profile, "agent"),
                broker: value_str(profile, "broker"),
                status: value_str(profile, "status"),
            })
            .collect();
        Self {
            status: value_str(payload, "status"),
            runtime: value_str(payload, "runtime"),
            credential_stores_inspected: value_bool(payload, "credential_stores_inspected"),
            environment_values_inspected: value_bool(payload, "environment_values_inspected"),
            secrets_printed: value_bool(payload, "secrets_printed"),
            profiles,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalProbe {
    pub(crate) color: String,
    pub(crate) motion: String,
    pub(crate) line_mode: String,
    pub(crate) pet_image: String,
}

impl TerminalProbe {
    pub(crate) fn from_settings(settings: TuiSettings, protocol: Option<ImageProtocol>) -> Self {
        Self {
            color: if settings.color_enabled {
                "enabled".to_string()
            } else {
                "disabled".to_string()
            },
            motion: match settings.motion_mode {
                MotionMode::Animated => "animated".to_string(),
                MotionMode::Reduced => "reduced".to_string(),
            },
            line_mode: if settings.line_mode {
                "enabled".to_string()
            } else {
                "disabled".to_string()
            },
            pet_image: protocol
                .map(image_protocol_label)
                .unwrap_or("portable half-block fallback")
                .to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DoctorCheckSummary {
    pub(crate) name: String,
    pub(crate) ok: bool,
    pub(crate) detail: String,
    pub(crate) remedy: String,
}

impl From<Check> for DoctorCheckSummary {
    fn from(check: Check) -> Self {
        Self {
            name: check.name,
            ok: check.ok,
            detail: check.detail,
            remedy: check.remedy,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct HistoryDetailState {
    pub(crate) diff_text: Option<String>,
    pub(crate) diff_error: Option<String>,
    pub(crate) scroll: usize,
}

impl HistoryDetailState {
    pub(crate) fn set_diff(&mut self, text: String) {
        self.diff_text = Some(text);
        self.diff_error = None;
        self.scroll = 0;
    }

    pub(crate) fn set_error(&mut self, error: impl ToString) {
        self.diff_text = None;
        self.diff_error = Some(error.to_string());
        self.scroll = 0;
    }

    pub(crate) fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub(crate) fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }
}

#[derive(Debug)]
pub(crate) struct DiagnosticsState {
    pub(crate) egress: Vec<DiagnosticEgressEntry>,
    pub(crate) egress_error: Option<String>,
    pub(crate) auth: Vec<DiagnosticAuthEntry>,
    pub(crate) auth_error: Option<String>,
    pub(crate) auth_status: AuthStatusSummary,
    pub(crate) terminal: TerminalProbe,
}

impl DiagnosticsState {
    fn new(settings: TuiSettings, protocol: Option<ImageProtocol>) -> Self {
        Self {
            egress: Vec::new(),
            egress_error: None,
            auth: Vec::new(),
            auth_error: None,
            auth_status: AuthStatusSummary::from_payload(&auth_status_payload()),
            terminal: TerminalProbe::from_settings(settings, protocol),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct DoctorState {
    pub(crate) checks: Vec<DoctorCheckSummary>,
    pub(crate) error: Option<String>,
}

#[derive(Debug)]
pub(crate) struct HistoryState {
    pub(crate) records: Vec<RunHistorySummary>,
    pub(crate) selected: usize,
    pub(crate) error: Option<String>,
    pub(crate) detail: HistoryDetailState,
    pub(crate) diagnostics: DiagnosticsState,
    pub(crate) doctor: DoctorState,
}

impl HistoryState {
    pub(crate) fn new(settings: TuiSettings, protocol: Option<ImageProtocol>) -> Self {
        Self {
            records: Vec::new(),
            selected: 0,
            error: None,
            detail: HistoryDetailState::default(),
            diagnostics: DiagnosticsState::new(settings, protocol),
            doctor: DoctorState::default(),
        }
    }

    pub(crate) fn refresh_records(&mut self) {
        let previous = self.selected_run_id().map(ToOwned::to_owned);
        match read_run_records(HISTORY_LIMIT) {
            Ok(records) => {
                self.records = records
                    .iter()
                    .rev()
                    .filter_map(RunHistorySummary::from_record)
                    .collect();
                if let Some(previous) = previous
                    && let Some(index) = self.records.iter().position(|run| run.run_id == previous)
                {
                    self.selected = index;
                } else {
                    self.selected = self.selected.min(self.records.len().saturating_sub(1));
                }
                self.error = None;
            }
            Err(error) => {
                self.records.clear();
                self.selected = 0;
                self.error = Some(error.to_string());
            }
        }
    }

    pub(crate) fn select_next(&mut self) {
        if !self.records.is_empty() {
            self.selected = (self.selected + 1).min(self.records.len() - 1);
        }
    }

    pub(crate) fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub(crate) fn selected_record(&self) -> Option<&RunHistorySummary> {
        self.records.get(self.selected)
    }

    pub(crate) fn selected_run_id(&self) -> Option<&str> {
        self.selected_record().map(|record| record.run_id.as_str())
    }

    pub(crate) fn refresh_selected_diff(&mut self) {
        let Some(run_id) = self.selected_run_id().map(ToOwned::to_owned) else {
            self.detail.set_error("No run record selected.");
            return;
        };
        match run_diff_text(&run_id) {
            Ok(text) => self.detail.set_diff(text),
            Err(error) => self.detail.set_error(error),
        }
    }

    pub(crate) fn refresh_diagnostics(
        &mut self,
        settings: TuiSettings,
        protocol: Option<ImageProtocol>,
    ) {
        match read_egress_policy_log(DIAGNOSTIC_LOG_LIMIT) {
            Ok(entries) => {
                self.diagnostics.egress = entries
                    .iter()
                    .rev()
                    .filter_map(DiagnosticEgressEntry::from_record)
                    .collect();
                self.diagnostics.egress_error = None;
            }
            Err(error) => {
                self.diagnostics.egress.clear();
                self.diagnostics.egress_error = Some(error.to_string());
            }
        }
        match read_auth_broker_log(DIAGNOSTIC_LOG_LIMIT) {
            Ok(entries) => {
                self.diagnostics.auth = entries
                    .iter()
                    .rev()
                    .map(DiagnosticAuthEntry::from_record)
                    .collect();
                self.diagnostics.auth_error = None;
            }
            Err(error) => {
                self.diagnostics.auth.clear();
                self.diagnostics.auth_error = Some(error.to_string());
            }
        }
        self.diagnostics.auth_status = AuthStatusSummary::from_payload(&auth_status_payload());
        self.diagnostics.terminal = TerminalProbe::from_settings(settings, protocol);
    }

    pub(crate) fn refresh_doctor(&mut self) {
        let result = std::panic::catch_unwind(collect_checks);
        match result {
            Ok(checks) => {
                self.doctor.checks = checks.into_iter().map(DoctorCheckSummary::from).collect();
                self.doctor.error = None;
            }
            Err(_) => {
                self.doctor.checks.clear();
                self.doctor.error = Some("doctor checks panicked before completing".to_string());
            }
        }
    }
}

pub(crate) fn visible_diff_lines(detail: &HistoryDetailState, height: u16) -> Vec<String> {
    let Some(text) = detail.diff_text.as_deref() else {
        return Vec::new();
    };
    let lines = text.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
    let height = usize::from(height.max(1));
    let max_start = lines.len().saturating_sub(height);
    let start = detail.scroll.min(max_start);
    lines.into_iter().skip(start).take(height).collect()
}

pub(crate) fn doctor_summary(checks: &[DoctorCheckSummary]) -> String {
    let failed = checks.iter().filter(|check| !check.ok).count();
    if checks.is_empty() {
        return "not checked".to_string();
    }
    if failed == 0 {
        format!("{} checks passing", checks.len())
    } else {
        format!("{failed}/{} checks need attention", checks.len())
    }
}

fn image_protocol_label(protocol: ImageProtocol) -> &'static str {
    match protocol {
        ImageProtocol::Kitty => "kitty graphics",
        ImageProtocol::KittyLocalFile => "kitty local-file graphics",
        ImageProtocol::Sixel => "sixel graphics",
    }
}

fn value_str(record: &Value, key: &str) -> String {
    record
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string()
}

fn value_json_string(record: &Value, key: &str, fallback: &str) -> String {
    record
        .get(key)
        .map(Value::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn value_bool(record: &Value, key: &str) -> bool {
    record.get(key).and_then(Value::as_bool).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn history_summary_uses_safe_record_fields() {
        let summary = RunHistorySummary::from_record(&json!({
            "run_id": "abc",
            "profile": "codex",
            "workspace": "/Users/example/project",
            "network": "provider",
            "status": "succeeded",
            "return_code": 0,
            "timestamp": "2026-06-27T00:00:00Z",
            "provider_policy": {"denied": 2},
            "auth_broker": {"denied": 1},
            "cleanup": {"provider_network": "removed"},
            "git": {"available": "false", "reason": "not-a-git-worktree"}
        }))
        .expect("summary");

        assert_eq!(summary.run_id, "abc");
        assert_eq!(summary.provider_denied, 2);
        assert_eq!(summary.auth_denied, 1);
        assert!(summary.git_summary.contains("not-a-git-worktree"));
    }

    #[test]
    fn diagnostics_entries_keep_only_metadata_fields() {
        let egress = DiagnosticEgressEntry::from_record(&json!({
            "timestamp": "now",
            "profile": "codex",
            "decision": "denied",
            "host": "example.invalid",
            "port": 443,
            "count": 3,
            "reason": "not-in-allowlist",
            "run_id": "abc"
        }))
        .expect("egress");
        assert_eq!(egress.host, "example.invalid");
        assert_eq!(egress.port, "443");
        assert_eq!(egress.count, "3");

        let auth = DiagnosticAuthEntry::from_record(&json!({
            "timestamp": "now",
            "profile": "claude",
            "broker": "api-key",
            "decision": "allowed",
            "method": "POST",
            "path": "/v1/messages",
            "upstream_status": 200,
            "count": 1,
            "reason": "profile-match",
            "run_id": "abc",
            "authorization": "Bearer secret"
        }));
        assert_eq!(auth.path, "/v1/messages");
        assert_eq!(auth.upstream_status, "200");
    }

    #[test]
    fn terminal_probe_reports_fallback_without_graphics_protocol() {
        let settings = TuiSettings::default();
        let probe = TerminalProbe::from_settings(settings, None);

        assert_eq!(probe.color, "enabled");
        assert_eq!(probe.pet_image, "portable half-block fallback");
    }

    #[test]
    fn doctor_summary_counts_failures() {
        let checks = vec![
            DoctorCheckSummary {
                name: "one".to_string(),
                ok: true,
                detail: "ok".to_string(),
                remedy: "none".to_string(),
            },
            DoctorCheckSummary {
                name: "two".to_string(),
                ok: false,
                detail: "bad".to_string(),
                remedy: "fix".to_string(),
            },
        ];

        assert_eq!(doctor_summary(&checks), "1/2 checks need attention");
    }
}
