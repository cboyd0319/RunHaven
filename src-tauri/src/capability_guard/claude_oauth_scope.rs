//! Trust-boundary guard for the `claude-oauth` capability (design C, Section 4).
//! The host Claude token custody surface is credential-sensitive, so it lives in
//! its own main-window-only capability, isolated from run control and agent
//! sign-in, that read-only surfaces never gain. Kept in its own file so
//! `command_scopes.rs` stays under its ownership ceiling.

use super::support::{
    CLAUDE_OAUTH_PERMISSIONS, assert_generated_permission_allows_only, assert_same_permission_set,
    capability_permissions, capability_windows,
};

/// Pin the claude-oauth capability's exact permission set, its main-window-only
/// scope, and that neither the read-only main capability, the read-only monitor
/// windows, nor the sibling agent-sign-in / run-control capabilities ever gain
/// any Claude Keychain permission.
#[test]
fn claude_oauth_capability_stays_narrow_main_only_and_isolated() {
    let claude_oauth = capability_permissions("claude-oauth.json");
    let expected = CLAUDE_OAUTH_PERMISSIONS
        .iter()
        .map(|(permission, _)| *permission)
        .collect::<Vec<_>>();
    assert_same_permission_set("claude-oauth", &claude_oauth, &expected);

    // Main-window-only: a read-only monitor window must never reach the token
    // custody surface.
    assert_eq!(
        capability_windows("claude-oauth.json"),
        vec!["main".to_string()],
        "claude-oauth capability must target exactly the main window"
    );

    // Every permission maps to exactly its fixed command, and it appears in no
    // other capability: not the read-only main capability, not the monitor
    // windows, and not the sibling credential-adjacent capabilities.
    let others = [
        "main-read.json",
        "session-monitor.json",
        "run-control.json",
        "agent-sign-in.json",
        "launch-run.json",
    ];
    for (permission, command) in CLAUDE_OAUTH_PERMISSIONS {
        assert_generated_permission_allows_only(command);
        for other in others {
            assert!(
                !capability_permissions(other)
                    .iter()
                    .any(|granted| granted == permission),
                "{other} must not grant the Claude Keychain permission {permission}"
            );
        }
    }
}
