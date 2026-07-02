//! Claude Keychain sign-in commands (design C, Section 4, Slice 4).
//!
//! Three commands behind one isolated, main-window-only capability let the
//! WebView drive the host Claude token custody surface WITHOUT ever seeing the
//! token. The trust boundary is strict:
//!
//! - The WebView is untrusted. No response carries a token, a token prefix, or
//!   any token-derived value; responses hold only ids, a sanitized status enum
//!   string, and fixed, path-free strings.
//! - Every mutation (`start_claude_oauth_signin`, `clear_claude_oauth`) is gated
//!   by a Rust-owned native confirmation naming Claude explicitly. On cancel,
//!   nothing runs.
//! - `get_claude_oauth_status` derives its enum from Keychain item PRESENCE plus
//!   the in-flight sign-in phase, never from the token. Presence is checked with
//!   an attributes-only query (`token_present`) that does not decrypt the secret,
//!   so polling status never triggers the user-presence prompt and never reads
//!   the token. It never calls `read_token`.
//!
//! The capture itself runs on the host (never in the sandbox) on a background
//! thread; the WebView polls a sanitized status by the returned sign-in id.

use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use runhaven_core::runtime::launch::new_run_id;

use super::dialog::confirm_native;
use crate::claude_oauth_signin::{
    ClaudeOAuthSignInRegistry, InFlightSnapshot, spawn_claude_oauth_capture,
};
use crate::contracts::{
    ClaudeOAuthClearRequest, ClaudeOAuthClearResponse, ClaudeOAuthStartRequest,
    ClaudeOAuthStartResponse, ClaudeOAuthStatusResponse,
};

const STATUS_NOT_SIGNED_IN: &str = "not_signed_in";
const STATUS_SIGNING: &str = "signing";
const STATUS_SIGNED_IN: &str = "signed_in";
const STATUS_FAILED: &str = "failed";

const SIGN_IN_CANCELED: &str = "Claude sign-in canceled before it started.";
const CLEAR_CANCELED: &str = "Nothing was removed.";
const START_MESSAGE: &str = "RunHaven will open Claude sign-in on your Mac and save the token to \
     your Keychain. The token never goes inside the sandbox. Continue?";
const CLEAR_MESSAGE: &str = "Remove the Claude token RunHaven saved on your Mac? This deletes only \
     RunHaven's own Keychain item, not your other logins. It does not revoke the token on \
     Anthropic's side.";

/// A clear outcome, kept local so the command layer compiles on every platform
/// while the Keychain-touching seam stays macOS-gated.
pub(crate) enum ClearOutcome {
    Cleared,
    AlreadyAbsent,
}

/// Non-secret presence plus the token's age, as the status command sees it.
/// Defined here (not gated) so `build_status` and the test seam compile on every
/// platform; the macOS seam fills it from the Keychain's attributes-only query
/// (no token read, no prompt), and non-macOS reports absent.
pub(crate) struct KeychainPresence {
    /// Whether RunHaven's Keychain item exists.
    pub present: bool,
    /// Whole days since the token was saved, from the item's own non-secret
    /// creation date, or `None` when absent or the date could not be parsed.
    pub age_days: Option<u32>,
}

// -- start ----------------------------------------------------------------

#[tauri::command]
pub(crate) async fn start_claude_oauth_signin(
    app: AppHandle,
    request: ClaudeOAuthStartRequest,
    claude_oauth: State<'_, ClaudeOAuthSignInRegistry>,
) -> Result<ClaudeOAuthStartResponse, String> {
    let registry = claude_oauth.inner().clone();
    authorize_start(&request)?;
    if !trusted_start_confirmed(app).await? {
        return Err(SIGN_IN_CANCELED.to_string());
    }
    start_signin_flow(&registry, host_capture)
}

fn authorize_start(request: &ClaudeOAuthStartRequest) -> Result<(), String> {
    if !request.confirm {
        return Err("Confirm before RunHaven starts Claude sign-in.".to_string());
    }
    Ok(())
}

async fn trusted_start_confirmed(app: AppHandle) -> Result<bool, String> {
    confirm_native(
        app,
        "Confirm Claude sign-in",
        "Start Claude sign-in",
        MessageDialogKind::Info,
        START_MESSAGE.to_string(),
    )
    .await
}

