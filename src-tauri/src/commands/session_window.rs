//! Open and identify per-session monitor windows.
//!
//! `open_session_window` is main-window-only (its own one-permission
//! `session-window-open` capability) and creates a read-only monitor window
//! for one existing managed session. It mutates nothing: no run state, no
//! records, no files. Because it is benign, it needs no native confirmation
//! dialog; it is still fully validated in Rust, capped, and never trusts the
//! WebView for a window label.
//!
//! `get_session_window_target` is monitor-window-only (the `session-monitor`
//! capability). It answers "which session does this window show" from the
//! Rust-side registry, keyed by the calling window's label as supplied by
//! Tauri's invocation context. The WebView sends no arguments, so a monitor
//! window cannot claim to be a different window.

#[cfg(not(test))]
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

use runhaven_core::ui_contracts::ManagedSessionSummaryData;

use crate::contracts::{
    OpenSessionWindowRequest, OpenSessionWindowResponse, SessionWindowTargetResponse,
};
use crate::registry_support::validate_registry_id;
use crate::session_windows::SessionWindowRegistry;
#[cfg(not(test))]
use crate::session_windows::SessionWindowSlot;

/// Fixed monitor URL: the constant `monitor=1` flag is the only query data,
/// so no user-influenced byte ever reaches a window URL. The monitor WebView
/// learns its session through `get_session_window_target`, not the URL.
#[cfg(not(test))]
const MONITOR_WEBVIEW_PATH: &str = "index.html?monitor=1";

#[cfg(not(test))]
const MONITOR_WINDOW_SIZE: (f64, f64) = (760.0, 640.0);
#[cfg(not(test))]
const MONITOR_WINDOW_MIN_SIZE: (f64, f64) = (560.0, 480.0);

#[cfg(not(test))]
#[tauri::command]
pub(crate) async fn open_session_window(
    app: AppHandle,
    request: OpenSessionWindowRequest,
    registry: State<'_, SessionWindowRegistry>,
) -> Result<OpenSessionWindowResponse, String> {
    let session = monitor_session(&request.session_id)?;
    // Serialize reserve-then-build: a duplicate request racing an unfinished
    // build would otherwise prune the fresh reservation (its window does not
    // exist yet) and open a second window for the same session.
    let _open = registry.serialize_opens()?;
    let slot = registry.reserve(&session.id, |label| app.get_webview_window(label).is_some())?;
    match slot {
        SessionWindowSlot::FocusExisting(label) => {
            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.set_focus();
            }
            Ok(OpenSessionWindowResponse {
                window_label: label,
                focused_existing: true,
            })
        }
        SessionWindowSlot::Create(label) => {
            // The window title is the session title (agent plus project
            // label), never a filesystem path; project_label reduces the
            // workspace to its final component when the record is built.
            let build = WebviewWindowBuilder::new(
                &app,
                &label,
                WebviewUrl::App(MONITOR_WEBVIEW_PATH.into()),
            )
            .title(&session.title)
            .inner_size(MONITOR_WINDOW_SIZE.0, MONITOR_WINDOW_SIZE.1)
            .min_inner_size(MONITOR_WINDOW_MIN_SIZE.0, MONITOR_WINDOW_MIN_SIZE.1)
            .build();
            if let Err(error) = build {
                registry.release(&label);
                return Err(format!(
                    "RunHaven could not open the session window: {error}"
                ));
            }
            Ok(OpenSessionWindowResponse {
                window_label: label,
                focused_existing: false,
            })
        }
    }
}

/// Test-config command so `generate_handler!` compiles under `cfg(test)`;
/// window creation itself needs a live Tauri app and is exercised by the
/// shared `tauri dev` smoke. Validation is shared with production through
/// [`monitor_session`].
#[cfg(test)]
#[tauri::command]
pub(crate) fn open_session_window(
    request: OpenSessionWindowRequest,
) -> Result<OpenSessionWindowResponse, String> {
    let _session = monitor_session(&request.session_id)?;
    Ok(OpenSessionWindowResponse {
        window_label: String::new(),
        focused_existing: false,
    })
}

#[cfg(not(test))]
#[tauri::command]
pub(crate) fn get_session_window_target(
    window: tauri::Window,
    registry: State<'_, SessionWindowRegistry>,
) -> Result<SessionWindowTargetResponse, String> {
    session_window_target(&registry, window.label())
}

#[cfg(test)]
#[tauri::command]
pub(crate) fn get_session_window_target() -> Result<SessionWindowTargetResponse, String> {
    Err("session window targets resolve from a live monitor window".to_string())
}

fn session_window_target(
    registry: &SessionWindowRegistry,
    window_label: &str,
) -> Result<SessionWindowTargetResponse, String> {
    Ok(SessionWindowTargetResponse {
        session_id: registry.session_for_label(window_label)?,
    })
}

/// Validate a monitor open request: the id must have a sane registry-id shape
/// (no control characters, whitespace, or path separators) and must belong to
/// an existing managed session in the backend read model. The returned
/// summary is backend truth; nothing from the request is used afterwards.
fn monitor_session(session_id: &str) -> Result<ManagedSessionSummaryData, String> {
    validate_registry_id(session_id)?;
    let (session, _run_id) = super::sessions::managed_session_for_id(session_id)?;
    Ok(session)
}

#[cfg(test)]
pub(crate) fn monitor_session_for_test(
    session_id: &str,
) -> Result<ManagedSessionSummaryData, String> {
    monitor_session(session_id)
}

#[cfg(test)]
pub(crate) fn session_window_target_for_test(
    registry: &SessionWindowRegistry,
    window_label: &str,
) -> Result<SessionWindowTargetResponse, String> {
    session_window_target(registry, window_label)
}
