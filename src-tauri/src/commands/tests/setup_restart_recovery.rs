use super::{command_output, ready_host};
use crate::commands::restart_system_confirmation_message_for_test;
use crate::commands::setup_recovery::perform_setup_recovery_for_test;
use crate::contracts::{SetupActionId, SetupRecoveryRequest};

#[test]
fn approved_restart_runs_the_exact_stop_start_readiness_sequence() {
    let request = SetupRecoveryRequest {
        action_id: SetupActionId::RestartContainerSystem,
    };
    let mut commands = Vec::new();

    let response = perform_setup_recovery_for_test(
        request,
        ready_host(),
        |_| Ok(true),
        |program, args| {
            commands.push((program.to_path_buf(), args.join(" ")));
            match args {
                ["--version"] => Ok(command_output(true, "container CLI version 1.1.0")),
                ["system", "status"] => Ok(command_output(true, "running")),
                ["system", "stop"] | ["system", "start", "--enable-kernel-install"] => {
                    Ok(command_output(true, ""))
                }
                _ => panic!("unexpected restart command {args:?}"),
            }
        },
    )
    .expect("approved restart");

    let actions = commands
        .iter()
        .map(|(_, args)| args.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        actions,
        vec![
            "--version",
            "system status",
            "--version",
            "system status",
            "system stop",
            "system start --enable-kernel-install",
            "system status",
        ]
    );
    assert_eq!(response.label, "Container service restarted");
}

#[test]
fn restart_failure_reports_the_failed_phase_and_stops_the_sequence() {
    for (failed_phase, expected_error, expected_commands) in [
        (
            "stop",
            "RunHaven could not stop the container service, so nothing was restarted. Check that no other container work is running, then try again.",
            5,
        ),
        (
            "start",
            "The container service stopped, but RunHaven could not start it again. Choose Check again, then Start container service.",
            6,
        ),
        (
            "ready",
            "The container service restarted but is not ready. Choose Check again to see the next fix.",
            7,
        ),
    ] {
        for runner_error in [true, false] {
            let request = SetupRecoveryRequest {
                action_id: SetupActionId::RestartContainerSystem,
            };
            let mut commands = Vec::new();
            let mut status_checks = 0;
            let failed_command = || {
                if runner_error {
                    Err("/Users/alice/private token=secret".to_string())
                } else {
                    Ok(command_output(false, "/Users/alice/private token=secret"))
                }
            };

            let error = perform_setup_recovery_for_test(
                request,
                ready_host(),
                |_| Ok(true),
                |_, args| {
                    commands.push(args.join(" "));
                    match args {
                        ["--version"] => Ok(command_output(true, "container CLI version 1.1.0")),
                        ["system", "status"] => {
                            status_checks += 1;
                            if failed_phase == "ready" && status_checks == 3 {
                                failed_command()
                            } else {
                                Ok(command_output(true, "running"))
                            }
                        }
                        ["system", "stop"] if failed_phase == "stop" => failed_command(),
                        ["system", "start", "--enable-kernel-install"]
                            if failed_phase == "start" =>
                        {
                            failed_command()
                        }
                        ["system", "stop"] | ["system", "start", "--enable-kernel-install"] => {
                            Ok(command_output(true, ""))
                        }
                        _ => panic!("unexpected restart command {args:?}"),
                    }
                },
            )
            .expect_err("restart phase failure");

            assert_eq!(error, expected_error);
            assert_eq!(commands.len(), expected_commands);
            assert!(!error.contains("/Users/"));
            assert!(!error.contains("secret"));
        }
    }
}

#[test]
fn restart_confirmation_names_every_session_it_ends() {
    let message = restart_system_confirmation_message_for_test();

    assert!(message.contains("Current RunHaven sessions will end."));
    assert!(message.contains("outside RunHaven will also stop"));
}
