use super::sessions::run_record;
use super::*;

#[test]
fn managed_session_auth_uses_run_owned_broker_proof_before_static_profile_status() {
    let mut brokered = run_record("2026-07-01T01:00:00Z", "run-brokered", "codex", "brokered");
    brokered["api_key_broker_env"] = serde_json::json!("OPENAI_API_KEY");
    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [
            run_record("2026-07-01T02:00:00Z", "run-login-state", "codex", "login"),
            brokered,
        ],
        serde_json::json!({
            "profiles": [{"name": "codex", "status": "api-key-broker"}],
            "token": "should-not-leak"
        }),
    );

    let login_state = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-login-state")
        .expect("login-state session");
    assert_eq!(login_state.auth_status, "login-state");
    assert_eq!(login_state.next_action.kind, "monitor");
    let brokered = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-brokered")
        .expect("brokered session");
    assert_eq!(brokered.auth_status, "api-key-broker");
    assert_eq!(brokered.next_action.kind, "monitor");

    let encoded = serde_json::to_string(&data).expect("serialize managed sessions");
    for forbidden in [
        "OPENAI_API_KEY",
        "/sensitive/workspaces",
        "runhaven-codex-login-home",
        "runhaven-codex-brokered-home",
        "should-not-leak",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session payload leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_auth_marks_shell_no_auth_needed() {
    let mut record = run_record("2026-07-01T03:00:00Z", "run-shell", "shell", "shell");
    record["network"] = serde_json::json!("internet");
    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [record],
        serde_json::json!({"profiles": [{"name": "shell", "status": "n/a"}]}),
    );
    let session = data.sessions.first().expect("shell session");
    assert_eq!(session.auth_status, "no-auth-needed");
    assert_eq!(session.next_action.kind, "monitor");
    assert_eq!(session.events[1].detail, "No sign-in needed");
}

#[test]
fn ordinary_isolated_login_runs_never_request_another_sign_in() {
    for agent in ["antigravity", "claude", "codex", "copilot"] {
        for scope in ["agent", "project"] {
            let mut record = run_record(
                "2026-07-12T12:00:00Z",
                &format!("run-{agent}-{scope}"),
                agent,
                &format!("{agent}-{scope}"),
            );
            record["auth_scope"] = serde_json::json!(scope);
            let data = ManagedSessionListData::from_active_runs_and_auth_status(
                [record],
                serde_json::json!({"profiles": [{"name": agent, "status": "unknown"}]}),
            );
            let session = data.sessions.first().expect("managed session");
            assert_eq!(session.auth_status, "login-state", "{agent}/{scope}");
            assert_eq!(session.next_action.kind, "monitor", "{agent}/{scope}");
            assert_eq!(
                session.events[1].detail, "Session sign-in",
                "{agent}/{scope}"
            );
        }
    }
}

#[test]
fn managed_session_finished_run_proves_broker_from_history_auth_broker_channel() {
    let finished_record = |run_id: &str, session: &str| {
        serde_json::json!({
            "timestamp": "2026-07-01T04:00:00Z", "started_at": "2026-07-01T02:00:00Z",
            "finished_at": "2026-07-01T04:00:00Z", "run_id": run_id, "profile": "codex",
            "workspace": format!("/sensitive/workspaces/{session}"), "network": "provider",
            "status": "succeeded", "auth_scope": "project",
            "state_volume": format!("runhaven-codex-{session}-home"), "session": session
        })
    };
    let mut brokered = finished_record("run-history-brokered", "brokered");
    brokered["auth_broker"] = serde_json::json!({"broker": "api-key", "entries": 0});
    let mut keychain = finished_record("run-history-keychain", "keychain");
    keychain["profile"] = serde_json::json!("claude");
    keychain["auth_broker"] = serde_json::json!({"broker": "keychain-oauth", "entries": 0});
    let mut sign_in = finished_record("run-history-sign-in", "signin");
    sign_in["auth_broker"] = serde_json::json!({"broker": null, "entries": 0});
    let data = ManagedSessionListData::from_active_and_recent_runs(
        [],
        [brokered, keychain, sign_in],
        serde_json::json!({"profiles": [{"name": "codex", "status": "api-key-broker"}]}),
    );

    let brokered = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-history-brokered")
        .expect("brokered finished session");
    assert_eq!(brokered.auth_status, "api-key-broker");
    let keychain = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-history-keychain")
        .expect("keychain finished session");
    assert_eq!(keychain.auth_status, "keychain-oauth");
    assert_eq!(keychain.events[1].detail, "Legacy Claude Keychain sign-in");
    assert_eq!(keychain.next_action.kind, "none");
    let sign_in = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-history-sign-in")
        .expect("sign-in finished session");
    assert_eq!(sign_in.auth_status, "login-state");
}

#[test]
fn managed_session_running_keychain_oauth_run_is_authed_not_needing_sign_in() {
    let running = serde_json::json!({
        "timestamp": "2026-07-04T18:00:00Z", "started_at": "2026-07-04T17:57:00Z",
        "run_id": "run-active-keychain", "profile": "claude",
        "workspace": "/sensitive/workspaces/keychain-live", "network": "provider",
        "status": "running", "auth_scope": "agent",
        "state_volume": "runhaven-claude-keychain-live-home", "session": "keychain-live",
        "claude_auth_source": "keychain-oauth"
    });
    let data = ManagedSessionListData::from_active_and_recent_runs(
        [running],
        [],
        serde_json::json!({"profiles": [{"name": "claude", "status": "unknown"}]}),
    );
    let session = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-active-keychain")
        .expect("running keychain session");
    assert_eq!(session.auth_status, "keychain-oauth");
    assert_ne!(session.next_action.kind, "auth_attention");
    assert_eq!(session.next_action.kind, "monitor");
}
