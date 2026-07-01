use super::*;

#[test]
fn log_snapshot_requires_sensitive_output_confirmation() {
    let request = LogSnapshotRequest {
        run_id: "runhaven-log-test".to_string(),
        lines: Some(200),
        confirm_sensitive_output: false,
    };

    let error = crate::commands::log_snapshot::validated_lines(&request)
        .expect_err("confirmation required");

    assert!(error.contains("Confirm raw log viewing"));
}

#[test]
fn log_snapshot_response_caps_lines_and_bytes() {
    let too_many_lines = LogSnapshotRequest {
        run_id: "runhaven-log-test".to_string(),
        lines: Some(501),
        confirm_sensitive_output: true,
    };

    let error = crate::commands::log_snapshot::validated_lines(&too_many_lines)
        .expect_err("line cap required");

    assert!(error.contains("500"));

    let response = crate::commands::log_snapshot::log_snapshot_response(
        "runhaven-log-test",
        200,
        b"first line\nsecond line\nthird line\n",
        18,
    )
    .expect("snapshot response");

    assert_eq!(response.run_id, "runhaven-log-test");
    assert_eq!(response.requested_lines, 200);
    assert!(response.truncated);
    assert!(response.text.len() <= 18);
    assert_eq!(response.cursor, response.captured_at);
    assert_eq!(response.entries.len(), 1);
    let entry = &response.entries[0];
    assert_eq!(entry.source, "container-stdio");
    assert_eq!(entry.timestamp, response.captured_at);
    assert_eq!(entry.text, response.text);
    assert_eq!(entry.line_count, response.returned_lines);
    assert!(
        response
            .warnings
            .iter()
            .any(|warning| warning.contains("Raw"))
    );
}

#[test]
fn log_snapshot_rejects_invalid_run_id_before_container_access() {
    let request = LogSnapshotRequest {
        run_id: "../not-a-run".to_string(),
        lines: Some(200),
        confirm_sensitive_output: true,
    };

    let error =
        crate::commands::log_snapshot::get_log_snapshot(request).expect_err("invalid run id");

    assert!(error.contains("invalid run id"));
}

#[test]
fn log_snapshot_native_denial_stops_before_output_read() {
    let request = LogSnapshotRequest {
        run_id: "runhaven-log-denied".to_string(),
        lines: Some(20),
        confirm_sensitive_output: true,
    };

    let error = crate::commands::log_snapshot::get_log_snapshot_with_approval_for_test(
        request,
        |run_id, lines| {
            assert_eq!(run_id, "runhaven-log-denied");
            assert_eq!(lines, 20);
            Ok(false)
        },
    )
    .expect_err("native denial should stop the read");

    assert_eq!(error, "Raw output view canceled.");
}

#[test]
fn log_snapshot_rejects_non_runhaven_container_names() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "bad-container-log-run",
        json!({
            "timestamp": "2026-06-16T00:00:00Z",
            "run_id": "bad-container-log-run",
            "profile": "codex",
            "workspace": "/tmp/runhaven-active",
            "network": "provider",
            "status": "running",
            "container_name": "other-project-container",
            "state_volume": "runhaven-codex-active-home",
            "session": "default"
        }),
    )
    .expect("active run payload");
    let request = LogSnapshotRequest {
        run_id: "bad-container-log-run".to_string(),
        lines: Some(200),
        confirm_sensitive_output: true,
    };

    let error =
        crate::commands::log_snapshot::get_log_snapshot(request).expect_err("container rejected");

    assert!(error.contains("RunHaven-owned"));
}
