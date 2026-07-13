//! Declarative menu-bar contents: every submenu, item id, label, and
//! accelerator. The builder in `mod.rs` and the tests below walk the same
//! tables, so the tested id/accelerator data cannot drift from the built menu.
//! The frontend maps these id strings (`ui/src/app/useMenuEvents.ts`);
//! renaming one is a cross-boundary contract change.

/// One custom menu item: a stable id the frontend maps, a plain label, an
/// optional accelerator in muda syntax (`Mod+Mod+Key`), and the enabled state
/// the menu is *built* with. Contextual Run items start off until
/// `sync_menu_state` (see `state.rs`) enables them once they apply.
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

/// A contextual item built disabled until `sync_menu_state` (see `state.rs`)
/// enables it once a matching target exists. Building these items disabled
/// makes the no-selection launch state correct before the first sync arrives.
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
    // Two settings do not earn a separate window. This command expands the
    // inline Settings section in the main window's sidebar.
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
    command("new-run", "New Session...", Some("Cmd+N")),
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
];

const VIEW_MENU: &[Entry] = &[
    // Static menus cannot retitle on state, so the toggle names both actions.
    command(
        "toggle-sidebar",
        "Show or Hide Sidebar",
        Some("Option+Cmd+S"),
    ),
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
    initially_disabled("cleanup-session", "Clean Up Run...", None),
    Entry::Separator,
    initially_disabled("open-session-window", "Open in New Window", None),
];

const WINDOW_MENU: &[Entry] = &[
    Entry::Predefined(Predef::Minimize),
    Entry::Predefined(Predef::Zoom),
    Entry::Separator,
    Entry::Predefined(Predef::BringAllToFront),
];

// Keep the standard macOS Help menu shell so the system owns Help search. Do
// not advertise application Help commands until real Help content exists.
const HELP_MENU: &[Entry] = &[];

/// The whole menu bar, in macOS-standard order. On macOS the first submenu
/// becomes the application menu.
pub(super) const MENU_BAR: &[(&str, &[Entry])] = &[
    ("RunHaven", APP_MENU),
    ("File", FILE_MENU),
    ("Edit", EDIT_MENU),
    ("View", VIEW_MENU),
    ("Run", SESSION_MENU),
    ("Window", WINDOW_MENU),
    ("Help", HELP_MENU),
];

fn collect_command_ids(entries: &'static [Entry], ids: &mut Vec<&'static str>) {
    for entry in entries {
        match entry {
            Entry::Command(spec) => ids.push(spec.id),
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
/// commands of the Run menu, derived from the same table the builder
/// walks so this list cannot drift from the built menu. The dispatcher in
/// `mod.rs` drops these when a monitor window is focused, because the
/// selection they act on is not the session that window shows. Every other
/// id is global (Settings, New Session, or the sidebar toggle) and unambiguous
/// from any window.
pub(super) fn session_scoped_ids() -> Vec<&'static str> {
    let mut ids = Vec::new();
    collect_command_ids(SESSION_MENU, &mut ids);
    ids
}

#[cfg(test)]
mod tests;
