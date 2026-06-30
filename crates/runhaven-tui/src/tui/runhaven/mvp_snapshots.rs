use super::*;
use crate::test_backend::VT100Backend;
use crate::tui::runhaven::service::AgentLaunchPreview;
use crate::tui::runhaven::service::CURRENT_DIRECTORY_WORKSPACE_LABEL;
use crate::tui::runhaven::service::GIT_REPOSITORY_ROOT_WORKSPACE_DESCRIPTION;
use crate::tui::runhaven::service::GIT_REPOSITORY_ROOT_WORKSPACE_LABEL;
use crate::tui::runhaven::service::LaunchPreviewPayload;
use crate::tui::runhaven::service::WorkspaceLaunchPreview;
use ratatui::Terminal;
use runhaven_core::ui_contracts::AgentCatalogItemData;
use runhaven_core::ui_contracts::AuthDecisionData;
use runhaven_core::ui_contracts::AuthProfileStatusData;
use runhaven_core::ui_contracts::AuthStatusData;
use runhaven_core::ui_contracts::EgressDecisionData;
use runhaven_core::ui_contracts::LaunchBoundaryData;
use runhaven_core::ui_contracts::LaunchNetworkData;
use runhaven_core::ui_contracts::LaunchPlanData;
use runhaven_core::ui_contracts::RunHistoryListData;
use runhaven_core::ui_contracts::RunHistorySummaryData;

const SNAPSHOT_WORKSPACE: &str = "/selected-workspace";
const SNAPSHOT_NESTED_WORKSPACE: &str = "/selected-workspace/app";

#[test]
fn runhaven_mvp_snapshot_matrix() {
    snapshot_workspace_picker("runhaven_mvp_workspace_picker_80x24", 80, 24);
    snapshot_workspace_picker("runhaven_mvp_workspace_picker_120x48", 120, 48);
    snapshot_workspace_picker_repo_root_selected(
        "runhaven_mvp_workspace_picker_repo_root_80x24",
        80,
        24,
    );
    snapshot_workspace_picker_repo_root_selected(
        "runhaven_mvp_workspace_picker_repo_root_120x48",
        120,
        48,
    );

    snapshot_launch_step("runhaven_mvp_agent_picker_80x24", 80, 24, |_| {});
    snapshot_launch_step("runhaven_mvp_agent_picker_120x48", 120, 48, |_| {});

    snapshot_launch_step("runhaven_mvp_review_80x24", 80, 24, |view| {
        view.handle_key_event(key(KeyCode::Enter));
    });
    snapshot_launch_step("runhaven_mvp_review_120x48", 120, 48, |view| {
        view.handle_key_event(key(KeyCode::Enter));
    });

    snapshot_launch_step("runhaven_mvp_confirm_80x24", 80, 24, |view| {
        view.handle_key_event(key(KeyCode::Enter));
        view.handle_key_event(key(KeyCode::Enter));
    });
    snapshot_launch_step("runhaven_mvp_confirm_120x48", 120, 48, |view| {
        view.handle_key_event(key(KeyCode::Enter));
        view.handle_key_event(key(KeyCode::Enter));
    });

    snapshot_confirm_required_step("runhaven_mvp_typed_confirm_80x24", 80, 24);
    snapshot_confirm_required_step("runhaven_mvp_typed_confirm_120x48", 120, 48);

    snapshot_static_screen("runhaven_mvp_active_runs_80x24", 80, 24, active_runs_view());
    snapshot_static_screen(
        "runhaven_mvp_active_runs_120x48",
        120,
        48,
        active_runs_view(),
    );
    snapshot_static_screen(
        "runhaven_mvp_log_confirmation_80x24",
        80,
        24,
        log_confirmation_view(),
    );
    snapshot_static_screen(
        "runhaven_mvp_log_confirmation_120x48",
        120,
        48,
        log_confirmation_view(),
    );
    snapshot_static_screen(
        "runhaven_mvp_loaded_log_snapshot_80x24",
        80,
        24,
        loaded_log_snapshot_view(),
    );
    snapshot_static_screen(
        "runhaven_mvp_loaded_log_snapshot_120x48",
        120,
        48,
        loaded_log_snapshot_view(),
    );
    snapshot_static_screen("runhaven_mvp_history_80x24", 80, 24, history_view());
    snapshot_static_screen("runhaven_mvp_history_120x48", 120, 48, history_view());
    snapshot_static_screen("runhaven_mvp_diagnostics_80x24", 80, 24, diagnostics_view());
    snapshot_static_screen(
        "runhaven_mvp_diagnostics_120x48",
        120,
        48,
        diagnostics_view(),
    );

    snapshot_post_run("runhaven_mvp_recovery_80x24", 80, 24);
    snapshot_post_run("runhaven_mvp_recovery_120x48", 120, 48);
}

