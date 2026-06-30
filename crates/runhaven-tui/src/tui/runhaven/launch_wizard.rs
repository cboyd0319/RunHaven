use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use super::service::AgentLaunchPreview;
use super::service::LaunchPreviewError;
#[cfg(test)]
use super::service::LaunchPreviewPayload;
use super::service::PreparedLaunch;
use super::service::WorkspaceLaunchPreview;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;
use runhaven_core::ui_contracts::AgentCatalogItemData;
use runhaven_core::ui_contracts::LaunchPlanData;

use crate::key_hint;
use crate::keymap::RuntimeKeymap;
use crate::render::renderable::Renderable;
use crate::style::accent_style;
use crate::style::boundary_style;
use crate::style::danger_style;
use crate::style::muted_but_readable_style;
use crate::style::safe_style;
use crate::style::selected_row_style;
use crate::style::warning_style;
use crate::tui::app_event::AppEvent;
use crate::tui::app_event_sender::AppEventSender;
use crate::tui::bottom_pane::BottomPaneView;
use crate::tui::bottom_pane::CancellationEvent;
use crate::tui::bottom_pane::ColumnWidthMode;
use crate::tui::bottom_pane::ListSelectionView;
use crate::tui::bottom_pane::SelectionItem;
use crate::tui::bottom_pane::SelectionRowDisplay;
use crate::tui::bottom_pane::SelectionViewParams;
use crate::tui::bottom_pane::SideContentWidth;
use crate::tui::bottom_pane::TextArea;
use crate::tui::bottom_pane::ViewCompletion;

