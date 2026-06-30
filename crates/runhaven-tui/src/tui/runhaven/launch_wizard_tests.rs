use super::*;
use crate::tui::app_event::AppEvent;
use crate::tui::bottom_pane::AppEventSender;
use crate::tui::bottom_pane::BottomPane;
use crate::tui::bottom_pane::BottomPaneParams;
use crate::tui::runhaven::service::LaunchPreviewError;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use runhaven_core::runtime::plans::AgentRunPlan;
use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::NetworkMode;
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::ui_contracts::LaunchBoundaryData;
use runhaven_core::ui_contracts::LaunchNetworkData;
use tokio::sync::mpsc;

fn ready_preview(name: &str) -> AgentLaunchPreview {
    AgentLaunchPreview {
        agent: agent(name),
        plan: Ok(prepared_launch(name)),
    }
}

fn confirm_required_preview(name: &str) -> AgentLaunchPreview {
    let mut plan = plan(name);
    plan.confirm_required = true;
    plan.safety_notes
        .push("This plan uses a less safe launch option.".to_string());
    AgentLaunchPreview {
        agent: agent(name),
        plan: Ok(PreparedLaunch::from_parts_for_tests(
            plan,
            executable_plan(name),
        )),
    }
}

fn internet_preview(name: &str) -> AgentLaunchPreview {
    let mut data = plan(name);
    data.network.mode = "internet".to_string();
    data.network.summary = "internet unrestricted".to_string();
    data.network.provider_allowed_hosts.clear();
    data.confirm_required = true;
    let mut executable = executable_plan(name);
    executable.network_mode = NetworkMode::Internet;
    executable.egress_summary = "internet unrestricted".to_string();
    executable.provider_allowed_hosts.clear();
    AgentLaunchPreview {
        agent: agent(name),
        plan: Ok(PreparedLaunch::from_parts_for_tests(data, executable)),
    }
}

fn blocked_preview(name: &str) -> AgentLaunchPreview {
    AgentLaunchPreview {
        agent: agent(name),
        plan: Err(LaunchPreviewError::PlanBuildFailed {
            detail: "workspace is blocked".to_string(),
        }),
    }
}

fn agent(name: &str) -> AgentCatalogItemData {
    AgentCatalogItemData {
        name: name.to_string(),
        description: format!("{name} description"),
        image: format!("runhaven/{name}:0.1.0"),
        sign_in: "runhaven login".to_string(),
        broker: "no".to_string(),
        default_network: "provider".to_string(),
        provider_host_count: 1,
    }
}

fn prepared_launch(name: &str) -> PreparedLaunch {
    PreparedLaunch::from_parts_for_tests(plan(name), executable_plan(name))
}

fn executable_plan(name: &str) -> AgentRunPlan {
    AgentRunPlan {
        command: vec![
            "container".to_string(),
            "run".to_string(),
            "--name".to_string(),
            format!("runhaven-{name}"),
            format!("runhaven/{name}:0.1.0"),
        ],
        preflight: Vec::new(),
        workspace: PathBuf::from("/tmp/project"),
        state_volume: format!("runhaven-{name}-state"),
        session: "none".to_string(),
        container_name: format!("runhaven-{name}"),
        profile_name: name.to_string(),
        workspace_scope: WorkspaceScope::Current,
        workspace_scope_note: None,
        auth_scope: AuthScope::Agent,
        worktree: None,
        run_id: None,
        network_name: Some("runhaven-provider".to_string()),
        network_mode: NetworkMode::Provider,
        egress_summary: "provider allowlist".to_string(),
        image: format!("runhaven/{name}:0.1.0"),
        provider_allowed_hosts: vec!["example.com".to_string()],
        api_key_broker_env: None,
        security_notices: Vec::new(),
    }
}

