use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

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
use crate::tui::app_event_sender::AppEventSender;
use crate::tui::bottom_pane::BottomPaneView;
use crate::tui::bottom_pane::ColumnWidthMode;
use crate::tui::bottom_pane::ListSelectionView;
use crate::tui::bottom_pane::SelectionItem;
use crate::tui::bottom_pane::SelectionRowDisplay;
use crate::tui::bottom_pane::SelectionViewParams;
use crate::tui::bottom_pane::SideContentWidth;
use crate::tui::bottom_pane::ViewCompletion;
use crate::tui::bottom_pane::render_menu_surface;

pub(crate) struct AgentLaunchPreview {
    pub(crate) agent: AgentCatalogItemData,
    pub(crate) plan: Result<LaunchPlanData, String>,
}

pub(crate) struct LaunchWizardView {
    workspace_title: String,
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    picker: ListSelectionView,
    screen: LaunchWizardScreen,
    confirm_input: String,
    confirm_notice: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentDecisionVm {
    agent: AgentCatalogItemData,
    plan: Result<LaunchPlanData, String>,
    status_label: String,
    auth_scope_label: String,
    auth_label: String,
    network_label: String,
    boundary_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchWizardScreen {
    ChooseAgent,
    ReviewPlan,
    ConfirmLaunch,
}

const CONFIRM_PHRASE: &str = "launch";

impl LaunchWizardView {
    pub(crate) fn new(
        workspace: PathBuf,
        previews: Vec<AgentLaunchPreview>,
        image_smoke_status: Option<Line<'static>>,
    ) -> Self {
        let decisions = Arc::new(
            previews
                .into_iter()
                .map(AgentDecisionVm::from)
                .collect::<Vec<_>>(),
        );
        let selected_idx = Arc::new(AtomicUsize::new(0));
        let params = selection_params(
            workspace.display().to_string(),
            Arc::clone(&decisions),
            Arc::clone(&selected_idx),
            image_smoke_status,
        );
        let picker = ListSelectionView::new(
            params,
            AppEventSender::default(),
            RuntimeKeymap::defaults().list,
        );
        let workspace_title = workspace_title(&workspace);

        Self {
            workspace_title,
            decisions,
            selected_idx,
            picker,
            screen: LaunchWizardScreen::ChooseAgent,
            confirm_input: String::new(),
            confirm_notice: None,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) {
        match self.screen {
            LaunchWizardScreen::ChooseAgent => self.handle_picker_key(key),
            LaunchWizardScreen::ReviewPlan => self.handle_review_key(key),
            LaunchWizardScreen::ConfirmLaunch => self.handle_confirm_key(key),
        }
    }

    fn handle_picker_key(&mut self, key: KeyEvent) {
        self.picker.handle_key_event(key);
        if let Some(selected) = self.picker.selected_index() {
            self.selected_idx.store(selected, Ordering::Relaxed);
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
        match key.code {
            KeyCode::Esc => self.screen = LaunchWizardScreen::ReviewPlan,
            KeyCode::Char('b') if key.modifiers == KeyModifiers::NONE => {
                self.screen = LaunchWizardScreen::ReviewPlan;
            }
            KeyCode::Backspace if self.confirm_input.is_empty() => {
                self.screen = LaunchWizardScreen::ReviewPlan;
            }
            KeyCode::Backspace => {
                self.confirm_input.pop();
                self.confirm_notice = None;
            }
            KeyCode::Enter => self.confirm_current_plan(),
            KeyCode::Char(ch)
                if self.selected_plan_requires_typed_confirmation()
                    && accepts_confirmation_char(key.modifiers, ch)
                    && self.confirm_input.len() < 32 =>
            {
                self.confirm_input.push(ch.to_ascii_lowercase());
                self.confirm_notice = None;
            }
            _ => {}
        }
    }

    fn open_confirm(&mut self) {
        if self.selected_plan().is_none() {
            return;
        }
        self.confirm_input.clear();
        self.confirm_notice = None;
        self.screen = LaunchWizardScreen::ConfirmLaunch;
    }

    fn confirm_current_plan(&mut self) {
        let Some(plan) = self.selected_plan() else {
            self.confirm_notice = Some("Plan could not be built.".to_string());
            return;
        };

        if plan.confirm_required && self.confirm_input.trim() != CONFIRM_PHRASE {
            self.confirm_notice = Some(format!("Type {CONFIRM_PHRASE} before confirming."));
            return;
        }

        self.confirm_notice = Some("Confirmed. TUI launch is still disabled.".to_string());
    }

    fn selected_plan(&self) -> Option<&LaunchPlanData> {
        selected_decision(&self.decisions, &self.selected_idx)
            .and_then(|decision| decision.plan.as_ref().ok())
    }

    fn selected_plan_requires_typed_confirmation(&self) -> bool {
        self.selected_plan()
            .is_some_and(|plan| plan.confirm_required)
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.screen == LaunchWizardScreen::ChooseAgent
            && self.picker.completion() == Some(ViewCompletion::Cancelled)
    }

    #[cfg(test)]
    pub(crate) fn selected_index(&self) -> usize {
        self.selected_idx.load(Ordering::Relaxed)
    }

    pub(crate) fn selected_agent_name(&self) -> Option<&str> {
        selected_decision(&self.decisions, &self.selected_idx)
            .map(|decision| decision.agent.name.as_str())
    }

    #[cfg(test)]
    pub(crate) fn agent_count(&self) -> usize {
        self.decisions.len()
    }

    #[cfg(test)]
    pub(crate) fn search_values_are_populated(&self) -> bool {
        self.decisions.iter().all(|decision| {
            let search_value = decision.search_value();
            !search_value.trim().is_empty()
                && search_value.contains(&decision.agent.name)
                && search_value.contains(&decision.network_label)
        })
    }

    pub(crate) fn is_reviewing(&self) -> bool {
        self.screen == LaunchWizardScreen::ReviewPlan
    }

    pub(crate) fn is_confirming(&self) -> bool {
        self.screen == LaunchWizardScreen::ConfirmLaunch
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
            line.push_span(Span::styled(
                decision.network_label.clone(),
                decision.network_style(),
            ));
            line.push_span(" · ");
            line.push_span(Span::styled(
                decision.boundary_label.clone(),
                boundary_style(),
            ));
        }

        line.push_span(" · ");
        line.push_span(Span::styled("? help", muted_but_readable_style()));
        line
    }

    pub(crate) fn terminal_title(&self) -> String {
        let agent = self.selected_agent_name().unwrap_or("no agent");
        format!(
            "RunHaven | {} | {} | {agent}",
            self.workspace_title,
            self.step_label()
        )
    }

    fn step_label(&self) -> &'static str {
        match self.screen {
            LaunchWizardScreen::ChooseAgent => "Choose agent",
            LaunchWizardScreen::ReviewPlan => "Review plan",
            LaunchWizardScreen::ConfirmLaunch => "Confirm launch",
        }
    }
}

impl Renderable for LaunchWizardView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            LaunchWizardScreen::ChooseAgent => self.picker.render(area, buf),
            LaunchWizardScreen::ReviewPlan => ReviewPlan {
                decisions: Arc::clone(&self.decisions),
                selected_idx: Arc::clone(&self.selected_idx),
            }
            .render(area, buf),
            LaunchWizardScreen::ConfirmLaunch => ConfirmLaunch {
                decisions: Arc::clone(&self.decisions),
                selected_idx: Arc::clone(&self.selected_idx),
                confirm_input: self.confirm_input.clone(),
                confirm_notice: self.confirm_notice.clone(),
            }
            .render(area, buf),
        }
    }

    fn desired_height(&self, width: u16) -> u16 {
        match self.screen {
            LaunchWizardScreen::ChooseAgent => self.picker.desired_height(width),
            LaunchWizardScreen::ReviewPlan => ReviewPlan {
                decisions: Arc::clone(&self.decisions),
                selected_idx: Arc::clone(&self.selected_idx),
            }
            .desired_height(width),
            LaunchWizardScreen::ConfirmLaunch => ConfirmLaunch {
                decisions: Arc::clone(&self.decisions),
                selected_idx: Arc::clone(&self.selected_idx),
                confirm_input: self.confirm_input.clone(),
                confirm_notice: self.confirm_notice.clone(),
            }
            .desired_height(width),
        }
    }
}