pub(crate) struct LaunchWizardView {
    workspace_choices: Arc<Vec<WorkspaceDecisionVm>>,
    selected_workspace_idx: Arc<AtomicUsize>,
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    workspace_picker: Option<ListSelectionView>,
    picker: ListSelectionView,
    screen: LaunchWizardScreen,
    confirm_composer: TextArea,
    confirm_notice: Option<String>,
    launch_prepared: bool,
    app_event_tx: Option<AppEventSender>,
    completion: Option<ViewCompletion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkspaceDecisionVm {
    label: String,
    description: String,
    workspace: PathBuf,
    decisions: Arc<Vec<AgentDecisionVm>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentDecisionVm {
    agent: AgentCatalogItemData,
    plan: Result<PreparedLaunch, LaunchPreviewError>,
    status_label: String,
    auth_scope_label: String,
    auth_label: String,
    network_label: String,
    boundary_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchWizardScreen {
    ChooseWorkspace,
    ChooseAgent,
    ReviewPlan,
    ConfirmLaunch,
}

const CONFIRM_PHRASE: &str = "launch";
const LAUNCH_PREPARED_NOTICE: &str = "Launch prepared. Starting in the terminal.";
pub(crate) const LAUNCH_WIZARD_VIEW_ID: &str = "runhaven.launch_wizard";

#[path = "launch_wizard_render.rs"]
mod rendering;

impl LaunchWizardView {
    #[cfg(test)]
    pub(crate) fn new(workspace: PathBuf, previews: Vec<AgentLaunchPreview>) -> Self {
        Self::new_with_workspace_choices(vec![WorkspaceLaunchPreview {
            label: "Current directory".to_string(),
            description: workspace.display().to_string(),
            payload: LaunchPreviewPayload {
                workspace,
                previews,
            },
        }])
    }

    pub(crate) fn new_with_workspace_choices(
        workspace_choices: Vec<WorkspaceLaunchPreview>,
    ) -> Self {
        let workspaces = Arc::new(
            workspace_choices
                .into_iter()
                .map(WorkspaceDecisionVm::from)
                .collect::<Vec<_>>(),
        );
        let selected_workspace_idx = Arc::new(AtomicUsize::new(0));
        let decisions = selected_workspace(&workspaces, &selected_workspace_idx)
            .map(|workspace| Arc::clone(&workspace.decisions))
            .unwrap_or_default();
        let selected_idx = Arc::new(AtomicUsize::new(0));
        let workspace_picker = (workspaces.len() > 1).then(|| {
            let params = workspace_selection_params(
                Arc::clone(&workspaces),
                Arc::clone(&selected_workspace_idx),
            );
            let (app_event_tx, _app_event_rx) = tokio::sync::mpsc::unbounded_channel();
            ListSelectionView::new(
                params,
                AppEventSender::new(app_event_tx),
                RuntimeKeymap::defaults().list,
            )
        });
        let params = agent_selection_params(
            selected_workspace_display(&workspaces, &selected_workspace_idx),
            Arc::clone(&decisions),
            Arc::clone(&selected_idx),
            if workspace_picker.is_some() {
                "Step 2/4: Choose agent"
            } else {
                "Step 1/4: Choose agent"
            },
        );
        let (app_event_tx, _app_event_rx) = tokio::sync::mpsc::unbounded_channel();
        let picker = ListSelectionView::new(
            params,
            AppEventSender::new(app_event_tx),
            RuntimeKeymap::defaults().list,
        );
        let screen = if workspace_picker.is_some() {
            LaunchWizardScreen::ChooseWorkspace
        } else {
            LaunchWizardScreen::ChooseAgent
        };

        Self {
            workspace_choices: workspaces,
            selected_workspace_idx,
            decisions,
            selected_idx,
            workspace_picker,
            picker,
            screen,
            confirm_composer: TextArea::new(),
            confirm_notice: None,
            launch_prepared: false,
            app_event_tx: None,
            completion: None,
        }
    }

    pub(crate) fn set_app_event_sender(&mut self, app_event_tx: AppEventSender) {
        self.app_event_tx = Some(app_event_tx);
    }

    #[cfg(test)]
    pub(crate) fn handle_key(&mut self, key: KeyEvent) {
        self.handle_key_event(key);
    }

    fn handle_workspace_key(&mut self, key: KeyEvent) {
        let Some(picker) = self.workspace_picker.as_mut() else {
            self.screen = LaunchWizardScreen::ChooseAgent;
            return;
        };
        picker.handle_key_event(key);
        if let Some(selected) = picker.selected_index() {
            self.selected_workspace_idx
                .store(selected, Ordering::Relaxed);
        }
        if picker.completion() == Some(ViewCompletion::Cancelled) {
            self.completion = Some(ViewCompletion::Cancelled);
            return;
        }
        if let Some(selected) = picker.take_last_selected_index() {
            self.activate_workspace(selected);
            self.screen = LaunchWizardScreen::ChooseAgent;
        }
    }

    fn activate_workspace(&mut self, selected: usize) {
        self.selected_workspace_idx
            .store(selected, Ordering::Relaxed);
        self.selected_idx.store(0, Ordering::Relaxed);
        self.decisions = selected_workspace(&self.workspace_choices, &self.selected_workspace_idx)
            .map(|workspace| Arc::clone(&workspace.decisions))
            .unwrap_or_default();
        let params = agent_selection_params(
            selected_workspace_display(&self.workspace_choices, &self.selected_workspace_idx),
            Arc::clone(&self.decisions),
            Arc::clone(&self.selected_idx),
            if self.workspace_picker.is_some() {
                "Step 2/4: Choose agent"
            } else {
                "Step 1/4: Choose agent"
            },
        );
        let (app_event_tx, _app_event_rx) = tokio::sync::mpsc::unbounded_channel();
        self.picker = ListSelectionView::new(
            params,
            AppEventSender::new(app_event_tx),
            RuntimeKeymap::defaults().list,
        );
    }

    fn handle_picker_key(&mut self, key: KeyEvent) {
        self.picker.handle_key_event(key);
        if let Some(selected) = self.picker.selected_index() {
            self.selected_idx.store(selected, Ordering::Relaxed);
        }
        if self.picker.completion() == Some(ViewCompletion::Cancelled) {
            self.completion = Some(ViewCompletion::Cancelled);
            return;
        }
        if let Some(selected) = self.picker.take_last_selected_index() {
            self.selected_idx.store(selected, Ordering::Relaxed);
            self.screen = LaunchWizardScreen::ReviewPlan;
        }
    }

    fn handle_review_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace => self.screen = LaunchWizardScreen::ChooseAgent,
            KeyCode::Char('b') if key.modifiers == KeyModifiers::NONE => {
                self.screen = LaunchWizardScreen::ChooseAgent;
            }
            KeyCode::Enter => self.open_confirm(),
            _ => {}
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) {
        if self.selected_plan_requires_typed_confirmation() {
            match key.code {
                KeyCode::Esc => self.screen = LaunchWizardScreen::ReviewPlan,
                KeyCode::Enter => self.confirm_current_plan(),
                _ => {
                    self.confirm_composer.input(key);
                    self.confirm_notice = None;
                }
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Backspace => self.screen = LaunchWizardScreen::ReviewPlan,
            KeyCode::Char('b') if key.modifiers == KeyModifiers::NONE => {
                self.screen = LaunchWizardScreen::ReviewPlan;
            }
            KeyCode::Enter => self.confirm_current_plan(),
            _ => {}
        }
    }

    fn open_confirm(&mut self) {
        if self.selected_launch().is_none() {
            return;
        }
        self.confirm_composer = TextArea::new();
        self.confirm_notice = None;
        self.launch_prepared = false;
        self.screen = LaunchWizardScreen::ConfirmLaunch;
    }

    fn confirm_current_plan(&mut self) {
        let Some(launch) = self.selected_launch().cloned() else {
            self.confirm_notice = Some("Plan could not be built.".to_string());
            return;
        };

        if launch.data.confirm_required && !confirmation_matches(self.confirm_composer.text()) {
            self.confirm_notice = Some(format!("Type {CONFIRM_PHRASE} before confirming."));
            return;
        }

        if !self.launch_prepared {
            if let Some(app_event_tx) = &self.app_event_tx {
                app_event_tx.send(AppEvent::RunHavenLaunchPrepared {
                    launch: Box::new(launch),
                });
            }
            self.launch_prepared = true;
        }
        self.confirm_notice = Some(LAUNCH_PREPARED_NOTICE.to_string());
    }

    fn selected_plan(&self) -> Option<&LaunchPlanData> {
        self.selected_launch().map(|launch| &launch.data)
    }

    fn selected_launch(&self) -> Option<&PreparedLaunch> {
        selected_decision(&self.decisions, &self.selected_idx)
            .and_then(|decision| decision.plan.as_ref().ok())
    }

    fn selected_plan_requires_typed_confirmation(&self) -> bool {
        self.selected_plan()
            .is_some_and(|plan| plan.confirm_required)
    }

    #[cfg(test)]
    pub(crate) fn is_cancelled(&self) -> bool {
        self.completion == Some(ViewCompletion::Cancelled)
    }

    pub(crate) fn selected_agent_name(&self) -> Option<&str> {
        selected_decision(&self.decisions, &self.selected_idx)
            .map(|decision| decision.agent.name.as_str())
    }

    #[cfg(test)]
    pub(crate) fn is_choosing_workspace(&self) -> bool {
        self.screen == LaunchWizardScreen::ChooseWorkspace
    }

    #[cfg(test)]
    pub(crate) fn selected_workspace_path(&self) -> Option<&Path> {
        selected_workspace(&self.workspace_choices, &self.selected_workspace_idx)
            .map(|workspace| workspace.workspace.as_path())
    }

    pub(crate) fn is_reviewing(&self) -> bool {
        self.screen == LaunchWizardScreen::ReviewPlan
    }

    pub(crate) fn is_confirming(&self) -> bool {
        self.screen == LaunchWizardScreen::ConfirmLaunch
    }

    pub(crate) fn confirm_accepts_text_input(&self) -> bool {
        self.is_confirming() && self.selected_plan_requires_typed_confirmation()
    }

    pub(crate) fn handle_paste(&mut self, pasted: &str) {
        if !self.confirm_accepts_text_input() {
            return;
        }
        if !pasted.is_empty() {
            self.confirm_notice = Some(format!(
                "Type {CONFIRM_PHRASE} by hand. Paste is ignored here."
            ));
        }
    }

    pub(crate) fn confirm_cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        rendering::confirm_cursor_position(self, area)
    }

    #[cfg(test)]
    pub(crate) fn confirm_text(&self) -> &str {
        self.confirm_composer.text()
    }

    pub(crate) fn footer_status_line(&self) -> Line<'static> {
        let mut line = Line::from(vec![
            Span::styled("RunHaven", selected_row_style()),
            Span::raw(format!(" v{}", env!("CARGO_PKG_VERSION"))),
            Span::raw(" · "),
            Span::styled(self.step_label(), boundary_style()),
        ]);

        if let Some(decision) = selected_decision(&self.decisions, &self.selected_idx) {
            line.push_span(" · ");
            line.push_span(Span::styled(
                decision.agent.name.clone(),
                decision.status_style(),
            ));
            line.push_span(" · ");
            let network_label = if matches!(
                self.screen,
                LaunchWizardScreen::ChooseWorkspace | LaunchWizardScreen::ChooseAgent
            ) {
                decision.first_step_network_label().to_string()
            } else {
                decision.network_label.clone()
            };
            line.push_span(Span::styled(network_label, decision.network_style()));
            line.push_span(" · ");
            line.push_span(Span::styled(
                decision.boundary_label.clone(),
                boundary_style(),
            ));
        }

        if !self.confirm_accepts_text_input() {
            line.push_span(" · ");
            line.push_span(Span::styled("? help", muted_but_readable_style()));
        }
        line
    }

    pub(crate) fn terminal_title(&self) -> String {
        let agent = self.selected_agent_name().unwrap_or("no agent");
        format!(
            "RunHaven | {} | {} | {agent}",
            selected_workspace_title(&self.workspace_choices, &self.selected_workspace_idx),
            self.step_label()
        )
    }

    fn step_label(&self) -> &'static str {
        match self.screen {
            LaunchWizardScreen::ChooseWorkspace => "Choose workspace",
            LaunchWizardScreen::ChooseAgent => "Choose agent",
            LaunchWizardScreen::ReviewPlan => "Review plan",
            LaunchWizardScreen::ConfirmLaunch => "Confirm launch",
        }
    }
}

