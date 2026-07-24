use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use runhaven_core::runtime::launch::{AgentCommandContext, AgentCommandRunner};
use runhaven_core::runtime::plans::{AgentRunPlan, AuthScope, NetworkMode, WorkspaceScope};

use crate::launch_status::LaunchStatusRegistry;
use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;

fn stdin_wait_command() -> Vec<String> {
    vec![
        "sh".to_string(),
        "-c".to_string(),
        "IFS= read -r _line".to_string(),
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
        claude_auth_source: None,
        security_notices: Vec::new(),
    }
}

#[cfg(target_os = "macos")]
fn process_is_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(target_os = "macos")]
fn kill_process(pid: u32) {
    let _ = Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(target_os = "macos")]
#[test]
fn mark_running_failure_reaps_child_joins_output_and_unregisters_input() {
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = test_plan(workspace.path());
    let pid_path = workspace.path().join("spawned.pid");
    let command = vec![
        "sh".to_string(),
        "-c".to_string(),
        "echo $$ > \"$RUNHAVEN_TEST_PID\"; echo capture-ready; exec sleep 30".to_string(),
    ];
    let input_registry = SessionInputRegistry::default();
    let output_registry = SessionOutputRegistry::default();
    let mut runner = super::TauriLaunchRunner::new(
        input_registry.clone(),
        output_registry.clone(),
        LaunchStatusRegistry::default(),
    );

    let error = runner
        .run_agent_command_with_mark_running(
            &command,
            Some((
                "RUNHAVEN_TEST_PID",
                pid_path.to_str().expect("pid path utf-8"),
            )),
            AgentCommandContext {
                plan: &plan,
                run_id: "tauri-runner-seam",
                started_at: "2026-06-30T00:00:00Z",
            },
            |_registry, run_id| {
                let deadline = Instant::now() + Duration::from_secs(2);
                while !pid_path.exists()
                    || output_registry
                        .snapshot(run_id, 20)?
                        .is_none_or(|snapshot| !snapshot.text.contains("capture-ready"))
                {
                    assert!(
                        Instant::now() < deadline,
                        "spawned child did not become observable"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                Err("injected mark-running failure".to_string())
            },
        )
        .expect_err("mark-running failure must fail the launch");

    let pid = fs::read_to_string(&pid_path)
        .expect("spawned pid")
        .trim()
        .parse::<u32>()
        .expect("numeric pid");
    let child_survived_return = process_is_alive(pid);
    if child_survived_return {
        kill_process(pid);
    }

    assert!(error.to_string().contains("injected mark-running failure"));
    assert!(
        !child_survived_return,
        "spawned child must be killed and reaped"
    );
    assert!(
        !input_registry
            .has_writer("tauri-runner-seam", "tauri-runner-seam")
            .expect("session input registry"),
        "session input must be unregistered before the error returns"
    );
    assert!(
        output_registry
            .snapshot("tauri-runner-seam", 20)
            .expect("output snapshot")
            .expect("captured output")
            .text
            .contains("capture-ready"),
        "captured output must be drained before the error returns"
    );
}

#[test]
fn tauri_launch_runner_registers_stdin_until_child_exits() {
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = test_plan(workspace.path());
    let registry = SessionInputRegistry::default();
    let runner_registry = registry.clone();
    let launch_status = LaunchStatusRegistry::default();
    let runner_launch_status = launch_status.clone();

    let handle = thread::spawn(move || {
        let mut runner = super::TauriLaunchRunner::new(
            runner_registry,
            SessionOutputRegistry::default(),
            runner_launch_status,
        );
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
    while !registry
        .has_writer("tauri-runner-seam", "tauri-runner-seam")
        .expect("session input registry")
    {
        assert!(
            Instant::now() < deadline,
            "session input transport was not registered"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let writer_id = registry
        .writer_identity("tauri-runner-seam", "tauri-runner-seam")
        .expect("writer identity");
    let accepted = registry
        .write_exact(
            "tauri-runner-seam",
            "tauri-runner-seam",
            writer_id,
            "hello\n",
        )
        .unwrap_or_else(|error| panic!("write session input: {error}"));

    assert_eq!(accepted, "hello\n".len());
    assert_eq!(
        handle.join().expect("runner thread").expect("runner exit"),
        0
    );
    assert_eq!(
        launch_status
            .get("tauri-runner-seam")
            .expect("launch status")
            .state,
        "running"
    );
    let missing = registry
        .write_exact(
            "tauri-runner-seam",
            "tauri-runner-seam",
            writer_id,
            "again\n",
        )
        .expect_err("writer unregistered after child exit");
    assert!(missing.contains("session input transport not found"));
}
