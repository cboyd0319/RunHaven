//! Trust-boundary guard for the `terminal` capability (terminal design section
//! 7). A live host PTY into a sandbox is the app's most sensitive interactive
//! grant, so it lives in its own main-window-only capability that read-only
//! surfaces never gain. Kept in its own file so `command_scopes.rs` stays under
//! its ownership ceiling.

use super::support::{
    TERMINAL_PERMISSIONS, assert_generated_permission_allows_only, assert_same_permission_set,
    capability_permissions, capability_windows,
};

/// Pin the terminal capability's exact permission set, its main-window-only
/// scope, and that neither the read-only main capability nor the read-only
/// monitor windows ever gain any terminal permission.
#[test]
fn terminal_capability_stays_narrow_main_only_and_off_read_only_surfaces() {
    let terminal_permissions = capability_permissions("terminal.json");
    let expected = TERMINAL_PERMISSIONS
        .iter()
        .map(|(permission, _)| *permission)
        .collect::<Vec<_>>();
    assert_same_permission_set("terminal", &terminal_permissions, &expected);

    // Main-window-only: a read-only monitor window must never get a PTY.
    assert_eq!(
        capability_windows("terminal.json"),
        vec!["main".to_string()],
        "terminal capability must target exactly the main window"
    );

    let main_read = capability_permissions("main-read.json");
    let session_monitor = capability_permissions("session-monitor.json");
    for (permission, command) in TERMINAL_PERMISSIONS {
        assert_generated_permission_allows_only(command);
        assert!(
            !main_read.iter().any(|granted| granted == permission),
            "main-read must stay read-only and cannot grant {permission}"
        );
        assert!(
            !session_monitor.iter().any(|granted| granted == permission),
            "session-monitor must stay a read-only viewer and cannot grant {permission}"
        );
    }
}
