#[cfg(test)]
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[cfg(test)]
use super::service::AgentLaunchPreview;
#[cfg(test)]
use super::service::LaunchPreviewPayload;
use super::service::PreparedLaunch;
use super::service::WorkspaceLaunchPreview;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::text::Span;
use runhaven_core::ui_contracts::LaunchPlanData;

use crate::keymap::RuntimeKeymap;
use crate::style::boundary_style;
use crate::style::muted_but_readable_style;
use crate::style::selected_row_style;
use crate::tui::app_event::AppEvent;
use crate::tui::app_event_sender::AppEventSender;
use crate::tui::bottom_pane::BottomPaneView;
use crate::tui::bottom_pane::ListSelectionView;
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

#[path = "launch_wizard_bottom_pane.rs"]
mod bottom_pane_adapter;
#[path = "launch_wizard_picker.rs"]
mod picker;
#[path = "launch_wizard_render.rs"]
mod rendering;
#[path = "launch_wizard_state.rs"]
mod state;

use state::AgentDecisionVm;
use state::WorkspaceDecisionVm;
use state::confirm_footer_line;
use state::confirmation_matches;
use state::label_value;
use state::paragraph;
use state::review_footer_line;
use state::selected_decision;
use state::selected_workspace;
use state::workspace_title;
use state::worktree_label;

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
        let decisions = selected_workspace(workspaces.as_ref(), &selected_workspace_idx)
            .map(|workspace| Arc::clone(&workspace.decisions))
            .unwrap_or_default();
        let selected_idx = Arc::new(AtomicUsize::new(0));
        let workspace_picker = (workspaces.len() > 1).then(|| {
            let params = picker::workspace_selection_params(
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
        let params = picker::agent_selection_params(
            picker::selected_workspace_display(&workspaces, &selected_workspace_idx),
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
        self.decisions = selected_workspace(
            self.workspace_choices.as_ref(),
            &self.selected_workspace_idx,
        )
        .map(|workspace| Arc::clone(&workspace.decisions))
        .unwrap_or_default();
        let params = picker::agent_selection_params(
            picker::selected_workspace_display(
                &self.workspace_choices,
                &self.selected_workspace_idx,
            ),
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
        selected_decision(self.decisions.as_ref(), &self.selected_idx)
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
        selected_decision(self.decisions.as_ref(), &self.selected_idx)
            .map(|decision| decision.agent.name.as_str())
    }

    #[cfg(test)]
    pub(crate) fn is_choosing_workspace(&self) -> bool {
        self.screen == LaunchWizardScreen::ChooseWorkspace
    }

    #[cfg(test)]
    pub(crate) fn selected_workspace_path(&self) -> Option<&std::path::Path> {
        selected_workspace(
            self.workspace_choices.as_ref(),
            &self.selected_workspace_idx,
        )
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

        if let Some(decision) = selected_decision(self.decisions.as_ref(), &self.selected_idx) {
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
            picker::selected_workspace_title(&self.workspace_choices, &self.selected_workspace_idx),
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

#[cfg(test)]
#[path = "launch_wizard_tests.rs"]
mod tests;
