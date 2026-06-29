use std::path::PathBuf;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;
use runhaven_core::runtime::active::DEFAULT_LOG_SNAPSHOT_LINES;
use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::NetworkMode;
use runhaven_core::ui_contracts::ActiveRunListData;
use runhaven_core::ui_contracts::ActiveRunLogSnapshotData;
use runhaven_core::ui_contracts::ActiveRunSummaryData;
use runhaven_core::ui_contracts::AuthDecisionData;
use runhaven_core::ui_contracts::EgressDecisionData;
use runhaven_core::ui_contracts::LaunchPlanData;
use runhaven_core::ui_contracts::RunHavenDiagnosticsData;

use super::launch_wizard::LaunchWizardView;
use super::service::LaunchPolicySelection;
use super::service::NetworkPolicySelection;
use super::service::PreparedLaunch;
use super::service::RunHavenTuiService;
use crate::key_hint;
use crate::render::renderable::Renderable;
use crate::style::accent_style;
use crate::style::boundary_style;
use crate::style::danger_style;
use crate::style::muted_but_readable_style;
use crate::style::safe_style;
use crate::style::selected_row_style;
use crate::style::warning_style;
use crate::tui::app_event_sender::AppEventSender;
use crate::tui::bottom_pane::BottomPaneView;
use crate::tui::bottom_pane::CancellationEvent;
use crate::tui::bottom_pane::ViewCompletion;
use crate::tui::bottom_pane::render_menu_surface;

pub(crate) const RUNHAVEN_MVP_VIEW_ID: &str = "runhaven.mvp";
const LOG_CONFIRM_PHRASE: &str = "logs";
const DIAGNOSTICS_LIMIT: usize = 20;

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
            MvpScreen::ActiveRuns(_) | MvpScreen::RunLogs(_) => self.show_diagnostics(),
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
            KeyCode::Char('r') => self.show_diagnostics(),
            _ => {}
        }
    }

    fn handle_post_run_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('1') => self.show_launch(),
            KeyCode::Char('2') => self.show_active_runs(),
            KeyCode::Tab | KeyCode::Char('3') => self.show_diagnostics(),
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
                Span::styled("1 launch 2 runs 3 diagnostics", muted_but_readable_style()),
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
                Span::styled("1 launch 2 runs r refresh", muted_but_readable_style()),
            ]),
            MvpScreen::PostRun(outcome) => Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(" · run finished · "),
                Span::styled(
                    format!("exit {}", outcome.exit_code),
                    exit_style(outcome.exit_code, outcome.error.as_ref()),
                ),
            ]),
        }
    }
}

enum ActiveRunAction {
    None,
    Launch,
    Diagnostics,
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
                    ActiveRunAction::Refresh => self.show_active_runs(),
                    ActiveRunAction::OpenLogs => self.show_logs_for_selected_run(),
                }
            }
            MvpScreen::RunLogs(_) => self.handle_logs_key(key_event),
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
            ],
            MvpScreen::ActiveRuns(_) => vec![
                ("up/down".to_string(), "choose".to_string()),
                ("enter".to_string(), "logs".to_string()),
                ("r".to_string(), "refresh".to_string()),
                ("1".to_string(), "launch".to_string()),
                ("3".to_string(), "diagnostics".to_string()),
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
            ],
            MvpScreen::PostRun(_) => vec![
                ("enter".to_string(), "new launch".to_string()),
                ("2".to_string(), "runs".to_string()),
                ("3".to_string(), "diagnostics".to_string()),
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
        match &self.screen {
            MvpScreen::Launch => self.launch.render(area, buf),
            MvpScreen::ActiveRuns(screen) => render_active_runs(screen, area, buf),
            MvpScreen::RunLogs(screen) => render_run_logs(screen, area, buf),
            MvpScreen::Diagnostics(screen) => render_diagnostics(screen, area, buf),
            MvpScreen::PostRun(outcome) => render_post_run(outcome, area, buf),
        }
    }

    fn desired_height(&self, width: u16) -> u16 {
        match &self.screen {
            MvpScreen::Launch => self.launch.desired_height(width),
            MvpScreen::ActiveRuns(screen) => {
                paragraph(active_runs_lines(screen)).line_count(width.saturating_sub(4).max(1))
                    as u16
                    + 2
            }
            MvpScreen::RunLogs(screen) => {
                paragraph(run_logs_lines(screen)).line_count(width.saturating_sub(4).max(1)) as u16
                    + 2
            }
            MvpScreen::Diagnostics(screen) => {
                paragraph(diagnostics_lines(screen)).line_count(width.saturating_sub(4).max(1))
                    as u16
                    + 2
            }
            MvpScreen::PostRun(outcome) => {
                paragraph(post_run_lines(outcome)).line_count(width.saturating_sub(4).max(1)) as u16
                    + 2
            }
        }
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

fn render_active_runs(screen: &ActiveRunsScreen, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, active_runs_lines(screen));
}

