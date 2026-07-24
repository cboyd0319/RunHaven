use super::*;

#[test]
fn foreground_runtime_launch_call_stays_in_ui_thread_handoff_owner() {
    let app_event_sender_source = include_str!("../app_event_sender.rs");
    let launch_handoff_source = include_str!("../runhaven/launch_handoff.rs");
    let runtime_launch_marker = ["launch", "_run", "_plan"].concat();
    let owners = tui_rust_sources()
        .into_iter()
        .filter_map(|path| {
            let relative = relative_tui_source(&path);
            let source = std::fs::read_to_string(&path).expect("source should be readable");
            source
                .contains(&runtime_launch_marker)
                .then(|| relative.display().to_string())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        owners,
        ["runhaven/launch_handoff.rs"],
        "only the UI-thread handoff owner may call the foreground runtime launch function"
    );
    assert!(
        launch_handoff_source.contains(&runtime_launch_marker)
            && launch_handoff_source.contains("with_restored("),
        "RunHaven foreground launch must have one UI-thread handoff owner that restores the Codex terminal before launch"
    );
    assert!(
        app_event_sender_source.contains("AppEvent::RunHavenLaunchPrepared { .. }")
            && app_event_sender_source.contains("session_log::log_inbound_app_event(&event)"),
        "RunHaven launch-prepared events carry full plan data and must stay excluded from Codex session logging"
    );
}

#[test]
fn run_control_stays_in_runhaven_service_with_typed_confirmation() {
    let service_runs_source = include_str!("../runhaven/service_runs.rs");
    let protocol_source = include_str!("../runhaven/protocol.rs");
    let session_source = include_str!("../runhaven/app_server_session.rs");
    let mvp_source = include_str!("../runhaven/mvp.rs");
    let mvp_input_source = include_str!("../runhaven/mvp_input.rs");
    let mvp_bottom_source = include_str!("../runhaven/mvp_bottom_pane.rs");
    let app_shell_source = include_str!("../app_shell.rs");

    for marker in [
        ["stop", "_active", "_run"].concat(),
        ["kill", "_active", "_run"].concat(),
        ["repair", "_active", "_run"].concat(),
    ] {
        let owners = tui_rust_sources()
            .into_iter()
            .filter_map(|path| {
                let relative = relative_tui_source(&path);
                if is_guard_source(&relative) {
                    return None;
                }
                let source = std::fs::read_to_string(&path).expect("source should be readable");
                source
                    .contains(&marker)
                    .then(|| relative.display().to_string())
            })
            .collect::<Vec<_>>();

        assert_eq!(
            owners,
            ["runhaven/service_runs.rs"],
            "direct core run-control call {marker:?} must stay in the RunHaven TUI service"
        );
    }

    assert!(
        service_runs_source.contains("validate_run_control_confirmation")
            && service_runs_source.contains("Confirm stop before stopping this run.")
            && service_runs_source.contains("Confirm hard stop before killing this run.")
            && service_runs_source
                .contains("Confirm repair before changing this active-run marker."),
        "run-control service calls must validate explicit confirmation before backend lookup"
    );
    assert!(
        protocol_source.contains("RunHavenRunStop")
            && protocol_source.contains("RunHavenRunKill")
            && protocol_source.contains("RunHavenRunRepair")
            && session_source.contains("RunHavenRunStop")
            && session_source.contains("RunHavenRunKill")
            && session_source.contains("RunHavenRunRepair"),
        "app-server facade must expose only the reviewed RunHaven run-control methods"
    );
    assert!(
        mvp_source.contains("RunControlState::Confirm")
            && mvp_input_source.contains("typed.trim() != screen.action.phrase()")
            && mvp_bottom_source.contains("Paste is ignored here."),
        "TUI run-control screens must keep separate typed confirmation and reject paste"
    );
    for marker in [
        "stop_active_run",
        "kill_active_run",
        "repair_active_run",
        "RunHavenRunStop",
        "RunHavenRunKill",
        "RunHavenRunRepair",
    ] {
        assert!(
            !app_shell_source.contains(marker),
            "app_shell.rs must not own run-control marker {marker:?}"
        );
    }
}

#[test]
fn run_diff_stays_in_runhaven_service_with_typed_confirmation() {
    let service_runs_source = include_str!("../runhaven/service_runs.rs");
    let protocol_source = include_str!("../runhaven/protocol.rs");
    let session_source = include_str!("../runhaven/app_server_session.rs");
    let mvp_input_source = include_str!("../runhaven/mvp_input.rs");
    let mvp_bottom_source = include_str!("../runhaven/mvp_bottom_pane.rs");
    let app_shell_source = include_str!("../app_shell.rs");

    let owners = tui_rust_sources()
        .into_iter()
        .filter_map(|path| {
            let relative = relative_tui_source(&path);
            if is_guard_source(&relative) {
                return None;
            }
            let source = std::fs::read_to_string(&path).expect("source should be readable");
            source
                .contains("run_diff_text")
                .then(|| relative.display().to_string())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        owners,
        ["runhaven/service_runs.rs"],
        "direct core run-diff calls must stay in the RunHaven TUI service"
    );
    assert!(
        service_runs_source.contains("validate_sensitive_diff_confirmation")
            && service_runs_source
                .contains("Confirm diff viewing before loading workspace file contents."),
        "run-diff service calls must validate explicit confirmation before backend lookup"
    );
    assert!(
        protocol_source.contains("RunHavenRunDiff")
            && protocol_source.contains("runhaven/run/diff")
            && session_source.contains("RunHavenRunDiff"),
        "app-server facade must expose only the reviewed RunHaven run-diff method"
    );
    assert!(
        mvp_input_source.contains("RunDiffState::Confirm")
            && mvp_input_source.contains("typed.trim() != DIFF_CONFIRM_PHRASE")
            && mvp_bottom_source.contains("Paste is ignored here."),
        "TUI run-diff screen must keep separate typed confirmation and reject paste"
    );
    for marker in ["run_diff_text", "RunHavenRunDiff", "runhaven/run/diff"] {
        assert!(
            !app_shell_source.contains(marker),
            "app_shell.rs must not own run-diff marker {marker:?}"
        );
    }
}

#[test]
fn app_event_shared_shrinks_only() {
    let source = include_str!("../app_event_shared.rs");
    let inline_modules = top_level_inline_module_declarations(source);

    assert_eq!(
        inline_modules,
        [
            "app",
            "app_server_session",
            "chatwidget",
            "goal_files",
            "hooks_rpc",
        ],
        "app_event_shared.rs may shrink as real Codex modules activate, but must not grow new bridge modules"
    );
    for marker in [
        "std::env",
        "std::fs",
        "std::process::Command",
        "reqwest",
        "runhaven_core::",
        "legacy_core",
    ] {
        assert!(
            !source.contains(marker),
            "app_event_shared.rs must stay an inert type bridge and not grow host-reaching marker {marker:?}"
        );
    }
}
