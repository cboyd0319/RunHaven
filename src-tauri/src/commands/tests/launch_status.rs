use super::*;

#[test]
fn launch_status_command_reads_backend_owned_status() {
    let registry = LaunchStatusRegistry::default();

    let missing = crate::commands::launch_status::get_launch_status_for_test(
        LaunchStatusRequest {
            run_id: "missing-launch-status".to_string(),
        },
        &registry,
    )
    .expect_err("unknown launch status rejected");
    assert!(missing.contains("launch status not found"));

    registry
        .mark_starting("launch-status-command", "codex", "project")
        .expect("starting status");
    let status = crate::commands::launch_status::get_launch_status_for_test(
        LaunchStatusRequest {
            run_id: "launch-status-command".to_string(),
        },
        &registry,
    )
    .expect("launch status");

    assert_eq!(status.run_id, "launch-status-command");
    assert_eq!(status.state, "starting");
    assert_eq!(status.label, "Starting");
    assert!(status.detail.contains("Waiting for the managed session"));
    assert!(
        !serde_json::to_string(&status)
            .expect("serialize launch status")
            .contains("/Users/")
    );
}