fn plan(name: &str) -> LaunchPlanData {
    LaunchPlanData {
        profile_name: name.to_string(),
        workspace: "/tmp/project".to_string(),
        workspace_scope: "current".to_string(),
        workspace_scope_note: None,
        auth_scope: "agent".to_string(),
        session: "none".to_string(),
        state_volume: format!("runhaven-{name}-state"),
        container_name: format!("runhaven-{name}"),
        image: format!("runhaven/{name}:0.1.0"),
        worktree: None,
        network: LaunchNetworkData {
            mode: "provider".to_string(),
            name: Some("runhaven-provider".to_string()),
            summary: "provider allowlist".to_string(),
            provider_allowed_hosts: vec!["example.com".to_string()],
            api_key_broker_env: None,
        },
        boundary: LaunchBoundaryData {
            mounted_workspace: "/tmp/project -> /workspace".to_string(),
            mounted_state_volume: format!("runhaven-{name}-state -> /home/agent"),
            not_shared: vec![
                "host home folder".to_string(),
                "raw SSH keys".to_string(),
                "browser profiles".to_string(),
            ],
        },
        preflight_commands: Vec::new(),
        command: format!("container run --name runhaven-{name} runhaven/{name}:0.1.0"),
        safety_notes: Vec::new(),
        confirm_required: false,
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn modified_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

fn render_to_text(view: &impl Renderable, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal
        .draw(|frame| view.render(frame.area(), frame.buffer_mut()))
        .expect("draw");
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| {
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn bottom_pane_with_launch_wizard(previews: Vec<AgentLaunchPreview>) -> BottomPane {
    let (app_event_tx, _app_event_rx) = mpsc::unbounded_channel();
    let mut pane = BottomPane::new(BottomPaneParams {
        app_event_tx: AppEventSender::new(app_event_tx),
        frame_requester: crate::tui::FrameRequester::test_dummy(),
        has_input_focus: true,
        enhanced_keys_supported: false,
        placeholder_text: String::new(),
        disable_paste_burst: true,
        animations_enabled: true,
        skills: None,
    });
    pane.show_view(Box::new(LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        previews,
    )));
    pane
}

#[test]
fn enter_on_ready_plan_opens_review() {
    let mut view =
        LaunchWizardView::new(PathBuf::from("/tmp/project"), vec![ready_preview("codex")]);

    view.handle_key(key(KeyCode::Enter));

    assert!(view.is_reviewing());
    assert!(!view.is_cancelled());
    assert_eq!(view.selected_agent_name(), Some("codex"));
}

#[test]
fn workspace_picker_selects_git_root_before_agent_review() {
    let mut root_plan = plan("codex");
    root_plan.workspace = "/tmp/repo".to_string();
    root_plan.boundary.mounted_workspace = "/tmp/repo -> /workspace".to_string();
    let root_preview = AgentLaunchPreview {
        agent: agent("codex"),
        plan: Ok(PreparedLaunch::from_parts_for_tests(
            root_plan,
            executable_plan("codex"),
        )),
    };
    let mut view = LaunchWizardView::new_with_workspace_choices(vec![
        WorkspaceLaunchPreview {
            label: "Current directory".to_string(),
            description: "/tmp/repo/nested".to_string(),
            payload: LaunchPreviewPayload {
                workspace: PathBuf::from("/tmp/repo/nested"),
                previews: vec![ready_preview("codex")],
            },
        },
        WorkspaceLaunchPreview {
            label: "Git repository root".to_string(),
            description: "Mount the full repository instead of only the nested folder.".to_string(),
            payload: LaunchPreviewPayload {
                workspace: PathBuf::from("/tmp/repo"),
                previews: vec![root_preview],
            },
        },
    ]);

    assert!(view.is_choosing_workspace());
    assert!(render_to_text(&view, 100, 32).contains("Step 1/4: Choose workspace"));

    view.handle_key(key(KeyCode::Down));
    view.handle_key(key(KeyCode::Enter));

    assert!(!view.is_choosing_workspace());
    assert_eq!(view.selected_workspace_path(), Some(Path::new("/tmp/repo")));
    assert!(render_to_text(&view, 100, 32).contains("Step 2/4: Choose agent"));

    view.handle_key(key(KeyCode::Enter));

    assert!(view.is_reviewing());
    assert!(render_to_text(&view, 100, 32).contains("/tmp/repo -> /workspace"));
}

#[test]
fn launch_copy_matches_foreground_handoff_behavior() {
    let mut view =
        LaunchWizardView::new(PathBuf::from("/tmp/project"), vec![ready_preview("codex")]);

    let output = render_to_text(&view, 120, 32);
    assert!(output.contains("Enter opens the plan. Nothing starts until you confirm."));
    assert!(output.contains("Ready. Provider only. Workspace only."));
    assert!(!output.contains("Launch is still disabled"));

    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));
    let output = render_to_text(&view, 120, 32);
    assert!(output.contains("RunHaven starts it after restoring the terminal."));
    assert!(!output.contains("will not start the container"));
}

#[test]
fn first_step_internet_mode_stays_cautionary() {
    let view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![internet_preview("shell")],
    );

    let output = render_to_text(&view, 80, 24);
    let footer = format!("{:?}", view.footer_status_line());

    assert!(output.contains("Unrestricted internet"));
    assert!(output.contains("Needs confirmation. Review before launch."));
    assert!(footer.contains("Unrestricted internet"));
    assert!(!output.contains("internet unrestricted"));
}

