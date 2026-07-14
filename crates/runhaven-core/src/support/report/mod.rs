use std::path::{Path, PathBuf};

use anyhow::Result;
use time::OffsetDateTime;

#[cfg(test)]
use serde_json::Value;

mod app_log;
mod collect;
mod process;
mod render;
mod write;

pub const SUPPORT_ISSUE_URL: &str =
    "https://github.com/cboyd0319/RunHaven/issues/new?template=runhaven-support.yml";
pub const MAX_REPORT_BYTES: usize = 256 * 1024;
pub const MAX_REPORT_EVENTS: usize = 80;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppLogSource<'a> {
    pub path: &'a Path,
    pub trusted_parent: &'a Path,
}

#[derive(Clone, Debug)]
pub struct SupportReportRequest<'a> {
    pub app_version: &'a str,
    pub created_at: OffsetDateTime,
    pub app_log: Option<AppLogSource<'a>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatedSupportReport {
    pub path: PathBuf,
    pub file_name: String,
}

pub fn default_support_reports_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("RunHaven could not find Application Support."))?;
    prepare_default_support_reports_dir(&data_dir)
}

fn prepare_default_support_reports_dir(data_dir: &Path) -> Result<PathBuf> {
    write::prepare_default_support_reports_dir(data_dir)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SupportReportData {
    app_version: String,
    created_at: String,
    system: Vec<Invariant>,
    images: Vec<ImageInvariant>,
    sessions: Vec<SessionSummary>,
    egress: Vec<DecisionSummary>,
    auth: Vec<DecisionSummary>,
    app_events: Vec<AppEvent>,
    omitted_app_log_lines: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Invariant {
    name: String,
    status: String,
    value: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ImageInvariant {
    agent: String,
    image: String,
    status: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SessionSummary {
    timestamp: String,
    profile: String,
    network: String,
    auth_scope: String,
    status: String,
    return_code: i64,
    provider_allowed: u64,
    provider_denied: u64,
    auth_allowed: u64,
    auth_denied: u64,
    cleanup: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DecisionSummary {
    timestamp: String,
    profile: String,
    decision: String,
    count: u64,
    category: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AppEvent {
    timestamp: String,
    component: String,
    action: String,
    outcome: String,
    duration_ms: Option<u64>,
    exit_code: Option<i64>,
}

pub fn create_support_report(
    request: SupportReportRequest<'_>,
    output_dir: &Path,
) -> Result<CreatedSupportReport> {
    let data = collect::collect_report_data(&request);
    let report = render_report(&data);
    write::write_report(output_dir, request.app_version, request.created_at, &report)
}

#[cfg(test)]
fn project_run_record(record: &Value) -> Option<SessionSummary> {
    collect::project_run_record(record)
}

#[cfg(test)]
fn project_egress_record(record: &Value) -> Option<DecisionSummary> {
    collect::project_egress_record(record)
}

#[cfg(test)]
fn project_auth_record(record: &Value) -> Option<DecisionSummary> {
    collect::project_auth_record(record)
}

#[cfg(test)]
fn parse_app_log(text: &str) -> (Vec<AppEvent>, usize) {
    app_log::parse_app_log(text)
}

fn render_report(data: &SupportReportData) -> String {
    render::render_report(data)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
