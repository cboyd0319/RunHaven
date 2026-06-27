use super::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;
use tempfile::tempdir;

use crate::runhaven::runtime::plans::{NetworkMode, WorkspaceScope};

fn test_app() -> App {
    App::with_settings_and_workspace(TuiSettings::default(), PathBuf::from("/workspace"))
}

fn test_app_with_workspace(workspace: PathBuf) -> App {
    App::with_settings_and_workspace(TuiSettings::default(), workspace)
}

fn fake_review(requires_typed_confirm: bool) -> launcher::PlanReview {
    launcher::PlanReview {
        plan: AgentRunPlan {
            command: vec![
                "container".to_string(),
                "run".to_string(),
                "runhaven/codex:0.1.0".to_string(),
                "codex".to_string(),
            ],
            preflight: Vec::new(),
            workspace: PathBuf::from("/workspace"),
            state_volume: "runhaven-codex-shared-home".to_string(),
            session: "default".to_string(),
            container_name: "runhaven-codex-demo-run".to_string(),
            profile_name: "codex".to_string(),
            workspace_scope: WorkspaceScope::Current,
            workspace_scope_note: None,
            worktree: None,
            run_id: None,
            network_name: Some("runhaven-codex-provider".to_string()),
            network_mode: NetworkMode::Provider,
            egress_summary: "provider allowlist egress through runtime proxy: api.openai.com"
                .to_string(),
            image: "runhaven/codex:0.1.0".to_string(),
            provider_allowed_hosts: vec!["api.openai.com".to_string()],
            api_key_broker_env: None,
            security_notices: if requires_typed_confirm {
                vec!["Unrestricted internet egress is enabled.".to_string()]
            } else {
                Vec::new()
            },
        },
        cli_command: "runhaven run codex --workspace /workspace --network provider".to_string(),
        requires_typed_confirm,
    }
}

fn fake_run() -> runs::RunSummary {
    runs::RunSummary {
        run_id: "abcdef1234567890".to_string(),
        profile: "codex".to_string(),
        workspace: "/Users/example/project".to_string(),
        network: "provider".to_string(),
        marker_status: "running".to_string(),
        container_name: "runhaven-codex-abcdef-run".to_string(),
        state_volume: "runhaven-codex-shared-home".to_string(),
        timestamp: "2026-06-27T00:00:00Z".to_string(),
    }
}

fn fake_status() -> runs::RunStatus {
    runs::RunStatus {
        marker_status: "running".to_string(),
        container_state: "running".to_string(),
        started_at: "2026-06-27T00:00:01Z".to_string(),
        image: "runhaven/codex:0.1.0".to_string(),
        resources: "4 cpu / 4 GiB".to_string(),
        networks: vec!["runhaven-codex-provider ipv4=192.0.2.2/24 hostname=runhaven".to_string()],
    }
}

fn fake_egress() -> Vec<runs::EgressDecision> {
    vec![
        runs::EgressDecision {
            timestamp: "2026-06-27T00:00:02Z".to_string(),
            decision: "allowed".to_string(),
            host: "api.openai.com".to_string(),
            port: "443".to_string(),
            reason: "allowed".to_string(),
            matched_rule: "api.openai.com".to_string(),
            count: "3".to_string(),
        },
        runs::EgressDecision {
            timestamp: "2026-06-27T00:00:03Z".to_string(),
            decision: "denied".to_string(),
            host: "example.invalid".to_string(),
            port: "443".to_string(),
            reason: "not-in-allowlist".to_string(),
            matched_rule: "-".to_string(),
            count: "1".to_string(),
        },
    ]
}

fn fake_history_record() -> history::RunHistorySummary {
    history::RunHistorySummary {
        run_id: "run-20260627".to_string(),
        profile: "codex".to_string(),
        workspace: "/Users/example/project".to_string(),
        network: "provider".to_string(),
        status: "succeeded".to_string(),
        return_code: "0".to_string(),
        timestamp: "2026-06-27T00:00:00Z".to_string(),
        provider_denied: 1,
        auth_denied: 0,
        cleanup: "removed".to_string(),
        git_summary: "Git: changed=true before=1111111 after=2222222 files=2".to_string(),
    }
}

