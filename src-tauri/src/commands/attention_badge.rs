//! Dock badge command: the count of sessions that need the user, nothing else.
//!
//! The WebView computes the needs-you count with the same attention rules the
//! Home uses (`ui/src/app/sessionAttention.ts`) and reports it here.
//! This is deliberately the narrowest possible surface: one validated integer
//! in, nothing out, and the only effect is the app icon badge. A compromised
//! WebView could at worst draw a wrong number on the Dock icon; absurd counts
//! are rejected so it cannot even do that convincingly. Nothing in this module
//! reads, mutates, or reveals session state.

use tauri::{AppHandle, Manager, Runtime};

use crate::MAIN_WINDOW_LABEL;
use crate::contracts::AttentionBadgeRequest;

/// Highest count RunHaven will show on the Dock. The frontend clamps to this
/// before sending; anything above it is a forged or buggy request, not a
/// workload, so the command rejects it instead of trusting the WebView.
const MAX_ATTENTION_BADGE_COUNT: u32 = 99;

#[tauri::command]
pub(crate) fn set_attention_badge<R: Runtime>(
    app: AppHandle<R>,
    request: AttentionBadgeRequest,
) -> Result<(), String> {
    let badge = badge_value(request.count)?;
    apply_badge(&app, badge)
}

/// Clear the Dock badge at startup so a stale number from a crashed exit never
/// survives into a fresh launch. Best effort: a failure only leaves the badge
/// blank, which is the safe direction.
pub(crate) fn clear_badge_on_start<R: Runtime>(app: &AppHandle<R>) {
    let _ = apply_badge(app, None);
}

/// Map a validated count onto the Dock badge value: zero clears the badge,
/// 1 through [`MAX_ATTENTION_BADGE_COUNT`] shows the number.
fn badge_value(count: u32) -> Result<Option<i64>, String> {
    if count > MAX_ATTENTION_BADGE_COUNT {
        return Err(format!(
            "Attention badge count {count} is out of range (0-{MAX_ATTENTION_BADGE_COUNT})."
        ));
    }
    Ok((count > 0).then_some(i64::from(count)))
}

fn apply_badge<R: Runtime>(app: &AppHandle<R>, badge: Option<i64>) -> Result<(), String> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Err("The main window is not available.".to_string());
    };
    window
        .set_badge_count(badge)
        .map_err(|_| "RunHaven could not update the Dock badge.".to_string())
}

#[cfg(test)]
mod tests {
    use super::{MAX_ATTENTION_BADGE_COUNT, badge_value};

    #[test]
    fn zero_clears_the_badge() {
        assert_eq!(badge_value(0), Ok(None));
    }

    #[test]
    fn counts_in_range_show_as_is() {
        assert_eq!(badge_value(1), Ok(Some(1)));
        assert_eq!(
            badge_value(MAX_ATTENTION_BADGE_COUNT),
            Ok(Some(i64::from(MAX_ATTENTION_BADGE_COUNT)))
        );
    }

    #[test]
    fn absurd_counts_are_rejected_not_clamped() {
        let error = badge_value(MAX_ATTENTION_BADGE_COUNT + 1).unwrap_err();
        assert!(error.contains("out of range"));
        assert!(badge_value(u32::MAX).is_err());
    }
}