impl From<AgentLaunchPreview> for AgentDecisionVm {
    fn from(preview: AgentLaunchPreview) -> Self {
        let status_label = match &preview.plan {
            Ok(plan) if plan.confirm_required => "review".to_string(),
            Ok(_) => "ready".to_string(),
            Err(_) => "blocked".to_string(),
        };
        let auth_scope_label = preview
            .plan
            .as_ref()
            .map(|plan| plan.auth_scope.clone())
            .unwrap_or_else(|_| "unknown".to_string());
        let auth_label = match preview.agent.sign_in.as_str() {
            "n/a" => "no sign-in".to_string(),
            sign_in => format!("{sign_in}, {auth_scope_label} state"),
        };
        let network_label = preview.plan.as_ref().map_or_else(
            |_| network_mode_label(&preview.agent.default_network).to_string(),
            network_label,
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

impl AgentDecisionVm {
    fn selection_item(&self) -> SelectionItem {
        let plan_error = self.plan.as_ref().err().cloned();
        SelectionItem {
            name: self.agent.name.clone(),
            description: Some(format!(
                "{} | {} | {} | {}",
                self.status_label, self.auth_label, self.network_label, self.boundary_label
            )),
            selected_description: Some(format!(
                "{} | broker: {} | image: {}",
                self.agent.description, self.agent.broker, self.agent.image
            )),
            is_disabled: plan_error.is_some(),
            disabled_reason: plan_error,
            dismiss_on_select: false,
            search_value: Some(self.search_value()),
            ..Default::default()
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
}

fn selection_params(
    workspace: String,
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    image_smoke_status: Option<Line<'static>>,
) -> SelectionViewParams {
    let items = decisions
        .iter()
        .map(AgentDecisionVm::selection_item)
        .collect::<Vec<_>>();
    let header = SafetyHeader {
        workspace,
        decisions: Arc::clone(&decisions),
        selected_idx: Arc::clone(&selected_idx),
        image_smoke_status,
    };
    let preview = PlanPreview {
        decisions: Arc::clone(&decisions),
        selected_idx: Arc::clone(&selected_idx),
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
            "Enter reviews the plan. Launch is still disabled in this preview.",
        )),
        footer_hint: Some(footer_hint_line()),
        items,
        is_searchable: false,
        col_width_mode: ColumnWidthMode::AutoAllRows,
        row_display: SelectionRowDisplay::SingleLine,
        name_column_width: Some(13),
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
struct SafetyHeader {
    workspace: String,
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    image_smoke_status: Option<Line<'static>>,
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
                Span::styled("Step 1/4: Choose agent", boundary_style()),
            ]),
            Line::from(vec![
                Span::styled("Boundary  ", muted_but_readable_style()),
                Span::styled("/workspace only", boundary_style()),
                Span::raw("  "),
                Span::styled("Host home  ", muted_but_readable_style()),
                Span::styled("not mounted", safe_style()),
                Span::raw("  "),
                Span::styled("Credentials  ", muted_but_readable_style()),
                Span::styled("not mounted by default", safe_style()),
            ]),
        ];

        if let Some(selected) = self.selected() {
            lines.push(Line::from(vec![
                Span::styled("Network  ", muted_but_readable_style()),
                Span::styled(selected.network_label.clone(), selected.network_style()),
                Span::raw("  "),
                Span::styled("Auth scope  ", muted_but_readable_style()),
                Span::styled(selected.auth_scope_label.clone(), safe_style()),
                Span::raw("  "),
                Span::styled("Selected  ", muted_but_readable_style()),
                Span::styled(selected.agent.name.clone(), selected.status_style()),
            ]));
        }
        lines.push(label_value(
            "Workspace",
            self.workspace.clone(),
            boundary_style(),
        ));
        if let Some(status) = &self.image_smoke_status {
            lines.push(status.clone());
        }
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

#[derive(Clone)]
struct ReviewPlan {
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
}

impl ReviewPlan {
    fn selected(&self) -> Option<&AgentDecisionVm> {
        selected_decision(&self.decisions, &self.selected_idx)
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let Some(decision) = self.selected() else {
            return vec![
                Line::from(vec![
                    Span::styled("RunHaven", selected_row_style()),
                    Span::raw(format!(" v{}  ", env!("CARGO_PKG_VERSION"))),
                    Span::styled("Step 3/4: Review plan", boundary_style()),
                ]),
                Line::from("No agent plan is selected."),
                review_footer_line(),
            ];
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(format!(" v{}  ", env!("CARGO_PKG_VERSION"))),
                Span::styled("Step 3/4: Review plan", boundary_style()),
            ]),
            Line::from("Check what RunHaven will share before launch."),
            Line::from(""),
            label_value("Agent", decision.agent.name.clone(), accent_style()),
            label_value(
                "Status",
                decision.status_label.clone(),
                decision.status_style(),
            ),
            label_value(
                "Auth scope",
                decision.auth_scope_label.clone(),
                safe_style(),
            ),
            label_value(
                "Network",
                decision.network_label.clone(),
                decision.network_style(),
            ),
            label_value(
                "Boundary",
                decision.boundary_label.clone(),
                boundary_style(),
            ),
            label_value("Host home", "not mounted", safe_style()),
            label_value("Credentials", "not mounted by default", safe_style()),
        ];

        match &decision.plan {
            Ok(plan) => append_review_plan_lines(&mut lines, plan),
            Err(message) => {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "Plan could not be built.",
                    danger_style(),
                )]));
                lines.push(Line::from(message.clone()));
            }
        }

        lines.push(Line::from(""));
        lines.push(review_footer_line());
        lines
    }
}

