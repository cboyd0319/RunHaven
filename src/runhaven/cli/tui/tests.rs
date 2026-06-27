use super::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use tempfile::tempdir;

use crate::plans::{NetworkMode, WorkspaceScope};

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
    let blocks = text.matches('\u{2580}').count() + text.matches('\u{2584}').count();
    assert!(blocks > 40, "expected mascot half-blocks, got {blocks}");
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
