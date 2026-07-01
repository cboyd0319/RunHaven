//! Background execution of a bundled agent-image rebuild.
//!
//! The build is the tested core `ImageBuildPlan` (the same command the CLI
//! `image build`/`image rebuild` runs). RunHaven spawns it on a dedicated thread
//! so the Tauri command never blocks on the full build, streams stdout and
//! stderr line by line into the sanitized `ImageBuildStatusRegistry`, and marks
//! the terminal state when the builder exits.

use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::thread;

use runhaven_core::image::build::ImageBuildPlan;

use crate::image_build_status::ImageBuildStatusRegistry;

/// Spawn the build on a background thread. Returns once the thread is started;
/// progress and the terminal state are recorded through `registry`.
pub(crate) fn spawn_image_build(
    build_id: String,
    plan: ImageBuildPlan,
    registry: ImageBuildStatusRegistry,
) -> Result<(), String> {
    thread::Builder::new()
        .name(format!("runhaven-image-build-{build_id}"))
        .spawn(move || match run_build(&build_id, &plan, &registry) {
            Ok(0) => {
                if let Err(error) = registry.mark_succeeded(&build_id) {
                    eprintln!("runhaven image build {build_id} status update failed: {error}");
                }
            }
            Ok(_) => mark_failed(
                &registry,
                &build_id,
                "The agent image rebuild finished with errors. Open diagnostics or try again."
                    .to_string(),
            ),
            Err(error) => {
                eprintln!("runhaven image build {build_id} failed: {error}");
                mark_failed(
                    &registry,
                    &build_id,
                    "RunHaven could not run the agent image rebuild. Open diagnostics or try again."
                        .to_string(),
                );
            }
        })
        .map(|_| ())
        .map_err(|error| format!("could not start background image rebuild: {error}"))
}

fn run_build(
    build_id: &str,
    plan: &ImageBuildPlan,
    registry: &ImageBuildStatusRegistry,
) -> Result<i32, String> {
    let Some(program) = plan.command.first() else {
        return Err("image build command is empty".to_string());
    };
    let mut child = Command::new(program)
        .args(&plan.command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start image build: {error}"))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_handle = stdout.map(|reader| capture_output(build_id, reader, registry));
    let stderr_handle = stderr.map(|reader| capture_output(build_id, reader, registry));

    let status = child
        .wait()
        .map_err(|error| format!("could not wait for image build: {error}"))?;
    if let Some(handle) = stdout_handle {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }
    Ok(status.code().unwrap_or(1))
}

fn capture_output<R>(
    build_id: &str,
    reader: R,
    registry: &ImageBuildStatusRegistry,
) -> thread::JoinHandle<()>
where
    R: Read + Send + 'static,
{
    let build_id = build_id.to_string();
    let registry = registry.clone();
    thread::spawn(move || {
        let mut lines = BufReader::new(reader).lines();
        while let Some(Ok(line)) = lines.next() {
            if let Err(error) = registry.append_output(&build_id, &line) {
                eprintln!("runhaven image build {build_id} output capture failed: {error}");
                return;
            }
        }
    })
}

fn mark_failed(registry: &ImageBuildStatusRegistry, build_id: &str, detail: String) {
    if let Err(error) = registry.mark_failed(build_id, detail) {
        eprintln!("runhaven image build {build_id} status update failed: {error}");
    }
}