fn snapshot_launch_step(
    name: &str,
    width: u16,
    height: u16,
    drive: impl FnOnce(&mut RunHavenMvpView),
) {
    let mut view = snapshot_launch_view(vec![
        launch_preview("antigravity", false),
        launch_preview("shell", true),
    ]);
    drive(&mut view);
    assert_snapshot(name, &view, width, height);
}

fn snapshot_confirm_required_step(name: &str, width: u16, height: u16) {
    let mut view = snapshot_launch_view(vec![launch_preview("shell", true)]);
    view.handle_key_event(key(KeyCode::Enter));
    view.handle_key_event(key(KeyCode::Enter));
    assert_snapshot(name, &view, width, height);
}

fn snapshot_workspace_picker(name: &str, width: u16, height: u16) {
    assert_workspace_picker_snapshot(name, width, height, |_| {});
}

fn snapshot_workspace_picker_repo_root_selected(name: &str, width: u16, height: u16) {
    assert_workspace_picker_snapshot(name, width, height, |view| {
        view.handle_key_event(key(KeyCode::Down));
    });
}

fn assert_workspace_picker_snapshot(
    name: &str,
    width: u16,
    height: u16,
    drive: impl FnOnce(&mut RunHavenMvpView),
) {
    let mut view = RunHavenMvpView::from_launch_wizard_for_tests(
        SNAPSHOT_NESTED_WORKSPACE.into(),
        LaunchWizardView::new_with_workspace_choices(vec![
            WorkspaceLaunchPreview {
                label: CURRENT_DIRECTORY_WORKSPACE_LABEL.to_string(),
                description: SNAPSHOT_NESTED_WORKSPACE.to_string(),
                payload: LaunchPreviewPayload {
                    workspace: SNAPSHOT_NESTED_WORKSPACE.into(),
                    previews: vec![launch_preview_for_workspace(
                        "codex",
                        false,
                        SNAPSHOT_NESTED_WORKSPACE,
                    )],
                },
            },
            WorkspaceLaunchPreview {
                label: GIT_REPOSITORY_ROOT_WORKSPACE_LABEL.to_string(),
                description: GIT_REPOSITORY_ROOT_WORKSPACE_DESCRIPTION.to_string(),
                payload: LaunchPreviewPayload {
                    workspace: SNAPSHOT_WORKSPACE.into(),
                    previews: vec![launch_preview_for_workspace(
                        "codex",
                        false,
                        SNAPSHOT_WORKSPACE,
                    )],
                },
            },
        ]),
    );
    drive(&mut view);
    assert_snapshot(name, &view, width, height);
}

fn snapshot_static_screen(name: &str, width: u16, height: u16, view: RunHavenMvpView) {
    assert_snapshot(name, &view, width, height);
}

fn snapshot_post_run(name: &str, width: u16, height: u16) {
    let launch = launch_preview("codex", false).plan.expect("codex plan");
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.show_post_run(PostRunOutcome::from_launch(&launch, 7, None));

    assert_snapshot(name, &view, width, height);
}

fn snapshot_launch_view(previews: Vec<AgentLaunchPreview>) -> RunHavenMvpView {
    RunHavenMvpView::from_launch_wizard_for_tests(
        SNAPSHOT_WORKSPACE.into(),
        LaunchWizardView::new(SNAPSHOT_WORKSPACE.into(), previews),
    )
}

fn assert_snapshot(name: &str, view: &RunHavenMvpView, width: u16, height: u16) {
    let mut terminal = Terminal::new(VT100Backend::new(width, height)).expect("terminal");
    terminal
        .draw(|frame| view.render(frame.area(), frame.buffer_mut()))
        .expect("draw");
    let rendered = terminal.backend().to_string();

    insta::with_settings!({snapshot_path => "../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_snapshot!(name, rendered);
    });
}

fn active_runs_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::ActiveRuns(Box::new(ActiveRunsScreen {
        runs: ActiveRunListData {
            runs: vec![active_run()],
        },
        selected_idx: 0,
        notice: None,
    }));
    view
}

fn log_confirmation_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::RunLogs(Box::new(RunLogsScreen {
        run: active_run(),
        state: RunLogsState::Confirm {
            typed: String::new(),
            notice: None,
        },
    }));
    view
}

fn loaded_log_snapshot_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::RunLogs(Box::new(RunLogsScreen {
        run: active_run(),
        state: RunLogsState::Loaded(ActiveRunLogSnapshotData {
            run_id: "run-20260629-001".to_string(),
            captured_at: "2026-06-29T17:45:00Z".to_string(),
            requested_lines: 20,
            text: [
                "RunHaven started codex in /workspace.",
                "Provider egress: allowed api.openai.com:443.",
                "Agent exited with status 0.",
            ]
            .join("\n"),
            returned_lines: 3,
            truncated: false,
            source: "container-stdio".to_string(),
            warnings: vec![
                "Raw container output can contain secrets or workspace content.".to_string(),
            ],
        }),
    }));
    view
}

