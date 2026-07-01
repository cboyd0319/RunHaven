use super::*;
use crate::tui::app_event::AppEvent;

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
