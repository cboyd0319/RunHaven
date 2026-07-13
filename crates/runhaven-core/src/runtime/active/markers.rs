use std::fs;
use std::io::Write;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::runtime::lock::acquire_active_run_lock;
use crate::runtime::plans::{AgentRunPlan, ClaudeAuthSource};
use crate::runtime::worktrees::worktree_record;
use crate::support::paths::{active_run_path, active_runs_dir, create_private_file};
use crate::support::validators::{validate_run_id, validate_runhaven_container_name};

pub fn write_active_run_record(plan: &AgentRunPlan, run_id: &str, started_at: &str) -> Result<()> {
    let mut payload = json!({
        "timestamp": started_at,
        "run_id": run_id,
        "profile": plan.profile_name,
        "workspace": plan.workspace.display().to_string(),
        "workspace_scope": plan.workspace_scope.as_str(),
        "network": plan.network_mode.as_str(),
        "auth_scope": plan.auth_scope.as_str(),
        "status": "running",
        "container_name": plan.container_name,
        "state_volume": plan.state_volume,
        "session": plan.session,
        "network_name": plan.network_name,
        "host_pid": std::process::id(),
    });
    if let Some(worktree) = &plan.worktree {
        payload["worktree"] = worktree_record(worktree);
    }
    // Persist only the explicitly configured environment-variable NAME. The
    // value is never part of the plan or marker.
    match &plan.claude_auth_source {
        Some(ClaudeAuthSource::ApiKeyBrokerEnv(name)) => {
            payload["api_key_broker_env"] = json!(name);
        }
        None => {}
    }
    write_active_run_payload(run_id, payload)
}

pub fn write_active_run_payload(run_id: &str, payload: Value) -> Result<()> {
    let _lock = acquire_active_run_lock(run_id)?;
    write_active_run_payload_unlocked(run_id, payload)
}

fn write_active_run_payload_unlocked(run_id: &str, payload: Value) -> Result<()> {
    let path = active_run_path(run_id)?;
    let temporary = path.with_extension("tmp");
    {
        let mut file = create_private_file(&temporary)?;
        writeln!(file, "{}", serde_json::to_string(&payload)?)?;
        file.flush()?;
    }
    fs::rename(temporary, path)?;
    Ok(())
}

pub fn find_active_run_record(run_id: &str) -> Result<Value> {
    let _lock = acquire_active_run_lock(run_id)?;
    find_active_run_record_unlocked(run_id)?
        .ok_or_else(|| anyhow::anyhow!("active run not found: {run_id}"))
}

fn find_active_run_record_unlocked(run_id: &str) -> Result<Option<Value>> {
    let path = active_run_path(run_id)?;
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let payload: Value = serde_json::from_str(&text)?;
    if !payload.is_object() {
        bail!("active run record is invalid: {run_id}");
    }
    Ok(Some(payload))
}

pub fn mark_active_run_stop_requested(run_id: &str) -> Result<()> {
    mark_active_run_status(run_id, "stop-requested", "stop_requested_at")
}

pub fn clear_active_run_stop_requested(run_id: &str) -> Result<()> {
    clear_active_run_status(run_id, "stop_requested_at")
}

pub fn mark_active_run_kill_requested(run_id: &str) -> Result<()> {
    mark_active_run_status(run_id, "kill-requested", "kill_requested_at")
}

pub fn clear_active_run_kill_requested(run_id: &str) -> Result<()> {
    clear_active_run_status(run_id, "kill_requested_at")
}

fn mark_active_run_status(run_id: &str, status: &str, timestamp_key: &str) -> Result<()> {
    let updated = update_active_run_record_if_present(run_id, |record| {
        record["status"] = json!(status);
        record[timestamp_key] = json!(crate::provider::observability::utc_timestamp());
    })?;
    if !updated {
        bail!("active run not found: {run_id}");
    }
    Ok(())
}

fn clear_active_run_status(run_id: &str, timestamp_key: &str) -> Result<()> {
    update_active_run_record_if_present(run_id, |record| {
        record["status"] = json!("running");
        if let Some(object) = record.as_object_mut() {
            object.remove(timestamp_key);
        }
    })?;
    Ok(())
}

