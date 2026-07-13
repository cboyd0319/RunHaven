//! The menu state contract and its id-to-flag mapping.
//!
//! The WebView reports which menu state applies right now, using the same
//! persisted preference and session helpers the interface already uses
//! (`ui/src/app/useMenuStateSync.ts`). [`MenuState`] carries booleans only: no
//! session ids, paths, or secrets ever cross this boundary, so a compromised
//! WebView could at worst gray the wrong menu items.
//! `apply_menu_state` in `mod.rs` drives the retained menu item handles through
//! [`menu_item_enabled`]; this module owns the mapping and stays free of any
//! Tauri runtime types so it is testable as pure logic.

use serde::Deserialize;

/// The checked preference and selection-scoped items for the main window.
/// Every field is a primitive; adding an id, path, or other
/// identifying value here would be a boundary regression, and
/// `menu_state_carries_only_primitive_fields` fails the build if it happens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MenuState {
    /// Whether the Dock shows the count of sessions that need attention.
    pub show_attention_badge: bool,
    /// Whether the main sidebar is visible, used to name the toggle's action.
    pub sidebar_open: bool,
    /// The selected session is in a lifecycle that accepts Stop and Hard Stop
    /// (running/starting/stop-requested/kill-requested).
    pub selected_session_can_run_control: bool,
    /// The selected session shows its message composer.
    pub selected_session_can_send_input: bool,
    /// The selected session offers a standalone clean-up (not the worktree
    /// review, which owns its own removal).
    pub selected_session_has_cleanup: bool,
}

/// The enable state for one dynamic menu id under `state`, or `None` when the
/// id is not sync-controlled (a global or unknown item this path never
/// touches). This is the single source of truth: the retained-handle set is
/// derived from it via [`is_dynamic_menu_id`], and the `spec.rs` initial-state
/// test reads the same function, so the three cannot drift apart.
pub(crate) fn menu_item_enabled(id: &str, state: &MenuState) -> Option<bool> {
    Some(match id {
        "send-message" => state.selected_session_can_send_input,
        // Stop and Hard Stop share one gate: the selected session is in a
        // controllable lifecycle.
        "stop-session" | "hard-stop-session" => state.selected_session_can_run_control,
        "cleanup-session" => state.selected_session_has_cleanup,
        _ => return None,
    })
}

/// The sidebar command always names the action it will perform.
pub(crate) const fn sidebar_menu_label(state: &MenuState) -> &'static str {
    if state.sidebar_open {
        "Hide Sidebar"
    } else {
        "Show Sidebar"
    }
}

/// The checked state for one sync-controlled menu id, or `None` when the item
/// is not a check item owned by this state contract.
pub(crate) fn menu_item_checked(id: &str, state: &MenuState) -> Option<bool> {
    match id {
        "show-attention-badge" => Some(state.show_attention_badge),
        _ => None,
    }
}

/// Whether `sync_menu_state` drives this id, so its handle must be retained at
/// build. Derived from [`menu_item_enabled`] so it cannot list a different set.
#[cfg(test)]
pub(crate) fn is_dynamic_menu_id(id: &str) -> bool {
    menu_item_enabled(id, &MenuState::default()).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `MenuState` where every contextual capability is present: the
    /// "everything applies" end of the range.
    fn full() -> MenuState {
        MenuState {
            show_attention_badge: true,
            sidebar_open: true,
            selected_session_can_run_control: true,
            selected_session_can_send_input: true,
            selected_session_has_cleanup: true,
        }
    }

    #[test]
    fn no_selection_disables_every_session_scoped_item() {
        let state = MenuState::default();
        for id in [
            "send-message",
            "stop-session",
            "hard-stop-session",
            "cleanup-session",
        ] {
            assert_eq!(
                menu_item_enabled(id, &state),
                Some(false),
                "`{id}` must be disabled with no selection"
            );
        }
    }

    #[test]
    fn run_control_gate_covers_stop_and_hard_stop() {
        let can_control = MenuState {
            selected_session_can_run_control: true,
            ..MenuState::default()
        };
        for id in ["stop-session", "hard-stop-session"] {
            assert_eq!(menu_item_enabled(id, &can_control), Some(true));
        }
        // A different capability does not enable run control.
        let send_only = MenuState {
            selected_session_can_send_input: true,
            ..MenuState::default()
        };
        for id in ["stop-session", "hard-stop-session"] {
            assert_eq!(menu_item_enabled(id, &send_only), Some(false));
        }
    }

    #[test]
    fn each_capability_maps_to_its_own_item() {
        let state = full();
        assert_eq!(menu_item_enabled("send-message", &state), Some(true));
        assert_eq!(menu_item_enabled("cleanup-session", &state), Some(true));
    }

    #[test]
    fn attention_badge_maps_to_a_checked_item_not_an_enabled_item() {
        let shown = full();
        assert_eq!(
            menu_item_checked("show-attention-badge", &shown),
            Some(true)
        );

        let hidden = MenuState {
            show_attention_badge: false,
            ..shown
        };
        assert_eq!(
            menu_item_checked("show-attention-badge", &hidden),
            Some(false)
        );
        assert_eq!(menu_item_enabled("show-attention-badge", &shown), None);
        assert!(!is_dynamic_menu_id("show-attention-badge"));
        assert_eq!(menu_item_checked("not-a-real-id", &shown), None);
    }

    #[test]
    fn attention_badge_uses_the_frontend_camel_case_contract() {
        let state: MenuState = serde_json::from_value(serde_json::json!({
            "showAttentionBadge": true,
            "sidebarOpen": true,
            "selectedSessionCanRunControl": false,
            "selectedSessionCanSendInput": false,
            "selectedSessionHasCleanup": false
        }))
        .expect("menu state");

        assert!(state.show_attention_badge);
    }

    #[test]
    fn global_and_unknown_ids_are_not_dynamic() {
        // Global and removed items are never in the handle set and never
        // synced, so the mapping returns None for them.
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
            "select-session-1",
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
            "stop-session",
            "hard-stop-session",
            "cleanup-session",
        ] {
            assert!(is_dynamic_menu_id(id), "`{id}` must be dynamic");
        }
    }

    #[test]
    fn removed_navigation_ids_are_not_dynamic() {
        for id in [
            "select-session-1",
            "select-session-9",
            "next-session",
            "previous-session",
            "jump-needs-you",
        ] {
            assert!(!is_dynamic_menu_id(id), "removed menu id `{id}`");
        }
    }

    #[test]
    fn menu_state_carries_only_primitive_fields() {
        // Boundary guard: MenuState must stay booleans. The exhaustive
        // destructure forces a new field to be classified here, and the
        // explicit `bool` bindings reject anything that is not a primitive
        // flag (an id, path, or other identifying value would fail to compile).
        let MenuState {
            show_attention_badge,
            sidebar_open,
            selected_session_can_run_control,
            selected_session_can_send_input,
            selected_session_has_cleanup,
        } = MenuState::default();
        let _flags: [bool; 5] = [
            show_attention_badge,
            sidebar_open,
            selected_session_can_run_control,
            selected_session_can_send_input,
            selected_session_has_cleanup,
        ];
    }

    #[test]
    fn sidebar_label_describes_the_action() {
        assert_eq!(sidebar_menu_label(&MenuState::default()), "Show Sidebar");
        assert_eq!(sidebar_menu_label(&full()), "Hide Sidebar");
    }
}
