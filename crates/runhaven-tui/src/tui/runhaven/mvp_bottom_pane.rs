use crossterm::event::KeyEvent;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::style::boundary_style;
use crate::style::muted_but_readable_style;
use crate::style::safe_style;
use crate::style::selected_row_style;
use crate::style::warning_style;
use crate::tui::bottom_pane::BottomPaneView;
use crate::tui::bottom_pane::CancellationEvent;
use crate::tui::bottom_pane::ViewCompletion;

use super::ActiveRunAction;
use super::DIFF_CONFIRM_PHRASE;
use super::LOG_CONFIRM_PHRASE;
use super::MvpScreen;
use super::RUNHAVEN_MVP_VIEW_ID;
use super::RunControlState;
use super::RunDiffState;
use super::RunHavenMvpView;
use super::RunLogsState;
use super::mvp_render;

impl RunHavenMvpView {
    pub(super) fn footer_status(&self) -> Line<'static> {
        match &self.screen {
            MvpScreen::Launch => {
                let mut line = self.launch.footer_status_line();
                line.push_span(" · ");
                line.push_span(Span::styled(
                    format!("network {}", self.policy.network.label()),
                    boundary_style(),
                ));
                line.push_span(" · ");
                line.push_span(Span::styled(
                    format!("auth {}", self.policy.auth_scope.as_str()),
                    safe_style(),
                ));
                line
            }
            MvpScreen::ActiveRuns(screen) => Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(" · active runs · "),
                Span::styled(
                    format!("{} found", screen.runs.runs.len()),
                    boundary_style(),
                ),
                Span::raw(" · "),
                Span::styled(
                    "1 launch 2 runs 3 diagnostics h history",
                    muted_but_readable_style(),
                ),
            ]),
            MvpScreen::RunLogs(screen) => Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(" · logs · "),
                Span::styled(screen.run.run_id.clone(), warning_style()),
                Span::raw(" · "),
                Span::styled("raw output", warning_style()),
            ]),
            MvpScreen::RunDiff(screen) => Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(" · run review · "),
                Span::styled(screen.run.run_id.clone(), warning_style()),
                Span::raw(" · "),
                Span::styled("workspace diff", warning_style()),
            ]),
            MvpScreen::RunControl(screen) => Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(" · run control · "),
                Span::styled(screen.action.phrase(), warning_style()),
                Span::raw(" · "),
                Span::styled(screen.run.run_id.clone(), boundary_style()),
            ]),
            MvpScreen::Diagnostics(_) => Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(" · diagnostics · "),
                Span::styled("secret-free", safe_style()),
                Span::raw(" · "),
                Span::styled(
                    "1 launch 2 runs h history r refresh",
                    muted_but_readable_style(),
                ),
            ]),
            MvpScreen::History(screen) => Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(" · history · "),
                Span::styled(format!("{} found", screen.run_count()), boundary_style()),
                Span::raw(" · "),
                Span::styled(
                    "enter review 1 launch 2 runs 3 diagnostics",
                    muted_but_readable_style(),
                ),
            ]),
            MvpScreen::PostRun(outcome) => Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(" · run finished · "),
                Span::styled(
                    format!("exit {}", outcome.exit_code),
                    mvp_render::exit_style(outcome.exit_code, outcome.error.as_ref()),
                ),
            ]),
        }
    }
}
impl BottomPaneView for RunHavenMvpView {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.completion.is_some() {
            return;
        }

