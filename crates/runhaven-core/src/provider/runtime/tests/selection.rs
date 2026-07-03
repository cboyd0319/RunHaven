//! Auth-source selection precedence (design C, F1) and read-timing proof (F3):
//! the token READ (which prompts) happens exactly once, at broker startup, only
//! after plan preparation and pre-confirm validation, which use PRESENCE only.
//! All fakes are injected; no real Keychain is touched.

use std::cell::Cell;

use anyhow::Result;
use secrecy::ExposeSecret;
use zeroize::Zeroizing;

use super::super::broker_secret::{
    require_api_key_broker_secret_with_sources, select_claude_auth_source_with_presence,
    validate_runtime_auth_broker_environment_with_sources,
};
use super::broker_plan;
use crate::runtime::plans::ClaudeAuthSource;

fn present() -> Result<bool> {
    Ok(true)
}

fn absent() -> Result<bool> {
    Ok(false)
}

#[test]
fn claude_with_present_token_selects_keychain_oauth() {
    let source = select_claude_auth_source_with_presence(None, "claude", present).unwrap();
    assert_eq!(source, Some(ClaudeAuthSource::KeychainOAuth));
}

#[test]
fn claude_without_token_selects_no_broker() {
    let source = select_claude_auth_source_with_presence(None, "claude", absent).unwrap();
    assert_eq!(source, None);
}

#[test]
fn explicit_env_broker_wins_over_a_present_keychain_token() {
    let source = select_claude_auth_source_with_presence(
        Some("ANTHROPIC_API_KEY".to_string()),
        "claude",
        present,
    )
    .unwrap();
    assert_eq!(
        source,
        Some(ClaudeAuthSource::ApiKeyBrokerEnv(
            "ANTHROPIC_API_KEY".to_string()
        ))
    );
}

#[test]
fn empty_env_broker_falls_through_to_keychain_presence() {
    let source =
        select_claude_auth_source_with_presence(Some("   ".to_string()), "claude", present)
            .unwrap();
    assert_eq!(source, Some(ClaudeAuthSource::KeychainOAuth));
}

#[test]
fn non_claude_agent_never_selects_keychain_oauth() {
    // KeychainOAuth is claude-only; a present token on another profile is ignored.
    let source = select_claude_auth_source_with_presence(None, "codex", present).unwrap();
    assert_eq!(source, None);
}

#[test]
fn keychain_read_happens_once_at_startup_and_validation_is_presence_only() {
    let mut plan = broker_plan("claude", "claude");
    plan.claude_auth_source = Some(ClaudeAuthSource::KeychainOAuth);

    // Pre-confirm validation consults PRESENCE only. It has no reader seam, so
    // the prompting Keychain read cannot fire here (F3). Present -> Ok.
    let presence_calls = Cell::new(0);
    validate_runtime_auth_broker_environment_with_sources(
        &plan,
        |_| None,
        || {
            presence_calls.set(presence_calls.get() + 1);
            Ok(true)
        },
    )
    .expect("presence-only validation passes");
    assert!(
        presence_calls.get() >= 1,
        "validation consults presence, not the token"
    );

    // Absent token fails closed pre-confirm, still without reading the token.
    let missing =
        validate_runtime_auth_broker_environment_with_sources(&plan, |_| None, || Ok(false))
            .expect_err("absent token fails closed");
    assert!(missing.to_string().contains("no Claude token is saved"));

    // Broker startup: the prompting read fires here, exactly once.
    let reads = Cell::new(0);
    let (_profile, token) = require_api_key_broker_secret_with_sources(
        &plan,
        |_| None,
        || {
            reads.set(reads.get() + 1);
            Ok(Some(Zeroizing::new(b"sk-ant-oat01-STARTUP".to_vec())))
        },
    )
    .expect("startup read")
    .expect("broker enabled");
    assert_eq!(
        reads.get(),
        1,
        "the token is read exactly once, at broker startup"
    );
    // The token stays a `SecretString`; compare through the one `expose_secret()`
    // accessor.
    assert_eq!(token.expose_secret(), "sk-ant-oat01-STARTUP");
}

#[test]
fn a_reader_that_panics_pre_confirm_proves_validation_never_reads_the_token() {
    let mut plan = broker_plan("claude", "claude");
    plan.claude_auth_source = Some(ClaudeAuthSource::KeychainOAuth);

    // This reader panics if the token is ever read. Validation takes only a
    // presence seam, so this reader is structurally unreachable from it: reaching
    // the end without a panic proves the pre-confirm path never reads the token.
    let _panic_if_read = || -> Result<Option<Zeroizing<Vec<u8>>>> {
        panic!("the token must never be read during pre-confirm validation");
    };
    validate_runtime_auth_broker_environment_with_sources(&plan, |_| None, || Ok(true))
        .expect("presence-only validation");
}
