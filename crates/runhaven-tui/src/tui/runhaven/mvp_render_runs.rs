use super::*;

pub(super) fn render_active_runs(screen: &ActiveRunsScreen, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, active_runs_lines(screen));
}

pub(super) fn active_runs_lines(screen: &ActiveRunsScreen) -> Vec<Line<'static>> {
    let mut lines = vec![header_line("Active runs"), tab_line(), Line::from("")];
    if screen.runs.runs.is_empty() {
        lines.push(Line::from("No active RunHaven runs found."));
        lines.push(Line::from(
            "Launch an agent or refresh after another terminal starts one.",
        ));
        return lines;
    }
    lines.push(Line::from(
        "Enter opens logs. s Stop. K Hard stop. x Repair marker. r Refresh.",
    ));
    lines.push(Line::from(""));
    for (idx, run) in screen.runs.runs.iter().enumerate() {
        let selected = idx == screen.selected_idx;
        lines.push(Line::from(vec![
            Span::styled(if selected { "> " } else { "  " }, accent_style()),
            Span::styled(
                run.run_id.clone(),
                if selected {
                    selected_row_style()
                } else {
                    boundary_style()
                },
            ),
            Span::raw("  "),
            Span::styled(run.profile.clone(), accent_style()),
            Span::raw("  "),
            Span::styled(run.network.clone(), network_style(&run.network)),
            Span::raw("  "),
            Span::styled(run.status.clone(), status_style(&run.status)),
        ]));
    }
    if let Some(run) = screen.selected_run() {
        lines.push(Line::from(""));
        lines.push(label_value("Profile", run.profile.clone(), accent_style()));
        lines.push(label_value("Run ID", run.run_id.clone(), boundary_style()));
        lines.push(label_value(
            "Container",
            run.container_name.clone(),
            muted_but_readable_style(),
        ));
        lines.push(label_value("State", run.state_volume.clone(), safe_style()));
        lines.push(label_value(
            "Session",
            run.session.clone(),
            muted_but_readable_style(),
        ));
        lines.push(label_value(
            "Started",
            run.timestamp.clone(),
            muted_but_readable_style(),
        ));
    }
    if let Some(notice) = &screen.notice {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(notice.clone(), warning_style())));
    }
    lines
}

pub(super) fn render_run_control(screen: &RunControlScreen, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, run_control_lines(screen));
}

pub(super) fn run_control_lines(screen: &RunControlScreen) -> Vec<Line<'static>> {
    let mut lines = vec![
        header_line(screen.action.title()),
        tab_line(),
        Line::from("This action only targets the selected RunHaven-owned container."),
        Line::from(""),
        label_value("Run ID", screen.run.run_id.clone(), boundary_style()),
        label_value("Profile", screen.run.profile.clone(), accent_style()),
        label_value(
            "Container",
            screen.run.container_name.clone(),
            muted_but_readable_style(),
        ),
        label_value(
            "Status",
            screen.run.status.clone(),
            status_style(&screen.run.status),
        ),
        Line::from(""),
    ];

    match &screen.state {
        RunControlState::Confirm { typed, notice } => {
            lines.push(Line::from(Span::styled(
                run_control_warning(screen.action),
                warning_style(),
            )));
            lines.push(Line::from(vec![
                Span::raw("Type "),
                Span::styled(screen.action.phrase(), selected_row_style()),
                Span::raw(", then press Enter."),
            ]));
            lines.push(label_value(
                "Typed",
                typed.clone(),
                muted_but_readable_style(),
            ));
            if let Some(notice) = notice {
                lines.push(Line::from(Span::styled(notice.clone(), warning_style())));
            }
        }
        RunControlState::Complete(result) => {
            lines.push(Line::from(Span::styled(
                result.message.clone(),
                result_style(&result.status),
            )));
            lines.push(label_value(
                "Result",
                result.status.clone(),
                result_style(&result.status),
            ));
            if let Some(code) = result.return_code {
                lines.push(label_value(
                    "Exit code",
                    code.to_string(),
                    if code == 0 {
                        safe_style()
                    } else {
                        warning_style()
                    },
                ));
            }
            if let Some(marker_removed) = result.marker_removed {
                lines.push(label_value(
                    "Marker",
                    if marker_removed { "removed" } else { "kept" },
                    if marker_removed {
                        safe_style()
                    } else {
                        warning_style()
                    },
                ));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(
                "Press r to refresh active runs, or Esc to go back.",
            ));
        }
        RunControlState::Error(message) => {
            lines.push(Line::from(Span::styled(
                "Run control failed.",
                danger_style(),
            )));
            lines.push(Line::from(message.clone()));
            lines.push(Line::from(""));
            lines.push(Line::from(
                "Press r to refresh active runs, or Esc to go back.",
            ));
        }
    }
    lines
}

fn run_control_warning(action: RunControlAction) -> &'static str {
    match action {
        RunControlAction::Stop => {
            "Stop asks the container to exit cleanly. Work inside the agent may stop."
        }
        RunControlAction::Kill => {
            "Hard stop kills the container now. Unsaved work inside the agent may be lost."
        }
        RunControlAction::Repair => {
            "Repair checks whether the container still exists before changing the marker."
        }
    }
}

