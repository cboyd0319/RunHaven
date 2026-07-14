use std::fs;

use serde_json::json;
use time::{Date, Month, PrimitiveDateTime, Time};

use super::*;

const PRIVATE_PATH: &str = "/Users/Alice/My Secret Project";
const PRIVATE_EMAIL: &str = "alice@example.com";
const PRIVATE_TOKEN: &str = "sk-ant-api03-abcdefghijklmnopqrstuvwxyz0123456789";

fn fixed_time() -> OffsetDateTime {
    PrimitiveDateTime::new(
        Date::from_calendar_date(2026, Month::July, 14).expect("date"),
        Time::from_hms(3, 15, 22).expect("time"),
    )
    .assume_utc()
}

#[test]
fn raw_run_record_is_projected_through_an_allowlist() {
    let record = json!({
        "timestamp": "2026-07-14T03:00:00Z",
        "profile": "codex",
        "network": "provider",
        "auth_scope": "agent",
        "status": "failed",
        "return_code": 1,
        "provider_policy": {"allowed": 2, "denied": 3},
        "auth_broker": {"allowed": 4, "denied": 5},
        "cleanup": {"provider_network": "deleted", "run_container": "removed"},
        "run_id": "private-run-id",
        "workspace": PRIVATE_PATH,
        "session": "customer-secret-session",
        "state_volume": "private-volume",
        "git": {"diff": "password=hunter2"},
        "prompt": "copy private source"
    });

    let summary = project_run_record(&record).expect("known record");

    assert_eq!(summary.profile, "codex");
    assert_eq!(summary.status, "failed");
    assert_eq!(summary.provider_denied, 3);
    let debug = format!("{summary:?}");
    for forbidden in [
        PRIVATE_PATH,
        "private-run-id",
        "customer-secret-session",
        "private-volume",
        "hunter2",
        "copy private source",
    ] {
        assert!(!debug.contains(forbidden), "leaked {forbidden}");
    }
}

#[test]
fn network_and_auth_projections_exclude_hosts_paths_reasons_and_ids() {
    let poisoned = json!({
        "timestamp": "2026-07-14T03:01:00Z",
        "profile": "claude",
        "decision": "denied",
        "count": 7,
        "host": "private.customer.example",
        "port": 443,
        "path": "/v1/responses/customer-123?token=secret",
        "reason": PRIVATE_PATH,
        "matched_rule": PRIVATE_EMAIL,
        "run_id": "private-run-id",
        "method": "POST",
        "upstream_status": 403,
        "broker": "api-key"
    });

    let egress = project_egress_record(&poisoned).expect("egress record");
    let auth = project_auth_record(&poisoned).expect("auth record");
    for projected in [format!("{egress:?}"), format!("{auth:?}")] {
        for forbidden in [
            "private.customer.example",
            "customer-123",
            "token=secret",
            PRIVATE_PATH,
            PRIVATE_EMAIL,
            "private-run-id",
        ] {
            assert!(!projected.contains(forbidden), "leaked {forbidden}");
        }
    }
}

#[test]
fn app_log_parser_keeps_only_normalized_known_events() {
    let text = format!(
        "[2026-07-14][03:00:00][runhaven_tauri][INFO] RunHaven desktop 0.7.0 started; log file: {PRIVATE_PATH}/runhaven.log\n\
[2026-07-14][03:00:01][webview:forward@http://127.0.0.1][ERROR] invoke launch_run: failed in 12.5ms: workspace={PRIVATE_PATH} token={PRIVATE_TOKEN}\n\
[2026-07-14][03:00:02][runhaven_tauri][ERROR] panic in thread 'main': prompt and diff {PRIVATE_EMAIL}\n\
[2026-07-14][03:00:03][unknown][WARN] Authorization: Bearer {PRIVATE_TOKEN}\n"
    );

    let (events, omitted) = parse_app_log(&text);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, "start");
    assert_eq!(omitted, 3);
    let normalized = format!("{events:?}");
    for forbidden in [PRIVATE_PATH, PRIVATE_EMAIL, PRIVATE_TOKEN, "prompt", "diff"] {
        assert!(!normalized.contains(forbidden), "leaked {forbidden}");
    }
}

