//! Non-secret age of RunHaven's stored Claude OAuth token (design C,
//! `docs/design/native-gui-claude-oauth-broker.md`, Section 1 Hardening 4).
//!
//! `claude setup-token` mints a static one-year token with no refresh, so
//! RunHaven owns aging it. The age is derived ONLY from the Keychain item's own
//! non-secret `kSecAttrCreationDate`, read through the attributes-only presence
//! query (`claude_oauth_keychain::token_presence`) that never requests the
//! secret data and so never triggers the user-presence prompt. This module is
//! pure date math over that non-secret creation-date string: it never sees the
//! token, and its input is a calendar date, never a token-derived value.
//!
//! The parser is deliberately strict. It accepts only the fixed form the
//! Keychain emits (`YYYY-MM-DD HH:MM:SS ±HHMM`, always `+0000` in practice); a
//! missing offset or any other malformed shape yields `None` ("age unknown")
//! rather than a guess, so a format drift can never raise a false "your token
//! is old" prompt.

use time::OffsetDateTime;

/// Whole days since `cdat` (the Keychain item's `kSecAttrCreationDate`), or
/// `None` when the date cannot be parsed. A future creation date (clock skew)
/// reports `0`, never a negative or wrapped age, so it is never treated as old.
pub(crate) fn age_days_from_cdat(cdat: &str) -> Option<u32> {
    let created = parse_utc_cdat(cdat)?;
    let elapsed = OffsetDateTime::now_utc() - created;
    u32::try_from(elapsed.whole_days().max(0)).ok()
}

/// Parse the Keychain creation-date string. Only the fixed UTC form the macOS
/// Security framework prints for a `CFDate` attribute is accepted:
/// `YYYY-MM-DD HH:MM:SS +0000`. Anything else returns `None`.
fn parse_utc_cdat(cdat: &str) -> Option<OffsetDateTime> {
    // Built at runtime (no `time` macros feature needed). The offset component
    // is mandatory-signed so only an explicit `+0000`/`-HHMM` form matches; a
    // date with no offset would fail to parse and read as "age unknown".
    let format = time::format_description::parse_borrowed::<2>(
        "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]",
    )
    .ok()?;
    OffsetDateTime::parse(cdat.trim(), &format).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_fixed_utc_keychain_form() {
        // The exact shape observed from a live `kSecAttrCreationDate` read.
        let created = parse_utc_cdat("2026-07-02 20:35:21 +0000").expect("parses");
        assert_eq!(created.year(), 2026);
        assert_eq!(created.month() as u8, 7);
        assert_eq!(created.day(), 2);
        assert_eq!(created.hour(), 20);
        assert_eq!(created.offset(), time::UtcOffset::UTC);
    }

    #[test]
    fn a_clearly_old_creation_date_yields_a_large_positive_age() {
        // Well over a decade in the past: the age must be a large day count,
        // proving the cadence prompt can fire.
        let age = age_days_from_cdat("2000-01-01 00:00:00 +0000").expect("age");
        assert!(age > 9_000, "expected a multi-thousand-day age, got {age}");
    }

    #[test]
    fn a_future_creation_date_is_never_negative_or_old() {
        // Clock skew (a creation date in the far future) reports 0, never a
        // wrapped or negative age, so it is never treated as past the cadence.
        assert_eq!(age_days_from_cdat("2999-01-01 00:00:00 +0000"), Some(0));
    }

    #[test]
    fn malformed_or_offsetless_dates_read_as_unknown() {
        for bad in [
            "",
            "2026-07-02",          // date only
            "2026-07-02 20:35:21", // no offset (fails the mandatory-sign offset)
            "not-a-date",
            "20260702203521",
        ] {
            // Every malformed or offset-less input must read as "age unknown"
            // (None) rather than a guessed age, so a format drift never fires a
            // false cadence prompt.
            assert_eq!(
                age_days_from_cdat(bad),
                None,
                "malformed `{bad}` must read as unknown"
            );
        }
    }

    #[test]
    fn age_is_non_secret_a_token_shaped_input_is_not_an_age() {
        // The age source is a calendar date, never the token. Feeding a
        // token-shaped string proves the age function is a date parser: it
        // cannot pass a token value through as an "age". It returns None, so no
        // token byte can ever be surfaced as a day count.
        assert_eq!(
            age_days_from_cdat("sk-ant-oat01-SECRET-must-never-become-an-age"),
            None
        );
    }
}
