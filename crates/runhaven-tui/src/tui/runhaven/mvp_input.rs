use crate::tui::bottom_pane::BottomPaneView;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use runhaven_core::runtime::active::DEFAULT_LOG_SNAPSHOT_LINES;
use runhaven_core::ui_contracts::ActiveRunSummaryData;
use runhaven_core::ui_contracts::RunHistorySummaryData;

use super::ActiveRunsScreen;
use super::DIFF_CONFIRM_PHRASE;
use super::HistoryScreen;
use super::LOG_CONFIRM_PHRASE;
use super::MvpScreen;
use super::RunControlAction;
use super::RunControlState;
use super::RunDiffState;
use super::RunHavenMvpView;
use super::RunLogsState;

impl RunHavenMvpView {
    pub(super) fn handle_launch_key(&mut self, key_event: KeyEvent) {
        if self.launch.confirm_accepts_text_input() {
            self.launch.handle_key_event(key_event);
            return;
        }

        match key_event.code {
            KeyCode::Tab => self.cycle_section(),
            KeyCode::Char('1') if key_event.modifiers == KeyModifiers::NONE => self.show_launch(),
            KeyCode::Char('2') if key_event.modifiers == KeyModifiers::NONE => {
                self.show_active_runs();
            }
            KeyCode::Char('3') if key_event.modifiers == KeyModifiers::NONE => {
                self.show_diagnostics();
            }
            KeyCode::Char('4') | KeyCode::Char('h')
                if key_event.modifiers == KeyModifiers::NONE =>
            {
                self.show_history();
            }
            KeyCode::Char('n') if key_event.modifiers == KeyModifiers::NONE => {
                self.cycle_network_policy();
            }
            KeyCode::Char('a') if key_event.modifiers == KeyModifiers::NONE => {
                self.toggle_auth_scope();
            }
            KeyCode::Char('r') if key_event.modifiers == KeyModifiers::NONE => {
                self.rebuild_launch();
            }
            _ => {
                self.launch.handle_key_event(key_event);
                if self.launch.is_complete() {
                    self.completion = self.launch.completion();
                }
            }
        }
    }

    pub(super) fn handle_active_runs_key(
        screen: &mut ActiveRunsScreen,
        key_event: KeyEvent,
    ) -> ActiveRunAction {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('1') => ActiveRunAction::Launch,
            KeyCode::Tab | KeyCode::Char('3') => ActiveRunAction::Diagnostics,
            KeyCode::Char('4') | KeyCode::Char('h') => ActiveRunAction::History,
            KeyCode::Char('r') => ActiveRunAction::Refresh,
            KeyCode::Char('s') => {
                if screen.selected_run().is_some() {
                    ActiveRunAction::Control(RunControlAction::Stop)
                } else {
                    screen.notice = Some("No active runs are available.".to_string());
                    ActiveRunAction::None
                }
            }
            KeyCode::Char('K') => {
                if screen.selected_run().is_some() {
                    ActiveRunAction::Control(RunControlAction::Kill)
                } else {
                    screen.notice = Some("No active runs are available.".to_string());
                    ActiveRunAction::None
                }
            }
            KeyCode::Char('x') => {
                if screen.selected_run().is_some() {
                    ActiveRunAction::Control(RunControlAction::Repair)
                } else {
                    screen.notice = Some("No active runs are available.".to_string());
                    ActiveRunAction::None
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                screen.select_previous();
                ActiveRunAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                screen.select_next();
                ActiveRunAction::None
            }
            KeyCode::Enter => {
                if screen.selected_run().is_some() {
                    ActiveRunAction::OpenLogs
                } else {
                    screen.notice = Some("No active runs are available.".to_string());
                    ActiveRunAction::None
                }
            }
            _ => ActiveRunAction::None,
        }
    }

