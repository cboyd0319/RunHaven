use super::*;
use crate::commands::agent_sign_in::start_agent_sign_in_for_test;
use crate::commands::agent_sign_in_status::get_agent_sign_in_status_for_test;
use crate::contracts::{AgentSignInRequest, AgentSignInStatusRequest};
use crate::tail_status::AgentSignInStatusRegistry;

fn sign_in_request(agent: &str, confirm: bool) -> AgentSignInRequest {
    AgentSignInRequest {
        agent: agent.to_string(),
        confirm,
    }
}

fn status_request(sign_in_id: &str) -> AgentSignInStatusRequest {
    AgentSignInStatusRequest {
        sign_in_id: sign_in_id.to_string(),
    }
}

#[test]
fn start_agent_sign_in_fails_closed_without_confirmation() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = AgentSignInStatusRegistry::default();

    let error = start_agent_sign_in_for_test(sign_in_request("codex", false), &registry, |_| {
        panic!("must not reach confirmation without confirm flag")
    })
    .expect_err("unconfirmed sign-in must fail closed");

    assert!(error.contains("Confirm before"));
    assert!(
        get_agent_sign_in_status_for_test(status_request("any-sign-in"), &registry).is_err(),
        "no sign-in may be recorded when the request is unconfirmed"
    );
}

#[test]
fn start_agent_sign_in_rejects_unknown_agent() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = AgentSignInStatusRegistry::default();

    let error =
        start_agent_sign_in_for_test(sign_in_request("../etc/passwd", true), &registry, |_| {
            panic!("must not confirm an unknown agent")
        })
        .expect_err("unknown agent must be rejected");

    assert!(error.contains("known agent"));
}

#[test]
fn start_agent_sign_in_redirects_claude_to_the_terminal() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = AgentSignInStatusRegistry::default();

    // Claude needs a token pasted into stdin on the host, which the GUI does not
    // host, so it fails closed before any confirmation and points to the CLI.
    let error = start_agent_sign_in_for_test(sign_in_request("claude", true), &registry, |_| {
        panic!("claude must not reach in-app confirmation")
    })
    .expect_err("claude must not sign in from the GUI");

    assert!(error.contains("runhaven login claude"));
    assert!(error.contains("terminal"));
}

#[test]
fn start_agent_sign_in_does_not_run_when_confirmation_denied() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = AgentSignInStatusRegistry::default();

    let mut confirmed_agent = String::new();
    let error = start_agent_sign_in_for_test(sign_in_request("codex", true), &registry, |agent| {
        confirmed_agent = agent.to_string();
        Ok(false)
    })
    .expect_err("denied sign-in must not start");

    assert!(error.contains("canceled"));
    // The confirmation named the canonical agent, not raw WebView input, and
    // denial left no sign-in behind.
    assert_eq!(confirmed_agent, "codex");
    assert!(
        get_agent_sign_in_status_for_test(status_request("any-sign-in"), &registry).is_err(),
        "denied sign-in must not record progress"
    );
}
