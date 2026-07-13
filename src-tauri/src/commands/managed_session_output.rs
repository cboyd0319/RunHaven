use runhaven_core::ui_contracts::ManagedSessionSummaryData;
#[cfg(not(test))]
use tauri::{AppHandle, State};

use crate::contracts::{
    LogSnapshotRequest, LogSnapshotResponse, ManagedSessionLogSnapshotRequest,
    ManagedSessionLogSnapshotResponse,
};
use crate::session_output::SessionOutputRegistry;

const RAW_OUTPUT_REDACTION_MESSAGE: &str =
    "Known secrets and control characters were removed automatically. Review before sharing.";
const PAST_OUTPUT_UNAVAILABLE: &str = "RunHaven keeps session output only until the app closes. Output for this run is no longer available.";

struct PreparedManagedSessionLogSnapshot {
    session: ManagedSessionSummaryData,
    run_id: String,
    lines: u32,
}

#[cfg(not(test))]
#[tauri::command]
pub(crate) async fn get_managed_session_log_snapshot(
    app: AppHandle,
    request: ManagedSessionLogSnapshotRequest,
    output_registry: State<'_, SessionOutputRegistry>,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    let output_registry = output_registry.inner().clone();
    let prepared = prepare_managed_session_log_snapshot(request)?;
    preflight_managed_session_log_snapshot(&prepared, Some(&output_registry))?;
    if !super::log_snapshot::trusted_sensitive_output_approved(
        app,
        super::log_snapshot::RAW_OUTPUT_CONFIRM_TITLE,
        trusted_session_output_message(
            &prepared.session.title,
            &prepared.session.status_label,
            prepared.lines,
        ),
    )
    .await?
    {
        return Err(super::log_snapshot::raw_output_canceled_error());
    }
    read_prepared_managed_session_log_snapshot(prepared, Some(&output_registry))
}

fn preflight_managed_session_log_snapshot(
    prepared: &PreparedManagedSessionLogSnapshot,
    output_registry: Option<&SessionOutputRegistry>,
) -> Result<(), String> {
    if super::sessions::managed_session_allows_run_control(&prepared.session.lifecycle) {
        return Ok(());
    }
    let retained = match output_registry {
        Some(registry) => registry.snapshot(&prepared.session.run_id, prepared.lines)?,
        None => None,
    };
    if retained.is_none() {
        return Err(PAST_OUTPUT_UNAVAILABLE.to_string());
    }
    Ok(())
}

#[cfg(test)]
#[tauri::command]
pub(crate) fn get_managed_session_log_snapshot(
    request: ManagedSessionLogSnapshotRequest,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    get_managed_session_log_snapshot_with_approval_for_test(request, None, |_, _| Ok(true))
}

fn prepare_managed_session_log_snapshot(
    request: ManagedSessionLogSnapshotRequest,
) -> Result<PreparedManagedSessionLogSnapshot, String> {
    let (session, run_id) = super::sessions::managed_session_for_id(&request.session_id)?;
    let snapshot_request = LogSnapshotRequest {
        run_id: run_id.clone(),
        lines: request.lines,
        confirm_sensitive_output: request.confirm_sensitive_output,
    };
    let lines = super::log_snapshot::validated_lines(&snapshot_request)?;
    Ok(PreparedManagedSessionLogSnapshot {
        session,
        run_id,
        lines,
    })
}

fn read_prepared_managed_session_log_snapshot(
    prepared: PreparedManagedSessionLogSnapshot,
    output_registry: Option<&SessionOutputRegistry>,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    read_prepared_managed_session_log_snapshot_with_fallback(
        prepared,
        output_registry,
        super::log_snapshot::read_log_snapshot,
    )
}