fn diagnostics_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::Diagnostics(Box::new(DiagnosticsScreen {
        result: Ok(RunHavenDiagnosticsData {
            auth_status: AuthStatusData {
                status: "ready".to_string(),
                runtime: "runhaven isolated login state".to_string(),
                profiles: vec![
                    AuthProfileStatusData {
                        name: "codex".to_string(),
                        status: "signed in".to_string(),
                    },
                    AuthProfileStatusData {
                        name: "claude".to_string(),
                        status: "login needed".to_string(),
                    },
                ],
            },
            egress_log: vec![EgressDecisionData {
                timestamp: "2026-06-29T17:45:00Z".to_string(),
                profile: "codex".to_string(),
                decision: "allowed".to_string(),
                host: "api.openai.com".to_string(),
                port: 443,
                count: 3,
                reason: "provider-allowlist".to_string(),
                matched_rule: "api.openai.com".to_string(),
                run_id: "run-20260629-001".to_string(),
            }],
            auth_log: vec![AuthDecisionData {
                timestamp: "2026-06-29T17:45:01Z".to_string(),
                profile: "codex".to_string(),
                broker: "api-key".to_string(),
                decision: "allowed".to_string(),
                method: "POST".to_string(),
                path: "/v1/responses".to_string(),
                upstream_status: Some(200),
                count: 1,
                reason: "brokered".to_string(),
                run_id: "run-20260629-001".to_string(),
            }],
        }),
    }));
    view
}

fn history_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::History(Box::new(HistoryScreen {
        result: Ok(RunHistoryListData {
            runs: vec![
                RunHistorySummaryData {
                    run_id: "run-20260629-003".to_string(),
                    profile: "codex".to_string(),
                    network: "provider".to_string(),
                    status: "failed".to_string(),
                    started_at: "2026-06-29T18:10:00Z".to_string(),
                    finished_at: "2026-06-29T18:13:00Z".to_string(),
                    return_code: Some(1),
                    workspace_scope: "current".to_string(),
                    session: "none".to_string(),
                    state_volume: "runhaven-codex-shared-home".to_string(),
                    provider_allowed: 2,
                    provider_denied: 1,
                    auth_allowed: 0,
                    auth_denied: 1,
                    cleanup_provider_network: "removed".to_string(),
                    git_summary: "Git: changed=true before=abc1234 after=def5678 files=12"
                        .to_string(),
                    worktree_branch: Some(
                        "runhaven/codex/run-20260629-003-long-worktree-branch".to_string(),
                    ),
                    review_command: "runhaven runs show run-20260629-003".to_string(),
                },
                RunHistorySummaryData {
                    run_id: "run-20260629-002".to_string(),
                    profile: "antigravity".to_string(),
                    network: "provider".to_string(),
                    status: "succeeded".to_string(),
                    started_at: "2026-06-29T17:50:00Z".to_string(),
                    finished_at: "2026-06-29T17:59:00Z".to_string(),
                    return_code: Some(0),
                    workspace_scope: "repository".to_string(),
                    session: "agent".to_string(),
                    state_volume: "runhaven-antigravity-shared-home".to_string(),
                    provider_allowed: 8,
                    provider_denied: 0,
                    auth_allowed: 2,
                    auth_denied: 0,
                    cleanup_provider_network: "removed".to_string(),
                    git_summary: "Git: changed=true before=1234567 after=89abcde files=3"
                        .to_string(),
                    worktree_branch: Some(
                        "runhaven/antigravity/run-20260629-002-review-branch".to_string(),
                    ),
                    review_command: "runhaven runs show run-20260629-002".to_string(),
                },
                RunHistorySummaryData {
                    run_id: "run-20260629-001".to_string(),
                    profile: "shell".to_string(),
                    network: "internet".to_string(),
                    status: "succeeded".to_string(),
                    started_at: "2026-06-29T17:44:00Z".to_string(),
                    finished_at: "2026-06-29T17:45:00Z".to_string(),
                    return_code: None,
                    workspace_scope: "current".to_string(),
                    session: "none".to_string(),
                    state_volume: "runhaven-shell-shared-home".to_string(),
                    provider_allowed: 0,
                    provider_denied: 0,
                    auth_allowed: 0,
                    auth_denied: 0,
                    cleanup_provider_network: "n/a".to_string(),
                    git_summary: "Git: unavailable (not-a-git-worktree)".to_string(),
                    worktree_branch: None,
                    review_command: "runhaven runs show run-20260629-001".to_string(),
                },
            ],
        }),
        selected_idx: 1,
    }));
    view
}

