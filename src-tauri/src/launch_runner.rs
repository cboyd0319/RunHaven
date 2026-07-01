use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result, bail};
use runhaven_core::runtime::launch::{AgentCommandContext, AgentCommandRunner};

use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;

pub(crate) struct TauriLaunchRunner {
    input_registry: SessionInputRegistry,
    output_registry: SessionOutputRegistry,
}

impl TauriLaunchRunner {
    pub(crate) fn new(
        input_registry: SessionInputRegistry,
        output_registry: SessionOutputRegistry,
    ) -> Self {
        Self {
            input_registry,
            output_registry,
        }
    }

    fn register_stdin(
        &self,
        run_id: &str,
        stdin: std::process::ChildStdin,
    ) -> Result<InputRegistration<'_>> {
        let stdin = Arc::new(Mutex::new(stdin));
        self.input_registry
            .register_writer(run_id, run_id, move |input| {
                let mut stdin = stdin
                    .lock()
                    .map_err(|_| "session input writer is unavailable".to_string())?;
                stdin
                    .write_all(input)
                    .map_err(|error| format!("could not write session input: {error}"))?;
                stdin
                    .flush()
                    .map_err(|error| format!("could not flush session input: {error}"))?;
                Ok(input.len())
            })
            .map_err(anyhow::Error::msg)?;
        Ok(InputRegistration {
            registry: &self.input_registry,
            session_id: run_id.to_string(),
        })
    }

    fn capture_output<R>(&self, run_id: &str, mut reader: R) -> thread::JoinHandle<Result<()>>
    where
        R: Read + Send + 'static,
    {
        let run_id = run_id.to_string();
        let registry = self.output_registry.clone();
        thread::spawn(move || {
            let mut buffer = [0_u8; 8192];
            loop {
                let read = reader
                    .read(&mut buffer)
                    .with_context(|| format!("could not read session output for run {run_id}"))?;
                if read == 0 {
                    return Ok(());
                }
                registry
                    .append(&run_id, &buffer[..read])
                    .map_err(anyhow::Error::msg)?;
            }
        })
    }
}

impl AgentCommandRunner for TauriLaunchRunner {
    fn run_agent_command(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        context: AgentCommandContext<'_>,
    ) -> Result<i32> {
        let Some(program) = command.first() else {
            bail!("agent command is empty");
        };
        let mut agent_command = Command::new(program);
        agent_command
            .args(&command[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some((name, value)) = env {
            agent_command.env(name, value);
        }

        let mut child = agent_command
            .spawn()
            .with_context(|| format!("could not launch agent command {program:?}"))?;
        let output_registration = match self.output_registry.register(context.run_id) {
            Ok(registration) => registration,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(anyhow::Error::msg(error));
            }
        };
        let Some(stdin) = child.stdin.take() else {
            let _ = child.kill();
            let _ = child.wait();
            bail!("agent command did not expose a stdin handle");
        };
        let registration = match self.register_stdin(context.run_id, stdin) {
            Ok(registration) => registration,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
        };
        let Some(stdout) = child.stdout.take() else {
            let _ = child.kill();
            let _ = child.wait();
            bail!("agent command did not expose a stdout handle");
        };
        let Some(stderr) = child.stderr.take() else {
            let _ = child.kill();
            let _ = child.wait();
            bail!("agent command did not expose a stderr handle");
        };
        let stdout_handle = self.capture_output(context.run_id, stdout);
        let stderr_handle = self.capture_output(context.run_id, stderr);

        let status = child
            .wait()
            .with_context(|| format!("could not wait for agent command {program:?}"))?;
        join_output_capture(stdout_handle)?;
        join_output_capture(stderr_handle)?;
        drop(registration);
        drop(output_registration);
        Ok(status.code().unwrap_or(1))
    }
}

fn join_output_capture(handle: thread::JoinHandle<Result<()>>) -> Result<()> {
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("session output reader panicked"))?
}

struct InputRegistration<'a> {
    registry: &'a SessionInputRegistry,
    session_id: String,
}

impl Drop for InputRegistration<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.registry.unregister(&self.session_id) {
            eprintln!(
                "runhaven session input unregister {} failed: {error}",
                self.session_id
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};

    use runhaven_core::runtime::launch::{AgentCommandContext, AgentCommandRunner};
    use runhaven_core::runtime::plans::{AgentRunPlan, AuthScope, NetworkMode, WorkspaceScope};

    use crate::session_input::SessionInputRegistry;
    use crate::session_output::SessionOutputRegistry;

    #[cfg(unix)]
    fn stdin_wait_command() -> Vec<String> {
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "IFS= read -r _line".to_string(),
        ]
    }

    #[cfg(windows)]
    fn stdin_wait_command() -> Vec<String> {
        vec![
            "cmd".to_string(),
            "/C".to_string(),
            "set /p line=".to_string(),
        ]
    }

    fn test_plan(workspace: &std::path::Path) -> AgentRunPlan {
        AgentRunPlan {
            command: stdin_wait_command(),
            preflight: Vec::new(),
            workspace: workspace.to_path_buf(),
            state_volume: "runhaven-tauri-runner-home".to_string(),
            session: "default".to_string(),
            container_name: "runhaven-tauri-runner-run".to_string(),
            profile_name: "shell".to_string(),
            workspace_scope: WorkspaceScope::Current,
            workspace_scope_note: None,
            auth_scope: AuthScope::Agent,
            worktree: None,
            run_id: Some("tauri-runner-seam".to_string()),
            network_name: None,
            network_mode: NetworkMode::Internet,
            egress_summary: "unrestricted internet egress".to_string(),
            image: "image".to_string(),
            provider_allowed_hosts: Vec::new(),
            api_key_broker_env: None,
            security_notices: Vec::new(),
        }
    }

    #[test]
    fn tauri_launch_runner_registers_stdin_until_child_exits() {
        let workspace = tempfile::tempdir().expect("workspace");
        let plan = test_plan(workspace.path());
        let registry = SessionInputRegistry::default();
        let runner_registry = registry.clone();

        let handle = thread::spawn(move || {
            let mut runner =
                super::TauriLaunchRunner::new(runner_registry, SessionOutputRegistry::default());
            runner.run_agent_command(
                &stdin_wait_command(),
                None,
                AgentCommandContext {
                    plan: &plan,
                    run_id: "tauri-runner-seam",
                    started_at: "2026-06-30T00:00:00Z",
                },
            )
        });

        let deadline = Instant::now() + Duration::from_secs(2);
        let accepted = loop {
            match registry.write("tauri-runner-seam", "tauri-runner-seam", "hello\n") {
                Ok(accepted) => break accepted,
                Err(error)
                    if error.contains("session input transport not found")
                        && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("write session input: {error}"),
            }
        };

        assert_eq!(accepted, "hello\n".len());
        assert_eq!(
            handle.join().expect("runner thread").expect("runner exit"),
            0
        );
        let missing = registry
            .write("tauri-runner-seam", "tauri-runner-seam", "again\n")
            .expect_err("writer unregistered after child exit");
        assert!(missing.contains("session input transport not found"));
    }
}