impl BottomPaneView for LaunchWizardView {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.completion.is_some() {
            return;
        }
        match self.screen {
            LaunchWizardScreen::ChooseWorkspace => self.handle_workspace_key(key_event),
            LaunchWizardScreen::ChooseAgent => self.handle_picker_key(key_event),
            LaunchWizardScreen::ReviewPlan => self.handle_review_key(key_event),
            LaunchWizardScreen::ConfirmLaunch => self.handle_confirm_key(key_event),
        }
    }

    fn is_complete(&self) -> bool {
        self.completion.is_some()
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.completion
    }

    fn selected_index(&self) -> Option<usize> {
        Some(self.selected_idx.load(Ordering::Relaxed))
    }

    fn view_id(&self) -> Option<&'static str> {
        Some(LAUNCH_WIZARD_VIEW_ID)
    }

    fn terminal_title(&self) -> Option<String> {
        Some(LaunchWizardView::terminal_title(self))
    }

    fn footer_status_line(&self) -> Option<Line<'static>> {
        Some(LaunchWizardView::footer_status_line(self))
    }

    fn accepts_text_input(&self) -> bool {
        self.confirm_accepts_text_input()
    }

    fn footer_help_items(&self) -> Option<Vec<(String, String)>> {
        let mut items = if self.confirm_accepts_text_input() {
            vec![
                ("esc".to_string(), "back".to_string()),
                ("enter".to_string(), "confirm".to_string()),
            ]
        } else if self.screen == LaunchWizardScreen::ChooseWorkspace {
            vec![
                ("up/down".to_string(), "choose".to_string()),
                ("enter".to_string(), "select".to_string()),
                ("q".to_string(), "quit".to_string()),
            ]
        } else if self.is_reviewing() || self.is_confirming() {
            vec![
                ("b".to_string(), "back".to_string()),
                ("esc".to_string(), "back".to_string()),
                ("enter".to_string(), "confirm".to_string()),
                ("q".to_string(), "quit".to_string()),
            ]
        } else {
            vec![
                ("up/down".to_string(), "choose".to_string()),
                ("enter".to_string(), "review".to_string()),
                ("q".to_string(), "quit".to_string()),
            ]
        };
        items.push(("?".to_string(), "hide help".to_string()));
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
        if !self.confirm_accepts_text_input() || pasted.is_empty() {
            return false;
        }
        LaunchWizardView::handle_paste(self, &pasted);
        true
    }
}

