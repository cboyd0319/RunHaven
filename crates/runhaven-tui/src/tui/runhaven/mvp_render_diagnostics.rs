use super::*;

pub(super) fn render_diagnostics(screen: &DiagnosticsScreen, area: Rect, buf: &mut Buffer) {
    render_panel(area, buf, diagnostics_lines(screen));
}

pub(super) fn diagnostics_lines(screen: &DiagnosticsScreen) -> Vec<Line<'static>> {
    let mut lines = vec![
        header_line("Diagnostics"),
        tab_line(),
        Line::from("Preflight status, auth metadata, and recent broker/proxy decisions."),
        Line::from(""),
    ];
    match &screen.result {
        Ok(data) => append_diagnostics_data(&mut lines, data),
        Err(error) => {
            lines.push(Line::from(Span::styled(
                "Could not load diagnostics.",
                danger_style(),
            )));
            lines.push(Line::from(error.clone()));
        }
    }
    lines
}

fn append_diagnostics_data(lines: &mut Vec<Line<'static>>, data: &RunHavenDiagnosticsData) {
    append_doctor_lines(lines, &data.doctor_checks);
    lines.push(label_value(
        "Auth",
        data.auth_status.status.clone(),
        safe_style(),
    ));
    lines.push(label_value(
        "Runtime",
        data.auth_status.runtime.clone(),
        muted_but_readable_style(),
    ));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Auth profiles",
        selected_row_style(),
    )]));
    for profile in data.auth_status.profiles.iter().take(8) {
        lines.push(Line::from(format!("{}  {}", profile.name, profile.status)));
    }
    append_egress_lines(lines, &data.egress_log);
    append_auth_lines(lines, &data.auth_log);
}

fn append_doctor_lines(lines: &mut Vec<Line<'static>>, checks: &[DoctorCheckData]) {
    lines.push(Line::from(vec![Span::styled(
        "Preflight",
        selected_row_style(),
    )]));
    if checks.is_empty() {
        lines.push(Line::from("No preflight checks returned."));
        lines.push(Line::from(""));
        return;
    }
    let passed = checks.iter().filter(|check| check.ok).count();
    if passed > 0 {
        lines.push(Line::from(vec![
            Span::styled("ok", safe_style()),
            Span::raw("  "),
            Span::styled(check_count_label(passed), muted_but_readable_style()),
        ]));
    }

    let failed: Vec<_> = checks.iter().filter(|check| !check.ok).collect();
    if failed.is_empty() {
        lines.push(Line::from(""));
        return;
    }

    for check in failed.iter().take(6) {
        lines.push(Line::from(vec![
            Span::styled("fix", warning_style()),
            Span::raw("  "),
            Span::styled(check.name.clone(), boundary_style()),
            Span::raw("  "),
            Span::styled(check.detail.clone(), muted_but_readable_style()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(check.remedy.clone(), warning_style()),
        ]));
    }
    if failed.len() > 6 {
        lines.push(Line::from(vec![
            Span::styled("fix", warning_style()),
            Span::raw("  "),
            Span::styled(
                format!("{} more checks need attention.", failed.len() - 6),
                warning_style(),
            ),
        ]));
    }
    lines.push(Line::from(""));
}

fn check_count_label(count: usize) -> String {
    if count == 1 {
        "1 check passed".to_string()
    } else {
        format!("{count} checks passed")
    }
}

fn append_egress_lines(lines: &mut Vec<Line<'static>>, entries: &[EgressDecisionData]) {
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Provider egress",
        selected_row_style(),
    )]));
    if entries.is_empty() {
        lines.push(Line::from("No provider egress decisions found."));
        return;
    }
    for entry in entries.iter().take(8) {
        lines.push(Line::from(format!(
            "{}  {}  {}:{}  count={}  {}",
            entry.profile, entry.decision, entry.host, entry.port, entry.count, entry.reason
        )));
    }
}

fn append_auth_lines(lines: &mut Vec<Line<'static>>, entries: &[AuthDecisionData]) {
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Auth broker",
        selected_row_style(),
    )]));
    if entries.is_empty() {
        lines.push(Line::from("No auth broker decisions found."));
        return;
    }
    for entry in entries.iter().take(8) {
        let status = entry
            .upstream_status
            .map(|status| status.to_string())
            .unwrap_or_else(|| "-".to_string());
        lines.push(Line::from(format!(
            "{}  {}  {} {}  upstream={}  count={}",
            entry.profile, entry.decision, entry.method, entry.path, status, entry.count
        )));
    }
}