fn seed_history(app: &mut App) {
    app.history.records = vec![fake_history_record()];
    app.history.diagnostics.auth_status = history::AuthStatusSummary {
        status: "available".to_string(),
        runtime: "host broker".to_string(),
        credential_stores_inspected: false,
        environment_values_inspected: false,
        secrets_printed: false,
        profiles: vec![
            history::AuthProfileSummary {
                agent: "codex".to_string(),
                broker: "yes".to_string(),
                status: "api-key-broker".to_string(),
            },
            history::AuthProfileSummary {
                agent: "copilot".to_string(),
                broker: "no".to_string(),
                status: "isolated-login".to_string(),
            },
        ],
    };
    app.history.diagnostics.terminal = history::TerminalProbe {
        color: "enabled".to_string(),
        motion: "animated".to_string(),
        line_mode: "disabled".to_string(),
        pet_image: "kitty graphics".to_string(),
    };
    app.history.diagnostics.egress = vec![history::DiagnosticEgressEntry {
        timestamp: "2026-06-27T00:00:01Z".to_string(),
        profile: "codex".to_string(),
        decision: "denied".to_string(),
        host: "example.invalid".to_string(),
        port: "443".to_string(),
        count: "1".to_string(),
        reason: "not-in-allowlist".to_string(),
        run_id: "run-20260627".to_string(),
    }];
    app.history.diagnostics.auth = vec![history::DiagnosticAuthEntry {
        timestamp: "2026-06-27T00:00:02Z".to_string(),
        profile: "codex".to_string(),
        broker: "codex-api-key".to_string(),
        decision: "allowed".to_string(),
        method: "POST".to_string(),
        path: "/v1/responses".to_string(),
        upstream_status: "200".to_string(),
        count: "2".to_string(),
        reason: "profile-match".to_string(),
        run_id: "run-20260627".to_string(),
    }];
    app.history.doctor.checks = vec![
        history::DoctorCheckSummary {
            name: "macOS".to_string(),
            ok: true,
            detail: "26.0".to_string(),
            remedy: "Use a macOS 26+ host.".to_string(),
        },
        history::DoctorCheckSummary {
            name: "Apple container CLI".to_string(),
            ok: false,
            detail: "not found on PATH".to_string(),
            remedy: "Install Apple container 1.0.0 and run `container system start`.".to_string(),
        },
    ];
}

fn seed_dashboard(app: &mut App) {
    app.run_manager.runs = vec![fake_run()];
    app.run_manager.status = Some(fake_status());
    app.run_manager.egress = fake_egress();
}

#[test]
fn app_loads_all_agent_profiles() {
    let app = test_app();
    assert_eq!(app.agents.len(), 6);
    assert_eq!(app.list.selected(), Some(0));
}

#[test]
fn home_banner_shows_mascot_and_brand() {
    let mut terminal = Terminal::new(TestBackend::new(60, 30)).unwrap();
    let mut app = test_app();
    terminal.draw(|f| app.render(f)).unwrap();
    let buf = terminal.backend().buffer();
    let mut text = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            text.push_str(buf[(x, y)].symbol());
        }
    }
    assert!(text.contains("RunHaven"), "brand text missing");
    assert!(text.contains("launch: [1 agent]"), "launch stepper missing");
    assert!(text.contains("agent: antigravity"), "agent context missing");
    assert!(
        text.contains("workspace: /workspace"),
        "workspace context missing"
    );
    assert!(
        text.contains("boundary: /workspace only"),
        "boundary context missing"
    );
    assert!(text.contains("next: r review plan"), "next action missing");
    let blocks = text.matches('\u{2580}').count() + text.matches('\u{2584}').count();
    assert!(blocks > 40, "expected mascot half-blocks, got {blocks}");
}