impl Renderable for LaunchWizardView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        rendering::render(self, area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        rendering::desired_height(self, width)
    }

    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        self.confirm_cursor_position(area)
    }
}

impl From<AgentLaunchPreview> for AgentDecisionVm {
    fn from(preview: AgentLaunchPreview) -> Self {
        let status_label = match &preview.plan {
            Ok(launch) if launch.data.confirm_required => "review".to_string(),
            Ok(_) => "ready".to_string(),
            Err(_) => "blocked".to_string(),
        };
        let auth_scope_label = preview
            .plan
            .as_ref()
            .map(|launch| launch.data.auth_scope.clone())
            .unwrap_or_else(|_| "unknown".to_string());
        let auth_label = match preview.agent.sign_in.as_str() {
            "n/a" => "no sign-in".to_string(),
            sign_in => format!("{sign_in}, {auth_scope_label} state"),
        };
        let network_label = preview.plan.as_ref().map_or_else(
            |_| network_mode_label(&preview.agent.default_network).to_string(),
            |launch| network_label(&launch.data),
        );

        Self {
            agent: preview.agent,
            plan: preview.plan,
            status_label,
            auth_scope_label,
            auth_label,
            network_label,
            boundary_label: "/workspace only".to_string(),
        }
    }
}

impl From<WorkspaceLaunchPreview> for WorkspaceDecisionVm {
    fn from(choice: WorkspaceLaunchPreview) -> Self {
        Self {
            label: choice.label,
            description: choice.description,
            workspace: choice.payload.workspace,
            decisions: Arc::new(
                choice
                    .payload
                    .previews
                    .into_iter()
                    .map(AgentDecisionVm::from)
                    .collect(),
            ),
        }
    }
}

