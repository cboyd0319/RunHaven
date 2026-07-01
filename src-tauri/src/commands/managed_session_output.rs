use runhaven_core::ui_contracts::ManagedSessionSummaryData;
#[cfg(not(test))]
use tauri::{AppHandle, State};

use crate::contracts::{
    LogSnapshotEntry, LogSnapshotRequest, LogSnapshotResponse, ManagedSessionLogSnapshotRequest,
    ManagedSessionLogSnapshotResponse, SessionLogEntry, SessionLogRedaction,
};
use crate::session_output::SessionOutputRegistry;

const RAW_OUTPUT_REDACTION_MESSAGE: &str = "Raw output is not automatically redacted.";

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
    if !super::log_snapshot::trusted_sensitive_output_approved(
        app,
        "Confirm raw output view",
        trusted_session_output_message(&prepared.session, prepared.lines),
    )
    .await?
    {
        return Err(super::log_snapshot::raw_output_canceled_error());
    }
    read_prepared_managed_session_log_snapshot(prepared, Some(&output_registry))
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
    let PreparedManagedSessionLogSnapshot {
        session,
        run_id,
        lines,
    } = prepared;
    let container_snapshot = super::log_snapshot::read_log_snapshot(&run_id, lines);
    let snapshot = match container_snapshot {
        Ok(snapshot) => snapshot,
        Err(container_error) => {
            if let Some(output_registry) = output_registry {
                if let Some(snapshot) = output_registry.snapshot(&session.run_id, lines)? {
                    snapshot
                } else {
                    return Err(container_error);
                }
            } else {
                return Err(container_error);
            }
        }
    };
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
    if !approve(&prepared.session, prepared.lines)? {
        return Err(super::log_snapshot::raw_output_canceled_error());
    }
    read_prepared_managed_session_log_snapshot(prepared, output_registry)
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

fn managed_session_log_snapshot_response(
    session: ManagedSessionSummaryData,
    snapshot: LogSnapshotResponse,
) -> ManagedSessionLogSnapshotResponse {
    let entries = snapshot
        .entries
        .iter()
        .map(|entry| session_log_entry(&session, &snapshot, entry))
        .collect();

    ManagedSessionLogSnapshotResponse {
        session,
        cursor: snapshot.cursor.clone(),
        entries,
        snapshot,
    }
}

fn session_log_entry(
    session: &ManagedSessionSummaryData,
    snapshot: &LogSnapshotResponse,
    entry: &LogSnapshotEntry,
) -> SessionLogEntry {
    SessionLogEntry {
        session_id: session.id.clone(),
        run_id: session.run_id.clone(),
        source: entry.source.clone(),
        cursor: snapshot.cursor.clone(),
        timestamp: entry.timestamp.clone(),
        text: entry.text.clone(),
        line_count: entry.line_count,
        redaction: SessionLogRedaction {
            status: "notApplied".to_string(),
            message: RAW_OUTPUT_REDACTION_MESSAGE.to_string(),
        },
    }
}

#[cfg(not(test))]
fn trusted_session_output_message(session: &ManagedSessionSummaryData, lines: u32) -> String {
    format!(
        "Session: {}\nSession ID: {}\nStatus: {}\nLines: {lines}\n\nRaw output may include secrets. Show output?",
        session.title, session.id, session.status_label
    )
}
