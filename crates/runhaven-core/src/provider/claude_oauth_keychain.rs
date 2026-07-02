//! macOS Keychain custody for RunHaven's OWN Claude OAuth token (Design C,
//! `docs/design/native-gui-claude-oauth-broker.md`, Slice 1). This is the
//! custody surface only: store, read, and clear a single RunHaven-owned
//! Keychain item, plus the host `claude setup-token` capture that fills it. The
//! broker proxy profile, the `ClaudeAuthSource` selector, the Tauri command,
//! and the GUI are later slices and are NOT built here.
//!
//! Security invariants this module holds (see the design's claim set):
//!
//! - It touches ONLY the item at (service `RunHaven Claude OAuth`, account
//!   `claude`). Every read/delete query is an exact service + account match; it
//!   never issues a match-all or account-only query, so "reads only the one
//!   item it created" is true by construction (claim 2).
//! - On the release data-protection keychain the stored item carries a
//!   `SecAccessControl` with the user-presence flag, so a read triggers Touch
//!   ID / Apple Watch / device password and RunHaven cannot read the token
//!   silently. On an unsigned dev build the legacy-keychain fallback
//!   (`select_backend`) may not enforce that gate; it is a live-probe item
//!   (F3), so the silent-read guarantee is scoped to signed release builds
//!   until proven on the legacy path.
//! - The item is `AccessibleWhenUnlockedThisDeviceOnly`: readable only while
//!   the keychain is unlocked, never synced to iCloud or copied by backup.
//! - The setup-token capture holds the transient stdout and the extracted
//!   token in `Zeroizing` buffers overwritten on drop, and keeps NO persistent
//!   copy anywhere but the Keychain. On any parse failure it emits only the
//!   operation and the agent, never the captured buffer (the buffer may BE the
//!   token). A source tripwire test denies a list of known file/log/record sink
//!   names in this module's own source (slice-1, single-file scope; not a proof
//!   that any conceivable sink is impossible). Later slices that add a token
//!   reader need their own review or scan.
//!
//! `security-framework`'s safe wrappers own all FFI, so nothing here needs
//! `unsafe`; this composes with the workspace `unsafe_code = forbid` lint.

use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow, bail};
use security_framework::access_control::{ProtectionMode, SecAccessControl};
use security_framework::item::{ItemClass, ItemSearchOptions, Limit};
use security_framework::passwords::{
    AccessControlOptions, PasswordOptions, delete_generic_password_options, generic_password,
    set_generic_password_options,
};
use zeroize::Zeroizing;

use crate::doctor::find_on_path;

/// RunHaven-owned service label. Distinct from any label Claude Code uses, so a
/// query for this service can only match RunHaven's own item.
const SERVICE: &str = "RunHaven Claude OAuth";

/// Account within the service. Fixed today; room for future per-agent items.
const ACCOUNT: &str = "claude";

/// Apple `errSecItemNotFound` OSStatus: no matching Keychain item. Stable
/// Security-framework status; kept local to avoid a second direct dependency on
/// `security-framework-sys` just for the constant (matches the deleted
/// `token_store.rs` convention at `60e37db^`).
const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

/// Result of clearing the stored token, so callers report the same outcome
/// however the item was (or was not) present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeychainClear {
    /// A stored item existed and was removed.
    Cleared,
    /// There was no stored item to remove.
    Missing,
}

/// Which keychain the item lives in. The data-protection keychain's silent
/// cross-app isolation needs entitlements only a signed, notarized release
/// build carries (design Section 1, F3), so the backend is chosen by build
/// profile: release uses the data-protection keychain, an unsigned dev build
/// uses the legacy file keychain. The user-presence `SecAccessControl` is
/// attached in BOTH; only the cross-app isolation claim is scoped to release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeychainBackend {
    /// Data-protection keychain (`kSecUseDataProtectionKeychain`). Release.
    DataProtection,
    /// Legacy file keychain. Dev-build fallback for unsigned binaries.
    LegacyFile,
}

/// Deterministic backend for this build, identical across store, read, and
/// delete so an item stored in one process is found in another. Release
/// (`debug_assertions` off, the profile a notarized build uses) selects the
/// data-protection keychain; a dev build selects the legacy file keychain.
///
/// F3 (live probe, maintainer step): confirm on a real unsigned dev binary and
/// a real signed release binary that this split behaves as designed, including
/// whether the user-presence `SecAccessControl` binds on the legacy keychain.
const fn select_backend() -> KeychainBackend {
    if cfg!(debug_assertions) {
        KeychainBackend::LegacyFile
    } else {
        KeychainBackend::DataProtection
    }
}

