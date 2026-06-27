use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, ListState};

use super::history::{self, HistoryState, RunHistorySummary};
use super::theme::{Palette, TuiSettings};
use super::widgets::{
    layout, push_wrapped_line, render_footer, render_screen_body, render_screen_title,
    truncate_to_width,
};

pub(super) fn render_history(
    frame: &mut Frame,
    history: &HistoryState,
    settings: TuiSettings,
    palette: Palette,
) {
    let [header, body, footer] = layout(frame);
    render_screen_title(frame, header, "Run History", settings, palette);

    let [list_area, detail_area] =
        Layout::vertical([Constraint::Length(9), Constraint::Min(0)]).areas(body);
    let items = history
        .records
        .iter()
        .map(|record| history_item(record, list_area.width as usize, palette))
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(history.selected));
    }
    let mut list = List::new(items)
        .highlight_symbol("> ")
        .highlight_style(palette.selected())
        .style(palette.text());
    if !settings.line_mode {
        list = list.block(
            Block::bordered()
                .title(" Records ")
                .border_style(palette.border()),
        );
    }
    frame.render_stateful_widget(list, list_area, &mut state);

    let detail = history_overview_lines(history, detail_area.width as usize, palette);
    render_screen_body(frame, detail_area, " Summary ", detail, settings, palette);
    render_footer(
        frame,
        footer,
        "up/down select | enter diff | r refresh | g checks | esc home",
        "Run history is loaded from secret-free RunHaven records.",
        palette,
    );
}

pub(super) fn render_history_detail(
    frame: &mut Frame,
    history: &HistoryState,
    settings: TuiSettings,
    palette: Palette,
) {
    let [header, body, footer] = layout(frame);
    render_screen_title(frame, header, "Run Diff", settings, palette);
    let [meta_area, diff_area] =
        Layout::vertical([Constraint::Length(7), Constraint::Min(0)]).areas(body);
    render_screen_body(
        frame,
        meta_area,
        " Run ",
        selected_history_lines(history, meta_area.width as usize, palette),
        settings,
        palette,
    );

    let inner_height = diff_area
        .height
        .saturating_sub(if settings.line_mode { 0 } else { 2 });
    let lines = diff_lines(history, diff_area.width as usize, inner_height, palette);
    render_screen_body(frame, diff_area, " Diff ", lines, settings, palette);
    render_footer(
        frame,
        footer,
        "up/down scroll | r reload diff | esc history | q quit",
        "Diff review verifies the recorded repo, HEAD, and path set before reading.",
        palette,
    );
}

pub(super) fn render_diagnostics(
    frame: &mut Frame,
    history: &HistoryState,
    settings: TuiSettings,
    palette: Palette,
) {
    let [header, body, footer] = layout(frame);
    render_screen_title(frame, header, "Checks", settings, palette);
    let [auth_status_area, terminal_area, egress_area, auth_log_area] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(4),
        Constraint::Length(7),
        Constraint::Min(0),
    ])
    .areas(body);

    render_screen_body(
        frame,
        auth_status_area,
        " Auth Status ",
        auth_status_lines(history, auth_status_area.width as usize, palette),
        settings,
        palette,
    );
    render_screen_body(
        frame,
        terminal_area,
        " Terminal ",
        terminal_probe_lines(history, terminal_area.width as usize, palette),
        settings,
        palette,
    );
    render_screen_body(
        frame,
        egress_area,
        " Network Log ",
        diagnostic_egress_lines(history, egress_area.width as usize, palette),
        settings,
        palette,
    );
    render_screen_body(
        frame,
        auth_log_area,
        " Auth Log ",
        diagnostic_auth_lines(history, auth_log_area.width as usize, palette),
        settings,
        palette,
    );
    render_footer(
        frame,
        footer,
        "r refresh | d doctor | h history | esc home | q quit",
        "Diagnostics show metadata only, not secrets or workspace file contents.",
        palette,
    );
}