fn read_prepared_managed_session_log_snapshot_with_fallback(
    prepared: PreparedManagedSessionLogSnapshot,
    output_registry: Option<&SessionOutputRegistry>,
    fallback: impl FnOnce(&str, u32) -> Result<LogSnapshotResponse, String>,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    let PreparedManagedSessionLogSnapshot {
        session,
        run_id,
        lines,
    } = prepared;
    // Serve from the in-memory streamed output first: it is instant, while the
    // container path spawns the `container logs` CLI (measured 1-3.6s per poll,
    // a real lag on a live log view that the SLOW-invoke WARN surfaced). Both
    // paths bound and redact through the same `log_snapshot_payload_from_stdout`
    // (`SessionOutputRegistry::snapshot` and `read_log_snapshot` alike), so this
    // reorder never widens what a viewer can see. An active session may fall
    // back to the container CLI when the registry has nothing; a past session
    // cannot use that active-only reader and returns a fixed error instead.
    let registry_snapshot = match output_registry {
        Some(registry) => registry.snapshot(&session.run_id, lines)?,
        None => None,
    };
    if let Some(snapshot) = registry_snapshot {
        return Ok(managed_session_log_snapshot_response(session, snapshot));
    }
    if !super::sessions::managed_session_allows_run_control(&session.lifecycle) {
        return Err(PAST_OUTPUT_UNAVAILABLE.to_string());
    }
    let snapshot = fallback(&run_id, lines)?;
    Ok(managed_session_log_snapshot_response(session, snapshot))
}

#[cfg(test)]
pub(crate) fn get_managed_session_log_snapshot_for_test(
    request: ManagedSessionLogSnapshotRequest,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    get_managed_session_log_snapshot_with_approval_for_test(request, None, |_, _| Ok(true))
}

#[cfg(test)]
pub(crate) fn get_managed_session_log_snapshot_with_output_for_test(
    request: ManagedSessionLogSnapshotRequest,
    output_registry: &SessionOutputRegistry,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    get_managed_session_log_snapshot_with_approval_for_test(
        request,
        Some(output_registry),
        |_, _| Ok(true),
    )
}

#[cfg(test)]
pub(crate) fn get_managed_session_log_snapshot_with_approval_for_test(
    request: ManagedSessionLogSnapshotRequest,
    output_registry: Option<&SessionOutputRegistry>,
    approve: impl FnOnce(&ManagedSessionSummaryData, u32) -> Result<bool, String>,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    let prepared = prepare_managed_session_log_snapshot(request)?;
    preflight_managed_session_log_snapshot(&prepared, output_registry)?;
    if !approve(&prepared.session, prepared.lines)? {
        return Err(super::log_snapshot::raw_output_canceled_error());
    }
    read_prepared_managed_session_log_snapshot(prepared, output_registry)
}

#[cfg(test)]
pub(crate) fn get_managed_session_log_snapshot_with_fallback_for_test(
    request: ManagedSessionLogSnapshotRequest,
    output_registry: Option<&SessionOutputRegistry>,
    approve: impl FnOnce(&ManagedSessionSummaryData, u32) -> Result<bool, String>,
    fallback: impl FnOnce(&str, u32) -> Result<LogSnapshotResponse, String>,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    let prepared = prepare_managed_session_log_snapshot(request)?;
    preflight_managed_session_log_snapshot(&prepared, output_registry)?;
    if !approve(&prepared.session, prepared.lines)? {
        return Err(super::log_snapshot::raw_output_canceled_error());
    }
    read_prepared_managed_session_log_snapshot_with_fallback(prepared, output_registry, fallback)
}

#[cfg(test)]
pub(crate) fn managed_session_log_snapshot_response_for_test(
    request: ManagedSessionLogSnapshotRequest,
    stdout: &[u8],
    max_bytes: usize,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    let PreparedManagedSessionLogSnapshot {
        session,
        run_id,
        lines,
    } = prepare_managed_session_log_snapshot(request)?;
    let snapshot = super::log_snapshot::log_snapshot_response(&run_id, lines, stdout, max_bytes)?;
    Ok(managed_session_log_snapshot_response(session, snapshot))
}

/// Wrap the core snapshot for a managed session, adding the redaction notice
/// as a warning so every viewer of session output sees it alongside the
/// bounded, redacted text.
fn managed_session_log_snapshot_response(
    session: ManagedSessionSummaryData,
    mut snapshot: LogSnapshotResponse,
) -> ManagedSessionLogSnapshotResponse {
    snapshot
        .warnings
        .push(RAW_OUTPUT_REDACTION_MESSAGE.to_string());
    ManagedSessionLogSnapshotResponse { session, snapshot }
}

pub(crate) fn trusted_session_output_message(_title: &str, _status: &str, lines: u32) -> String {
    super::log_snapshot::trusted_run_output_message(lines)
}
