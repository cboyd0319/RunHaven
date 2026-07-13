//! Tests for the declarative menu tables. Split out of `spec.rs` to keep that
//! file under the repo line ceiling; `super` is the `spec` module.

use super::*;

use crate::menu::state::is_dynamic_menu_id;

/// Modifier tokens verified against the pinned muda 0.19 accelerator
/// parser (`parse_modifier`).
const MODIFIER_TOKENS: &[&str] = &["Cmd", "Option", "Shift", "Ctrl"];

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
                Entry::Command(spec) | Entry::Check(spec, _) => specs.push(spec),
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

fn check_specs() -> Vec<(&'static CommandSpec, bool)> {
    let mut checks = Vec::new();
    for (_, entries) in MENU_BAR {
        for entry in *entries {
            if let Entry::Check(spec, checked) = entry {
                checks.push((spec, *checked));
            }
        }
    }
    checks
}

#[test]
fn menu_bar_has_only_the_standard_and_contextual_menus_in_order() {
    let labels: Vec<&str> = MENU_BAR.iter().map(|(label, _)| *label).collect();
    assert_eq!(
        labels,
        ["RunHaven", "File", "Edit", "View", "Session", "Window"]
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
            "new-run",
            "toggle-sidebar",
            "show-attention-badge",
            "send-message",
            "stop-session",
            "hard-stop-session",
            "cleanup-session",
        ]
    );
}

#[test]
fn attention_badge_is_one_standard_checked_view_command() {
    let checks = check_specs();
    assert_eq!(checks.len(), 1);
    let (spec, checked) = checks[0];
    assert_eq!(spec.id, "show-attention-badge");
    assert_eq!(spec.label, "Show Count on App Icon");
    assert!(spec.enabled);
    assert!(checked, "the default persisted preference is on");
    assert!(!command_ids().contains(&"settings"));
    let view_entries = MENU_BAR
        .iter()
        .find_map(|(label, entries)| (*label == "View").then_some(*entries))
        .expect("View menu");
    assert!(
        view_entries.iter().any(
            |entry| matches!(entry, Entry::Check(item, _) if item.id == "show-attention-badge")
        )
    );
}

#[test]
fn initial_enabled_state_matches_each_items_category() {
    // Every custom item earns its place now. Contextual Run items start
    // disabled until `sync_menu_state` says they apply; all global commands are
    // immediately usable. There are no permanently disabled placeholders.
    for spec in all_specs() {
        let is_dynamic = is_dynamic_menu_id(spec.id);
        let expected_enabled = !is_dynamic;
        assert_eq!(
            spec.enabled, expected_enabled,
            "menu item `{}` built enabled={} but its category expects enabled={}",
            spec.id, spec.enabled, expected_enabled
        );
    }
}

#[test]
fn menu_has_no_disabled_placeholders() {
    let dead: Vec<&str> = all_specs()
        .into_iter()
        .filter(|spec| !spec.enabled && !is_dynamic_menu_id(spec.id))
        .map(|spec| spec.id)
        .collect();
    assert!(dead.is_empty(), "dead menu placeholders remain: {dead:?}");
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