#[test]
fn home_banner_context_prefers_active_run_next_action() {
    let mut app = test_app();
    app.run_manager.runs = vec![fake_run()];

    let context = app.home_banner_context();

    assert!(context.iter().any(|line| line.starts_with("RunHaven v")));
    assert!(
        context
            .iter()
            .any(|line| line.contains("d dashboard (1 active)"))
    );
}

#[test]
fn navigation_clamps_within_bounds() {
    let mut app = test_app();
    let last = app.agents.len() - 1;
    app.handle_key(KeyCode::Up);
    assert_eq!(app.list.selected(), Some(0));
    app.handle_key(KeyCode::Down);
    assert_eq!(app.list.selected(), Some(1));
    for _ in 0..app.agents.len() + 3 {
        app.handle_key(KeyCode::Down);
    }
    assert_eq!(app.list.selected(), Some(last));
}

#[test]
fn enter_opens_detail_and_esc_returns_home() {
    let mut app = test_app();
    assert!(matches!(app.screen, Screen::Home));
    app.handle_key(KeyCode::Enter);
    assert!(matches!(app.screen, Screen::Detail));
    app.handle_key(KeyCode::Esc);
    assert!(matches!(app.screen, Screen::Home));
}

#[test]
fn q_quits_from_either_screen() {
    let mut app = test_app();
    assert!(matches!(
        app.handle_key(KeyCode::Char('q')),
        Some(TuiAction::Exit(0))
    ));
    app.handle_key(KeyCode::Enter);
    assert!(matches!(
        app.handle_key(KeyCode::Char('q')),
        Some(TuiAction::Exit(0))
    ));
}

#[test]
fn workspace_picker_selects_a_child_workspace() {
    let root = tempdir().unwrap();
    let child = root.path().join("sample-app");
    std::fs::create_dir(&child).unwrap();
    let mut app = test_app_with_workspace(root.path().to_path_buf());

    app.handle_key(KeyCode::Char('w'));
    for ch in "sample".chars() {
        app.handle_key(KeyCode::Char(ch));
    }
    app.handle_key(KeyCode::Enter);

    assert!(matches!(app.screen, Screen::Home));
    assert_eq!(app.launcher.workspace, child.canonicalize().unwrap());
}

#[test]
fn review_plan_uses_selected_workspace_and_agent() {
    let workspace = tempdir().unwrap();
    let mut app = test_app_with_workspace(workspace.path().to_path_buf());

    app.handle_key(KeyCode::Char('r'));

    assert!(matches!(app.screen, Screen::Plan));
    let review = app.launcher.review.as_ref().expect("review");
    assert_eq!(
        review.plan.workspace,
        workspace.path().canonicalize().unwrap()
    );
    assert_eq!(review.plan.profile_name, "antigravity");
    assert!(review.cli_command.contains("runhaven run antigravity"));
}

#[test]
fn secure_confirm_returns_launch_action() {
    let workspace = tempdir().unwrap();
    let mut app = test_app_with_workspace(workspace.path().to_path_buf());

    app.handle_key(KeyCode::Char('r'));
    app.handle_key(KeyCode::Enter);
    let action = app.handle_key(KeyCode::Enter);

    assert!(matches!(app.screen, Screen::Confirm));
    assert!(matches!(action, Some(TuiAction::Launch(_))));
}

#[test]
fn dashboard_opens_from_home_and_returns() {
    let mut app = test_app();

    app.handle_key(KeyCode::Char('d'));
    assert!(matches!(app.screen, Screen::Runs));
    app.handle_key(KeyCode::Esc);
    assert!(matches!(app.screen, Screen::Home));
}