/// Base query for RunHaven's item: exactly the generic-password class plus the
/// exact service and account, targeting the selected keychain. This is the ONE
/// place service and account enter a query, so every operation pins both
/// attributes and none can widen to a match-all (claim 2).
fn base_options(backend: KeychainBackend) -> PasswordOptions {
    let mut options = PasswordOptions::new_generic_password(SERVICE, ACCOUNT);
    if backend == KeychainBackend::DataProtection {
        options.use_protected_keychain();
    }
    options
}

/// The user-presence access control for the stored item:
/// `AccessibleWhenUnlockedThisDeviceOnly` protection plus the user-presence
/// flag (Touch ID / Apple Watch / device password on every read).
///
/// User presence is chosen over biometry-only flags so a Mac without Touch ID,
/// or with biometry off, can still approve with the device password (design
/// Section 1: "secure path is the easy path"), while every read still forces a
/// live approval.
fn user_presence_access_control() -> Result<SecAccessControl> {
    SecAccessControl::create_with_protection(
        Some(ProtectionMode::AccessibleWhenUnlockedThisDeviceOnly),
        AccessControlOptions::USER_PRESENCE.bits(),
    )
    .map_err(|err| {
        anyhow!(
            "could not build the Claude token access control (status {})",
            err.code()
        )
    })
}

/// Add a generic-password item for `(service, account)` with an optional access
/// control. Overwrites any existing value (the caller deletes first for a clean
/// replace). Errors carry only the operation, never the secret.
fn put(backend: KeychainBackend, access: Option<SecAccessControl>, secret: &[u8]) -> Result<()> {
    let mut options = base_options(backend);
    if let Some(access) = access {
        options.set_access_control(access);
    }
    set_generic_password_options(secret, options)
        .map_err(anyhow::Error::new)
        .context("could not save the Claude token to the Keychain")
}

/// Read the item for `(service, account)`. A stored user-presence item triggers
/// the LocalAuthentication prompt here. `Ok(None)` means no item exists; the
/// bytes are wrapped in `Zeroizing` so the caller's copy is overwritten on
/// drop. Errors carry only the operation.
fn get(backend: KeychainBackend) -> Result<Option<Zeroizing<Vec<u8>>>> {
    match generic_password(base_options(backend)) {
        Ok(bytes) => Ok(Some(Zeroizing::new(bytes))),
        Err(err) if err.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(None),
        Err(err) => Err(anyhow::Error::new(err))
            .context("could not read the Claude token from the Keychain"),
    }
}

/// Check whether RunHaven's item exists WITHOUT reading its secret. Queries only
/// the item's attributes (`load_attributes`, i.e. `kSecReturnAttributes`) and
/// NEVER its secret data (no `kSecReturnData`). Decrypting the
/// secret is what invokes LocalAuthentication, so an attributes-only query
/// reports presence with NO Touch ID / device-password prompt. The same exact
/// `(service, account)` filter as every other query (claim 2). `Ok(false)` means
/// no item; errors carry only the operation.
fn present(backend: KeychainBackend) -> Result<bool> {
    let mut options = ItemSearchOptions::new();
    options
        .class(ItemClass::generic_password())
        .service(SERVICE)
        .account(ACCOUNT)
        // Attributes only. The secret data is never requested (no
        // `kSecReturnData`): loading it is the one thing that triggers the
        // user-presence prompt and returns the token, and a presence check must
        // do neither. At least one return type is required or `search()` would
        // report a spurious empty result, so attributes stand in.
        .load_attributes(true)
        .limit(Limit::Max(1));
    if backend == KeychainBackend::DataProtection {
        options.ignore_legacy_keychains();
    }
    match options.search() {
        Ok(results) => Ok(!results.is_empty()),
        Err(err) if err.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(false),
        Err(err) => {
            Err(anyhow::Error::new(err)).context("could not check the Claude token in the Keychain")
        }
    }
}

/// Delete the item for `(service, account)`. Removes only RunHaven's own item.
fn remove(backend: KeychainBackend) -> Result<KeychainClear> {
    match delete_generic_password_options(base_options(backend)) {
        Ok(()) => Ok(KeychainClear::Cleared),
        Err(err) if err.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(KeychainClear::Missing),
        Err(err) => {
            Err(anyhow::Error::new(err)).context("could not clear the Claude token in the Keychain")
        }
    }
}

