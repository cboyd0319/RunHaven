use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn write_ready_input_session(run_id: &str) {
    write_active_run_payload(
        run_id,
        json!({
            "timestamp": "2026-07-13T12:00:00Z",
            "run_id": run_id,
            "profile": "codex",
            "workspace": "/sensitive/workspaces/input-security",
            "network": "provider",
            "status": "running",
            "container_name": format!("runhaven-codex-{run_id}"),
            "state_volume": "runhaven-codex-hidden-home",
            "session": "input-security",
            "api_key_broker_env": "OPENAI_API_KEY"
        }),
    )
    .expect("active run payload");
}

#[test]
fn managed_session_input_request_rejects_renderer_approval_fields() {
    let parsed = serde_json::from_value::<WriteSessionInputRequest>(json!({
        "sessionId": "managed-session-input",
        "input": "private-message-marker",
        "confirmSend": true,
        "clientTarget": "session:managed-session-input"
    }));

    assert!(
        parsed.is_err(),
        "legacy renderer approval fields must be rejected"
    );
    let error = parsed.expect_err("legacy fields rejected").to_string();
    assert!(error.contains("unknown field"));
    assert!(!error.contains("private-message-marker"));
}

#[test]
fn managed_session_input_rejects_hidden_or_control_formatting_without_echoing_input() {
    let registry = SessionInputRegistry::default();
    for input in [
        "private\tmessage",
        "private\rmessage",
        "private\u{202e}message",
        "private\u{2066}message",
        "private\u{200b}message",
        "private\u{2028}message",
        "private\u{feff}message",
    ] {
        let error = crate::commands::sessions::write_session_input_for_test(
            WriteSessionInputRequest {
                session_id: "missing-session".to_string(),
                input: input.to_string(),
            },
            &registry,
        )
        .expect_err("hidden formatting rejected");

        assert_eq!(
            error,
            "This message contains hidden formatting that RunHaven can’t send safely. Paste it again as plain text, then try again."
        );
        assert!(!error.contains(input));
    }
}

#[test]
fn managed_session_input_confirmation_bounds_safe_target_and_head_tail_preview() {
    let input = [
        format!("HEAD-{}", "a".repeat(180)),
        "line-two".to_string(),
        "line-three".to_string(),
        "private-middle-marker".to_string(),
        "line-five".to_string(),
        "line-six".to_string(),
        "line-seven".to_string(),
        format!("{}-TAIL", "z".repeat(180)),
    ]
    .join("\n");
    let message =
        crate::commands::managed_session_input::session_input_confirmation_message_for_test(
            "Project\nStatus: forged\u{202e}",
            &input,
        );

    assert!(message.contains("To: Project Status: forged"));
    assert!(message.contains(&format!("Message size: {} bytes", input.len())));
    assert!(message.contains("Message preview (shortened):"));
    assert!(message.lines().any(|line| line == "[middle omitted]"));
    assert!(!message.contains('\u{202e}'));
    assert_eq!(
        message
            .lines()
            .filter(|line| line.starts_with("Status:"))
            .count(),
        0
    );
    let preview = message
        .split_once("Message preview (shortened):\n")
        .expect("preview heading")
        .1
        .split_once("\n\nRunHaven will send the full message")
        .expect("preview boundary")
        .0;
    assert!(preview.chars().count() <= 240);
    assert!(preview.lines().count() <= 6);
    assert!(preview.matches('\n').count() <= 5);
    assert!(preview.contains("HEAD-"));
    assert!(preview.contains("-TAIL"));
    assert!(!preview.contains("private-middle-marker"));
}

#[test]
fn managed_session_input_confirmation_names_an_unshortened_exact_message() {
    let message =
        crate::commands::managed_session_input::session_input_confirmation_message_for_test(
            "Codex in Project",
            "Please update the welcome text.",
        );

    assert!(message.contains("Message:\nPlease update the welcome text."));
    assert!(!message.contains("Message preview (shortened):"));
    assert!(message.contains(
        "RunHaven will send this exact message to the running assistant. The assistant can act on it in this session."
    ));
}

