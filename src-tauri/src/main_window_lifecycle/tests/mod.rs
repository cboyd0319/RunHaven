use super::*;

mod cleanup;
mod restore;

#[derive(Default)]
struct LifecycleProbe {
    main_exists: bool,
    calls: Vec<String>,
}

impl LifecycleProbe {
    fn missing() -> Self {
        Self::default()
    }

    fn hidden() -> Self {
        Self {
            main_exists: true,
            calls: Vec::new(),
        }
    }
}

impl MainWindowLifecycleEffects for LifecycleProbe {
    fn main_window_exists(&self) -> bool {
        self.main_exists
    }

    fn create_main_window(&mut self) -> Result<(), String> {
        self.main_exists = true;
        self.calls.push("create-main".to_string());
        Ok(())
    }

    fn show_main_window(&mut self) -> Result<(), String> {
        self.calls.push("show-main".to_string());
        Ok(())
    }

    fn focus_main_window(&mut self) -> Result<(), String> {
        self.calls.push("focus-main".to_string());
        Ok(())
    }

    fn emit_main_menu_command(&mut self, command: &str) -> Result<(), String> {
        self.calls.push(format!("emit:{command}"));
        Ok(())
    }

    fn close_terminal_sessions(&mut self) {
        self.calls.push("close-terminals".to_string());
    }

    fn shutdown_background_jobs(&mut self) {
        self.calls.push("shutdown-background".to_string());
    }
}

impl MainWindowCloseEffects for LifecycleProbe {
    fn prevent_close(&mut self) {
        self.calls.push("prevent-close".to_string());
    }

    fn hide_main_window(&mut self) -> Result<(), String> {
        self.calls.push("hide-main".to_string());
        Ok(())
    }
}
