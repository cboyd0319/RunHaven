//! Where an agent's host-side OAuth token lives. The default is a private
//! `0600` file in the RunHaven cache, byte-for-byte RunHaven's original
//! behavior. macOS users can opt into the login Keychain instead by setting
//! `RUNHAVEN_TOKEN_STORE=keychain`. The selection is per process and applies to
//! all three token operations (store on login, read on run, clear on
//! `login <agent> --clear`), so opting in routes every path to the same store.
//!
//! Only agents whose credential RunHaven holds host-side (currently `claude`)
//! use a `TokenStore`. The in-sandbox device-login agents keep their token in an
//! isolated home volume and never reach here.
//!
//! Implementations must never log or print the token, and their error messages
//! must not carry the token value.

use std::io::{Read, Write};

use anyhow::{Context, Result};

use crate::support::paths::{create_private_file, oauth_token_path};

/// Result of clearing a stored token, so callers print the same message
/// whichever backend is active.
pub enum TokenClear {
    /// A stored token existed and was removed.
    Cleared,
    /// There was no stored token to remove.
    Missing,
}

/// Store, read, and clear one agent's host-side OAuth token.
pub trait TokenStore {
    /// Persist `token` for `agent`, replacing any existing value.
    fn store(&self, agent: &str, token: &str) -> Result<()>;
    /// The stored token for `agent`, or `None` when there is none (or it is
    /// empty or unreadable).
    fn read(&self, agent: &str) -> Option<String>;
    /// Remove any stored token for `agent`.
    fn clear(&self, agent: &str) -> Result<TokenClear>;
}

/// Which backend the environment selects. `keychain` selects the macOS
/// Keychain; every other value, and no value, selects the file store, so
/// existing users are unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenStoreKind {
    /// Private `0600` file under the RunHaven cache. The default.
    File,
    /// macOS login Keychain generic-password item. Opt-in.
    Keychain,
}

/// The default backend: a private `0600` file under the RunHaven cache. This is
/// byte-for-byte RunHaven's original token behavior.
pub struct FileTokenStore;

impl TokenStore for FileTokenStore {
    fn store(&self, agent: &str, token: &str) -> Result<()> {
        let path = oauth_token_path(agent);
        // `create_private_file` ensures the private parent and opens `0600`.
        let mut file = create_private_file(&path)
            .with_context(|| format!("could not write login store: {}", path.display()))?;
        file.write_all(token.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn read(&self, agent: &str) -> Option<String> {
        let mut file = std::fs::File::open(oauth_token_path(agent)).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        let token = contents.trim();
        (!token.is_empty()).then(|| token.to_string())
    }

    fn clear(&self, agent: &str) -> Result<TokenClear> {
        let path = oauth_token_path(agent);
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("could not remove stored login: {}", path.display()))?;
            Ok(TokenClear::Cleared)
        } else {
            Ok(TokenClear::Missing)
        }
    }
}

/// The backend a `RUNHAVEN_TOKEN_STORE` value selects. Pure and env-free so the
/// selection logic is unit-testable without mutating process state.
fn selected_kind(setting: Option<&str>) -> TokenStoreKind {
    match setting {
        Some("keychain") => TokenStoreKind::Keychain,
        _ => TokenStoreKind::File,
    }
}

/// The concrete store for a kind. The `Keychain` kind resolves to the macOS
/// backend only on macOS; anywhere else it falls back to the file store so the
/// mapping stays total (RunHaven targets macOS, so this fallback is unreachable
/// through [`resolve_token_store`] on a supported host).
fn store_for(kind: TokenStoreKind) -> Box<dyn TokenStore> {
    match kind {
        #[cfg(target_os = "macos")]
        TokenStoreKind::Keychain => Box::new(keychain::KeychainTokenStore),
        _ => Box::new(FileTokenStore),
    }
}

/// Resolve the token store for this process. Defaults to the file store;
/// `RUNHAVEN_TOKEN_STORE=keychain` opts into the macOS login Keychain. Any other
/// value (or unset) keeps the file store, so existing users see no change.
pub fn resolve_token_store() -> Box<dyn TokenStore> {
    store_for(selected_kind(
        std::env::var("RUNHAVEN_TOKEN_STORE").ok().as_deref(),
    ))
}

#[cfg(target_os = "macos")]
mod keychain {
    use anyhow::{Context, Result};
    use security_framework::passwords::{
        delete_generic_password, get_generic_password, set_generic_password,
    };

    use super::{TokenClear, TokenStore};

    /// Fixed Keychain service name for RunHaven OAuth tokens. Generic-password
    /// items are keyed by (service, account); the account is the agent name.
    const SERVICE: &str = "RunHaven OAuth token";

    /// Apple `errSecItemNotFound` OSStatus: no matching Keychain item. Stable
    /// Security-framework status code, kept local to avoid a second direct
    /// dependency on `security-framework-sys` just for the constant.
    const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

