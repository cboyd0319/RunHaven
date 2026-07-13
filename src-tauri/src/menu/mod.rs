//! Native macOS menu bar for the main window.
//!
//! Security boundary: the menu adds no new mutation path. Every custom item
//! only emits its own id string to the main window WebView over the
//! [`MENU_COMMAND_EVENT`] event. The frontend listener
//! (`ui/src/app/useMenuEvents.ts`) translates ids into the same UI affordances
//! a click would use: open New Session, toggle the app-icon count, or focus an
//! existing confirm control. Mutations keep flowing through the existing
//! invoke commands and their Rust-owned native dialogs; in particular the
//! Stop and Hard Stop items never call `stop_managed_session` or
//! `kill_managed_session` themselves, they only surface the existing stop
//! affordance; the Rust-owned native dialog remains the confirmation boundary.
//!
//! Enable-state sync: contextual Run items are built disabled and turned on by
//! [`sync_menu_state`] (see [`apply_menu_state`]) as the main window's
//! selection changes, so an action with no target grays out instead of silently
//! doing nothing. The frontend sends only primitive state ([`MenuState`]) when
//! the derived state actually changes, not on every poll; Rust owns the item
//! handles and the id-to-flag mapping. Enabling an item never lets it mutate:
//! the frontend handler plus the Rust-owned native dialog still gate every
//! action.
//!
//! Standard system items (About, Services, Hide, Quit, Undo/Redo, clipboard,
//! Close Window, Minimize, Zoom, Full Screen) use [`PredefinedMenuItem`] so
//! the native behavior and reserved shortcuts stay the system's own.
//!
//! The menu contents live as declarative tables in [`spec`].

mod spec;
mod state;

use std::collections::HashMap;

use tauri::menu::{
    AboutMetadata, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use spec::{Entry, MENU_BAR, Predef, command_ids};
use state::{menu_item_checked, menu_item_enabled, sidebar_menu_label};

pub(crate) use state::MenuState;

// Menu events target only the main window (crate::MAIN_WINDOW_LABEL); future
// auxiliary windows must opt in deliberately instead of inheriting dispatch.
use crate::MAIN_WINDOW_LABEL;

/// The single Rust-to-WebView menu channel. Payload is the menu item id.
pub const MENU_COMMAND_EVENT: &str = "menu://command";

/// Retained handles for the menu items `sync_menu_state` updates. muda's
/// Keeping the handles the builder already created is more reliable than
/// looking them up after build; each handle is an `Arc` clone, so this is cheap.
struct DynamicMenuItems<R: Runtime> {
    command_items: HashMap<&'static str, MenuItem<R>>,
    checked_items: HashMap<&'static str, CheckMenuItem<R>>,
}

/// Build the full menu bar. Passed to `tauri::Builder::menu`. Retains the
/// dynamic item handles in managed state so [`apply_menu_state`] can toggle
/// them later.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    let mut command_items = HashMap::new();
    let mut checked_items = HashMap::new();
    for (label, entries) in MENU_BAR {
        let submenu = Submenu::new(app, *label, true)?;
        for entry in *entries {
            append_entry(app, &submenu, entry, &mut command_items, &mut checked_items)?;
        }
        #[cfg(target_os = "macos")]
        {
            // Give the Window menu its native open-window list.
            if *label == "Window" {
                submenu.set_as_windows_menu_for_nsapp()?;
            }
        }
        menu.append(&submenu)?;
    }
    app.manage(DynamicMenuItems {
        command_items,
        checked_items,
    });
    Ok(menu)
}

/// Apply the WebView's reported [`MenuState`] to the retained dynamic items:
/// enable exactly the contextual Run items that apply right now and check the
/// app-icon count item when that persisted preference is on. Called only by the
/// `sync_menu_state` command. Menu presentation is not a mutation of session
/// state; the frontend handler and Rust-owned dialogs still gate actions.
pub(crate) fn apply_menu_state<R: Runtime>(
    app: &AppHandle<R>,
    state: &MenuState,
) -> Result<(), String> {
    let Some(handles) = app.try_state::<DynamicMenuItems<R>>() else {
        return Err("The menu is not ready yet.".to_string());
    };
    for (id, item) in &handles.command_items {
        if let Some(enabled) = menu_item_enabled(id, state) {
            item.set_enabled(enabled)
                .map_err(|_| "RunHaven could not update the menu.".to_string())?;
        }
    }
    if let Some(item) = handles.command_items.get("toggle-sidebar") {
        item.set_text(sidebar_menu_label(state))
            .map_err(|_| "RunHaven could not update the menu.".to_string())?;
    }
    for (id, item) in &handles.checked_items {
        if let Some(checked) = menu_item_checked(id, state) {
            item.set_checked(checked)
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
    // Emission fails only when the main window is gone; nothing to do then.
    let _ = app.emit_to(MAIN_WINDOW_LABEL, MENU_COMMAND_EVENT, id);
}

fn append_entry<R: Runtime>(
    app: &AppHandle<R>,
    submenu: &Submenu<R>,
    entry: &'static Entry,
    enabled_items: &mut HashMap<&'static str, MenuItem<R>>,
    checked_items: &mut HashMap<&'static str, CheckMenuItem<R>>,
) -> tauri::Result<()> {
    match entry {
        Entry::Command(spec) => {
            let item = MenuItem::with_id(app, spec.id, spec.label, spec.enabled, spec.accelerator)?;
            // Retain every command handle. Contextual items use it for enable
            // state and the sidebar toggle uses it for its action label.
            enabled_items.insert(spec.id, item.clone());
            submenu.append(&item)
        }
        Entry::Check(spec, checked) => {
            let item = CheckMenuItem::with_id(
                app,
                spec.id,
                spec.label,
                spec.enabled,
                *checked,
                spec.accelerator,
            )?;
            if menu_item_checked(spec.id, &MenuState::default()).is_some() {
                checked_items.insert(spec.id, item.clone());
            }
            submenu.append(&item)
        }
        Entry::Separator => submenu.append(&PredefinedMenuItem::separator(app)?),
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
