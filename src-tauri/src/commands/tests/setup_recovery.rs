use super::{command_output, ready_host};
use crate::commands::setup_recovery::{
    RecoveryHost, SetupRecoveryGate, perform_setup_recovery_for_test,
};
use crate::commands::{check_status, desktop_setup_checks};
use crate::contracts::{SetupActionId, SetupRecoveryRequest};
use runhaven_core::doctor::Check;
use std::path::PathBuf;

#[test]
fn desktop_setup_hides_the_source_build_rust_check() {
    let checks = desktop_setup_checks(vec![
        Check {
            name: "rust".to_string(),
            ok: true,
            detail: "0.7.0".to_string(),
            remedy: "Use the pinned Rust toolchain.".to_string(),
        },
        Check {
            name: "macOS".to_string(),
            ok: true,
            detail: "26.0".to_string(),
            remedy: "Use macOS 26 or newer.".to_string(),
        },
    ]);

    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].name, "macOS version");
}

#[test]
fn desktop_setup_merges_mac_checks_and_renames_the_processor() {
    let checks = desktop_setup_checks(vec![
        Check {
            name: "operating system".to_string(),
            ok: true,
            detail: "Darwin".to_string(),
            remedy: "Use macOS.".to_string(),
        },
        Check {
            name: "macOS".to_string(),
            ok: true,
            detail: "26.0".to_string(),
            remedy: "Use macOS 26 or newer.".to_string(),
        },
        Check {
            name: "architecture".to_string(),
            ok: true,
            detail: "arm64".to_string(),
            remedy: "Use Apple silicon.".to_string(),
        },
    ]);

    assert_eq!(checks.len(), 2);
    assert_eq!(checks[0].name, "macOS version");
    assert_eq!(checks[1].name, "Mac processor");
    assert!(checks.iter().all(|check| check.detail == "Ready."));
}

#[test]
fn failed_container_system_check_offers_only_the_fixed_start_action() {
    let status = check_status(Check {
        name: "container system".to_string(),
        ok: false,
        detail: "apiserver is not running".to_string(),
        remedy: "Run `container system start`.".to_string(),
    });

    let action = status.action.expect("failed service check needs recovery");
    assert_eq!(status.name, "Container service");
    assert_eq!(action.id, SetupActionId::StartContainerSystem);
    assert_eq!(action.label, "Start container service");

    let unrelated = check_status(Check {
        name: "macOS".to_string(),
        ok: false,
        detail: "25.0".to_string(),
        remedy: "Use macOS 26 or newer.".to_string(),
    });
    assert!(unrelated.action.is_none());
}

#[test]
fn desktop_setup_collapses_runtime_file_checks_into_one_reinstall_action() {
    let checks = desktop_setup_checks(vec![
        Check {
            name: "Apple container runtime commit".to_string(),
            ok: false,
            detail: "raw commit mismatch".to_string(),
            remedy: "developer remedy".to_string(),
        },
        Check {
            name: "Apple container builder image".to_string(),
            ok: false,
            detail: "raw builder mismatch".to_string(),
            remedy: "developer remedy".to_string(),
        },
        Check {
            name: "Apple container vminit image".to_string(),
            ok: true,
            detail: "raw vminit id".to_string(),
            remedy: "developer remedy".to_string(),
        },
        Check {
            name: "Apple container kernel".to_string(),
            ok: false,
            detail: "raw kernel mismatch".to_string(),
            remedy: "developer remedy".to_string(),
        },
    ]);

    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].name, "Apple container files");
    assert_eq!(
        checks[0].detail,
        "Apple container's files do not match the supported version."
    );
    let action = checks[0].action.as_ref().expect("reinstall action");
    assert_eq!(action.id, SetupActionId::OpenAppleContainerInstaller);
    assert_eq!(action.label, "Reinstall Apple container");
    let serialized = serde_json::to_string(&checks).expect("serialize checks");
    for technical_name in ["runtime commit", "builder image", "vminit", "kernel"] {
        assert!(!serialized.contains(technical_name));
    }
}

#[test]
fn desktop_setup_check_copy_never_exposes_raw_doctor_output() {
    let status = check_status(Check {
        name: "container system".to_string(),
        ok: false,
        detail: "appRoot /Users/example/Library/Application Support/com.apple.container"
            .to_string(),
        remedy: "Run `/usr/local/bin/container system start`.".to_string(),
    });

    let serialized = serde_json::to_string(&status).expect("serialize check");
    assert!(!serialized.contains("/Users/example"));
    assert!(!serialized.contains("/usr/local/bin"));
    assert_eq!(
        status.detail,
        "Apple container's local service is not running."
    );
    assert_eq!(status.remedy, "Start the service, then check again.");
}

