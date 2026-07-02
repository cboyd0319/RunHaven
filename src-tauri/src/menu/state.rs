//! The menu enable-state contract and its id-to-flag mapping.
//!
//! The WebView reports which selection-scoped menu items apply right now, using
//! the same helpers the sidebar and attention badge already compute from
//! (`ui/src/app/useMenuStateSync.ts`). [`MenuState`] carries booleans and one
//! small count only: no session ids, paths, or secrets ever cross this
//! boundary, so a compromised WebView could at worst gray the wrong menu items.
//! `apply_menu_state` in `mod.rs` drives the retained menu item handles through
//! [`menu_item_enabled`]; this module owns the mapping and stays free of any
//! Tauri runtime types so it is testable as pure logic.

use serde::Deserialize;

/// Which selection-scoped menu items apply for the main window's current
/// selection and session list. Every field is a primitive; adding an id, path,
/// or other identifying value here would be a boundary regression, and
/// `menu_state_carries_only_primitive_fields` fails the build if it happens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MenuState {
    /// A real session is selected in the main window.
    pub has_selected_session: bool,
    /// The selected session is in a lifecycle that accepts Stop, Hard Stop, and
    /// Repair (running/starting/stop-requested/kill-requested).
    pub selected_session_can_run_control: bool,
    /// The selected session shows its message composer (monitor next-action).
    pub selected_session_can_send_input: bool,
    /// The selected session shows its output confirm surface.
    pub selected_session_can_view_output: bool,
    /// The selected session offers a standalone clean-up (not the worktree
    /// review, which owns its own removal).
    pub selected_session_has_cleanup: bool,
    /// The selected session can open a read-only monitor window.
    pub selected_session_can_open_window: bool,
    /// At least one session needs the user, so Jump has a target.
    pub has_attention_session: bool,
    /// How many sessions the sidebar lists, for Cmd-1..9 and next/previous.
    pub session_count: u32,
}

/// The enable state for one dynamic menu id under `state`, or `None` when the
/// id is not sync-controlled (a global or placeholder item this path never
/// touches). This is the single source of truth: the retained-handle set is
/// derived from it via [`is_dynamic_menu_id`], and the `spec.rs` initial-state
/// test reads the same function, so the three cannot drift apart.
pub(crate) fn menu_item_enabled(id: &str, state: &MenuState) -> Option<bool> {
    if let Some(index) = id.strip_prefix("select-session-") {
        // Cmd-1..9 select the Nth sidebar row; enabled only when that row
        // exists. A non-numeric suffix is not a select item.
        return index
            .parse::<u32>()
            .ok()
            .map(|n| n >= 1 && n <= state.session_count);
    }
    Some(match id {
        "send-message" => state.selected_session_can_send_input,
        "show-output" => state.selected_session_can_view_output,
        // Stop, Hard Stop, and Repair share the run-control card, so they share
        // one gate: the selected session is in a controllable lifecycle.
        "stop-session" | "hard-stop-session" | "repair-session" => {
            state.selected_session_can_run_control
        }
        "cleanup-session" => state.selected_session_has_cleanup,
        "open-session-window" => state.selected_session_can_open_window,
        "next-session" | "previous-session" => state.session_count >= 1,
        "jump-needs-you" => state.has_attention_session,
        _ => return None,
    })
}