impl Renderable for ReviewPlan {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let content = render_menu_surface(area, buf);
        paragraph(self.lines()).render(content, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        paragraph(self.lines()).line_count(width.saturating_sub(4).max(1)) as u16 + 2
    }
}

#[derive(Clone)]
struct ConfirmLaunch {
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    confirm_input: String,
    confirm_notice: Option<String>,
}

impl ConfirmLaunch {
    fn selected(&self) -> Option<&AgentDecisionVm> {
        selected_decision(&self.decisions, &self.selected_idx)
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let Some(decision) = self.selected() else {
            return vec![
                Line::from(vec![
                    Span::styled("RunHaven", selected_row_style()),
                    Span::raw(format!(" v{}  ", env!("CARGO_PKG_VERSION"))),
                    Span::styled("Step 4/4: Confirm launch", boundary_style()),
                ]),
                Line::from("No agent plan is selected."),
                confirm_footer_line(),
            ];
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("RunHaven", selected_row_style()),
                Span::raw(format!(" v{}  ", env!("CARGO_PKG_VERSION"))),
                Span::styled("Step 4/4: Confirm launch", boundary_style()),
            ]),
            Line::from("Final check before launch."),
            Line::from(""),
            label_value("Agent", decision.agent.name.clone(), accent_style()),
            label_value(
                "Network",
                decision.network_label.clone(),
                decision.network_style(),
            ),
            label_value(
                "Boundary",
                decision.boundary_label.clone(),
                boundary_style(),
            ),
            label_value("Host home", "not mounted", safe_style()),
            label_value("Credentials", "not mounted by default", safe_style()),
        ];

        match &decision.plan {
            Ok(plan) => append_confirm_plan_lines(
                &mut lines,
                plan,
                self.confirm_input.as_str(),
                self.confirm_notice.as_deref(),
            ),
            Err(message) => {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "Plan could not be built.",
                    danger_style(),
                )]));
                lines.push(Line::from(message.clone()));
            }
        }

        lines.push(Line::from(""));
        lines.push(confirm_footer_line());
        lines
    }
}