#[test]
fn setup_recovery_request_accepts_only_an_opaque_supported_action_id() {
    let request: SetupRecoveryRequest = serde_json::from_value(serde_json::json!({
        "actionId": "start-container-system"
    }))
    .expect("supported opaque action id");
    assert_eq!(request.action_id, SetupActionId::StartContainerSystem);

    assert!(
        serde_json::from_value::<SetupRecoveryRequest>(serde_json::json!({
            "actionId": "delete-user-data"
        }))
        .is_err(),
        "unknown actions must fail before reaching command code"
    );
    assert!(
        serde_json::from_value::<SetupRecoveryRequest>(serde_json::json!({
            "actionId": "start-container-system",
            "executable": "/bin/rm",
            "args": ["-rf", "/"]
        }))
        .is_err(),
        "the WebView contract must reject executable and argument fields"
    );

    let restart: SetupRecoveryRequest = serde_json::from_value(serde_json::json!({
        "actionId": "restart-container-system"
    }))
    .expect("supported restart action id");
    assert_eq!(
        serde_json::to_string(&restart.action_id).expect("serialize restart action"),
        r#""restart-container-system""#
    );
}

#[test]
fn approved_setup_recovery_runs_the_exact_rust_owned_command() {
    let request = SetupRecoveryRequest {
        action_id: SetupActionId::StartContainerSystem,
    };
    let mut confirmed = None;
    let mut commands = Vec::new();
    let mut status_checks = 0;

    let response = perform_setup_recovery_for_test(
        request,
        ready_host(),
        |action| {
            confirmed = Some(action);
            Ok(true)
        },
        |program, args| {
            commands.push((
                program.to_path_buf(),
                args.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ));
            match args {
                ["--version"] => Ok(command_output(true, "container CLI version 1.1.0")),
                ["system", "status"] => {
                    status_checks += 1;
                    Ok(command_output(status_checks == 3, ""))
                }
                ["system", "start", "--enable-kernel-install"] => Ok(command_output(true, "")),
                _ => panic!("unexpected setup command {args:?}"),
            }
        },
    )
    .expect("approved supported recovery");

    assert_eq!(confirmed, Some(SetupActionId::StartContainerSystem));
    assert_eq!(
        commands
            .iter()
            .map(|(program, args)| (program.clone(), args.clone()))
            .collect::<Vec<_>>(),
        vec![
            (
                PathBuf::from("/Applications/container/bin/container"),
                vec!["--version".to_string()]
            ),
            (
                PathBuf::from("/Applications/container/bin/container"),
                vec!["system".to_string(), "status".to_string()]
            ),
            (
                PathBuf::from("/Applications/container/bin/container"),
                vec!["--version".to_string()]
            ),
            (
                PathBuf::from("/Applications/container/bin/container"),
                vec!["system".to_string(), "status".to_string()]
            ),
            (
                PathBuf::from("/Applications/container/bin/container"),
                vec![
                    "system".to_string(),
                    "start".to_string(),
                    "--enable-kernel-install".to_string()
                ]
            ),
            (
                PathBuf::from("/Applications/container/bin/container"),
                vec!["system".to_string(), "status".to_string()]
            ),
        ]
    );
    assert_eq!(
        serde_json::to_string(&response).expect("serialize response"),
        r#"{"actionId":"start-container-system","status":"succeeded","label":"Container service started","detail":"RunHaven started the local container service."}"#
    );
}

#[test]
fn setup_recovery_does_not_run_without_native_approval() {
    let error = perform_setup_recovery_for_test(
        SetupRecoveryRequest {
            action_id: SetupActionId::StartContainerSystem,
        },
        ready_host(),
        |_| Ok(false),
        |_, args| match args {
            ["--version"] => Ok(command_output(true, "container CLI version 1.1.0")),
            ["system", "status"] => Ok(command_output(false, "not running")),
            _ => panic!("denied setup recovery must not start the service"),
        },
    )
    .expect_err("native denial must fail closed");

    assert_eq!(error, "Setup recovery canceled; no changes were made.");
}

#[test]
fn setup_confirmation_failure_is_fixed_and_secret_free() {
    let error = perform_setup_recovery_for_test(
        SetupRecoveryRequest {
            action_id: SetupActionId::OpenAppleContainerInstaller,
        },
        RecoveryHost {
            container_program: None,
            ..ready_host()
        },
        |_| Err("/Users/alice token=secret".to_string()),
        |_, _| panic!("failed confirmation must not open anything"),
    )
    .expect_err("native confirmation failure");

    assert_eq!(
        error,
        "RunHaven could not show the setup confirmation. No changes were made."
    );
    assert!(!error.contains("/Users/"));
    assert!(!error.contains("secret"));
}