/// Whether `sync_menu_state` drives this id, so its handle must be retained at
/// build. Derived from [`menu_item_enabled`] so it cannot list a different set.
pub(crate) fn is_dynamic_menu_id(id: &str) -> bool {
    menu_item_enabled(id, &MenuState::default()).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `MenuState` where every capability is present and there are many
    /// sessions: the "everything applies" end of the range.
    fn full() -> MenuState {
        MenuState {
            has_selected_session: true,
            selected_session_can_run_control: true,
            selected_session_can_send_input: true,
            selected_session_can_view_output: true,
            selected_session_has_cleanup: true,
            selected_session_can_open_window: true,
            has_attention_session: true,
            session_count: 9,
        }
    }

    #[test]
    fn no_selection_disables_every_session_scoped_item() {
        let state = MenuState::default();
        for id in [
            "send-message",
            "show-output",
            "stop-session",
            "hard-stop-session",
            "repair-session",
            "cleanup-session",
            "open-session-window",
            "next-session",
            "previous-session",
            "jump-needs-you",
            "select-session-1",
            "select-session-9",
        ] {
            assert_eq!(
                menu_item_enabled(id, &state),
                Some(false),
                "`{id}` must be disabled with no selection and no sessions"
            );
        }
    }

    #[test]
    fn run_control_gate_covers_stop_hard_stop_and_repair() {
        let can_control = MenuState {
            selected_session_can_run_control: true,
            ..MenuState::default()
        };
        for id in ["stop-session", "hard-stop-session", "repair-session"] {
            assert_eq!(menu_item_enabled(id, &can_control), Some(true));
        }
        // A different capability does not enable run control.
        let send_only = MenuState {
            selected_session_can_send_input: true,
            ..MenuState::default()
        };
        for id in ["stop-session", "hard-stop-session", "repair-session"] {
            assert_eq!(menu_item_enabled(id, &send_only), Some(false));
        }
    }

    #[test]
    fn each_capability_maps_to_its_own_item() {
        let state = full();
        assert_eq!(menu_item_enabled("send-message", &state), Some(true));
        assert_eq!(menu_item_enabled("show-output", &state), Some(true));
        assert_eq!(menu_item_enabled("cleanup-session", &state), Some(true));
        assert_eq!(menu_item_enabled("open-session-window", &state), Some(true));
        assert_eq!(menu_item_enabled("jump-needs-you", &state), Some(true));
    }

    #[test]
    fn select_session_tracks_the_count() {
        let state = MenuState {
            session_count: 3,
            ..MenuState::default()
        };
        assert_eq!(menu_item_enabled("select-session-1", &state), Some(true));
        assert_eq!(menu_item_enabled("select-session-3", &state), Some(true));
        assert_eq!(menu_item_enabled("select-session-4", &state), Some(false));
        assert_eq!(menu_item_enabled("select-session-9", &state), Some(false));
    }

    #[test]
    fn next_and_previous_need_at_least_one_session() {
        let none = MenuState::default();
        assert_eq!(menu_item_enabled("next-session", &none), Some(false));
        assert_eq!(menu_item_enabled("previous-session", &none), Some(false));
        let one = MenuState {
            session_count: 1,
            ..MenuState::default()
        };
        assert_eq!(menu_item_enabled("next-session", &one), Some(true));
        assert_eq!(menu_item_enabled("previous-session", &one), Some(true));
    }

    #[test]
    fn global_and_placeholder_ids_are_not_dynamic() {
        // Global items and unbuilt placeholders are never in the handle set and
        // never synced, so the mapping returns None for them.
        for id in [
            "new-run",
            "settings",
            "go-home",
            "go-agents",
            "check-again",
            "toggle-sidebar",
            "agent-sign-in",
            "filter-sessions",
            "help",
            "select-session-", // empty suffix: not a real select item
            "not-a-real-id",
        ] {
            assert_eq!(menu_item_enabled(id, &full()), None, "`{id}`");
            assert!(!is_dynamic_menu_id(id), "`{id}` must not be dynamic");
        }
    }

    #[test]
    fn dynamic_ids_are_recognized() {
        for id in [
            "send-message",
            "show-output",
            "stop-session",
            "hard-stop-session",
            "repair-session",
            "cleanup-session",
            "open-session-window",
            "next-session",
            "previous-session",
            "jump-needs-you",
            "select-session-1",
            "select-session-9",
        ] {
            assert!(is_dynamic_menu_id(id), "`{id}` must be dynamic");
        }
    }

    #[test]
    fn menu_state_carries_only_primitive_fields() {
        // Boundary guard: MenuState must stay booleans plus one count. The
        // exhaustive destructure forces a new field to be classified here, and
        // the explicit `bool`/`u32` bindings reject anything that is not a
        // primitive flag or count (an id, path, or other identifying value
        // would fail to compile).
        let MenuState {
            has_selected_session,
            selected_session_can_run_control,
            selected_session_can_send_input,
            selected_session_can_view_output,
            selected_session_has_cleanup,
            selected_session_can_open_window,
            has_attention_session,
            session_count,
        } = MenuState::default();
        let _flags: [bool; 7] = [
            has_selected_session,
            selected_session_can_run_control,
            selected_session_can_send_input,
            selected_session_can_view_output,
            selected_session_has_cleanup,
            selected_session_can_open_window,
            has_attention_session,
        ];
        let _count: u32 = session_count;
    }
}