#[test]
fn enter_on_blocked_plan_stays_in_picker() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![blocked_preview("blocked")],
    );

    view.handle_key(key(KeyCode::Enter));

    assert!(!view.is_reviewing());
    assert!(!view.is_cancelled());
    assert_eq!(view.selected_agent_name(), Some("blocked"));
}

#[test]
fn bottom_pane_view_selection_opens_review_without_completing() {
    let mut view =
        LaunchWizardView::new(PathBuf::from("/tmp/project"), vec![ready_preview("codex")]);

    BottomPaneView::handle_key_event(&mut view, key(KeyCode::Enter));

    assert!(view.is_reviewing());
    assert!(!BottomPaneView::is_complete(&view));
    assert_eq!(BottomPaneView::completion(&view), None);
}

#[test]
fn bottom_pane_view_picker_cancel_completes_view() {
    let mut view =
        LaunchWizardView::new(PathBuf::from("/tmp/project"), vec![ready_preview("codex")]);

    BottomPaneView::handle_key_event(&mut view, key(KeyCode::Esc));

    assert!(view.is_cancelled());
    assert!(BottomPaneView::is_complete(&view));
    assert_eq!(
        BottomPaneView::completion(&view),
        Some(ViewCompletion::Cancelled)
    );
}

#[test]
fn back_from_review_keeps_selected_agent() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![ready_preview("codex"), ready_preview("claude")],
    );
    view.handle_key(key(KeyCode::Down));
    view.handle_key(key(KeyCode::Enter));

    assert!(view.is_reviewing());
    assert_eq!(view.selected_agent_name(), Some("claude"));

    view.handle_key(key(KeyCode::Esc));

    assert!(!view.is_reviewing());
    assert_eq!(view.selected_agent_name(), Some("claude"));
}

#[test]
fn review_enter_opens_confirm_step_and_keeps_selected_agent() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![ready_preview("codex"), ready_preview("claude")],
    );
    view.handle_key(key(KeyCode::Down));
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));

    assert!(view.is_confirming());
    assert_eq!(view.selected_agent_name(), Some("claude"));
    assert!(view.terminal_title().contains("Confirm launch"));
}

#[test]
fn confirm_back_keys_return_to_review() {
    let mut view =
        LaunchWizardView::new(PathBuf::from("/tmp/project"), vec![ready_preview("codex")]);
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));

    assert!(view.is_confirming());
    view.handle_key(key(KeyCode::Backspace));

    assert!(view.is_reviewing());

    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Esc));

    assert!(view.is_reviewing());
}

#[test]
fn secure_plan_confirm_enter_prepares_foreground_launch_handoff() {
    let (app_event_tx, mut app_event_rx) = mpsc::unbounded_channel();
    let mut view =
        LaunchWizardView::new(PathBuf::from("/tmp/project"), vec![ready_preview("codex")]);
    view.set_app_event_sender(AppEventSender::new(app_event_tx));
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));

    assert!(view.is_confirming());
    assert_eq!(view.confirm_notice.as_deref(), Some(LAUNCH_PREPARED_NOTICE));
    match app_event_rx.try_recv().expect("launch prepared event") {
        AppEvent::RunHavenLaunchPrepared { launch } => {
            assert_eq!(launch.data.profile_name, "codex");
            assert_eq!(
                launch.data.command,
                "container run --name runhaven-codex runhaven/codex:0.1.0"
            );
            assert_eq!(launch.data.command, launch.executable.shell_command());
        }
        other => panic!("unexpected app event: {other:?}"),
    }
    view.handle_key(key(KeyCode::Enter));
    assert!(
        app_event_rx.try_recv().is_err(),
        "confirmed launch should be emitted once"
    );
}