fn active_runs_lines(screen: &ActiveRunsScreen) -> Vec<Line<'static>> {
    let mut lines = vec![header_line("Active runs"), tab_line(), Line::from("")];
    if screen.runs.runs.is_empty() {
        lines.push(Line::from("No active RunHaven runs found."));
        lines.push(Line::from(
            "Launch an agent or refresh after another terminal starts one.",
        ));
        return lines;
    }
    lines.push(Line::from(
        "Enter opens a raw log confirmation for the selected run.",
    ));
    lines.push(Line::from(""));
    for (idx, run) in screen.runs.runs.iter().enumerate() {
        let selected = idx == screen.selected_idx;
        lines.push(Line::from(vec![
            Span::styled(if selected { "> " } else { "  " }, accent_style()),
            Span::styled(
                run.run_id.clone(),
                if selected {
                    selected_row_style()
                } else {
                    boundary_style()
                },
            ),
            Span::raw("  "),
            Span::styled(run.profile.clone(), accent_style()),
            Span::raw("  "),
            Span::styled(run.network.clone(), network_style(&run.network)),
            Span::raw("  "),
            Span::styled(run.status.clone(), status_style(&run.status)),
        ]));
    }
    if let Some(run) = screen.selected_run() {
        lines.push(Line::from(""));
        lines.push(label_value("Profile", run.profile.clone(), accent_style()));
        lines.push(label_value("Run ID", run.run_id.clone(), boundary_style()));
        lines.push(label_value(
            "Container",
            run.container_name.clone(),
            muted_but_readable_style(),
        ));
        lines.push(label_value("State", run.state_volume.clone(), safe_style()));
        lines.push(label_value(
            "Session",
            run.session.clone(),
            muted_but_readable_style(),
        ));
        lines.push(label_value(
            "Started",
            run.timestamp.clone(),
            muted_but_readable_style(),
        ));
    }
    if let Some(notice) = &screen.notice {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(notice.clone(), warning_style())));
    }
    lines
}

fn render_run_logs(screen: &RunLogsScreen, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, run_logs_lines(screen));
}

fn run_logs_lines(screen: &RunLogsScreen) -> Vec<Line<'static>> {
    let mut lines = vec![
        header_line("Run logs"),
        Line::from(vec![
            Span::styled("Run ID  ", muted_but_readable_style()),
            Span::styled(screen.run.run_id.clone(), boundary_style()),
            Span::raw("  "),
            Span::styled("Profile  ", muted_but_readable_style()),
            Span::styled(screen.run.profile.clone(), accent_style()),
        ]),
        Line::from(""),
    ];

    match &screen.state {
        RunLogsState::Confirm { typed, notice } => {
            lines.push(Line::from(vec![Span::styled(
                "Raw container output can contain secrets or workspace content.",
                warning_style(),
            )]));
            lines.push(Line::from(vec![
                Span::raw("Type "),
                Span::styled(LOG_CONFIRM_PHRASE, selected_row_style()),
                Span::raw(", then press Enter to load a bounded snapshot."),
            ]));
            lines.push(label_value(
                "Typed",
                typed.clone(),
                muted_but_readable_style(),
            ));
            if let Some(notice) = notice {
                lines.push(Line::from(Span::styled(notice.clone(), warning_style())));
            }
        }
        RunLogsState::Loaded(snapshot) => {
            lines.push(label_value(
                "Captured",
                snapshot.captured_at.clone(),
                muted_but_readable_style(),
            ));
            lines.push(label_value(
                "Returned",
                format!("{} lines", snapshot.returned_lines),
                safe_style(),
            ));
            if snapshot.truncated {
                lines.push(Line::from(Span::styled(
                    "Output was truncated to the bounded snapshot size.",
                    warning_style(),
                )));
            }
            for warning in &snapshot.warnings {
                lines.push(Line::from(Span::styled(warning.clone(), warning_style())));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Snapshot",
                selected_row_style(),
            )]));
            if snapshot.text.is_empty() {
                lines.push(Line::from("(no output returned)"));
            } else {
                for line in snapshot.text.lines().take(120) {
                    lines.push(Line::from(line.to_string()));
                }
            }
        }
        RunLogsState::Error(message) => {
            lines.push(Line::from(Span::styled(
                "Could not load run logs.",
                danger_style(),
            )));
            lines.push(Line::from(message.clone()));
        }
    }
    lines
}

