//! Native macOS menu bar for the main window.
//!
//! Security boundary: the menu adds no new mutation path. Every custom item
//! only emits its own id string to the main window WebView over the
//! [`MENU_COMMAND_EVENT`] event. The frontend listener
//! (`ui/src/app/useMenuEvents.ts`) translates ids into the same UI affordances
//! a click would use: open the launch sheet, open the settings dialog, select
//! a sidebar row, or focus an existing confirm control. Mutations keep
//! flowing through the existing
//! invoke commands and their Rust-owned native dialogs; in particular the
//! Stop and Hard Stop items never call `stop_managed_session` or
//! `kill_managed_session` themselves, they only surface the existing stop
//! affordance which still requires its confirm step plus the native dialog.
//!
//! Enable-state sync: session-scoped items and the Cmd-1..9 selectors are
//! built disabled and turned on by [`sync_menu_state`] (see
//! [`apply_menu_state`]) as the main window's selection and session list
//! change, so a shortcut with no target grays out instead of silently doing
//! nothing. The frontend sends only booleans and a count ([`MenuState`]) when
//! the derived state actually changes, not on every poll; Rust owns the item
//! handles and the id-to-flag mapping. Enabling an item never lets it mutate:
//! the frontend handler plus the Rust-owned native dialog still gate every
//! action. The only permanently disabled items are placeholders for surfaces
//! that do not exist yet (the session filter and Help content).
//!
//! Focus rule: session-scoped items ([`spec::session_scoped_ids`], the
//! Session menu) act on the main window's selected session, so they dispatch
//! only while no monitor window is focused. A user watching session A in a
//! monitor window must not trigger a shortcut that silently targets the main
//! window's selection (possibly session B); the event is dropped instead.
//! Global items (navigation, session selection, refresh, New Run, agent
//! surfaces) are unambiguous and dispatch from any window.
//!
//! Standard system items (About, Services, Hide, Quit, Undo/Redo, clipboard,
//! Close Window, Minimize, Zoom, Full Screen) use [`PredefinedMenuItem`] so
//! the native behavior and reserved shortcuts stay the system's own.
//!
//! The menu contents live as declarative tables in [`spec`].

mod spec;
mod state;

use std::collections::HashMap;

use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use spec::{Entry, MENU_BAR, Predef, command_ids, session_scoped_ids};
use state::{is_dynamic_menu_id, menu_item_enabled};

pub(crate) use state::MenuState;

// Menu events target only the main window (crate::MAIN_WINDOW_LABEL); future
// auxiliary windows must opt in deliberately instead of inheriting dispatch.
use crate::MAIN_WINDOW_LABEL;
use crate::session_windows::SESSION_WINDOW_LABEL_PREFIX;

/// The single Rust-to-WebView menu channel. Payload is the menu item id.
pub const MENU_COMMAND_EVENT: &str = "menu://command";

/// Retained handles for the menu items `sync_menu_state` toggles. muda's
/// `Menu::get` and `Submenu::get` search only their own direct children
/// (verified against the pinned tauri 2.11.3 source), so nested items (the
/// Session menu, the Select Session submenu) could not be looked up by id
/// after build. Keeping the `MenuItem` handles the builder already created is
/// the reliable path; each handle is an `Arc` clone, so this is cheap.
struct DynamicMenuItems<R: Runtime> {
    items: HashMap<&'static str, MenuItem<R>>,
}

/// Build the full menu bar. Passed to `tauri::Builder::menu`. Retains the
/// dynamic item handles in managed state so [`apply_menu_state`] can toggle
/// them later.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    let mut dynamic_items = HashMap::new();
    for (label, entries) in MENU_BAR {
        let submenu = Submenu::new(app, *label, true)?;
        for entry in *entries {
            append_entry(app, &submenu, entry, &mut dynamic_items)?;
        }
        #[cfg(target_os = "macos")]
        {
            // Gives the Window menu its native open-window list and lets the
            // system add its search field to Help.
            if *label == "Window" {
                submenu.set_as_windows_menu_for_nsapp()?;
            }
            if *label == "Help" {
                submenu.set_as_help_menu_for_nsapp()?;
            }
        }
        menu.append(&submenu)?;
    }
    app.manage(DynamicMenuItems {
        items: dynamic_items,
    });
    Ok(menu)
}

/// Apply the WebView's reported [`MenuState`] to the retained dynamic items:
/// enable exactly the session-scoped and selector items that apply right now,
/// and disable the rest. Called only by the `sync_menu_state` command. Toggling
/// enable state is not a mutation of session state; the frontend handler and
/// the Rust-owned native dialog still gate every action behind these items.
pub(crate) fn apply_menu_state<R: Runtime>(
    app: &AppHandle<R>,
    state: &MenuState,
) -> Result<(), String> {
    let Some(handles) = app.try_state::<DynamicMenuItems<R>>() else {
        return Err("The menu is not ready yet.".to_string());
    };
    for (id, item) in &handles.items {
        if let Some(enabled) = menu_item_enabled(id, state) {
            item.set_enabled(enabled)
                .map_err(|_| "RunHaven could not update the menu.".to_string())?;
        }
    }
    Ok(())
}

