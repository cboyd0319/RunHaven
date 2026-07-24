use super::*;

#[test]
fn active_run_log_snapshot_route_stays_in_runhaven_facade() {
    let active_snapshot_marker = ["active_run_", "log_snapshot_payload"].concat();
    let method_marker = ["runhaven/run/", "logSnapshot"].concat();
    let service_requests_source = include_str!("../runhaven/service_requests.rs");
    let service_runs_source = include_str!("../runhaven/service_runs.rs");
    let protocol_source = include_str!("../runhaven/protocol.rs");
    let session_source = include_str!("../runhaven/app_server_session.rs");
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
                .contains(&active_snapshot_marker)
                .then(|| relative.display().to_string())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        owners,
        ["runhaven/service_runs.rs"],
        "only the RunHaven TUI service facade may call the active-run log snapshot runtime API"
    );
    assert!(
        protocol_source.contains(&method_marker)
            && session_source.contains("run_log_snapshot")
            && service_requests_source.contains("self.active_run_log_snapshot_data(")
            && service_runs_source.contains(
                "self.validate_sensitive_log_confirmation(confirm_sensitive_output, method)?;"
            )
            && service_runs_source.contains("self.validate_log_snapshot_lines(lines, method)?;")
            && service_runs_source
                .contains("self.active_run_log_snapshot_payload(run_id, lines, method)"),
        "log snapshots must stay a typed RunHaven method with sensitive-output confirmation and validation before backend lookup"
    );
    for marker in [
        method_marker.as_str(),
        active_snapshot_marker.as_str(),
        "run_log_snapshot",
    ] {
        assert!(
            !app_shell_source.contains(marker),
            "the temporary app_shell must not grow active-run log product behavior or direct runtime calls"
        );
    }
}

