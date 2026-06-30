use std::future::Future;
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
                runtime.block_on(show_recovery_after_launch(&mut state, launch, |launch| {
                    super::runhaven::launch_handoff::launch_prepared(&mut tui, launch)
                }));
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

async fn show_recovery_after_launch<F, Fut>(
    state: &mut ShellState,
    launch: PreparedLaunch,
    launcher: F,
) where
    F: FnOnce(PreparedLaunch) -> Fut,
    Fut: Future<Output = Result<i32>>,
{
    let success_outcome = PostRunOutcome::from_launch(&launch, 0, None);
    match launcher(launch).await {
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
#[path = "app_shell_tests.rs"]
mod tests;
