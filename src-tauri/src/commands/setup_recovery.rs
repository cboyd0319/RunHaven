//! Narrow, backend-owned recovery for setup blockers the desktop app can fix.
//!
//! The WebView supplies one opaque enum value. Rust revalidates the supported
//! macOS host and pinned Apple container CLI, owns the exact argument lists,
//! rejects duplicate requests, and requires native confirmation before the
//! local service is started.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use runhaven_core::apple_container::{self, INSTALLER_URL};
use runhaven_core::doctor::{
    PINNED_APPLE_CONTAINER_VERSION, parse_container_cli_version, parse_major_version,
};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use super::dialog::confirm_native;
use crate::contracts::{SetupActionId, SetupRecoveryRequest, SetupRecoveryResponse};

const RECOVERY_BUSY: &str = "Setup recovery is already running.";
const RECOVERY_CANCELED: &str = "Setup recovery canceled; no changes were made.";
const START_FAILED: &str =
    "RunHaven could not start the container service. Open System Check or try again.";
const NOT_READY: &str =
    "The container service did not become ready. Open System Check or try again.";
const INSTALLER_OPEN_FAILED: &str = "RunHaven could not open Apple's installer download. Check your internet connection or open System Check.";
const CONFIRM_FAILED: &str =
    "RunHaven could not show the setup confirmation. No changes were made.";
const INSTALLER_PROGRAM: &str = "/usr/bin/open";

#[derive(Clone, Debug)]
pub(crate) struct RecoveryCommandOutput {
    pub success: bool,
    pub text: String,
}

#[derive(Clone, Debug)]
pub(crate) struct RecoveryHost {
    pub operating_system: String,
    pub macos_major: Option<u32>,
    pub architecture: String,
    pub container_program: Option<PathBuf>,
}

#[derive(Debug)]
enum PreparedRecovery {
    OpenInstaller,
    StartSystem { container_program: PathBuf },
}

impl PreparedRecovery {
    fn action(&self) -> SetupActionId {
        match self {
            Self::OpenInstaller => SetupActionId::OpenAppleContainerInstaller,
            Self::StartSystem { .. } => SetupActionId::StartContainerSystem,
        }
    }
}

/// One setup mutation may be in flight. The mutex protects only the flag, so
/// no blocking lock guard is held across the native-dialog await.
#[derive(Debug, Default)]
pub(crate) struct SetupRecoveryGate {
    active: Mutex<bool>,
}

#[derive(Debug)]
pub(crate) struct SetupRecoveryLease<'a> {
    gate: &'a SetupRecoveryGate,
}

impl SetupRecoveryGate {
    pub(crate) fn try_begin(&self) -> Result<SetupRecoveryLease<'_>, String> {
        let mut active = self.active.lock().map_err(|_| RECOVERY_BUSY.to_string())?;
        if *active {
            return Err(RECOVERY_BUSY.to_string());
        }
        *active = true;
        Ok(SetupRecoveryLease { gate: self })
    }
}

impl Drop for SetupRecoveryLease<'_> {
    fn drop(&mut self) {
        match self.gate.active.lock() {
            Ok(mut active) => *active = false,
            Err(poisoned) => *poisoned.into_inner() = false,
        }
    }
}

#[tauri::command]
pub(crate) async fn perform_setup_recovery(
    app: AppHandle,
    request: SetupRecoveryRequest,
    gate: State<'_, SetupRecoveryGate>,
) -> Result<SetupRecoveryResponse, String> {
    let _lease = gate.try_begin()?;
    let action = request.action_id;
    let prepared = tauri::async_runtime::spawn_blocking(move || {
        let mut run = run_command;
        prepare_recovery(action, inspect_host(), &mut run)
    })
    .await
    .map_err(|_| "RunHaven could not check setup. Open System Check or try again.".to_string())??;

    if !trusted_setup_recovery_confirmed(app, action).await? {
        return Err(RECOVERY_CANCELED.to_string());
    }

    tauri::async_runtime::spawn_blocking(move || {
        let mut run = run_command;
        complete_recovery(prepared, inspect_host(), &mut run)
    })
    .await
    .map_err(|_| START_FAILED.to_string())?
}