#[test]
fn setup_recovery_hides_process_errors_from_the_webview() {
    let error = perform_setup_recovery_for_test(
        SetupRecoveryRequest {
            action_id: SetupActionId::StartContainerSystem,
        },
        ready_host(),
        |_| Ok(true),
        |_, args| match args {
            ["--version"] => Ok(command_output(true, "container CLI version 1.1.0")),
            ["system", "status"] => Ok(command_output(false, "not running")),
            ["system", "start", "--enable-kernel-install"] => {
                Err("/Users/alice/private token=secret".to_string())
            }
            _ => panic!("unexpected setup command {args:?}"),
        },
    )
    .expect_err("runner failure");

    assert_eq!(
        error,
        "RunHaven could not start the container service. Open System details or try again."
    );
    assert!(!error.contains("/Users/"));
    assert!(!error.contains("secret"));
}

#[test]
fn setup_recovery_rejects_unsupported_hosts_before_any_process() {
    for host in [
        RecoveryHost {
            operating_system: "linux".to_string(),
            ..ready_host()
        },
        RecoveryHost {
            macos_major: Some(25),
            ..ready_host()
        },
        RecoveryHost {
            architecture: "x86_64".to_string(),
            ..ready_host()
        },
        RecoveryHost {
            container_program: None,
            ..ready_host()
        },
    ] {
        let error = perform_setup_recovery_for_test(
            SetupRecoveryRequest {
                action_id: SetupActionId::StartContainerSystem,
            },
            host,
            |_| panic!("unsupported host must not reach native confirmation"),
            |_, _| panic!("unsupported host must not run a process"),
        )
        .expect_err("unsupported host");
        assert!(!error.contains("/Users/"));
    }
}

#[test]
fn setup_recovery_rejects_unusable_or_wrong_container_versions() {
    for version_result in [
        command_output(false, "container CLI version 1.0.0"),
        command_output(true, "version unknown"),
        command_output(true, "container 1.1.0"),
    ] {
        let mut calls = 0;
        let error = perform_setup_recovery_for_test(
            SetupRecoveryRequest {
                action_id: SetupActionId::StartContainerSystem,
            },
            ready_host(),
            |_| panic!("wrong version must not reach native confirmation"),
            |_, args| {
                calls += 1;
                assert_eq!(args, ["--version"]);
                Ok(version_result.clone())
            },
        )
        .expect_err("wrong version");
        assert_eq!(calls, 1);
        assert!(error.contains("Apple container 1.1.0"));
    }
}

#[test]
fn setup_recovery_rejects_a_stale_action_when_service_is_already_ready() {
    let error = perform_setup_recovery_for_test(
        SetupRecoveryRequest {
            action_id: SetupActionId::StartContainerSystem,
        },
        ready_host(),
        |_| panic!("ready service must not reach native confirmation"),
        |_, args| match args {
            ["--version"] => Ok(command_output(true, "container CLI version 1.1.0")),
            ["system", "status"] => Ok(command_output(true, "running")),
            _ => panic!("stale action must not start the service"),
        },
    )
    .expect_err("stale recovery action");

    assert_eq!(
        error,
        "The container service is already running. Check setup again."
    );
}

#[test]
fn exit_zero_is_not_success_until_the_service_recheck_passes() {
    let mut status_checks = 0;
    let error = perform_setup_recovery_for_test(
        SetupRecoveryRequest {
            action_id: SetupActionId::StartContainerSystem,
        },
        ready_host(),
        |_| Ok(true),
        |_, args| match args {
            ["--version"] => Ok(command_output(true, "container CLI version 1.1.0")),
            ["system", "status"] => {
                status_checks += 1;
                Ok(command_output(false, "not running"))
            }
            ["system", "start", "--enable-kernel-install"] => Ok(command_output(true, "")),
            _ => panic!("unexpected setup command {args:?}"),
        },
    )
    .expect_err("post-start status is still failed");

    assert_eq!(status_checks, 3);
    assert_eq!(
        error,
        "The container service did not become ready. Open System details or try again."
    );
}

#[test]
fn setup_recovery_gate_rejects_duplicate_invokes_until_the_first_finishes() {
    let gate = SetupRecoveryGate::default();
    let first = gate.try_begin().expect("first recovery");
    assert_eq!(
        gate.try_begin().expect_err("duplicate recovery"),
        "Setup recovery is already running."
    );
    drop(first);
    assert!(gate.try_begin().is_ok(), "gate releases after completion");
}
