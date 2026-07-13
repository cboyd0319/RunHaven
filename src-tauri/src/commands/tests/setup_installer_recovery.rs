use super::{command_output, ready_host};
use crate::commands::check_status;
use crate::commands::setup_recovery::{
    RecoveryHost, perform_setup_recovery_for_test,
    perform_setup_recovery_with_current_host_for_test,
};
use crate::contracts::{SetupActionId, SetupRecoveryRequest};
use runhaven_core::doctor::Check;
use std::path::PathBuf;

#[test]
fn missing_apple_container_check_offers_only_the_fixed_installer_action() {
    let status = check_status(Check {
        name: "Apple container CLI".to_string(),
        ok: false,
        detail: "not found".to_string(),
        remedy: "Install Apple container 1.1.0.".to_string(),
    });

    let action = status.action.expect("missing CLI needs installer action");
    assert_eq!(action.id, SetupActionId::OpenAppleContainerInstaller);
    assert_eq!(action.label, "Download Apple container");
}

#[test]
fn wrong_apple_container_version_offers_the_required_signed_version() {
    let status = check_status(Check {
        name: "Apple container version".to_string(),
        ok: false,
        detail: "container CLI version 1.0.0; expected 1.1.0".to_string(),
        remedy: "Install Apple container 1.1.0.".to_string(),
    });

    let action = status.action.expect("wrong version needs a recovery path");
    assert_eq!(action.id, SetupActionId::OpenAppleContainerInstaller);
    assert_eq!(action.label, "Download required version");
}

#[test]
fn setup_recovery_request_accepts_the_fixed_apple_container_installer_action() {
    let request: SetupRecoveryRequest = serde_json::from_value(serde_json::json!({
        "actionId": "open-apple-container-installer"
    }))
    .expect("fixed installer action");

    assert_eq!(
        request.action_id,
        SetupActionId::OpenAppleContainerInstaller
    );
}

#[test]
fn approved_installer_action_opens_only_apples_pinned_signed_package() {
    let mut confirmed = None;
    let mut commands = Vec::new();
    let response = perform_setup_recovery_for_test(
        SetupRecoveryRequest {
            action_id: SetupActionId::OpenAppleContainerInstaller,
        },
        RecoveryHost {
            container_program: None,
            ..ready_host()
        },
        |action| {
            confirmed = Some(action);
            Ok(true)
        },
        |program, args| {
            commands.push((
                program.to_path_buf(),
                args.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ));
            Ok(command_output(true, ""))
        },
    )
    .expect("approved fixed installer action");

    assert_eq!(confirmed, Some(SetupActionId::OpenAppleContainerInstaller));
    assert_eq!(
        commands,
        vec![(
            PathBuf::from("/usr/bin/open"),
            vec![
                "-u".to_string(),
                "https://github.com/apple/container/releases/download/1.1.0/container-1.1.0-installer-signed.pkg".to_string(),
            ],
        )]
    );
    assert_eq!(response.status, "succeeded");
    assert!(response.detail.contains("browser"));
    assert!(!response.detail.contains("installed"));
}

#[test]
fn installer_action_can_open_the_required_package_over_a_wrong_version() {
    let mut commands = Vec::new();
    let response = perform_setup_recovery_for_test(
        SetupRecoveryRequest {
            action_id: SetupActionId::OpenAppleContainerInstaller,
        },
        ready_host(),
        |_| Ok(true),
        |program, args| {
            commands.push((program.to_path_buf(), args.join(" ")));
            match args {
                ["--version"] => Ok(command_output(true, "container CLI version 1.0.0")),
                ["-u", _] => Ok(command_output(true, "")),
                _ => panic!("unexpected installer recovery command {args:?}"),
            }
        },
    )
    .expect("wrong version may download the required signed package");

    assert_eq!(response.status, "succeeded");
    assert_eq!(
        commands
            .iter()
            .filter(|(_, args)| args == "--version")
            .count(),
        2
    );
    assert_eq!(
        commands.last().expect("open command").0,
        PathBuf::from("/usr/bin/open")
    );
}

#[test]
fn installer_action_stops_if_the_cli_appears_during_confirmation() {
    let error = perform_setup_recovery_with_current_host_for_test(
        SetupRecoveryRequest {
            action_id: SetupActionId::OpenAppleContainerInstaller,
        },
        RecoveryHost {
            container_program: None,
            ..ready_host()
        },
        ready_host(),
        |_| Ok(true),
        |_, args| match args {
            ["--version"] => Ok(command_output(true, "container CLI version 1.1.0")),
            _ => panic!("stale installer action must not open the browser"),
        },
    )
    .expect_err("CLI installed while confirmation was open");

    assert_eq!(
        error,
        "Apple container is already installed. Choose Check again."
    );
}