#[test]
fn confirm_required_plan_rejects_missing_phrase() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![confirm_required_preview("codex")],
    );
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));

    assert!(view.is_confirming());
    assert_eq!(
        view.confirm_notice.as_deref(),
        Some("Type launch before confirming.")
    );
}

#[test]
fn confirm_required_plan_accepts_phrase_after_backspace_edit() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![confirm_required_preview("codex")],
    );
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));

    view.handle_key(modified_key(KeyCode::Char('L'), KeyModifiers::SHIFT));
    for ch in ['a', 'x'] {
        view.handle_key(key(KeyCode::Char(ch)));
    }
    view.handle_key(key(KeyCode::Backspace));
    for ch in ['u', 'n', 'c', 'h'] {
        view.handle_key(key(KeyCode::Char(ch)));
    }
    view.handle_key(modified_key(KeyCode::Char('!'), KeyModifiers::SHIFT));

    assert_eq!(view.confirm_text(), "Launch!");

    view.handle_key(key(KeyCode::Enter));
    assert_eq!(
        view.confirm_notice.as_deref(),
        Some("Type launch before confirming.")
    );

    view.handle_key(key(KeyCode::Backspace));
    assert_eq!(view.confirm_text(), "Launch");
    view.handle_key(key(KeyCode::Enter));

    assert_eq!(view.confirm_notice.as_deref(), Some(LAUNCH_PREPARED_NOTICE));
}

#[test]
fn confirm_required_plan_emits_prepared_launch_after_typed_phrase() {
    let (app_event_tx, mut app_event_rx) = mpsc::unbounded_channel();
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![confirm_required_preview("codex")],
    );
    view.set_app_event_sender(AppEventSender::new(app_event_tx));
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));

    for ch in "launch".chars() {
        view.handle_key(key(KeyCode::Char(ch)));
    }
    view.handle_key(key(KeyCode::Enter));

    assert_eq!(view.confirm_notice.as_deref(), Some(LAUNCH_PREPARED_NOTICE));
    match app_event_rx.try_recv().expect("launch prepared event") {
        AppEvent::RunHavenLaunchPrepared { launch } => {
            assert!(launch.data.confirm_required);
            assert_eq!(launch.data.command, launch.executable.shell_command());
        }
        other => panic!("unexpected app event: {other:?}"),
    }
    view.handle_key(key(KeyCode::Enter));
    assert!(
        app_event_rx.try_recv().is_err(),
        "typed confirmation should emit one launch intent"
    );
}

#[test]
fn confirm_required_composer_treats_q_as_text() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![confirm_required_preview("codex")],
    );
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));

    view.handle_key(key(KeyCode::Char('q')));

    assert!(view.is_confirming());
    assert_eq!(view.confirm_text(), "q");
}

#[test]
fn confirm_required_plan_rejects_pasted_phrase() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![confirm_required_preview("codex")],
    );
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));

    view.handle_paste("launch");
    view.handle_key(key(KeyCode::Enter));

    assert!(view.confirm_text().is_empty());
    assert_eq!(
        view.confirm_notice.as_deref(),
        Some("Type launch before confirming.")
    );

    for ch in ['l', 'a', 'u', 'n', 'c', 'h'] {
        view.handle_key(key(KeyCode::Char(ch)));
    }
    view.handle_key(key(KeyCode::Enter));

    assert_eq!(view.confirm_notice.as_deref(), Some(LAUNCH_PREPARED_NOTICE));
}