        match &mut self.screen {
            MvpScreen::Launch => self.handle_launch_key(key_event),
            MvpScreen::ActiveRuns(screen) => {
                match Self::handle_active_runs_key(screen, key_event) {
                    ActiveRunAction::None => {}
                    ActiveRunAction::Launch => self.show_launch(),
                    ActiveRunAction::Diagnostics => self.show_diagnostics(),
                    ActiveRunAction::History => self.show_history(),
                    ActiveRunAction::Refresh => self.show_active_runs(),
                    ActiveRunAction::OpenLogs => self.show_logs_for_selected_run(),
                    ActiveRunAction::Control(action) => {
                        self.show_run_control_for_selected_run(action);
                    }
                }
            }
            MvpScreen::RunLogs(_) => self.handle_logs_key(key_event),
            MvpScreen::RunDiff(_) => self.handle_run_diff_key(key_event),
            MvpScreen::RunControl(_) => self.handle_run_control_key(key_event),
            MvpScreen::History(_) => self.handle_history_key(key_event),
            MvpScreen::Diagnostics(_) => self.handle_diagnostics_key(key_event),
            MvpScreen::PostRun(_) => self.handle_post_run_key(key_event),
        }
    }

    fn is_complete(&self) -> bool {
        self.completion.is_some()
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.completion
    }

    fn selected_index(&self) -> Option<usize> {
        match &self.screen {
            MvpScreen::Launch => self.launch.selected_index(),
            MvpScreen::ActiveRuns(screen) => Some(screen.selected_idx),
            MvpScreen::History(screen) => Some(screen.selected_idx),
            _ => None,
        }
    }

    fn view_id(&self) -> Option<&'static str> {
        match &self.screen {
            MvpScreen::Launch => self.launch.view_id(),
            _ => Some(RUNHAVEN_MVP_VIEW_ID),
        }
    }

    fn terminal_title(&self) -> Option<String> {
        let title = match &self.screen {
            MvpScreen::Launch => self.launch.terminal_title(),
            MvpScreen::ActiveRuns(_) => "RunHaven | Active runs".to_string(),
            MvpScreen::RunLogs(screen) => format!("RunHaven | Logs | {}", screen.run.run_id),
            MvpScreen::RunDiff(screen) => format!("RunHaven | Run review | {}", screen.run.run_id),
            MvpScreen::RunControl(screen) => {
                format!(
                    "RunHaven | {} | {}",
                    screen.action.title(),
                    screen.run.run_id
                )
            }
            MvpScreen::History(_) => "RunHaven | History".to_string(),
            MvpScreen::Diagnostics(_) => "RunHaven | Diagnostics".to_string(),
            MvpScreen::PostRun(outcome) => {
                format!("RunHaven | Run finished | exit {}", outcome.exit_code)
            }
        };
        Some(title)
    }

    fn footer_status_line(&self) -> Option<Line<'static>> {
        Some(self.footer_status())
    }

    fn accepts_text_input(&self) -> bool {
        match &self.screen {
            MvpScreen::Launch => self.launch.accepts_text_input(),
            MvpScreen::RunLogs(screen) => matches!(screen.state, RunLogsState::Confirm { .. }),
            MvpScreen::RunDiff(screen) => matches!(screen.state, RunDiffState::Confirm { .. }),
            MvpScreen::RunControl(screen) => {
                matches!(screen.state, RunControlState::Confirm { .. })
            }
            _ => false,
        }
    }

    fn footer_help_items(&self) -> Option<Vec<(String, String)>> {
        let mut items = match &self.screen {
            MvpScreen::Launch if self.launch.accepts_text_input() => {
                self.launch.footer_help_items().unwrap_or_default()
            }
            MvpScreen::Launch => vec![
                ("up/down".to_string(), "choose".to_string()),
                ("enter".to_string(), "review".to_string()),
                ("n".to_string(), "network".to_string()),
                ("a".to_string(), "auth scope".to_string()),
                ("2".to_string(), "runs".to_string()),
                ("3".to_string(), "diagnostics".to_string()),
                ("h".to_string(), "history".to_string()),
            ],
            MvpScreen::ActiveRuns(_) => vec![
                ("up/down".to_string(), "choose".to_string()),
                ("enter".to_string(), "logs".to_string()),
                ("r".to_string(), "refresh".to_string()),
                ("s".to_string(), "stop".to_string()),
                ("K".to_string(), "hard stop".to_string()),
                ("x".to_string(), "repair marker".to_string()),
                ("1".to_string(), "launch".to_string()),
                ("3".to_string(), "diagnostics".to_string()),
                ("h".to_string(), "history".to_string()),
            ],
            MvpScreen::RunControl(screen) => match screen.state {
                RunControlState::Confirm { .. } => vec![
                    (
                        screen.action.phrase().to_string(),
                        "type to confirm".to_string(),
                    ),
                    ("enter".to_string(), "run".to_string()),
                    ("esc".to_string(), "back".to_string()),
                ],
                RunControlState::Complete(_) | RunControlState::Error(_) => vec![
                    ("r".to_string(), "refresh runs".to_string()),
                    ("esc".to_string(), "back".to_string()),
                    ("1".to_string(), "launch".to_string()),
                    ("3".to_string(), "diagnostics".to_string()),
                    ("h".to_string(), "history".to_string()),
                ],
            },
            MvpScreen::RunLogs(screen) => match screen.state {
                RunLogsState::Confirm { .. } => vec![
                    ("logs".to_string(), "type to load".to_string()),
                    ("enter".to_string(), "confirm".to_string()),
                    ("esc".to_string(), "back".to_string()),
                ],
                RunLogsState::Loaded(_) | RunLogsState::Error(_) => vec![
                    ("r".to_string(), "refresh".to_string()),
                    ("esc".to_string(), "back".to_string()),
                ],
            },
            MvpScreen::RunDiff(screen) => match screen.state {
                RunDiffState::Confirm { .. } => vec![
                    ("diff".to_string(), "type to load".to_string()),
                    ("enter".to_string(), "confirm".to_string()),
                    ("esc".to_string(), "history".to_string()),
                ],
                RunDiffState::Loaded(_) | RunDiffState::Error(_) => vec![
                    ("r".to_string(), "refresh".to_string()),
                    ("h".to_string(), "history".to_string()),
                    ("esc".to_string(), "history".to_string()),
                ],
            },
            MvpScreen::Diagnostics(_) => vec![
                ("r".to_string(), "refresh".to_string()),
                ("1".to_string(), "launch".to_string()),
                ("2".to_string(), "runs".to_string()),
                ("h".to_string(), "history".to_string()),
            ],
            MvpScreen::History(_) => vec![
                ("up/down".to_string(), "choose".to_string()),
                ("enter".to_string(), "review".to_string()),
                ("r".to_string(), "refresh".to_string()),
                ("1".to_string(), "launch".to_string()),
                ("2".to_string(), "runs".to_string()),
                ("3".to_string(), "diagnostics".to_string()),
            ],
            MvpScreen::PostRun(_) => vec![
                ("enter".to_string(), "new launch".to_string()),
                ("2".to_string(), "runs".to_string()),
                ("3".to_string(), "diagnostics".to_string()),
                ("h".to_string(), "history".to_string()),
                ("q".to_string(), "quit".to_string()),
            ],
        };
        if !items.iter().any(|(key, _)| key == "?") {
            items.push(("?".to_string(), "hide help".to_string()));
        }
        Some(items)
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.completion = Some(ViewCompletion::Cancelled);
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool {
        true
    }

    fn handle_paste(&mut self, pasted: String) -> bool {
        match &mut self.screen {
            MvpScreen::Launch => {
                if pasted.is_empty() {
                    false
                } else {
                    self.launch.handle_paste(&pasted);
                    true
                }
            }
            MvpScreen::RunLogs(screen) => {
                if let RunLogsState::Confirm { notice, .. } = &mut screen.state
                    && !pasted.is_empty()
                {
                    *notice = Some(format!(
                        "Type {LOG_CONFIRM_PHRASE} by hand. Paste is ignored here."
                    ));
                    return true;
                }
                false
            }
            MvpScreen::RunDiff(screen) => {
                if let RunDiffState::Confirm { notice, .. } = &mut screen.state
                    && !pasted.is_empty()
                {
                    *notice = Some(format!(
                        "Type {DIFF_CONFIRM_PHRASE} by hand. Paste is ignored here."
                    ));
                    return true;
                }
                false
            }
            MvpScreen::RunControl(screen) => {
                if let RunControlState::Confirm { notice, .. } = &mut screen.state
                    && !pasted.is_empty()
                {
                    *notice = Some(screen.action.paste_notice());
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}
