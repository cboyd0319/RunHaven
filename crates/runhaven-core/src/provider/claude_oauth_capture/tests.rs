//! Hermetic unit tests for the host `claude setup-token` capture. The PTY spawn
//! and Keychain store are exercised by the maintainer live smoke (F4), not here;
//! these cover the token extractor, its ANSI/wrap/glue handling, and the fixed,
//! buffer-free failure messages.

use super::*;

/// A realistic ink-formatted setup-token screen: bold success line, the
/// "Your OAuth token (valid for 1 year):" occurrence, the delimited
/// `export CLAUDE_CODE_OAUTH_TOKEN=` occurrence, and a trailing sentence. Both
/// token occurrences are soft-wrapped across a line (a bare `\n` mid-token) and
/// end with a reset escape, and the sentence "Store this token securely." is
/// glued on right after the reset. This is the exact shape that broke a naive
/// strip (it glued `sk-ant-...Store`).
const EXPECTED_TOKEN: &str = "sk-ant-oat01-AAAABBBBCCCCDDDDEEEEFFFF";

fn ink_sample() -> Vec<u8> {
    let mut sample = String::new();
    sample.push_str("\x1b[1mLong-lived authentication token created successfully!\x1b[0m\r\n");
    sample.push_str("\r\n");
    sample.push_str("\x1b[2mYour OAuth token (valid for 1 year):\x1b[0m\r\n");
    // First occurrence, soft-wrapped mid-token, reset then glued sentence.
    sample.push_str("\x1b[32msk-ant-oat01-AAAABBBBCCCC\r\n");
    sample.push_str("DDDDEEEEFFFF\x1b[0mStore this token securely.\r\n");
    sample.push_str("\r\n");
    sample.push_str("\x1b[2mSet this as an environment variable:\x1b[0m\r\n");
    // Preferred occurrence: export line, color escape between `=` and the token,
    // soft-wrapped mid-token, reset then glued sentence.
    sample.push_str("export CLAUDE_CODE_OAUTH_TOKEN=\x1b[32msk-ant-oat01-AAAABBBBCCCC\r\n");
    sample
        .push_str("DDDDEEEEFFFF\x1b[0mStore this token securely. It will not be shown again.\r\n");
    sample.into_bytes()
}

#[test]
fn extracts_the_exact_token_from_a_wrapped_ansi_ink_sample() {
    let token = extract_token(&ink_sample()).expect("token");
    // The extractor now returns a `SecretString`; read the plaintext through the
    // one auditable `expose_secret()` accessor for the value comparison.
    let token = token.expose_secret();
    assert_eq!(token, EXPECTED_TOKEN, "the exact de-wrapped token");
    assert!(
        !token.contains("Store"),
        "must not swallow the glued trailing sentence"
    );
    assert!(
        !token.contains(' ') && !token.contains('\n'),
        "no whitespace or newline may survive in the token"
    );
}

#[test]
fn prefers_the_env_var_occurrence_but_both_yield_the_same_token() {
    // The sample carries both the "Your OAuth token:" run and the
    // `CLAUDE_CODE_OAUTH_TOKEN=` run; either path must land the same value.
    let only_plain = b"\x1b[32msk-ant-oat01-AAAABBBBCCCC\r\nDDDDEEEEFFFF\x1b[0m\r\n".as_slice();
    let token = extract_token(only_plain).expect("token from the plain occurrence");
    assert_eq!(token.expose_secret(), EXPECTED_TOKEN);
}

#[test]
fn rejects_a_half_streamed_token_until_a_boundary_arrives() {
    // A chunk that ends mid-token (no trailing boundary) must not be accepted,
    // or the reader would store a truncated token.
    let partial = b"export CLAUDE_CODE_OAUTH_TOKEN=sk-ant-oat01-AAAABBBBCCCC".as_slice();
    assert!(
        extract_token(partial).is_none(),
        "a token with no terminating boundary is still streaming"
    );
    // The same bytes plus a boundary now extract.
    let mut whole = partial.to_vec();
    whole.extend_from_slice(b"\n");
    // Still no continuation after the newline -> the newline terminates it.
    let token = extract_token(&whole).expect("token once terminated");
    assert_eq!(token.expose_secret(), "sk-ant-oat01-AAAABBBBCCCC");
}

#[test]
fn rejects_blank_garbage_and_a_bare_prefix() {
    for buffer in [
        b"".as_slice(),
        b"   \r\n\r\n".as_slice(),
        b"totally-unrelated-output\r\n".as_slice(),
        b"\x1b[32msk-ant-\x1b[0m\r\n".as_slice(),
    ] {
        assert!(
            extract_token(buffer).is_none(),
            "no token must be extracted from non-token output"
        );
    }
}

#[test]
fn the_no_token_failure_message_is_fixed_and_never_echoes_the_buffer() {
    // The reader maps a no-token outcome to ERR_NO_TOKEN. Prove that message
    // carries no host path and cannot echo a captured buffer that might BE the
    // token.
    let buffer = b"sk-ant-oat01-SECRETLOOKINGBUTINCOMPLETE";
    assert!(extract_token(buffer).is_none());
    assert_eq!(
        ERR_NO_TOKEN,
        "could not read a Claude token; nothing was saved"
    );
    let printable = String::from_utf8_lossy(buffer);
    assert!(
        !ERR_NO_TOKEN.contains(printable.trim()),
        "the fixed error must not echo the captured buffer"
    );
}

#[test]
fn all_failure_messages_are_path_free() {
    // No fixed failure message may leak a filesystem path; each maps a fail-closed
    // outcome to plain language only.
    for message in [
        ERR_NOT_INSTALLED,
        ERR_START,
        ERR_RUN,
        ERR_NO_TOKEN,
        ERR_TIMEOUT,
    ] {
        assert!(!message.is_empty());
        assert!(!message.contains('/'), "no path in `{message}`");
        assert!(!message.contains('\\'), "no path in `{message}`");
    }
}

#[test]
fn strip_ansi_turns_escapes_into_boundaries_not_glue() {
    // A reset escape between the token and following text must become a boundary
    // so the token cannot glue onto prose.
    let cleaned = strip_ansi(b"sk-ant-oat01-AAAA\x1b[0mStore");
    let text = String::from_utf8(cleaned.to_vec()).expect("ascii");
    assert!(
        text.contains("AAAA Store"),
        "the escape must become a space boundary, got {text:?}"
    );
}

#[test]
fn resolve_setup_token_binary_reports_a_path_free_error_when_missing() {
    // Not asserting resolution (PATH-dependent); assert the miss message is the
    // fixed, path-free one the confirmation relies on.
    assert_eq!(
        ERR_NOT_INSTALLED,
        "Claude Code is not installed, so `claude setup-token` cannot run; nothing was saved"
    );
}
