use super::*;
use crossterm::event::KeyEvent;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use runhaven_core::runtime::active::write_active_run_payload;
use runhaven_core::support::paths::{
    auth_broker_log_path, egress_policy_log_path, ensure_private_parent,
    override_cache_root_for_tests,
};
use std::io::Write;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn render_to_text(view: &mut RunHavenMvpView, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| {
            let area = frame.area();
            view.render(area, frame.buffer_mut());
        })
        .expect("draw");
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("")
}

fn line_text(line: Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<Vec<_>>()
        .join("")
}

#[test]
fn policy_keys_rebuild_launch_plan_without_custom_shell_code() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());

    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Pick the agent to run"));
    assert!(output.contains("review the plan before anything starts"));
    assert!(line_text(view.footer_status()).contains("auth agent"));

    view.handle_key_event(key(KeyCode::Char('a')));
    render_to_text(&mut view, 120, 32);
    assert!(line_text(view.footer_status()).contains("auth project"));

    view.handle_key_event(key(KeyCode::Char('n')));
    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Local only"));
    assert!(line_text(view.footer_status()).contains("network internal"));
}

#[test]
fn active_runs_view_omits_workspace_paths_and_opens_log_confirmation() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    write_active_run_payload(
        "run-123",
        serde_json::json!({
            "timestamp": "2026-06-29T00:00:00Z",
            "run_id": "run-123",
            "profile": "codex",
            "workspace": "/Users/c/secret/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-project-run",
            "state_volume": "runhaven-codex-shared-home",
            "session": "none"
        }),
    )
    .expect("active marker");
    let workspace = tempfile::tempdir().expect("workspace");
    let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());

    view.handle_key_event(key(KeyCode::Char('2')));
    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Active runs"));
    assert!(output.contains("run-123"));
    assert!(!output.contains("/Users/c/secret/project"));

    view.handle_key_event(key(KeyCode::Enter));
    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Raw container output can contain secrets"));
    assert!(output.contains("Type logs"));
}

#[test]
fn log_confirmation_rejects_paste_and_wrong_phrase() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());
    view.screen = MvpScreen::RunLogs(Box::new(RunLogsScreen {
        run: ActiveRunSummaryData {
            run_id: "run-123".to_string(),
            profile: "codex".to_string(),
            network: "provider".to_string(),
            status: "running".to_string(),
            timestamp: "2026-06-29T00:00:00Z".to_string(),
            state_volume: "runhaven-codex-shared-home".to_string(),
            session: "none".to_string(),
            container_name: "runhaven-codex-project-run".to_string(),
        },
        state: RunLogsState::Confirm {
            typed: String::new(),
            notice: None,
        },
    }));

    assert!(view.handle_paste("logs".to_string()));
    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Paste is ignored"));

    for ch in "nope".chars() {
        view.handle_key_event(key(KeyCode::Char(ch)));
    }
    view.handle_key_event(key(KeyCode::Enter));
    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Type logs before loading logs."));
}

#[test]
fn loaded_log_snapshot_is_visible_only_after_confirm_state() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());
    view.screen = MvpScreen::RunLogs(Box::new(RunLogsScreen {
        run: ActiveRunSummaryData {
            run_id: "run-123".to_string(),
            profile: "codex".to_string(),
            network: "provider".to_string(),
            status: "running".to_string(),
            timestamp: "2026-06-29T00:00:00Z".to_string(),
            state_volume: "runhaven-codex-shared-home".to_string(),
            session: "none".to_string(),
            container_name: "runhaven-codex-project-run".to_string(),
        },
        state: RunLogsState::Loaded(ActiveRunLogSnapshotData {
            run_id: "run-123".to_string(),
            captured_at: "2026-06-29T00:00:00Z".to_string(),
            requested_lines: 20,
            text: "agent output\n".to_string(),
            returned_lines: 1,
            truncated: false,
            source: "container-stdio".to_string(),
            warnings: vec![
                "Raw container output can contain secrets or workspace content.".to_string(),
            ],
        }),
    }));

    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Snapshot"));
    assert!(output.contains("agent output"));
    assert!(output.contains("Raw container output can contain secrets or workspace content."));
}

#[test]
fn diagnostics_view_omits_secret_and_workspace_fields() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    ensure_private_parent(&egress_policy_log_path()).expect("egress parent");
    ensure_private_parent(&auth_broker_log_path()).expect("auth parent");
    {
        let mut file = std::fs::File::create(egress_policy_log_path()).expect("egress log file");
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "profile": "codex",
                "decision": "denied",
                "host": "example.com",
                "port": 443,
                "count": 1,
                "reason": "not-in-allowlist",
                "matched_rule": "",
                "run_id": "run-123",
                "workspace": "/Users/c/secret/project"
            })
        )
        .expect("egress write");
    }
    {
        let mut file = std::fs::File::create(auth_broker_log_path()).expect("auth log file");
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "profile": "codex",
                "broker": "api-key",
                "decision": "allowed",
                "method": "POST",
                "path": "/v1/responses?token=secret#fragment",
                "upstream_status": 200,
                "count": 1,
                "reason": "-",
                "run_id": "run-123",
                "authorization": "Bearer secret"
            })
        )
        .expect("auth write");
    }
    let workspace = tempfile::tempdir().expect("workspace");
    let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());

    view.handle_key_event(key(KeyCode::Char('3')));
    let output = render_to_text(&mut view, 120, 40);

    assert!(output.contains("Diagnostics"));
    assert!(output.contains("example.com"));
    assert!(output.contains("/v1/responses"));
    assert!(!output.contains("token=secret"));
    assert!(!output.contains("/Users/c/secret/project"));
    assert!(!output.contains("Bearer secret"));
}

#[test]
fn post_run_recovery_screen_can_return_to_launch() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());
    let launch = view
        .service
        .launch_preview_payload(workspace.path())
        .previews
        .into_iter()
        .find(|preview| preview.agent.name == "codex")
        .expect("codex preview")
        .plan
        .expect("codex plan");

    view.show_post_run(PostRunOutcome::from_launch(&launch, 0, None));
    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Run finished"));
    assert!(output.contains("The terminal is back in RunHaven."));

    view.handle_key_event(key(KeyCode::Enter));
    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Choose agent"));
}

#[test]
fn post_run_recovery_preserves_effective_workspace_and_policy() {
    let root = tempfile::tempdir().expect("root");
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested).expect("nested workspace");
    let mut view = RunHavenMvpView::new(nested.clone());
    let policy = LaunchPolicySelection {
        network: NetworkPolicySelection::Fixed(NetworkMode::Internet),
        auth_scope: AuthScope::Project,
    };
    let launch = view
        .service
        .launch_preview_payload_with_policy(root.path(), policy)
        .previews
        .into_iter()
        .find(|preview| preview.agent.name == "codex")
        .expect("codex preview")
        .plan
        .expect("codex plan");

    view.show_post_run(PostRunOutcome::from_launch(&launch, 0, None));
    assert_eq!(view.workspace, launch.executable.workspace);
    assert_eq!(view.policy, policy);

    view.handle_key_event(key(KeyCode::Enter));
    assert_eq!(view.workspace, launch.executable.workspace);
    assert_eq!(view.policy, policy);
}