fn result_style(status: &str) -> Style {
    match status {
        "requested" | "removed" => safe_style(),
        "kept" => warning_style(),
        _ => danger_style(),
    }
}

pub(super) fn render_run_logs(screen: &RunLogsScreen, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, run_logs_lines(screen));
}

pub(super) fn run_logs_lines(screen: &RunLogsScreen) -> Vec<Line<'static>> {
    let mut lines = vec![
        header_line("Run logs"),
        Line::from(vec![
            Span::styled("Run ID  ", muted_but_readable_style()),
            Span::styled(screen.run.run_id.clone(), boundary_style()),
            Span::raw("  "),
            Span::styled("Profile  ", muted_but_readable_style()),
            Span::styled(screen.run.profile.clone(), accent_style()),
        ]),
        Line::from(""),
    ];

    match &screen.state {
        RunLogsState::Confirm { typed, notice } => {
            lines.push(Line::from(vec![Span::styled(
                "Raw container output can contain secrets or workspace content.",
                warning_style(),
            )]));
            lines.push(Line::from(vec![
                Span::raw("Type "),
                Span::styled(LOG_CONFIRM_PHRASE, selected_row_style()),
                Span::raw(", then press Enter to load a bounded snapshot."),
            ]));
            lines.push(label_value(
                "Typed",
                typed.clone(),
                muted_but_readable_style(),
            ));
            if let Some(notice) = notice {
                lines.push(Line::from(Span::styled(notice.clone(), warning_style())));
            }
        }
        RunLogsState::Loaded(snapshot) => {
            lines.push(label_value(
                "Captured",
                snapshot.captured_at.clone(),
                muted_but_readable_style(),
            ));
            lines.push(label_value(
                "Returned",
                format!("{} lines", snapshot.returned_lines),
                safe_style(),
            ));
            if snapshot.truncated {
                lines.push(Line::from(Span::styled(
                    "Output was truncated to the bounded snapshot size.",
                    warning_style(),
                )));
            }
            for warning in &snapshot.warnings {
                lines.push(Line::from(Span::styled(warning.clone(), warning_style())));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Snapshot",
                selected_row_style(),
            )]));
            if snapshot.text.is_empty() {
                lines.push(Line::from("(no output returned)"));
            } else {
                for line in snapshot.text.lines().take(120) {
                    lines.push(Line::from(line.to_string()));
                }
            }
        }
        RunLogsState::Error(message) => {
            lines.push(Line::from(Span::styled(
                "Could not load run logs.",
                danger_style(),
            )));
            lines.push(Line::from(message.clone()));
        }
    }
    lines
}

pub(super) fn render_run_diff(screen: &RunDiffScreen, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, run_diff_lines(screen));
}

pub(super) fn run_diff_lines(screen: &RunDiffScreen) -> Vec<Line<'static>> {
    let mut lines = vec![
        header_line("Run review"),
        Line::from(vec![
            Span::styled("Run ID  ", muted_but_readable_style()),
            Span::styled(screen.run.run_id.clone(), boundary_style()),
            Span::raw("  "),
            Span::styled("Profile  ", muted_but_readable_style()),
            Span::styled(screen.run.profile.clone(), accent_style()),
        ]),
        label_value(
            "Git",
            screen.run.git_summary.clone(),
            muted_but_readable_style(),
        ),
        label_value(
            "Fallback",
            screen.run.review_command.clone(),
            boundary_style(),
        ),
        Line::from(""),
    ];

    match &screen.state {
        RunDiffState::Confirm { typed, notice } => {
            lines.push(Line::from(Span::styled(
                "The diff can show workspace file contents.",
                warning_style(),
            )));
            lines.push(Line::from(vec![
                Span::raw("Type "),
                Span::styled(DIFF_CONFIRM_PHRASE, selected_row_style()),
                Span::raw(", then press Enter to load a bounded preview."),
            ]));
            lines.push(label_value(
                "Typed",
                typed.clone(),
                muted_but_readable_style(),
            ));
            if let Some(notice) = notice {
                lines.push(Line::from(Span::styled(notice.clone(), warning_style())));
            }
        }
        RunDiffState::Loaded(diff) => append_loaded_diff(&mut lines, diff),
        RunDiffState::Error(message) => {
            lines.push(Line::from(Span::styled(
                "Could not load run review.",
                danger_style(),
            )));
            lines.push(Line::from(message.clone()));
            lines.push(Line::from(""));
            lines.push(Line::from(
                "Use the fallback command above if you need the full record.",
            ));
        }
    }
    lines
}

fn append_loaded_diff(lines: &mut Vec<Line<'static>>, diff: &RunDiffData) {
    lines.push(label_value(
        "Returned",
        format!("{} lines", diff.returned_lines),
        safe_style(),
    ));
    lines.push(label_value("Source", diff.source.clone(), safe_style()));
    for warning in &diff.warnings {
        lines.push(Line::from(Span::styled(warning.clone(), warning_style())));
    }
    if diff.truncated {
        lines.push(Line::from(Span::styled(
            "Only the first lines are shown here.",
            warning_style(),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Diff preview",
        selected_row_style(),
    )]));
    if diff.text.is_empty() {
        lines.push(Line::from("(no diff output returned)"));
    } else {
        for line in diff.text.lines().take(160) {
            lines.push(Line::from(line.to_string()));
        }
    }
}