/// Forward custom menu activations to the main window WebView. Predefined
/// items handle themselves natively and are never forwarded.
pub fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let id = event.id().as_ref();
    if !command_ids().contains(&id) {
        return;
    }
    // Focus rule (module docs): session-scoped shortcuts pressed while a
    // monitor window is focused would act on the main window's hidden
    // selection, not the session the user is watching, so they drop here.
    if session_scoped_ids().contains(&id) && monitor_window_is_focused(app) {
        return;
    }
    // Emission fails only when the main window is gone; nothing to do then.
    let _ = app.emit_to(MAIN_WINDOW_LABEL, MENU_COMMAND_EVENT, id);
}

/// True when a per-session monitor window (`session-*`) is the focused
/// window. An unreadable focus state counts as not focused, which preserves
/// dispatch to the main window; the actions behind session-scoped ids stay
/// confirm-gated there either way.
fn monitor_window_is_focused<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.webview_windows().into_iter().any(|(label, window)| {
        label.starts_with(SESSION_WINDOW_LABEL_PREFIX) && window.is_focused().unwrap_or(false)
    })
}

fn append_entry<R: Runtime>(
    app: &AppHandle<R>,
    submenu: &Submenu<R>,
    entry: &'static Entry,
    dynamic_items: &mut HashMap<&'static str, MenuItem<R>>,
) -> tauri::Result<()> {
    match entry {
        Entry::Command(spec) => {
            let item = MenuItem::with_id(app, spec.id, spec.label, spec.enabled, spec.accelerator)?;
            // Retain a handle for items the sync path drives so their enabled
            // state can be updated after build; global and placeholder items
            // keep their built state and need no handle.
            if is_dynamic_menu_id(spec.id) {
                dynamic_items.insert(spec.id, item.clone());
            }
            submenu.append(&item)
        }
        Entry::Separator => submenu.append(&PredefinedMenuItem::separator(app)?),
        Entry::Submenu { label, entries } => {
            let child = Submenu::new(app, *label, true)?;
            for child_entry in *entries {
                append_entry(app, &child, child_entry, dynamic_items)?;
            }
            submenu.append(&child)
        }
        Entry::Predefined(kind) => append_predefined(app, submenu, kind),
    }
}

fn append_predefined<R: Runtime>(
    app: &AppHandle<R>,
    submenu: &Submenu<R>,
    kind: &Predef,
) -> tauri::Result<()> {
    let item = match kind {
        Predef::About => {
            let package = app.package_info();
            PredefinedMenuItem::about(
                app,
                Some("About RunHaven"),
                Some(AboutMetadata {
                    name: Some(package.name.clone()),
                    version: Some(package.version.to_string()),
                    ..Default::default()
                }),
            )?
        }
        Predef::Services => PredefinedMenuItem::services(app, None)?,
        Predef::Hide => PredefinedMenuItem::hide(app, None)?,
        Predef::HideOthers => PredefinedMenuItem::hide_others(app, None)?,
        Predef::ShowAll => PredefinedMenuItem::show_all(app, None)?,
        Predef::Quit => PredefinedMenuItem::quit(app, None)?,
        Predef::Undo => PredefinedMenuItem::undo(app, None)?,
        Predef::Redo => PredefinedMenuItem::redo(app, None)?,
        Predef::Cut => PredefinedMenuItem::cut(app, None)?,
        Predef::Copy => PredefinedMenuItem::copy(app, None)?,
        Predef::Paste => PredefinedMenuItem::paste(app, None)?,
        Predef::SelectAll => PredefinedMenuItem::select_all(app, None)?,
        Predef::CloseWindow => PredefinedMenuItem::close_window(app, None)?,
        Predef::Minimize => PredefinedMenuItem::minimize(app, None)?,
        Predef::Zoom => PredefinedMenuItem::maximize(app, Some("Zoom"))?,
        Predef::Fullscreen => PredefinedMenuItem::fullscreen(app, None)?,
        Predef::BringAllToFront => PredefinedMenuItem::bring_all_to_front(app, None)?,
    };
    submenu.append(&item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_name_uses_only_valid_tauri_event_characters() {
        // Tauri event names permit only alphanumeric, '-', '/', ':', '_'.
        assert!(
            MENU_COMMAND_EVENT
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '/' | ':' | '_'))
        );
    }

    #[test]
    fn menu_module_never_reaches_runtime_commands() {
        // The menu is dispatch-only. Build the markers at runtime so this
        // test's own source cannot satisfy the search.
        let command_marker = format!("{}{}", "commands", "::");
        let core_marker = format!("{}{}", "runhaven_", "core");
        for source in [
            include_str!("mod.rs"),
            include_str!("spec.rs"),
            include_str!("state.rs"),
        ] {
            let production = match source.find("#[cfg(test)]") {
                Some(first_test) => &source[..first_test],
                None => source,
            };
            assert!(
                !production.contains(&command_marker) && !production.contains(&core_marker),
                "the menu module must never call runtime commands; it only emits ids to the WebView"
            );
        }
    }
}
