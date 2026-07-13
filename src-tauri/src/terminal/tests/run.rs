//! Behavior tests for the interactive agent-run terminal (task #23, Option A).
//! The prepare+confirm step and the real host-PTY run spawn are injected, so
//! these cover id validation, confirm-cancel, fail-closed on a spawn failure, the
//! shared cap across attach and run, that the command forces `--interactive
//! --tty` before spawning (so the agent runs its TUI, not headless `--print`),
//! and that input routes so the user can type their prompt. The live run
//! round-trip (launch a Claude run, type a prompt, the broker authenticates,
//! output appears) is the maintainer's `tauri dev` smoke.

use std::sync::{Arc, Mutex};

use runhaven_core::runtime::plans::{
    AgentRunPlan, AuthScope, NetworkMode, RunOptions, WorkspaceScope, WorktreeRun, build_run_plan,
};
use runhaven_core::runtime::profiles::get_profile;
use runhaven_core::support::paths::override_cache_root_for_tests;

use super::{FakeTerminal, open_fake};
use crate::terminal::{
    MAX_TERMINAL_SESSIONS, SpawnedTerminal, TerminalSessionRegistry,
    open_agent_run_terminal_for_test,
};

/// A GUI-built run plan as `prepare_confirmed_terminal_run` returns it:
/// `--interactive` but not `--tty`, non-root `agent` user. The command rewrites it
/// to a TTY run before spawning. Built through the normal seam; no container.
fn interactive_run_plan() -> AgentRunPlan {
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = build_run_plan(RunOptions {
        profile: get_profile("shell").expect("profile"),
        workspace: workspace.path().to_path_buf(),
        agent_args: vec![
            "/bin/bash".to_string(),
            "-lc".to_string(),
            "true".to_string(),
        ],
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network: NetworkMode::Internet,
        workspace_scope: WorkspaceScope::Current,
        session: None,
        auth_scope: AuthScope::Agent,
        read_only_workspace: true,
        ssh: false,
        env: Vec::new(),
        user: "agent".to_string(),
        interactive: true,
        tty: false,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        provider_hosts: Vec::new(),
        claude_auth_source: None,
        worktree: None,
        run_id: Some("term-run".to_string()),
    })
    .expect("plan");
    // Precondition the command relies on: the launch path emits --interactive
    // without --tty.
    assert!(plan.command.iter().any(|arg| arg == "--interactive"));
    assert!(!plan.command.iter().any(|arg| arg == "--tty"));
    plan
}

#[test]
fn run_terminal_invalid_id_is_rejected_before_confirm_or_spawn() {
    let registry = TerminalSessionRegistry::default();
    let result = open_agent_run_terminal_for_test(
        &registry,
        "bad id\u{1b}[2J",
        || panic!("must not prepare/confirm on an invalid id"),
        |_| panic!("must not spawn on an invalid id"),
    );
    assert!(result.is_err(), "control-character run id must be rejected");
}

#[test]
fn run_terminal_confirm_cancel_spawns_nothing() {
    // The user cancelled the native launch confirmation: nothing runs, no
    // worktree branch, no registry entry.
    let registry = TerminalSessionRegistry::default();
    let result = open_agent_run_terminal_for_test(
        &registry,
        "run-cancel",
        || Ok(None),
        |_| panic!("cancel must spawn nothing"),
    );
    assert_eq!(result, Ok(false));
    assert!(registry.remove("run-cancel").is_none());
}

#[test]
fn run_terminal_forces_interactive_tty_then_registers_and_routes_input() {
    let registry = TerminalSessionRegistry::default();
    let fake = FakeTerminal::default();
    let spawn_fake = fake.clone();
    let spawned_command: Arc<Mutex<Option<Vec<String>>>> = Arc::new(Mutex::new(None));
    let recorder = spawned_command.clone();
    let result = open_agent_run_terminal_for_test(
        &registry,
        "run-io",
        || Ok(Some(interactive_run_plan())),
        move |plan| {
            *recorder.lock().expect("record") = Some(plan.command.clone());
            Ok(SpawnedTerminal::without_reader(spawn_fake.handle()))
        },
    );
    assert!(result.expect("run open"));

    // The command forced the argv to an interactive TTY run before spawn, so the
    // agent starts its TUI instead of headless --print, and it stays non-root.
    let command = spawned_command
        .lock()
        .expect("record")
        .clone()
        .expect("spawn saw the plan");
    let interactive = command
        .iter()
        .position(|arg| arg == "--interactive")
        .expect("--interactive present");
    assert_eq!(
        command.get(interactive + 1).map(String::as_str),
        Some("--tty"),
        "the command must force --tty right after --interactive"
    );
    assert!(command.windows(2).any(|w| w == ["--user", "agent"]));

    // Input routes to the run terminal so the user can type their prompt.
    registry
        .write_input("run-io", b"do the thing\n")
        .expect("prompt input routes to the run terminal");
    registry.close("run-io").expect("close");
    assert!(registry.remove("run-io").is_none());
}

#[test]
fn run_terminal_fails_closed_and_registers_nothing_when_the_spawn_fails() {
    // Fail closed: if core cannot open the run PTY (container missing, image not
    // built, proxy setup fails), nothing is registered so the UI keeps its
    // background-run fallback.
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let worktree = tempfile::tempdir().expect("worktree");
    let mut plan = interactive_run_plan();
    plan.worktree = Some(WorktreeRun {
        source_workspace: worktree.path().to_path_buf(),
        source_repo_root: worktree.path().to_path_buf(),
        worktree_root: worktree.path().to_path_buf(),
        mounted_workspace: worktree.path().to_path_buf(),
        branch: "runhaven/shell/term-run".to_string(),
        base_head: Some("base-head".to_string()),
        recovery_commands: Vec::new(),
    });
    let registry = TerminalSessionRegistry::default();
    let result = open_agent_run_terminal_for_test(
        &registry,
        "run-fail",
        || Ok(Some(plan)),
        |_| Err("RunHaven could not open the run terminal.".to_string()),
    );
    assert!(result.is_err(), "a spawn failure must fail the open");
    assert!(
        registry.remove("run-fail").is_none(),
        "a failed run open must leave no registry entry"
    );
    let record = runhaven_core::records::find_run_record("term-run")
        .expect("worktree spawn failure recovery record");
    assert_eq!(record["status"], "failed");
    assert_eq!(record["cleanup"]["worktree"], "kept-for-recovery");
}

#[test]
fn run_terminal_shares_the_terminal_cap_with_attach() {
    // The run open goes through the same registry, so it counts against the
    // shared cap alongside attach and login sessions.
    let registry = TerminalSessionRegistry::default();
    open_fake(&registry, "attach-run")
        .0
        .expect("attach within cap");
    let fake = FakeTerminal::default();
    let spawn_fake = fake.clone();
    open_agent_run_terminal_for_test(
        &registry,
        "run-one",
        || Ok(Some(interactive_run_plan())),
        move |_| Ok(SpawnedTerminal::without_reader(spawn_fake.handle())),
    )
    .expect("run within cap");
    // Cap is now full (MAX_TERMINAL_SESSIONS == 2); a third open is refused before
    // any prepare/confirm or spawn.
    let result = open_agent_run_terminal_for_test(
        &registry,
        "run-overflow",
        || panic!("cap must be enforced before prepare/confirm"),
        |_| panic!("cap must be enforced before spawn"),
    );
    let error = result.expect_err("cap must hold across attach and run");
    assert!(error.contains(&MAX_TERMINAL_SESSIONS.to_string()));
}