impl AgentDecisionVm {
    fn selection_item(&self) -> SelectionItem {
        let plan_error = self.plan.as_ref().err().cloned();
        let picker_description = self.picker_description();
        SelectionItem {
            name: self.agent.name.clone(),
            description: Some(picker_description.clone()),
            selected_description: Some(picker_description),
            is_disabled: plan_error.is_some(),
            disabled_reason: plan_error.map(|error| error.to_string()),
            dismiss_on_select: false,
            search_value: Some(self.search_value()),
            ..Default::default()
        }
    }

    fn picker_description(&self) -> String {
        match self.status_label.as_str() {
            "ready" => format!(
                "Ready. {}. Workspace only.",
                self.first_step_network_label()
            ),
            "review" => "Needs confirmation. Review before launch.".to_string(),
            _ => "Blocked. Fix the issue before launch.".to_string(),
        }
    }

    fn search_value(&self) -> String {
        format!(
            "{} {} {} {} {} {} {}",
            self.agent.name,
            self.agent.description,
            self.status_label,
            self.auth_label,
            self.network_label,
            self.boundary_label,
            self.agent.image
        )
    }

    fn network_style(&self) -> Style {
        if self.network_label.contains("internet") {
            warning_style()
        } else {
            safe_style()
        }
    }

    fn status_style(&self) -> Style {
        match self.status_label.as_str() {
            "ready" => safe_style(),
            "review" => warning_style(),
            _ => danger_style(),
        }
    }

    fn first_step_network_label(&self) -> &'static str {
        match self.network_label.as_str() {
            "provider allowlist" => "Provider only",
            "local only" => "Local only",
            "internet unrestricted" => "Unrestricted internet",
            _ => "Custom network",
        }
    }
}

