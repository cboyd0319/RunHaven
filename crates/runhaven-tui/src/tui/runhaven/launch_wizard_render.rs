use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::widgets::Widget;
use runhaven_core::ui_contracts::LaunchPlanData;

use crate::render::renderable::Renderable;
use crate::style::accent_style;
use crate::style::boundary_style;
use crate::style::danger_style;
use crate::style::muted_but_readable_style;
use crate::style::safe_style;
use crate::style::selected_row_style;
use crate::style::warning_style;
use crate::tui::bottom_pane::render_menu_surface;

use super::AgentDecisionVm;
use super::LaunchWizardScreen;
use super::LaunchWizardView;
use super::label_value;
use super::paragraph;
use super::review_footer_line;
use super::selected_decision;
use super::worktree_label;

#[path = "launch_wizard_render_confirm.rs"]
mod confirm;

pub(super) fn confirm_cursor_position(view: &LaunchWizardView, area: Rect) -> Option<(u16, u16)> {
    confirm::cursor_position(view, area)
}

pub(super) fn render(view: &LaunchWizardView, area: Rect, buf: &mut Buffer) {
    match view.screen {
        LaunchWizardScreen::ChooseWorkspace => view
            .workspace_picker
            .as_ref()
            .expect("workspace picker exists")
            .render(area, buf),
        LaunchWizardScreen::ChooseAgent => view.picker.render(area, buf),
        LaunchWizardScreen::ReviewPlan => ReviewPlan {
            decisions: Arc::clone(&view.decisions),
            selected_idx: Arc::clone(&view.selected_idx),
        }
        .render(area, buf),
        LaunchWizardScreen::ConfirmLaunch => confirm::render(view, area, buf),
    }
}

pub(super) fn desired_height(view: &LaunchWizardView, width: u16) -> u16 {
    match view.screen {
        LaunchWizardScreen::ChooseWorkspace => view
            .workspace_picker
            .as_ref()
            .expect("workspace picker exists")
            .desired_height(width),
        LaunchWizardScreen::ChooseAgent => view.picker.desired_height(width),
        LaunchWizardScreen::ReviewPlan => ReviewPlan {
            decisions: Arc::clone(&view.decisions),
            selected_idx: Arc::clone(&view.selected_idx),
        }
        .desired_height(width),
        LaunchWizardScreen::ConfirmLaunch => confirm::desired_height(view, width),
    }
}

#[derive(Clone)]
struct ReviewPlan {
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
}

impl ReviewPlan {
    fn selected(&self) -> Option<&AgentDecisionVm> {
        selected_decision(self.decisions.as_ref(), &self.selected_idx)
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
            Ok(launch) => append_review_plan_lines(&mut lines, &launch.data),
            Err(error) => {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    error.reason(),
                    danger_style(),
                )]));
                lines.push(Line::from(error.detail().to_string()));
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

pub(super) fn append_safety_note_lines(
    lines: &mut Vec<Line<'static>>,
    plan: &LaunchPlanData,
    limit: usize,
) {
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
