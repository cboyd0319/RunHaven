//! Behavior tests for the shared agent sign-in terminal. The real
//! host-PTY login spawn is injected, so these cover id validation, confirm-
//! cancel, fail-closed on a spawn failure, the shared cap across attach and
//! login, input routing, and the exact four-agent allowlist.

use super::{FakeTerminal, open_fake};
use crate::terminal::{
    MAX_TERMINAL_SESSIONS, SpawnedTerminal, TERMINAL_CANCELED, TerminalSessionRegistry,
    open_agent_sign_in_terminal_for_test,
};

/// Open an agent sign-in terminal in the registry with a fake handle and
/// always-approve, mirroring `open_fake` for the login path.
fn open_login_fake(
    registry: &TerminalSessionRegistry,
    agent: &str,
    login_id: &str,
) -> (Result<(), String>, FakeTerminal) {
    let fake = FakeTerminal::default();
    let spawn_fake = fake.clone();
    let result = open_agent_sign_in_terminal_for_test(
        registry,
        agent,
        login_id,
        |_| Ok(true),
        move |_, _| Ok(SpawnedTerminal::without_reader(spawn_fake.handle())),
    );
    (result, fake)
}

#[test]
fn login_terminal_invalid_id_is_rejected_before_confirm_or_spawn() {
    let registry = TerminalSessionRegistry::default();
    let result = open_agent_sign_in_terminal_for_test(
        &registry,
        "claude",
        "bad id\u{1b}[2J",
        |_| panic!("must not confirm on an invalid id"),
        |_, _| panic!("must not spawn on an invalid id"),
    );
    assert!(
        result.is_err(),
        "control-character login id must be rejected"
    );
}

#[test]
fn login_terminal_confirm_cancel_spawns_nothing() {
    let registry = TerminalSessionRegistry::default();
    let result = open_agent_sign_in_terminal_for_test(
        &registry,
        "claude",
        "login-cancel",
        |_| Ok(false),
        |_, _| panic!("cancel must spawn nothing"),
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
    let result = open_agent_sign_in_terminal_for_test(
        &registry,
        "claude",
        "login-fail",
        |_| Ok(true),
        |_, _| Err("RunHaven could not open the sign-in terminal.".to_string()),
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
    open_login_fake(&registry, "claude", "login-one")
        .0
        .expect("login within cap");
    // Cap is now full (MAX_TERMINAL_SESSIONS == 2); a second login is refused
    // before any dialog or spawn.
    let result = open_agent_sign_in_terminal_for_test(
        &registry,
        "codex",
        "login-overflow",
        |_| panic!("cap must be enforced before confirmation"),
        |_, _| panic!("cap must be enforced before spawn"),
    );
    let error = result.expect_err("cap must hold across attach and login");
    assert!(error.contains(&MAX_TERMINAL_SESSIONS.to_string()));
}

#[test]
fn login_terminal_registers_and_input_routes_for_paste_back() {
    // The login terminal accepts input so the user can paste the OAuth code.
    let registry = TerminalSessionRegistry::default();
    let (result, _fake) = open_login_fake(&registry, "antigravity", "login-io");
    result.expect("login open");
    registry
        .write_input("login-io", b"paste-code-123\n")
        .expect("paste routes to the login terminal");
    registry.close("login-io").expect("close");
    assert!(registry.remove("login-io").is_none());
}

#[test]
fn login_terminal_accepts_exactly_the_four_product_agents() {
    for agent in ["claude", "codex", "antigravity", "copilot"] {
        let registry = TerminalSessionRegistry::default();
        open_login_fake(&registry, agent, &format!("login-{agent}"))
            .0
            .unwrap_or_else(|error| panic!("{agent} should open: {error}"));
    }

    let registry = TerminalSessionRegistry::default();
    let error = open_agent_sign_in_terminal_for_test(
        &registry,
        "shell",
        "login-shell",
        |_| panic!("unsupported agent must fail before confirmation"),
        |_, _| panic!("unsupported agent must fail before spawn"),
    )
    .expect_err("shell is not a sign-in agent");
    assert!(error.contains("supported agent"));
}

#[test]
fn login_terminal_uses_a_rust_minted_execution_id_not_the_webview_session_id() {
    let registry = TerminalSessionRegistry::default();
    let mut observed = None;
    open_agent_sign_in_terminal_for_test(
        &registry,
        "codex",
        "existing-run-id",
        |_| Ok(true),
        |_, execution_id| {
            observed = Some(execution_id.to_string());
            Ok(SpawnedTerminal::without_reader(
                FakeTerminal::default().handle(),
            ))
        },
    )
    .expect("open login");

    let execution_id = observed.expect("execution id");
    assert_ne!(execution_id, "existing-run-id");
    assert_eq!(
        execution_id.len(),
        32,
        "new_run_id is a UUID without dashes"
    );
}
