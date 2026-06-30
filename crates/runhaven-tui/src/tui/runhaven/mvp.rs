use std::path::PathBuf;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::text::Span;
use runhaven_core::runtime::active::DEFAULT_LOG_SNAPSHOT_LINES;
use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::NetworkMode;
use runhaven_core::ui_contracts::ActiveRunListData;
use runhaven_core::ui_contracts::ActiveRunLogSnapshotData;
use runhaven_core::ui_contracts::ActiveRunSummaryData;
use runhaven_core::ui_contracts::LaunchPlanData;
use runhaven_core::ui_contracts::RunHavenDiagnosticsData;
use runhaven_core::ui_contracts::RunHistoryListData;
use runhaven_core::ui_contracts::RunHistorySummaryData;

use super::launch_wizard::LaunchWizardView;
use super::service::LaunchPolicySelection;
use super::service::NetworkPolicySelection;
use super::service::PreparedLaunch;
use super::service::RunHavenTuiService;
use crate::render::renderable::Renderable;
use crate::style::boundary_style;
use crate::style::muted_but_readable_style;
use crate::style::safe_style;
use crate::style::selected_row_style;
use crate::style::warning_style;
use crate::tui::app_event_sender::AppEventSender;
use crate::tui::bottom_pane::BottomPaneView;
use crate::tui::bottom_pane::CancellationEvent;
use crate::tui::bottom_pane::ViewCompletion;

pub(crate) const RUNHAVEN_MVP_VIEW_ID: &str = "runhaven.mvp";
const LOG_CONFIRM_PHRASE: &str = "logs";
const DIAGNOSTICS_LIMIT: usize = 20;
const HISTORY_LIMIT: usize = 10;

#[path = "mvp_render.rs"]
mod mvp_render;

pub(crate) struct RunHavenMvpView {
    workspace: PathBuf,
    service: RunHavenTuiService,
    policy: LaunchPolicySelection,
    launch: LaunchWizardView,
    screen: MvpScreen,
    app_event_tx: Option<AppEventSender>,
    completion: Option<ViewCompletion>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostRunOutcome {
    pub(crate) plan: LaunchPlanData,
    pub(crate) workspace: PathBuf,
    pub(crate) policy: LaunchPolicySelection,
    pub(crate) exit_code: i32,
    pub(crate) error: Option<String>,
}

impl PostRunOutcome {
    pub(crate) fn from_launch(
        launch: &PreparedLaunch,
        exit_code: i32,
        error: Option<String>,
    ) -> Self {
        Self {
            plan: launch.data.clone(),
            workspace: launch.executable.workspace.clone(),
            policy: launch.policy,
            exit_code,
            error,
        }
    }
}

enum MvpScreen {
    Launch,
    ActiveRuns(Box<ActiveRunsScreen>),
    RunLogs(Box<RunLogsScreen>),
    History(Box<HistoryScreen>),
    Diagnostics(Box<DiagnosticsScreen>),
    PostRun(Box<PostRunOutcome>),
}

#[derive(Clone)]
struct ActiveRunsScreen {
    runs: ActiveRunListData,
    selected_idx: usize,
    notice: Option<String>,
}

#[derive(Clone)]
struct RunLogsScreen {
    run: ActiveRunSummaryData,
    state: RunLogsState,
}

#[derive(Clone)]
enum RunLogsState {
    Confirm {
        typed: String,
        notice: Option<String>,
    },
    Loaded(ActiveRunLogSnapshotData),
    Error(String),
}

#[derive(Clone)]
struct DiagnosticsScreen {
    result: Result<RunHavenDiagnosticsData, String>,
}

#[derive(Clone)]
struct HistoryScreen {
    result: Result<RunHistoryListData, String>,
    selected_idx: usize,
}

impl RunHavenMvpView {
    pub(crate) fn new(workspace: PathBuf) -> Self {
        Self::with_service(workspace, RunHavenTuiService::new())
    }

