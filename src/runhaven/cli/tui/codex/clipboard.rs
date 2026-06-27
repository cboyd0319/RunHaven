//! OSC 52 terminal-clipboard write path.
//!
//! Derived from openai/codex (`codex-rs/tui/src/clipboard_copy.rs`), licensed
//! under Apache-2.0, copyright 2025 OpenAI. Modified by RunHaven on 2026-06-26:
//! only the OSC 52 escape-emission path was extracted (`osc52_sequence`,
//! `write_osc52_to_writer`, `osc52_copy`, and the `OSC52_MAX_RAW_BYTES` guard).
//! The native/`arboard`, WSL/PowerShell, and tmux clipboard backends, the
//! `CopyEnvironment` selection logic, and the `crate::clipboard_paste` coupling
//! were excluded. The upstream `tracing::debug!` fallback logging was removed.
//!
//! The full license text is in `licenses/codex-Apache-2.0.txt` and the required
//! attribution notice is in `THIRD_PARTY_NOTICES.md` at the repo root.

use base64::Engine;
use std::io::Write;

/// Maximum raw bytes we will base64-encode into an OSC 52 sequence.
/// Large payloads are rejected before encoding to avoid overwhelming the terminal.
pub(crate) const OSC52_MAX_RAW_BYTES: usize = 100_000;

/// Write text to the clipboard via the OSC 52 terminal escape sequence.
pub(crate) fn osc52_copy(text: &str) -> Result<(), String> {
    let sequence = osc52_sequence(text, std::env::var_os("TMUX").is_some())?;
    #[cfg(unix)]
    {
        match std::fs::OpenOptions::new().write(true).open("/dev/tty") {
            Ok(tty) => match write_osc52_to_writer(tty, &sequence) {
                Ok(()) => return Ok(()),
                Err(_err) => {
                    // Fall back to stdout. (Codex logged this via tracing::debug!;
                    // that logging coupling is removed in RunHaven.)
                }
            },
            Err(_err) => {
                // Fall back to stdout. (Codex logged this via tracing::debug!;
                // that logging coupling is removed in RunHaven.)
            }
        }
    }

    write_osc52_to_writer(std::io::stdout().lock(), &sequence)
}

pub(crate) fn write_osc52_to_writer(mut writer: impl Write, sequence: &str) -> Result<(), String> {
    writer
        .write_all(sequence.as_bytes())
        .map_err(|e| format!("failed to write OSC 52: {e}"))?;
    writer
        .flush()
        .map_err(|e| format!("failed to flush OSC 52: {e}"))
}

pub(crate) fn osc52_sequence(text: &str, tmux: bool) -> Result<String, String> {
    let raw_bytes = text.len();
    if raw_bytes > OSC52_MAX_RAW_BYTES {
        return Err(format!(
            "OSC 52 payload too large ({raw_bytes} bytes; max {OSC52_MAX_RAW_BYTES})"
        ));
    }

    let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
    if tmux {
        Ok(format!("\x1bPtmux;\x1b\x1b]52;c;{encoded}\x07\x1b\\"))
    } else {
        Ok(format!("\x1b]52;c;{encoded}\x07"))
    }
}

#[cfg(test)]
mod tests {
    use super::OSC52_MAX_RAW_BYTES;
    use super::osc52_sequence;
    use super::write_osc52_to_writer;

    #[test]
    fn osc52_encoding_roundtrips() {
        use base64::Engine;
        let text = "# Hello\n\n```rust\nfn main() {}\n```\n";
        let sequence = osc52_sequence(text, /*tmux*/ false).expect("OSC 52 sequence");
        let encoded = sequence
            .trim_start_matches("\u{1b}]52;c;")
            .trim_end_matches('\u{7}');
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        assert_eq!(decoded, text.as_bytes());
    }

    #[test]
    fn osc52_rejects_payload_larger_than_limit() {
        let text = "x".repeat(OSC52_MAX_RAW_BYTES + 1);
        assert_eq!(
            osc52_sequence(&text, /*tmux*/ false),
            Err(format!(
                "OSC 52 payload too large ({} bytes; max {OSC52_MAX_RAW_BYTES})",
                OSC52_MAX_RAW_BYTES + 1
            ))
        );
    }

    #[test]
    fn osc52_wraps_tmux_passthrough() {
        assert_eq!(
            osc52_sequence("hello", /*tmux*/ true),
            Ok("\u{1b}Ptmux;\u{1b}\u{1b}]52;c;aGVsbG8=\u{7}\u{1b}\\".to_string())
        );
    }

    #[test]
    fn write_osc52_to_writer_emits_sequence_verbatim() {
        let sequence = "\u{1b}]52;c;aGVsbG8=\u{7}";
        let mut output = Vec::new();
        assert_eq!(write_osc52_to_writer(&mut output, sequence), Ok(()));
        assert_eq!(output, sequence.as_bytes());
    }
}
