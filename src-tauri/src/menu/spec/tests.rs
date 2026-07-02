//! Tests for the declarative menu tables. Split out of `spec.rs` to keep that
//! file under the repo line ceiling; `super` is the `spec` module.

use super::*;

use crate::menu::state::is_dynamic_menu_id;

/// Modifier tokens verified against the pinned muda 0.19 accelerator
/// parser (`parse_modifier`).
const MODIFIER_TOKENS: &[&str] = &["Cmd", "Option", "Shift", "Ctrl"];

/// Placeholders for surfaces that do not exist yet. These stay disabled and
/// are never touched by `sync_menu_state`; every other disabled item is
/// selection-scoped and driven by the sync path.
const PLACEHOLDER_IDS: &[&str] = &[
    "filter-sessions",
    "help",
    "help-agent-reach",
    "help-report-problem",
];

/// Key tokens verified against the pinned muda 0.19 accelerator parser:
/// single letters, digits, and the punctuation keys this menu uses.
fn is_valid_key_token(token: &str) -> bool {
    let mut chars = token.chars();
    let (Some(first), None) = (chars.next(), chars.next()) else {
        return false;
    };
    first.is_ascii_alphanumeric() || matches!(first, '.' | '[' | ']' | ',')
}

fn all_specs() -> Vec<&'static CommandSpec> {
    fn walk(entries: &'static [Entry], specs: &mut Vec<&'static CommandSpec>) {
        for entry in entries {
            match entry {
                Entry::Command(spec) => specs.push(spec),
                Entry::Submenu { entries, .. } => walk(entries, specs),
                Entry::Separator | Entry::Predefined(_) => {}
            }
        }
    }
    let mut specs = Vec::new();
    for (_, entries) in MENU_BAR {
        walk(entries, &mut specs);
    }
    specs
}

#[test]
fn menu_bar_has_the_eight_spec_menus_in_order() {
    let labels: Vec<&str> = MENU_BAR.iter().map(|(label, _)| *label).collect();
    assert_eq!(
        labels,
        [
            "RunHaven", "File", "Edit", "View", "Session", "Agent", "Window", "Help"
        ]
    );
}

#[test]
fn command_ids_are_stable_unique_and_kebab_case() {
    let ids = command_ids();
    assert!(!ids.is_empty());
    for id in &ids {
        assert!(
            id.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "menu id `{id}` must stay lowercase kebab-case; the frontend maps these strings"
        );
    }
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "menu ids must be unique");
    // The frontend listener maps these exact ids; renaming one is a
    // cross-boundary contract change.
    assert_eq!(
        ids,
        [
            "settings",
            "new-run",
            "filter-sessions",
            "toggle-sidebar",
            "go-home",
            "go-agents",
            "go-diagnostics",
            "go-maintenance",
            "select-session-1",
            "select-session-2",
            "select-session-3",
            "select-session-4",
            "select-session-5",
            "select-session-6",
            "select-session-7",
            "select-session-8",
            "select-session-9",
            "next-session",
            "previous-session",
            "jump-needs-you",
            "check-again",
            "send-message",
            "show-output",
            "stop-session",
            "hard-stop-session",
            "repair-session",
            "cleanup-session",
            "open-session-window",
            "agent-sign-in",
            "agent-rebuild-image",
            "agent-auth-details",
            "help",
            "help-agent-reach",
            "help-report-problem",
        ]
    );
}

#[test]
fn session_scoped_ids_are_exactly_the_session_menu_commands() {
    // These ids act on the main window's selected session and are
    // dropped while a monitor window is focused; every id not listed
    // here keeps working from any window. Changing this set changes
    // which shortcuts a monitor window swallows.
    let scoped = session_scoped_ids();
    assert_eq!(
        scoped,
        [
            "send-message",
            "show-output",
            "stop-session",
            "hard-stop-session",
            "repair-session",
            "cleanup-session",
            "open-session-window",
        ]
    );
    let all = command_ids();
    for id in &scoped {
        assert!(
            all.contains(id),
            "session-scoped id `{id}` must be a menu id"
        );
    }
}

#[test]
fn initial_enabled_state_matches_each_items_category() {
    // "No dead navigation" (spec section 10): an enabled item must do
    // something user-visible. An item is *built* disabled only when it is a
    // placeholder for a surface that does not exist yet, or a selection-scoped
    // item that `sync_menu_state` turns on once it applies. Everything else
    // (Settings, New Run, destinations, refresh, agent surfaces, sidebar
    // toggle) is built enabled. The dynamic set is read from the backend
    // mapping (`is_dynamic_menu_id`), so this pin and the sync path cannot
    // disagree about which ids start disabled.
    for spec in all_specs() {
        let is_placeholder = PLACEHOLDER_IDS.contains(&spec.id);
        let is_dynamic = is_dynamic_menu_id(spec.id);
        assert!(
            !(is_placeholder && is_dynamic),
            "menu id `{}` cannot be both a placeholder and sync-controlled",
            spec.id
        );
        let expected_enabled = !(is_placeholder || is_dynamic);
        assert_eq!(
            spec.enabled, expected_enabled,
            "menu item `{}` built enabled={} but its category expects enabled={}",
            spec.id, spec.enabled, expected_enabled
        );
    }
}

#[test]
fn every_placeholder_id_is_a_real_menu_item() {
    let ids = command_ids();
    for placeholder in PLACEHOLDER_IDS {
        assert!(
            ids.contains(placeholder),
            "placeholder id `{placeholder}` must be a menu id"
        );
    }
}

#[test]
fn accelerators_use_only_verified_muda_tokens() {
    for spec in all_specs() {
        let Some(accelerator) = spec.accelerator else {
            continue;
        };
        let tokens: Vec<&str> = accelerator.split('+').collect();
        assert!(
            tokens.len() >= 2,
            "accelerator `{accelerator}` on `{}` needs at least one modifier",
            spec.id
        );
        let (key, modifiers) = tokens.split_last().expect("non-empty tokens");
        for modifier in modifiers {
            assert!(
                MODIFIER_TOKENS.contains(modifier),
                "accelerator `{accelerator}` on `{}` uses unverified modifier `{modifier}`",
                spec.id
            );
        }
        assert!(
            is_valid_key_token(key),
            "accelerator `{accelerator}` on `{}` uses unverified key token `{key}`",
            spec.id
        );
    }
}

#[test]
fn accelerators_are_unique() {
    let mut seen = Vec::new();
    for spec in all_specs() {
        if let Some(accelerator) = spec.accelerator {
            assert!(
                !seen.contains(&accelerator),
                "accelerator `{accelerator}` is assigned twice"
            );
            seen.push(accelerator);
        }
    }
}