/// Store `token` as RunHaven's Claude OAuth item, replacing any existing value.
/// Delete-then-add gives a clean replace so the fresh user-presence access
/// control and value fully overwrite a prior login (re-login may prompt to
/// remove the old item, which is expected during an explicit re-sign-in).
pub fn store_token(token: &[u8]) -> Result<()> {
    let backend = select_backend();
    // Ignore the clear outcome: a missing prior item is the common first-login
    // case and must not fail the store.
    let _ = remove(backend);
    put(backend, Some(user_presence_access_control()?), token)
}

/// Read RunHaven's stored Claude OAuth token. Triggers the user-presence
/// prompt. `Ok(None)` means nothing is stored. Used by the broker in a later
/// slice; exposed now so the custody surface is complete and testable.
pub fn read_token() -> Result<Option<Zeroizing<Vec<u8>>>> {
    get(select_backend())
}

/// Report whether RunHaven has a stored Claude OAuth token, WITHOUT reading it.
/// This never triggers the user-presence prompt and never touches the token, so
/// the desktop app's sanitized status can be derived from presence alone. Status
/// polling MUST use this, never [`read_token`], which decrypts the secret and
/// prompts.
pub fn token_present() -> Result<bool> {
    present(select_backend())
}

/// Delete RunHaven's stored Claude OAuth token. Deletes only RunHaven's item.
pub fn clear_token() -> Result<KeychainClear> {
    remove(select_backend())
}

/// Run the host `claude setup-token`, capture its token transiently, and store
/// it to the Keychain, keeping no copy anywhere else. Fails closed: on a
/// missing binary, a nonzero exit or user cancel, or a malformed token, nothing
/// is stored and the error names only the operation and the agent.
///
/// F4 (live probe, maintainer step): confirm on macOS 26+ with the pinned
/// Claude Code that `setup-token` prints the token to a pipe (non-TTY stdout)
/// as it does to a TTY. This capture reuses the previously shipped pipe path
/// (deleted `login_claude` at `60e37db^`); if the current CLI suppresses the
/// token off a terminal, capture must move under a host PTY
/// (`runtime/terminal.rs` `open_gui_attach_pty`).
pub fn capture_and_store_setup_token() -> Result<()> {
    let claude = find_on_path("claude").ok_or_else(|| {
        anyhow!(
            "Claude Code is not installed, so `claude setup-token` cannot run; nothing was saved"
        )
    })?;
    // Capture stdout (the token); inherit stdin/stderr so the browser OAuth
    // flow and any prompts reach the user normally.
    let output = Command::new(&claude)
        .arg("setup-token")
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .output()
        .map_err(|_| anyhow!("could not run `claude setup-token`; nothing was saved"))?;
    if !output.status.success() {
        bail!("Claude sign-in did not finish; nothing was saved");
    }
    // Hold the raw output and the extracted token only for the store window,
    // overwritten on drop. Residual outside our reach (documented, not claimed
    // wiped): the kernel pipe buffer, any reallocated capacity, the child
    // process memory, and the un-zeroized `CFData` copy security-framework makes
    // inside the Keychain store call.
    let stdout = Zeroizing::new(output.stdout);
    let token = parse_token(&stdout)?;
    store_token(token.as_bytes())
}