fn render_diagnostics(screen: &DiagnosticsScreen, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, diagnostics_lines(screen));
}

fn diagnostics_lines(screen: &DiagnosticsScreen) -> Vec<Line<'static>> {
    let mut lines = vec![
        header_line("Diagnostics"),
        tab_line(),
        Line::from("Secret-free status and recent broker/proxy decisions."),
        Line::from(""),
    ];
    match &screen.result {
        Ok(data) => append_diagnostics_data(&mut lines, data),
        Err(error) => {
            lines.push(Line::from(Span::styled(
                "Could not load diagnostics.",
                danger_style(),
            )));
            lines.push(Line::from(error.clone()));
        }
    }
    lines
}

fn append_diagnostics_data(lines: &mut Vec<Line<'static>>, data: &RunHavenDiagnosticsData) {
    lines.push(label_value(
        "Auth",
        data.auth_status.status.clone(),
        safe_style(),
    ));
    lines.push(label_value(
        "Runtime",
        data.auth_status.runtime.clone(),
        muted_but_readable_style(),
    ));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Auth profiles",
        selected_row_style(),
    )]));
    for profile in data.auth_status.profiles.iter().take(8) {
        lines.push(Line::from(format!("{}  {}", profile.name, profile.status)));
    }
    append_egress_lines(lines, &data.egress_log);
    append_auth_lines(lines, &data.auth_log);
}

fn append_egress_lines(lines: &mut Vec<Line<'static>>, entries: &[EgressDecisionData]) {
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Provider egress",
        selected_row_style(),
    )]));
    if entries.is_empty() {
        lines.push(Line::from("No provider egress decisions found."));
        return;
    }
    for entry in entries.iter().take(8) {
        lines.push(Line::from(format!(
            "{}  {}  {}:{}  count={}  {}",
            entry.profile, entry.decision, entry.host, entry.port, entry.count, entry.reason
        )));
    }
}

fn append_auth_lines(lines: &mut Vec<Line<'static>>, entries: &[AuthDecisionData]) {
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Auth broker",
        selected_row_style(),
    )]));
    if entries.is_empty() {
        lines.push(Line::from("No auth broker decisions found."));
        return;
    }
    for entry in entries.iter().take(8) {
        let status = entry
            .upstream_status
            .map(|status| status.to_string())
            .unwrap_or_else(|| "-".to_string());
        lines.push(Line::from(format!(
            "{}  {}  {} {}  upstream={}  count={}",
            entry.profile, entry.decision, entry.method, entry.path, status, entry.count
        )));
    }
}

fn render_post_run(outcome: &PostRunOutcome, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, post_run_lines(outcome));
}

fn post_run_lines(outcome: &PostRunOutcome) -> Vec<Line<'static>> {
    let mut lines = vec![
        header_line("Run finished"),
        Line::from("The terminal is back in RunHaven."),
        Line::from(""),
        label_value("Agent", outcome.plan.profile_name.clone(), accent_style()),
        label_value(
            "Exit",
            outcome.exit_code.to_string(),
            exit_style(outcome.exit_code, outcome.error.as_ref()),
        ),
        label_value(
            "Network",
            outcome.plan.network.mode.clone(),
            network_style(&outcome.plan.network.mode),
        ),
        label_value("Auth scope", outcome.plan.auth_scope.clone(), safe_style()),
        label_value("Boundary", "/workspace only", boundary_style()),
        label_value("State", outcome.plan.state_volume.clone(), safe_style()),
    ];
    if let Some(error) = &outcome.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Launch error", danger_style())));
        lines.push(Line::from(error.clone()));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Enter starts another launch. 2 shows active runs. 3 shows diagnostics. q quits.",
    ));
    lines
}

fn render_panel(area: Rect, buf: &mut Buffer, lines: Vec<Line<'static>>) {
    Clear.render(area, buf);
    let content = render_menu_surface(area, buf);
    paragraph(lines).render(content, buf);
}

fn paragraph(lines: Vec<Line<'static>>) -> Paragraph<'static> {
    Paragraph::new(lines).wrap(Wrap { trim: false })
}

fn header_line(title: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled("RunHaven", selected_row_style()),
        Span::raw(format!(" v{}  ", env!("CARGO_PKG_VERSION"))),
        Span::styled(title, boundary_style()),
    ])
}

