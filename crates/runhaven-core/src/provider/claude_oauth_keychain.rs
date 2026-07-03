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
//! - Presence and the token's AGE (`token_presence`) come from an
//!   attributes-only query that requests `kSecReturnAttributes` and NEVER
//!   `kSecReturnData`, so it reports whether the item exists and how old it is
//!   (from the item's own non-secret `kSecAttrCreationDate`) without decrypting
//!   the secret and without a user-presence prompt. The age is a day count, not
//!   a token-derived value, and it drives only the re-sign-in cadence prompt.
//! - The host `claude setup-token` capture that fills this item lives in the
//!   sibling [`crate::provider::claude_oauth_capture`] module (re-exported below
//!   so callers keep one import surface). It runs setup-token under a host PTY,
//!   holds the transient output and the extracted token in `Zeroizing` buffers,
//!   and stores only through [`store_token`]. Both modules carry their own
//!   no-persist source tripwire; the capture's PTY reader is covered by the
//!   capture module's tripwire, this module's by its own.
//!
//! `security-framework`'s safe wrappers own all FFI, so nothing here needs
//! `unsafe`; this composes with the workspace `unsafe_code = forbid` lint.

use anyhow::{Context, Result, anyhow};
use security_framework::access_control::{ProtectionMode, SecAccessControl};
use security_framework::item::{ItemClass, ItemSearchOptions, Limit};
use security_framework::passwords::{
    AccessControlOptions, PasswordOptions, delete_generic_password_options, generic_password,
    set_generic_password_options,
};
use zeroize::Zeroizing;

use super::claude_oauth_age::age_days_from_cdat;

// The host `claude setup-token` capture lives in the sibling module; re-exported
// so the sign-in command keeps a single `claude_oauth_keychain::` import surface
// (design C, Section 2). The custody surface below (store/read/clear/presence)
// is the only sink the capture writes to.
pub use super::claude_oauth_capture::{capture_and_store_setup_token, resolve_setup_token_binary};

/// RunHaven-owned service label. Distinct from any label Claude Code uses, so a
/// query for this service can only match RunHaven's own item.
const SERVICE: &str = "RunHaven Claude OAuth";

/// Account within the service. Fixed today; room for future per-agent items.
const ACCOUNT: &str = "claude";

/// The item's `kSecAttrCreationDate` attribute key, returned by an
/// attributes-only query. Non-secret; used only to age the token (Hardening 4).
const CREATION_DATE_ATTR: &str = "cdat";

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

/// Non-secret presence of RunHaven's stored token, plus its age when present.
/// Both fields are derived from the attributes-only query (no secret read, no
/// user-presence prompt): `present` from item existence, `age_days` from the
/// item's own `kSecAttrCreationDate`. Never carries the token or any
/// token-derived value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenPresence {
    /// Whether RunHaven's item exists.
    pub present: bool,
    /// Whole days since the item was (re)created, or `None` when absent or the
    /// creation date could not be parsed. Non-secret; never token-derived.
    pub age_days: Option<u32>,
}

impl TokenPresence {
    /// The no-item outcome: absent, with no age.
    const fn absent() -> Self {
        Self {
            present: false,
            age_days: None,
        }
    }
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

/// Check whether RunHaven's item exists WITHOUT reading its secret, and read the
/// item's own non-secret creation date to age the token. Queries only the item's
/// attributes (`load_attributes`, i.e. `kSecReturnAttributes`) and NEVER its
/// secret data (no `kSecReturnData`). Decrypting the secret is what invokes
/// LocalAuthentication, so this attributes-only query reports presence and age
/// with NO Touch ID / device-password prompt, and the token is never read. The
/// same exact `(service, account)` filter as every other query (claim 2). The
/// age comes from `kSecAttrCreationDate` (a non-secret attribute of the SAME one
/// item, so "reads only the one item it created" still holds); an unparseable
/// date reads as `None`. `TokenPresence::absent()` means no item; errors carry
/// only the operation.
fn presence(backend: KeychainBackend) -> Result<TokenPresence> {
    let mut options = ItemSearchOptions::new();
    options
        .class(ItemClass::generic_password())
        .service(SERVICE)
        .account(ACCOUNT)
        // Attributes only. The secret data is never requested (no
        // `kSecReturnData`): loading it is the one thing that triggers the
        // user-presence prompt and returns the token, and a presence check must
        // do neither. At least one return type is required or `search()` would
        // report a spurious empty result, so attributes stand in and also carry
        // the non-secret creation date used for the age.
        .load_attributes(true)
        .limit(Limit::Max(1));
    if backend == KeychainBackend::DataProtection {
        options.ignore_legacy_keychains();
    }
    match options.search() {
        Ok(results) => match results.first() {
            None => Ok(TokenPresence::absent()),
            Some(result) => Ok(TokenPresence {
                present: true,
                // The creation date is the only attribute read for the age; the
                // secret data is never requested, so this cannot surface a
                // token. An absent or unparseable date reads as "age unknown".
                age_days: result
                    .simplify_dict()
                    .and_then(|attributes| attributes.get(CREATION_DATE_ATTR).cloned())
                    .and_then(|cdat| age_days_from_cdat(&cdat)),
            }),
        },
        Err(err) if err.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(TokenPresence::absent()),
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
/// polling MUST use this or [`token_presence`], never [`read_token`], which
/// decrypts the secret and prompts.
pub fn token_present() -> Result<bool> {
    Ok(presence(select_backend())?.present)
}

/// Report presence PLUS the token's non-secret age, WITHOUT reading the token.
/// Same attributes-only path as [`token_present`] (no prompt, no secret read);
/// the age comes from the item's own `kSecAttrCreationDate` and is used only to
/// drive the re-sign-in cadence prompt (Hardening 4). Never carries a secret.
pub fn token_presence() -> Result<TokenPresence> {
    presence(select_backend())
}

/// Delete RunHaven's stored Claude OAuth token. Deletes only RunHaven's item.
pub fn clear_token() -> Result<KeychainClear> {
    remove(select_backend())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod source_tripwire_tests;
