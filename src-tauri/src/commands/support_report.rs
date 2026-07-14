use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

use runhaven_core::support::report::{
    AppLogSource, CreatedSupportReport, SUPPORT_ISSUE_URL, SupportReportRequest,
    create_support_report as create_core_support_report, default_support_reports_dir,
};
use tauri::{AppHandle, Manager, State};
use time::OffsetDateTime;

use crate::contracts::SupportReportResponse;

#[cfg(test)]
use std::path::PathBuf;

pub(crate) mod confirmation;

pub(crate) const OPEN_PROGRAM: &str = "/usr/bin/open";

#[derive(Debug, Default)]
pub(crate) struct SupportReportGate {
    active: Mutex<bool>,
}

#[derive(Debug)]
struct SupportReportLease<'a> {
    gate: &'a SupportReportGate,
}

impl SupportReportGate {
    fn try_begin(&self) -> Result<SupportReportLease<'_>, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "Support report creation is already running.".to_string())?;
        if *active {
            return Err("Support report creation is already running.".to_string());
        }
        *active = true;
        Ok(SupportReportLease { gate: self })
    }
}

impl Drop for SupportReportLease<'_> {
    fn drop(&mut self) {
        match self.gate.active.lock() {
            Ok(mut active) => *active = false,
            Err(poisoned) => *poisoned.into_inner() = false,
        }
    }
}

#[tauri::command]
pub(crate) async fn create_support_report(
    app: AppHandle,
    gate: State<'_, SupportReportGate>,
) -> Result<Option<SupportReportResponse>, String> {
    let _lease = gate.try_begin()?;
    if !confirmation::trusted_support_report_confirmed(app.clone()).await? {
        return Ok(None);
    }
    let app_version = super::desktop_app_version();
    let output_dir = default_support_reports_dir().map_err(|_| {
        "RunHaven could not find a private place for the support report. No apps were opened."
            .to_string()
    })?;
    let app_log_dir = app.path().app_log_dir().ok();
    tauri::async_runtime::spawn_blocking(move || {
        complete_support_report(
            &app_version,
            &output_dir,
            app_log_dir.as_deref(),
            &mut create_core_support_report,
            &mut run_open,
        )
    })
    .await
    .map_err(|_| "RunHaven could not create the support report. No apps were opened.".to_string())?
    .map(Some)
}

#[cfg(test)]
fn perform_support_report_with(
    app_version: &str,
    output_dir: &Path,
    app_log_dir: Option<&Path>,
    mut confirm: impl FnMut() -> Result<bool, String>,
    mut create: impl FnMut(SupportReportRequest<'_>, &Path) -> anyhow::Result<CreatedSupportReport>,
    mut open: impl FnMut(&Path, &[OsString]) -> std::io::Result<bool>,
) -> Result<Option<SupportReportResponse>, String> {
    if !confirm()? {
        return Ok(None);
    }
    complete_support_report(app_version, output_dir, app_log_dir, &mut create, &mut open).map(Some)
}

fn complete_support_report(
    app_version: &str,
    output_dir: &Path,
    app_log_dir: Option<&Path>,
    create: &mut impl FnMut(SupportReportRequest<'_>, &Path) -> anyhow::Result<CreatedSupportReport>,
    open: &mut impl FnMut(&Path, &[OsString]) -> std::io::Result<bool>,
) -> Result<SupportReportResponse, String> {
    let app_log_path = app_log_dir.map(|dir| dir.join("runhaven.log"));
    let app_log = app_log_dir
        .zip(app_log_path.as_deref())
        .map(|(trusted_parent, path)| AppLogSource {
            path,
            trusted_parent,
        });
    let artifact = create(
        SupportReportRequest {
            app_version,
            created_at: OffsetDateTime::now_utc(),
            app_log,
        },
        output_dir,
    )
    .map_err(|_| {
        "RunHaven could not create the support report. No apps were opened.".to_string()
    })?;
    validate_created_report(output_dir, &artifact.path).map_err(|_| {
        "RunHaven could not verify the support report. No apps were opened.".to_string()
    })?;

    let issue_opened = open(
        Path::new(OPEN_PROGRAM),
        &[OsString::from("-u"), OsString::from(SUPPORT_ISSUE_URL)],
    )
    .unwrap_or(false);
    let finder_revealed = open(
        Path::new(OPEN_PROGRAM),
        &[OsString::from("-R"), artifact.path.clone().into_os_string()],
    )
    .unwrap_or(false);
    Ok(SupportReportResponse {
        file_name: artifact.file_name,
        issue_opened,
        finder_revealed,
    })
}

fn validate_created_report(output_dir: &Path, report_path: &Path) -> anyhow::Result<()> {
    let metadata = std::fs::symlink_metadata(report_path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        anyhow::bail!("support report is not a regular file")
    }
    let output_dir = std::fs::canonicalize(output_dir)?;
    let report_path = std::fs::canonicalize(report_path)?;
    if report_path.parent() != Some(output_dir.as_path()) {
        anyhow::bail!("support report escaped its private directory")
    }
    Ok(())
}

fn run_open(program: &Path, args: &[OsString]) -> std::io::Result<bool> {
    Ok(Command::new(program).args(args).status()?.success())
}

#[cfg(test)]
pub(crate) fn perform_support_report_for_test(
    app_version: &str,
    output_dir: &Path,
    app_log_dir: &Path,
    confirm: impl FnMut() -> Result<bool, String>,
    create: impl FnMut(SupportReportRequest<'_>, &Path) -> anyhow::Result<CreatedSupportReport>,
    open: impl FnMut(&Path, &[OsString]) -> std::io::Result<bool>,
) -> Result<Option<SupportReportResponse>, String> {
    perform_support_report_with(
        app_version,
        output_dir,
        Some(app_log_dir),
        confirm,
        create,
        open,
    )
}

#[cfg(test)]
pub(crate) fn fixed_report_for_test(path: PathBuf) -> CreatedSupportReport {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("RunHaven-support.txt")
        .to_string();
    CreatedSupportReport { path, file_name }
}
