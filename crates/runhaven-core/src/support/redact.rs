//! Best-effort redaction for raw agent run output before it reaches a user
//! surface (GUI log snapshot, CLI `runs logs`).
//!
//! This is only for the ONE unstructured surface: verbatim agent stdout/stderr,
//! which can echo an injected token or a printed API key. Structured records and
//! diagnostics already fail closed through field allowlisting and must NOT be
//! routed through this scrubber; a regex pass there would add false confidence
//! and re-parsing risk without benefit.
//!
//! It removes terminal control sequences and well-known secret token shapes. It
//! is defense in depth, not a guarantee: callers still warn the viewer that raw
//! output may contain secrets.

use std::sync::LazyLock;

use regex::Regex;

const REDACTED: &str = "[redacted]";

// Terminal control: ANSI CSI/OSC and other escape sequences, plus C0/DEL control
// bytes except tab, newline, and carriage return so log text stays multi-line.
static CONTROL_SEQUENCES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]|[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]",
    )
    .expect("valid control-sequence regex")
});

// Well-known secret token shapes for the agents RunHaven supports. Ordered so a
// more specific prefix is redacted before a broader pattern can partially match.
static SECRET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"sk-ant-[A-Za-z0-9_-]{10,}",  // Anthropic API keys and OAuth tokens
        r"sk-[A-Za-z0-9]{20,}",        // OpenAI-style API keys
        r"gh[opsru]_[A-Za-z0-9]{20,}", // GitHub tokens (Copilot)
        r"AIza[A-Za-z0-9_-]{20,}",     // Google API keys (Antigravity)
        r"ya29\.[A-Za-z0-9_-]{20,}",   // Google OAuth access tokens (Antigravity device flow)
        r"1//[A-Za-z0-9_-]{20,}",      // Google OAuth refresh tokens
        r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}", // JWTs (OAuth/device-code)
        r"(?i)bearer\s+[A-Za-z0-9._~+/-]{16,}={0,2}", // Authorization: Bearer <token>
    ]
    .into_iter()
    .map(|pattern| Regex::new(pattern).expect("valid secret regex"))
    .collect()
});

/// Strip terminal control sequences and redact well-known secret token shapes
/// from raw run output. Preserves ordinary text, tabs, and line structure.
pub fn redact_run_output(text: &str) -> String {
    let mut out = CONTROL_SEQUENCES.replace_all(text, "").into_owned();
    for pattern in SECRET_PATTERNS.iter() {
        out = pattern.replace_all(&out, REDACTED).into_owned();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_known_secret_token_shapes() {
        let cases = [
            "sk-ant-api03-abcdefghij_KLmnopqrst-uvwxyz1234",
            "sk-ant-oat01-ABCDEFGHIJ1234567890abcdefghij",
            "sk-proj0123456789ABCDEFabcdef",
            "ghp_ABCDEFGHIJ0123456789abcdefghij",
            "gho_ABCDEFGHIJ0123456789abcdefghij",
            "AIzaSyABCDEFGHIJ0123456789_abcdefghij",
            "ya29.A0ARrdaM-abcdefghij0123456789_KLmnop",
            "1//0abcdefghij0123456789_KLmnopqrstuv",
            "eyJhbGciOiJIUzI1.eyJzdWIiOiIxMjM0NTY.SflKxwRJSMeKKF2QT4f",
        ];
        for secret in cases {
            let line = format!("agent said: {secret} done");
            let redacted = redact_run_output(&line);
            assert!(
                !redacted.contains(secret),
                "secret leaked through redaction: {secret}"
            );
            assert!(redacted.contains("[redacted]"), "no marker for {secret}");
        }
    }

    #[test]
    fn redacts_bearer_authorization_tokens() {
        let redacted = redact_run_output("Authorization: Bearer abc123DEF456ghi789JKL0mno.pqr-stu");
        assert!(!redacted.contains("abc123DEF456ghi789JKL0mno"));
        assert!(redacted.contains("[redacted]"));
    }

    #[test]
    fn strips_terminal_controls_but_keeps_lines_and_tabs() {
        let raw = "line one\x1b[31mred\x1b[0m\ncol\tsplit\x07\x00\nlast";
        let cleaned = redact_run_output(raw);
        assert!(!cleaned.contains('\x1b'));
        assert!(!cleaned.contains('\x07'));
        assert!(!cleaned.contains('\x00'));
        assert_eq!(cleaned, "line onered\ncol\tsplit\nlast");
    }

    #[test]
    fn preserves_ordinary_output_without_false_positives() {
        let raw = "Building project sk-module and task-runner; ok in 12s.\nsk-1 short";
        assert_eq!(redact_run_output(raw), raw);
    }
}
