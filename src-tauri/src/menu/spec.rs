//! Declarative menu-bar contents: every submenu, item id, label, and
//! accelerator. The builder in `mod.rs` and the tests below walk the same
//! tables, so the tested id/accelerator data cannot drift from the built menu.
//! The frontend maps these id strings (`ui/src/app/useMenuEvents.ts`);
//! renaming one is a cross-boundary contract change.

/// One custom menu item: a stable id the frontend maps, a plain label, an
/// optional accelerator in muda syntax (`Mod+Mod+Key`), and a static
/// enabled flag (`false` only for not-yet-built placeholder surfaces).
pub(super) struct CommandSpec {
    pub(super) id: &'static str,
    pub(super) label: &'static str,
    pub(super) accelerator: Option<&'static str>,
    pub(super) enabled: bool,
}

/// Declarative menu entry.
pub(super) enum Entry {
    Command(CommandSpec),
    Separator,
    Submenu {
        label: &'static str,
        entries: &'static [Entry],
    },
    Predefined(Predef),
}

/// The predefined (system-behavior) items this menu uses.
pub(super) enum Predef {
    About,
    Services,
    Hide,
    HideOthers,
    ShowAll,
    Quit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    CloseWindow,
    Minimize,
    Zoom,
    Fullscreen,
    BringAllToFront,
}

const fn command(
    id: &'static str,
    label: &'static str,
    accelerator: Option<&'static str>,
) -> Entry {
    Entry::Command(CommandSpec {
        id,
        label,
        accelerator,
        enabled: true,
    })
}

const fn placeholder(
    id: &'static str,
    label: &'static str,
    accelerator: Option<&'static str>,
) -> Entry {
    Entry::Command(CommandSpec {
        id,
        label,
        accelerator,
        enabled: false,
    })
}

const APP_MENU: &[Entry] = &[
    Entry::Predefined(Predef::About),
    Entry::Separator,
    // Settings opens as a dialog in the main window (B3 shipped lean: too few
    // settings today to justify a separate settings window; the gap tracker's
    // B3 section records the deviation from spec 3.9).
    command("settings", "Settings...", Some("Cmd+,")),
    Entry::Separator,
    Entry::Predefined(Predef::Services),
    Entry::Separator,
    Entry::Predefined(Predef::Hide),
    Entry::Predefined(Predef::HideOthers),
    Entry::Predefined(Predef::ShowAll),
    Entry::Separator,
    Entry::Predefined(Predef::Quit),
];

// Open Recent needs a Rust-side recent-projects store and ships as its own
// follow-up sub-slice.
const FILE_MENU: &[Entry] = &[
    command("new-run", "New Run...", Some("Cmd+N")),
    Entry::Separator,
    Entry::Predefined(Predef::CloseWindow),
];

const EDIT_MENU: &[Entry] = &[
    Entry::Predefined(Predef::Undo),
    Entry::Predefined(Predef::Redo),
    Entry::Separator,
    Entry::Predefined(Predef::Cut),
    Entry::Predefined(Predef::Copy),
    Entry::Predefined(Predef::Paste),
    Entry::Predefined(Predef::SelectAll),
    Entry::Separator,
    // The session filter field does not exist yet; the item and its shortcut
    // are reserved and disabled until the filter ships.
    placeholder("filter-sessions", "Filter Sessions", Some("Cmd+F")),
];

const SELECT_SESSION_MENU: &[Entry] = &[
    command("select-session-1", "Session 1", Some("Cmd+1")),
    command("select-session-2", "Session 2", Some("Cmd+2")),
    command("select-session-3", "Session 3", Some("Cmd+3")),
    command("select-session-4", "Session 4", Some("Cmd+4")),
    command("select-session-5", "Session 5", Some("Cmd+5")),
    command("select-session-6", "Session 6", Some("Cmd+6")),
    command("select-session-7", "Session 7", Some("Cmd+7")),
    command("select-session-8", "Session 8", Some("Cmd+8")),
    command("select-session-9", "Session 9", Some("Cmd+9")),
];