#[test]
fn managed_session_input_rejects_a_writer_replaced_during_native_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "managed-session-replaced-writer";
    write_ready_input_session(run_id);
    let registry = SessionInputRegistry::default();
    let original_calls = Arc::new(AtomicUsize::new(0));
    let original_capture = Arc::clone(&original_calls);
    registry
        .register_writer(run_id, run_id, move |bytes| {
            original_capture.fetch_add(1, Ordering::SeqCst);
            Ok(bytes.len())
        })
        .expect("original writer");
    let replacement_calls = Arc::new(AtomicUsize::new(0));
    let replacement_capture = Arc::clone(&replacement_calls);

    let error = crate::commands::sessions::write_session_input_with_approval_for_test(
        WriteSessionInputRequest {
            session_id: run_id.to_string(),
            input: "private-replaced-writer-message".to_string(),
        },
        &registry,
        |_, _| {
            registry.register_writer(run_id, run_id, move |bytes| {
                replacement_capture.fetch_add(1, Ordering::SeqCst);
                Ok(bytes.len())
            })?;
            Ok(true)
        },
    )
    .expect_err("replaced writer must fail closed");

    assert_eq!(
        error,
        "The session changed before RunHaven could send the message. Nothing was sent. Check the session status, then try again."
    );
    assert!(!error.contains("private-replaced-writer-message"));
    assert_eq!(original_calls.load(Ordering::SeqCst), 0);
    assert_eq!(replacement_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn managed_session_input_rejects_an_inactive_session_after_native_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "managed-session-removed-during-approval";
    write_ready_input_session(run_id);
    let registry = SessionInputRegistry::default();
    let writer_calls = Arc::new(AtomicUsize::new(0));
    let writer_capture = Arc::clone(&writer_calls);
    registry
        .register_writer(run_id, run_id, move |bytes| {
            writer_capture.fetch_add(1, Ordering::SeqCst);
            Ok(bytes.len())
        })
        .expect("writer");

    let error = crate::commands::sessions::write_session_input_with_approval_for_test(
        WriteSessionInputRequest {
            session_id: run_id.to_string(),
            input: "private-message-after-removal".to_string(),
        },
        &registry,
        |_, _| {
            runhaven_core::runtime::active::remove_active_run_record(run_id)
                .map_err(|error| error.to_string())?;
            Ok(true)
        },
    )
    .expect_err("inactive session must fail closed after approval");

    assert_eq!(
        error,
        "This session ended, so it can’t receive messages. Start a new session to continue."
    );
    assert!(!error.contains("private-message-after-removal"));
    assert_eq!(writer_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn managed_session_input_rechecks_monitor_state_after_native_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "managed-session-stopped-during-approval";
    write_ready_input_session(run_id);
    let registry = SessionInputRegistry::default();
    let writer_calls = Arc::new(AtomicUsize::new(0));
    let writer_capture = Arc::clone(&writer_calls);
    registry
        .register_writer(run_id, run_id, move |bytes| {
            writer_capture.fetch_add(1, Ordering::SeqCst);
            Ok(bytes.len())
        })
        .expect("writer");

    let error = crate::commands::sessions::write_session_input_with_approval_for_test(
        WriteSessionInputRequest {
            session_id: run_id.to_string(),
            input: "private-message-after-stop".to_string(),
        },
        &registry,
        |_, _| {
            write_active_run_payload(
                run_id,
                json!({
                    "timestamp": "2026-07-13T12:00:00Z",
                    "run_id": run_id,
                    "profile": "codex",
                    "workspace": "/sensitive/workspaces/input-security",
                    "network": "provider",
                    "status": "stopped",
                    "container_name": format!("runhaven-codex-{run_id}"),
                    "state_volume": "runhaven-codex-hidden-home",
                    "session": "input-security",
                    "api_key_broker_env": "OPENAI_API_KEY"
                }),
            )
            .map_err(|error| error.to_string())?;
            Ok(true)
        },
    )
    .expect_err("non-monitor session must fail closed after approval");

    assert_eq!(
        error,
        "This session can’t receive messages now. Check its status before you try again."
    );
    assert!(!error.contains("private-message-after-stop"));
    assert_eq!(writer_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn managed_session_input_hides_writer_errors_and_message_content() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "managed-session-private-writer-error";
    write_ready_input_session(run_id);
    let registry = SessionInputRegistry::default();
    registry
        .register_writer(run_id, run_id, |_| {
            Err("private-message-marker at /sensitive/workspaces/input-security".to_string())
        })
        .expect("writer");

    let error = crate::commands::sessions::write_session_input_with_approval_for_test(
        WriteSessionInputRequest {
            session_id: run_id.to_string(),
            input: "private-message-marker".to_string(),
        },
        &registry,
        |_, _| Ok(true),
    )
    .expect_err("writer error must be sanitized");

    assert_eq!(
        error,
        "RunHaven couldn’t send this message. Nothing was sent. Try again."
    );
    assert!(!error.contains("private-message-marker"));
    assert!(!error.contains("/sensitive/workspaces"));
}

#[test]
fn managed_session_input_rejects_past_running_record_before_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "managed-session-past-running-input";
    write_history_run_record(json!({
        "timestamp": "2026-07-13T11:00:00Z",
        "run_id": run_id,
        "profile": "codex",
        "workspace": "/sensitive/workspaces/past-input",
        "network": "provider",
        "status": "running",
        "container_name": "runhaven-codex-past-running-input",
        "state_volume": "runhaven-codex-hidden-home",
        "session": "past-input",
        "api_key_broker_env": "OPENAI_API_KEY"
    }));
    let registry = SessionInputRegistry::default();
    let writer_calls = Arc::new(AtomicUsize::new(0));
    let writer_capture = Arc::clone(&writer_calls);
    registry
        .register_writer(run_id, run_id, move |bytes| {
            writer_capture.fetch_add(1, Ordering::SeqCst);
            Ok(bytes.len())
        })
        .expect("writer");
    let mut approval_requested = false;

    let error = crate::commands::sessions::write_session_input_with_approval_for_test(
        WriteSessionInputRequest {
            session_id: run_id.to_string(),
            input: "private-past-message".to_string(),
        },
        &registry,
        |_, _| {
            approval_requested = true;
            Ok(true)
        },
    )
    .expect_err("past record must not accept input");

    assert!(!approval_requested);
    assert_eq!(
        error,
        "This session ended, so it can’t receive messages. Start a new session to continue."
    );
    assert!(!error.contains("private-past-message"));
    assert_eq!(writer_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn managed_session_input_allows_only_one_app_wide_approval_flight() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "managed-session-single-flight-input";
    write_ready_input_session(run_id);
    let registry = SessionInputRegistry::default();
    registry
        .register_writer(run_id, run_id, |bytes| Ok(bytes.len()))
        .expect("writer");
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let first_registry = registry.clone();
    let first = thread::spawn(move || {
        crate::commands::sessions::write_session_input_with_approval_for_test(
            WriteSessionInputRequest {
                session_id: run_id.to_string(),
                input: "first-private-message".to_string(),
            },
            &first_registry,
            |_, _| {
                entered_tx.send(()).expect("approval entered");
                release_rx.recv().expect("release approval");
                Ok(true)
            },
        )
    });
    entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("first approval entered");
    let second_approval_requested = Arc::new(AtomicBool::new(false));
    let second_approval_capture = Arc::clone(&second_approval_requested);

    let second = crate::commands::sessions::write_session_input_with_approval_for_test(
        WriteSessionInputRequest {
            session_id: run_id.to_string(),
            input: "second-private-message".to_string(),
        },
        &registry,
        move |_, _| {
            second_approval_capture.store(true, Ordering::SeqCst);
            Ok(false)
        },
    )
    .expect_err("second approval flight rejected");
    release_tx.send(()).expect("release first approval");
    let first = first.join().expect("first thread").expect("first send");

    assert!(first.accepted);
    assert_eq!(
        second,
        "Another message is waiting for confirmation. Finish or cancel it, then try again."
    );
    assert!(!second_approval_requested.load(Ordering::SeqCst));
    assert!(!second.contains("second-private-message"));
}
