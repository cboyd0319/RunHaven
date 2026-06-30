use super::*;
use crate::tui::runhaven::service::confirm_required_preview_for_tests;
use crossterm::event::KeyModifiers;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::cell::Cell;
use std::rc::Rc;

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

fn confirm_required_shell_state() -> ShellState {
    ShellState::from_launch_wizard(LaunchWizardView::new(
        PathBuf::from("/tmp/project"),
        vec![confirm_required_preview_for_tests()],
    ))
    .expect("state")
}

fn render_to_text(state: &mut ShellState, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal.draw(|frame| render(frame, state)).expect("draw");
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

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(terminal.size().expect("terminal size").width as usize)
        .map(|row| {
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
