//! Menu enable-state sync command.
//!
//! The main window reports which selection-scoped menu items apply right now so
//! the native menu can gray out actions that would do nothing (navigation
//! invariant 5, "no dead affordances"). The payload
//! ([`crate::menu::MenuState`]) is booleans and one small count only: no session
//! ids, paths, or secrets cross this boundary. The only effect is menu item
//! enable state; this adds no mutation path and reveals nothing to the WebView.
//! Enabling a menu item never lets it mutate: the frontend handler plus the
//! Rust-owned native dialog still gate every action.

use tauri::{AppHandle, Runtime};

use crate::menu::{MenuState, apply_menu_state};

#[tauri::command]
pub(crate) fn sync_menu_state<R: Runtime>(
    app: AppHandle<R>,
    state: MenuState,
) -> Result<(), String> {
    apply_menu_state(&app, &state)
}