fn active_run() -> ActiveRunSummaryData {
    ActiveRunSummaryData {
        run_id: "run-20260629-001".to_string(),
        profile: "codex".to_string(),
        network: "provider".to_string(),
        status: "running".to_string(),
        timestamp: "2026-06-29T17:44:00Z".to_string(),
        state_volume: "runhaven-codex-shared-home".to_string(),
        session: "none".to_string(),
        container_name: "runhaven-codex-001-run".to_string(),
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn launch_preview(name: &str, confirm_required: bool) -> AgentLaunchPreview {
    launch_preview_for_workspace(name, confirm_required, SNAPSHOT_WORKSPACE)
}

fn launch_preview_for_workspace(
    name: &str,
    confirm_required: bool,
    workspace: &str,
) -> AgentLaunchPreview {
    use runhaven_core::runtime::plans::AgentRunPlan;
    use runhaven_core::runtime::plans::AuthScope;
    use runhaven_core::runtime::plans::NetworkMode;
    use runhaven_core::runtime::plans::WorkspaceScope;

    let network_mode = if confirm_required {
        NetworkMode::Internet
    } else {
        NetworkMode::Provider
    };
    let network_summary = if confirm_required {
        "unrestricted internet"
    } else {
        "provider allowlist"
    };
    let provider_allowed_hosts = if confirm_required {
        Vec::new()
    } else {
        vec![
            "api.openai.com".to_string(),
            "oauth2.googleapis.com".to_string(),
        ]
    };
    let safety_notes = if confirm_required {
        vec!["Unrestricted internet can reach hosts outside the provider allowlist.".to_string()]
    } else {
        Vec::new()
    };
    let image = format!("runhaven/{name}:0.1.0");
    let command = vec![
        "container".to_string(),
        "run".to_string(),
        "--rm".to_string(),
        "--init".to_string(),
        "--name".to_string(),
        format!("runhaven-{name}-snapshot-run"),
        "--read-only".to_string(),
        "--mount".to_string(),
        format!("type=bind,source={workspace},target=/workspace"),
        "--network".to_string(),
        if confirm_required {
            "internet".to_string()
        } else {
            "runhaven-provider".to_string()
        },
        image.clone(),
        name.to_string(),
    ];
    let executable = AgentRunPlan {
        command,
        preflight: Vec::new(),
        workspace: workspace.into(),
        state_volume: format!("runhaven-{name}-shared-home"),
        session: "none".to_string(),
        container_name: format!("runhaven-{name}-snapshot-run"),
        profile_name: name.to_string(),
        workspace_scope: WorkspaceScope::Current,
        workspace_scope_note: None,
        auth_scope: AuthScope::Agent,
        worktree: None,
        run_id: None,
        network_name: (!confirm_required).then(|| "runhaven-provider".to_string()),
        network_mode,
        egress_summary: network_summary.to_string(),
        image: image.clone(),
        provider_allowed_hosts: provider_allowed_hosts.clone(),
        api_key_broker_env: None,
        security_notices: safety_notes.clone(),
    };
    let data = LaunchPlanData {
        profile_name: name.to_string(),
        workspace: workspace.to_string(),
        workspace_scope: "current".to_string(),
        workspace_scope_note: None,
        auth_scope: "agent".to_string(),
        session: "none".to_string(),
        state_volume: format!("runhaven-{name}-shared-home"),
        container_name: format!("runhaven-{name}-snapshot-run"),
        image: image.clone(),
        worktree: None,
        network: LaunchNetworkData {
            mode: network_mode.as_str().to_string(),
            name: executable.network_name.clone(),
            summary: network_summary.to_string(),
            provider_allowed_hosts,
            api_key_broker_env: None,
        },
        boundary: LaunchBoundaryData {
            mounted_workspace: format!("{workspace} -> /workspace"),
            mounted_state_volume: format!("runhaven-{name}-shared-home -> /home/agent"),
            not_shared: vec![
                "host home folder".to_string(),
                "raw SSH keys".to_string(),
                "browser profiles".to_string(),
                "cloud credential folders".to_string(),
                "arbitrary host environment variables".to_string(),
            ],
        },
        preflight_commands: Vec::new(),
        command: executable.shell_command(),
        safety_notes,
        confirm_required,
    };

    AgentLaunchPreview {
        agent: AgentCatalogItemData {
            name: name.to_string(),
            description: format!("{name} snapshot profile"),
            image,
            sign_in: format!("runhaven login {name}"),
            broker: "no".to_string(),
            default_network: network_mode.as_str().to_string(),
            provider_host_count: if confirm_required { 0 } else { 2 },
        },
        plan: Ok(PreparedLaunch::from_parts_for_tests(data, executable)),
    }
}
