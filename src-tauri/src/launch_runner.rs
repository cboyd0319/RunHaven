use std::io::{Read, Write};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use runhaven_core::apple_container;
use runhaven_core::runtime::launch::{AgentCommandContext, AgentCommandRunner};

use crate::launch_status::LaunchStatusRegistry;
use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;

pub(crate) struct TauriLaunchRunner {
    input_registry: SessionInputRegistry,
    output_registry: SessionOutputRegistry,
    launch_status: LaunchStatusRegistry,
}

impl TauriLaunchRunner {
    pub(crate) fn new(
        input_registry: SessionInputRegistry,
        output_registry: SessionOutputRegistry,
        launch_status: LaunchStatusRegistry,
    ) -> Self {
        Self {
            input_registry,
            output_registry,
            launch_status,
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

    fn run_agent_command_with_mark_running(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        context: AgentCommandContext<'_>,
        mark_running: impl FnOnce(&LaunchStatusRegistry, &str) -> Result<(), String>,
    ) -> Result<i32> {
        let run_id = context.run_id;
        // Wall-clock from just before spawn. The Apple `container` startup steps
        // ("[0/6] Fetching image" .. "[6/6] Starting container") are opaque child
        // stdout captured into the sanitized session-output registry, not lines
        // this runner emits, so per-step timing cannot be intercepted here
        // without touching core launch.rs or routing captured child bytes (an
        // agent-output leak surface). Instead, log the two boundaries this runner
        // owns -- process spawn and process exit -- with elapsed time, so a slow
        // launch shows spawn-to-running cost and total run duration.
        let launch_start = Instant::now();
        log::info!("run {run_id}: launching container process");
        let Some(program) = command.first() else {
            bail!("agent command is empty");
        };
        let mut agent_command = apple_container::command_for_program(program)?;
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
        if let Err(error) = self.output_registry.register(context.run_id) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow::Error::msg(error));
        }
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
        if let Err(error) = mark_running(&self.launch_status, context.run_id) {
            let launch_error = anyhow::Error::msg(error);
            if let Err(cleanup_error) =
                cleanup_failed_spawn(&mut child, stdout_handle, stderr_handle, registration)
            {
                return Err(launch_error.context(format!(
                    "could not fully clean up the spawned agent command: {cleanup_error}"
                )));
            }
            return Err(launch_error);
        }
        log::info!(
            "run {run_id}: container process spawned and running after {} ms",
            launch_start.elapsed().as_millis()
        );

        let status = child
            .wait()
            .with_context(|| format!("could not wait for agent command {program:?}"))?;
        join_output_capture(stdout_handle)?;
        join_output_capture(stderr_handle)?;
        drop(registration);
        let code = status.code().unwrap_or(1);
        log::info!(
            "run {run_id}: container process exited code={code} after {} ms total",
            launch_start.elapsed().as_millis()
        );
        Ok(code)
    }
}

impl AgentCommandRunner for TauriLaunchRunner {
    fn run_agent_command(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        context: AgentCommandContext<'_>,
    ) -> Result<i32> {
        self.run_agent_command_with_mark_running(
            command,
            env,
            context,
            LaunchStatusRegistry::mark_running,
        )
    }
}

fn join_output_capture(handle: thread::JoinHandle<Result<()>>) -> Result<()> {
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("session output reader panicked"))?
}

fn cleanup_failed_spawn(
    child: &mut std::process::Child,
    stdout_handle: thread::JoinHandle<Result<()>>,
    stderr_handle: thread::JoinHandle<Result<()>>,
    registration: InputRegistration<'_>,
) -> Result<()> {
    let mut failures = Vec::new();
    match child.try_wait() {
        Ok(Some(_)) => {}
        Ok(None) => {
            if let Err(error) = child.kill() {
                failures.push(format!("could not stop child: {error}"));
            }
        }
        Err(error) => {
            failures.push(format!("could not inspect child: {error}"));
            if let Err(error) = child.kill() {
                failures.push(format!("could not stop child: {error}"));
            }
        }
    }
    if let Err(error) = child.wait() {
        failures.push(format!("could not reap child: {error}"));
    }
    if let Err(error) = join_output_capture(stdout_handle) {
        failures.push(format!("stdout capture failed: {error:#}"));
    }
    if let Err(error) = join_output_capture(stderr_handle) {
        failures.push(format!("stderr capture failed: {error:#}"));
    }
    drop(registration);
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(failures.join("; "))
    }
}

struct InputRegistration<'a> {
    registry: &'a SessionInputRegistry,
    session_id: String,
}

impl Drop for InputRegistration<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.registry.unregister(&self.session_id) {
            log::warn!(
                "run {}: session input unregister failed: {error}",
                self.session_id
            );
        }
    }
}

#[cfg(test)]
mod tests;
