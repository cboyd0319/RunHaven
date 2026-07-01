use std::path::PathBuf;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::NetworkMode;
use runhaven_core::ui_contracts::ActiveRunListData;
use runhaven_core::ui_contracts::ActiveRunLogSnapshotData;
use runhaven_core::ui_contracts::ActiveRunSummaryData;
use runhaven_core::ui_contracts::LaunchPlanData;
use runhaven_core::ui_contracts::RunControlResultData;
use runhaven_core::ui_contracts::RunDiffData;
use runhaven_core::ui_contracts::RunHavenDiagnosticsData;
use runhaven_core::ui_contracts::RunHistoryListData;
use runhaven_core::ui_contracts::RunHistorySummaryData;

use super::launch_wizard::LaunchWizardView;
use super::service::LaunchPolicySelection;
use super::service::NetworkPolicySelection;
use super::service::PreparedLaunch;
use super::service::RunHavenTuiService;
use crate::render::renderable::Renderable;
use crate::tui::app_event_sender::AppEventSender;
use crate::tui::bottom_pane::ViewCompletion;

pub(crate) const RUNHAVEN_MVP_VIEW_ID: &str = "runhaven.mvp";
const LOG_CONFIRM_PHRASE: &str = "logs";
const DIFF_CONFIRM_PHRASE: &str = "diff";
const DIAGNOSTICS_LIMIT: usize = 20;
const HISTORY_LIMIT: usize = 10;

#[path = "mvp_bottom_pane.rs"]
mod mvp_bottom_pane;
#[path = "mvp_input.rs"]
mod mvp_input;
#[path = "mvp_render.rs"]
mod mvp_render;

use mvp_input::ActiveRunAction;

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
    RunDiff(Box<RunDiffScreen>),
    RunControl(Box<RunControlScreen>),
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
struct RunDiffScreen {
    run: RunHistorySummaryData,
    state: RunDiffState,
}

#[derive(Clone)]
enum RunDiffState {
    Confirm {
        typed: String,
        notice: Option<String>,
    },
    Loaded(RunDiffData),
    Error(String),
}

#[derive(Clone)]
struct RunControlScreen {
    run: ActiveRunSummaryData,
    action: RunControlAction,
    state: RunControlState,
}

#[derive(Clone)]
enum RunControlState {
    Confirm {
        typed: String,
        notice: Option<String>,
    },
    Complete(RunControlResultData),
    Error(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunControlAction {
    Stop,
    Kill,
    Repair,
}

impl RunControlAction {
    fn title(self) -> &'static str {
        match self {
            Self::Stop => "Stop run",
            Self::Kill => "Hard stop run",
            Self::Repair => "Repair marker",
        }
    }

    fn phrase(self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::Kill => "kill",
            Self::Repair => "repair",
        }
    }

    fn method(self) -> &'static str {
        match self {
            Self::Stop => "runhaven/run/stop",
            Self::Kill => "runhaven/run/kill",
            Self::Repair => "runhaven/run/repair",
        }
    }

    fn missing_confirmation_notice(self) -> &'static str {
        match self {
            Self::Stop => "Type stop before stopping this run.",
            Self::Kill => "Type kill before hard-stopping this run.",
            Self::Repair => "Type repair before changing this active-run marker.",
        }
    }

    fn paste_notice(self) -> String {
        format!("Type {} by hand. Paste is ignored here.", self.phrase())
    }
}

#[derive(Clone)]
struct DiagnosticsScreen {
    result: Result<RunHavenDiagnosticsData, String>,
}

#[derive(Clone)]
struct HistoryScreen {
    result: Result<RunHistoryListData, String>,
    selected_idx: usize,
    notice: Option<String>,
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
            notice: None,
        }));
    }

    fn show_diff_for_history_run(&mut self, run: RunHistorySummaryData) {
        self.screen = MvpScreen::RunDiff(Box::new(RunDiffScreen {
            run,
            state: RunDiffState::Confirm {
                typed: String::new(),
                notice: None,
            },
        }));
    }

    fn show_diff_for_history_index(&mut self, selected_idx: usize) {
        let run = match &self.screen {
            MvpScreen::History(screen) => screen.runs().get(selected_idx).cloned(),
            _ => None,
        };

        if let Some(run) = run {
            self.show_diff_for_history_run(run);
        } else if let MvpScreen::History(screen) = &mut self.screen {
            screen.notice = Some("No run records are available.".to_string());
        }
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

    fn show_run_control_for_selected_run(&mut self, action: RunControlAction) {
        let Some(run) = self
            .active_runs_screen()
            .and_then(ActiveRunsScreen::selected_run)
            .cloned()
        else {
            return;
        };
        self.screen = MvpScreen::RunControl(Box::new(RunControlScreen {
            run,
            action,
            state: RunControlState::Confirm {
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
            MvpScreen::ActiveRuns(_)
            | MvpScreen::RunLogs(_)
            | MvpScreen::RunDiff(_)
            | MvpScreen::RunControl(_) => self.show_history(),
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
