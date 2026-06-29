use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Clear;
use ratatui::widgets::Widget;

use super::runhaven::launch_wizard::LAUNCH_WIZARD_VIEW_ID;
#[cfg(test)]
use super::runhaven::launch_wizard::LaunchWizardView;
use super::runhaven::mvp::PostRunOutcome;
use super::runhaven::mvp::RUNHAVEN_MVP_VIEW_ID;
use super::runhaven::mvp::RunHavenMvpView;
use super::runhaven::service::PreparedLaunch;
use crate::key_hint;
use crate::render::renderable::Renderable;
use crate::tui::app_event::AppEvent;
use crate::tui::bottom_pane::AppEventSender;
use crate::tui::bottom_pane::BottomPane;
use crate::tui::bottom_pane::BottomPaneParams;
use crate::tui::bottom_pane::FooterKeyHints;
use crate::tui::bottom_pane::FooterMode;
use crate::tui::bottom_pane::FooterProps;
use crate::tui::codex_runtime;
use crate::tui::codex_runtime::TuiEvent;
use crate::tui::codex_runtime::restore_after_exit;
use crate::tui::terminal_title::SetTerminalTitleResult;
use crate::tui::terminal_title::clear_terminal_title;
use crate::tui::terminal_title::set_terminal_title;
#[cfg(test)]
use ratatui::Frame;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

pub(crate) fn run() -> Result<i32> {
    let initialized = codex_runtime::init().context("initialize Codex terminal runtime")?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .context("start Codex TUI runtime")?;
    let mut tui = {
        let _guard = runtime.enter();
        codex_runtime::Tui::new(
            initialized.terminal,
            initialized.enhanced_keys_supported,
            initialized.stderr_guard,
        )
    };
    let mut restore_guard = CodexTerminalRestoreGuard::new();
    let mut state = ShellState::for_current_dir(tui.frame_requester())?;

    let exit_result = loop {
        match runtime.block_on(run_loop(&mut tui, &mut state)) {
            Ok(ShellExit::Quit) => {
                tui.terminal.clear().context("clear terminal UI")?;
                break Ok(state.process_exit_code());
            }
            Ok(ShellExit::Launch(launch)) => {
                let launch = *launch;
                let success_outcome = PostRunOutcome::from_launch(&launch, 0, None);
                match runtime.block_on(super::runhaven::launch_handoff::launch_prepared(
                    &mut tui, launch,
                )) {
                    Ok(exit_code) => {
                        state.show_post_run(PostRunOutcome {
                            exit_code,
                            ..success_outcome
                        });
                    }
                    Err(error) => {
                        state.show_post_run(PostRunOutcome {
                            exit_code: 1,
                            error: Some(error.to_string()),
                            ..success_outcome
                        });
                    }
                }
                tui.frame_requester().schedule_frame();
            }
            Err(error) => break Err(error),
        }
    };
    drop(tui);
    restore_guard.restore()?;
    exit_result
}

struct CodexTerminalRestoreGuard {
    active: bool,
}

impl CodexTerminalRestoreGuard {
    fn new() -> Self {
        Self { active: true }
    }

    fn restore(&mut self) -> Result<()> {
        if self.active {
            let _ = clear_terminal_title();
            restore_after_exit().context("restore Codex terminal runtime")?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for CodexTerminalRestoreGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = clear_terminal_title();
            if let Err(err) = restore_after_exit() {
                tracing::warn!(error = %err, "failed to restore Codex terminal runtime");
            }
            self.active = false;
        }
    }
}

async fn run_loop(tui: &mut codex_runtime::Tui, state: &mut ShellState) -> Result<ShellExit> {
    let tui_events = tui.event_stream();
    tokio::pin!(tui_events);
    tui.frame_requester().schedule_frame();

    while let Some(event) = tui_events.next().await {
        match event {
            TuiEvent::Key(key) => match state.handle_key(key) {
                ShellAction::Continue => tui.frame_requester().schedule_frame(),
                ShellAction::Quit => return Ok(ShellExit::Quit),
                ShellAction::Launch(launch) => return Ok(ShellExit::Launch(launch)),
            },
            TuiEvent::Paste(pasted) => {
                state.handle_paste(&pasted);
                tui.frame_requester().schedule_frame();
            }
            TuiEvent::Resize | TuiEvent::Draw => {
                state.refresh_terminal_title();
                let height = tui.terminal.size()?.height;
                tui.draw(height, |frame| render_custom(frame, state))?;
            }
        }
    }
    Ok(ShellExit::Quit)
}

struct ShellState {
    workspace: PathBuf,
    bottom_pane: BottomPane,
    app_event_tx: AppEventSender,
    app_event_rx: mpsc::UnboundedReceiver<crate::app_event::AppEvent>,
    show_footer_help: bool,
    last_terminal_title: Option<String>,
    process_exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ShellAction {
    Continue,
    Quit,
    Launch(Box<PreparedLaunch>),
}

#[derive(Debug)]
enum ShellExit {
    Quit,
    Launch(Box<PreparedLaunch>),
}

impl ShellState {
    fn for_current_dir(frame_requester: crate::tui::FrameRequester) -> Result<Self> {
        Self::for_workspace_with_frame_requester(std::env::current_dir()?, frame_requester)
    }

