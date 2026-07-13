//! Background execution of a bundled agent-image rebuild.
//!
//! The build is the tested core `ImageBuildPlan` (the same command the CLI
//! `image build`/`image rebuild` runs). RunHaven spawns it on a dedicated thread
//! so the Tauri command never blocks on the full build, streams stdout and
//! stderr line by line into the sanitized `ImageBuildStatusRegistry`, and marks
//! the terminal state when the builder exits.

use std::io::Read;
use std::process::Stdio;
use std::thread;

use runhaven_core::apple_container;
use runhaven_core::image::build::ImageBuildPlan;

use crate::background_jobs::JobLease;
use crate::registry_support::capture_lines;
use crate::tail_status::ImageBuildStatusRegistry;

/// Spawn the build on a background thread. Returns once the thread is started;
/// progress and the terminal state are recorded through `registry`.
pub(crate) fn spawn_image_build(
    build_id: String,
    plan: ImageBuildPlan,
    registry: ImageBuildStatusRegistry,
    lease: JobLease,
) -> Result<(), String> {
    thread::Builder::new()
        .name(format!("runhaven-image-build-{build_id}"))
        .spawn(
            move || match run_build(&build_id, &plan, &registry, &lease) {
                Ok(0) => {
                    if let Err(error) = registry.mark_succeeded(&build_id) {
                        log::error!("image build {build_id}: status update failed: {error}");
                    }
                }
                Ok(_) => mark_failed(
                    &registry,
                    &build_id,
                    "The agent installation finished with errors. Open diagnostics or try again."
                        .to_string(),
                ),
                Err(error) => {
                    log::error!("image build {build_id}: failed: {error}");
                    mark_failed(
                    &registry,
                    &build_id,
                        "RunHaven could not run the agent installation. Open diagnostics or try again."
                        .to_string(),
                );
                }
            },
        )
        .map(|_| ())
        .map_err(|error| format!("could not start background image rebuild: {error}"))
}

fn run_build(
    build_id: &str,
    plan: &ImageBuildPlan,
    registry: &ImageBuildStatusRegistry,
    lease: &JobLease,
) -> Result<i32, String> {
    let Some(program) = plan.command.first() else {
        return Err("image build command is empty".to_string());
    };
    let mut command =
        apple_container::command_for_program(program).map_err(|error| error.to_string())?;
    command
        .args(&plan.command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = lease.spawn_child(&mut command)?;

    let stdout = child.take_stdout();
    let stderr = child.take_stderr();
    let stdout_handle = stdout.map(|reader| capture_output(build_id, reader, registry));
    let stderr_handle = stderr.map(|reader| capture_output(build_id, reader, registry));

    let status = child.wait()?;
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
        if let Err(error) = capture_lines(reader, |line| registry.append_output(&build_id, line)) {
            log::warn!("image build {build_id}: output capture failed: {error}");
        }
    })
}

fn mark_failed(registry: &ImageBuildStatusRegistry, build_id: &str, detail: String) {
    if let Err(error) = registry.mark_failed(build_id, detail) {
        log::error!("image build {build_id}: status update failed: {error}");
    }
}