    pub(super) fn handle_logs_key(&mut self, key_event: KeyEvent) {
        let MvpScreen::RunLogs(screen) = &mut self.screen else {
            return;
        };
        match &mut screen.state {
            RunLogsState::Confirm { typed, notice } => match key_event.code {
                KeyCode::Esc => self.show_active_runs(),
                KeyCode::Backspace => {
                    typed.pop();
                    *notice = None;
                }
                KeyCode::Enter => {
                    if typed.trim() != LOG_CONFIRM_PHRASE {
                        *notice = Some(format!("Type {LOG_CONFIRM_PHRASE} before loading logs."));
                        return;
                    }
                    screen.state = self
                        .service
                        .active_run_log_snapshot_data(
                            &screen.run.run_id,
                            DEFAULT_LOG_SNAPSHOT_LINES,
                            true,
                            "runhaven/run/logSnapshot",
                        )
                        .map(RunLogsState::Loaded)
                        .unwrap_or_else(|error| RunLogsState::Error(error.to_string()));
                }
                KeyCode::Char(ch) => {
                    typed.push(ch);
                    *notice = None;
                }
                _ => {}
            },
            RunLogsState::Loaded(_) | RunLogsState::Error(_) => match key_event.code {
                KeyCode::Esc => self.show_active_runs(),
                KeyCode::Char('r') => {
                    screen.state = self
                        .service
                        .active_run_log_snapshot_data(
                            &screen.run.run_id,
                            DEFAULT_LOG_SNAPSHOT_LINES,
                            true,
                            "runhaven/run/logSnapshot",
                        )
                        .map(RunLogsState::Loaded)
                        .unwrap_or_else(|error| RunLogsState::Error(error.to_string()));
                }
                _ => {}
            },
        }
    }

    pub(super) fn handle_run_diff_key(&mut self, key_event: KeyEvent) {
        let MvpScreen::RunDiff(screen) = &mut self.screen else {
            return;
        };
        match &mut screen.state {
            RunDiffState::Confirm { typed, notice } => match key_event.code {
                KeyCode::Esc => self.show_history(),
                KeyCode::Backspace => {
                    typed.pop();
                    *notice = None;
                }
                KeyCode::Enter => {
                    if typed.trim() != DIFF_CONFIRM_PHRASE {
                        *notice = Some(format!(
                            "Type {DIFF_CONFIRM_PHRASE} before loading the diff."
                        ));
                        return;
                    }
                    screen.state = self
                        .service
                        .run_diff_data(&screen.run.run_id, true, "runhaven/run/diff")
                        .map(RunDiffState::Loaded)
                        .unwrap_or_else(|error| RunDiffState::Error(error.to_string()));
                }
                KeyCode::Char(ch) => {
                    typed.push(ch);
                    *notice = None;
                }
                _ => {}
            },
            RunDiffState::Loaded(_) | RunDiffState::Error(_) => match key_event.code {
                KeyCode::Esc | KeyCode::Char('4') | KeyCode::Char('h') => self.show_history(),
                KeyCode::Char('1') => self.show_launch(),
                KeyCode::Char('2') => self.show_active_runs(),
                KeyCode::Tab | KeyCode::Char('3') => self.show_diagnostics(),
                KeyCode::Char('r') => {
                    screen.state = self
                        .service
                        .run_diff_data(&screen.run.run_id, true, "runhaven/run/diff")
                        .map(RunDiffState::Loaded)
                        .unwrap_or_else(|error| RunDiffState::Error(error.to_string()));
                }
                _ => {}
            },
        }
    }

    pub(super) fn handle_run_control_key(&mut self, key_event: KeyEvent) {
        let mut execute = None;
        let MvpScreen::RunControl(screen) = &mut self.screen else {
            return;
        };
        match &mut screen.state {
            RunControlState::Confirm { typed, notice } => match key_event.code {
                KeyCode::Esc => self.show_active_runs(),
                KeyCode::Backspace => {
                    typed.pop();
                    *notice = None;
                }
                KeyCode::Enter => {
                    if typed.trim() != screen.action.phrase() {
                        *notice = Some(screen.action.missing_confirmation_notice().to_string());
                        return;
                    }
                    execute = Some((screen.action, screen.run.run_id.clone()));
                }
                KeyCode::Char(ch) => {
                    typed.push(ch);
                    *notice = None;
                }
                _ => {}
            },
            RunControlState::Complete(_) | RunControlState::Error(_) => match key_event.code {
                KeyCode::Esc | KeyCode::Char('2') | KeyCode::Char('r') => self.show_active_runs(),
                KeyCode::Char('1') => self.show_launch(),
                KeyCode::Tab | KeyCode::Char('3') => self.show_diagnostics(),
                KeyCode::Char('4') | KeyCode::Char('h') => self.show_history(),
                _ => {}
            },
        }

        if let Some((action, run_id)) = execute {
            let state = self.run_control_result_state(action, &run_id);
            if let MvpScreen::RunControl(screen) = &mut self.screen {
                screen.state = state;
            }
        }
    }