impl Renderable for ConfirmLaunch {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let content = render_menu_surface(area, buf);
        paragraph(self.lines()).render(content, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        paragraph(self.lines()).line_count(width.saturating_sub(4).max(1)) as u16 + 2
    }
}

#[derive(Clone)]
struct PlanPreview {
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
}

impl PlanPreview {
    fn selected(&self) -> Option<&AgentDecisionVm> {
        selected_decision(&self.decisions, &self.selected_idx)
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let Some(decision) = self.selected() else {
            return vec![Line::from("No agents are configured.")];
        };
        let mut lines = vec![
            Line::from(vec![Span::styled("Plan Preview", selected_row_style())]),
            label_value("Agent", decision.agent.name.clone(), accent_style()),
            label_value(
                "Status",
                decision.status_label.clone(),
                decision.status_style(),
            ),
            label_value("Sign in", decision.agent.sign_in.clone(), safe_style()),
            label_value(
                "Auth scope",
                decision.auth_scope_label.clone(),
                safe_style(),
            ),
            label_value(
                "Network",
                decision.network_label.clone(),
                decision.network_style(),
            ),
            label_value(
                "Boundary",
                decision.boundary_label.clone(),
                boundary_style(),
            ),
            label_value("Host home", "not mounted", safe_style()),
            label_value("Credentials", "not mounted by default", safe_style()),
        ];

        match &decision.plan {
            Ok(plan) => append_plan_lines(&mut lines, plan),
            Err(message) => {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "Plan could not be built.",
                    danger_style(),
                )]));
                lines.push(Line::from(message.clone()));
            }
        }

        lines
    }
}

