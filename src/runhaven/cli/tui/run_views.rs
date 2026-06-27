use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListState};

use super::runs;
use super::theme::{Palette, TuiSettings};
use super::widgets::{
    egress_ledger_lines, layout, log_header_lines, log_view_lines, push_wrapped_line,
    render_footer, render_screen_body, render_screen_title, run_list_item, run_status_lines,
};

pub(super) fn render_runs(
    frame: &mut Frame,
    run_manager: &runs::RunManagerState,
    settings: TuiSettings,
    palette: Palette,
) {
    let [header, body, footer] = layout(frame);
    render_screen_title(frame, header, "Run Dashboard", settings, palette);

    let [runs_area, status_area, egress_area] = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Min(0),
    ])
    .areas(body);

    let run_items = run_manager
        .runs
        .iter()
        .map(|run| run_list_item(run, runs_area.width as usize, palette))
        .collect::<Vec<_>>();
    let mut run_state = ListState::default();
    if !run_items.is_empty() {
        run_state.select(Some(run_manager.selected));
    }
    let mut run_list = List::new(run_items)
        .highlight_symbol("> ")
        .highlight_style(palette.selected())
        .style(palette.text());
    if !settings.line_mode {
        run_list = run_list.block(
            Block::bordered()
                .title(" Active Runs ")
                .border_style(palette.border()),
        );
    }
    frame.render_stateful_widget(run_list, runs_area, &mut run_state);

    let notifications = run_manager.notifications();
    let mut status_lines = Vec::new();
    if !notifications.is_empty() {
        for notification in notifications {
            push_wrapped_line(
                &mut status_lines,
                format!("Notice: {notification}"),
                palette.accent(),
                status_area.width as usize,
            );
        }
        status_lines.push(Line::from(""));
    }
    status_lines.extend(if run_manager.runs.is_empty() {
        let mut lines = Vec::new();
        push_wrapped_line(
            &mut lines,
            "No active RunHaven runs found.",
            palette.muted(),
            status_area.width as usize,
        );
        lines
    } else {
        run_status_lines(
            run_manager.status.as_ref(),
            run_manager.status_error.as_deref(),
            status_area.width as usize,
            palette,
        )
    });
    if let Some(message) = &run_manager.message {
        status_lines.push(Line::from(""));
        push_wrapped_line(
            &mut status_lines,
            format!("Message: {message}"),
            palette.accent(),
            status_area.width as usize,
        );
    }
    render_screen_body(
        frame,
        status_area,
        " Status ",
        status_lines,
        settings,
        palette,
    );

    render_screen_body(
        frame,
        egress_area,
        " Network Log ",
        egress_ledger_lines(
            &run_manager.egress,
            run_manager.egress_error.as_deref(),
            egress_area.width as usize,
            palette,
        ),
        settings,
        palette,
    );

    render_footer(
        frame,
        footer,
        "up/down select · r refresh · enter/l logs · s stop · x kill · e repair · esc home",
        "Log snapshots are bounded and explicit because raw output can contain secrets.",
        palette,
    );
}

pub(super) fn render_logs(
    frame: &mut Frame,
    run_manager: &runs::RunManagerState,
    settings: TuiSettings,
    palette: Palette,
) {
    let [header, body, footer] = layout(frame);
    render_screen_title(frame, header, "Run Logs", settings, palette);
    let [meta_area, log_area] =
        Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).areas(body);
    render_screen_body(
        frame,
        meta_area,
        " Snapshot ",
        log_header_lines(&run_manager.logs, meta_area.width as usize, palette),
        settings,
        palette,
    );
    let inner_height = log_area
        .height
        .saturating_sub(if settings.line_mode { 0 } else { 2 });
    let lines = log_view_lines(
        &run_manager.logs,
        log_area
            .width
            .saturating_sub(if settings.line_mode { 0 } else { 2 }),
        inner_height,
        palette,
    );
    render_screen_body(frame, log_area, " Output ", lines, settings, palette);
    let hints = if run_manager.logs.search_editing {
        "type search · enter done · backspace edit · esc done"
    } else {
        "/ search · r reload · up/down scroll · t tail · esc dashboard · q quit"
    };
    render_footer(
        frame,
        footer,
        hints,
        "ANSI output is rendered through a parser, not replayed into the terminal.",
        palette,
    );
}

pub(super) fn render_control(
    frame: &mut Frame,
    run_manager: &runs::RunManagerState,
    settings: TuiSettings,
    palette: Palette,
) {
    let [header, body, footer] = layout(frame);
    render_screen_title(frame, header, "Confirm Control", settings, palette);
    let mut lines = Vec::new();
    if let Some(dialog) = &run_manager.control {
        push_wrapped_line(
            &mut lines,
            format!("Action: {}", dialog.action.label()),
            palette.accent(),
            body.width as usize,
        );
        push_wrapped_line(
            &mut lines,
            format!("Run: {}", dialog.run_id),
            palette.text(),
            body.width as usize,
        );
        push_wrapped_line(
            &mut lines,
            format!("Container: {}", dialog.container_name),
            palette.text(),
            body.width as usize,
        );
        push_wrapped_line(
            &mut lines,
            format!("This {}.", dialog.action.description()),
            palette.muted(),
            body.width as usize,
        );
        lines.push(Line::from(""));
        push_wrapped_line(
            &mut lines,
            format!("Type {} to confirm.", dialog.action.phrase()),
            palette.accent(),
            body.width as usize,
        );
        push_wrapped_line(
            &mut lines,
            format!("confirm: {}", dialog.input),
            palette.text(),
            body.width as usize,
        );
    } else {
        push_wrapped_line(
            &mut lines,
            "No run-control action is active.",
            palette.muted(),
            body.width as usize,
        );
    }
    render_screen_body(frame, body, " Control ", lines, settings, palette);
    render_footer(
        frame,
        footer,
        "enter execute · esc cancel",
        "Run controls validate the active marker and RunHaven-owned container first.",
        palette,
    );
}