#[test]
fn poisoned_record_fields_collapse_to_fixed_values() {
    let record = json!({
        "timestamp": "2026-07-14T03:00:00Z\u{202e}txt.exe",
        "profile": "codex\u{001b}[31m",
        "network": "provider\nAuthorization: Bearer secret",
        "auth_scope": "agent\u{200b}",
        "status": "failed\rpassword=hunter2",
        "return_code": 1,
        "cleanup": {"result": "complete\n#{PRIVATE_TOKEN}"}
    });

    let summary = project_run_record(&record).expect("known record shape");
    assert_eq!(summary.timestamp, "unavailable");
    assert_eq!(summary.profile, "unknown");
    assert_eq!(summary.network, "unknown");
    assert_eq!(summary.auth_scope, "unknown");
    assert_eq!(summary.status, "unknown");
    assert_eq!(summary.cleanup, "unavailable");
    let rendered = format!("{summary:?}");
    for forbidden in ["\u{001b}", "\u{202e}", "\u{200b}", "hunter2", PRIVATE_TOKEN] {
        assert!(!rendered.contains(forbidden), "leaked {forbidden:?}");
    }
}

#[test]
fn cleanup_projection_never_calls_known_failures_complete() {
    for (field, status) in [
        ("provider_network", "cleanup-error"),
        ("provider_network", "delete-failed"),
        ("provider_network", "launch-failed"),
        ("run_container", "remove-failed"),
    ] {
        let record = json!({
            "timestamp": "2026-07-14T03:00:00Z",
            "profile": "codex",
            "status": "failed",
            "cleanup": {field: status},
        });

        let summary = project_run_record(&record).expect("known record");
        assert_eq!(summary.cleanup, "failed", "{field}={status}");
    }
}

#[test]
fn app_log_event_limit_counts_discarded_known_and_unknown_lines() {
    let mut lines = (0..35)
        .map(|second| {
            format!(
                "[2026-07-14][03:00:{second:02}][runhaven_tauri][INFO] run 0123456789abcdef0123456789abcdef: launching container process"
            )
        })
        .collect::<Vec<_>>();
    lines.push(format!(
        "[2026-07-14][03:01:00][webview][ERROR] password=hunter2 {PRIVATE_TOKEN}"
    ));

    let (events, omitted) = parse_app_log(&lines.join("\n"));

    assert_eq!(events.len(), 30);
    assert_eq!(omitted, 6);
    assert!(events.iter().all(|event| event.action == "launch"));
    assert!(!format!("{events:?}").contains(PRIVATE_TOKEN));
}

#[test]
fn webview_invoke_lines_never_become_report_actions() {
    let (events, omitted) = parse_app_log(
        "[2026-07-14][03:00:00][webview:forward@http://127.0.0.1][INFO] invoke attacker_chosen_action: ok in 42ms",
    );

    assert!(events.is_empty());
    assert_eq!(omitted, 1);
}

#[test]
fn rendered_report_has_a_visible_privacy_contract_and_no_poisoned_values() {
    let data = SupportReportData {
        app_version: "0.7.0 Debug safe-build".to_string(),
        created_at: "2026-07-14T03:15:22Z".to_string(),
        system: vec![Invariant {
            name: "Apple container service".to_string(),
            status: "not ready".to_string(),
            value: "unavailable".to_string(),
        }],
        images: vec![ImageInvariant {
            agent: "codex".to_string(),
            image: "runhaven/codex:0.7.0".to_string(),
            status: "missing".to_string(),
        }],
        sessions: vec![SessionSummary {
            timestamp: "2026-07-14T03:00:00Z".to_string(),
            profile: "codex".to_string(),
            status: "failed".to_string(),
            return_code: 1,
            ..SessionSummary::default()
        }],
        omitted_app_log_lines: 4,
        ..SupportReportData::default()
    };

    let report = render_report(&data);

    assert!(report.contains("Review this file before sharing it"));
    assert!(report.contains("Nothing was uploaded"));
    assert!(report.contains("terminal output"));
    assert!(report.contains("project names, paths, files, contents, and changes"));
    assert!(report.contains("Apple container service: not ready (unavailable)"));
    assert!(report.contains("codex: missing (runhaven/codex:0.7.0)"));
    assert!(report.len() <= MAX_REPORT_BYTES);
    for forbidden in [PRIVATE_PATH, PRIVATE_EMAIL, PRIVATE_TOKEN] {
        assert!(!report.contains(forbidden), "leaked {forbidden}");
    }
}

#[cfg(unix)]
#[test]
fn report_files_are_private_collision_safe_and_never_archives() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().expect("root");
    let request = SupportReportRequest {
        app_version: "0.7.0",
        created_at: fixed_time(),
        app_log: None,
    };

    let first = create_support_report(request.clone(), root.path()).expect("first report");
    let first_contents = fs::read(&first.path).expect("first contents");
    let second = create_support_report(request, root.path()).expect("second report");

    assert_ne!(first.path, second.path);
    assert_eq!(
        fs::read(&first.path).expect("first preserved"),
        first_contents
    );
    assert_eq!(
        first.path.extension().and_then(|value| value.to_str()),
        Some("txt")
    );
    assert!(!first.file_name.ends_with(".tar.gz"));
    assert_eq!(
        fs::metadata(root.path())
            .expect("dir metadata")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(&first.path)
            .expect("file metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let entries = fs::read_dir(root.path())
        .expect("report directory")
        .map(|entry| entry.expect("report entry").path())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .all(|path| path.extension().and_then(|value| value.to_str()) == Some("txt"))
    );
}