fn workspace_selection_params(
    workspaces: Arc<Vec<WorkspaceDecisionVm>>,
    selected_workspace_idx: Arc<AtomicUsize>,
) -> SelectionViewParams {
    let items = workspaces
        .iter()
        .map(WorkspaceDecisionVm::selection_item)
        .collect::<Vec<_>>();
    let header = WorkspaceHeader {
        workspaces: Arc::clone(&workspaces),
        selected_workspace_idx: Arc::clone(&selected_workspace_idx),
    };
    let preview = WorkspacePreview {
        workspaces: Arc::clone(&workspaces),
        selected_workspace_idx: Arc::clone(&selected_workspace_idx),
    };
    let on_selection_changed = {
        let selected_workspace_idx = Arc::clone(&selected_workspace_idx);
        Some(Box::new(move |idx, _sender: &AppEventSender| {
            selected_workspace_idx.store(idx, Ordering::Relaxed);
        })
            as Box<dyn Fn(usize, &AppEventSender) + Send + Sync>)
    };

    SelectionViewParams {
        view_id: Some("runhaven-launch-workspace"),
        footer_note: Some(Line::from(
            "Choose what RunHaven mounts as /workspace before reviewing an agent.",
        )),
        footer_hint: Some(workspace_footer_hint_line()),
        items,
        is_searchable: false,
        col_width_mode: ColumnWidthMode::AutoAllRows,
        row_display: SelectionRowDisplay::SingleLine,
        name_column_width: Some(22),
        header: Box::new(header),
        initial_selected_idx: Some(0),
        side_content: Box::new(preview.clone()),
        side_content_width: SideContentWidth::Half,
        side_content_min_width: 44,
        stacked_side_content: Some(Box::new(preview)),
        preserve_side_content_bg: false,
        on_selection_changed,
        allow_cancel: true,
        ..Default::default()
    }
}

fn agent_selection_params(
    workspace: String,
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    step_label: &'static str,
) -> SelectionViewParams {
    let items = decisions
        .iter()
        .map(AgentDecisionVm::selection_item)
        .collect::<Vec<_>>();
    let header = SafetyHeader {
        workspace,
        decisions: Arc::clone(&decisions),
        selected_idx: Arc::clone(&selected_idx),
        step_label,
    };
    let on_selection_changed = {
        let selected_idx = Arc::clone(&selected_idx);
        Some(Box::new(move |idx, _sender: &AppEventSender| {
            selected_idx.store(idx, Ordering::Relaxed);
        })
            as Box<dyn Fn(usize, &AppEventSender) + Send + Sync>)
    };

    SelectionViewParams {
        view_id: Some("runhaven-launch-agent"),
        title: None,
        subtitle: None,
        footer_note: Some(Line::from(
            "Enter opens the plan. Nothing starts until you confirm.",
        )),
        footer_hint: Some(footer_hint_line()),
        items,
        is_searchable: false,
        col_width_mode: ColumnWidthMode::AutoAllRows,
        row_display: SelectionRowDisplay::SingleLine,
        name_column_width: Some(13),
        header: Box::new(header),
        initial_selected_idx: Some(0),
        on_selection_changed,
        allow_cancel: true,
        ..Default::default()
    }
}

impl WorkspaceDecisionVm {
    fn selection_item(&self) -> SelectionItem {
        SelectionItem {
            name: self.label.clone(),
            description: Some(self.description.clone()),
            selected_description: Some(format!(
                "{} agents available for this workspace",
                self.decisions.len()
            )),
            search_value: Some(format!(
                "{} {} {}",
                self.label,
                self.description,
                self.workspace.display()
            )),
            dismiss_on_select: false,
            ..Default::default()
        }
    }
}

fn workspace_footer_hint_line() -> Line<'static> {
    Line::from(vec![
        Span::raw("Use "),
        key_hint::plain(KeyCode::Up).into(),
        Span::raw("/"),
        key_hint::plain(KeyCode::Down).into(),
        Span::raw(" or j/k to choose. "),
        key_hint::plain(KeyCode::Enter).into(),
        Span::raw(" selects. "),
        key_hint::plain(KeyCode::Esc).into(),
        Span::raw(" or q quits."),
    ])
}

