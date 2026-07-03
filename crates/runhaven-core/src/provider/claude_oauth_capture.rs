//! Host `claude setup-token` capture for the Claude OAuth broker (design C,
//! `docs/design/native-gui-claude-oauth-broker.md`, Section 2). This is the
//! capture mechanism only: run the resolved `claude setup-token` under a host
//! PTY, read its output until the one-year token appears, then hand the token to
//! the Keychain custody surface ([`crate::provider::claude_oauth_keychain`]).
//! Split out of `claude_oauth_keychain.rs` by ownership (custody vs. capture) to
//! stay under the repo structure ceiling.
//!
//! Why a PTY, not a pipe (F4, verified live on macOS 26+): `claude setup-token`
//! is a Claude Code (ink / React-for-terminal) polling flow. With a non-TTY
//! stdout its UI and its OAuth polling loop never run, so the process HANGS and
//! never prints a token. Under a real controlling terminal the user authorizes
//! in the browser (no paste, no stdin) and setup-token polls
//! `platform.claude.com` and prints the token. So the capture gives the child a
//! host PTY (a real TTY on stdout) exactly as `runtime/terminal.rs` does for the
//! in-app terminal. Observed quirk: setup-token does NOT exit promptly after
//! printing the token, so this READS UNTIL THE TOKEN (or a timeout / EOF) and
//! then reaps the child; it never waits for a clean exit, which would hang.
//!
//! Security invariants this module holds:
//!
//! - It runs exactly the caller-resolved absolute `claude` path
//!   ([`resolve_setup_token_binary`] finds it once; the confirmation shows it;
//!   the capture runs that SAME string) with the exact argument list
//!   `["setup-token"]` and never a shell string. There is no second PATH lookup
//!   between the shown and the run binary, so a PATH change after approval cannot
//!   swap the binary (Codex F8).
//! - The captured PTY bytes live only in `Zeroizing` buffers and the extracted
//!   token in a `SecretString` (both overwritten on drop; `SecretString` also
//!   redacts Debug), and reach NO file, log, record, registry, or response; the
//!   only sink is the Keychain store, exposed at the single `expose_secret()`
//!   call in [`capture_and_store_setup_token`]. A source tripwire denies the
//!   known file/log/record sink names in this module's own source.
//! - Every failure path (claude missing, PTY spawn failure, timeout with no
//!   token, buffer cap exceeded, extraction failure, empty/short token, Keychain
//!   store failure) fails closed: nothing is stored and the error names only the
//!   operation, never the captured buffer (the buffer may itself BE the token)
//!   and never a host path.
//!
//! `pty-process` owns all `unsafe` for the PTY syscalls, so nothing here needs
//! `unsafe`; this composes with the workspace `unsafe_code = forbid` lint.

use std::io::Read;
use std::process::Child;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Result, anyhow};
use pty_process::Size;
use pty_process::blocking::{Command, Pty};
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroizing;

use crate::doctor::find_on_path;

/// Overall wall-clock budget for the whole flow: the human opens the browser,
/// authorizes, and setup-token polls `platform.claude.com`. Generous because it
/// spans a human round-trip; on expiry the capture fails closed with nothing
/// stored. This is the read budget, not a wait-for-exit (setup-token lingers
/// after printing the token).
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(180);

/// Hard cap on captured PTY bytes. ink redraws are chatty, but the token and its
/// framing sit far under this; exceeding it means the output is not what this
/// capture expects, so it fails closed rather than grow unbounded.
const MAX_CAPTURE_BYTES: usize = 256 * 1024;

/// PTY geometry wide enough that ink lays the one-year token on a single line, so
/// the common case needs no soft-wrap reconstruction. The extractor still
/// de-wraps defensively.
const CAPTURE_ROWS: u16 = 200;
const CAPTURE_COLS: u16 = 512;

/// Per-read scratch size for the PTY reader.
const READ_CHUNK: usize = 4096;

/// Minimum plausible token length past the `sk-ant-` prefix, matching the prior
/// capture's length guard so a bare prefix is not accepted as a token.
const MIN_TOKEN_LEN: usize = 20;