/// Extract the OAuth token from `claude setup-token` output: the last non-empty
/// line shaped like an Anthropic OAuth token (reuses the deleted `extract_token`
/// logic and its heuristic). The token line is copied into a `Zeroizing` buffer.
///
/// On ANY failure the error names only the operation and the agent, never the
/// captured bytes, because the buffer may itself BE the token.
fn parse_token(stdout: &[u8]) -> Result<Zeroizing<String>> {
    let text = std::str::from_utf8(stdout)
        .map_err(|_| anyhow!("could not read a Claude token; nothing was saved"))?;
    text.lines()
        .map(str::trim)
        .rfind(|line| line.starts_with("sk-ant-") && line.len() > 20)
        .map(|line| Zeroizing::new(line.to_string()))
        .ok_or_else(|| anyhow!("could not read a Claude token; nothing was saved"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_and_account_are_the_runhaven_owned_pair() {
        // The exact identity the "reads only our item" claim rests on.
        assert_eq!(SERVICE, "RunHaven Claude OAuth");
        assert_eq!(ACCOUNT, "claude");
    }

    #[test]
    #[allow(deprecated)] // introspects the built query to prove claim 2
    fn generic_password_query_pins_exactly_service_and_account() {
        // A generic-password query built from (service, account) is exactly
        // three attributes: class, service, account. No fourth match key means
        // no `kSecMatchLimitAll` and no broad/account-only match can slip in.
        let options = PasswordOptions::new_generic_password(SERVICE, ACCOUNT);
        assert_eq!(
            options.query.len(),
            3,
            "the query must pin class + service + account only, never a match-all"
        );
    }

    #[test]
    fn access_control_flag_is_user_presence_only() {
        // Bit 0 is `kSecAccessControlUserPresence`. Assert we pass exactly that
        // flag: it is set, and it is not a biometry-only flag that would lock
        // out a Mac without Touch ID (design Section 1's explicit choice).
        let flags = AccessControlOptions::USER_PRESENCE.bits();
        assert_eq!(flags, 1, "user presence is bit 0");
        assert_eq!(
            flags & AccessControlOptions::BIOMETRY_CURRENT_SET.bits(),
            0,
            "must not be biometry-only, which would exclude non-Touch-ID Macs"
        );
    }

    #[test]
    fn user_presence_access_control_constructs() {
        // The real security-framework call must succeed with the design's
        // protection mode and flag; a build error here means the API drifted.
        user_presence_access_control().expect("build user-presence access control");
    }

    #[test]
    fn parse_token_takes_the_last_token_shaped_line() {
        let out = b"noise\nsk-ant-oat01-AAAAAAAAAAAAAAAAAAAAAAAA\ntrailing\n";
        let token = parse_token(out).expect("token");
        assert_eq!(&*token, "sk-ant-oat01-AAAAAAAAAAAAAAAAAAAAAAAA");
    }

    #[test]
    fn parse_token_rejects_blank_and_garbage_without_echoing_the_buffer() {
        for buffer in [
            b"".as_slice(),
            b"   \n\n".as_slice(),
            b"totally-unrelated-output".as_slice(),
        ] {
            let err = parse_token(buffer).expect_err("must reject");
            let message = err.to_string();
            assert_eq!(message, "could not read a Claude token; nothing was saved");
            // The failure path must never echo the captured bytes: the buffer
            // could itself BE the token.
            let printable = String::from_utf8_lossy(buffer);
            let printable = printable.trim();
            if !printable.is_empty() {
                assert!(
                    !message.contains(printable),
                    "the error must not echo the captured buffer"
                );
            }
        }
    }

    #[test]
    fn parse_token_rejects_a_short_token_shaped_line() {
        // Guards the length heuristic: a bare prefix is not a token.
        assert!(parse_token(b"sk-ant-\n").is_err());
    }

    /// Live Keychain round-trip. Ignored by default because it touches the real
    /// login keychain. It uses a DISTINCT test service and NO access control, so
    /// it never prompts and never touches the real `RunHaven Claude OAuth` item,
    /// and it cleans up after itself. Run with:
    /// `cargo test -p runhaven-core -- --ignored keychain_round_trip`.
    #[test]
    #[ignore = "touches the real macOS Keychain"]
    fn keychain_round_trip_on_a_test_service() {
        // A throwaway generic-password item, legacy keychain, no ACL -> no
        // prompt. This never uses SERVICE/ACCOUNT, so the production item is
        // untouched.
        let test_service = "RunHaven Claude OAuth (test-roundtrip)";
        let value = b"sk-ant-NOT-A-REAL-TOKEN-roundtrip-000000";

        let mut store = PasswordOptions::new_generic_password(test_service, ACCOUNT);
        // Force a clean start in case a prior aborted run left the item.
        let _ = delete_generic_password_options(PasswordOptions::new_generic_password(
            test_service,
            ACCOUNT,
        ));
        store.set_comment("RunHaven Slice 1 round-trip test item");
        set_generic_password_options(value, store).expect("store test item");

        let read = generic_password(PasswordOptions::new_generic_password(test_service, ACCOUNT))
            .expect("read test item");
        assert_eq!(read, value);

        delete_generic_password_options(PasswordOptions::new_generic_password(
            test_service,
            ACCOUNT,
        ))
        .expect("delete test item");
    }
}

#[cfg(test)]
mod source_tripwire_tests;
