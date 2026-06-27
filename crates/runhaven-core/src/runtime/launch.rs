use std::process::Command;

use anyhow::{Result, bail};
use serde_json::json;

use super::lock::acquire_state_lock;
use crate::doctor::find_on_path;
use crate::provider::observability::utc_timestamp;
use crate::provider::runtime as provider_runtime;
use crate::records::{RunRecordInput, write_run_record};
use crate::runtime::active::{
    active_run_terminal_status, remove_active_run_record, write_active_run_record,
};
use crate::runtime::plans::{AgentRunPlan, NetworkMode};
use crate::support::git::{capture_git_snapshot, summarize_git_change};

pub fn new_run_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn launch_run_plan(plan: &AgentRunPlan) -> Result<i32> {
    provider_runtime::validate_runtime_auth_broker_environment(plan)?;
    require_container_cli()?;
    ensure_agent_image_built(plan)?;
    let _state_lock = acquire_state_lock(&plan.state_volume)?;
    if plan.network_mode == NetworkMode::Provider {
        return provider_runtime::run_provider_agent(plan);
    }
    for command in &plan.preflight {
        provider_runtime::run_preflight(command)?;
    }
    run_standard_agent(plan)
}

/// Fail early with an actionable message when the agent image is not built,
/// instead of letting `container run` try to pull a RunHaven image and return a
/// confusing `registry-1.docker.io 401`.
fn ensure_agent_image_built(plan: &AgentRunPlan) -> Result<()> {
    if crate::image::doctor::image_is_built(&plan.image)? {
        return Ok(());
    }
    bail!(
        "{}",
        image_not_built_message(&plan.image, &plan.profile_name)
    );
}

fn image_not_built_message(image: &str, agent: &str) -> String {
    if image.starts_with("runhaven/") {
        format!(
            "The {agent} image is not built yet, so the sandbox cannot start. Build it once with:\n  runhaven image build {agent}"
        )
    } else {
        format!("The image {image} is not available locally. Build or load it before running.")
    }
}

pub fn run_standard_agent(plan: &AgentRunPlan) -> Result<i32> {
    let run_id = plan.run_id.clone().unwrap_or_else(new_run_id);
    let git_before = capture_git_snapshot(&plan.workspace);
    let started_at = utc_timestamp();
    eprintln!("Run id: {run_id}");
    write_active_run_record(plan, &run_id, &started_at)?;
    let injection = crate::runtime::login::run_token_injection(plan);
    let command = match &injection {
        Some((env, _)) => crate::runtime::login::with_token_env(&plan.command, &plan.image, env),
        None => plan.command.clone(),
    };
    let mut agent_command = Command::new(&command[0]);
    agent_command.args(&command[1..]);
    if let Some((env, value)) = &injection {
        eprintln!(
            "Using your stored {} login (injected into the sandbox env).",
            plan.profile_name
        );
        agent_command.env(env, value);
    }
    let status_result = agent_command.status();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_not_built_message_points_to_the_build_command_for_bundled_images() {
        let bundled = image_not_built_message("runhaven/codex:0.1.0", "codex");
        assert!(bundled.contains("runhaven image build codex"));
        // A custom --image is the user's to provide, so no build hint.
        let custom = image_not_built_message("my/custom:1.0", "codex");
        assert!(!custom.contains("runhaven image build"));
        assert!(custom.contains("my/custom:1.0"));
    }
}