/// The Anthropic OAuth token prefix (observed `sk-ant-oat01-`).
const TOKEN_PREFIX: &str = "sk-ant-";

/// The more-delimited occurrence: `export CLAUDE_CODE_OAUTH_TOKEN=<token>`.
const ENV_ANCHOR: &[u8] = b"CLAUDE_CODE_OAUTH_TOKEN=";

// Fixed, path-free, buffer-free failure messages. Every failure ends with no
// stored token and one of these; none names a host path or echoes the buffer.
const ERR_NOT_INSTALLED: &str =
    "Claude Code is not installed, so `claude setup-token` cannot run; nothing was saved";
const ERR_START: &str = "could not start Claude sign-in; nothing was saved";
const ERR_RUN: &str = "could not run `claude setup-token`; nothing was saved";
const ERR_NO_TOKEN: &str = "could not read a Claude token; nothing was saved";
const ERR_TIMEOUT: &str = "Claude sign-in did not finish in time; nothing was saved";

/// Resolve the absolute `claude` path RunHaven will run for `setup-token`, from
/// the first PATH match. Surfaced so the sign-in command can SHOW the exact
/// binary in its native confirmation and then run that SAME resolved path: with
/// no PATH re-lookup between the shown path and the run, a PATH change mid-flow
/// (or a poisoned entry appearing after approval) cannot swap the binary the
/// user approved. Fails closed with a fixed, path-free message when `claude` is
/// not on PATH.
pub fn resolve_setup_token_binary() -> Result<String> {
    find_on_path("claude").ok_or_else(|| anyhow!(ERR_NOT_INSTALLED))
}

/// Run the host `claude setup-token` at the caller-resolved `claude` path under a
/// host PTY, capture its token transiently, and store it to the Keychain, keeping
/// no copy anywhere else. `claude` is the exact path
/// [`resolve_setup_token_binary`] returned and the confirmation showed, so there
/// is no TOCTOU between the shown and run binary. Fails closed: on any failure
/// nothing is stored and the error names only the operation, never a host path or
/// the captured buffer.
pub fn capture_and_store_setup_token(claude: &str) -> Result<()> {
    let token = capture_setup_token(claude)?;
    // The one plaintext read of the extracted token: expose its bytes at the
    // single Keychain-store call. Grep `expose_secret` to audit it.
    crate::provider::claude_oauth_keychain::store_token(token.expose_secret().as_bytes())
}

/// Spawn `claude setup-token` on a host PTY, stream its output on a worker
/// thread, and return the extracted token. The worker owns the PTY master and
/// its `Zeroizing` buffer; this thread enforces the wall-clock budget and reaps
/// the child on every path so nothing is left running.
fn capture_setup_token(claude: &str) -> Result<SecretString> {
    let (pty, pts) = pty_process::blocking::open().map_err(|_| anyhow!(ERR_START))?;
    pty.resize(Size::new(CAPTURE_ROWS, CAPTURE_COLS))
        .map_err(|_| anyhow!(ERR_START))?;
    // Exact program (the resolved path) and exact args; never a shell string, no
    // PATH re-lookup here (the path is already resolved by the caller, F8).
    let child = Command::new(claude)
        .arg("setup-token")
        .spawn(pts)
        .map_err(|_| anyhow!(ERR_RUN))?;

    let (tx, rx) = mpsc::channel::<CaptureOutcome>();
    let reader = match thread::Builder::new()
        .name("runhaven-claude-setup-token".to_string())
        .spawn(move || read_until_token(pty, &tx))
    {
        Ok(handle) => handle,
        Err(_) => {
            reap(child);
            return Err(anyhow!(ERR_START));
        }
    };

    let outcome = rx.recv_timeout(CAPTURE_TIMEOUT);
    // Reap the child on every path: on success (setup-token lingers after
    // printing) and on every failure, so nothing is orphaned. Killing the child
    // EOFs the PTY, which unblocks and ends the reader; its `Zeroizing` buffer is
    // wiped when that thread returns. The handle is detached (no join) so a child
    // that ignores the kill cannot hang this thread.
    reap(child);
    drop(reader);

    match outcome {
        Ok(CaptureOutcome::Token(token)) => Ok(token),
        Ok(_) => Err(anyhow!(ERR_NO_TOKEN)),
        Err(_) => Err(anyhow!(ERR_TIMEOUT)),
    }
}

