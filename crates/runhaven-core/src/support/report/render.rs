use std::fmt::Write;

use super::{MAX_REPORT_BYTES, MAX_REPORT_EVENTS, SupportReportData};

pub(super) fn render_report(data: &SupportReportData) -> String {
    let mut out = String::new();
    writeln!(out, "RunHaven support report").expect("write string");
    writeln!(out, "Report format: 1").expect("write string");
    writeln!(out, "Created: {}", safe(&data.created_at)).expect("write string");
    writeln!(out, "RunHaven: {}", safe(&data.app_version)).expect("write string");
    writeln!(out).expect("write string");
    writeln!(out, "Review this file before sharing it.").expect("write string");
    writeln!(out, "Nothing was uploaded or submitted by RunHaven.").expect("write string");
    writeln!(out, "Included: versions, setup and image results, and recent normalized RunHaven action outcomes.").expect("write string");
    writeln!(out, "Not included: raw logs, terminal output, prompts or messages, project names, paths, files, contents, and changes, environment data, credentials, account identifiers, network hosts, request paths, or browser data.").expect("write string");
    writeln!(out, "For a possible security or privacy issue, do not post this report publicly. Use GitHub private vulnerability reporting.").expect("write string");

    section(&mut out, "System and setup");
    if data.system.is_empty() {
        writeln!(out, "unavailable").expect("write string");
    } else {
        for item in &data.system {
            writeln!(
                out,
                "{}: {} ({})",
                safe(&item.name),
                safe(&item.status),
                safe(&item.value)
            )
            .expect("write string");
        }
    }

    section(&mut out, "Assistant images");
    if data.images.is_empty() {
        writeln!(out, "unavailable").expect("write string");
    } else {
        for item in &data.images {
            writeln!(
                out,
                "{}: {} ({})",
                safe(&item.agent),
                safe(&item.status),
                safe(&item.image)
            )
            .expect("write string");
        }
    }

    section(&mut out, "Recent sessions");
    if data.sessions.is_empty() {
        writeln!(out, "none recorded").expect("write string");
    } else {
        for (index, item) in data.sessions.iter().take(10).enumerate() {
            writeln!(
                out,
                "R{} {} profile={} status={} return={} network={} auth-scope={} provider-allowed={} provider-denied={} auth-allowed={} auth-denied={} cleanup={}",
                index + 1,
                safe(&item.timestamp),
                safe(&item.profile),
                safe(&item.status),
                item.return_code,
                safe(&item.network),
                safe(&item.auth_scope),
                item.provider_allowed,
                item.provider_denied,
                item.auth_allowed,
                item.auth_denied,
                safe(&item.cleanup),
            )
            .expect("write string");
        }
    }

    decision_section(&mut out, "Recent network policy decisions", &data.egress);
    decision_section(&mut out, "Recent sign-in broker decisions", &data.auth);

    section(&mut out, "Recent app actions");
    if data.app_events.is_empty() {
        writeln!(out, "none available").expect("write string");
    } else {
        for event in data.app_events.iter().take(MAX_REPORT_EVENTS) {
            write!(
                out,
                "{} component={} action={} outcome={}",
                safe(&event.timestamp),
                safe(&event.component),
                safe(&event.action),
                safe(&event.outcome),
            )
            .expect("write string");
            if let Some(duration) = event.duration_ms {
                write!(out, " duration-ms={duration}").expect("write string");
            }
            if let Some(code) = event.exit_code {
                write!(out, " exit-code={code}").expect("write string");
            }
            writeln!(out).expect("write string");
        }
    }
    writeln!(
        out,
        "Unstructured app-log lines omitted: {}",
        data.omitted_app_log_lines
    )
    .expect("write string");

    if out.len() > MAX_REPORT_BYTES {
        out.truncate(MAX_REPORT_BYTES.saturating_sub(64));
        while !out.is_char_boundary(out.len()) {
            out.pop();
        }
        out.push_str("\n[report truncated at the privacy-safe size limit]\n");
    }
    out
}

fn decision_section(out: &mut String, title: &str, decisions: &[super::DecisionSummary]) {
    section(out, title);
    if decisions.is_empty() {
        writeln!(out, "none recorded").expect("write string");
    } else {
        for item in decisions.iter().take(20) {
            writeln!(
                out,
                "{} profile={} decision={} count={} category={}",
                safe(&item.timestamp),
                safe(&item.profile),
                safe(&item.decision),
                item.count,
                safe(&item.category),
            )
            .expect("write string");
        }
    }
}

fn section(out: &mut String, title: &str) {
    writeln!(out, "\n{title}").expect("write string");
    writeln!(out, "{}", "-".repeat(title.len())).expect("write string");
}

fn safe(value: &str) -> String {
    let normalized = value
        .chars()
        .take(160)
        .map(|character| {
            if character.is_ascii_alphanumeric()
                || matches!(
                    character,
                    ' ' | '.' | ',' | ':' | ';' | '-' | '_' | '+' | '/' | '='
                )
            {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if normalized.trim().is_empty() {
        "unavailable".to_string()
    } else {
        normalized
    }
}
