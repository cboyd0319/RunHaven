use super::*;

#[test]
fn shell_confirm_enter_requests_foreground_launch_handoff() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    let action = state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let ShellAction::Launch(prepared) = action else {
        panic!("expected launch handoff action, got {action:?}");
    };
    let output = render_to_text(&mut state, 120, 32);

    assert!(prepared.data.command.contains("container run"));
    assert_eq!(prepared.data.command, prepared.executable.shell_command());
    assert!(output.contains("Launch prepared. Starting in the terminal."));
    assert!(output.contains("container run"));
}

#[tokio::test]
async fn shell_confirmed_launch_runs_launcher_and_returns_to_post_run_recovery() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    let action = state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let ShellAction::Launch(prepared) = action else {
        panic!("expected launch handoff action, got {action:?}");
    };
    let launch = *prepared;
    let expected_command = launch.executable.command.clone();
    let expected_workspace = launch.executable.workspace.clone();
    let launcher_called = Rc::new(Cell::new(false));
    let launcher_called_for_closure = Rc::clone(&launcher_called);

    show_recovery_after_launch(&mut state, launch, move |launch| {
        let expected_command = expected_command.clone();
        let expected_workspace = expected_workspace.clone();
        let launcher_called = Rc::clone(&launcher_called_for_closure);
        async move {
            assert_eq!(launch.executable.command, expected_command);
            assert_eq!(launch.executable.workspace, expected_workspace);
            assert_eq!(launch.data.command, launch.executable.shell_command());
            launcher_called.set(true);
            Ok(13)
        }
    })
    .await;

    assert!(launcher_called.get());
    assert_eq!(state.process_exit_code(), 13);
    let output = render_to_text(&mut state, 120, 32);
    assert!(output.contains("Run finished"));
    assert!(output.contains("exit 13"));
    assert!(output.contains("Enter starts another launch."));
}

#[test]
fn shell_post_run_recovery_keeps_tui_open_with_exit_code() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");
    let launch = confirm_required_preview_for_tests()
        .plan
        .expect("prepared launch");

    state.show_post_run(PostRunOutcome::from_launch(&launch, 7, None));

    let output = render_to_text(&mut state, 120, 32);
    assert!(output.contains("Run finished"));
    assert!(output.contains("exit 7"));
    assert_eq!(state.workspace, launch.executable.workspace);
    assert_eq!(state.process_exit_code(), 7);

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    let output = render_to_text(&mut state, 120, 32);
    assert!(
        output.contains("Choose agent") || output.contains("Choose workspace"),
        "expected launch flow after post-run recovery, got: {output}"
    );
}

#[test]
fn shell_terminal_title_tracks_selected_agent_and_step() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    let title = state.terminal_title();
    assert!(title.contains("RunHaven"));
    assert!(title.contains("Choose agent"));
    assert!(title.contains("antigravity"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    let title = state.terminal_title();
    assert!(title.contains("Choose agent"));
    assert!(title.contains("claude"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    let title = state.terminal_title();
    assert!(title.contains("Review plan"));
    assert!(title.contains("claude"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    let title = state.terminal_title();
    assert!(title.contains("Confirm launch"));
    assert!(title.contains("claude"));
}
