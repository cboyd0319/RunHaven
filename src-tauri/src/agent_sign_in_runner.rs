//! Background execution of an in-app agent sign-in.
//!
//! The sign-in is the tested core device-code login plan (the same in-sandbox
//! login the CLI `runhaven login <agent>` runs) executed through the core
//! `AgentCommandRunner` seam. RunHaven spawns it on a dedicated thread so the
//! Tauri command never blocks on the whole login, captures the agent's stdout
//! and stderr line by line into the sanitized `AgentSignInStatusRegistry` (so
//! the sign-in URL and short-lived device code show while any leaked secret is
//! scrubbed), and marks the terminal state when the login exits.
//!
//! The desktop app only triggers the sign-in; the resulting credential is owned
//! by RunHaven core. Nothing here reads, stores, or forwards a token.

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;

use anyhow::{Context, Result, bail};
use runhaven_core::runtime::launch::{
    AgentCommandContext, AgentCommandRunner, launch_run_plan_with_runner,
};
use runhaven_core::runtime::plans::AgentRunPlan;

use crate::registry_support::capture_lines;
use crate::tail_status::AgentSignInStatusRegistry;

/// Spawn the sign-in on a background thread. Returns once the thread is started;
/// progress and the terminal state are recorded through `registry`.
pub(crate) fn spawn_agent_sign_in(
    sign_in_id: String,
    agent: String,
    plan: AgentRunPlan,
    registry: AgentSignInStatusRegistry,
) -> Result<(), String> {
    thread::Builder::new()
        .name(format!("runhaven-agent-sign-in-{sign_in_id}"))
        .spawn(move || {
            let mut runner = AgentSignInRunner::new(sign_in_id.clone(), registry.clone());
            match launch_run_plan_with_runner(&plan, &mut runner) {
                Ok(0) => {
                    if let Err(error) = registry.mark_succeeded(&sign_in_id, &agent) {
                        eprintln!("runhaven sign-in {sign_in_id} status update failed: {error}");
                    }
                }
                Ok(_) => mark_failed(
                    &registry,
                    &sign_in_id,
                    "Sign-in did not finish. Open the link again or try once more.".to_string(),
                ),
                Err(error) => {
                    eprintln!("runhaven sign-in {sign_in_id} failed: {error}");
                    mark_failed(
                        &registry,
                        &sign_in_id,
                        "RunHaven could not run the sign-in. Finish setup or try again."
                            .to_string(),
                    );
                }
            }
        })
        .map(|_| ())
        .map_err(|error| format!("could not start background sign-in: {error}"))
}

/// A core command runner that captures the login command's output into the
/// sanitized status registry instead of a terminal.
struct AgentSignInRunner {
    sign_in_id: String,
    registry: AgentSignInStatusRegistry,
}

impl AgentSignInRunner {
    fn new(sign_in_id: String, registry: AgentSignInStatusRegistry) -> Self {
        Self {
            sign_in_id,
            registry,
        }
    }

    fn capture_output<R>(&self, reader: R) -> thread::JoinHandle<()>
    where
        R: Read + Send + 'static,
    {
        let sign_in_id = self.sign_in_id.clone();
        let registry = self.registry.clone();
        thread::spawn(move || {
            if let Err(error) =
                capture_lines(reader, |line| registry.append_output(&sign_in_id, line))
            {
                eprintln!("runhaven sign-in {sign_in_id} output capture failed: {error}");
            }
        })
    }
}

impl AgentCommandRunner for AgentSignInRunner {
    fn run_agent_command(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        _context: AgentCommandContext<'_>,
    ) -> Result<i32> {
        let Some(program) = command.first() else {
            bail!("sign-in command is empty");
        };
        let mut sign_in_command = Command::new(program);
        sign_in_command
            .args(&command[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // A device-code login carries no injected env, but honor it defensively
        // if core ever supplies one; the value is never captured or logged.
        if let Some((name, value)) = env {
            sign_in_command.env(name, value);
        }

        let mut child = sign_in_command
            .spawn()
            .with_context(|| format!("could not start sign-in command {program:?}"))?;
        let stdout_handle = child
            .stdout
            .take()
            .map(|reader| self.capture_output(reader));
        let stderr_handle = child
            .stderr
            .take()
            .map(|reader| self.capture_output(reader));

        let status = child
            .wait()
            .with_context(|| format!("could not wait for sign-in command {program:?}"))?;
        if let Some(handle) = stdout_handle {
            let _ = handle.join();
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }
        Ok(status.code().unwrap_or(1))
    }
}

fn mark_failed(registry: &AgentSignInStatusRegistry, sign_in_id: &str, detail: String) {
    if let Err(error) = registry.mark_failed(sign_in_id, detail) {
        eprintln!("runhaven sign-in {sign_in_id} status update failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runhaven_core::runtime::plans::{AgentRunPlan, AuthScope, NetworkMode, WorkspaceScope};

    fn capture_plan(command: Vec<String>, workspace: &std::path::Path) -> AgentRunPlan {
        AgentRunPlan {
            command,
            preflight: Vec::new(),
            workspace: workspace.to_path_buf(),
            state_volume: "runhaven-sign-in-runner-home".to_string(),
            session: "default".to_string(),
            container_name: "runhaven-sign-in-runner-run".to_string(),
            profile_name: "codex".to_string(),
            workspace_scope: WorkspaceScope::Current,
            workspace_scope_note: None,
            auth_scope: AuthScope::Agent,
            worktree: None,
            run_id: Some("sign-in-runner-seam".to_string()),
            network_name: None,
            network_mode: NetworkMode::Internet,
            egress_summary: "unrestricted internet egress".to_string(),
            image: "image".to_string(),
            provider_allowed_hosts: Vec::new(),
            claude_auth_source: None,
            security_notices: Vec::new(),
        }
    }

    #[cfg(unix)]
    #[test]
    fn sign_in_runner_captures_redacted_output() {
        // Drive the runner directly (no container) so the seam is exercised
        // hermetically: it must capture output line by line and redact secrets.
        let workspace = tempfile::tempdir().expect("workspace");
        let plan = capture_plan(
            vec![
                "sh".to_string(),
                "-c".to_string(),
                "printf 'open https://example.com/device code WXYZ-9876\\ntoken sk-ant-api03-abcdefghij_KLmnopqrstuvwxyz1234\\n'".to_string(),
            ],
            workspace.path(),
        );
        let registry = AgentSignInStatusRegistry::default();
        registry
            .mark_starting("sign-in-runner-seam", "codex")
            .expect("starting");

        let mut runner =
            AgentSignInRunner::new("sign-in-runner-seam".to_string(), registry.clone());
        let code = runner
            .run_agent_command(
                &plan.command,
                None,
                AgentCommandContext {
                    plan: &plan,
                    run_id: "sign-in-runner-seam",
                    started_at: "2026-07-01T00:00:00Z",
                },
            )
            .expect("runner exit");

        assert_eq!(code, 0);
        let status = registry.get("sign-in-runner-seam").expect("status");
        assert_eq!(status.state, "needs_browser");
        assert!(status.output_tail.contains("WXYZ-9876"));
        assert!(!status.output_tail.contains("sk-ant-api03"));
    }
}