impl Renderable for PlanPreview {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        paragraph(self.lines()).render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        paragraph(self.lines()).line_count(width) as u16
    }
}

fn append_review_plan_lines(lines: &mut Vec<Line<'static>>, plan: &LaunchPlanData) {
    lines.push(label_value(
        "Workspace",
        plan.workspace.clone(),
        boundary_style(),
    ));
    lines.push(label_value(
        "Mount",
        plan.boundary.mounted_workspace.clone(),
        boundary_style(),
    ));
    lines.push(label_value(
        "State",
        plan.state_volume.clone(),
        safe_style(),
    ));
    lines.push(label_value(
        "Image",
        plan.image.clone(),
        muted_but_readable_style(),
    ));
    lines.push(label_value(
        "Worktree",
        worktree_label(plan),
        muted_but_readable_style(),
    ));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Exact command",
        selected_row_style(),
    )]));
    lines.push(Line::from(plan.command.clone()));
    append_not_shared_lines(lines, plan);
    append_provider_host_lines(lines, plan, 6);
    append_safety_note_lines(lines, plan, 4);
}

fn append_confirm_plan_lines(
    lines: &mut Vec<Line<'static>>,
    plan: &LaunchPlanData,
    confirm_input: &str,
    confirm_notice: Option<&str>,
) {
    lines.push(Line::from(""));
    if plan.confirm_required {
        lines.push(Line::from(vec![Span::styled(
            "This plan needs typed confirmation.",
            warning_style(),
        )]));
        lines.push(Line::from(vec![
            Span::raw("Type "),
            Span::styled(CONFIRM_PHRASE, selected_row_style()),
            Span::raw(", then press Enter."),
        ]));
        lines.push(Line::from("This preview will not start the container yet."));
        lines.push(label_value(
            "Typed",
            if confirm_input.is_empty() {
                "<nothing>".to_string()
            } else {
                confirm_input.to_string()
            },
            muted_but_readable_style(),
        ));
    } else {
        lines.push(Line::from(
            "Press Enter to confirm. This preview will not start the container yet.",
        ));
    }

    if let Some(notice) = confirm_notice {
        let style = if notice.starts_with("Confirmed") {
            safe_style()
        } else {
            warning_style()
        };
        lines.push(Line::from(vec![Span::styled(notice.to_string(), style)]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Exact command",
        selected_row_style(),
    )]));
    lines.push(Line::from(plan.command.clone()));
    append_safety_note_lines(lines, plan, 3);
}

