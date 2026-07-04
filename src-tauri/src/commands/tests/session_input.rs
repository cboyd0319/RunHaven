use super::*;

#[test]
fn managed_session_input_contract_rejects_before_write() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = SessionInputRegistry::default();
    write_active_run_payload(
        "managed-session-input",
        json!({
            "timestamp": "2026-06-30T05:00:00Z",
            "run_id": "managed-session-input",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session-input",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let missing = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "missing-session".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:missing-session".to_string(),
        },
        &registry,
    )
    .expect_err("unknown session rejected");
    assert!(missing.contains("managed session not found"));

    let oversized_id = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "s".repeat(129),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: format!("session:{}", "s".repeat(129)),
        },
        &registry,
    )
    .expect_err("oversized session id rejected");
    assert!(oversized_id.contains("session id is too long"));

    let unconfirmed = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-input".to_string(),
            input: "hello".to_string(),
            confirm_send: false,
            client_target: "session:managed-session-input".to_string(),
        },
        &registry,
    )
    .expect_err("confirmation required");
    assert!(unconfirmed.contains("Confirm the send"));

    for (label, input) in [
        ("empty input", ""),
        ("oversized input", &"a".repeat(8 * 1024 + 1)),
        ("nul input", "hello\0there"),
        ("escape input", "hello\x1b[31mthere"),
    ] {
        let error = crate::commands::sessions::write_session_input_for_test(
            WriteSessionInputRequest {
                session_id: "managed-session-input".to_string(),
                input: input.to_string(),
                confirm_send: true,
                client_target: "session:managed-session-input".to_string(),
            },
            &registry,
        )
        .expect_err(label);
        assert!(
            error.contains("input"),
            "{label} returned unexpected error: {error}"
        );
    }

    let stale_target = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-input".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:older-session".to_string(),
        },
        &registry,
    )
    .expect_err("stale target rejected");
    assert!(stale_target.contains("Selected session changed"));
}

#[test]
fn managed_session_input_contract_accepts_only_ready_session() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = SessionInputRegistry::default();
    write_active_run_payload(
        "managed-session-ready-input",
        json!({
            "timestamp": "2026-06-30T06:00:00Z",
            "run_id": "managed-session-ready-input",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/ready",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-ready-input",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "ready",
            "token": "should-not-leak",
            "api_key_broker_env": "OPENAI_API_KEY"
        }),
    )
    .expect("ready active run payload");
    write_active_run_payload(
        "managed-session-auth-input",
        json!({
            "timestamp": "2026-06-30T07:00:00Z",
            "run_id": "managed-session-auth-input",
            "profile": "antigravity",
            "workspace": "/sensitive/workspaces/auth",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-antigravity-auth-input",
            "state_volume": "runhaven-antigravity-hidden-home",
            "session": "auth",
            "token": "should-not-leak"
        }),
    )
    .expect("auth active run payload");

    let auth_blocked = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-auth-input".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-auth-input".to_string(),
        },
        &registry,
    )
    .expect_err("auth attention rejected");
    assert!(auth_blocked.contains("Finish sign-in before sending input"));

    registry
        .register_writer(
            "managed-session-ready-input",
            "managed-session-ready-input",
            |input| {
                assert_eq!(String::from_utf8_lossy(input), "hello\nthere");
                Ok(input.len())
            },
        )
        .expect("register ready session writer");
    let response = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-ready-input".to_string(),
            input: "hello\nthere".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-ready-input".to_string(),
        },
        &registry,
    )
    .expect("ready session input");

    assert!(response.accepted);
    assert_eq!(response.accepted_bytes, "hello\nthere".len());
    assert_eq!(response.session.id, "managed-session-ready-input");
    assert_eq!(response.session.run_id, "managed-session-ready-input");
    assert_eq!(response.status, "accepted");

    let encoded = serde_json::to_string(&response).expect("serialize response");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-hidden-home",
        "should-not-leak",
        "attach_session",
        "resize_session",
        "filesystem_rpc",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session input response leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_input_requires_backend_approval_grant() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = SessionInputRegistry::default();
    write_active_run_payload(
        "managed-session-grant-input",
        json!({
            "timestamp": "2026-06-30T06:30:00Z",
            "run_id": "managed-session-grant-input",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/grant",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-grant-input",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "grant",
            "api_key_broker_env": "OPENAI_API_KEY"
        }),
    )
    .expect("grant active run payload");
    let writes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_writes = std::sync::Arc::clone(&writes);
    registry
        .register_writer(
            "managed-session-grant-input",
            "managed-session-grant-input",
            move |input| {
                captured_writes
                    .lock()
                    .expect("writes lock")
                    .push(String::from_utf8_lossy(input).to_string());
                Ok(input.len())
            },
        )
        .expect("register grant writer");

    let request = WriteSessionInputRequest {
        session_id: "managed-session-grant-input".to_string(),
        input: "first".to_string(),
        confirm_send: true,
        client_target: "session:managed-session-grant-input".to_string(),
    };
    let denied = crate::commands::sessions::write_session_input_with_approval_for_test(
        request.clone(),
        &registry,
        |session, run_id| {
            assert_eq!(session.id, "managed-session-grant-input");
            assert_eq!(run_id, "managed-session-grant-input");
            Ok(false)
        },
    )
    .expect_err("approval denial rejected");
    assert!(denied.contains("not approved"));
    assert!(writes.lock().expect("writes lock").is_empty());

    let accepted = crate::commands::sessions::write_session_input_with_approval_for_test(
        request,
        &registry,
        |session, run_id| {
            assert_eq!(session.id, "managed-session-grant-input");
            assert_eq!(run_id, "managed-session-grant-input");
            Ok(true)
        },
    )
    .expect("approved input accepted");
    assert!(accepted.accepted);

    let second = crate::commands::sessions::write_session_input_with_approval_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-grant-input".to_string(),
            input: "second".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-grant-input".to_string(),
        },
        &registry,
        |_, _| panic!("existing backend grant should be reused"),
    )
    .expect("existing grant accepted");
    assert!(second.accepted);
    assert_eq!(
        *writes.lock().expect("writes lock"),
        vec!["first".to_string(), "second".to_string()]
    );
}

