use ratatui::crossterm::event::KeyCode;

use super::{App, Screen, TuiAction, runs};
use crate::runhaven::runtime::profiles::AgentProfile;

impl App {
    /// Handle a key press. Returns `Some(action)` to leave the TUI, `None` to continue.
    pub(super) fn handle_key(&mut self, code: KeyCode) -> Option<TuiAction> {
        match self.screen {
            Screen::Home => match code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(TuiAction::Exit(0)),
                KeyCode::Down | KeyCode::Char('j') => self.select_next(),
                KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
                KeyCode::Enter | KeyCode::Char('l') => self.screen = Screen::Detail,
                KeyCode::Char('r') => self.open_plan_review(),
                KeyCode::Char('d') => self.open_run_dashboard(),
                KeyCode::Char('h') => self.open_history(),
                KeyCode::Char('g') => self.open_diagnostics(),
                KeyCode::Char('w') => {
                    self.launcher.open_workspace_picker();
                    self.screen = Screen::Workspace;
                }
                KeyCode::Char('p') => self.toggle_pet(),
                _ => {}
            },
            Screen::Detail => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Enter | KeyCode::Char('r') => self.open_plan_review(),
                KeyCode::Char('d') => self.open_run_dashboard(),
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => {
                    self.screen = Screen::Home;
                }
                _ => {}
            },
            Screen::Workspace => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc => self.screen = Screen::Home,
                KeyCode::Down | KeyCode::Char('j') => self.launcher.workspace_picker.select_next(),
                KeyCode::Up | KeyCode::Char('k') => {
                    self.launcher.workspace_picker.select_previous()
                }
                KeyCode::Backspace => self.launcher.workspace_picker.pop_query_char(),
                KeyCode::Enter => {
                    if let Err(error) = self.launcher.confirm_workspace_selection() {
                        self.launcher.plan_error = Some(error.to_string());
                    }
                    self.screen = Screen::Home;
                }
                KeyCode::Char(ch) => self.launcher.workspace_picker.push_query_char(ch),
                _ => {}
            },
            Screen::Plan => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => {
                    self.screen = Screen::Home
                }
                KeyCode::Char('w') => {
                    self.launcher.open_workspace_picker();
                    self.screen = Screen::Workspace;
                }
                KeyCode::Enter if self.launcher.review.is_some() => {
                    self.launcher.confirm_input.clear();
                    self.screen = Screen::Confirm;
                }
                _ => {}
            },
            Screen::Confirm => match code {
                KeyCode::Char('q') | KeyCode::Esc => self.screen = Screen::Plan,
                KeyCode::Backspace => {
                    self.launcher.confirm_input.pop();
                }
                KeyCode::Enter if self.launcher.confirm_ready() => {
                    let plan = self.launcher.launch_plan()?;
                    return Some(TuiAction::Launch(Box::new(plan)));
                }
                KeyCode::Char(ch) if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' => {
                    self.launcher.confirm_input.push(ch);
                }
                _ => {}
            },
            Screen::Runs => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => {
                    self.screen = Screen::Home
                }
                KeyCode::Down | KeyCode::Char('j') => self.run_manager.select_next(),
                KeyCode::Up | KeyCode::Char('k') => self.run_manager.select_previous(),
                KeyCode::Char('r') => self.run_manager.refresh_dashboard(),
                KeyCode::Char('l') | KeyCode::Enter => {
                    self.run_manager.refresh_logs();
                    self.screen = Screen::Logs;
                }
                KeyCode::Char('s') => self.begin_run_control(runs::RunControlAction::Stop),
                KeyCode::Char('x') => self.begin_run_control(runs::RunControlAction::Kill),
                KeyCode::Char('e') => self.begin_run_control(runs::RunControlAction::Repair),
                _ => {}
            },
            Screen::Logs => match code {
                KeyCode::Esc if self.run_manager.logs.search_editing => {
                    self.run_manager.logs.finish_search()
                }
                KeyCode::Enter if self.run_manager.logs.search_editing => {
                    self.run_manager.logs.finish_search()
                }
                KeyCode::Backspace if self.run_manager.logs.search_editing => {
                    self.run_manager.logs.pop_search_char()
                }
                KeyCode::Char(ch) if self.run_manager.logs.search_editing => {
                    self.run_manager.logs.push_search_char(ch)
                }
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc | KeyCode::Char('h') => self.screen = Screen::Runs,
                KeyCode::Char('r') => self.run_manager.refresh_logs(),
                KeyCode::Up | KeyCode::Char('k') => self.run_manager.logs.scroll_up(),
                KeyCode::Down | KeyCode::Char('j') => self.run_manager.logs.scroll_down(),
                KeyCode::Char('t') => self.run_manager.logs.follow_tail(),
                KeyCode::Char('/') => self.run_manager.logs.begin_search(),
                _ => {}
            },
            Screen::Control => match code {
                KeyCode::Char('q') | KeyCode::Esc => self.screen = Screen::Runs,
                KeyCode::Backspace => {
                    if let Some(dialog) = &mut self.run_manager.control {
                        dialog.input.pop();
                    }
                }
                KeyCode::Enter => {
                    if let Err(error) = self.run_manager.execute_control() {
                        self.run_manager.message = Some(error.to_string());
                    }
                    self.screen = Screen::Runs;
                }
                KeyCode::Char(ch) if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' => {
                    if let Some(dialog) = &mut self.run_manager.control {
                        dialog.input.push(ch);
                    }
                }
                _ => {}
            },
            Screen::History => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc | KeyCode::Backspace => self.screen = Screen::Home,
                KeyCode::Down | KeyCode::Char('j') => self.history.select_next(),
                KeyCode::Up | KeyCode::Char('k') => self.history.select_previous(),
                KeyCode::Char('r') => self.history.refresh_records(),
                KeyCode::Char('g') => self.open_diagnostics(),
                KeyCode::Enter | KeyCode::Char('l') => self.open_history_detail(),
                _ => {}
            },
            Screen::HistoryDetail => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => {
                    self.screen = Screen::History;
                }
                KeyCode::Char('r') => self.history.refresh_selected_diff(),
                KeyCode::Up | KeyCode::Char('k') => self.history.detail.scroll_up(),
                KeyCode::Down | KeyCode::Char('j') => self.history.detail.scroll_down(),
                _ => {}
            },
            Screen::Diagnostics => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc | KeyCode::Backspace => self.screen = Screen::Home,
                KeyCode::Char('r') => {
                    self.history
                        .refresh_diagnostics(self.settings, self.pet_image_protocol);
                }
                KeyCode::Char('d') => self.open_doctor(),
                KeyCode::Char('h') => self.open_history(),
                _ => {}
            },
            Screen::Doctor => match code {
                KeyCode::Char('q') => return Some(TuiAction::Exit(0)),
                KeyCode::Esc | KeyCode::Backspace => self.screen = Screen::Home,
                KeyCode::Char('r') => self.history.refresh_doctor(),
                KeyCode::Char('g') => self.open_diagnostics(),
                _ => {}
            },
        }
        None
    }

    pub(super) fn selected(&self) -> Option<&AgentProfile> {
        self.list.selected().and_then(|i| self.agents.get(i))
    }

    fn select_next(&mut self) {
        if self.agents.is_empty() {
            return;
        }
        let next = self.list.selected().unwrap_or(0) + 1;
        self.list.select(Some(next.min(self.agents.len() - 1)));
    }

    fn select_previous(&mut self) {
        let current = self.list.selected().unwrap_or(0);
        self.list.select(Some(current.saturating_sub(1)));
    }

    fn open_plan_review(&mut self) {
        if let Some(agent) = self.selected().cloned() {
            self.launcher.build_review(&agent);
            self.screen = Screen::Plan;
        }
    }

    fn open_run_dashboard(&mut self) {
        self.run_manager.refresh_dashboard();
        self.screen = Screen::Runs;
    }

    fn open_history(&mut self) {
        self.history.refresh_records();
        self.screen = Screen::History;
    }

    fn open_history_detail(&mut self) {
        self.history.refresh_selected_diff();
        self.screen = Screen::HistoryDetail;
    }

    fn open_diagnostics(&mut self) {
        self.history
            .refresh_diagnostics(self.settings, self.pet_image_protocol);
        self.screen = Screen::Diagnostics;
    }

    fn open_doctor(&mut self) {
        self.history.refresh_doctor();
        self.screen = Screen::Doctor;
    }

    fn begin_run_control(&mut self, action: runs::RunControlAction) {
        if let Err(error) = self.run_manager.begin_control(action) {
            self.run_manager.message = Some(error.to_string());
        } else {
            self.screen = Screen::Control;
        }
    }
}