    #[cfg(test)]
    fn for_workspace(workspace: impl AsRef<Path>) -> Result<Self> {
        Self::for_workspace_with_frame_requester(
            workspace,
            crate::tui::FrameRequester::test_dummy(),
        )
    }

    fn for_workspace_with_frame_requester(
        workspace: impl AsRef<Path>,
        frame_requester: crate::tui::FrameRequester,
    ) -> Result<Self> {
        let workspace = workspace.as_ref().to_path_buf();
        let mvp_view = RunHavenMvpView::new(workspace.clone());
        Self::from_mvp_view_with_frame_requester(workspace, mvp_view, frame_requester)
    }

    #[cfg(test)]
    fn from_launch_wizard(launch_wizard: LaunchWizardView) -> Result<Self> {
        Self::from_mvp_view_with_frame_requester(
            PathBuf::from("/tmp/project"),
            RunHavenMvpView::from_launch_wizard_for_tests(
                PathBuf::from("/tmp/project"),
                launch_wizard,
            ),
            crate::tui::FrameRequester::test_dummy(),
        )
    }

    fn from_mvp_view_with_frame_requester(
        workspace: PathBuf,
        mvp_view: RunHavenMvpView,
        frame_requester: crate::tui::FrameRequester,
    ) -> Result<Self> {
        let (app_event_tx, app_event_rx) = mpsc::unbounded_channel();
        let app_event_tx = AppEventSender::new(app_event_tx);
        let mut mvp_view = mvp_view;
        mvp_view.set_app_event_sender(app_event_tx.clone());
        let mut bottom_pane = BottomPane::new(BottomPaneParams {
            app_event_tx: app_event_tx.clone(),
            frame_requester,
            has_input_focus: true,
            enhanced_keys_supported: false,
            placeholder_text: String::new(),
            disable_paste_burst: true,
            animations_enabled: true,
            skills: None,
        });
        bottom_pane.show_view(Box::new(mvp_view));

        Ok(Self {
            workspace,
            bottom_pane,
            app_event_tx,
            app_event_rx,
            show_footer_help: false,
            last_terminal_title: None,
            process_exit_code: 0,
        })
    }

    fn handle_key(&mut self, key: KeyEvent) -> ShellAction {
        if key.kind != KeyEventKind::Press {
            return ShellAction::Continue;
        }

        if self.confirm_accepts_text_input() {
            self.show_footer_help = false;
            self.bottom_pane.handle_key_event(key);
            return match self.drain_app_events() {
                Some(launch) => ShellAction::Launch(Box::new(launch)),
                None => ShellAction::Continue,
            };
        }

        if matches!(key.code, KeyCode::Char('q')) {
            return ShellAction::Quit;
        }

        if matches!(key.code, KeyCode::Char('?')) {
            self.show_footer_help = !self.show_footer_help;
            return ShellAction::Continue;
        }

        if self.show_footer_help && matches!(key.code, KeyCode::Esc) {
            self.show_footer_help = false;
            return ShellAction::Continue;
        }

        self.bottom_pane.handle_key_event(key);
        if let Some(launch) = self.drain_app_events() {
            return ShellAction::Launch(Box::new(launch));
        }
        if self.confirm_accepts_text_input() {
            self.show_footer_help = false;
        }
        if !self.bottom_pane.has_active_view() {
            ShellAction::Quit
        } else {
            ShellAction::Continue
        }
    }

    fn handle_paste(&mut self, pasted: &str) {
        self.show_footer_help = false;
        self.bottom_pane.handle_paste(pasted.to_string());
    }

    fn drain_app_events(&mut self) -> Option<PreparedLaunch> {
        let mut prepared_launch = None;
        while let Ok(event) = self.app_event_rx.try_recv() {
            match event {
                AppEvent::RunHavenLaunchPrepared { launch } => {
                    prepared_launch = Some(*launch);
                }
                _ => {
                    tracing::debug!("staging RunHaven TUI shell ignored unsupported app event");
                }
            }
        }
        prepared_launch
    }

    fn show_post_run(&mut self, outcome: PostRunOutcome) {
        self.process_exit_code = outcome.exit_code;
        self.workspace = outcome.workspace.clone();
        let _ = self
            .bottom_pane
            .dismiss_active_view_if_id(LAUNCH_WIZARD_VIEW_ID);
        let _ = self
            .bottom_pane
            .dismiss_active_view_if_id(RUNHAVEN_MVP_VIEW_ID);
        let mut view = RunHavenMvpView::new(outcome.workspace.clone());
        view.show_post_run(outcome);
        view.set_app_event_sender(self.app_event_tx.clone());
        self.bottom_pane.show_view(Box::new(view));
    }

    fn process_exit_code(&self) -> i32 {
        self.process_exit_code
    }

