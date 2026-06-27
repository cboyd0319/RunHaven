use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, ListItem, Paragraph};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::theme::{Palette, TuiSettings};
use super::{launcher, runs};
use crate::runhaven::runtime::plans::default_network_mode;
use crate::runhaven::runtime::profiles::AgentProfile;

/// The home banner: Cubby on the left, brand and tagline on the right.
pub(super) fn render_banner(
    frame: &mut Frame,
    area: Rect,
    mascot_width: u16,
    mascot_lines: Vec<Line<'static>>,
    palette: Palette,
) -> Rect {
    let [mascot_area, brand_area] =
        Layout::horizontal([Constraint::Length(mascot_width + 2), Constraint::Min(0)]).areas(area);

    frame.render_widget(Paragraph::new(mascot_lines), mascot_area);

    // Vertically center the brand against the mascot.
    let brand = [
        Line::styled("RunHaven", palette.accent()),
        Line::styled(format!("v{}", env!("CARGO_PKG_VERSION")), palette.muted()),
        Line::from(""),
        Line::styled("run agents in a safe haven", palette.muted()),
    ];
    let pad = area.height.saturating_sub(brand.len() as u16) / 2;
    let mut lines = vec![Line::from(""); pad as usize];
    lines.extend(brand);
    frame.render_widget(Paragraph::new(lines), brand_area);
    mascot_area
}

pub(super) fn render_line_banner(frame: &mut Frame, area: Rect, palette: Palette) {
    let lines = vec![
        Line::styled("RunHaven", palette.accent()),
        Line::styled(format!("v{}", env!("CARGO_PKG_VERSION")), palette.muted()),
        Line::styled("run agents in a safe haven", palette.muted()),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

pub(super) fn render_launcher_summary(
    frame: &mut Frame,
    area: Rect,
    selected: Option<&AgentProfile>,
    launcher: &launcher::LauncherState,
    settings: TuiSettings,
    palette: Palette,
) {
    let agent_line = selected.map_or_else(
        || "agent: none".to_string(),
        |agent| {
            format!(
                "agent: {}  default network: {}",
                agent.name,
                default_network_mode(agent).as_str()
            )
        },
    );
    let mut lines = vec![
        Line::styled(
            truncate_to_width(
                &format!("workspace: {}", launcher.workspace.display()),
                area.width.saturating_sub(2) as usize,
            ),
            palette.text(),
        ),
        Line::styled(
            truncate_to_width(&agent_line, area.width.saturating_sub(2) as usize),
            palette.text(),
        ),
    ];
    if let Some(message) = &launcher.plan_error {
        lines.push(Line::styled(
            truncate_to_width(
                &format!("message: {message}"),
                area.width.saturating_sub(2) as usize,
            ),
            palette.muted(),
        ));
    }

    let mut paragraph = Paragraph::new(Text::from(lines)).style(palette.text());
    if !settings.line_mode {
        paragraph = paragraph.block(
            Block::bordered()
                .title(" Launcher ")
                .border_style(palette.border()),
        );
    }
    frame.render_widget(paragraph, area);
}

pub(super) fn render_screen_title(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    settings: TuiSettings,
    palette: Palette,
) {
    let mut paragraph =
        Paragraph::new(Line::styled(title.to_string(), palette.accent())).centered();
    if !settings.line_mode {
        paragraph = paragraph.block(Block::bordered().border_style(palette.border()));
    }
    frame.render_widget(paragraph, area);
}

pub(super) fn render_screen_body(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    lines: Vec<Line<'static>>,
    settings: TuiSettings,
    palette: Palette,
) {
    let mut paragraph = Paragraph::new(Text::from(lines)).style(palette.text());
    if !settings.line_mode {
        paragraph = paragraph.block(
            Block::bordered()
                .title(title)
                .border_style(palette.border()),
        );
    }
    frame.render_widget(paragraph, area);
}

pub(super) fn workspace_candidate_item(
    candidate: &launcher::WorkspaceCandidate,
    width: usize,
    palette: Palette,
) -> ListItem<'static> {
    let label_width = 18;
    let detail_width = width.saturating_sub(label_width + 4);
    ListItem::new(Line::from(vec![
        Span::styled(
            format!(
                "{:<label_width$}",
                candidate.label,
                label_width = label_width
            ),
            palette.text(),
        ),
        Span::styled(
            truncate_to_width(&candidate.detail, detail_width),
            palette.muted(),
        ),
    ]))
}

pub(super) fn plan_review_lines(
    review: &launcher::PlanReview,
    width: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    let plan = &review.plan;
    let mut lines = Vec::new();
    push_wrapped_line(
        &mut lines,
        format!(
            "Workspace mount: {} -> /workspace",
            plan.workspace.display()
        ),
        palette.text(),
        width,
    );
    if let Some(note) = &plan.workspace_scope_note {
        push_wrapped_line(
            &mut lines,
            format!("Workspace note: {note}"),
            palette.muted(),
            width,
        );
    }
    push_wrapped_line(
        &mut lines,
        format!("Agent home state: {} -> /home/agent", plan.state_volume),
        palette.text(),
        width,
    );
    push_wrapped_line(
        &mut lines,
        format!("Network mode: {}", plan.network_mode.as_str()),
        palette.text(),
        width,
    );
    push_wrapped_line(
        &mut lines,
        format!("Egress: {}", plan.egress_summary),
        palette.text(),
        width,
    );
    if !plan.provider_allowed_hosts.is_empty() {
        push_wrapped_line(
            &mut lines,
            format!("Provider hosts: {}", plan.provider_allowed_hosts.join(", ")),
            palette.muted(),
            width,
        );
    }
    push_wrapped_line(
        &mut lines,
        "Will not mount: host home, cloud credential folders, raw SSH keys, browser profiles.",
        palette.accent(),
        width,
    );
    lines.push(Line::from(""));
    push_wrapped_line(
        &mut lines,
        format!("CLI: {}", review.cli_command),
        palette.muted(),
        width,
    );
    if !plan.security_notices.is_empty() {
        lines.push(Line::from(""));
        push_wrapped_line(&mut lines, "Security notices:", palette.accent(), width);
        for notice in &plan.security_notices {
            push_wrapped_line(&mut lines, format!("- {notice}"), palette.muted(), width);
        }
    }
    lines
}

pub(super) fn run_list_item(
    run: &runs::RunSummary,
    width: usize,
    palette: Palette,
) -> ListItem<'static> {
    let head = format!(
        "{:<10} {:<12} {:<9} {:<14}",
        short_run_id(&run.run_id),
        run.profile,
        run.network,
        run.marker_status
    );
    let detail_width = width.saturating_sub(46);
    ListItem::new(Line::from(vec![
        Span::styled(head, palette.text()),
        Span::styled(
            truncate_to_width(&run.workspace, detail_width),
            palette.muted(),
        ),
    ]))
}

