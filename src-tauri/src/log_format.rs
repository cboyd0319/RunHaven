//! Diagnostic-log line formatting for the desktop app's `tauri-plugin-log` sink.
//!
//! Two jobs, both belt-and-suspenders on top of the global level cap in
//! `lib.rs` (DEBUG in dev, INFO in release) and the http/tls WARN pins that
//! stop wire dumps at the source:
//!
//! 1. Reproduce the plugin's CURRENT human layout so switching to a custom
//!    `.format(...)` does not change what a reader sees:
//!    `[YYYY-MM-DD][HH:MM:SS][target][level] message` (UTC, matching the
//!    plugin default's desktop format and `UseUtc` timezone strategy).
//! 2. Scrub any residual secret SHAPES from the final message text as a last
//!    line of defense. This is NOT a replacement for the level cap: the cap
//!    still prevents wire dumps; this only catches a secret that a future bug
//!    might route into an app-level log string. It targets the APP-LEVEL secret
//!    shapes (five of the six triggers `scripts/scan-dev-log.sh` greps:
//!    `sk-ant-`, `bearer`, the `authorization`/`x-api-key` headers, and the
//!    `*_TOKEN=`/`*_API_KEY=` assignments), removing the header name / token
//!    prefix (not just the value) so those literal triggers cannot survive. The
//!    sixth trigger -- the raw hex wire-dump form (`4265 6172 6572`, "Bearer")
//!    -- is NOT redacted here; it only appears at TRACE, which the level cap
//!    blocks, so the cap (not redaction) owns that shape. Redaction is
//!    shape-based, so a marker-less bare token (a raw JWT with no `bearer`/
//!    header/assignment context) is a known belt-and-suspenders gap; the cap,
//!    the SecretString type, and the no-console guard are the real guarantees.
//!
//! Redaction uses simple byte scanning, not `regex`: src-tauri does not depend
//! on `regex`, and the shape set here is small and fixed. This is deliberately
//! aggressive (over-redaction is safe for a diagnostic trail): the literal word
//! "bearer " followed by token-shaped text is always redacted, which is correct
//! for RunHaven's own log lines where "bearer" only ever precedes a token.

use time::OffsetDateTime;

/// Fixed marker written in place of any redacted secret shape. Contains no
/// character class the dev-log scan greps for, so it can never re-trip the scan.
const REDACTED: &str = "[REDACTED]";

/// Render the UTC timestamp prefix exactly as the plugin default does on
/// desktop: `[year-month-day][hour:minute:second]`.
pub(crate) fn utc_log_timestamp() -> String {
    let now = OffsetDateTime::now_utc();
    format!(
        "[{:04}-{:02}-{:02}][{:02}:{:02}:{:02}]",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    )
}

/// Scrub residual secret shapes from a finished log message. Ordering: consume
/// full `authorization:`/`x-api-key:` header lines first (they can carry a
/// bearer token), then standalone `bearer <token>`, then bare `sk-ant-` tokens,
/// then `*_TOKEN=`/`*_API_KEY=` assignments.
pub(crate) fn redact_log_line(message: &str) -> String {
    let mut out = redact_header(message, "authorization");
    out = redact_header(&out, "x-api-key");
    out = redact_prefixed_token(&out, "bearer ", is_bearer_value_byte);
    out = redact_prefixed_token(&out, "sk-ant-", is_token_value_byte);
    redact_secret_assignment(&out)
}

/// Case-insensitive ASCII substring search from byte offset `from`. Returns the
/// match start. Safe on UTF-8: an ASCII needle only matches at ASCII bytes,
/// which are always char boundaries (continuation bytes are >= 0x80).
fn find_ci(haystack: &str, needle_lower: &str, from: usize) -> Option<usize> {
    let hay = haystack.as_bytes();
    let need = needle_lower.as_bytes();
    if need.is_empty() || hay.len() < need.len() {
        return None;
    }
    let mut i = from;
    while i + need.len() <= hay.len() {
        if hay[i..i + need.len()]
            .iter()
            .zip(need)
            .all(|(h, n)| h.to_ascii_lowercase() == *n)
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn is_token_value_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

fn is_bearer_value_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b'+' | b'/' | b'-' | b'=')
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Replace `needle` (case-insensitive) plus the following value-byte run with
/// `[REDACTED]`. Used for `bearer <token>` and bare `sk-ant-<token>`: the whole
/// match, prefix included, is removed so no trigger substring survives.
fn redact_prefixed_token(input: &str, needle_lower: &str, is_value: fn(u8) -> bool) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while let Some(pos) = find_ci(input, needle_lower, i) {
        out.push_str(&input[i..pos]);
        let mut end = pos + needle_lower.len();
        while end < bytes.len() && is_value(bytes[end]) {
            end += 1;
        }
        out.push_str(REDACTED);
        i = end;
    }
    out.push_str(&input[i..]);
    out
}

