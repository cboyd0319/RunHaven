//! Hermetic unit tests for the Claude OAuth Keychain custody surface.
//! Split from the module body to keep the source file under the structure
//! ceiling. The live Keychain round-trip is `#[ignore]`d.

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
fn access_for_backend_gates_only_the_data_protection_keychain() {
    // F3: an access control is a data-protection-keychain feature; the legacy
    // keychain (unsigned dev) rejects it, so an item there carries NO gate and
    // stores successfully, while the data-protection keychain (signed release)
    // carries the user-presence gate.
    assert!(
        access_for_backend(KeychainBackend::LegacyFile)
            .expect("legacy access")
            .is_none(),
        "the legacy keychain must store without an access control"
    );
    assert!(
        access_for_backend(KeychainBackend::DataProtection)
            .expect("data-protection access")
            .is_some(),
        "the data-protection keychain must carry the user-presence gate"
    );
}

// Token extraction (`extract_token`) and the exact-resolved-path capture moved
// to `claude_oauth_capture` and are tested there (`claude_oauth_capture::tests`);
// the Tauri command's show-and-run-the-same-path check covers F8 at that layer.

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

    delete_generic_password_options(PasswordOptions::new_generic_password(test_service, ACCOUNT))
        .expect("delete test item");
}