const VIEW_MENU: &[Entry] = &[
    // Static menus cannot retitle on state, so the toggle names both actions.
    command(
        "toggle-sidebar",
        "Show or Hide Sidebar",
        Some("Option+Cmd+S"),
    ),
    Entry::Separator,
    command("go-home", "Go to Home", Some("Cmd+0")),
    command("go-agents", "Go to Agents", Some("Option+Cmd+1")),
    command("go-diagnostics", "Go to Diagnostics", Some("Option+Cmd+2")),
    command("go-maintenance", "Go to Maintenance", Some("Option+Cmd+3")),
    Entry::Separator,
    Entry::Submenu {
        label: "Select Session",
        entries: SELECT_SESSION_MENU,
    },
    command("next-session", "Next Session", Some("Shift+Cmd+]")),
    command("previous-session", "Previous Session", Some("Shift+Cmd+[")),
    command(
        "jump-needs-you",
        "Next Session That Needs You",
        Some("Cmd+J"),
    ),
    Entry::Separator,
    command("check-again", "Check Again", Some("Cmd+R")),
    Entry::Separator,
    Entry::Predefined(Predef::Fullscreen),
];

// Acts on the selected session; every item no-ops in the frontend when no
// session is selected or its control is not on screen. Worktree review
// actions (Merge, Keep, Recover, Discard) and Open in New Window stay off the
// menu until their slices land; the review card in the session view is the
// affordance for worktree actions today.
const SESSION_MENU: &[Entry] = &[
    command("send-message", "Send Message...", None),
    command("show-output", "Show Latest Output", Some("Shift+Cmd+O")),
    Entry::Separator,
    command("stop-session", "Stop", Some("Cmd+.")),
    command("hard-stop-session", "Hard Stop", Some("Option+Cmd+.")),
    Entry::Separator,
    command("repair-session", "Repair Session Record...", None),
    command("cleanup-session", "Clean Up Session...", None),
];

// All three land on the Agents view, where sign-in, rebuild, and auth details
// live; per-agent targeting arrives with Agents-view selection focus work.
const AGENT_MENU: &[Entry] = &[
    command("agent-sign-in", "Sign In...", None),
    command("agent-rebuild-image", "Rebuild Image...", None),
    command("agent-auth-details", "Show Auth Details", None),
];

const WINDOW_MENU: &[Entry] = &[
    Entry::Predefined(Predef::Minimize),
    Entry::Predefined(Predef::Zoom),
    Entry::Separator,
    Entry::Predefined(Predef::BringAllToFront),
];

// Help content surfaces are not built yet; reserved and disabled.
const HELP_MENU: &[Entry] = &[
    placeholder("help", "RunHaven Help", None),
    placeholder("help-agent-reach", "What Can an Agent Reach?", None),
    placeholder("help-report-problem", "Report a Problem...", None),
];

/// The whole menu bar, in macOS-standard order. On macOS the first submenu
/// becomes the application menu.
pub(super) const MENU_BAR: &[(&str, &[Entry])] = &[
    ("RunHaven", APP_MENU),
    ("File", FILE_MENU),
    ("Edit", EDIT_MENU),
    ("View", VIEW_MENU),
    ("Session", SESSION_MENU),
    ("Agent", AGENT_MENU),
    ("Window", WINDOW_MENU),
    ("Help", HELP_MENU),
];

/// Every custom command id in the menu bar, in declaration order.
pub(super) fn command_ids() -> Vec<&'static str> {
    fn walk(entries: &'static [Entry], ids: &mut Vec<&'static str>) {
        for entry in entries {
            match entry {
                Entry::Command(spec) => ids.push(spec.id),
                Entry::Submenu { entries, .. } => walk(entries, ids),
                Entry::Separator | Entry::Predefined(_) => {}
            }
        }
    }
    let mut ids = Vec::new();
    for (_, entries) in MENU_BAR {
        walk(entries, &mut ids);
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn only_unbuilt_surfaces_are_disabled() {
        // "No dead navigation" (spec section 10): an enabled item must do
        // something user-visible, so an item may be disabled only while its
        // surface does not exist. Settings is live (in-window dialog, B3).
        let disabled: Vec<&str> = all_specs()
            .iter()
            .filter(|spec| !spec.enabled)
            .map(|spec| spec.id)
            .collect();
        assert_eq!(
            disabled,
            [
                "filter-sessions",
                "help",
                "help-agent-reach",
                "help-report-problem"
            ]
        );
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
}