#[test]
fn history_and_diagnostics_screens_are_reachable() {
    let mut app = test_app();

    app.handle_key(KeyCode::Char('?'));
    assert!(matches!(app.screen, Screen::Guide));
    app.handle_key(KeyCode::Esc);
    assert!(matches!(app.screen, Screen::Home));

    app.handle_key(KeyCode::Char('h'));
    assert!(matches!(app.screen, Screen::History));
    app.handle_key(KeyCode::Char('g'));
    assert!(matches!(app.screen, Screen::Diagnostics));
    app.handle_key(KeyCode::Char('d'));
    assert!(matches!(app.screen, Screen::Doctor));
    app.handle_key(KeyCode::Char('g'));
    assert!(matches!(app.screen, Screen::Diagnostics));
    app.handle_key(KeyCode::Esc);
    assert!(matches!(app.screen, Screen::Home));
}

#[test]
fn guide_routes_to_primary_workflows() {
    let mut app = test_app();
    app.screen = Screen::Guide;

    app.handle_key(KeyCode::Char('g'));
    assert!(matches!(app.screen, Screen::Diagnostics));
    app.screen = Screen::Guide;
    app.handle_key(KeyCode::Char('h'));
    assert!(matches!(app.screen, Screen::History));
    app.screen = Screen::Guide;
    app.handle_key(KeyCode::Char('d'));
    assert!(matches!(app.screen, Screen::Runs));
}

#[test]
fn guide_can_toggle_cubby_display() {
    let mut app = test_app();
    app.screen = Screen::Guide;

    assert!(app.settings.pet_enabled);
    app.handle_key(KeyCode::Char('p'));
    assert!(!app.settings.pet_enabled);
    app.handle_key(KeyCode::Char('p'));
    assert!(app.settings.pet_enabled);
}

#[test]
fn first_run_guide_depends_on_run_history_log() {
    let dir = tempdir().expect("tempdir");
    let runs_log = dir.path().join("runs.jsonl");

    assert!(App::should_start_with_guide(&runs_log));

    std::fs::write(&runs_log, "").expect("write empty run log");
    assert!(App::should_start_with_guide(&runs_log));

    std::fs::write(&runs_log, "{\"run_id\":\"rh-test\"}\n").expect("write run log");
    assert!(!App::should_start_with_guide(&runs_log));
}

#[test]
fn zork_easter_egg_is_hidden_home_only_screen() {
    let mut app = test_app();

    app.handle_key(KeyCode::Char('~'));
    assert!(matches!(app.screen, Screen::Zork));
    assert!(app.zork.transcript().contains("ZORK I"));
    assert!(app.zork.transcript().contains("West of House"));
    app.handle_key(KeyCode::Char('q'));
    assert!(matches!(app.screen, Screen::Zork));
    app.handle_key(KeyCode::Esc);
    assert!(matches!(app.screen, Screen::Home));

    app.screen = Screen::Guide;
    app.handle_key(KeyCode::Char('~'));
    assert!(matches!(app.screen, Screen::Guide));
}

#[test]
fn history_detail_scrolls_loaded_diff() {
    let mut app = test_app();
    seed_history(&mut app);
    app.history.detail.set_diff("one\ntwo\nthree\n".to_string());
    app.screen = Screen::HistoryDetail;

    app.handle_key(KeyCode::Down);
    assert_eq!(app.history.detail.scroll, 1);
    app.handle_key(KeyCode::Up);
    assert_eq!(app.history.detail.scroll, 0);
    app.handle_key(KeyCode::Esc);
    assert!(matches!(app.screen, Screen::History));
}

#[test]
fn run_control_confirm_requires_typed_phrase() {
    let mut app = test_app();
    seed_dashboard(&mut app);
    app.screen = Screen::Runs;

    app.handle_key(KeyCode::Char('s'));
    assert!(matches!(app.screen, Screen::Control));
    for ch in "sto".chars() {
        app.handle_key(KeyCode::Char(ch));
    }
    assert!(!app.run_manager.control.as_ref().unwrap().ready());
    app.handle_key(KeyCode::Char('p'));

    assert!(app.run_manager.control.as_ref().unwrap().ready());
}

