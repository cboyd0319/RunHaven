//! Explicit user-requested end behavior. Teardown keeps using the separate,
//! noninteractive `close_terminal_session` command.

use super::*;

#[test]
fn user_requested_end_confirms_before_closing_the_terminal() {
    let registry = TerminalSessionRegistry::default();
    let (opened, fake) = open_fake(&registry, "end-after-confirm");
    opened.expect("open terminal");

    let outcome =
        end_terminal_session_with_approval_for_test(&registry, "end-after-confirm", |_| {
            assert_eq!(
                fake.shutdowns(),
                0,
                "native confirmation must occur before the terminal child is killed"
            );
            Ok(true)
        });

    assert_eq!(outcome, Ok(true));
    assert_eq!(fake.shutdowns(), 1);
}

#[test]
fn canceled_user_requested_end_preserves_the_live_terminal() {
    let registry = TerminalSessionRegistry::default();
    let (opened, fake) = open_fake(&registry, "end-canceled");
    opened.expect("open terminal");

    let outcome =
        end_terminal_session_with_approval_for_test(&registry, "end-canceled", |_| Ok(false));

    assert_eq!(outcome, Ok(false), "cancel is an explicit outcome");
    assert_eq!(fake.shutdowns(), 0, "cancel must not kill the child");
    registry
        .write_input("end-canceled", b"still live\n")
        .expect("cancel keeps the registry entry writable");
}

#[test]
fn confirmed_user_requested_end_revalidates_then_closes() {
    let registry = TerminalSessionRegistry::default();
    let (opened, fake) = open_fake(&registry, "end-confirmed");
    opened.expect("open terminal");
    let mut approvals = 0;

    let outcome = end_terminal_session_with_approval_for_test(&registry, "end-confirmed", |_| {
        approvals += 1;
        Ok(true)
    });

    assert_eq!(outcome, Ok(true));
    assert_eq!(approvals, 1);
    assert_eq!(fake.shutdowns(), 1);
    assert!(registry.remove("end-confirmed").is_none());
}

#[test]
fn target_replaced_during_confirmation_fails_closed_without_closing_replacement() {
    let registry = TerminalSessionRegistry::default();
    let (opened, original) = open_fake(&registry, "end-replaced");
    opened.expect("open original terminal");
    let mut replacement = None;

    let outcome = end_terminal_session_with_approval_for_test(&registry, "end-replaced", |_| {
        registry.close("end-replaced")?;
        let (opened, fake) = open_fake(&registry, "end-replaced");
        opened?;
        replacement = Some(fake);
        Ok(true)
    });

    assert!(
        outcome.is_err(),
        "a different terminal under the same id is not the approved target"
    );
    assert_eq!(original.shutdowns(), 1);
    let replacement = replacement.expect("replacement terminal");
    assert_eq!(
        replacement.shutdowns(),
        0,
        "stale approval must not close the replacement"
    );
    registry
        .write_input("end-replaced", b"replacement remains live\n")
        .expect("replacement stays registered");
}

#[test]
fn unknown_user_requested_end_fails_before_confirmation_without_mutation() {
    let registry = TerminalSessionRegistry::default();
    let mut confirmation_requested = false;

    let outcome = end_terminal_session_with_approval_for_test(&registry, "end-missing", |_| {
        confirmation_requested = true;
        Ok(true)
    });

    assert!(outcome.is_err(), "unknown terminal must fail closed");
    assert!(
        !confirmation_requested,
        "an unknown target must fail before native confirmation"
    );
}

#[test]
fn teardown_close_terminal_session_remains_noninteractive() {
    let source = include_str!("../mod.rs");
    let body = source
        .split_once("pub(crate) fn close_terminal_session(")
        .expect("close terminal command")
        .1
        .split_once("\n}\n")
        .expect("close terminal command body")
        .0;

    assert!(body.contains("registry.close(&session_id)"));
    assert!(!body.contains("confirm"));
    assert!(!body.contains("await"));
}