    fn with_service(workspace: PathBuf, service: RunHavenTuiService) -> Self {
        let policy = LaunchPolicySelection::default();
        let launch = launch_wizard_for(&service, &workspace, policy, None);
        Self {
            workspace,
            service,
            policy,
            launch,
            screen: MvpScreen::Launch,
            app_event_tx: None,
            completion: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_launch_wizard_for_tests(
        workspace: PathBuf,
        launch: LaunchWizardView,
    ) -> Self {
        Self {
            workspace,
            service: RunHavenTuiService::new(),
            policy: LaunchPolicySelection::default(),
            launch,
            screen: MvpScreen::Launch,
            app_event_tx: None,
            completion: None,
        }
    }

    pub(crate) fn set_app_event_sender(&mut self, app_event_tx: AppEventSender) {
        self.app_event_tx = Some(app_event_tx.clone());
        self.launch.set_app_event_sender(app_event_tx);
    }

    pub(crate) fn show_post_run(&mut self, outcome: PostRunOutcome) {
        self.workspace = outcome.workspace.clone();
        self.policy = outcome.policy;
        self.screen = MvpScreen::PostRun(Box::new(outcome));
    }

    fn rebuild_launch(&mut self) {
        self.launch = launch_wizard_for(
            &self.service,
            &self.workspace,
            self.policy,
            self.app_event_tx.clone(),
        );
    }

    fn show_launch(&mut self) {
        self.screen = MvpScreen::Launch;
        self.rebuild_launch();
    }

    fn show_active_runs(&mut self) {
        self.screen = MvpScreen::ActiveRuns(Box::new(ActiveRunsScreen {
            runs: self.service.active_runs_payload(),
            selected_idx: 0,
            notice: None,
        }));
    }

    fn show_diagnostics(&mut self) {
        self.screen = MvpScreen::Diagnostics(Box::new(DiagnosticsScreen {
            result: self
                .service
                .diagnostics_payload(DIAGNOSTICS_LIMIT)
                .map_err(|error| error.to_string()),
        }));
    }

    fn show_history(&mut self) {
        self.screen = MvpScreen::History(Box::new(HistoryScreen {
            result: self
                .service
                .run_history_payload(HISTORY_LIMIT)
                .map_err(|error| error.to_string()),
            selected_idx: 0,
        }));
    }

    fn show_logs_for_selected_run(&mut self) {
        let Some(run) = self
            .active_runs_screen()
            .and_then(ActiveRunsScreen::selected_run)
            .cloned()
        else {
            return;
        };
        self.screen = MvpScreen::RunLogs(Box::new(RunLogsScreen {
            run,
            state: RunLogsState::Confirm {
                typed: String::new(),
                notice: None,
            },
        }));
    }

    fn active_runs_screen(&self) -> Option<&ActiveRunsScreen> {
        match &self.screen {
            MvpScreen::ActiveRuns(screen) => Some(screen),
            _ => None,
        }
    }

    fn cycle_section(&mut self) {
        match self.screen {
            MvpScreen::Launch => self.show_active_runs(),
            MvpScreen::ActiveRuns(_) | MvpScreen::RunLogs(_) => self.show_history(),
            MvpScreen::History(_) => self.show_diagnostics(),
            MvpScreen::Diagnostics(_) | MvpScreen::PostRun(_) => self.show_launch(),
        }
    }

    fn cycle_network_policy(&mut self) {
        self.policy.network = match self.policy.network {
            NetworkPolicySelection::Default => NetworkPolicySelection::Fixed(NetworkMode::Internal),
            NetworkPolicySelection::Fixed(NetworkMode::Internal) => {
                NetworkPolicySelection::Fixed(NetworkMode::Internet)
            }
            NetworkPolicySelection::Fixed(NetworkMode::Internet) => {
                NetworkPolicySelection::Fixed(NetworkMode::Provider)
            }
            NetworkPolicySelection::Fixed(NetworkMode::Provider) => NetworkPolicySelection::Default,
        };
        self.rebuild_launch();
    }

    fn toggle_auth_scope(&mut self) {
        self.policy.auth_scope = match self.policy.auth_scope {
            AuthScope::Agent => AuthScope::Project,
            AuthScope::Project => AuthScope::Agent,
        };
        self.rebuild_launch();
    }

    fn handle_launch_key(&mut self, key_event: KeyEvent) {
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

    fn handle_active_runs_key(
        screen: &mut ActiveRunsScreen,
        key_event: KeyEvent,
    ) -> ActiveRunAction {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('1') => ActiveRunAction::Launch,
            KeyCode::Tab | KeyCode::Char('3') => ActiveRunAction::Diagnostics,
            KeyCode::Char('4') | KeyCode::Char('h') => ActiveRunAction::History,
            KeyCode::Char('r') => ActiveRunAction::Refresh,
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

    fn handle_logs_key(&mut self, key_event: KeyEvent) {
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

    fn handle_diagnostics_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('1') => self.show_launch(),
            KeyCode::Tab | KeyCode::Char('2') => self.show_active_runs(),
            KeyCode::Char('4') | KeyCode::Char('h') => self.show_history(),
            KeyCode::Char('r') => self.show_diagnostics(),
            _ => {}
        }
    }

    fn handle_history_key(&mut self, key_event: KeyEvent) {
        let MvpScreen::History(screen) = &mut self.screen else {
            return;
        };
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('1') => self.show_launch(),
            KeyCode::Char('2') => self.show_active_runs(),
            KeyCode::Tab | KeyCode::Char('3') => self.show_diagnostics(),
            KeyCode::Char('r') => self.show_history(),
            KeyCode::Up | KeyCode::Char('k') => screen.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => screen.select_next(),
            _ => {}
        }
    }

    fn handle_post_run_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('1') => self.show_launch(),
            KeyCode::Char('2') => self.show_active_runs(),
            KeyCode::Tab | KeyCode::Char('3') => self.show_diagnostics(),
            KeyCode::Char('4') | KeyCode::Char('h') => self.show_history(),
            _ => {}
        }
    }