/// Replace an `authorization:`/`x-api-key:` header (case-insensitive), from the
/// header name through end of line, with `[REDACTED]`. Removing the name too
/// means the scan's `x-api-key: ` / `authorization: bearer` literals cannot
/// remain. Over-redacts to end of line on purpose: such a header should never
/// reach an app log at all, so eating trailing context is the safe choice.
fn redact_header(input: &str, header_lower: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while let Some(pos) = find_ci(input, header_lower, i) {
        let after = pos + header_lower.len();
        if after < bytes.len() && bytes[after] == b':' {
            out.push_str(&input[i..pos]);
            let mut end = after;
            while end < bytes.len() && bytes[end] != b'\n' {
                end += 1;
            }
            out.push_str(REDACTED);
            i = end;
        } else {
            // A bare substring, not a header; keep it and keep scanning.
            out.push_str(&input[i..after]);
            i = after;
        }
    }
    out.push_str(&input[i..]);
    out
}

/// Replace `NAME=value` where `NAME` ends in `_TOKEN` or `_API_KEY`
/// (case-insensitive) with `[REDACTED]`, name included, so `_TOKEN=<alnum>` and
/// `_API_KEY=<alnum>` cannot appear. Value is the non-whitespace run after `=`.
fn redact_secret_assignment(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut copied = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'=' {
            let mut name_start = i;
            while name_start > 0 && is_ident_byte(bytes[name_start - 1]) {
                name_start -= 1;
            }
            let name_lower = input[name_start..i].to_ascii_lowercase();
            if !name_lower.is_empty()
                && (name_lower.ends_with("_token") || name_lower.ends_with("_api_key"))
            {
                let mut end = i + 1;
                while end < bytes.len() && !bytes[end].is_ascii_whitespace() {
                    end += 1;
                }
                out.push_str(&input[copied..name_start]);
                out.push_str(REDACTED);
                copied = end;
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out.push_str(&input[copied..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_matches_plugin_bracket_layout() {
        let ts = utc_log_timestamp();
        // `[YYYY-MM-DD][HH:MM:SS]` -> two bracket groups, 22 ASCII bytes.
        assert_eq!(ts.len(), 22, "unexpected timestamp width: {ts}");
        assert!(ts.starts_with('['));
        assert_eq!(ts.matches('[').count(), 2);
        assert_eq!(ts.matches(']').count(), 2);
    }

    #[test]
    fn redacts_bare_sk_ant_token() {
        let out = redact_log_line("run x: token sk-ant-api03-ABCdef_ghij-klmnop done");
        assert!(!out.contains("sk-ant-"), "sk-ant- leaked: {out}");
        assert!(out.contains("[REDACTED]"));
        assert!(out.contains("run x:") && out.ends_with("done"));
    }

    #[test]
    fn redacts_bearer_token() {
        let out = redact_log_line("Authorization uses bearer abc123.DEF-456/ghi=");
        assert!(
            !out.to_lowercase().contains("bearer abc"),
            "bearer leaked: {out}"
        );
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_api_key_and_authorization_headers() {
        let out = redact_log_line("x-api-key: super-secret-value");
        assert!(!out.contains("x-api-key: "), "header literal leaked: {out}");
        assert!(!out.contains("super-secret-value"));
        assert!(out.contains("[REDACTED]"));

        let auth = redact_log_line("authorization: Bearer abc.def-ghi");
        assert!(!auth.to_lowercase().contains("authorization: bearer"));
        assert!(auth.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_token_and_api_key_assignments() {
        let out =
            redact_log_line("env GITHUB_TOKEN=ghp_plain123 ANTHROPIC_API_KEY=plainsecret rest");
        let lower = out.to_lowercase();
        assert!(!lower.contains("_token="), "_TOKEN= leaked: {out}");
        assert!(!lower.contains("_api_key="), "_API_KEY= leaked: {out}");
        assert!(!out.contains("ghp_plain123"));
        assert!(!out.contains("plainsecret"));
        assert!(out.contains("rest"));
    }

    #[test]
    fn scrubbed_line_carries_no_dev_log_scan_trigger() {
        // Mirror scripts/scan-dev-log.sh: the finished line must contain none of
        // its literal triggers after redaction.
        let raw = "leak sk-ant-oat01-ABCDEFGHIJ x-api-key: v1 authorization: Bearer t0ken \
                   FOO_TOKEN=abc123 BAR_API_KEY=xyz789";
        let out = redact_log_line(raw).to_lowercase();
        assert!(!out.contains("sk-ant-"));
        assert!(!out.contains("x-api-key: "));
        assert!(!out.contains("authorization: bearer"));
        assert!(!out.contains("_token=a"));
        assert!(!out.contains("_api_key=x"));
    }

    #[test]
    fn preserves_ordinary_log_text() {
        let raw = "run 01J8ABC: container process running; exit code 0 in 1.2s";
        assert_eq!(redact_log_line(raw), raw);
    }
}
