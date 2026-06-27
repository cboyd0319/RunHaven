use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
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
use crate::tui::app_event_sender::AppEventSender;
use crate::tui::bottom_pane::BottomPaneView;
use crate::tui::bottom_pane::ColumnWidthMode;
use crate::tui::bottom_pane::ListSelectionView;
use crate::tui::bottom_pane::SelectionItem;
use crate::tui::bottom_pane::SelectionRowDisplay;
use crate::tui::bottom_pane::SelectionViewParams;
use crate::tui::bottom_pane::SideContentWidth;
use crate::tui::bottom_pane::ViewCompletion;

pub(crate) struct AgentLaunchPreview {
    pub(crate) agent: AgentCatalogItemData,
    pub(crate) plan: Result<LaunchPlanData, String>,
}

pub(crate) struct LaunchWizardView {
    #[cfg(test)]
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    picker: ListSelectionView,
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

        Self {
            #[cfg(test)]
            decisions,
            selected_idx,
            picker,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) {
        self.picker.handle_key_event(key);
        if let Some(selected) = self.picker.selected_index() {
            self.selected_idx.store(selected, Ordering::Relaxed);
        }
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.picker.completion() == Some(ViewCompletion::Cancelled)
    }

    #[cfg(test)]
    pub(crate) fn selected_index(&self) -> usize {
        self.selected_idx.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn selected_agent_name(&self) -> Option<&str> {
        self.selected_decision()
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

    #[cfg(test)]
    fn selected_decision(&self) -> Option<&AgentDecisionVm> {
        let selected = self.selected_idx.load(Ordering::Relaxed);
        self.decisions
            .get(selected)
            .or_else(|| self.decisions.first())
    }
}

impl Renderable for LaunchWizardView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.picker.render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        self.picker.desired_height(width)
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
            "Review shows the exact command before launch. Enter does not launch.",
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

fn append_plan_lines(lines: &mut Vec<Line<'static>>, plan: &LaunchPlanData) {
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

    if !plan.network.provider_allowed_hosts.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Provider hosts",
            selected_row_style(),
        )]));
        for host in plan.network.provider_allowed_hosts.iter().take(4) {
            lines.push(Line::from(format!("- {host}")));
        }
        if plan.network.provider_allowed_hosts.len() > 4 {
            lines.push(Line::from(format!(
                "- {} more",
                plan.network.provider_allowed_hosts.len() - 4
            )));
        }
    }

    if !plan.safety_notes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Safety notes",
            warning_style(),
        )]));
        for note in plan.safety_notes.iter().take(3) {
            lines.push(Line::from(format!("- {note}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Exact command before launch",
        selected_row_style(),
    )]));
    lines.push(Line::from(plan.command.clone()));
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
