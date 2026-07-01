use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;
use runhaven_core::ui_contracts::AuthDecisionData;
use runhaven_core::ui_contracts::DoctorCheckData;
use runhaven_core::ui_contracts::EgressDecisionData;
use runhaven_core::ui_contracts::RunDiffData;
use runhaven_core::ui_contracts::RunHavenDiagnosticsData;
use runhaven_core::ui_contracts::RunHistorySummaryData;

use crate::key_hint;
use crate::render::renderable::Renderable;
use crate::style::accent_style;
use crate::style::boundary_style;
use crate::style::danger_style;
use crate::style::muted_but_readable_style;
use crate::style::safe_style;
use crate::style::selected_row_style;
use crate::style::warning_style;
use crate::tui::bottom_pane::render_menu_surface;

use super::ActiveRunsScreen;
use super::DIFF_CONFIRM_PHRASE;
use super::DiagnosticsScreen;
use super::HistoryScreen;
use super::LOG_CONFIRM_PHRASE;
use super::MvpScreen;
use super::PostRunOutcome;
use super::RunControlAction;
use super::RunControlScreen;
use super::RunControlState;
use super::RunDiffScreen;
use super::RunDiffState;
use super::RunHavenMvpView;
use super::RunLogsScreen;
use super::RunLogsState;

#[path = "mvp_render_diagnostics.rs"]
mod diagnostics;
#[path = "mvp_render_history.rs"]
mod history;
#[path = "mvp_render_runs.rs"]
mod runs;

pub(super) fn render(view: &RunHavenMvpView, area: Rect, buf: &mut Buffer) {
    match &view.screen {
        MvpScreen::Launch => view.launch.render(area, buf),
        MvpScreen::ActiveRuns(screen) => runs::render_active_runs(screen, area, buf),
        MvpScreen::RunLogs(screen) => runs::render_run_logs(screen, area, buf),
        MvpScreen::RunDiff(screen) => runs::render_run_diff(screen, area, buf),
        MvpScreen::RunControl(screen) => runs::render_run_control(screen, area, buf),
        MvpScreen::History(screen) => history::render_history(screen, area, buf),
        MvpScreen::Diagnostics(screen) => diagnostics::render_diagnostics(screen, area, buf),
        MvpScreen::PostRun(outcome) => render_post_run(outcome, area, buf),
    }
}

pub(super) fn desired_height(view: &RunHavenMvpView, width: u16) -> u16 {
    match &view.screen {
        MvpScreen::Launch => view.launch.desired_height(width),
        MvpScreen::ActiveRuns(screen) => {
            paragraph(runs::active_runs_lines(screen)).line_count(width.saturating_sub(4).max(1))
                as u16
                + 2
        }
        MvpScreen::RunLogs(screen) => {
            paragraph(runs::run_logs_lines(screen)).line_count(width.saturating_sub(4).max(1))
                as u16
                + 2
        }
        MvpScreen::RunDiff(screen) => {
            paragraph(runs::run_diff_lines(screen)).line_count(width.saturating_sub(4).max(1))
                as u16
                + 2
        }
        MvpScreen::RunControl(screen) => {
            paragraph(runs::run_control_lines(screen)).line_count(width.saturating_sub(4).max(1))
                as u16
                + 2
        }
        MvpScreen::History(screen) => {
            paragraph(history::history_lines(screen)).line_count(width.saturating_sub(4).max(1))
                as u16
                + 2
        }
        MvpScreen::Diagnostics(screen) => {
            paragraph(diagnostics::diagnostics_lines(screen))
                .line_count(width.saturating_sub(4).max(1)) as u16
                + 2
        }
        MvpScreen::PostRun(outcome) => {
            paragraph(post_run_lines(outcome)).line_count(width.saturating_sub(4).max(1)) as u16 + 2
        }
    }
}

fn render_post_run(outcome: &PostRunOutcome, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, post_run_lines(outcome));
}

fn post_run_lines(outcome: &PostRunOutcome) -> Vec<Line<'static>> {
    let mut lines = vec![
        header_line("Run finished"),
        Line::from("The terminal is back in RunHaven."),
        Line::from(""),
        label_value("Agent", outcome.plan.profile_name.clone(), accent_style()),
        label_value(
            "Exit",
            outcome.exit_code.to_string(),
            exit_style(outcome.exit_code, outcome.error.as_ref()),
        ),
        label_value(
            "Network",
            outcome.plan.network.mode.clone(),
            network_style(&outcome.plan.network.mode),
        ),
        label_value("Auth scope", outcome.plan.auth_scope.clone(), safe_style()),
        label_value("Boundary", "/workspace only", boundary_style()),
        label_value("State", outcome.plan.state_volume.clone(), safe_style()),
    ];
    if let Some(error) = &outcome.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Launch error", danger_style())));
        lines.push(Line::from(error.clone()));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Enter starts another launch. 2 shows active runs. 3 shows diagnostics. q quits.",
    ));
    lines
}

fn render_panel(area: Rect, buf: &mut Buffer, lines: Vec<Line<'static>>) {
    Clear.render(area, buf);
    let content = render_menu_surface(area, buf);
    paragraph(lines).render(content, buf);
}

fn paragraph(lines: Vec<Line<'static>>) -> Paragraph<'static> {
    Paragraph::new(lines).wrap(Wrap { trim: false })
}

fn header_line(title: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled("RunHaven", selected_row_style()),
        Span::raw(format!(" v{}  ", env!("CARGO_PKG_VERSION"))),
        Span::styled(title, boundary_style()),
    ])
}

fn tab_line() -> Line<'static> {
    Line::from(vec![
        key_hint::plain(KeyCode::Char('1')).into(),
        Span::raw(" Launch  "),
        key_hint::plain(KeyCode::Char('2')).into(),
        Span::raw(" Runs  "),
        key_hint::plain(KeyCode::Char('3')).into(),
        Span::raw(" Diagnostics  "),
        key_hint::plain(KeyCode::Char('h')).into(),
        Span::raw(" History"),
    ])
}

fn label_value(label: &'static str, value: impl Into<String>, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<12}"), muted_but_readable_style()),
        Span::styled(value.into(), value_style),
    ])
}

fn network_style(network: &str) -> Style {
    if network == "internet" {
        warning_style()
    } else {
        safe_style()
    }
}

fn status_style(status: &str) -> Style {
    match status {
        "running" => safe_style(),
        "stop-requested" | "kill-requested" => warning_style(),
        _ => muted_but_readable_style(),
    }
}

pub(super) fn exit_style(exit_code: i32, error: Option<&String>) -> Style {
    if exit_code == 0 && error.is_none() {
        safe_style()
    } else {
        warning_style()
    }
}
