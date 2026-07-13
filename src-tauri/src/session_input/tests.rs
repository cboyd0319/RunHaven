use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;

#[test]
fn grant_budget_boundary_covers_bytes_and_ttl() {
    assert!(grant_within_budget(0, Duration::ZERO));
    assert!(grant_within_budget(
        SESSION_INPUT_GRANT_MAX_BYTES - 1,
        Duration::from_secs(1)
    ));
    // Reaching the byte budget spends the grant.
    assert!(!grant_within_budget(
        SESSION_INPUT_GRANT_MAX_BYTES,
        Duration::from_secs(1)
    ));
    // Reaching the TTL spends the grant even with bytes to spare.
    assert!(!grant_within_budget(0, SESSION_INPUT_GRANT_TTL));
}

#[test]
fn input_grant_requires_reconfirm_past_byte_budget() {
    let registry = SessionInputRegistry::default();
    registry
        .register_writer("budget-session", "budget-session", |bytes| Ok(bytes.len()))
        .expect("register writer");
    registry
        .approve_input("budget-session", "budget-session")
        .expect("approve input");
    assert!(
        registry
            .input_approved("budget-session", "budget-session")
            .expect("approved fresh grant")
    );

    let chunk = "a".repeat(MAX_SESSION_INPUT_BYTES);
    let mut sent = 0;
    while sent < SESSION_INPUT_GRANT_MAX_BYTES {
        registry
            .write("budget-session", "budget-session", &chunk)
            .expect("send within grant");
        sent += chunk.len();
        if sent < SESSION_INPUT_GRANT_MAX_BYTES {
            assert!(
                registry
                    .input_approved("budget-session", "budget-session")
                    .expect("still approved under budget"),
                "grant should remain valid until the byte budget is reached"
            );
        }
    }

    // Cumulative bytes now meet the budget, so a fresh confirmation is required.
    assert!(
        !registry
            .input_approved("budget-session", "budget-session")
            .expect("grant checked"),
        "grant must require reconfirm once the byte budget is spent"
    );

    // A fresh approval restores the grant and resets the meter.
    registry
        .approve_input("budget-session", "budget-session")
        .expect("re-approve input");
    assert!(
        registry
            .input_approved("budget-session", "budget-session")
            .expect("approved after reconfirm")
    );
}

#[test]
fn concurrent_writes_cannot_overspend_grant_and_failed_write_rolls_back() {
    let registry = SessionInputRegistry::default();
    let chunk = "a".repeat(MAX_SESSION_INPUT_BYTES);
    registry
        .register_writer("concurrent-session", "concurrent-session", |bytes| {
            Ok(bytes.len())
        })
        .expect("register writer");
    registry
        .approve_input("concurrent-session", "concurrent-session")
        .expect("approve input");

    for _ in 0..(SESSION_INPUT_GRANT_MAX_BYTES / MAX_SESSION_INPUT_BYTES - 1) {
        registry
            .write("concurrent-session", "concurrent-session", &chunk)
            .expect("prefill grant");
    }

    let writer_calls = Arc::new(AtomicUsize::new(0));
    let captured_calls = Arc::clone(&writer_calls);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let captured_release = Arc::clone(&release_rx);
    registry
        .register_writer("concurrent-session", "concurrent-session", move |bytes| {
            if captured_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                entered_tx.send(()).expect("signal blocked writer");
                captured_release
                    .lock()
                    .expect("release receiver lock")
                    .recv()
                    .expect("release blocked writer");
            }
            Ok(bytes.len())
        })
        .expect("register controlled writer");

    let first_registry = registry.clone();
    let first_chunk = chunk.clone();
    let first = thread::spawn(move || {
        first_registry.write("concurrent-session", "concurrent-session", &first_chunk)
    });
    entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("first writer entered");

    let second_registry = registry.clone();
    let second_chunk = chunk.clone();
    let (second_result_tx, second_result_rx) = mpsc::channel();
    let second = thread::spawn(move || {
        let result =
            second_registry.write("concurrent-session", "concurrent-session", &second_chunk);
        second_result_tx.send(result).expect("send second result");
    });
    let second_result = match second_result_rx.recv_timeout(Duration::from_secs(2)) {
        Ok(result) => result,
        Err(error) => {
            let _ = release_tx.send(());
            let _ = first.join().expect("first write thread");
            second.join().expect("second write thread");
            panic!("second write did not finish while the first writer was blocked: {error}");
        }
    };
    second.join().expect("second write thread");
    release_tx.send(()).expect("release first writer");
    let first_result = first.join().expect("first write thread");

    assert!(first_result.is_ok(), "first reservation should be written");
    assert!(
        second_result.is_err(),
        "a concurrent write must not overspend the remaining grant"
    );
    assert_eq!(
        writer_calls.load(Ordering::SeqCst),
        1,
        "an over-budget write must be rejected before transport I/O"
    );

    registry
        .approve_input("concurrent-session", "concurrent-session")
        .expect("approve fresh grant");
    registry
        .register_writer("concurrent-session", "concurrent-session", |_| {
            Err("writer failed".to_string())
        })
        .expect("register failing writer");
    registry
        .write("concurrent-session", "concurrent-session", &chunk)
        .expect_err("writer failure must propagate");

    registry
        .register_writer("concurrent-session", "concurrent-session", |bytes| {
            Ok(bytes.len())
        })
        .expect("register recovered writer");
    for _ in 0..(SESSION_INPUT_GRANT_MAX_BYTES / MAX_SESSION_INPUT_BYTES) {
        registry
            .write("concurrent-session", "concurrent-session", &chunk)
            .expect("failed write must not consume the grant");
    }
}

#[test]
fn input_grant_requires_reconfirm_past_ttl() {
    let registry = SessionInputRegistry::default();
    registry
        .approve_input("ttl-session", "ttl-session")
        .expect("approve input");
    assert!(
        registry
            .input_approved("ttl-session", "ttl-session")
            .expect("approved fresh grant")
    );

    registry.expire_grant_for_test("ttl-session");
    assert!(
        !registry
            .input_approved("ttl-session", "ttl-session")
            .expect("grant checked"),
        "grant must require reconfirm once the wall-clock TTL passes"
    );
}

#[test]
fn grant_is_bound_to_run_and_cleared_on_unregister() {
    let registry = SessionInputRegistry::default();
    registry
        .approve_input("bound-session", "bound-run")
        .expect("approve input");
    assert!(
        registry
            .input_approved("bound-session", "bound-run")
            .expect("approved for exact run")
    );
    // A different run id never inherits the grant.
    assert!(
        !registry
            .input_approved("bound-session", "other-run")
            .expect("grant checked")
    );

    registry.unregister("bound-session").expect("unregister");
    assert!(
        !registry
            .input_approved("bound-session", "bound-run")
            .expect("grant checked after unregister"),
        "unregister must clear the grant"
    );
}