fn footer_hint_line() -> Line<'static> {
    Line::from(vec![
        Span::raw("Use "),
        key_hint::plain(KeyCode::Up).into(),
        Span::raw("/"),
        key_hint::plain(KeyCode::Down).into(),
        Span::raw(" or j/k to choose. "),
        key_hint::plain(KeyCode::Enter).into(),
        Span::raw(" reviews. "),
        key_hint::plain(KeyCode::Esc).into(),
        Span::raw(" or q quits."),
    ])
}

#[derive(Clone)]
struct WorkspaceHeader {
    workspaces: Arc<Vec<WorkspaceDecisionVm>>,
    selected_workspace_idx: Arc<AtomicUsize>,
}

impl WorkspaceHeader {
    fn selected(&self) -> Option<&WorkspaceDecisionVm> {
        selected_workspace(&self.workspaces, &self.selected_workspace_idx)
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(format!(" v{}  ", env!("CARGO_PKG_VERSION"))),
                Span::styled("Step 1/4: Choose workspace", boundary_style()),
            ]),
            Line::from(vec![
                Span::styled("Safety    ", muted_but_readable_style()),
                Span::styled("/workspace only", boundary_style()),
                Span::raw(". Host home not mounted."),
            ]),
            Line::from(vec![
                Span::styled("Credentials ", muted_but_readable_style()),
                Span::styled("not mounted by default", safe_style()),
            ]),
        ];

        if let Some(selected) = self.selected() {
            lines.push(label_value(
                "Selected",
                selected.label.clone(),
                accent_style(),
            ));
            lines.push(label_value(
                "Workspace",
                selected.workspace.display().to_string(),
                boundary_style(),
            ));
        }
        lines
    }
}

impl Renderable for WorkspaceHeader {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        paragraph(self.lines()).render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        paragraph(self.lines()).line_count(width) as u16
    }
}

#[derive(Clone)]
struct WorkspacePreview {
    workspaces: Arc<Vec<WorkspaceDecisionVm>>,
    selected_workspace_idx: Arc<AtomicUsize>,
}

impl WorkspacePreview {
    fn selected(&self) -> Option<&WorkspaceDecisionVm> {
        selected_workspace(&self.workspaces, &self.selected_workspace_idx)
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let Some(selected) = self.selected() else {
            return vec![Line::from("No workspace choices are available.")];
        };
        vec![
            Line::from(vec![Span::styled(
                "Workspace Preview",
                selected_row_style(),
            )]),
            label_value("Choice", selected.label.clone(), accent_style()),
            label_value(
                "Workspace",
                selected.workspace.display().to_string(),
                boundary_style(),
            ),
            label_value("Mount", "/workspace only", boundary_style()),
            label_value("Host home", "not mounted", safe_style()),
            label_value("Credentials", "not mounted by default", safe_style()),
            label_value(
                "Agents",
                selected.decisions.len().to_string(),
                muted_but_readable_style(),
            ),
            Line::from(""),
            Line::from(selected.description.clone()),
        ]
    }
}

impl Renderable for WorkspacePreview {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        paragraph(self.lines()).render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        paragraph(self.lines()).line_count(width) as u16
    }
}

#[derive(Clone)]
struct SafetyHeader {
    workspace: String,
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    step_label: &'static str,
}

impl SafetyHeader {
    fn selected(&self) -> Option<&AgentDecisionVm> {
        selected_decision(&self.decisions, &self.selected_idx)
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(format!(" v{}  ", env!("CARGO_PKG_VERSION"))),
                Span::styled(self.step_label, boundary_style()),
            ]),
            Line::from("Pick the agent to run. You will review the plan before anything starts."),
        ];

        if let Some(selected) = self.selected() {
            lines.push(Line::from(vec![
                Span::styled("Agent  ", muted_but_readable_style()),
                Span::styled(selected.agent.name.clone(), selected.status_style()),
                Span::raw("  "),
                Span::styled("Network  ", muted_but_readable_style()),
                Span::styled(
                    selected.first_step_network_label(),
                    selected.network_style(),
                ),
            ]));
        }
        lines.push(label_value(
            "Workspace",
            self.workspace.clone(),
            boundary_style(),
        ));
        lines.push(Line::from(vec![
            Span::styled("Safety  ", muted_but_readable_style()),
            Span::styled("/workspace only", boundary_style()),
            Span::raw(". Host home and credentials are not mounted."),
        ]));
        lines
    }
}

