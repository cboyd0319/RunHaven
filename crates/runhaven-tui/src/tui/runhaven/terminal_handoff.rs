//! Phase 4 terminal handoff proof for the Codex-vendored runtime.
//!
//! This is a narrow smoke hook, not the launch path. It lets us prove Codex
//! `Tui::with_restored` can release the terminal for a foreground child and
//! restore ownership before RunHaven wires real agent launch.

use std::ffi::OsStr;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use tokio::runtime::Builder;

use crate::tui::codex_runtime;
use crate::tui::terminal_title::clear_terminal_title;
use crate::tui::terminal_title::set_terminal_title;

const HANDOFF_SMOKE_ENV: &str = "RUNHAVEN_TUI_HANDOFF_SMOKE";
const SUCCESS_MARKER: &str = "runhaven terminal handoff child ok";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandoffSmokeMode {
    Success,
    EarlyError,
}

pub(crate) fn run_smoke_from_env() -> Result<Option<i32>> {
    let Some(value) = std::env::var_os(HANDOFF_SMOKE_ENV) else {
        return Ok(None);
    };
    let Some(mode) = HandoffSmokeMode::parse(&value) else {
        bail!("{HANDOFF_SMOKE_ENV} must be success or error");
    };

    run_smoke(mode).map(Some)
}

fn run_smoke(mode: HandoffSmokeMode) -> Result<i32> {
    let mut initialized = codex_runtime::init().context("initialize Codex terminal runtime")?;
    initialized
        .terminal
        .clear()
        .context("clear terminal before handoff smoke")?;

    let runtime = Builder::new_current_thread()
        .enable_time()
        .build()
        .context("start handoff smoke runtime")?;
    let mut tui = {
        let _guard = runtime.enter();
        codex_runtime::Tui::new(
            initialized.terminal,
            initialized.enhanced_keys_supported,
            initialized.stderr_guard,
        )
    };
    let mut restore_guard = CodexTerminalRestoreGuard::new();
    let _ = set_terminal_title("RunHaven terminal handoff smoke");
    if let Err(err) = tui.clear_pet_images() {
        tracing::warn!(error = %err, "failed to clear pet images before handoff smoke");
    }
    let _ = clear_terminal_title();

    let child_result = runtime.block_on(
        tui.with_restored(codex_runtime::RestoreMode::Full, || async move {
            run_child(mode)
        }),
    );

    if let Err(err) = tui.clear_pet_images() {
        tracing::warn!(error = %err, "failed to clear pet images after handoff smoke");
    }
    if let Err(err) = tui.terminal.clear() {
        tracing::warn!(error = %err, "failed to clear terminal after handoff smoke");
    }
    drop(tui);
    restore_guard.restore()?;

    let status = match child_result {
        Ok(status) => status,
        Err(err) => bail!("terminal handoff child failed to start: {err}"),
    };
    if !status.success() {
        bail!("terminal handoff child exited with {status}");
    }
    Ok(0)
}

fn run_child(mode: HandoffSmokeMode) -> std::io::Result<std::process::ExitStatus> {
    let (program, args) = mode.command();
    Command::new(program).args(args).status()
}

impl HandoffSmokeMode {
    fn parse(value: &OsStr) -> Option<Self> {
        let value = value.to_str()?.trim().to_ascii_lowercase();
        match value.as_str() {
            "success" => Some(Self::Success),
            "error" => Some(Self::EarlyError),
            _ => None,
        }
    }

    fn command(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Success => ("/usr/bin/printf", &["%s\n", SUCCESS_MARKER]),
            Self::EarlyError => ("/__runhaven_missing_terminal_handoff_child__", &[]),
        }
    }
}

struct CodexTerminalRestoreGuard {
    active: bool,
}

impl CodexTerminalRestoreGuard {
    fn new() -> Self {
        Self { active: true }
    }

    fn restore(&mut self) -> Result<()> {
        if self.active {
            let _ = clear_terminal_title();
            codex_runtime::restore_after_exit().context("restore Codex terminal runtime")?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for CodexTerminalRestoreGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = clear_terminal_title();
            if let Err(err) = codex_runtime::restore_after_exit() {
                tracing::warn!(error = %err, "failed to restore terminal after handoff smoke");
            }
            self.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    #[test]
    fn smoke_mode_parser_accepts_named_modes_only() {
        assert_eq!(
            HandoffSmokeMode::parse(&OsString::from("success")),
            Some(HandoffSmokeMode::Success)
        );
        assert_eq!(
            HandoffSmokeMode::parse(&OsString::from(" ERROR ")),
            Some(HandoffSmokeMode::EarlyError)
        );
        assert_eq!(HandoffSmokeMode::parse(&OsString::from("agent")), None);
    }

    #[test]
    fn success_smoke_uses_exact_harmless_command() {
        let (program, args) = HandoffSmokeMode::Success.command();
        assert_eq!(program, "/usr/bin/printf");
        assert_eq!(args, &["%s\n", SUCCESS_MARKER]);
    }

    #[test]
    fn early_error_smoke_uses_missing_absolute_command() {
        let (program, args) = HandoffSmokeMode::EarlyError.command();
        assert_eq!(program, "/__runhaven_missing_terminal_handoff_child__");
        assert!(args.is_empty());
        let err = run_child(HandoffSmokeMode::EarlyError).expect_err("missing child");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}
