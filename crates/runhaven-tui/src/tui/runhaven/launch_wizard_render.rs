use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::WidgetRef;
use runhaven_core::ui_contracts::LaunchPlanData;

use crate::render::renderable::Renderable;
use crate::style::accent_style;
use crate::style::boundary_style;
use crate::style::danger_style;
use crate::style::muted_but_readable_style;
use crate::style::safe_style;
use crate::style::selected_row_style;
use crate::style::warning_style;
use crate::tui::bottom_pane::TextArea;
use crate::tui::bottom_pane::menu_surface_inset;
use crate::tui::bottom_pane::render_menu_surface;

use super::AgentDecisionVm;
use super::CONFIRM_PHRASE;
use super::LaunchWizardScreen;
use super::LaunchWizardView;
use super::confirm_footer_line;
use super::label_value;
use super::paragraph;
use super::review_footer_line;
use super::selected_decision;
use super::worktree_label;

pub(super) fn confirm_cursor_position(view: &LaunchWizardView, area: Rect) -> Option<(u16, u16)> {
    if !view.confirm_accepts_text_input() {
        return None;
    }
    let composer_area = ConfirmLaunch {
        decisions: Arc::clone(&view.decisions),
        selected_idx: Arc::clone(&view.selected_idx),
        confirm_composer: &view.confirm_composer,
        confirm_notice: view.confirm_notice.clone(),
    }
    .composer_text_area(area)?;
    view.confirm_composer.cursor_pos(composer_area)
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
        LaunchWizardScreen::ConfirmLaunch => ConfirmLaunch {
            decisions: Arc::clone(&view.decisions),
            selected_idx: Arc::clone(&view.selected_idx),
            confirm_composer: &view.confirm_composer,
            confirm_notice: view.confirm_notice.clone(),
        }
        .render(area, buf),
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
        LaunchWizardScreen::ConfirmLaunch => ConfirmLaunch {
            decisions: Arc::clone(&view.decisions),
            selected_idx: Arc::clone(&view.selected_idx),
            confirm_composer: &view.confirm_composer,
            confirm_notice: view.confirm_notice.clone(),
        }
        .desired_height(width),
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

struct ConfirmLaunch<'a> {
    decisions: Arc<Vec<AgentDecisionVm>>,
    selected_idx: Arc<AtomicUsize>,
    confirm_composer: &'a TextArea,
    confirm_notice: Option<String>,
}

impl ConfirmLaunch<'_> {
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
                confirm_footer_line(false),
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
            Ok(launch) => append_confirm_plan_lines(
                &mut lines,
                &launch.data,
                self.confirm_composer.text(),
                self.confirm_notice.as_deref(),
            ),
            Err(error) => {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    error.reason(),
                    danger_style(),
                )]));
                lines.push(Line::from(error.detail().to_string()));
            }
        }

        let text_field_active = decision
            .plan
            .as_ref()
            .is_ok_and(|launch| launch.data.confirm_required);
        lines.push(Line::from(""));
        lines.push(confirm_footer_line(text_field_active));
        lines
    }

    fn layout(&self, area: Rect) -> (Rect, Option<Rect>) {
        let content = menu_surface_inset(area);
        let Some(plan) = self
            .selected()
            .and_then(|decision| decision.plan.as_ref().ok())
        else {
            return (content, None);
        };
        if !plan.data.confirm_required || content.height < 6 {
            return (content, None);
        }

        let composer_height = self
            .confirm_composer
            .desired_height(content.width.saturating_sub(2).max(1))
            .clamp(1, 3)
            .saturating_add(1);
        let composer_height = composer_height.min(content.height.saturating_sub(1));
        let body_height = content
            .height
            .saturating_sub(composer_height)
            .saturating_sub(1);
        let body = Rect {
            height: body_height,
            ..content
        };
        let composer = Rect {
            y: body.y.saturating_add(body.height).saturating_add(1),
            height: composer_height,
            ..content
        };
        (body, Some(composer))
    }

    fn composer_text_area(&self, area: Rect) -> Option<Rect> {
        let (_, composer) = self.layout(area);
        composer.map(|area| Rect {
            x: area.x.saturating_add(2),
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(1).max(1),
            ..area
        })
    }

    fn render_composer(&self, area: Rect, buf: &mut Buffer) {
        let label_area = Rect { height: 1, ..area };
        Paragraph::new(Line::from(vec![
            Span::styled("> ", accent_style()),
            Span::styled("confirmation phrase", muted_but_readable_style()),
        ]))
        .render(label_area, buf);

        let text_area = Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(1).max(1),
        };
        if self.confirm_composer.is_empty() {
            Paragraph::new(Line::from(Span::styled(
                "type launch",
                muted_but_readable_style(),
            )))
            .render(text_area, buf);
        } else {
            (&self.confirm_composer).render_ref(text_area, buf);
        }
    }
}

impl Renderable for ConfirmLaunch<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let content = render_menu_surface(area, buf);
        let (body, composer) = self.layout(area);
        let body = if body == content { content } else { body };
        paragraph(self.lines()).render(body, buf);
        if let Some(composer) = composer {
            self.render_composer(composer, buf);
        }
    }

    fn desired_height(&self, width: u16) -> u16 {
        let body = paragraph(self.lines()).line_count(width.saturating_sub(4).max(1)) as u16 + 2;
        let composer = self
            .selected()
            .and_then(|decision| decision.plan.as_ref().ok())
            .filter(|launch| launch.data.confirm_required)
            .map(|_| {
                self.confirm_composer
                    .desired_height(width.saturating_sub(6).max(1))
                    .clamp(1, 3)
                    .saturating_add(2)
            })
            .unwrap_or(0);
        body.saturating_add(composer)
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
        lines.push(Line::from(
            "This step prepares the launch. RunHaven starts it after restoring the terminal.",
        ));
        if !confirm_input.trim().is_empty() {
            lines.push(label_value(
                "Typed",
                confirm_input.trim().to_string(),
                muted_but_readable_style(),
            ));
        }
    } else {
        lines.push(Line::from(
            "Press Enter to prepare launch. RunHaven starts it after restoring the terminal.",
        ));
    }

    if let Some(notice) = confirm_notice {
        let style = if notice.starts_with("Launch prepared") {
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
