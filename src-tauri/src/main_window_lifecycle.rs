//! Main-window close, restore, menu-dispatch, and process-exit lifecycle.

use tauri::window::{Effect, EffectsBuilder};
use tauri::{
    AppHandle, Emitter, Manager, Runtime, WebviewWindow, WebviewWindowBuilder, Window, WindowEvent,
};

use crate::{MAIN_WINDOW_LABEL, background_jobs, terminal};

const RESTORE_ERROR: &str = "RunHaven could not restore the main window.";

trait MainWindowLifecycleEffects {
    fn main_window_exists(&self) -> bool;
    fn create_main_window(&mut self) -> Result<(), String>;
    fn show_main_window(&mut self) -> Result<(), String>;
    fn focus_main_window(&mut self) -> Result<(), String>;
    fn emit_main_menu_command(&mut self, command: &str) -> Result<(), String>;
    fn close_terminal_sessions(&mut self);
    fn shutdown_background_jobs(&mut self);
}

trait MainWindowCloseEffects {
    fn prevent_close(&mut self);
    fn hide_main_window(&mut self) -> Result<(), String>;
}

struct TauriMainWindowCloseEffects<'a, R: Runtime> {
    window: &'a Window<R>,
    api: &'a tauri::CloseRequestApi,
}

impl<R: Runtime> MainWindowCloseEffects for TauriMainWindowCloseEffects<'_, R> {
    fn prevent_close(&mut self) {
        self.api.prevent_close();
    }

    fn hide_main_window(&mut self) -> Result<(), String> {
        self.window
            .hide()
            .map_err(|_| "RunHaven could not hide the main window.".to_string())
    }
}

struct TauriLifecycleEffects<'a, R: Runtime> {
    app: &'a AppHandle<R>,
}

impl<R: Runtime> MainWindowLifecycleEffects for TauriLifecycleEffects<'_, R> {
    fn main_window_exists(&self) -> bool {
        self.app.get_webview_window(MAIN_WINDOW_LABEL).is_some()
    }

    fn create_main_window(&mut self) -> Result<(), String> {
        let config = self
            .app
            .config()
            .app
            .windows
            .iter()
            .find(|config| config.label == MAIN_WINDOW_LABEL)
            .cloned()
            .ok_or_else(|| RESTORE_ERROR.to_string())?;
        let window = WebviewWindowBuilder::from_config(self.app, &config)
            .and_then(WebviewWindowBuilder::build)
            .map_err(|_| RESTORE_ERROR.to_string())?;
        apply_sidebar_effect(&window);
        Ok(())
    }

    fn show_main_window(&mut self) -> Result<(), String> {
        let window = self
            .app
            .get_webview_window(MAIN_WINDOW_LABEL)
            .ok_or_else(|| RESTORE_ERROR.to_string())?;
        window.unminimize().map_err(|_| RESTORE_ERROR.to_string())?;
        window.show().map_err(|_| RESTORE_ERROR.to_string())
    }

    fn focus_main_window(&mut self) -> Result<(), String> {
        self.app
            .get_webview_window(MAIN_WINDOW_LABEL)
            .ok_or_else(|| RESTORE_ERROR.to_string())?
            .set_focus()
            .map_err(|_| RESTORE_ERROR.to_string())
    }

    fn emit_main_menu_command(&mut self, command: &str) -> Result<(), String> {
        self.app
            .emit_to(
                crate::MAIN_WINDOW_LABEL,
                crate::menu::MENU_COMMAND_EVENT,
                command,
            )
            .map_err(|_| "RunHaven could not perform that menu command.".to_string())
    }

    fn close_terminal_sessions(&mut self) {
        self.app
            .state::<terminal::TerminalSessionRegistry>()
            .close_all();
    }

    fn shutdown_background_jobs(&mut self) {
        self.app
            .state::<background_jobs::BackgroundJobRegistry>()
            .shutdown();
    }
}

fn restore_main_window(effects: &mut impl MainWindowLifecycleEffects) -> Result<(), String> {
    if !effects.main_window_exists() {
        effects.create_main_window()?;
    }
    effects.show_main_window()?;
    effects.focus_main_window()
}

fn dispatch_main_menu(
    effects: &mut impl MainWindowLifecycleEffects,
    command: &str,
) -> Result<(), String> {
    restore_main_window(effects)?;
    effects.emit_main_menu_command(command)
}

fn app_exit(effects: &mut impl MainWindowLifecycleEffects) {
    effects.close_terminal_sessions();
    effects.shutdown_background_jobs();
}

fn close_main_window(effects: &mut impl MainWindowCloseEffects) -> Result<(), String> {
    effects.prevent_close();
    effects.hide_main_window()
}

pub(crate) fn apply_main_window_effects<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        apply_sidebar_effect(&window);
    }
}

fn apply_sidebar_effect<R: Runtime>(window: &WebviewWindow<R>) {
    if let Err(error) = window.set_effects(EffectsBuilder::new().effect(Effect::Sidebar).build()) {
        log::warn!("macOS vibrancy unavailable, using solid background: {error}");
    }
}

pub(crate) fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }
    if let WindowEvent::CloseRequested { api, .. } = event
        && let Err(error) = close_main_window(&mut TauriMainWindowCloseEffects { window, api })
    {
        log::warn!("could not preserve the main window on close: {error}");
    }
}

pub(crate) fn dispatch_main_menu_command<R: Runtime>(
    app: &AppHandle<R>,
    command: &str,
) -> Result<(), String> {
    dispatch_main_menu(&mut TauriLifecycleEffects { app }, command)
}

fn dock_reopen<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    restore_main_window(&mut TauriLifecycleEffects { app })
}

pub(crate) fn on_run_event<R: Runtime>(app: &AppHandle<R>, event: tauri::RunEvent) {
    match event {
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen { .. } => {
            if let Err(error) = dock_reopen(app) {
                log::warn!("could not reopen the main window: {error}");
            }
        }
        tauri::RunEvent::Exit => app_exit(&mut TauriLifecycleEffects { app }),
        _ => {}
    }
}

#[cfg(test)]
fn main_window_destroyed_for_test(_effects: &mut impl MainWindowLifecycleEffects) {}

#[cfg(test)]
fn main_window_close_requested_for_test(
    effects: &mut impl MainWindowCloseEffects,
) -> Result<(), String> {
    close_main_window(effects)
}

#[cfg(test)]
fn app_exit_for_test(effects: &mut impl MainWindowLifecycleEffects) {
    app_exit(effects);
}

#[cfg(test)]
fn dock_reopen_for_test(effects: &mut impl MainWindowLifecycleEffects) -> Result<(), String> {
    restore_main_window(effects)
}

#[cfg(test)]
fn dispatch_main_menu_for_test(
    effects: &mut impl MainWindowLifecycleEffects,
    command: &str,
) -> Result<(), String> {
    dispatch_main_menu(effects, command)
}

#[cfg(test)]
mod tests;