fn append_plan_lines(lines: &mut Vec<Line<'static>>, plan: &LaunchPlanData) {
    append_not_shared_lines(lines, plan);
    lines.push(Line::from(""));
    lines.push(label_value(
        "Mount",
        plan.boundary.mounted_workspace.clone(),
        boundary_style(),
    ));
    lines.push(label_value(
        "State",
        plan.state_volume.clone(),
        safe_style(),
    ));
    lines.push(label_value(
        "Image",
        plan.image.clone(),
        muted_but_readable_style(),
    ));
    lines.push(label_value(
        "Worktree",
        worktree_label(plan),
        muted_but_readable_style(),
    ));

    append_provider_host_lines(lines, plan, 4);
    append_safety_note_lines(lines, plan, 3);

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Exact command before launch",
        selected_row_style(),
    )]));
    lines.push(Line::from(plan.command.clone()));
}

fn append_not_shared_lines(lines: &mut Vec<Line<'static>>, plan: &LaunchPlanData) {
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Not shared",
        selected_row_style(),
    )]));
    for item in &plan.boundary.not_shared {
        lines.push(Line::from(vec![
            Span::raw("- "),
            Span::styled(item.clone(), safe_style()),
        ]));
    }
}

fn append_provider_host_lines(lines: &mut Vec<Line<'static>>, plan: &LaunchPlanData, limit: usize) {
    if plan.network.provider_allowed_hosts.is_empty() {
        return;
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Provider hosts",
        selected_row_style(),
    )]));
    for host in plan.network.provider_allowed_hosts.iter().take(limit) {
        lines.push(Line::from(format!("- {host}")));
    }
    if plan.network.provider_allowed_hosts.len() > limit {
        lines.push(Line::from(format!(
            "- {} more",
            plan.network.provider_allowed_hosts.len() - limit
        )));
    }
}

fn append_safety_note_lines(lines: &mut Vec<Line<'static>>, plan: &LaunchPlanData, limit: usize) {
    if plan.safety_notes.is_empty() {
        return;
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Safety notes",
        warning_style(),
    )]));
    for note in plan.safety_notes.iter().take(limit) {
        lines.push(Line::from(format!("- {note}")));
    }
}

fn selected_decision<'a>(
    decisions: &'a [AgentDecisionVm],
    selected_idx: &AtomicUsize,
) -> Option<&'a AgentDecisionVm> {
    let selected = selected_idx.load(Ordering::Relaxed);
    decisions.get(selected).or_else(|| decisions.first())
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

fn confirm_footer_line() -> Line<'static> {
    Line::from(vec![
        key_hint::plain(KeyCode::Char('b')).into(),
        Span::raw(" or "),
        key_hint::plain(KeyCode::Esc).into(),
        Span::raw(" goes back. "),
        key_hint::plain(KeyCode::Enter).into(),
        Span::raw(" confirms. q quits."),
    ])
}

