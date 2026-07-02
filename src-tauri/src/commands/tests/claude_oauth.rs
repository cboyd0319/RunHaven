//! Hermetic tests for the Claude Keychain sign-in commands. The live host
//! `claude setup-token`, the real Keychain, and the biometric prompt are
//! maintainer smokes (Gate 2); here every Keychain/network touch is injected, so
//! these prove the trust-boundary behavior with no secret in reach.

use crate::claude_oauth_signin::ClaudeOAuthSignInRegistry;
use crate::commands::claude_oauth::{
    ClearOutcome, KeychainPresence, clear_claude_oauth_for_test, get_claude_oauth_status_for_test,
    start_claude_oauth_signin_for_test,
};
use crate::contracts::{
    ClaudeOAuthClearRequest, ClaudeOAuthClearResponse, ClaudeOAuthStartResponse,
    ClaudeOAuthStatusResponse,
};

/// A sentinel secret and host path that must never surface in any response.
const SENTINEL_TOKEN: &str = "sk-ant-oat01-SENTINEL-must-never-appear";
const SENTINEL_PATH: &str = "/Users/victim/Library/Keychains/login.keychain-db";

fn start_request(confirm: bool) -> crate::contracts::ClaudeOAuthStartRequest {
    crate::contracts::ClaudeOAuthStartRequest { confirm }
}

fn clear_request(confirm: bool) -> ClaudeOAuthClearRequest {
    ClaudeOAuthClearRequest { confirm }
}

/// The absent presence: no item, no age. Injected where status must resolve to
/// `not_signed_in`/`failed`.
fn absent() -> KeychainPresence {
    KeychainPresence {
        present: false,
        age_days: None,
    }
}

/// A present item with an optional non-secret age (days since it was saved).
/// `None` models a present token whose creation date could not be parsed.
fn signed_in(age_days: Option<u32>) -> KeychainPresence {
    KeychainPresence {
        present: true,
        age_days,
    }
}

// -- start: input validation and native-confirm gate ----------------------

#[test]
fn start_fails_closed_without_confirmation_flag() {
    let registry = ClaudeOAuthSignInRegistry::default();
    let error = start_claude_oauth_signin_for_test(
        start_request(false),
        &registry,
        || panic!("must not reach confirmation without the confirm flag"),
        || panic!("must not run capture without the confirm flag"),
    )
    .expect_err("unconfirmed start must fail closed");
    assert!(error.contains("Confirm before"));
    // Nothing was recorded: the singleton phase is still idle (not_signed_in).
    assert_status(&registry, || Ok(absent()), "not_signed_in");
}

#[test]
fn start_does_not_run_capture_when_native_confirm_denied() {
    let registry = ClaudeOAuthSignInRegistry::default();
    let error = start_claude_oauth_signin_for_test(
        start_request(true),
        &registry,
        || Ok(false), // user cancels the native dialog
        || panic!("capture must not run when the native confirmation is denied"),
    )
    .expect_err("denied confirmation must not start capture");
    assert!(error.contains("canceled"));
    // Denial left no sign-in behind.
    assert_status(&registry, || Ok(absent()), "not_signed_in");
}

#[test]
fn start_response_reports_signing_and_carries_no_token() {
    let registry = ClaudeOAuthSignInRegistry::default();
    // A capture that would have stored the sentinel token; the id-poll must
    // still carry nothing token-shaped.
    let response =
        start_claude_oauth_signin_for_test(start_request(true), &registry, || Ok(true), || Ok(()))
            .expect("confirmed start begins");
    assert_eq!(response.state, "signing");
    assert!(!response.sign_in_id.is_empty());
    assert_no_secret_in(&serialize_start(&response));
}

// -- status: presence -> enum, no token, no prompt ------------------------

#[test]
fn status_maps_presence_to_signed_in_without_reading_the_token() {
    let registry = ClaudeOAuthSignInRegistry::default();
    // The injected presence checker stands in for `token_presence`: it returns
    // presence (and a non-secret age) and NEVER yields the token. Here the age
    // is unknown, proving a present token still reports signed_in with no age.
    let response = get_claude_oauth_status_for_test(&registry, || Ok(signed_in(None)))
        .expect("status derives from presence");
    assert_eq!(response.status, "signed_in");
    assert!(response.reason.is_none());
    assert_eq!(response.age_days, None);
    assert_no_secret_in(&serialize_status(&response));
}

#[test]
fn status_signed_in_carries_the_non_secret_age() {
    let registry = ClaudeOAuthSignInRegistry::default();
    // A present token past the default 90-day cadence: the age is a plain day
    // count carried straight through, never a token or token-derived value.
    let response = get_claude_oauth_status_for_test(&registry, || Ok(signed_in(Some(97))))
        .expect("status derives from presence");
    assert_eq!(response.status, "signed_in");
    assert_eq!(response.age_days, Some(97));
    let serialized = serialize_status(&response);
    assert!(
        serialized.contains("\"ageDays\":97"),
        "the non-secret age must serialize as a plain day count: {serialized}"
    );
    assert_no_secret_in(&serialized);
}

#[test]
fn status_maps_absence_to_not_signed_in() {
    let registry = ClaudeOAuthSignInRegistry::default();
    let response =
        get_claude_oauth_status_for_test(&registry, || Ok(absent())).expect("status derives");
    assert_eq!(response.status, "not_signed_in");
    assert_eq!(response.age_days, None);
}

