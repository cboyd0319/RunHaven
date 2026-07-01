use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Widget;

use crate::key_hint;
use crate::render::renderable::Renderable;
use crate::style::accent_style;
use crate::style::boundary_style;
use crate::style::muted_but_readable_style;
use crate::style::safe_style;
use crate::style::selected_row_style;
use crate::tui::app_event_sender::AppEventSender;
use crate::tui::bottom_pane::ColumnWidthMode;
use crate::tui::bottom_pane::SelectionItem;
use crate::tui::bottom_pane::SelectionRowDisplay;
use crate::tui::bottom_pane::SelectionViewParams;
use crate::tui::bottom_pane::SideContentWidth;

use super::AgentDecisionVm;
use super::WorkspaceDecisionVm;
use super::label_value;
use super::paragraph;
use super::selected_decision;
use super::selected_workspace;
use super::workspace_title;

pub(super) fn workspace_selection_params(
    workspaces: Arc<Vec<WorkspaceDecisionVm>>,
    selected_workspace_idx: Arc<AtomicUsize>,
) -> SelectionViewParams {
    let items = workspaces
        .iter()
        .map(workspace_selection_item)
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

pub(super) fn agent_selection_params(
    workspace: String,
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    step_label: &'static str,
) -> SelectionViewParams {
    let items = decisions
        .iter()
        .map(agent_selection_item)
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
        footer_hint: Some(agent_footer_hint_line()),
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

pub(super) fn selected_workspace_display(
    workspaces: &[WorkspaceDecisionVm],
    selected_workspace_idx: &AtomicUsize,
) -> String {
    selected_workspace(workspaces, selected_workspace_idx)
        .map(|workspace| workspace.workspace.display().to_string())
        .unwrap_or_else(|| "workspace unavailable".to_string())
}

pub(super) fn selected_workspace_title(
    workspaces: &[WorkspaceDecisionVm],
    selected_workspace_idx: &AtomicUsize,
) -> String {
    selected_workspace(workspaces, selected_workspace_idx)
        .map(|workspace| workspace_title(&workspace.workspace))
        .unwrap_or_else(|| "workspace".to_string())
}

fn agent_selection_item(decision: &AgentDecisionVm) -> SelectionItem {
    let plan_error = decision.plan.as_ref().err().cloned();
    let description = agent_picker_description(decision);
    SelectionItem {
        name: decision.agent.name.clone(),
        description: Some(description.clone()),
        selected_description: Some(description),
        is_disabled: plan_error.is_some(),
        disabled_reason: plan_error.map(|error| error.to_string()),
        dismiss_on_select: false,
        search_value: Some(agent_search_value(decision)),
        ..Default::default()
    }
}

fn agent_picker_description(decision: &AgentDecisionVm) -> String {
    match decision.status_label.as_str() {
        "ready" => format!(
            "Ready. {}. Workspace only.",
            decision.first_step_network_label()
        ),
        "review" => "Needs confirmation. Review before launch.".to_string(),
        _ => "Blocked. Fix the issue before launch.".to_string(),
    }
}

fn agent_search_value(decision: &AgentDecisionVm) -> String {
    format!(
        "{} {} {} {} {} {} {}",
        decision.agent.name,
        decision.agent.description,
        decision.status_label,
        decision.auth_label,
        decision.network_label,
        decision.boundary_label,
        decision.agent.image
    )
}

fn workspace_selection_item(workspace: &WorkspaceDecisionVm) -> SelectionItem {
    SelectionItem {
        name: workspace.label.clone(),
        description: Some(workspace.description.clone()),
        selected_description: Some(format!(
            "{} agents available for this workspace",
            workspace.decisions.len()
        )),
        search_value: Some(format!(
            "{} {} {}",
            workspace.label,
            workspace.description,
            workspace.workspace.display()
        )),
        dismiss_on_select: false,
        ..Default::default()
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

fn agent_footer_hint_line() -> Line<'static> {
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
        selected_workspace(self.workspaces.as_ref(), &self.selected_workspace_idx)
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
        selected_workspace(self.workspaces.as_ref(), &self.selected_workspace_idx)
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
        selected_decision(self.decisions.as_ref(), &self.selected_idx)
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
