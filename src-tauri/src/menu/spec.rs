//! Declarative menu-bar contents: every submenu, item id, label, and
//! accelerator. The builder in `mod.rs` and the tests below walk the same
//! tables, so the tested id/accelerator data cannot drift from the built menu.
//! The frontend maps these id strings (`ui/src/app/useMenuEvents.ts`);
//! renaming one is a cross-boundary contract change.

/// One custom menu item: a stable id the frontend maps, a plain label, an
/// optional accelerator in muda syntax (`Mod+Mod+Key`), and the enabled state
/// the menu is *built* with. `false` covers two cases: a placeholder for a
/// surface that does not exist yet, and a selection-scoped item that starts
/// off until `sync_menu_state` (see `state.rs`) enables it once it applies.
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

/// An item built disabled. Two kinds use this: placeholders for surfaces that
/// do not exist yet (never synced), and selection-scoped items that
/// `sync_menu_state` (see `state.rs`) enables once a matching target exists.
/// Building them disabled makes the no-selection launch state correct before
/// the first sync arrives.
const fn initially_disabled(
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
    initially_disabled("filter-sessions", "Filter Sessions", Some("Cmd+F")),
];

// Cmd-1..9 select the Nth sidebar row. Each starts disabled and turns on only
// while the sidebar actually has that many sessions (sync_menu_state).
const SELECT_SESSION_MENU: &[Entry] = &[
    initially_disabled("select-session-1", "Session 1", Some("Cmd+1")),
    initially_disabled("select-session-2", "Session 2", Some("Cmd+2")),
    initially_disabled("select-session-3", "Session 3", Some("Cmd+3")),
    initially_disabled("select-session-4", "Session 4", Some("Cmd+4")),
    initially_disabled("select-session-5", "Session 5", Some("Cmd+5")),
    initially_disabled("select-session-6", "Session 6", Some("Cmd+6")),
    initially_disabled("select-session-7", "Session 7", Some("Cmd+7")),
    initially_disabled("select-session-8", "Session 8", Some("Cmd+8")),
    initially_disabled("select-session-9", "Session 9", Some("Cmd+9")),
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
    // Session movement needs at least one session; jump also needs a session
    // that actually needs the user. Both start disabled and sync on.
    initially_disabled("next-session", "Next Session", Some("Shift+Cmd+]")),
    initially_disabled("previous-session", "Previous Session", Some("Shift+Cmd+[")),
    initially_disabled(
        "jump-needs-you",
        "Next Session That Needs You",
        Some("Cmd+J"),
    ),
    Entry::Separator,
    command("check-again", "Check Again", Some("Cmd+R")),
    Entry::Separator,
    Entry::Predefined(Predef::Fullscreen),
];

// Acts on the selected session. Every item starts disabled and is enabled by
// `sync_menu_state` only while it applies (a session is selected and in the
// right state), so an inapplicable action grays out instead of silently doing
// nothing. Worktree review actions (Merge, Keep, Recover, Discard) stay off
// the menu until their slice lands; the review card in the session view is the
// affordance for worktree actions today. Open in New Window opens a read-only
// monitor window through the validated `open_session_window` command (benign,
// no mutation, so no confirm step); it is the one menu id whose frontend
// handler invokes a command instead of focusing a control.
const SESSION_MENU: &[Entry] = &[
    initially_disabled("send-message", "Send Message...", None),
    initially_disabled("show-output", "Show Latest Output", Some("Shift+Cmd+O")),
    Entry::Separator,
    initially_disabled("stop-session", "Stop", Some("Cmd+.")),
    initially_disabled("hard-stop-session", "Hard Stop", Some("Option+Cmd+.")),
    Entry::Separator,
    initially_disabled("repair-session", "Repair Session Record...", None),
    initially_disabled("cleanup-session", "Clean Up Session...", None),
    Entry::Separator,
    initially_disabled("open-session-window", "Open in New Window", None),
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
    initially_disabled("help", "RunHaven Help", None),
    initially_disabled("help-agent-reach", "What Can an Agent Reach?", None),
    initially_disabled("help-report-problem", "Report a Problem...", None),
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

fn collect_command_ids(entries: &'static [Entry], ids: &mut Vec<&'static str>) {
    for entry in entries {
        match entry {
            Entry::Command(spec) => ids.push(spec.id),
            Entry::Submenu { entries, .. } => collect_command_ids(entries, ids),
            Entry::Separator | Entry::Predefined(_) => {}
        }
    }
}

/// Every custom command id in the menu bar, in declaration order.
pub(super) fn command_ids() -> Vec<&'static str> {
    let mut ids = Vec::new();
    for (_, entries) in MENU_BAR {
        collect_command_ids(entries, &mut ids);
    }
    ids
}

/// Ids that act on the main window's selected session: exactly the custom
/// commands of the Session menu, derived from the same table the builder
/// walks so this list cannot drift from the built menu. The dispatcher in
/// `mod.rs` drops these when a monitor window is focused, because the
/// selection they act on is not the session that window shows. Every other
/// id is global (navigation, selection, refresh, New Run, agent surfaces)
/// and unambiguous from any window.
pub(super) fn session_scoped_ids() -> Vec<&'static str> {
    let mut ids = Vec::new();
    collect_command_ids(SESSION_MENU, &mut ids);
    ids
}

#[cfg(test)]
mod tests;