pub(super) fn run_status_lines(
    status: Option<&runs::RunStatus>,
    error: Option<&str>,
    width: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(error) = error {
        push_wrapped_line(
            &mut lines,
            format!("Status unavailable: {error}"),
            palette.muted(),
            width,
        );
        return lines;
    }
    let Some(status) = status else {
        push_wrapped_line(
            &mut lines,
            "Select an active run and press r to refresh status.",
            palette.muted(),
            width,
        );
        return lines;
    };
    for line in [
        format!("Marker: {}", status.marker_status),
        format!("Container: {}", status.container_state),
        format!("Started: {}", status.started_at),
        format!("Image: {}", status.image),
        format!("Resources: {}", status.resources),
    ] {
        push_wrapped_line(&mut lines, line, palette.text(), width);
    }
    for network in &status.networks {
        push_wrapped_line(
            &mut lines,
            format!("Network: {network}"),
            palette.muted(),
            width,
        );
    }
    lines
}

pub(super) fn egress_ledger_lines(
    decisions: &[runs::EgressDecision],
    error: Option<&str>,
    width: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(error) = error {
        push_wrapped_line(
            &mut lines,
            format!("Egress ledger unavailable: {error}"),
            palette.muted(),
            width,
        );
        return lines;
    }
    push_wrapped_line(
        &mut lines,
        format!("Summary: {}", runs::summarize_egress(decisions)),
        palette.text(),
        width,
    );
    if decisions.is_empty() {
        push_wrapped_line(
            &mut lines,
            "Provider decisions appear here while provider-mode runs log decisions.",
            palette.muted(),
            width,
        );
        return lines;
    }
    for decision in decisions.iter().rev().take(5).rev() {
        let style = if decision.decision == "denied" {
            palette.accent()
        } else {
            palette.muted()
        };
        push_wrapped_line(
            &mut lines,
            format!(
                "{} {}:{} count={} reason={} rule={}",
                decision.decision,
                decision.host,
                decision.port,
                decision.count,
                decision.reason,
                decision.matched_rule
            ),
            style,
            width,
        );
    }
    lines
}