fn tab_line() -> Line<'static> {
    Line::from(vec![
        key_hint::plain(KeyCode::Char('1')).into(),
        Span::raw(" Launch  "),
        key_hint::plain(KeyCode::Char('2')).into(),
        Span::raw(" Runs  "),
        key_hint::plain(KeyCode::Char('3')).into(),
        Span::raw(" Diagnostics"),
    ])
}

fn label_value(label: &'static str, value: impl Into<String>, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<12}"), muted_but_readable_style()),
        Span::styled(value.into(), value_style),
    ])
}

fn network_style(network: &str) -> Style {
    if network == "internet" {
        warning_style()
    } else {
        safe_style()
    }
}

fn status_style(status: &str) -> Style {
    match status {
        "running" => safe_style(),
        "stop-requested" | "kill-requested" => warning_style(),
        _ => muted_but_readable_style(),
    }
}

fn exit_style(exit_code: i32, error: Option<&String>) -> Style {
    if exit_code == 0 && error.is_none() {
        safe_style()
    } else {
        warning_style()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use runhaven_core::runtime::active::write_active_run_payload;
    use runhaven_core::support::paths::{
        auth_broker_log_path, egress_policy_log_path, ensure_private_parent,
        override_cache_root_for_tests,
    };
    use std::io::Write;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn render_to_text(view: &mut RunHavenMvpView, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                let area = frame.area();
                view.render(area, frame.buffer_mut());
            })
            .expect("draw");
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("")
    }

    fn line_text(line: Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn policy_keys_rebuild_launch_plan_without_custom_shell_code() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());

        let output = render_to_text(&mut view, 120, 32);
        assert!(output.contains("Choose an agent"));
        assert!(output.contains("RunHaven will show the full plan before launch"));
        assert!(line_text(view.footer_status()).contains("auth agent"));

        view.handle_key_event(key(KeyCode::Char('a')));
        render_to_text(&mut view, 120, 32);
        assert!(line_text(view.footer_status()).contains("auth project"));

        view.handle_key_event(key(KeyCode::Char('n')));
        let output = render_to_text(&mut view, 120, 32);
        assert!(output.contains("local only"));
        assert!(line_text(view.footer_status()).contains("network internal"));
    }

    #[test]
    fn active_runs_view_omits_workspace_paths_and_opens_log_confirmation() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = override_cache_root_for_tests(cache.path());
        write_active_run_payload(
            "run-123",
            serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "run_id": "run-123",
                "profile": "codex",
                "workspace": "/Users/c/secret/project",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-codex-project-run",
                "state_volume": "runhaven-codex-shared-home",
                "session": "none"
            }),
        )
        .expect("active marker");
        let workspace = tempfile::tempdir().expect("workspace");
        let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());

        view.handle_key_event(key(KeyCode::Char('2')));
        let output = render_to_text(&mut view, 120, 32);
        assert!(output.contains("Active runs"));
        assert!(output.contains("run-123"));
        assert!(!output.contains("/Users/c/secret/project"));

        view.handle_key_event(key(KeyCode::Enter));
        let output = render_to_text(&mut view, 120, 32);
        assert!(output.contains("Raw container output can contain secrets"));
        assert!(output.contains("Type logs"));
    }

    #[test]
    fn log_confirmation_rejects_paste_and_wrong_phrase() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());
        view.screen = MvpScreen::RunLogs(Box::new(RunLogsScreen {
            run: ActiveRunSummaryData {
                run_id: "run-123".to_string(),
                profile: "codex".to_string(),
                network: "provider".to_string(),
                status: "running".to_string(),
                timestamp: "2026-06-29T00:00:00Z".to_string(),
                state_volume: "runhaven-codex-shared-home".to_string(),
                session: "none".to_string(),
                container_name: "runhaven-codex-project-run".to_string(),
            },
            state: RunLogsState::Confirm {
                typed: String::new(),
                notice: None,
            },
        }));

        assert!(view.handle_paste("logs".to_string()));
        let output = render_to_text(&mut view, 120, 32);
        assert!(output.contains("Paste is ignored"));

        for ch in "nope".chars() {
            view.handle_key_event(key(KeyCode::Char(ch)));
        }
        view.handle_key_event(key(KeyCode::Enter));
        let output = render_to_text(&mut view, 120, 32);
        assert!(output.contains("Type logs before loading logs."));
    }

    #[test]
    fn loaded_log_snapshot_is_visible_only_after_confirm_state() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());
        view.screen = MvpScreen::RunLogs(Box::new(RunLogsScreen {
            run: ActiveRunSummaryData {
                run_id: "run-123".to_string(),
                profile: "codex".to_string(),
                network: "provider".to_string(),
                status: "running".to_string(),
                timestamp: "2026-06-29T00:00:00Z".to_string(),
                state_volume: "runhaven-codex-shared-home".to_string(),
                session: "none".to_string(),
                container_name: "runhaven-codex-project-run".to_string(),
            },
            state: RunLogsState::Loaded(ActiveRunLogSnapshotData {
                run_id: "run-123".to_string(),
                captured_at: "2026-06-29T00:00:00Z".to_string(),
                requested_lines: 20,
                text: "agent output\n".to_string(),
                returned_lines: 1,
                truncated: false,
                source: "container-stdio".to_string(),
                warnings: vec!["Raw container output can contain secrets.".to_string()],
            }),
        }));

        let output = render_to_text(&mut view, 120, 32);
        assert!(output.contains("Snapshot"));
        assert!(output.contains("agent output"));
        assert!(output.contains("Raw container output can contain secrets."));
    }

    #[test]
    fn diagnostics_view_omits_secret_and_workspace_fields() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = override_cache_root_for_tests(cache.path());
        ensure_private_parent(&egress_policy_log_path()).expect("egress parent");
        ensure_private_parent(&auth_broker_log_path()).expect("auth parent");
        {
            let mut file =
                std::fs::File::create(egress_policy_log_path()).expect("egress log file");
            writeln!(
                file,
                "{}",
                serde_json::json!({
                    "timestamp": "2026-06-29T00:00:00Z",
                    "profile": "codex",
                    "decision": "denied",
                    "host": "example.com",
                    "port": 443,
                    "count": 1,
                    "reason": "not-in-allowlist",
                    "matched_rule": "",
                    "run_id": "run-123",
                    "workspace": "/Users/c/secret/project"
                })
            )
            .expect("egress write");
        }
        {
            let mut file = std::fs::File::create(auth_broker_log_path()).expect("auth log file");
            writeln!(
                file,
                "{}",
                serde_json::json!({
                    "timestamp": "2026-06-29T00:00:00Z",
                    "profile": "codex",
                    "broker": "api-key",
                    "decision": "allowed",
                    "method": "POST",
                    "path": "/v1/responses?token=secret#fragment",
                    "upstream_status": 200,
                    "count": 1,
                    "reason": "-",
                    "run_id": "run-123",
                    "authorization": "Bearer secret"
                })
            )
            .expect("auth write");
        }
        let workspace = tempfile::tempdir().expect("workspace");
        let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());

        view.handle_key_event(key(KeyCode::Char('3')));
        let output = render_to_text(&mut view, 120, 40);

        assert!(output.contains("Diagnostics"));
        assert!(output.contains("example.com"));
        assert!(output.contains("/v1/responses"));
        assert!(!output.contains("token=secret"));
        assert!(!output.contains("/Users/c/secret/project"));
        assert!(!output.contains("Bearer secret"));
    }

    #[test]
    fn post_run_recovery_screen_can_return_to_launch() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());
        let launch = view
            .service
            .launch_preview_payload(workspace.path())
            .previews
            .into_iter()
            .find(|preview| preview.agent.name == "codex")
            .expect("codex preview")
            .plan
            .expect("codex plan");

        view.show_post_run(PostRunOutcome::from_launch(&launch, 0, None));
        let output = render_to_text(&mut view, 120, 32);
        assert!(output.contains("Run finished"));
        assert!(output.contains("The terminal is back in RunHaven."));

        view.handle_key_event(key(KeyCode::Enter));
        let output = render_to_text(&mut view, 120, 32);
        assert!(output.contains("Choose agent"));
    }

    #[test]
    fn post_run_recovery_preserves_effective_workspace_and_policy() {
        let root = tempfile::tempdir().expect("root");
        let nested = root.path().join("nested");
        std::fs::create_dir(&nested).expect("nested workspace");
        let mut view = RunHavenMvpView::new(nested.clone());
        let policy = LaunchPolicySelection {
            network: NetworkPolicySelection::Fixed(NetworkMode::Internet),
            auth_scope: AuthScope::Project,
        };
        let launch = view
            .service
            .launch_preview_payload_with_policy(root.path(), policy)
            .previews
            .into_iter()
            .find(|preview| preview.agent.name == "codex")
            .expect("codex preview")
            .plan
            .expect("codex plan");

        view.show_post_run(PostRunOutcome::from_launch(&launch, 0, None));
        assert_eq!(view.workspace, launch.executable.workspace);
        assert_eq!(view.policy, policy);

        view.handle_key_event(key(KeyCode::Enter));
        assert_eq!(view.workspace, launch.executable.workspace);
        assert_eq!(view.policy, policy);
    }
}