/// Kill and reap `child` so no process is left running. Mirrors the terminal
/// reap: kill, then wait to collect the exit status. Errors are ignored; the
/// child may already be gone.
fn reap(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Outcome the PTY reader hands back. Only [`CaptureOutcome::Token`] carries the
/// secret (in a redacted, zeroize-on-drop `SecretString`); the rest are
/// buffer-free signals mapped to fixed messages by [`capture_setup_token`].
enum CaptureOutcome {
    /// The token was extracted from the accumulated output.
    Token(SecretString),
    /// The child reached EOF without ever producing an extractable token.
    NoToken,
    /// The accumulated output exceeded [`MAX_CAPTURE_BYTES`].
    TooLarge,
    /// A read on the PTY master failed.
    ReadFailed,
}

/// Read the PTY master until the token is extractable (success), the child EOFs
/// (no token), the buffer cap is exceeded, or a read fails. Sends exactly one
/// [`CaptureOutcome`] then returns, dropping (and wiping) its `Zeroizing`
/// buffers. Never persists or logs the stream.
fn read_until_token(pty: Pty, tx: &mpsc::Sender<CaptureOutcome>) {
    let mut buffer = Zeroizing::new(Vec::<u8>::with_capacity(READ_CHUNK));
    let mut scratch = Zeroizing::new(vec![0u8; READ_CHUNK]);
    let mut reader: &Pty = &pty;
    loop {
        match reader.read(&mut scratch) {
            Ok(0) => {
                let _ = tx.send(CaptureOutcome::NoToken);
                return;
            }
            Ok(count) => {
                if buffer.len() + count > MAX_CAPTURE_BYTES {
                    let _ = tx.send(CaptureOutcome::TooLarge);
                    return;
                }
                buffer.extend_from_slice(&scratch[..count]);
                if let Some(token) = extract_token(&buffer) {
                    let _ = tx.send(CaptureOutcome::Token(token));
                    return;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => {
                let _ = tx.send(CaptureOutcome::ReadFailed);
                return;
            }
        }
    }
}

/// A base64url character: `[A-Za-z0-9_-]`. The token body (and the `sk-ant-`
/// prefix's hyphens) are drawn only from this set.
fn is_token_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'
}

/// Extract the Claude OAuth token from raw PTY bytes, or `None` if no complete
/// token is present yet. Strips ANSI/OSC escapes and control bytes first (so ink
/// framing cannot glue prose onto the token), then prefers the delimited
/// `CLAUDE_CODE_OAUTH_TOKEN=` occurrence, else the first `sk-ant-` run. Returns a
/// token only when it is TERMINATED by a real boundary in the de-wrapped stream,
/// so a token still streaming in is not accepted half-formed.
fn extract_token(raw: &[u8]) -> Option<SecretString> {
    let cleaned = strip_ansi(raw);
    // Build the token in a transient `Zeroizing<String>` (like the PTY byte
    // buffers), then wrap it into a `SecretString` at this boundary so the
    // returned credential is redacted and zeroize-on-drop. The transient builder
    // is wiped when it drops at the end of this scope.
    token_from_cleaned(&cleaned).map(|token| SecretString::from(token.as_str()))
}

/// Strip ANSI CSI/OSC escape sequences and control bytes from `raw`, keeping
/// printable bytes and newlines. Each escape sequence and each non-newline
/// control byte becomes a single space so it acts as a BOUNDARY, never silently
/// gluing the token to whatever styled text follows (the observed
/// `sk-ant-...Store` bug came from deleting the reset escape between them).
/// Newlines are kept so the extractor can tell a soft wrap from a real line
/// break; carriage returns are dropped. The result stays in `Zeroizing`.
fn strip_ansi(raw: &[u8]) -> Zeroizing<Vec<u8>> {
    let mut out = Zeroizing::new(Vec::<u8>::with_capacity(raw.len()));
    let mut index = 0;
    while index < raw.len() {
        let byte = raw[index];
        match byte {
            0x1b => {
                index = skip_escape(raw, index);
                out.push(b' ');
            }
            b'\n' => {
                out.push(b'\n');
                index += 1;
            }
            b'\r' => {
                index += 1;
            }
            _ if byte < 0x20 || byte == 0x7f => {
                out.push(b' ');
                index += 1;
            }
            _ => {
                out.push(byte);
                index += 1;
            }
        }
    }
    out
}

/// Return the index just past the escape sequence starting at `start` (`raw[start]
/// == 0x1b`). Handles CSI (`ESC [ ... final 0x40..=0x7e`), OSC (`ESC ] ... BEL` or
/// `ESC \`), and any other two-byte escape.
fn skip_escape(raw: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    if index >= raw.len() {
        return index;
    }
    match raw[index] {
        b'[' => {
            index += 1;
            while index < raw.len() && !(0x40..=0x7e).contains(&raw[index]) {
                index += 1;
            }
            if index < raw.len() {
                index += 1;
            }
        }
        b']' => {
            index += 1;
            while index < raw.len() {
                if raw[index] == 0x07 {
                    index += 1;
                    break;
                }
                if raw[index] == 0x1b && index + 1 < raw.len() && raw[index + 1] == b'\\' {
                    index += 2;
                    break;
                }
                index += 1;
            }
        }
        _ => {
            index += 1;
        }
    }
    index
}

/// Find the token in the cleaned stream: prefer the `CLAUDE_CODE_OAUTH_TOKEN=`
/// occurrence, else the first `sk-ant-` run. Only a token terminated by a real
/// boundary and passing the shape guard is returned.
fn token_from_cleaned(text: &[u8]) -> Option<Zeroizing<String>> {
    if let Some(position) = find_subslice(text, ENV_ANCHOR) {
        let mut start = position + ENV_ANCHOR.len();
        // Skip a former escape-turned-space between `=` and the token.
        while start < text.len() && text[start] == b' ' {
            start += 1;
        }
        if let Some(token) = gather_token(text, start) {
            return Some(token);
        }
    }
    let position = find_subslice(text, TOKEN_PREFIX.as_bytes())?;
    gather_token(text, position)
}

/// Gather a token run starting at `start`, de-wrapping soft line breaks, and
/// return it only if it is terminated by a real boundary and passes the shape
/// guard. A lone `\n` between two token characters is treated as a soft wrap and
/// skipped; any other boundary (space, a former escape, a blank line, a newline
/// before non-token text) ends the run. A run that reaches the end of the buffer
/// is treated as still streaming and rejected, so a half-formed token is never
/// accepted.
fn gather_token(text: &[u8], start: usize) -> Option<Zeroizing<String>> {
    let mut token = Zeroizing::new(String::new());
    let mut index = start;
    let terminated = loop {
        while index < text.len() && is_token_char(text[index]) {
            token.push(text[index] as char);
            index += 1;
        }
        if index >= text.len() {
            break false;
        }
        if text[index] == b'\n' && index + 1 < text.len() && is_token_char(text[index + 1]) {
            // Soft wrap: a single newline between two token characters. Skip it
            // and keep gathering the same token.
            index += 1;
            continue;
        }
        break true;
    };
    if terminated && is_valid_token(&token) {
        Some(token)
    } else {
        None
    }
}

/// Shape guard: the token must carry the Anthropic prefix and be longer than the
/// prefix alone, so a bare `sk-ant-` or a stray short run is rejected.
fn is_valid_token(token: &str) -> bool {
    token.starts_with(TOKEN_PREFIX) && token.len() > MIN_TOKEN_LEN
}

/// First index of `needle` in `haystack`, or `None`.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod source_tripwire_tests;
