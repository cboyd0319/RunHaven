//! Trust-boundary guard for the one-action setup recovery capability.

use super::support::{
    assert_generated_permission_allows_only, assert_same_permission_set, capability_permissions,
    capability_windows,
};

#[test]
fn setup_recovery_capability_stays_single_command_main_only_and_isolated() {
    let permission = "allow-perform-setup-recovery";
    let recovery = capability_permissions("setup-recovery.json");
    assert_same_permission_set("setup-recovery", &recovery, &[permission]);
    assert_eq!(
        capability_windows("setup-recovery.json"),
        vec!["main".to_string()],
        "setup recovery must stay main-window-only"
    );
    assert_generated_permission_allows_only("perform_setup_recovery");

    for other in ["main-read.json", "run-control.json"] {
        assert!(
            !capability_permissions(other)
                .iter()
                .any(|candidate| candidate == permission),
            "{other} must not gain the setup recovery mutation"
        );
    }
}
