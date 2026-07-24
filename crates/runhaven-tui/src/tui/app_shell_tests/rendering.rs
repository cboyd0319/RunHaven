use super::*;

#[test]
fn shell_render_shows_launch_contract_data() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");
    let output = render_to_text(&mut state, 120, 48);

    assert!(output.contains("RunHaven"));
    assert!(output.contains("Step 1/4: Choose agent"));
    assert!(output.contains("Pick the agent to run"));
    assert!(output.contains("review the plan before anything starts"));
    assert!(output.contains("Safety"));
    assert!(output.contains("/workspace only"));
    assert!(output.contains("Host home and credentials are not mounted"));
    assert!(output.contains("Provider only"));
    assert!(output.contains("Ready. Provider only. Workspace only."));
    assert!(!output.contains("Google Antigravity CLI"));
    assert!(!output.contains("Plan Preview"));
    assert!(!output.contains("Exact command"));
    assert!(!output.contains("container run"));
}

#[test]
fn shell_render_review_keeps_command_and_boundary_visible_on_80x24() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    let output = render_to_text(&mut state, 80, 24);

    assert!(output.contains("Step 3/4: Review plan"));
    assert!(output.contains("Boundary"));
    assert!(output.contains("/workspace only"));
    assert!(output.contains("Host home"));
    assert!(output.contains("Credentials"));
    assert!(output.contains("Exact command"));
    assert!(output.contains("container run"));
}

#[test]
fn shell_confirm_render_keeps_command_and_boundary_visible_on_80x24() {
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
    let output = render_to_text(&mut state, 80, 24);

    assert!(output.contains("Step 4/4: Confirm launch"));
    assert!(output.contains("Boundary"));
    assert!(output.contains("/workspace only"));
    assert!(output.contains("Host home"));
    assert!(output.contains("Credentials"));
    assert!(output.contains("Exact command"));
    assert!(output.contains("container run"));
}

#[test]
fn shell_review_render_clears_previous_picker_buffer() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).expect("test terminal");

    terminal
        .draw(|frame| render(frame, &mut state))
        .expect("draw");
    assert!(
        buffer_text(&terminal).contains("Pick the agent to run"),
        "test setup should render the simplified picker first"
    );

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    terminal
        .draw(|frame| render(frame, &mut state))
        .expect("draw");
    let output = buffer_text(&terminal);

    assert!(output.contains("Step 3/4: Review plan"));
    assert!(output.contains("Exact command"));
    assert!(!output.contains("Pick the agent to run"));
}

#[test]
fn shell_confirm_render_clears_previous_review_buffer() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).expect("test terminal");

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    terminal
        .draw(|frame| render(frame, &mut state))
        .expect("draw");
    assert!(
        buffer_text(&terminal).contains("Step 3/4: Review plan"),
        "test setup should render review first"
    );

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    terminal
        .draw(|frame| render(frame, &mut state))
        .expect("draw");
    let output = buffer_text(&terminal);

    assert!(output.contains("Step 4/4: Confirm launch"));
    assert!(output.contains("Exact command"));
    assert!(!output.contains("Step 3/4: Review plan"));
}

#[test]
fn shell_render_keeps_boundary_visible_on_80x24() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");
    let output = render_to_text(&mut state, 80, 24);

    assert!(output.contains("Safety"));
    assert!(output.contains("/workspace only"));
    assert!(output.contains("Host home and credentials"));
    assert!(output.contains("Provider only"));
}

#[test]
fn shell_selection_updates_source_picker_preview_state() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    assert!(state.terminal_title().contains("antigravity"));
    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        ShellAction::Continue
    );
    assert!(state.terminal_title().contains("claude"));
}

#[test]
fn shell_footer_shows_status_and_help_overlay() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut state = ShellState::for_workspace(workspace.path()).expect("state");

    let output = render_to_text(&mut state, 120, 32);
    assert!(output.contains("? help"));
    assert!(output.contains("Choose agent"));
    assert!(output.contains("Provider only"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)),
        ShellAction::Continue
    );
    let output = render_to_text(&mut state, 120, 32);

    assert!(output.contains("up/down"));
    assert!(output.contains("review"));
    assert!(output.contains("hide help"));
}

#[test]
fn shell_confirm_footer_help_and_status_track_step_four() {
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
    let output = render_to_text(&mut state, 120, 32);
    assert!(output.contains("? help"));
    assert!(output.contains("Confirm launch"));

    assert_eq!(
        state.handle_key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)),
        ShellAction::Continue
    );
    let output = render_to_text(&mut state, 120, 32);

    assert!(output.contains("enter"));
    assert!(output.contains("confirm"));
    assert!(output.contains("hide help"));
}