fn inspect_host() -> RecoveryHost {
    let macos_major = (std::env::consts::OS == "macos")
        .then(|| {
            Command::new("/usr/bin/sw_vers")
                .arg("-productVersion")
                .output()
                .ok()
        })
        .flatten()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|version| parse_major_version(version.trim()));
    RecoveryHost {
        operating_system: std::env::consts::OS.to_string(),
        macos_major,
        architecture: std::env::consts::ARCH.to_string(),
        container_program: apple_container::resolve_program(),
    }
}

fn run_command(program: &Path, args: &[&str]) -> Result<RecoveryCommandOutput, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|_| "command failed".to_string())?;
    let text = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    Ok(RecoveryCommandOutput {
        success: output.status.success(),
        text: String::from_utf8_lossy(text).trim().to_string(),
    })
}

fn prepare_recovery(
    action: SetupActionId,
    host: RecoveryHost,
    run: &mut impl FnMut(&Path, &[&str]) -> Result<RecoveryCommandOutput, String>,
) -> Result<PreparedRecovery, String> {
    if host.operating_system != "macos" || host.macos_major.is_none_or(|major| major < 26) {
        return Err("RunHaven setup recovery requires macOS 26 or newer.".to_string());
    }
    if !matches!(host.architecture.as_str(), "arm64" | "aarch64") {
        return Err("RunHaven setup recovery requires an Apple silicon Mac.".to_string());
    }
    match action {
        SetupActionId::OpenAppleContainerInstaller => prepare_installer(host, run),
        SetupActionId::StartContainerSystem => prepare_start_system(host, run),
    }
}

fn prepare_installer(
    host: RecoveryHost,
    run: &mut impl FnMut(&Path, &[&str]) -> Result<RecoveryCommandOutput, String>,
) -> Result<PreparedRecovery, String> {
    let Some(program) = host.container_program else {
        return Ok(PreparedRecovery::OpenInstaller);
    };
    let installed = run(&program, &["--version"]);
    if installed.is_ok_and(|output| {
        output.success
            && parse_container_cli_version(&output.text).as_deref()
                == Some(PINNED_APPLE_CONTAINER_VERSION)
    }) {
        return Err("Apple container is already installed. Choose Check again.".to_string());
    }
    Ok(PreparedRecovery::OpenInstaller)
}

fn prepare_start_system(
    host: RecoveryHost,
    run: &mut impl FnMut(&Path, &[&str]) -> Result<RecoveryCommandOutput, String>,
) -> Result<PreparedRecovery, String> {
    let container_program = host.container_program.ok_or_else(|| {
        format!(
            "Apple container {PINNED_APPLE_CONTAINER_VERSION} is not installed. Choose Download Apple container."
        )
    })?;

    let version = run(&container_program, &["--version"]).map_err(|_| {
        format!(
            "RunHaven could not verify Apple container {PINNED_APPLE_CONTAINER_VERSION}. Open System Check."
        )
    })?;
    if !version.success
        || parse_container_cli_version(&version.text).as_deref()
            != Some(PINNED_APPLE_CONTAINER_VERSION)
    {
        return Err(format!(
            "RunHaven requires Apple container {PINNED_APPLE_CONTAINER_VERSION} before it can repair setup."
        ));
    }

    let status = run(&container_program, &["system", "status"]).map_err(|_| {
        "RunHaven could not check the container service. Open System Check.".to_string()
    })?;
    if status.success {
        return Err("The container service is already running. Check setup again.".to_string());
    }

    Ok(PreparedRecovery::StartSystem { container_program })
}

