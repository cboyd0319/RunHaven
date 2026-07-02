//! Behavior tests for the Claude sign-in terminal (login path A). The real
//! host-PTY login spawn is injected, so these cover id validation, confirm-
//! cancel, fail-closed on a spawn failure, the shared cap across attach and
//! login, and input routing for the pasted-back OAuth code. The live
//! `claude auth login` round-trip is the slice `tauri dev` smoke.

use super::{FakeTerminal, open_fake};
use crate::terminal::{
    MAX_TERMINAL_SESSIONS, SpawnedTerminal, TERMINAL_CANCELED, TerminalSessionRegistry,
    open_claude_sign_in_terminal_for_test,
};

/// Open a Claude sign-in terminal in the registry with a fake handle and
/// always-approve, mirroring `open_fake` for the login path.
fn open_login_fake(
    registry: &TerminalSessionRegistry,
    login_id: &str,
) -> (Result<(), String>, FakeTerminal) {
    let fake = FakeTerminal::default();
    let spawn_fake = fake.clone();
    let result = open_claude_sign_in_terminal_for_test(
        registry,
        login_id,
        || Ok(true),
        move || Ok(SpawnedTerminal::without_reader(spawn_fake.handle())),
    );
    (result, fake)
}

#[test]
fn login_terminal_invalid_id_is_rejected_before_confirm_or_spawn() {
    let registry = TerminalSessionRegistry::default();
    let result = open_claude_sign_in_terminal_for_test(
        &registry,
        "bad id\u{1b}[2J",
        || panic!("must not confirm on an invalid id"),
        || panic!("must not spawn on an invalid id"),
    );
    assert!(
        result.is_err(),
        "control-character login id must be rejected"
    );
}

#[test]
fn login_terminal_confirm_cancel_spawns_nothing() {
    let registry = TerminalSessionRegistry::default();
    let result = open_claude_sign_in_terminal_for_test(
        &registry,
        "login-cancel",
        || Ok(false),
        || panic!("cancel must spawn nothing"),
    );
    assert_eq!(result, Err(TERMINAL_CANCELED.to_string()));
    assert!(registry.remove("login-cancel").is_none());
}

#[test]
fn login_terminal_fails_closed_and_registers_nothing_when_the_spawn_fails() {
    // Fail closed: if core cannot open the login PTY (container missing, image
    // not built, proxy setup fails), nothing is registered so the UI keeps its
    // "run it in your terminal" fallback.
    let registry = TerminalSessionRegistry::default();
    let result = open_claude_sign_in_terminal_for_test(
        &registry,
        "login-fail",
        || Ok(true),
        || Err("RunHaven could not open the Claude sign-in terminal.".to_string()),
    );
    assert!(result.is_err(), "a spawn failure must fail the open");
    assert!(
        registry.remove("login-fail").is_none(),
        "a failed login open must leave no registry entry"
    );
}

#[test]
fn login_terminal_shares_the_terminal_cap_with_attach() {
    // The login open goes through the same registry, so it counts against the
    // shared cap alongside attach sessions.
    let registry = TerminalSessionRegistry::default();
    open_fake(&registry, "attach-run")
        .0
        .expect("attach within cap");
    open_login_fake(&registry, "login-one")
        .0
        .expect("login within cap");
    // Cap is now full (MAX_TERMINAL_SESSIONS == 2); a second login is refused
    // before any dialog or spawn.
    let result = open_claude_sign_in_terminal_for_test(
        &registry,
        "login-overflow",
        || panic!("cap must be enforced before confirmation"),
        || panic!("cap must be enforced before spawn"),
    );
    let error = result.expect_err("cap must hold across attach and login");
    assert!(error.contains(&MAX_TERMINAL_SESSIONS.to_string()));
}

#[test]
fn login_terminal_registers_and_input_routes_for_paste_back() {
    // The login terminal accepts input so the user can paste the OAuth code.
    let registry = TerminalSessionRegistry::default();
    let (result, _fake) = open_login_fake(&registry, "login-io");
    result.expect("login open");
    registry
        .write_input("login-io", b"paste-code-123\n")
        .expect("paste routes to the login terminal");
    registry.close("login-io").expect("close");
    assert!(registry.remove("login-io").is_none());
}