#[test]
fn log_search_mode_accepts_command_letters() {
    let mut app = test_app();
    seed_dashboard(&mut app);
    app.screen = Screen::Logs;

    app.handle_key(KeyCode::Char('/'));
    for ch in "query".chars() {
        assert!(app.handle_key(KeyCode::Char(ch)).is_none());
    }

    assert_eq!(app.run_manager.logs.search, "query");
    assert!(matches!(app.screen, Screen::Logs));
    app.handle_key(KeyCode::Enter);
    assert!(!app.run_manager.logs.search_editing);
    assert!(matches!(
        app.handle_key(KeyCode::Char('q')),
        Some(TuiAction::Exit(0))
    ));
}

#[test]
fn lower_security_confirm_requires_typed_phrase() {
    let workspace = tempdir().unwrap();
    let mut app = test_app_with_workspace(workspace.path().to_path_buf());
    for _ in 0..app.agents.len() {
        app.handle_key(KeyCode::Down);
    }

    app.handle_key(KeyCode::Char('r'));
    app.handle_key(KeyCode::Enter);
    assert!(app.handle_key(KeyCode::Enter).is_none());
    for ch in launcher::CONFIRM_PHRASE.chars() {
        app.handle_key(KeyCode::Char(ch));
    }

    assert!(matches!(
        app.handle_key(KeyCode::Enter),
        Some(TuiAction::Launch(_))
    ));
}

#[test]
fn both_screens_render_without_panicking() {
    let mut terminal = Terminal::new(TestBackend::new(80, 20)).expect("terminal");
    let mut app = test_app();
    terminal.draw(|frame| app.render(frame)).expect("home");
    app.handle_key(KeyCode::Enter);
    terminal.draw(|frame| app.render(frame)).expect("detail");
}

#[test]
fn no_color_rendering_leaves_color_cells_reset() {
    let settings = TuiSettings {
        color_enabled: false,
        ..TuiSettings::default()
    };
    let mut app = App::with_settings_and_workspace(settings, PathBuf::from("/workspace"));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("terminal");

    terminal.draw(|frame| app.render(frame)).expect("home");
    let buf = terminal.backend().buffer();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            assert_eq!(cell.fg, Color::Reset);
            assert_eq!(cell.bg, Color::Reset);
        }
    }
}

#[test]
fn line_mode_uses_text_banner_without_mascot_blocks() {
    let settings = TuiSettings {
        line_mode: true,
        ..TuiSettings::default()
    };
    let mut app = App::with_settings_and_workspace(settings, PathBuf::from("/workspace"));
    let snapshot = snapshot::render_vt100(60, 20, |frame| app.render(frame)).unwrap();

    assert!(snapshot.contains("RunHaven"));
    assert!(!snapshot.contains('\u{2580}'));
    assert!(!snapshot.contains('\u{2584}'));
}

#[test]
fn phase_five_screens_render_in_line_mode() {
    let settings = TuiSettings {
        line_mode: true,
        ..TuiSettings::default()
    };
    let mut app = App::with_settings_and_workspace(settings, PathBuf::from("/workspace"));
    seed_history(&mut app);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("terminal");

    for screen in [
        Screen::Guide,
        Screen::History,
        Screen::HistoryDetail,
        Screen::Diagnostics,
        Screen::Doctor,
    ] {
        app.screen = screen;
        terminal.draw(|frame| app.render(frame)).expect("render");
    }
}

#[test]
fn tick_updates_app_clock_state() {
    let mut app = test_app();
    app.on_tick(Tick {
        elapsed: Duration::from_millis(250),
    });

    assert_eq!(app.ticks, 1);
    assert_eq!(app.last_tick_elapsed, Duration::from_millis(250));
}