pub(super) fn render_doctor(
    frame: &mut Frame,
    history: &HistoryState,
    settings: TuiSettings,
    palette: Palette,
    ticks: u64,
) {
    let [header, body, footer] = layout(frame);
    render_screen_title(frame, header, "Doctor", settings, palette);
    let spinner = ["|", "/", "-", "\\"][(ticks as usize) % 4];
    let mut lines = Vec::new();
    push_wrapped_line(
        &mut lines,
        format!(
            "{spinner} Prerequisites: {}",
            history::doctor_summary(&history.doctor.checks)
        ),
        palette.accent(),
        body.width as usize,
    );
    if let Some(error) = &history.doctor.error {
        push_wrapped_line(
            &mut lines,
            format!("Doctor unavailable: {error}"),
            palette.muted(),
            body.width as usize,
        );
    }
    if history.doctor.checks.is_empty() && history.doctor.error.is_none() {
        push_wrapped_line(
            &mut lines,
            "Press r to run host prerequisite checks.",
            palette.muted(),
            body.width as usize,
        );
    }
    for check in &history.doctor.checks {
        let status = if check.ok { "ok" } else { "fix" };
        let style = if check.ok {
            palette.text()
        } else {
            palette.accent()
        };
        push_wrapped_line(
            &mut lines,
            format!("{status} {}: {}", check.name, check.detail),
            style,
            body.width as usize,
        );
        if !check.ok {
            push_wrapped_line(
                &mut lines,
                format!("remedy: {}", check.remedy),
                palette.muted(),
                body.width as usize,
            );
        }
    }
    render_screen_body(frame, body, " Checks ", lines, settings, palette);
    render_footer(
        frame,
        footer,
        "r rerun | g checks | esc home | q quit",
        "Checks do not change anything and show the smallest fix.",
        palette,
    );
}

fn history_item(record: &RunHistorySummary, width: usize, palette: Palette) -> ListItem<'static> {
    let status = format!(
        "{} {} return={} denied={}/{}",
        record.profile,
        record.status,
        record.return_code,
        record.provider_denied,
        record.auth_denied
    );
    let left = truncate_to_width(&record.timestamp, 20);
    let right_width = width.saturating_sub(24);
    let right = truncate_to_width(&format!("{status} {}", record.workspace), right_width);
    ListItem::new(Line::styled(format!("{left:<20}  {right}"), palette.text()))
}

fn history_overview_lines(
    history: &HistoryState,
    width: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(error) = &history.error {
        push_wrapped_line(
            &mut lines,
            format!("History unavailable: {error}"),
            palette.muted(),
            width,
        );
        return lines;
    }
    if history.records.is_empty() {
        push_wrapped_line(
            &mut lines,
            "No run records found yet. Launch a run, then return here.",
            palette.muted(),
            width,
        );
        return lines;
    }
    lines.extend(selected_history_lines(history, width, palette));
    push_wrapped_line(
        &mut lines,
        "Press enter to review the recorded diff for this run.",
        palette.muted(),
        width,
    );
    lines
}

fn selected_history_lines(
    history: &HistoryState,
    width: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let Some(record) = history.selected_record() else {
        push_wrapped_line(&mut lines, "No run selected.", palette.muted(), width);
        return lines;
    };
    for line in [
        format!("Run: {}", record.run_id),
        format!(
            "Profile/status: {} {} return={}",
            record.profile, record.status, record.return_code
        ),
        format!("Network: {} cleanup={}", record.network, record.cleanup),
        format!(
            "Provider denied: {}  auth denied: {}",
            record.provider_denied, record.auth_denied
        ),
        format!("Workspace: {}", record.workspace),
        record.git_summary.clone(),
    ] {
        push_wrapped_line(&mut lines, line, palette.text(), width);
    }
    lines
}