    fn footer_status(&self) -> Line<'static> {
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
                Span::styled("1 launch 2 runs 3 diagnostics", muted_but_readable_style()),
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

enum ActiveRunAction {
    None,
    Launch,
    Diagnostics,
    History,
    Refresh,
    OpenLogs,
}

impl ActiveRunsScreen {
    fn selected_run(&self) -> Option<&ActiveRunSummaryData> {
        self.runs
            .runs
            .get(self.selected_idx)
            .or_else(|| self.runs.runs.first())
    }

    fn select_previous(&mut self) {
        if self.runs.runs.is_empty() {
            return;
        }
        self.selected_idx = self.selected_idx.saturating_sub(1);
    }

    fn select_next(&mut self) {
        if self.runs.runs.is_empty() {
            return;
        }
        self.selected_idx = (self.selected_idx + 1).min(self.runs.runs.len() - 1);
    }
}

impl HistoryScreen {
    fn runs(&self) -> &[RunHistorySummaryData] {
        self.result.as_ref().map_or(&[], |history| &history.runs)
    }

    fn run_count(&self) -> usize {
        self.runs().len()
    }

    fn selected_run(&self) -> Option<&RunHistorySummaryData> {
        self.runs()
            .get(self.selected_idx)
            .or_else(|| self.runs().first())
    }

    fn select_previous(&mut self) {
        if self.runs().is_empty() {
            return;
        }
        self.selected_idx = self.selected_idx.saturating_sub(1);
    }

    fn select_next(&mut self) {
        if self.runs().is_empty() {
            return;
        }
        self.selected_idx = (self.selected_idx + 1).min(self.runs().len() - 1);
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
                }
            }
            MvpScreen::RunLogs(_) => self.handle_logs_key(key_event),
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
                ("1".to_string(), "launch".to_string()),
                ("3".to_string(), "diagnostics".to_string()),
                ("h".to_string(), "history".to_string()),
            ],
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
            MvpScreen::Diagnostics(_) => vec![
                ("r".to_string(), "refresh".to_string()),
                ("1".to_string(), "launch".to_string()),
                ("2".to_string(), "runs".to_string()),
                ("h".to_string(), "history".to_string()),
            ],
            MvpScreen::History(_) => vec![
                ("up/down".to_string(), "choose".to_string()),
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
            _ => false,
        }
    }
}

impl Renderable for RunHavenMvpView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        mvp_render::render(self, area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        mvp_render::desired_height(self, width)
    }

    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        match &self.screen {
            MvpScreen::Launch => self.launch.cursor_pos(area),
            _ => None,
        }
    }
}

fn launch_wizard_for(
    service: &RunHavenTuiService,
    workspace: &PathBuf,
    policy: LaunchPolicySelection,
    app_event_tx: Option<AppEventSender>,
) -> LaunchWizardView {
    let choices = if policy == LaunchPolicySelection::default() {
        service.launch_workspace_choices(workspace)
    } else {
        service.launch_workspace_choices_with_policy(workspace, policy)
    };
    let mut launch = LaunchWizardView::new_with_workspace_choices(choices);
    if let Some(app_event_tx) = app_event_tx {
        launch.set_app_event_sender(app_event_tx);
    }
    launch
}

#[cfg(test)]
#[path = "mvp_snapshots.rs"]
mod snapshot_tests;

#[cfg(test)]
#[path = "mvp_tests.rs"]
mod tests;
