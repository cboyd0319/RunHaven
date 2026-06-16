use std::process::Command;

use anyhow::{Result, bail};
use serde_json::json;

use super::lock::acquire_state_lock;
use crate::active::{
    active_run_terminal_status, remove_active_run_record, write_active_run_record,
};
use crate::doctor::find_on_path;
use crate::git::{capture_git_snapshot, summarize_git_change};
use crate::plans::{AgentRunPlan, NetworkMode};
use crate::provider_observability::utc_timestamp;
use crate::provider_runtime;
use crate::records::{RunRecordInput, write_run_record};

pub fn new_run_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn launch_run_plan(plan: &AgentRunPlan) -> Result<i32> {
    provider_runtime::validate_runtime_auth_broker_environment(plan)?;
    require_container_cli()?;
    let _state_lock = acquire_state_lock(&plan.state_volume)?;
    if plan.network_mode == NetworkMode::Provider {
        return provider_runtime::run_provider_agent(plan);
    }
    for command in &plan.preflight {
        provider_runtime::run_preflight(command)?;
    }
    run_standard_agent(plan)
}

pub fn run_standard_agent(plan: &AgentRunPlan) -> Result<i32> {
    let run_id = plan.run_id.clone().unwrap_or_else(new_run_id);
    let git_before = capture_git_snapshot(&plan.workspace);
    let started_at = utc_timestamp();
    eprintln!("Run id: {run_id}");
    write_active_run_record(plan, &run_id, &started_at)?;
    let status_result = Command::new(&plan.command[0])
        .args(&plan.command[1..])
        .status();
    let terminal_status = active_run_terminal_status(&run_id);
    let _ = remove_active_run_record(&run_id);
    let finished_at = utc_timestamp();
    let mut command_error = None;
    let return_code = match status_result {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            command_error = Some(anyhow::anyhow!(
                "could not launch agent command {:?}: {error}",
                plan.command[0]
            ));
            1
        }
    };
    let git = summarize_git_change(git_before, capture_git_snapshot(&plan.workspace));
    write_run_record(RunRecordInput {
        plan,
        run_id: &run_id,
        started_at: &started_at,
        finished_at: &finished_at,
        return_code,
        status: terminal_status.as_deref(),
        provider_decisions: &[],
        auth_decisions: None,
        cleanup: json!({"provider_network": "not-applicable"}),
        git,
    })?;
    if let Some(error) = command_error {
        return Err(error);
    }
    Ok(return_code)
}

pub fn require_container_cli() -> Result<()> {
    if find_on_path("container").is_some() {
        return Ok(());
    }
    bail!(
        "Apple container CLI was not found. Install it from https://github.com/apple/container/releases and run `container system start`."
    )
}
