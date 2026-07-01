use super::*;

pub(super) fn render_history(screen: &HistoryScreen, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, history_lines(screen));
}

pub(super) fn history_lines(screen: &HistoryScreen) -> Vec<Line<'static>> {
    let mut lines = vec![
        header_line("Run history"),
        tab_line(),
        Line::from("Recent run records. Host workspace paths are hidden in the TUI."),
        Line::from("Enter reviews the selected run diff after confirmation."),
        Line::from(""),
    ];
    match &screen.result {
        Ok(history) if history.runs.is_empty() => {
            lines.push(Line::from("No RunHaven run records found."));
            lines.push(Line::from(
                "Launch an agent, then return here after it exits.",
            ));
        }
        Ok(history) => {
            for (idx, run) in history.runs.iter().enumerate().take(10) {
                let selected = idx == screen.selected_idx;
                lines.push(history_row(run, selected));
            }
            if let Some(run) = screen.selected_run() {
                append_history_detail(&mut lines, run);
            }
        }
        Err(error) => {
            lines.push(Line::from(Span::styled(
                "Could not load run history.",
                danger_style(),
            )));
            lines.push(Line::from(error.clone()));
        }
    }
    if let Some(notice) = &screen.notice {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(notice.clone(), warning_style())));
    }
    lines
}

fn history_row(run: &RunHistorySummaryData, selected: bool) -> Line<'static> {
    Line::from(vec![
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
        Span::raw("  return="),
        Span::styled(format_return_code(run), muted_but_readable_style()),
    ])
}

fn append_history_detail(lines: &mut Vec<Line<'static>>, run: &RunHistorySummaryData) {
    lines.push(Line::from(""));
    lines.push(label_value("Run ID", run.run_id.clone(), boundary_style()));
    lines.push(label_value(
        "Started",
        run.started_at.clone(),
        muted_but_readable_style(),
    ));
    lines.push(label_value(
        "Finished",
        run.finished_at.clone(),
        muted_but_readable_style(),
    ));
    lines.push(label_value(
        "Scope",
        run.workspace_scope.clone(),
        safe_style(),
    ));
    lines.push(label_value("State", run.state_volume.clone(), safe_style()));
    lines.push(label_value(
        "Session",
        run.session.clone(),
        muted_but_readable_style(),
    ));
    lines.push(label_value(
        "Provider",
        format!(
            "allowed {} denied {}",
            run.provider_allowed, run.provider_denied
        ),
        if run.provider_denied == 0 {
            safe_style()
        } else {
            warning_style()
        },
    ));
    lines.push(label_value(
        "Auth",
        format!("allowed {} denied {}", run.auth_allowed, run.auth_denied),
        if run.auth_denied == 0 {
            safe_style()
        } else {
            warning_style()
        },
    ));
    lines.push(label_value(
        "Cleanup",
        run.cleanup_provider_network.clone(),
        muted_but_readable_style(),
    ));
    lines.push(label_value(
        "Git",
        run.git_summary.clone(),
        muted_but_readable_style(),
    ));
    if let Some(branch) = &run.worktree_branch {
        lines.push(label_value("Worktree", branch.clone(), warning_style()));
    }
    lines.push(Line::from(""));
    lines.push(label_value(
        "Review",
        run.review_command.clone(),
        boundary_style(),
    ));
}

fn format_return_code(run: &RunHistorySummaryData) -> String {
    run.return_code
        .map(|code| code.to_string())
        .unwrap_or_else(|| "-".to_string())
}