#[test]
fn bottom_pane_view_confirm_paste_is_handled_but_not_inserted() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![confirm_required_preview("codex")],
    );
    BottomPaneView::handle_key_event(&mut view, key(KeyCode::Enter));
    BottomPaneView::handle_key_event(&mut view, key(KeyCode::Enter));

    assert!(view.confirm_accepts_text_input());
    assert!(BottomPaneView::handle_paste(
        &mut view,
        "launch".to_string()
    ));
    assert!(view.confirm_text().is_empty());
    assert_eq!(
        view.confirm_notice.as_deref(),
        Some("Type launch by hand. Paste is ignored here.")
    );

    BottomPaneView::handle_key_event(&mut view, key(KeyCode::Enter));
    assert_eq!(
        view.confirm_notice.as_deref(),
        Some("Type launch before confirming.")
    );
}

#[test]
fn native_bottom_pane_hosts_launch_wizard_without_shell_shortcuts() {
    let mut pane = bottom_pane_with_launch_wizard(vec![confirm_required_preview("codex")]);

    pane.handle_key_event(key(KeyCode::Enter));
    assert!(render_to_text(&pane, 100, 32).contains("Step 3/4: Review plan"));

    pane.handle_key_event(key(KeyCode::Esc));
    assert!(render_to_text(&pane, 100, 32).contains("Step 1/4: Choose agent"));

    pane.handle_key_event(key(KeyCode::Enter));
    pane.handle_key_event(key(KeyCode::Enter));
    pane.handle_key_event(key(KeyCode::Char('q')));
    assert!(render_to_text(&pane, 100, 32).contains("q"));

    pane.handle_key_event(key(KeyCode::Esc));
    assert!(render_to_text(&pane, 100, 32).contains("Step 3/4: Review plan"));
    pane.handle_key_event(key(KeyCode::Esc));
    assert!(render_to_text(&pane, 100, 32).contains("Step 1/4: Choose agent"));
    pane.handle_key_event(key(KeyCode::Esc));
    assert!(!pane.has_active_view());
}

#[test]
fn confirm_render_keeps_command_and_safety_notes_visible() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![confirm_required_preview("codex")],
    );
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));
    let output = render_to_text(&view, 100, 32);

    assert!(output.contains("Step 4/4: Confirm launch"));
    assert!(output.contains("This plan needs typed confirmation."));
    assert!(output.contains("Type launch, then press Enter."));
    assert!(output.contains("Exact command"));
    assert!(output.contains("container run"));
    assert!(output.contains("Safety notes"));
    assert!(output.contains("less safe launch option"));
}

#[test]
fn confirm_step_footer_status_and_title_track_step_four() {
    let mut view =
        LaunchWizardView::new(PathBuf::from("/tmp/project"), vec![ready_preview("codex")]);
    view.handle_key(key(KeyCode::Enter));
    view.handle_key(key(KeyCode::Enter));

    let footer = format!("{:?}", view.footer_status_line());
    assert!(footer.contains("Confirm launch"));
    assert!(footer.contains("codex"));
    assert!(view.terminal_title().contains("Confirm launch"));
    assert!(view.terminal_title().contains("codex"));
}

#[test]
fn footer_status_and_title_track_selected_plan() {
    let mut view = LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![ready_preview("codex"), ready_preview("claude")],
    );

    let footer = format!("{:?}", view.footer_status_line());
    assert!(footer.contains("Choose agent"));
    assert!(footer.contains("codex"));
    assert!(footer.contains("Provider only"));
    assert!(view.terminal_title().contains("project"));
    assert!(view.terminal_title().contains("Choose agent"));
    assert!(view.terminal_title().contains("codex"));

    view.handle_key(key(KeyCode::Down));
    view.handle_key(key(KeyCode::Enter));

    let footer = format!("{:?}", view.footer_status_line());
    assert!(footer.contains("Review plan"));
    assert!(footer.contains("claude"));
    assert!(footer.contains("provider allowlist"));
    assert!(view.terminal_title().contains("Review plan"));
    assert!(view.terminal_title().contains("claude"));

    view.handle_key(key(KeyCode::Enter));

    let footer = format!("{:?}", view.footer_status_line());
    assert!(footer.contains("Confirm launch"));
    assert!(footer.contains("claude"));
    assert!(view.terminal_title().contains("Confirm launch"));
    assert!(view.terminal_title().contains("claude"));
}