fn complete_recovery(
    prepared: PreparedRecovery,
    current_host: RecoveryHost,
    run: &mut impl FnMut(&Path, &[&str]) -> Result<RecoveryCommandOutput, String>,
) -> Result<SetupRecoveryResponse, String> {
    let action = prepared.action();
    let PreparedRecovery::StartSystem { container_program } = prepared else {
        prepare_installer(current_host, run)?;
        let opened = run(Path::new(INSTALLER_PROGRAM), &["-u", INSTALLER_URL])
            .map_err(|_| INSTALLER_OPEN_FAILED.to_string())?;
        if !opened.success {
            return Err(INSTALLER_OPEN_FAILED.to_string());
        }
        return Ok(SetupRecoveryResponse {
            action_id: action,
            status: "succeeded".to_string(),
            label: "Apple installer download opened".to_string(),
            detail: format!(
                "Your browser is downloading Apple's signed Apple container {PINNED_APPLE_CONTAINER_VERSION} installer. Open the .pkg, finish the steps, then return to RunHaven and choose Check again."
            ),
        });
    };

    let PreparedRecovery::StartSystem {
        container_program: current_program,
    } = prepare_start_system(current_host, run)?
    else {
        unreachable!("start preparation returns a start action")
    };
    if current_program != container_program {
        return Err(
            "Apple container changed while RunHaven was waiting. Choose Check again.".to_string(),
        );
    }
    let started = run(
        &container_program,
        &["system", "start", "--enable-kernel-install"],
    )
    .map_err(|_| START_FAILED.to_string())?;
    if !started.success {
        return Err(START_FAILED.to_string());
    }

    let ready =
        run(&container_program, &["system", "status"]).map_err(|_| NOT_READY.to_string())?;
    if !ready.success {
        return Err(NOT_READY.to_string());
    }

    Ok(SetupRecoveryResponse {
        action_id: action,
        status: "succeeded".to_string(),
        label: "Container service started".to_string(),
        detail: "RunHaven started the local container service.".to_string(),
    })
}

async fn trusted_setup_recovery_confirmed(
    app: AppHandle,
    action: SetupActionId,
) -> Result<bool, String> {
    match action {
        SetupActionId::OpenAppleContainerInstaller => confirm_native(
            app,
            "Download Apple container?",
            "Download",
            MessageDialogKind::Info,
            format!(
                "RunHaven will open Apple's signed Apple container {PINNED_APPLE_CONTAINER_VERSION} installer download in your browser. If another version is installed, review Apple's installer before replacing it. After it downloads, open the .pkg and follow Apple's steps. The installer will ask for your Mac administrator password."
            ),
        )
        .await
        .map_err(|_| CONFIRM_FAILED.to_string()),
        SetupActionId::StartContainerSystem => {
            confirm_native(
                app,
                "Start the container service?",
                "Start",
                MessageDialogKind::Info,
                "RunHaven will start Apple container's local service on this Mac. The first start may download and install Apple's runtime components, including its reviewed kernel and base file system. This does not change RunHaven's sandbox settings."
                    .to_string(),
            )
            .await
            .map_err(|_| CONFIRM_FAILED.to_string())
        }
    }
}

#[cfg(test)]
pub(crate) fn perform_setup_recovery_for_test(
    request: SetupRecoveryRequest,
    host: RecoveryHost,
    approve: impl FnOnce(SetupActionId) -> Result<bool, String>,
    run: impl FnMut(&Path, &[&str]) -> Result<RecoveryCommandOutput, String>,
) -> Result<SetupRecoveryResponse, String> {
    perform_setup_recovery_with_current_host_for_test(request, host.clone(), host, approve, run)
}

#[cfg(test)]
pub(crate) fn perform_setup_recovery_with_current_host_for_test(
    request: SetupRecoveryRequest,
    initial_host: RecoveryHost,
    current_host: RecoveryHost,
    approve: impl FnOnce(SetupActionId) -> Result<bool, String>,
    mut run: impl FnMut(&Path, &[&str]) -> Result<RecoveryCommandOutput, String>,
) -> Result<SetupRecoveryResponse, String> {
    let prepared = prepare_recovery(request.action_id, initial_host, &mut run)?;
    if !approve(request.action_id).map_err(|_| CONFIRM_FAILED.to_string())? {
        return Err(RECOVERY_CANCELED.to_string());
    }
    complete_recovery(prepared, current_host, &mut run)
}
