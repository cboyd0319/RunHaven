use super::*;

#[test]
fn shell_state_builds_default_launch_previews() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    let output = render_to_text(&mut state, 120, 32);
    assert!(output.contains("antigravity"));
    assert!(output.contains("Ready. Provider only. Workspace only."));
    assert!(!output.contains("Google Antigravity CLI"));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn shell_key_handling_moves_selection_and_quits() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert!(render_to_text(&mut state, 120, 32).contains("Step 3/4: Review plan"));
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
        ShellAction::Quit
    );
}

#[test]
fn shell_escape_cancels_source_picker() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        ShellAction::Quit
    );
}

#[test]
fn shell_review_step_is_read_only_and_can_go_back() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert!(render_to_text(&mut state, 120, 32).contains("Step 3/4: Review plan"));

    let output = render_to_text(&mut state, 120, 48);
    assert!(output.contains("Step 3/4: Review plan"));
    assert!(output.contains("Check what RunHaven will share before launch."));
    assert!(output.contains("Exact command"));
    assert!(output.contains("container run"));
    assert!(output.contains("Host home"));
    assert!(output.contains("not mounted"));
    assert!(output.contains("Credentials"));
    assert!(output.contains("not mounted by default"));
    assert!(output.contains("opens confirmation"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert!(render_to_text(&mut state, 120, 32).contains("Step 1/4: Choose agent"));
}

#[test]
fn shell_review_escape_returns_to_picker_instead_of_quitting() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert!(render_to_text(&mut state, 120, 32).contains("Step 3/4: Review plan"));
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert!(render_to_text(&mut state, 120, 32).contains("Step 1/4: Choose agent"));
}

#[test]
fn shell_review_enter_opens_confirm_step() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert!(render_to_text(&mut state, 120, 32).contains("Step 3/4: Review plan"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );

    assert!(render_to_text(&mut state, 120, 32).contains("Step 4/4: Confirm launch"));
}

#[test]
fn shell_confirm_escape_returns_to_review_instead_of_quitting() {
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
    assert!(render_to_text(&mut state, 120, 32).contains("Step 4/4: Confirm launch"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        ShellAction::Continue
    );

    assert!(render_to_text(&mut state, 120, 32).contains("Step 3/4: Review plan"));
}

#[test]
fn shell_q_still_quits_from_confirm() {
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
    assert!(render_to_text(&mut state, 120, 32).contains("Step 4/4: Confirm launch"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
        ShellAction::Quit
    );
}

#[test]
fn shell_typed_confirm_captures_shortcuts_and_rejects_paste() {
    let mut state = confirm_required_shell_state();

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert!(state.show_footer_help);
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );

    assert!(state.confirm_accepts_text_input());
    assert!(!state.show_footer_help);
    let output = render_to_text(&mut state, 120, 32);
    assert!(!output.contains("? help"));
    assert!(!output.contains("q quits"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert!(render_to_text(&mut state, 120, 32).contains("?q"));
    assert!(!state.show_footer_help);

    state.handle_paste("launch");
    assert!(render_to_text(&mut state, 120, 32).contains("?q"));
    let output = render_to_text(&mut state, 120, 32);
    assert!(output.contains("Paste is ignored here."));

    let mut terminal = Terminal::new(TestBackend::new(120, 32)).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &mut state))
        .expect("draw");
    let cursor = terminal.backend().cursor_position();
    assert!(cursor.x > 0);
    assert!(cursor.y > 0);
}

#[test]
fn shell_typed_confirm_phrase_requests_foreground_launch_handoff() {
    let mut state = confirm_required_shell_state();

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    for ch in "launch".chars() {
        assert_eq!(
            state.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)),
            ShellAction::Continue
        );
    }

    let action = state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let ShellAction::Launch(prepared) = action else {
        panic!("expected typed launch handoff action, got {action:?}");
    };

    assert!(prepared.data.confirm_required);
    assert_eq!(prepared.data.command, prepared.executable.shell_command());
}