    /// macOS login Keychain backend, selected by `RUNHAVEN_TOKEN_STORE=keychain`.
    /// `security-framework` wraps the C Security API safely, so nothing here
    /// needs `unsafe`.
    pub struct KeychainTokenStore;

    impl TokenStore for KeychainTokenStore {
        fn store(&self, agent: &str, token: &str) -> Result<()> {
            set_generic_password(SERVICE, agent, token.as_bytes())
                .map_err(anyhow::Error::new)
                .with_context(|| format!("could not store the {agent} login in the Keychain"))
        }

        fn read(&self, agent: &str) -> Option<String> {
            let bytes = get_generic_password(SERVICE, agent).ok()?;
            let token = String::from_utf8(bytes).ok()?;
            let token = token.trim();
            (!token.is_empty()).then(|| token.to_string())
        }

        fn clear(&self, agent: &str) -> Result<TokenClear> {
            match delete_generic_password(SERVICE, agent) {
                Ok(()) => Ok(TokenClear::Cleared),
                Err(err) if err.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(TokenClear::Missing),
                Err(err) => Err(anyhow::Error::new(err))
                    .with_context(|| format!("could not clear the {agent} login in the Keychain")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run `body` with a temporary cache root so file-store I/O never touches
    /// the real user cache.
    fn with_temp_cache(body: impl FnOnce()) {
        let cache = tempfile::tempdir().expect("cache");
        let _guard = crate::support::paths::override_cache_root_for_tests(cache.path());
        body();
    }

    #[test]
    fn file_store_round_trips_the_token_at_the_private_oauth_path() {
        with_temp_cache(|| {
            let store = FileTokenStore;
            let placeholder = "sk-ant-NOT-A-REAL-TOKEN-file-roundtrip";
            store.store("claude", placeholder).expect("store");
            // The value comes back trimmed and identical.
            assert_eq!(store.read("claude").as_deref(), Some(placeholder));
            // The bytes on disk are the token plus a single trailing newline,
            // written at the same oauth_token_path as before.
            let path = crate::support::paths::oauth_token_path("claude");
            let bytes = std::fs::read(&path).expect("read file");
            assert_eq!(bytes, format!("{placeholder}\n").into_bytes());
        });
    }

    #[cfg(unix)]
    #[test]
    fn file_store_writes_0600() {
        use std::os::unix::fs::PermissionsExt;
        with_temp_cache(|| {
            FileTokenStore
                .store("claude", "sk-ant-NOT-A-REAL-TOKEN-perms")
                .expect("store");
            let path = crate::support::paths::oauth_token_path("claude");
            let mode = std::fs::metadata(&path)
                .expect("metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        });
    }

    #[test]
    fn file_store_read_is_none_when_absent_or_empty() {
        with_temp_cache(|| {
            let store = FileTokenStore;
            assert!(store.read("claude").is_none());
            // An empty (whitespace-only) file reads as no token, matching the
            // original read behavior.
            store.store("claude", "   ").expect("store");
            assert!(store.read("claude").is_none());
        });
    }

    #[test]
    fn file_store_clear_reports_cleared_then_missing() {
        with_temp_cache(|| {
            let store = FileTokenStore;
            store
                .store("claude", "sk-ant-NOT-A-REAL-TOKEN-clear")
                .expect("store");
            assert!(matches!(
                store.clear("claude").expect("clear"),
                TokenClear::Cleared
            ));
            let path = crate::support::paths::oauth_token_path("claude");
            assert!(!path.exists());
            // Clearing again when nothing is stored is not an error.
            assert!(matches!(
                store.clear("claude").expect("clear"),
                TokenClear::Missing
            ));
        });
    }

    #[test]
    fn selected_kind_defaults_to_file_and_opts_in_on_exact_keychain() {
        assert_eq!(selected_kind(Some("keychain")), TokenStoreKind::Keychain);
        assert_eq!(selected_kind(None), TokenStoreKind::File);
        assert_eq!(selected_kind(Some("")), TokenStoreKind::File);
        assert_eq!(selected_kind(Some("file")), TokenStoreKind::File);
        // Exact match only: any other spelling stays on the file default.
        assert_eq!(selected_kind(Some("Keychain")), TokenStoreKind::File);
        assert_eq!(selected_kind(Some("keychain ")), TokenStoreKind::File);
    }

    /// On macOS, constructing the Keychain backend and coercing it to
    /// `&dyn TokenStore` proves it compiles and implements the trait under
    /// `forbid(unsafe)`. The live Keychain round-trip is verified by hand on
    /// macOS; this test performs no Keychain I/O.
    #[cfg(target_os = "macos")]
    #[test]
    fn keychain_store_compiles_and_implements_the_trait() {
        let store = keychain::KeychainTokenStore;
        let _dyn_store: &dyn TokenStore = &store;
        // `store_for` routes the Keychain kind to the macOS backend.
        let _resolved = store_for(TokenStoreKind::Keychain);
    }
}
