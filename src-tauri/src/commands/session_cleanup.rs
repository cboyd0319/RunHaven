use runhaven_core::runtime::worktrees::{
    discard_worktree_for_run, load_worktree_lifecycle, verify_lifecycle,
};
use runhaven_core::ui_contracts::{ManagedSessionCleanupData, ManagedSessionSummaryData};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::contracts::{
    CleanupManagedSessionRequest, CleanupManagedSessionResponse, CleanupManagedSessionResult,
};

const CONFIRM_CLEANUP_ERROR: &str = "Confirm the cleanup before discarding this session worktree.";
const CANCEL_CLEANUP_ERROR: &str = "Cleanup canceled before discarding this session worktree.";
const ACTIVE_CLEANUP_ERROR: &str =
    "Managed session cleanup is only available after this session stops running.";
const HISTORY_CLEANUP_ERROR: &str = "Managed session history is not eligible for cleanup yet.";
const UNSUPPORTED_CLEANUP_ERROR: &str =
    "Managed session cleanup is only available for RunHaven-owned worktree sessions.";
const FAILED_CLEANUP_ERROR: &str =
    "RunHaven could not verify a safe cleanup target for this session.";
const CLEANUP_TARGET_WORKTREE: &str = "worktree";
const CLEANUP_STATUS_DISCARDED: &str = "discarded";

#[derive(Clone, Copy)]
enum ManagedSessionCleanupTarget {
    Worktree,
}

impl ManagedSessionCleanupTarget {
    fn label(self) -> &'static str {
        match self {
            Self::Worktree => "RunHaven worktree",
        }
    }

    fn effect(self) -> &'static str {
        match self {
            Self::Worktree => "delete the recorded worktree and branch for this finished session",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Worktree => CLEANUP_TARGET_WORKTREE,
        }
    }
}

#[tauri::command]
pub(crate) async fn cleanup_managed_session(
    app: AppHandle,
    request: CleanupManagedSessionRequest,
) -> Result<CleanupManagedSessionResponse, String> {
    let (mut session, run_id, target) = validate_cleanup_request(&request)?;
    if !trusted_session_cleanup_confirmed(app, &session, &run_id, target).await? {
        return Err(CANCEL_CLEANUP_ERROR.to_string());
    }
    let result = cleanup_target_checked(&run_id, target)?;
    clear_session_cleanup(&mut session);
    Ok(CleanupManagedSessionResponse { session, result })
}

#[cfg(test)]
pub(crate) fn cleanup_managed_session_for_test(
    request: CleanupManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
) -> Result<CleanupManagedSessionResponse, String> {
    let (mut session, run_id, target) = validate_cleanup_request(&request)?;
    if !approve(&session, &run_id)? {
        return Err(CANCEL_CLEANUP_ERROR.to_string());
    }
    let result = cleanup_target_checked(&run_id, target)?;
    clear_session_cleanup(&mut session);
    Ok(CleanupManagedSessionResponse { session, result })
}

fn validate_cleanup_request(
    request: &CleanupManagedSessionRequest,
) -> Result<
    (
        ManagedSessionSummaryData,
        String,
        ManagedSessionCleanupTarget,
    ),
    String,
> {
    if !request.confirm_cleanup {
        return Err(CONFIRM_CLEANUP_ERROR.to_string());
    }
    let (session, run_id) = super::sessions::managed_session_for_id(&request.session_id)?;
    ensure_no_active_run_marker(&run_id)?;
    let target = resolve_cleanup_target(&session, &run_id)?;
    Ok((session, run_id, target))
}

fn resolve_cleanup_target(
    session: &ManagedSessionSummaryData,
    run_id: &str,
) -> Result<ManagedSessionCleanupTarget, String> {
    match session.lifecycle.trim().to_lowercase().as_str() {
        "finished" | "failed" | "stopped" => {}
        "running" | "starting" | "stop-requested" | "kill-requested" => {
            return Err(ACTIVE_CLEANUP_ERROR.to_string());
        }
        _ => return Err(HISTORY_CLEANUP_ERROR.to_string()),
    }

    let lifecycle = load_worktree_lifecycle(run_id).map_err(cleanup_target_error)?;
    verify_lifecycle(&lifecycle).map_err(cleanup_target_error)?;
    Ok(ManagedSessionCleanupTarget::Worktree)
}

fn cleanup_target_error(error: impl ToString) -> String {
    let message = error.to_string();
    if message.contains("is not a worktree run") {
        UNSUPPORTED_CLEANUP_ERROR.to_string()
    } else {
        FAILED_CLEANUP_ERROR.to_string()
    }
}

fn cleanup_target_checked(
    run_id: &str,
    target: ManagedSessionCleanupTarget,
) -> Result<CleanupManagedSessionResult, String> {
    ensure_no_active_run_marker(run_id)?;
    match target {
        ManagedSessionCleanupTarget::Worktree => {
            discard_worktree_for_run(run_id).map_err(cleanup_target_error)?;
            Ok(CleanupManagedSessionResult {
                run_id: run_id.to_string(),
                target: target.as_str().to_string(),
                status: CLEANUP_STATUS_DISCARDED.to_string(),
            })
        }
    }
}

fn ensure_no_active_run_marker(run_id: &str) -> Result<(), String> {
    match super::active_marker::active_run_marker_present(run_id) {
        Ok(true) => Err(ACTIVE_CLEANUP_ERROR.to_string()),
        Ok(false) => Ok(()),
        Err(()) => Err(FAILED_CLEANUP_ERROR.to_string()),
    }
}

fn clear_session_cleanup(session: &mut ManagedSessionSummaryData) {
    session.cleanup = ManagedSessionCleanupData {
        available: false,
        target: String::new(),
        label: String::new(),
        detail: String::new(),
    };
}

async fn trusted_session_cleanup_confirmed(
    app: AppHandle,
    session: &ManagedSessionSummaryData,
    run_id: &str,
    target: ManagedSessionCleanupTarget,
) -> Result<bool, String> {
    let message = format!(
        "Action: Clean up\nSession: {}\nRun ID: {}\nAgent: {}\nProject: {}\nStatus: {}\nTarget: {}\n\nRunHaven will {} after you confirm.",
        session.title,
        run_id,
        session.agent,
        session.project,
        session.status_label,
        target.label(),
        target.effect(),
    );
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .title("Confirm RunHaven cleanup")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Clean up".to_string(),
                "Cancel".to_string(),
            ))
            .blocking_show()
    })
    .await
    .map_err(|error| format!("could not show trusted cleanup confirmation: {error}"))
}
