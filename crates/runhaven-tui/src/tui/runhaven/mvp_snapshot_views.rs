use runhaven_core::ui_contracts::ActiveRunListData;
use runhaven_core::ui_contracts::ActiveRunLogSnapshotData;
use runhaven_core::ui_contracts::ActiveRunSummaryData;
use runhaven_core::ui_contracts::AuthDecisionData;
use runhaven_core::ui_contracts::AuthProfileStatusData;
use runhaven_core::ui_contracts::AuthStatusData;
use runhaven_core::ui_contracts::DoctorCheckData;
use runhaven_core::ui_contracts::EgressDecisionData;
use runhaven_core::ui_contracts::RunControlResultData;
use runhaven_core::ui_contracts::RunDiffData;
use runhaven_core::ui_contracts::RunHavenDiagnosticsData;
use runhaven_core::ui_contracts::RunHistoryListData;
use runhaven_core::ui_contracts::RunHistorySummaryData;

use super::super::*;
use super::SNAPSHOT_WORKSPACE;

pub(super) fn active_runs_view() -> RunHavenMvpView {
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

pub(super) fn run_control_stop_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::RunControl(Box::new(RunControlScreen {
        run: active_run(),
        action: RunControlAction::Stop,
        state: RunControlState::Confirm {
            typed: String::new(),
            notice: None,
        },
    }));
    view
}

pub(super) fn run_control_result_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::RunControl(Box::new(RunControlScreen {
        run: active_run(),
        action: RunControlAction::Repair,
        state: RunControlState::Complete(RunControlResultData {
            action: "repair".to_string(),
            run_id: "run-20260629-001".to_string(),
            container_name: "runhaven-codex-project-run".to_string(),
            return_code: Some(1),
            status: "removed".to_string(),
            marker_removed: Some(true),
            message:
                "Removed stale active marker for run run-20260629-001 (runhaven-codex-project-run)."
                    .to_string(),
        }),
    }));
    view
}

pub(super) fn log_confirmation_view() -> RunHavenMvpView {
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

pub(super) fn loaded_log_snapshot_view() -> RunHavenMvpView {
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

pub(super) fn diagnostics_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::Diagnostics(Box::new(DiagnosticsScreen {
        result: Ok(RunHavenDiagnosticsData {
            doctor_checks: vec![
                DoctorCheckData {
                    name: "macOS".to_string(),
                    ok: true,
                    detail: "26.0".to_string(),
                    remedy: "Use a macOS 26+ host.".to_string(),
                },
                DoctorCheckData {
                    name: "Apple container CLI".to_string(),
                    ok: false,
                    detail: "not found on PATH".to_string(),
                    remedy: "Install Apple container 1.0.0 and run `container system start`."
                        .to_string(),
                },
            ],
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

pub(super) fn history_view() -> RunHavenMvpView {
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
        notice: None,
    }));
    view
}

pub(super) fn run_diff_confirmation_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::RunDiff(Box::new(RunDiffScreen {
        run: history_run(),
        state: RunDiffState::Confirm {
            typed: String::new(),
            notice: None,
        },
    }));
    view
}

pub(super) fn loaded_run_diff_view() -> RunHavenMvpView {
    let mut view = RunHavenMvpView::new(SNAPSHOT_WORKSPACE.into());
    view.screen = MvpScreen::RunDiff(Box::new(RunDiffScreen {
        run: history_run(),
        state: RunDiffState::Loaded(RunDiffData {
            run_id: "run-20260629-002".to_string(),
            text: [
                "diff --git a/src/main.rs b/src/main.rs",
                "index 1111111..2222222 100644",
                "--- a/src/main.rs",
                "+++ b/src/main.rs",
                "@@ -1,3 +1,4 @@",
                " fn main() {",
                "+    println!(\"RunHaven reviewed this change\");",
                " }",
            ]
            .join("\n"),
            returned_lines: 8,
            truncated: false,
            source: "git diff".to_string(),
            warnings: vec!["Diff can include workspace file contents.".to_string()],
        }),
    }));
    view
}

fn history_run() -> RunHistorySummaryData {
    RunHistorySummaryData {
        run_id: "run-20260629-002".to_string(),
        profile: "codex".to_string(),
        network: "provider".to_string(),
        status: "succeeded".to_string(),
        started_at: "2026-06-29T17:50:00Z".to_string(),
        finished_at: "2026-06-29T17:59:00Z".to_string(),
        return_code: Some(0),
        workspace_scope: "repository".to_string(),
        session: "agent".to_string(),
        state_volume: "runhaven-codex-shared-home".to_string(),
        provider_allowed: 8,
        provider_denied: 0,
        auth_allowed: 2,
        auth_denied: 0,
        cleanup_provider_network: "removed".to_string(),
        git_summary: "Git: changed=true before=1234567 after=89abcde files=3".to_string(),
        worktree_branch: Some("runhaven/codex/run-20260629-002-review-branch".to_string()),
        review_command: "runhaven runs show run-20260629-002".to_string(),
    }
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