fn accepts_confirmation_char(modifiers: KeyModifiers, ch: char) -> bool {
    matches!(modifiers, KeyModifiers::NONE | KeyModifiers::SHIFT) && ch.is_ascii_alphanumeric()
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
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use runhaven_core::ui_contracts::LaunchBoundaryData;
    use runhaven_core::ui_contracts::LaunchNetworkData;

    fn ready_preview(name: &str) -> AgentLaunchPreview {
        AgentLaunchPreview {
            agent: agent(name),
            plan: Ok(plan(name)),
        }
    }

    fn confirm_required_preview(name: &str) -> AgentLaunchPreview {
        let mut plan = plan(name);
        plan.confirm_required = true;
        plan.safety_notes
            .push("This plan uses a less safe launch option.".to_string());
        AgentLaunchPreview {
            agent: agent(name),
            plan: Ok(plan),
        }
    }

    fn blocked_preview(name: &str) -> AgentLaunchPreview {
        AgentLaunchPreview {
            agent: agent(name),
            plan: Err("workspace is blocked".to_string()),
        }
    }

    fn agent(name: &str) -> AgentCatalogItemData {
        AgentCatalogItemData {
            name: name.to_string(),
            description: format!("{name} description"),
            image: format!("runhaven/{name}:0.1.0"),
            sign_in: "runhaven login".to_string(),
            broker: "no".to_string(),
            default_network: "provider".to_string(),
            provider_host_count: 1,
        }
    }

    fn plan(name: &str) -> LaunchPlanData {
        LaunchPlanData {
            profile_name: name.to_string(),
            workspace: "/tmp/project".to_string(),
            workspace_scope: "current".to_string(),
            workspace_scope_note: None,
            auth_scope: "agent".to_string(),
            session: "none".to_string(),
            state_volume: format!("runhaven-{name}-state"),
            container_name: format!("runhaven-{name}"),
            image: format!("runhaven/{name}:0.1.0"),
            worktree: None,
            network: LaunchNetworkData {
                mode: "provider".to_string(),
                name: Some("runhaven-provider".to_string()),
                summary: "provider allowlist".to_string(),
                provider_allowed_hosts: vec!["example.com".to_string()],
                api_key_broker_env: None,
            },
            boundary: LaunchBoundaryData {
                mounted_workspace: "/tmp/project -> /workspace".to_string(),
                mounted_state_volume: format!("runhaven-{name}-state -> /home/agent"),
                not_shared: vec![
                    "host home folder".to_string(),
                    "raw SSH keys".to_string(),
                    "browser profiles".to_string(),
                ],
            },
            preflight_commands: Vec::new(),
            command: format!("container run --name runhaven-{name} runhaven/{name}:0.1.0"),
            safety_notes: Vec::new(),
            confirm_required: false,
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn modified_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn render_to_text(view: &LaunchWizardView, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
        terminal
            .draw(|frame| view.render(frame.area(), frame.buffer_mut()))
            .expect("draw");
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

    #[test]
    fn enter_on_ready_plan_opens_review() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![ready_preview("codex")],
            None,
        );

        view.handle_key(key(KeyCode::Enter));

        assert!(view.is_reviewing());
        assert!(!view.is_cancelled());
        assert_eq!(view.selected_agent_name(), Some("codex"));
    }

    #[test]
    fn enter_on_blocked_plan_stays_in_picker() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![blocked_preview("blocked")],
            None,
        );

        view.handle_key(key(KeyCode::Enter));

        assert!(!view.is_reviewing());
        assert!(!view.is_cancelled());
        assert_eq!(view.selected_agent_name(), Some("blocked"));
    }

    #[test]
    fn back_from_review_keeps_selected_agent() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![ready_preview("codex"), ready_preview("claude")],
            None,
        );
        view.handle_key(key(KeyCode::Down));
        view.handle_key(key(KeyCode::Enter));

        assert!(view.is_reviewing());
        assert_eq!(view.selected_agent_name(), Some("claude"));

        view.handle_key(key(KeyCode::Esc));

        assert!(!view.is_reviewing());
        assert_eq!(view.selected_agent_name(), Some("claude"));
    }

    #[test]
    fn review_enter_opens_confirm_step_and_keeps_selected_agent() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![ready_preview("codex"), ready_preview("claude")],
            None,
        );
        view.handle_key(key(KeyCode::Down));
        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Enter));

        assert!(view.is_confirming());
        assert_eq!(view.selected_agent_name(), Some("claude"));
        assert!(view.terminal_title().contains("Confirm launch"));
    }

    #[test]
    fn confirm_back_keys_return_to_review() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![ready_preview("codex")],
            None,
        );
        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Enter));

        assert!(view.is_confirming());
        view.handle_key(key(KeyCode::Backspace));

        assert!(view.is_reviewing());

        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Esc));

        assert!(view.is_reviewing());
    }

    #[test]
    fn secure_plan_confirm_enter_sets_disabled_launch_notice() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![ready_preview("codex")],
            None,
        );
        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Enter));

        assert!(view.is_confirming());
        assert_eq!(
            view.confirm_notice.as_deref(),
            Some("Confirmed. TUI launch is still disabled.")
        );
    }

    #[test]
    fn confirm_required_plan_rejects_missing_phrase() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![confirm_required_preview("codex")],
            None,
        );
        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Enter));

        assert!(view.is_confirming());
        assert_eq!(
            view.confirm_notice.as_deref(),
            Some("Type launch before confirming.")
        );
    }

    #[test]
    fn confirm_required_plan_accepts_phrase_after_backspace_edit() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![confirm_required_preview("codex")],
            None,
        );
        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Enter));

        view.handle_key(modified_key(KeyCode::Char('L'), KeyModifiers::SHIFT));
        for ch in ['a', 'x'] {
            view.handle_key(key(KeyCode::Char(ch)));
        }
        view.handle_key(key(KeyCode::Backspace));
        for ch in ['u', 'n', 'c', 'h'] {
            view.handle_key(key(KeyCode::Char(ch)));
        }
        view.handle_key(modified_key(KeyCode::Char('!'), KeyModifiers::SHIFT));

        assert_eq!(view.confirm_input, "launch");

        view.handle_key(key(KeyCode::Enter));

        assert_eq!(
            view.confirm_notice.as_deref(),
            Some("Confirmed. TUI launch is still disabled.")
        );
    }

    #[test]
    fn confirm_render_keeps_command_and_safety_notes_visible() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![confirm_required_preview("codex")],
            None,
        );
        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Enter));
        let output = render_to_text(&view, 100, 32);

        assert!(output.contains("Step 4/4: Confirm launch"));
        assert!(output.contains("This plan needs typed confirmation."));
        assert!(output.contains("Type launch, then press Enter."));
        assert!(output.contains("Exact command"));
        assert!(output.contains("container run"));
        assert!(output.contains("Safety notes"));
        assert!(output.contains("less safe launch option"));
    }

    #[test]
    fn confirm_step_footer_status_and_title_track_step_four() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![ready_preview("codex")],
            None,
        );
        view.handle_key(key(KeyCode::Enter));
        view.handle_key(key(KeyCode::Enter));

        let footer = format!("{:?}", view.footer_status_line());
        assert!(footer.contains("Confirm launch"));
        assert!(footer.contains("codex"));
        assert!(view.terminal_title().contains("Confirm launch"));
        assert!(view.terminal_title().contains("codex"));
    }

    #[test]
    fn footer_status_and_title_track_selected_plan() {
        let mut view = LaunchWizardView::new(
            PathBuf::from("/tmp/project"),
            vec![ready_preview("codex"), ready_preview("claude")],
            None,
        );

        let footer = format!("{:?}", view.footer_status_line());
        assert!(footer.contains("Choose agent"));
        assert!(footer.contains("codex"));
        assert!(footer.contains("provider allowlist"));
        assert!(view.terminal_title().contains("project"));
        assert!(view.terminal_title().contains("Choose agent"));
        assert!(view.terminal_title().contains("codex"));

        view.handle_key(key(KeyCode::Down));
        view.handle_key(key(KeyCode::Enter));

        let footer = format!("{:?}", view.footer_status_line());
        assert!(footer.contains("Review plan"));
        assert!(footer.contains("claude"));
        assert!(view.terminal_title().contains("Review plan"));
        assert!(view.terminal_title().contains("claude"));

        view.handle_key(key(KeyCode::Enter));

        let footer = format!("{:?}", view.footer_status_line());
        assert!(footer.contains("Confirm launch"));
        assert!(footer.contains("claude"));
        assert!(view.terminal_title().contains("Confirm launch"));
        assert!(view.terminal_title().contains("claude"));
    }
}