#[cfg(unix)]
#[test]
fn existing_report_symlink_is_never_followed_or_overwritten() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("root");
    let outside = root.path().join("outside.txt");
    fs::write(&outside, "keep me").expect("outside");
    let base = root
        .path()
        .join("RunHaven-0.7.0-support-20260714T031522Z.txt");
    symlink(&outside, &base).expect("collision symlink");
    let request = SupportReportRequest {
        app_version: "0.7.0",
        created_at: fixed_time(),
        app_log: None,
    };

    let report = create_support_report(request, root.path()).expect("safe suffix report");

    assert_eq!(
        fs::read_to_string(&outside).expect("outside preserved"),
        "keep me"
    );
    assert_ne!(report.path, base);
    assert!(report.file_name.ends_with("-2.txt"));
}

#[cfg(unix)]
#[test]
fn symlinked_app_log_is_ignored_instead_of_read() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("root");
    let private = root.path().join("private.txt");
    fs::write(
        &private,
        format!(
            "[2026-07-14][03:00:00][webview][INFO] invoke launch_run: failed in 1ms: {PRIVATE_TOKEN}"
        ),
    )
    .expect("private log");
    let logs = root.path().join("Logs");
    fs::create_dir(&logs).expect("logs");
    let link = logs.join("runhaven.log");
    symlink(&private, &link).expect("log symlink");

    let (events, omitted) = super::app_log::read_app_events(&AppLogSource {
        path: &link,
        trusted_parent: &logs,
    });

    assert!(events.is_empty());
    assert_eq!(omitted, 0);
}

#[test]
fn report_truncation_preserves_utf8_and_the_size_cap() {
    let data = SupportReportData {
        app_version: "0.7.0".to_string(),
        created_at: "2026-07-14T03:15:22Z".to_string(),
        system: (0..5000)
            .map(|_| Invariant {
                name: "oversized\u{202e} invariant".to_string(),
                status: "ready".to_string(),
                value: "value".repeat(40),
            })
            .collect(),
        ..SupportReportData::default()
    };

    let report = render_report(&data);

    assert!(std::str::from_utf8(report.as_bytes()).is_ok());
    assert!(report.len() <= MAX_REPORT_BYTES);
    assert!(report.ends_with("[report truncated at the privacy-safe size limit]\n"));
    assert!(!report.contains('\u{202e}'));
}

#[cfg(unix)]
#[test]
fn symlinked_support_directory_fails_closed() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("root");
    let outside = tempfile::tempdir().expect("outside");
    let link = root.path().join("Support Reports");
    symlink(outside.path(), &link).expect("symlink");
    let request = SupportReportRequest {
        app_version: "0.7.0",
        created_at: fixed_time(),
        app_log: None,
    };

    let error = create_support_report(request, &link).expect_err("symlink must fail");

    assert!(error.to_string().contains("support report directory"));
    assert!(
        fs::read_dir(outside.path())
            .expect("outside dir")
            .next()
            .is_none()
    );
}

#[cfg(unix)]
#[test]
fn broken_symlinked_support_directory_fails_closed() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("root");
    let link = root.path().join("Support Reports");
    symlink(root.path().join("missing"), &link).expect("broken symlink");
    let request = SupportReportRequest {
        app_version: "0.7.0",
        created_at: fixed_time(),
        app_log: None,
    };

    let error = create_support_report(request, &link).expect_err("symlink must fail");

    assert!(error.to_string().contains("support report directory"));
}

#[cfg(unix)]
#[test]
fn symlinked_runhaven_data_directory_fails_before_report_creation() {
    use std::os::unix::fs::symlink;

    let data_dir = tempfile::tempdir().expect("Application Support");
    let outside = tempfile::tempdir().expect("outside");
    symlink(outside.path(), data_dir.path().join("RunHaven")).expect("app data symlink");

    let error = prepare_default_support_reports_dir(data_dir.path())
        .expect_err("app-owned symlink must fail");

    assert!(error.to_string().contains("support data directory"));
    assert!(
        fs::read_dir(outside.path())
            .expect("outside dir")
            .next()
            .is_none()
    );
}