pub(super) fn log_view_lines(
    viewer: &runs::LogViewerState,
    width: u16,
    height: u16,
    palette: Palette,
) -> Vec<Line<'static>> {
    if let Some(error) = &viewer.error {
        let mut lines = Vec::new();
        push_wrapped_line(
            &mut lines,
            format!("Log snapshot unavailable: {error}"),
            palette.muted(),
            usize::from(width),
        );
        return lines;
    }
    let mut lines = viewer
        .visible_lines(width, height)
        .into_iter()
        .map(|line| {
            if line.matched {
                Line::styled(line.text, palette.accent())
            } else {
                Line::styled(line.text, palette.text())
            }
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        push_wrapped_line(
            &mut lines,
            "Press r to load a bounded raw-output snapshot for the selected run.",
            palette.muted(),
            usize::from(width),
        );
    }
    lines
}

pub(super) fn log_header_lines(
    viewer: &runs::LogViewerState,
    width: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let summary = viewer.snapshot.as_ref().map_or_else(
        || format!("snapshot: not loaded  lines={}", viewer.requested_lines),
        |snapshot| {
            format!(
                "snapshot: {} returned / {} requested  truncated={}  tail={}",
                snapshot.returned_lines, snapshot.requested_lines, snapshot.truncated, viewer.tail
            )
        },
    );
    push_wrapped_line(&mut lines, summary, palette.text(), width);
    push_wrapped_line(
        &mut lines,
        format!("search: {}", viewer.search),
        palette.muted(),
        width,
    );
    if let Some(snapshot) = &viewer.snapshot {
        for warning in &snapshot.warnings {
            push_wrapped_line(
                &mut lines,
                format!("warning: {warning}"),
                palette.accent(),
                width,
            );
        }
    }
    lines
}

pub(super) fn push_wrapped_line(
    lines: &mut Vec<Line<'static>>,
    text: impl AsRef<str>,
    style: Style,
    width: usize,
) {
    let width = width.saturating_sub(4).max(1);
    for line in textwrap::wrap(text.as_ref(), width) {
        lines.push(Line::styled(line.into_owned(), style));
    }
}

pub(super) fn render_footer(
    frame: &mut Frame,
    area: Rect,
    hints: &str,
    tip: &str,
    palette: Palette,
) {
    let [hint_area, tip_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
    let hint = truncate_to_width(hints, hint_area.width as usize);
    let tip = truncate_to_width(tip, tip_area.width as usize);
    frame.render_widget(
        Paragraph::new(Line::styled(hint, palette.muted())).centered(),
        hint_area,
    );
    frame.render_widget(
        Paragraph::new(Line::styled(tip, palette.muted())).centered(),
        tip_area,
    );
}

pub(super) fn agent_list_item(profile: &AgentProfile, width: usize) -> ListItem<'static> {
    const NAME_WIDTH: usize = 12;
    const GAP: usize = 2;
    let prefix_width = NAME_WIDTH + GAP;
    let description_width = width.saturating_sub(prefix_width);
    let description = truncate_to_width(profile.description, description_width);
    ListItem::new(format!(
        "{:<name_width$}  {}",
        profile.name,
        description,
        name_width = NAME_WIDTH
    ))
}

pub(super) fn truncate_to_width(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let suffix = "...";
    let body_width = max_width - suffix.len();
    let mut out = String::new();
    let mut width = 0;
    for ch in text.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if width + ch_width > body_width {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push_str(suffix);
    out
}

fn short_run_id(run_id: &str) -> String {
    truncate_to_width(run_id, 8)
}

/// The shared three-row layout: a header, a flexible body, and a two-line footer.
pub(super) fn layout(frame: &Frame) -> [Rect; 3] {
    Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(frame.area())
}