#[test]
fn status_reports_signing_without_checking_presence() {
    let registry = ClaudeOAuthSignInRegistry::default();
    registry.begin("in-flight").expect("begin");
    // While signing, presence is not even consulted: the checker must not run.
    let response = get_claude_oauth_status_for_test(&registry, || {
        panic!("presence must not be checked while a sign-in is in flight")
    })
    .expect("status derives from in-flight phase");
    assert_eq!(response.status, "signing");
    assert_eq!(response.sign_in_id.as_deref(), Some("in-flight"));
}

#[test]
fn status_reports_failed_with_path_free_reason_when_absent() {
    let registry = ClaudeOAuthSignInRegistry::default();
    registry.begin("id").expect("begin");
    registry
        .mark_failed(
            "id",
            "Claude sign-in did not finish; nothing was saved.".to_string(),
        )
        .expect("fail");
    let response =
        get_claude_oauth_status_for_test(&registry, || Ok(absent())).expect("status derives");
    assert_eq!(response.status, "failed");
    assert_eq!(response.age_days, None);
    let reason = response.reason.clone().expect("failed carries a reason");
    assert!(!reason.contains("sk-ant-"));
    assert!(!reason.contains('/'));
    assert_no_secret_in(&serialize_status(&response));
}

// -- clear: cleared / already-absent, native-confirm gate -----------------

#[test]
fn clear_fails_closed_without_confirmation_flag() {
    let error = clear_claude_oauth_for_test(
        clear_request(false),
        || panic!("must not confirm without the flag"),
        || panic!("must not delete without the flag"),
    )
    .expect_err("unconfirmed clear must fail closed");
    assert!(error.contains("Confirm before"));
}

#[test]
fn clear_does_not_delete_when_native_confirm_denied() {
    let error = clear_claude_oauth_for_test(
        clear_request(true),
        || Ok(false),
        || panic!("delete must not run when the native confirmation is denied"),
    )
    .expect_err("denied confirmation must not delete");
    assert!(error.contains("Nothing was removed"));
}

#[test]
fn clear_maps_outcomes_to_cleared_and_already_absent() {
    let cleared = clear_claude_oauth_for_test(
        clear_request(true),
        || Ok(true),
        || Ok(ClearOutcome::Cleared),
    )
    .expect("clear runs");
    assert_eq!(cleared.outcome, "cleared");

    let absent = clear_claude_oauth_for_test(
        clear_request(true),
        || Ok(true),
        || Ok(ClearOutcome::AlreadyAbsent),
    )
    .expect("clear runs");
    assert_eq!(absent.outcome, "already_absent");
    assert_no_secret_in(&serialize_clear(&cleared));
    assert_no_secret_in(&serialize_clear(&absent));
}

// -- no-leak serialization across every response --------------------------

#[test]
fn no_response_ever_serializes_a_secret() {
    let registry = ClaudeOAuthSignInRegistry::default();

    // start
    let start =
        start_claude_oauth_signin_for_test(start_request(true), &registry, || Ok(true), || Ok(()))
            .expect("start");
    assert_no_secret_in(&serialize_start(&start));

    // every status variant, including a signed-in one carrying an age
    for presence in [signed_in(Some(120)), signed_in(None), absent()] {
        let status =
            get_claude_oauth_status_for_test(&ClaudeOAuthSignInRegistry::default(), move || {
                Ok(presence)
            })
            .expect("status");
        assert_no_secret_in(&serialize_status(&status));
    }
    let failed_registry = ClaudeOAuthSignInRegistry::default();
    failed_registry.begin("x").expect("begin");
    failed_registry
        .mark_failed(
            "x",
            "Claude sign-in did not finish; nothing was saved.".to_string(),
        )
        .expect("fail");
    let failed =
        get_claude_oauth_status_for_test(&failed_registry, || Ok(absent())).expect("status");
    assert_no_secret_in(&serialize_status(&failed));

    // clear
    for outcome in [ClearOutcome::Cleared, ClearOutcome::AlreadyAbsent] {
        let response =
            clear_claude_oauth_for_test(clear_request(true), || Ok(true), move || Ok(outcome))
                .expect("clear");
        assert_no_secret_in(&serialize_clear(&response));
    }
}

// -- helpers --------------------------------------------------------------

fn assert_status(
    registry: &ClaudeOAuthSignInRegistry,
    presence: impl FnOnce() -> Result<KeychainPresence, String>,
    expected: &str,
) {
    let response = get_claude_oauth_status_for_test(registry, presence).expect("status");
    assert_eq!(response.status, expected);
}

fn serialize_start(response: &ClaudeOAuthStartResponse) -> String {
    serde_json::to_string(response).expect("serialize start")
}

fn serialize_status(response: &ClaudeOAuthStatusResponse) -> String {
    serde_json::to_string(response).expect("serialize status")
}

fn serialize_clear(response: &ClaudeOAuthClearResponse) -> String {
    serde_json::to_string(response).expect("serialize clear")
}

/// Assert the serialized response carries no token, no token prefix, and no host
/// path. The word "token" is allowed (user-facing copy names it); only a token
/// VALUE, a prefix, or a raw path would be a leak.
fn assert_no_secret_in(serialized: &str) {
    for forbidden in [
        SENTINEL_TOKEN,
        "sk-ant-",
        SENTINEL_PATH,
        "/Users/",
        "/private/",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "response leaked `{forbidden}`: {serialized}"
        );
    }
}
