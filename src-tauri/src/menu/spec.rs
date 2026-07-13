//! Declarative menu-bar contents: every submenu, item id, label, and
//! accelerator. The builder in `mod.rs` and the tests below walk the same
//! tables, so the tested id/accelerator data cannot drift from the built menu.
//! The frontend maps these id strings (`ui/src/app/useMenuEvents.ts`);
//! renaming one is a cross-boundary contract change.

/// One custom menu item: a stable id the frontend maps, a plain label, an
/// optional accelerator in muda syntax (`Mod+Mod+Key`), and the enabled state
/// the menu is *built* with. Contextual Session items start off until
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
    Check(CommandSpec, bool),
    Separator,
    Predefined(Predef),
}

/// A standard checked command. The initial value matches RunHaven's default;
/// `sync_menu_state` replaces it with the persisted preference after startup.
const fn checked_command(
    id: &'static str,
    label: &'static str,
    accelerator: Option<&'static str>,
    checked: bool,
) -> Entry {
    Entry::Check(
        CommandSpec {
            id,
            label,
            accelerator,
            enabled: true,
        },
        checked,
    )
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
    command("new-run", "New Session…", Some("Cmd+N")),
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
    command("toggle-sidebar", "Hide Sidebar", Some("Ctrl+Cmd+S")),
    Entry::Separator,
    checked_command("show-attention-badge", "Show Count on App Icon", None, true),
    Entry::Separator,
    Entry::Predefined(Predef::Fullscreen),
];

// Acts on the selected session. Every item starts disabled and is enabled by
// `sync_menu_state` only while it applies (a session is selected and in the
// right state), so an inapplicable action grays out instead of silently doing
// nothing. Worktree review stays in the session view, where the changed files
// and consequences are visible. Destructive commands use an ellipsis because
// each opens a native confirmation before acting.
const SESSION_MENU: &[Entry] = &[
    initially_disabled("send-message", "Send Message…", None),
    Entry::Separator,
    initially_disabled("stop-session", "Stop…", Some("Cmd+.")),
    initially_disabled("hard-stop-session", "Force Stop…", Some("Option+Cmd+.")),
    Entry::Separator,
    initially_disabled("cleanup-session", "Delete Protected Copy…", None),
];

const WINDOW_MENU: &[Entry] = &[
    Entry::Predefined(Predef::Minimize),
    Entry::Predefined(Predef::Zoom),
    Entry::Separator,
    Entry::Predefined(Predef::BringAllToFront),
];

const HELP_MENU: &[Entry] = &[command("show-help", "RunHaven Help", None)];

/// The whole menu bar, in macOS-standard order. On macOS the first submenu
/// becomes the application menu.
pub(super) const MENU_BAR: &[(&str, &[Entry])] = &[
    ("RunHaven", APP_MENU),
    ("File", FILE_MENU),
    ("Edit", EDIT_MENU),
    ("View", VIEW_MENU),
    ("Session", SESSION_MENU),
    ("Window", WINDOW_MENU),
    ("Help", HELP_MENU),
];

fn collect_command_ids(entries: &'static [Entry], ids: &mut Vec<&'static str>) {
    for entry in entries {
        match entry {
            Entry::Command(spec) | Entry::Check(spec, _) => ids.push(spec.id),
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

#[cfg(test)]
mod tests;
