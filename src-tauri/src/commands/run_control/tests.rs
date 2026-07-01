use super::*;

#[test]
fn stop_run_requires_explicit_confirmation() {
    let error = validate_direct_run_control_request(RunControlAction::Stop, &"a".repeat(32), false)
        .expect_err("stop without confirmation should fail");
    assert!(error.contains("Confirm the stop"));
}

#[test]
fn stop_run_rejects_oversized_run_id() {
    let error = validate_direct_run_control_request(
        RunControlAction::Stop,
        &"a".repeat(MAX_RUN_ID_LEN + 1),
        true,
    )
    .expect_err("oversized run id should fail");
    assert!(error.contains("run id is too long"));
}

#[test]
fn stop_run_with_trusted_approval_denial_skips_mutation() {
    let mut mutated = false;
    let error = run_direct_run_control_for_test(
        RunControlAction::Stop,
        &"a".repeat(32),
        true,
        |_| Ok(false),
        |_| {
            mutated = true;
            Ok::<(), String>(())
        },
    )
    .expect_err("trusted denial should stop mutation");
    assert!(error.contains("Stop canceled"));
    assert!(!mutated);
}

#[test]
fn stop_run_payload_maps_success() {
    let payload = serde_json::json!({
        "run_id": "abc",
        "container_name": "runhaven-shell-run",
        "return_code": 0,
    });
    let outcome =
        map_run_control_payload(RunControlAction::Stop, &payload).expect("success payload");
    assert_eq!(outcome.run_id, "abc");
    assert_eq!(outcome.container_name, "runhaven-shell-run");
    assert_eq!(outcome.status, "stop-requested");
}

#[test]
fn stop_run_payload_reports_nonzero_stop() {
    let payload = serde_json::json!({
        "run_id": "abc",
        "container_name": "runhaven-shell-run",
        "return_code": 1,
    });
    let error =
        map_run_control_payload(RunControlAction::Stop, &payload).expect_err("nonzero stop fails");
    assert!(error.contains("container stop exited 1"));
}

#[test]
fn kill_run_requires_explicit_confirmation() {
    let error = validate_direct_run_control_request(RunControlAction::Kill, &"a".repeat(32), false)
        .expect_err("kill without confirmation should fail");
    assert!(error.contains("Confirm the hard-stop"));
}

#[test]
fn kill_run_with_trusted_approval_denial_skips_mutation() {
    let mut mutated = false;
    let error = run_direct_run_control_for_test(
        RunControlAction::Kill,
        &"a".repeat(32),
        true,
        |_| Ok(false),
        |_| {
            mutated = true;
            Ok::<(), String>(())
        },
    )
    .expect_err("trusted denial should stop mutation");
    assert!(error.contains("Hard-stop canceled"));
    assert!(!mutated);
}

#[test]
fn kill_run_payload_maps_success() {
    let payload = serde_json::json!({
        "run_id": "abc",
        "container_name": "runhaven-shell-run",
        "return_code": 0,
    });
    let outcome =
        map_run_control_payload(RunControlAction::Kill, &payload).expect("success payload");
    assert_eq!(outcome.status, "kill-requested");
}

#[test]
fn repair_run_requires_explicit_confirmation() {
    let error =
        validate_direct_run_control_request(RunControlAction::Repair, &"a".repeat(32), false)
            .expect_err("repair without confirmation should fail");
    assert!(error.contains("Confirm the repair"));
}

#[test]
fn repair_run_with_trusted_approval_denial_skips_mutation() {
    let mut mutated = false;
    let error = run_direct_run_control_for_test(
        RunControlAction::Repair,
        &"a".repeat(32),
        true,
        |_| Ok(false),
        |_| {
            mutated = true;
            Ok::<(), String>(())
        },
    )
    .expect_err("trusted denial should stop mutation");
    assert!(error.contains("Repair canceled"));
    assert!(!mutated);
}

#[test]
fn repair_run_payload_maps_removed_marker() {
    let payload = serde_json::json!({
        "run_id": "abc",
        "container_name": "runhaven-shell-run",
        "status": "removed",
        "marker_removed": true,
    });
    let outcome =
        map_run_control_payload(RunControlAction::Repair, &payload).expect("removed payload");
    assert_eq!(outcome.status, "removed");
    assert!(outcome.marker_removed);
}

/// The direct-path equivalent of the shared managed test seam: validate the
/// request, run an injected approval, then an injected mutation.
fn run_direct_run_control_for_test<Out>(
    action: RunControlAction,
    run_id: &str,
    confirmed: bool,
    approve: impl FnOnce(&str) -> Result<bool, String>,
    mutate: impl FnOnce(&str) -> Result<Out, String>,
) -> Result<Out, String> {
    validate_direct_run_control_request(action, run_id, confirmed)?;
    if !approve(run_id)? {
        return Err(action.canceled_error().to_string());
    }
    mutate(run_id)
}