fn diff_lines(
    history: &HistoryState,
    width: usize,
    height: u16,
    palette: Palette,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(error) = &history.detail.diff_error {
        push_wrapped_line(
            &mut lines,
            format!("Diff unavailable: {error}"),
            palette.muted(),
            width,
        );
        return lines;
    }
    let visible = history::visible_diff_lines(&history.detail, height);
    if visible.is_empty() {
        push_wrapped_line(
            &mut lines,
            "Press r to reload the recorded diff.",
            palette.muted(),
            width,
        );
        return lines;
    }
    for line in visible {
        let style = if line.starts_with('+') {
            palette.accent()
        } else if line.starts_with('-') {
            palette.muted()
        } else {
            palette.text()
        };
        push_wrapped_line(&mut lines, line, style, width);
    }
    lines
}

fn auth_status_lines(history: &HistoryState, width: usize, palette: Palette) -> Vec<Line<'static>> {
    let status = &history.diagnostics.auth_status;
    let mut lines = Vec::new();
    for line in [
        format!("Auth broker: {} runtime={}", status.status, status.runtime),
        format!(
            "Credential stores inspected={} env values inspected={} secrets printed={}",
            status.credential_stores_inspected,
            status.environment_values_inspected,
            status.secrets_printed
        ),
        format!(
            "Profiles: {}",
            status
                .profiles
                .iter()
                .map(|profile| format!("{}:{}:{}", profile.agent, profile.broker, profile.status))
                .collect::<Vec<_>>()
                .join(" ")
        ),
    ] {
        push_wrapped_line(&mut lines, line, palette.text(), width);
    }
    lines
}

fn terminal_probe_lines(
    history: &HistoryState,
    width: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    let probe = &history.diagnostics.terminal;
    let mut lines = Vec::new();
    for line in [
        format!("Color: {}  motion: {}", probe.color, probe.motion),
        format!(
            "Line mode: {}  terminal image: {}",
            probe.line_mode, probe.terminal_image
        ),
    ] {
        push_wrapped_line(&mut lines, line, palette.text(), width);
    }
    lines
}

fn diagnostic_egress_lines(
    history: &HistoryState,
    width: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(error) = &history.diagnostics.egress_error {
        push_wrapped_line(
            &mut lines,
            format!("Network log unavailable: {error}"),
            palette.muted(),
            width,
        );
        return lines;
    }
    if history.diagnostics.egress.is_empty() {
        push_wrapped_line(
            &mut lines,
            "No provider network log entries found.",
            palette.muted(),
            width,
        );
        return lines;
    }
    for entry in history.diagnostics.egress.iter().take(4) {
        push_wrapped_line(
            &mut lines,
            format!(
                "{} {} {}:{} count={} reason={} run={}",
                entry.profile,
                entry.decision,
                entry.host,
                entry.port,
                entry.count,
                entry.reason,
                entry.run_id
            ),
            if entry.decision == "denied" {
                palette.accent()
            } else {
                palette.text()
            },
            width,
        );
    }
    lines
}

fn diagnostic_auth_lines(
    history: &HistoryState,
    width: usize,
    palette: Palette,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(error) = &history.diagnostics.auth_error {
        push_wrapped_line(
            &mut lines,
            format!("Auth log unavailable: {error}"),
            palette.muted(),
            width,
        );
        return lines;
    }
    if history.diagnostics.auth.is_empty() {
        push_wrapped_line(
            &mut lines,
            "No auth broker log entries found.",
            palette.muted(),
            width,
        );
        return lines;
    }
    for entry in history.diagnostics.auth.iter().take(5) {
        push_wrapped_line(
            &mut lines,
            format!(
                "{} {} {} {} {} status={} count={} run={}",
                entry.profile,
                entry.broker,
                entry.decision,
                entry.method,
                entry.path,
                entry.upstream_status,
                entry.count,
                entry.run_id
            ),
            if entry.decision == "denied" {
                palette.accent()
            } else {
                palette.text()
            },
            width,
        );
    }
    lines
}