/// Mint an id, claim the singleton in-flight slot (fails closed if a sign-in is
/// already running), and spawn the host capture. Returns the id the WebView
/// polls. No token is returned; the capture stores it only in the Keychain.
fn start_signin_flow(
    registry: &ClaudeOAuthSignInRegistry,
    capture: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Result<ClaudeOAuthStartResponse, String> {
    let sign_in_id = new_run_id();
    registry.begin(&sign_in_id)?;
    if let Err(error) = spawn_claude_oauth_capture(sign_in_id.clone(), registry.clone(), capture) {
        let _ = registry.mark_failed(
            &sign_in_id,
            "RunHaven could not start Claude sign-in.".to_string(),
        );
        return Err(error);
    }
    Ok(ClaudeOAuthStartResponse {
        sign_in_id,
        state: STATUS_SIGNING.to_string(),
        label: "Signing in".to_string(),
        detail: "RunHaven started Claude sign-in on your Mac. Finish in your browser, then \
                 approve saving the token."
            .to_string(),
    })
}

/// The production capture: run the host `claude setup-token` and store the token
/// to the Keychain, collapsing any error to a fixed, path-free reason so no host
/// path or captured buffer can reach the polled status.
#[cfg(target_os = "macos")]
fn host_capture() -> Result<(), String> {
    runhaven_core::provider::claude_oauth_keychain::capture_and_store_setup_token()
        .map_err(|_| "Claude sign-in did not finish; nothing was saved.".to_string())
}

#[cfg(not(target_os = "macos"))]
fn host_capture() -> Result<(), String> {
    Err("Claude Keychain sign-in is only available on macOS.".to_string())
}

// -- status ---------------------------------------------------------------

#[tauri::command]
pub(crate) fn get_claude_oauth_status(
    claude_oauth: State<'_, ClaudeOAuthSignInRegistry>,
) -> Result<ClaudeOAuthStatusResponse, String> {
    build_status(claude_oauth.snapshot()?, keychain_presence)
}

/// Derive the sanitized status from the in-flight phase and Keychain presence.
/// While signing, presence is not even checked. Presence (and the non-secret
/// age) is queried only through `presence`, which never reads the token. The age
/// rides along ONLY when signed in, so `not_signed_in`/`failed`/`signing` never
/// carries a day count.
fn build_status(
    snapshot: InFlightSnapshot,
    presence: impl FnOnce() -> Result<KeychainPresence, String>,
) -> Result<ClaudeOAuthStatusResponse, String> {
    match snapshot {
        InFlightSnapshot::Signing { sign_in_id } => Ok(status_response(
            STATUS_SIGNING,
            None,
            Some(sign_in_id),
            None,
        )),
        InFlightSnapshot::Failed { sign_in_id, reason } => {
            let presence = presence()?;
            if presence.present {
                // A token exists despite a prior failed attempt: presence wins.
                Ok(status_response(
                    STATUS_SIGNED_IN,
                    None,
                    Some(sign_in_id),
                    presence.age_days,
                ))
            } else {
                Ok(status_response(
                    STATUS_FAILED,
                    Some(reason),
                    Some(sign_in_id),
                    None,
                ))
            }
        }
        InFlightSnapshot::Idle => {
            let presence = presence()?;
            if presence.present {
                Ok(status_response(
                    STATUS_SIGNED_IN,
                    None,
                    None,
                    presence.age_days,
                ))
            } else {
                Ok(status_response(STATUS_NOT_SIGNED_IN, None, None, None))
            }
        }
    }
}

fn status_response(
    status: &str,
    reason: Option<String>,
    sign_in_id: Option<String>,
    age_days: Option<u32>,
) -> ClaudeOAuthStatusResponse {
    ClaudeOAuthStatusResponse {
        status: status.to_string(),
        reason,
        sign_in_id,
        age_days,
    }
}

/// Presence-and-age check: attributes only, never the secret data, so no
/// user-presence prompt fires and the token is never read. The age is the item's
/// own non-secret creation date. Non-macOS has no Keychain broker, so it reports
/// not-signed-in with no age.
#[cfg(target_os = "macos")]
fn keychain_presence() -> Result<KeychainPresence, String> {
    let presence = runhaven_core::provider::claude_oauth_keychain::token_presence()
        .map_err(|_| "RunHaven could not check the Claude token in the Keychain.".to_string())?;
    Ok(KeychainPresence {
        present: presence.present,
        age_days: presence.age_days,
    })
}

#[cfg(not(target_os = "macos"))]
fn keychain_presence() -> Result<KeychainPresence, String> {
    Ok(KeychainPresence {
        present: false,
        age_days: None,
    })
}

// -- clear ----------------------------------------------------------------

#[tauri::command]
pub(crate) async fn clear_claude_oauth(
    app: AppHandle,
    request: ClaudeOAuthClearRequest,
) -> Result<ClaudeOAuthClearResponse, String> {
    authorize_clear(&request)?;
    if !trusted_clear_confirmed(app).await? {
        return Err(CLEAR_CANCELED.to_string());
    }
    run_clear(host_clear)
}

fn authorize_clear(request: &ClaudeOAuthClearRequest) -> Result<(), String> {
    if !request.confirm {
        return Err("Confirm before RunHaven removes the Claude token.".to_string());
    }
    Ok(())
}

async fn trusted_clear_confirmed(app: AppHandle) -> Result<bool, String> {
    confirm_native(
        app,
        "Remove the saved Claude token",
        "Remove token",
        MessageDialogKind::Warning,
        CLEAR_MESSAGE.to_string(),
    )
    .await
}

fn run_clear(
    clear: impl FnOnce() -> Result<ClearOutcome, String>,
) -> Result<ClaudeOAuthClearResponse, String> {
    match clear()? {
        ClearOutcome::Cleared => Ok(ClaudeOAuthClearResponse {
            outcome: "cleared".to_string(),
            detail: "RunHaven removed the Claude token it saved.".to_string(),
        }),
        ClearOutcome::AlreadyAbsent => Ok(ClaudeOAuthClearResponse {
            outcome: "already_absent".to_string(),
            detail: "There was no Claude token saved by RunHaven.".to_string(),
        }),
    }
}

/// Delete only RunHaven's own Keychain item (Slice 1's exact service + account
/// delete). Errors collapse to a fixed, path-free message.
#[cfg(target_os = "macos")]
fn host_clear() -> Result<ClearOutcome, String> {
    use runhaven_core::provider::claude_oauth_keychain::{KeychainClear, clear_token};
    match clear_token().map_err(|_| "RunHaven could not remove the Claude token.".to_string())? {
        KeychainClear::Cleared => Ok(ClearOutcome::Cleared),
        KeychainClear::Missing => Ok(ClearOutcome::AlreadyAbsent),
    }
}

#[cfg(not(target_os = "macos"))]
fn host_clear() -> Result<ClearOutcome, String> {
    Ok(ClearOutcome::AlreadyAbsent)
}

// -- test seams -----------------------------------------------------------

#[cfg(test)]
pub(crate) fn start_claude_oauth_signin_for_test(
    request: ClaudeOAuthStartRequest,
    registry: &ClaudeOAuthSignInRegistry,
    approve: impl FnOnce() -> Result<bool, String>,
    capture: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Result<ClaudeOAuthStartResponse, String> {
    authorize_start(&request)?;
    if !approve()? {
        return Err(SIGN_IN_CANCELED.to_string());
    }
    start_signin_flow(registry, capture)
}

#[cfg(test)]
pub(crate) fn get_claude_oauth_status_for_test(
    registry: &ClaudeOAuthSignInRegistry,
    presence: impl FnOnce() -> Result<KeychainPresence, String>,
) -> Result<ClaudeOAuthStatusResponse, String> {
    build_status(registry.snapshot()?, presence)
}

#[cfg(test)]
pub(crate) fn clear_claude_oauth_for_test(
    request: ClaudeOAuthClearRequest,
    approve: impl FnOnce() -> Result<bool, String>,
    clear: impl FnOnce() -> Result<ClearOutcome, String>,
) -> Result<ClaudeOAuthClearResponse, String> {
    authorize_clear(&request)?;
    if !approve()? {
        return Err(CLEAR_CANCELED.to_string());
    }
    run_clear(clear)
}
