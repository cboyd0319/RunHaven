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
//! Enabled-state tradeoff: menus are static. Items that act on the selected
//! session no-op gracefully in the frontend when nothing applies, instead of
//! running a chatty enabled-state sync protocol between the WebView and Rust.
//! The only statically disabled items are placeholders for surfaces that do
//! not exist yet (the session filter and Help content).
//!
//! Standard system items (About, Services, Hide, Quit, Undo/Redo, clipboard,
//! Close Window, Minimize, Zoom, Full Screen) use [`PredefinedMenuItem`] so
//! the native behavior and reserved shortcuts stay the system's own.
//!
//! The menu contents live as declarative tables in [`spec`].

mod spec;

use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Runtime};

use spec::{Entry, MENU_BAR, Predef, command_ids};

// Menu events target only the main window (crate::MAIN_WINDOW_LABEL); future
// auxiliary windows must opt in deliberately instead of inheriting dispatch.
use crate::MAIN_WINDOW_LABEL;

/// The single Rust-to-WebView menu channel. Payload is the menu item id.
pub const MENU_COMMAND_EVENT: &str = "menu://command";

/// Build the full menu bar. Passed to `tauri::Builder::menu`.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    for (label, entries) in MENU_BAR {
        let submenu = Submenu::new(app, *label, true)?;
        for entry in *entries {
            append_entry(app, &submenu, entry)?;
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
    Ok(menu)
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
) -> tauri::Result<()> {
    match entry {
        Entry::Command(spec) => submenu.append(&MenuItem::with_id(
            app,
            spec.id,
            spec.label,
            spec.enabled,
            spec.accelerator,
        )?),
        Entry::Separator => submenu.append(&PredefinedMenuItem::separator(app)?),
        Entry::Submenu { label, entries } => {
            let child = Submenu::new(app, *label, true)?;
            for child_entry in *entries {
                append_entry(app, &child, child_entry)?;
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
        for source in [include_str!("mod.rs"), include_str!("spec.rs")] {
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