    fn run_control_result_state(&self, action: RunControlAction, run_id: &str) -> RunControlState {
        let result = match action {
            RunControlAction::Stop => self.service.stop_run_data(run_id, true, action.method()),
            RunControlAction::Kill => self.service.kill_run_data(run_id, true, action.method()),
            RunControlAction::Repair => self.service.repair_run_data(run_id, true, action.method()),
        };
        result
            .map(RunControlState::Complete)
            .unwrap_or_else(|error| RunControlState::Error(error.to_string()))
    }

    pub(super) fn handle_diagnostics_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('1') => self.show_launch(),
            KeyCode::Tab | KeyCode::Char('2') => self.show_active_runs(),
            KeyCode::Char('4') | KeyCode::Char('h') => self.show_history(),
            KeyCode::Char('r') => self.show_diagnostics(),
            _ => {}
        }
    }

    pub(super) fn handle_history_key(&mut self, key_event: KeyEvent) {
        let action = {
            let MvpScreen::History(screen) = &mut self.screen else {
                return;
            };
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('1') => HistoryAction::Launch,
                KeyCode::Char('2') => HistoryAction::ActiveRuns,
                KeyCode::Tab | KeyCode::Char('3') => HistoryAction::Diagnostics,
                KeyCode::Char('r') => HistoryAction::Refresh,
                KeyCode::Up | KeyCode::Char('k') => {
                    screen.select_previous();
                    HistoryAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    screen.select_next();
                    HistoryAction::None
                }
                KeyCode::Enter => {
                    if screen.selected_run().is_some() {
                        HistoryAction::OpenDiff(screen.selected_idx)
                    } else {
                        screen.notice = Some("No run records are available.".to_string());
                        HistoryAction::None
                    }
                }
                _ => HistoryAction::None,
            }
        };

        match action {
            HistoryAction::None => {}
            HistoryAction::Launch => self.show_launch(),
            HistoryAction::ActiveRuns => self.show_active_runs(),
            HistoryAction::Diagnostics => self.show_diagnostics(),
            HistoryAction::Refresh => self.show_history(),
            HistoryAction::OpenDiff(selected_idx) => self.show_diff_for_history_index(selected_idx),
        }
    }

    pub(super) fn handle_post_run_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('1') => self.show_launch(),
            KeyCode::Char('2') => self.show_active_runs(),
            KeyCode::Tab | KeyCode::Char('3') => self.show_diagnostics(),
            KeyCode::Char('4') | KeyCode::Char('h') => self.show_history(),
            _ => {}
        }
    }
}

pub(super) enum ActiveRunAction {
    None,
    Launch,
    Diagnostics,
    History,
    Refresh,
    OpenLogs,
    Control(RunControlAction),
}

enum HistoryAction {
    None,
    Launch,
    ActiveRuns,
    Diagnostics,
    Refresh,
    OpenDiff(usize),
}

impl ActiveRunsScreen {
    pub(super) fn selected_run(&self) -> Option<&ActiveRunSummaryData> {
        self.runs
            .runs
            .get(self.selected_idx)
            .or_else(|| self.runs.runs.first())
    }

    pub(super) fn select_previous(&mut self) {
        if self.runs.runs.is_empty() {
            return;
        }
        self.selected_idx = self.selected_idx.saturating_sub(1);
    }

    pub(super) fn select_next(&mut self) {
        if self.runs.runs.is_empty() {
            return;
        }
        self.selected_idx = (self.selected_idx + 1).min(self.runs.runs.len() - 1);
    }
}

impl HistoryScreen {
    pub(super) fn runs(&self) -> &[RunHistorySummaryData] {
        self.result.as_ref().map_or(&[], |history| &history.runs)
    }

    pub(super) fn run_count(&self) -> usize {
        self.runs().len()
    }

    pub(super) fn selected_run(&self) -> Option<&RunHistorySummaryData> {
        self.runs()
            .get(self.selected_idx)
            .or_else(|| self.runs().first())
    }

    pub(super) fn select_previous(&mut self) {
        if self.runs().is_empty() {
            return;
        }
        self.selected_idx = self.selected_idx.saturating_sub(1);
    }

    pub(super) fn select_next(&mut self) {
        if self.runs().is_empty() {
            return;
        }
        self.selected_idx = (self.selected_idx + 1).min(self.runs().len() - 1);
    }
}