impl Renderable for SafetyHeader {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        paragraph(self.lines()).render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        paragraph(self.lines()).line_count(width) as u16
    }
}

fn selected_decision<'a>(
    decisions: &'a [AgentDecisionVm],
    selected_idx: &AtomicUsize,
) -> Option<&'a AgentDecisionVm> {
    let selected = selected_idx.load(Ordering::Relaxed);
    decisions.get(selected).or_else(|| decisions.first())
}

fn selected_workspace<'a>(
    workspaces: &'a [WorkspaceDecisionVm],
    selected_workspace_idx: &AtomicUsize,
) -> Option<&'a WorkspaceDecisionVm> {
    let selected = selected_workspace_idx.load(Ordering::Relaxed);
    workspaces.get(selected).or_else(|| workspaces.first())
}

fn selected_workspace_display(
    workspaces: &[WorkspaceDecisionVm],
    selected_workspace_idx: &AtomicUsize,
) -> String {
    selected_workspace(workspaces, selected_workspace_idx)
        .map(|workspace| workspace.workspace.display().to_string())
        .unwrap_or_else(|| "workspace unavailable".to_string())
}

fn selected_workspace_title(
    workspaces: &[WorkspaceDecisionVm],
    selected_workspace_idx: &AtomicUsize,
) -> String {
    selected_workspace(workspaces, selected_workspace_idx)
        .map(|workspace| workspace_title(&workspace.workspace))
        .unwrap_or_else(|| "workspace".to_string())
}

fn label_value(label: &'static str, value: impl Into<String>, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<12}"), muted_but_readable_style()),
        Span::styled(value.into(), value_style),
    ])
}

fn paragraph(lines: Vec<Line<'static>>) -> Paragraph<'static> {
    Paragraph::new(lines).wrap(Wrap { trim: true })
}

fn review_footer_line() -> Line<'static> {
    Line::from(vec![
        key_hint::plain(KeyCode::Char('b')).into(),
        Span::raw(" or "),
        key_hint::plain(KeyCode::Esc).into(),
        Span::raw(" goes back. "),
        key_hint::plain(KeyCode::Enter).into(),
        Span::raw(" opens confirmation. q quits."),
    ])
}

fn confirm_footer_line(text_field_active: bool) -> Line<'static> {
    let mut line = Line::from(vec![
        key_hint::plain(KeyCode::Esc).into(),
        Span::raw(" goes back. "),
        key_hint::plain(KeyCode::Enter).into(),
        Span::raw(" confirms."),
    ]);
    if !text_field_active {
        line.push_span(" ");
        line.push_span(key_hint::plain(KeyCode::Char('q')));
        line.push_span(" quits.");
    }
    line
}

fn confirmation_matches(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case(CONFIRM_PHRASE)
}

fn network_label(plan: &LaunchPlanData) -> String {
    network_mode_label(&plan.network.mode).to_string()
}

fn network_mode_label(mode: &str) -> &'static str {
    match mode {
        "provider" => "provider allowlist",
        "internal" => "local only",
        "internet" => "internet unrestricted",
        _ => "custom network",
    }
}

fn worktree_label(plan: &LaunchPlanData) -> String {
    plan.worktree
        .as_ref()
        .map(|worktree| format!("on, branch {}", worktree.branch))
        .unwrap_or_else(|| "off".to_string())
}

fn workspace_title(workspace: &Path) -> String {
    workspace
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("workspace")
        .to_string()
}

#[cfg(test)]
#[path = "launch_wizard_tests.rs"]
mod tests;
