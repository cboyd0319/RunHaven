//! Injected test seams for terminal commands. Keeping these outside the command
//! module preserves its production ownership boundary and file-size headroom.

use runhaven_core::runtime::plans::AgentRunPlan;

use super::{
    GuiAttachPlan, SpawnedTerminal, TERMINAL_CANCELED, TerminalSessionRegistry,
    record_unstarted_worktree_failure, register_and_stream, resolve_and_reserve,
    validate_terminal_session_id,
};

pub(crate) fn open_terminal_session_for_test(
    registry: &TerminalSessionRegistry,
    session_id: &str,
    resolve: impl FnOnce(&str) -> Result<GuiAttachPlan, String>,
    approve: impl FnOnce(&GuiAttachPlan) -> Result<bool, String>,
    spawn: impl FnOnce(&GuiAttachPlan) -> Result<SpawnedTerminal, String>,
) -> Result<(), String> {
    let (plan, reservation) = resolve_and_reserve(registry, session_id, resolve)?;
    if !approve(&plan)? {
        return Err(TERMINAL_CANCELED.to_string());
    }
    let spawned = spawn(&plan)?;
    register_and_stream(registry, reservation, spawned)
}

pub(crate) fn open_agent_sign_in_terminal_for_test(
    registry: &TerminalSessionRegistry,
    agent: &str,
    session_id: &str,
    approve: impl FnOnce(&str) -> Result<bool, String>,
    spawn: impl FnOnce(&str, &str) -> Result<SpawnedTerminal, String>,
) -> Result<(), String> {
    validate_terminal_session_id(session_id)?;
    if !runhaven_core::runtime::login::supports_sandbox_login(agent) {
        return Err("Choose a supported agent before opening sign-in.".to_string());
    }
    let reservation = registry.reserve(session_id)?;
    if !approve(agent)? {
        return Err(TERMINAL_CANCELED.to_string());
    }
    let execution_id = runhaven_core::runtime::launch::new_run_id();
    let spawned = spawn(agent, &execution_id)?;
    register_and_stream(registry, reservation, spawned)
}

pub(crate) fn open_agent_run_terminal_for_test(
    registry: &TerminalSessionRegistry,
    session_id: &str,
    prepare_confirm: impl FnOnce() -> Result<Option<AgentRunPlan>, String>,
    spawn: impl FnOnce(AgentRunPlan) -> Result<SpawnedTerminal, String>,
) -> Result<(), String> {
    let reservation = registry.reserve(session_id)?;
    let Some(mut plan) = prepare_confirm()? else {
        return Err(TERMINAL_CANCELED.to_string());
    };
    plan.make_interactive_tty();
    let recovery_plan = plan.worktree.as_ref().map(|_| plan.clone());
    let spawned = match spawn(plan) {
        Ok(spawned) => spawned,
        Err(error) => {
            if let Some(plan) = recovery_plan.as_ref() {
                record_unstarted_worktree_failure(plan)?;
            }
            return Err(error);
        }
    };
    register_and_stream(registry, reservation, spawned)
}
