//! Injected test seams for terminal commands. Keeping these outside the command
//! module preserves its production ownership boundary and file-size headroom.

use std::sync::Arc;

use runhaven_core::runtime::plans::AgentRunPlan;

use super::{
    GuiAttachPlan, SpawnedTerminal, TerminalHandle, TerminalSessionRegistry, pty,
    register_and_stream, resolve_and_reserve, spawn_confirmed_run_terminal,
    validate_terminal_session_id,
};
use crate::session_output::SessionOutputRegistry;

struct NoopTerminal;

impl TerminalHandle for NoopTerminal {
    fn write_input(&self, data: &[u8]) -> Result<usize, String> {
        Ok(data.len())
    }

    fn resize(&self, _cols: u16, _rows: u16) -> Result<(), String> {
        Ok(())
    }

    fn shutdown(&self) {}
}

pub(crate) fn open_terminal_session_for_test(
    registry: &TerminalSessionRegistry,
    session_id: &str,
    resolve: impl FnOnce(&str) -> Result<GuiAttachPlan, String>,
    approve: impl FnOnce(&GuiAttachPlan) -> Result<bool, String>,
    spawn: impl FnOnce(&GuiAttachPlan) -> Result<SpawnedTerminal, String>,
) -> Result<bool, String> {
    let (plan, reservation) = resolve_and_reserve(registry, session_id, resolve)?;
    if !approve(&plan)? {
        return Ok(false);
    }
    let spawned = spawn(&plan)?;
    register_and_stream(registry, reservation, spawned)?;
    Ok(true)
}

pub(crate) fn open_agent_sign_in_terminal_for_test(
    registry: &TerminalSessionRegistry,
    agent: &str,
    session_id: &str,
    approve: impl FnOnce(&str) -> Result<bool, String>,
    spawn: impl FnOnce(&str, &str) -> Result<SpawnedTerminal, String>,
) -> Result<bool, String> {
    validate_terminal_session_id(session_id)?;
    if !runhaven_core::runtime::login::supports_sandbox_login(agent) {
        return Err("Choose a supported agent before opening sign-in.".to_string());
    }
    let reservation = registry.reserve(session_id)?;
    if !approve(agent)? {
        return Ok(false);
    }
    let execution_id = runhaven_core::runtime::launch::new_run_id();
    let spawned = spawn(agent, &execution_id)?;
    register_and_stream(registry, reservation, spawned)?;
    Ok(true)
}

pub(crate) fn open_agent_run_terminal_for_test(
    registry: &TerminalSessionRegistry,
    output_registry: &SessionOutputRegistry,
    session_id: &str,
    prepare_confirm: impl FnOnce() -> Result<Option<AgentRunPlan>, String>,
    spawn: impl FnOnce(AgentRunPlan, &str, pty::RetainedRunOutput) -> Result<SpawnedTerminal, String>,
) -> Result<bool, String> {
    let reservation = registry.reserve(session_id)?;
    let Some(plan) = prepare_confirm()? else {
        return Ok(false);
    };
    spawn_confirmed_run_terminal(
        registry,
        reservation,
        output_registry,
        session_id,
        plan,
        spawn,
    )?;
    Ok(true)
}

pub(crate) fn open_agent_run_terminal_with_chunks_for_test(
    registry: &TerminalSessionRegistry,
    output_registry: &SessionOutputRegistry,
    session_id: &str,
    plan: AgentRunPlan,
    chunks: &[&[u8]],
) -> Result<bool, String> {
    open_agent_run_terminal_for_test(
        registry,
        output_registry,
        session_id,
        || Ok(Some(plan)),
        |_, _, retained_run_output| {
            pty::retain_terminal_chunks_for_test(&retained_run_output, chunks);
            Ok(SpawnedTerminal::without_reader(Arc::new(NoopTerminal)))
        },
    )
}