#[test]
fn agent_list_items_truncate_with_ascii_affordance() {
    assert_eq!(widgets::truncate_to_width("abcdef", 6), "abcdef");
    assert_eq!(widgets::truncate_to_width("abcdef", 5), "ab...");
    assert_eq!(widgets::truncate_to_width("abcdef", 2), "..");
    assert_eq!(widgets::truncate_to_width("abcdef", 0), "");
}

#[test]
fn home_snapshot_80x24() {
    let mut app = test_app();
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_home_80x24", snapshot);
}

#[test]
fn home_snapshot_120x36() {
    let mut app = test_app();
    let snapshot = snapshot::render_vt100(120, 36, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_home_120x36", snapshot);
}

#[test]
fn detail_snapshot_80x24() {
    let mut app = test_app();
    app.handle_key(KeyCode::Enter);
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_detail_80x24", snapshot);
}

#[test]
fn workspace_snapshot_80x24() {
    let mut app = test_app();
    app.handle_key(KeyCode::Char('w'));
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_workspace_80x24", snapshot);
}

#[test]
fn plan_snapshot_80x24() {
    let mut app = test_app();
    app.launcher.review = Some(fake_review(false));
    app.screen = Screen::Plan;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_plan_80x24", snapshot);
}

#[test]
fn confirm_snapshot_80x24() {
    let mut app = test_app();
    app.launcher.review = Some(fake_review(true));
    app.launcher.confirm_input = "ru".to_string();
    app.screen = Screen::Confirm;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_confirm_80x24", snapshot);
}

#[test]
fn dashboard_snapshot_80x24() {
    let mut app = test_app();
    seed_dashboard(&mut app);
    app.screen = Screen::Runs;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_dashboard_80x24", snapshot);
}

#[test]
fn logs_snapshot_80x24() {
    let mut app = test_app();
    seed_dashboard(&mut app);
    app.run_manager.logs.set_snapshot(runs::LogSnapshot {
        text: "\u{1b}[32mallowed api.openai.com\u{1b}[0m\ndenied example.invalid\n".to_string(),
        returned_lines: 2,
        requested_lines: 200,
        truncated: false,
        warnings: vec![
            "Raw container output can contain secrets or workspace content.".to_string(),
        ],
    });
    app.run_manager.logs.search = "denied".to_string();
    app.screen = Screen::Logs;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_logs_80x24", snapshot);
}

#[test]
fn control_snapshot_80x24() {
    let mut app = test_app();
    seed_dashboard(&mut app);
    app.run_manager
        .begin_control(runs::RunControlAction::Kill)
        .unwrap();
    app.run_manager.control.as_mut().unwrap().input = "ki".to_string();
    app.screen = Screen::Control;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_control_80x24", snapshot);
}

#[test]
fn guide_snapshot_80x24() {
    let mut app = test_app();
    app.screen = Screen::Guide;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_guide_80x24", snapshot);
}

#[test]
fn history_snapshot_80x24() {
    let mut app = test_app();
    seed_history(&mut app);
    app.screen = Screen::History;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_history_80x24", snapshot);
}

#[test]
fn history_detail_snapshot_80x24() {
    let mut app = test_app();
    seed_history(&mut app);
    app.history.detail.set_diff(
        "diff --git a/file.txt b/file.txt\n--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n"
            .to_string(),
    );
    app.screen = Screen::HistoryDetail;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_history_detail_80x24", snapshot);
}

#[test]
fn diagnostics_snapshot_80x24() {
    let mut app = test_app();
    seed_history(&mut app);
    app.screen = Screen::Diagnostics;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_diagnostics_80x24", snapshot);
}

#[test]
fn doctor_snapshot_80x24() {
    let mut app = test_app();
    seed_history(&mut app);
    app.screen = Screen::Doctor;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_doctor_80x24", snapshot);
}

#[test]
fn zork_snapshot_80x24() {
    let mut app = test_app();
    app.screen = Screen::Zork;
    let snapshot = snapshot::render_vt100(80, 24, |frame| app.render(frame)).unwrap();
    insta::assert_snapshot!("tui_zork_80x24", snapshot);
}