fn update_active_run_record_if_present(
    run_id: &str,
    update: impl FnOnce(&mut Value),
) -> Result<bool> {
    let _lock = acquire_active_run_lock(run_id)?;
    let Some(mut record) = find_active_run_record_unlocked(run_id)? else {
        return Ok(false);
    };
    update(&mut record);
    write_active_run_payload_unlocked(run_id, record)?;
    Ok(true)
}

pub fn active_run_terminal_status(run_id: &str) -> Option<String> {
    let record = find_active_run_record(run_id).ok()?;
    if record
        .get("kill_requested_at")
        .and_then(Value::as_str)
        .is_some()
    {
        return Some("killed".to_string());
    }
    if record
        .get("stop_requested_at")
        .and_then(Value::as_str)
        .is_some()
    {
        return Some("stopped".to_string());
    }
    None
}

pub fn remove_active_run_record(run_id: &str) -> Result<()> {
    let _lock = acquire_active_run_lock(run_id)?;
    remove_active_run_record_unlocked(run_id)
}

fn remove_active_run_record_unlocked(run_id: &str) -> Result<()> {
    let path = active_run_path(run_id)?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("could not remove active run marker for {run_id}"))
        }
    }
}

pub fn read_active_run_records() -> Vec<Value> {
    let dir = active_runs_dir();
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut records = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Some(run_id) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if validate_run_id(run_id).is_err() {
            continue;
        }
        let Ok(payload) = find_active_run_record(run_id) else {
            continue;
        };
        let Some(payload_run_id) = payload.get("run_id").and_then(Value::as_str) else {
            continue;
        };
        let Some(container_name) = payload.get("container_name").and_then(Value::as_str) else {
            continue;
        };
        if payload_run_id == run_id
            && validate_run_id(payload_run_id).is_ok()
            && validate_runhaven_container_name(container_name).is_ok()
        {
            records.push(payload);
        }
    }
    records.sort_by_key(|record| {
        (
            record
                .get("timestamp")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            record
                .get("run_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        )
    });
    records
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::support::paths::override_cache_root_for_tests;
    use std::os::unix::fs::PermissionsExt;

    use crate::runtime::plans::{AgentRunPlan, AuthScope, NetworkMode, WorkspaceScope};
    use std::path::PathBuf;

    fn isolated_cache() -> (tempfile::TempDir, crate::support::paths::CacheRootOverride) {
        let cache = tempfile::tempdir().expect("cache");
        let cache_home = override_cache_root_for_tests(cache.path());
        (cache, cache_home)
    }

    fn broker_plan(claude_auth_source: Option<ClaudeAuthSource>) -> AgentRunPlan {
        AgentRunPlan {
            command: vec!["container".to_string(), "run".to_string()],
            preflight: Vec::new(),
            workspace: PathBuf::from("/tmp/broker-workspace"),
            state_volume: "runhaven-codex-broker-home".to_string(),
            session: "broker".to_string(),
            container_name: "runhaven-codex-broker-run".to_string(),
            profile_name: "codex".to_string(),
            workspace_scope: WorkspaceScope::Current,
            workspace_scope_note: None,
            auth_scope: AuthScope::Agent,
            worktree: None,
            run_id: Some("run-broker".to_string()),
            network_name: None,
            network_mode: NetworkMode::Provider,
            egress_summary: "provider egress".to_string(),
            image: "image".to_string(),
            provider_allowed_hosts: Vec::new(),
            claude_auth_source,
            security_notices: Vec::new(),
        }
    }

    #[test]
    fn active_run_record_persists_api_key_broker_env_for_read_model() {
        let (_cache, _cache_home) = isolated_cache();

        write_active_run_record(
            &broker_plan(Some(ClaudeAuthSource::ApiKeyBrokerEnv(
                "OPENAI_API_KEY".to_string(),
            ))),
            "run-broker",
            "2026-07-01T00:00:00Z",
        )
        .expect("write brokered run");
        let brokered = find_active_run_record("run-broker").expect("brokered record");
        assert_eq!(
            brokered.get("api_key_broker_env").and_then(Value::as_str),
            Some("OPENAI_API_KEY"),
            "brokered run must persist broker proof for the managed-session read model"
        );

        write_active_run_record(&broker_plan(None), "run-broker", "2026-07-01T00:00:00Z")
            .expect("write non-brokered run");
        let plain = find_active_run_record("run-broker").expect("non-brokered record");
        assert!(
            plain.get("api_key_broker_env").is_none(),
            "runs without a broker env must not fabricate broker proof"
        );
    }

    #[test]
    fn active_run_markers_use_owner_only_permissions() {
        let (_cache, _cache_home) = isolated_cache();

        write_active_run_payload(
            "permission-test",
            json!({
                "run_id": "permission-test",
                "container_name": "runhaven-permission-test"
            }),
        )
        .expect("active run marker");

        let dir_mode = fs::metadata(active_runs_dir())
            .expect("active runs dir")
            .permissions()
            .mode()
            & 0o777;
        let file_mode = fs::metadata(active_run_path("permission-test").expect("marker path"))
            .expect("active run file")
            .permissions()
            .mode()
            & 0o777;

        assert_eq!(dir_mode, 0o700);
        assert_eq!(file_mode, 0o600);
    }

    #[test]
    fn removing_an_absent_active_marker_is_idempotent() {
        let (_cache, _cache_home) = isolated_cache();

        remove_active_run_record("already-absent").expect("missing marker is already removed");
    }

    #[test]
    fn active_marker_removal_propagates_non_not_found_errors() {
        let (_cache, _cache_home) = isolated_cache();
        let path = active_run_path("removal-error").expect("marker path");
        fs::create_dir_all(&path).expect("directory at marker path");

        let error = remove_active_run_record("removal-error")
            .expect_err("a real removal failure must not be reported as success");

        assert!(
            error
                .to_string()
                .contains("could not remove active run marker")
        );
        assert!(path.is_dir(), "the failed removal target must remain");
    }

    #[test]
    fn update_waiting_behind_removal_does_not_recreate_the_marker() {
        use std::sync::{Arc, Barrier};

        let (_cache, _cache_home) = isolated_cache();
        let run_id = "remove-before-update";
        write_active_run_payload(
            run_id,
            json!({
                "run_id": run_id,
                "container_name": "runhaven-remove-before-update"
            }),
        )
        .expect("seed marker");
        let held = acquire_active_run_lock(run_id).expect("hold marker lock");
        let ready = Arc::new(Barrier::new(2));
        let thread_ready = Arc::clone(&ready);
        let update = std::thread::spawn(move || {
            thread_ready.wait();
            mark_active_run_stop_requested(run_id)
        });

        ready.wait();
        remove_active_run_record_unlocked(run_id).expect("remove while serialized");
        drop(held);

        let error = update
            .join()
            .expect("update thread")
            .expect_err("a removed marker must not be recreated");
        assert!(error.to_string().contains("active run not found"));
        assert!(!active_run_path(run_id).expect("marker path").exists());
    }

    #[test]
    fn concurrent_marker_mutations_preserve_every_serialized_update() {
        use std::sync::{Arc, Barrier};

        const WRITERS: usize = 12;
        let (_cache, _cache_home) = isolated_cache();
        let run_id = "concurrent-marker-updates";
        write_active_run_payload(
            run_id,
            json!({
                "run_id": run_id,
                "container_name": "runhaven-concurrent-marker-updates"
            }),
        )
        .expect("seed marker");
        let ready = Arc::new(Barrier::new(WRITERS));
        let writers = (0..WRITERS)
            .map(|index| {
                let ready = Arc::clone(&ready);
                std::thread::spawn(move || {
                    ready.wait();
                    update_active_run_record_if_present(run_id, |record| {
                        record
                            .as_object_mut()
                            .expect("marker object")
                            .insert(format!("writer_{index}"), json!(true));
                    })
                    .expect("serialized update")
                })
            })
            .collect::<Vec<_>>();

        for writer in writers {
            assert!(writer.join().expect("writer thread"));
        }
        let record = find_active_run_record(run_id).expect("updated marker");
        for index in 0..WRITERS {
            assert_eq!(record[format!("writer_{index}")], true);
        }
    }
}