    fn refresh_terminal_title(&mut self) {
        let title = self.terminal_title();
        if self.last_terminal_title.as_deref() == Some(title.as_str()) {
            return;
        }

        match set_terminal_title(&title) {
            Ok(SetTerminalTitleResult::Applied) => {
                self.last_terminal_title = Some(title);
            }
            Ok(SetTerminalTitleResult::NoVisibleContent) => {
                self.last_terminal_title = None;
                let _ = clear_terminal_title();
            }
            Err(_) => {}
        }
    }

    fn terminal_title(&self) -> String {
        self.bottom_pane
            .active_view_terminal_title()
            .unwrap_or_else(|| "RunHaven".to_string())
    }

    fn confirm_accepts_text_input(&self) -> bool {
        self.bottom_pane.active_view_accepts_text_input()
    }

    fn footer_height(&self) -> u16 {
        if self.show_footer_help {
            return 2;
        }
        crate::tui::bottom_pane::footer_height(&self.footer_props())
    }

    fn render_footer(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Clear.render(area, buf);
        let props = self.footer_props();
        if self.show_footer_help {
            let status_area = Rect { height: 1, ..area };
            let help_area = Rect {
                y: area.y.saturating_add(1),
                height: area.height.saturating_sub(1),
                ..area
            };
            crate::tui::bottom_pane::render_footer_from_props(
                status_area,
                buf,
                &props,
                None,
                false,
                false,
                false,
            );
            crate::tui::bottom_pane::render_footer_hint_items(
                help_area,
                buf,
                &self.footer_help_items(),
            );
            return;
        }

        crate::tui::bottom_pane::render_footer_from_props(
            area, buf, &props, None, false, false, false,
        );
    }

    fn footer_props(&self) -> FooterProps {
        FooterProps {
            mode: FooterMode::ComposerEmpty,
            esc_backtrack_hint: false,
            use_shift_enter_hint: false,
            is_task_running: false,
            queue_submissions: false,
            collaboration_modes_enabled: false,
            is_wsl: false,
            quit_shortcut_key: key_hint::plain(KeyCode::Char('q')),
            status_line_value: self.bottom_pane.active_view_footer_status_line(),
            status_line_enabled: true,
            key_hints: FooterKeyHints {
                toggle_shortcuts: if self.confirm_accepts_text_input() {
                    None
                } else {
                    Some(key_hint::plain(KeyCode::Char('?')))
                },
                queue: None,
                insert_newline: None,
                external_editor: None,
                edit_previous: Some(key_hint::plain(KeyCode::Esc)),
                show_transcript: None,
                history_search: None,
                reasoning_down: None,
                reasoning_up: None,
            },
            active_agent_label: None,
        }
    }

    fn footer_help_items(&self) -> Vec<(String, String)> {
        self.bottom_pane
            .active_view_footer_help_items()
            .unwrap_or_default()
    }

    #[cfg(test)]
    fn selected_index(&self) -> Option<usize> {
        self.bottom_pane
            .selected_index_for_active_view(LAUNCH_WIZARD_VIEW_ID)
    }
}

#[cfg(test)]
fn render(frame: &mut Frame<'_>, state: &mut ShellState) {
    let area = frame.area();
    let cursor = render_buffer(area, frame.buffer_mut(), state);
    if let Some(cursor) = cursor {
        frame.set_cursor_position(cursor);
    }
}

fn render_custom(frame: &mut crate::custom_terminal::Frame<'_>, state: &mut ShellState) {
    let area = frame.area();
    let cursor = render_buffer(area, frame.buffer, state);
    if let Some(cursor) = cursor {
        frame.set_cursor_position(cursor);
    }
}

fn render_buffer(area: Rect, buf: &mut Buffer, state: &mut ShellState) -> Option<(u16, u16)> {
    Clear.render(area, buf);
    let footer_height = state.footer_height().min(area.height);
    let content_height = area.height.saturating_sub(footer_height);
    let content_area = Rect {
        height: content_height,
        ..area
    };
    let footer_area = Rect {
        y: area.y.saturating_add(content_height),
        height: footer_height,
        ..area
    };
    Renderable::render(&state.bottom_pane, content_area, buf);
    state.render_footer(footer_area, buf);
    state.bottom_pane.cursor_pos(content_area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::runhaven::service::confirm_required_preview_for_tests;
    use crossterm::event::KeyModifiers;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn shell_state_builds_default_launch_previews() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut state = ShellState::for_workspace(workspace.path()).expect("state");

        let output = render_to_text(&mut state, 120, 32);
        assert!(output.contains("antigravity"));
        assert!(output.contains("Google Antigravity CLI"));
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
        assert!(output.contains("Choose an agent"));
        assert!(output.contains("RunHaven will show the full plan before launch"));
        assert!(output.contains("Safety"));
        assert!(output.contains("/workspace only"));
        assert!(output.contains("Host home and credentials are not mounted"));
        assert!(output.contains("provider allowlist"));
        assert!(output.contains("Google Antigravity CLI"));
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
            buffer_text(&terminal).contains("Choose an agent"),
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
        assert!(!output.contains("Choose an agent. RunHaven will show the full plan"));
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
        assert!(output.contains("provider allowlist"));
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
        assert!(output.contains("provider allowlist"));

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
}
