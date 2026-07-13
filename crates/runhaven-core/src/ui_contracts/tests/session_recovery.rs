use std::collections::BTreeSet;

use super::*;

fn running_record(run_id: &str) -> serde_json::Value {
    serde_json::json!({
        "timestamp": "2026-07-01T05:00:00Z",
        "run_id": run_id,
        "profile": "codex",
        "workspace": format!("/sensitive/workspaces/{run_id}"),
        "network": "provider",
        "status": "running",
        "container_name": format!("runhaven-codex-{run_id}-run"),
        "state_volume": "runhaven-codex-hidden-home",
        "auth_scope": "project",
        "api_key_broker_env": "OPENAI_API_KEY",
        "session": run_id
    })
}

#[test]
fn mark_stale_sessions_flags_only_running_sessions_in_the_stale_set() {
    let mut data = ManagedSessionListData::from_active_runs_and_auth_status(
        [running_record("run-stale"), running_record("run-live")],
        serde_json::json!({"profiles": [{"name": "codex", "status": "api-key-broker"}]}),
    );
    for session in &data.sessions {
        assert_eq!(session.next_action.kind, "monitor");
    }

    let mut stale = BTreeSet::new();
    stale.insert("run-stale".to_string());
    data.mark_stale_sessions(&stale);

    let stale_session = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-stale")
        .expect("stale session");
    assert_eq!(stale_session.next_action.kind, "recover");
    assert_eq!(stale_session.status_label, "Session ended unexpectedly");

    let live = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-live")
        .expect("live session");
    assert_eq!(live.next_action.kind, "monitor");
    assert_eq!(live.status_label, "Running");
}

#[test]
fn mark_stale_sessions_never_touches_finished_sessions() {
    let finished = serde_json::json!({
        "timestamp": "2026-07-01T05:00:00Z",
        "started_at": "2026-07-01T04:00:00Z",
        "finished_at": "2026-07-01T05:00:00Z",
        "run_id": "run-finished",
        "profile": "codex",
        "workspace": "/sensitive/workspaces/finished",
        "network": "provider",
        "status": "succeeded",
        "auth_scope": "project",
        "state_volume": "runhaven-codex-hidden-home",
        "session": "finished"
    });
    let mut data = ManagedSessionListData::from_active_and_recent_runs(
        [],
        [finished],
        serde_json::json!({"profiles": [{"name": "codex", "status": "api-key-broker"}]}),
    );

    // Even if a finished run id is (wrongly) reported stale, it must not become a
    // recover affordance: a finished session has nothing to repair.
    let mut stale = BTreeSet::new();
    stale.insert("run-finished".to_string());
    data.mark_stale_sessions(&stale);

    let session = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-finished")
        .expect("finished session");
    assert_eq!(session.next_action.kind, "none");
}