#[test]
fn visible_run_log_ui_stays_confirmation_gated_in_runhaven_mvp() {
    let mvp_source = include_str!("../runhaven/mvp.rs");
    let mvp_input_source = include_str!("../runhaven/mvp_input.rs");
    let mvp_render_source = include_str!("../runhaven/mvp_render_runs.rs");
    let app_shell_source = include_str!("../app_shell.rs");
    let log_text_marker = ["snapshot", ".text.lines().take"].concat();
    let owners = tui_rust_sources()
        .into_iter()
        .filter_map(|path| {
            let relative = relative_tui_source(&path);
            if is_guard_source(&relative) {
                return None;
            }
            let source = std::fs::read_to_string(&path).expect("source should be readable");
            source
                .contains(&log_text_marker)
                .then(|| relative.display().to_string())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        owners,
        ["runhaven/mvp_render_runs.rs"],
        "raw log text rendering must stay in the RunHaven MVP render view"
    );
    assert!(
        mvp_source.contains("LOG_CONFIRM_PHRASE")
            && mvp_input_source.contains("typed.trim() != LOG_CONFIRM_PHRASE")
            && mvp_input_source.contains("active_run_log_snapshot_data(")
            && mvp_input_source.contains("\"runhaven/run/logSnapshot\"")
            && mvp_render_source.contains("Raw container output can contain secrets")
            && mvp_render_source.contains(&log_text_marker),
        "visible log UI must require typed confirmation before loading raw output"
    );
    for marker in [
        "Raw container output",
        log_text_marker.as_str(),
        "active_run_log_snapshot_data",
    ] {
        assert!(
            !app_shell_source.contains(marker),
            "app_shell.rs must not render or load active-run logs"
        );
    }
}

#[test]
fn chatwidget_branch_summary_uses_source_first_boundary() {
    let module_source = include_str!("../mod.rs");
    let bridge_source = include_str!("../app_event_shared.rs");
    let branch_summary_source = include_str!("../branch_summary.rs");
    let workspace_command_source = include_str!("../workspace_command.rs");
    let app_shell_source = include_str!("../app_shell.rs");
    let runhaven_sources = [
        include_str!("../runhaven/app_server_client.rs"),
        include_str!("../runhaven/app_server_session.rs"),
        include_str!("../runhaven/launch_handoff.rs"),
        include_str!("../runhaven/protocol.rs"),
        include_str!("../runhaven/service.rs"),
        include_str!("../runhaven/launch_wizard.rs"),
        include_str!("../runhaven/mvp.rs"),
        include_str!("../runhaven/terminal_handoff.rs"),
    ];

    assert!(
        module_declared(module_source, "branch_summary")
            && module_declared(module_source, "workspace_command"),
        "ChatWidget status promotion must use the real branch_summary.rs plus workspace_command.rs source boundary"
    );
    assert!(
        !bridge_source.contains("StatusLineGitSummary"),
        "app_event_shared.rs must not keep the StatusLineGitSummary bridge once branch_summary.rs is active"
    );
    for marker in [
        "std::process::Command",
        "use tokio::process",
        "tokio::process::Command::new",
        "std::env",
        "std::fs",
        "runhaven_core::",
    ] {
        assert!(
            !branch_summary_source.contains(marker),
            "branch_summary.rs must stay best-effort metadata over WorkspaceCommandExecutor, not direct host access marker {marker:?}"
        );
    }
    assert!(
        workspace_command_source
            .contains("#[cfg(any())]\npub(crate) struct AppServerWorkspaceCommandRunner")
            && workspace_command_source.contains("ClientRequest::OneOffCommandExec"),
        "workspace_command.rs may carry the upstream app-server runner only while compiled dormant"
    );
    assert!(
        !app_shell_source.contains("WorkspaceCommandRunner")
            && !app_shell_source.contains("AppServerWorkspaceCommandRunner"),
        "the temporary app_shell must not start app-server workspace command execution"
    );
    for source in runhaven_sources {
        assert!(
            !source.contains("AppServerWorkspaceCommandRunner"),
            "RunHaven-owned adapters must not activate app-server workspace command execution in this slice"
        );
    }
}

#[test]
fn token_usage_uses_source_first_boundary_before_full_status() {
    let module_source = include_str!("../mod.rs");
    let root_lib_source = include_str!("../../lib.rs");
    let footer_source = include_str!("../bottom_pane/footer.rs");
    let hooks_browser_source = include_str!("../bottom_pane/hooks_browser_view.rs");

    assert!(
        module_declared(module_source, "token_usage"),
        "ChatWidget/status integration must use the real token_usage.rs source model"
    );
    assert!(
        !module_declared(module_source, "status"),
        "Keep full status/ dormant until its config, model-provider, remote app-server, and status-card closure is promoted"
    );
    assert!(
        !root_lib_source.contains("pub(crate) use tui::status;"),
        "do not re-export a fake root status surface before full status/ is intentionally promoted"
    );
    for source in [footer_source, hooks_browser_source] {
        assert!(
            !source.contains("crate::status::") && !source.contains("use crate::status"),
            "active bottom-pane code must use the small RunHaven status_format helper instead of a root status bridge"
        );
    }
}

#[test]
fn session_log_uses_source_first_boundary_without_active_recording() {
    let module_source = include_str!("../mod.rs");
    let bridge_source = include_str!("../app_event_shared.rs");

    assert!(
        module_declared(module_source, "session_log"),
        "ChatWidget/AppEvent promotion must use the real vendored session_log.rs source"
    );
    assert!(
        !bridge_source.contains("mod session_log"),
        "app_event_shared.rs must not keep a session_log bridge once real session_log.rs is active"
    );

    let maybe_init_marker = ["session_log::", "maybe_init"].concat();
    let recording_env_marker = ["CODEX_TUI_", "RECORD_SESSION"].concat();
    for path in tui_rust_sources() {
        let relative = relative_tui_source(&path);
        if matches!(relative.to_str(), Some("session_log.rs") | Some("lib.rs")) {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("source should be readable");
        assert!(
            !source.contains(&maybe_init_marker) && !source.contains(&recording_env_marker),
            "only session_log.rs and dormant tui/lib.rs may mention Codex session recording markers; found one in {}",
            relative.display()
        );
    }
}
