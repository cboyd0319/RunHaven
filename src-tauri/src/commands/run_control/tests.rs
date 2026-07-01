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
    let error = stop_run_with_approval_for_test(
        StopRunRequest {
            run_id: "a".repeat(32),
            confirm_stop: true,
        },
        |_| Ok(false),
        |_| {
            mutated = true;
            Ok(StopRunResponse {
                run_id: "ignored".to_string(),
                container_name: "ignored".to_string(),
                status: "stop-requested".to_string(),
            })
        },
    )
    .expect_err("trusted denial should stop mutation");
    assert!(error.contains("Stop canceled"));
    assert!(!mutated);
}

#[test]
fn stop_run_response_maps_success_payload() {
    let payload = serde_json::json!({
        "run_id": "abc",
        "container_name": "runhaven-shell-run",
        "return_code": 0,
    });
    let response = stop_run_response(&payload).expect("success payload");
    assert_eq!(response.run_id, "abc");
    assert_eq!(response.container_name, "runhaven-shell-run");
    assert_eq!(response.status, "stop-requested");
}

#[test]
fn stop_run_response_reports_nonzero_stop() {
    let payload = serde_json::json!({
        "run_id": "abc",
        "container_name": "runhaven-shell-run",
        "return_code": 1,
    });
    let error = stop_run_response(&payload).expect_err("nonzero stop should fail");
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
    let error = kill_run_with_approval_for_test(
        KillRunRequest {
            run_id: "a".repeat(32),
            confirm_kill: true,
        },
        |_| Ok(false),
        |_| {
            mutated = true;
            Ok(KillRunResponse {
                run_id: "ignored".to_string(),
                container_name: "ignored".to_string(),
                status: "kill-requested".to_string(),
            })
        },
    )
    .expect_err("trusted denial should stop mutation");
    assert!(error.contains("Hard-stop canceled"));
    assert!(!mutated);
}

#[test]
fn kill_run_response_maps_success_payload() {
    let payload = serde_json::json!({
        "run_id": "abc",
        "container_name": "runhaven-shell-run",
        "return_code": 0,
    });
    let response = kill_run_response(&payload).expect("success payload");
    assert_eq!(response.status, "kill-requested");
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
    let error = repair_run_with_approval_for_test(
        RepairRunRequest {
            run_id: "a".repeat(32),
            confirm_repair: true,
        },
        |_| Ok(false),
        |_| {
            mutated = true;
            Ok(RepairRunResponse {
                run_id: "ignored".to_string(),
                container_name: "ignored".to_string(),
                status: "removed".to_string(),
                marker_removed: true,
            })
        },
    )
    .expect_err("trusted denial should stop mutation");
    assert!(error.contains("Repair canceled"));
    assert!(!mutated);
}

#[test]
fn repair_run_response_maps_removed_marker() {
    let payload = serde_json::json!({
        "run_id": "abc",
        "container_name": "runhaven-shell-run",
        "status": "removed",
        "marker_removed": true,
    });
    let response = repair_run_response(&payload).expect("removed payload");
    assert_eq!(response.status, "removed");
    assert!(response.marker_removed);
}

fn stop_run_with_approval_for_test(
    request: StopRunRequest,
    approve: impl FnOnce(&str) -> Result<bool, String>,
    stop: impl FnOnce(&str) -> Result<StopRunResponse, String>,
) -> Result<StopRunResponse, String> {
    validate_direct_run_control_request(
        RunControlAction::Stop,
        &request.run_id,
        request.confirm_stop,
    )?;
    if !approve(&request.run_id)? {
        return Err(RunControlAction::Stop.canceled_error().to_string());
    }
    stop(&request.run_id)
}

fn kill_run_with_approval_for_test(
    request: KillRunRequest,
    approve: impl FnOnce(&str) -> Result<bool, String>,
    kill: impl FnOnce(&str) -> Result<KillRunResponse, String>,
) -> Result<KillRunResponse, String> {
    validate_direct_run_control_request(
        RunControlAction::Kill,
        &request.run_id,
        request.confirm_kill,
    )?;
    if !approve(&request.run_id)? {
        return Err(RunControlAction::Kill.canceled_error().to_string());
    }
    kill(&request.run_id)
}

fn repair_run_with_approval_for_test(
    request: RepairRunRequest,
    approve: impl FnOnce(&str) -> Result<bool, String>,
    repair: impl FnOnce(&str) -> Result<RepairRunResponse, String>,
) -> Result<RepairRunResponse, String> {
    validate_direct_run_control_request(
        RunControlAction::Repair,
        &request.run_id,
        request.confirm_repair,
    )?;
    if !approve(&request.run_id)? {
        return Err(RunControlAction::Repair.canceled_error().to_string());
    }
    repair(&request.run_id)
}