#[test]
fn managed_session_input_requires_backend_owned_transport() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-transport-a",
        json!({
            "timestamp": "2026-06-30T08:00:00Z",
            "run_id": "managed-session-transport-a",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/a",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-transport-a",
            "state_volume": "runhaven-codex-hidden-home-a",
            "session": "a",
            "api_key_broker_env": "OPENAI_API_KEY"
        }),
    )
    .expect("active run payload a");
    write_active_run_payload(
        "managed-session-transport-b",
        json!({
            "timestamp": "2026-06-30T08:01:00Z",
            "run_id": "managed-session-transport-b",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/b",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-transport-b",
            "state_volume": "runhaven-codex-hidden-home-b",
            "session": "b",
            "api_key_broker_env": "OPENAI_API_KEY"
        }),
    )
    .expect("active run payload b");

    let registry = SessionInputRegistry::default();
    let missing = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-transport-a".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-transport-a".to_string(),
        },
        &registry,
    )
    .expect_err("unregistered transport rejected");
    assert!(missing.contains("session input transport not found"));

    registry
        .register_writer("managed-session-transport-a", "older-run", |_| Ok(1))
        .expect("register mismatched transport");
    let mismatch = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-transport-a".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-transport-a".to_string(),
        },
        &registry,
    )
    .expect_err("mismatched run rejected");
    assert!(mismatch.contains("session input transport no longer matches"));

    let session_a_writes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let session_b_writes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let session_a_capture = std::sync::Arc::clone(&session_a_writes);
    registry
        .register_writer(
            "managed-session-transport-a",
            "managed-session-transport-a",
            move |input| {
                session_a_capture
                    .lock()
                    .expect("session a writes lock")
                    .push(String::from_utf8_lossy(input).to_string());
                Ok(input.len())
            },
        )
        .expect("register session a transport");
    let session_b_capture = std::sync::Arc::clone(&session_b_writes);
    registry
        .register_writer(
            "managed-session-transport-b",
            "managed-session-transport-b",
            move |input| {
                session_b_capture
                    .lock()
                    .expect("session b writes lock")
                    .push(String::from_utf8_lossy(input).to_string());
                Ok(input.len())
            },
        )
        .expect("register session b transport");

    let response = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-transport-a".to_string(),
            input: "message for a".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-transport-a".to_string(),
        },
        &registry,
    )
    .expect("session a input accepted");

    assert!(response.accepted);
    assert_eq!(response.accepted_bytes, "message for a".len());
    assert_eq!(
        *session_a_writes.lock().expect("session a writes lock"),
        vec!["message for a".to_string()]
    );
    assert!(
        session_b_writes
            .lock()
            .expect("session b writes lock")
            .is_empty(),
        "session b must not receive session a input"
    );
}
