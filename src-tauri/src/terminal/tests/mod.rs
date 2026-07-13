//! Behavior tests for the terminal session boundary. The real host-PTY spawn is
//! injected, so these cover id validation, unknown/inactive rejection, the
//! concurrent cap, confirm-cancel, register/remove/write/resize/close, reader
//! spawn failure, poisoned-registry teardown, and the validated attach argv.
//! The child kill/reap deadlock-safety (F1) is tested in
//! `runhaven_core::runtime::terminal`, where the child lifecycle now lives. The
//! no-persist/no-log source tripwire lives in `terminal::source_tripwire_tests`;
//! the live PTY behavior itself is the slice-4 `tauri dev` smoke.

use std::sync::{Arc, Mutex};

use runhaven_core::runtime::active::{gui_attach_plan, write_active_run_payload};
use runhaven_core::support::paths::override_cache_root_for_tests;
use serde_json::json;

use super::*;

// The agent sign-in terminal and the interactive agent-run terminal
// (task #23, Option A) tests live in their own files so this attach-focused module
// stays under its size ceiling; they reuse the fakes here.
mod login;
mod run;

#[derive(Default)]
struct FakeCalls {
    writes: Vec<Vec<u8>>,
    resizes: Vec<(u16, u16)>,
    shutdowns: usize,
}

/// A terminal handle that records calls instead of touching a PTY, so the
/// registry and open flow are testable with no live container.
#[derive(Clone, Default)]
struct FakeTerminal {
    calls: Arc<Mutex<FakeCalls>>,
}

impl FakeTerminal {
    fn shutdowns(&self) -> usize {
        self.calls.lock().expect("fake calls").shutdowns
    }
    fn handle(&self) -> Arc<dyn TerminalHandle> {
        Arc::new(self.clone())
    }
}

impl TerminalHandle for FakeTerminal {
    fn write_input(&self, data: &[u8]) -> Result<usize, String> {
        self.calls
            .lock()
            .expect("fake calls")
            .writes
            .push(data.to_vec());
        Ok(data.len())
    }
    fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.calls
            .lock()
            .expect("fake calls")
            .resizes
            .push((cols, rows));
        Ok(())
    }
    fn shutdown(&self) {
        self.calls.lock().expect("fake calls").shutdowns += 1;
    }
}

fn fake_plan(run_id: &str) -> GuiAttachPlan {
    GuiAttachPlan {
        run_id: run_id.to_string(),
        container_name: format!("runhaven-codex-{run_id}"),
        args: vec![
            "exec".to_string(),
            "--interactive".to_string(),
            "--tty".to_string(),
            "--user".to_string(),
            "agent".to_string(),
            "--workdir".to_string(),
            "/workspace".to_string(),
            format!("runhaven-codex-{run_id}"),
            "/bin/bash".to_string(),
        ],
    }
}

/// Open a session in the registry with a fake handle and always-approve.
fn open_fake(
    registry: &TerminalSessionRegistry,
    run_id: &str,
) -> (Result<(), String>, FakeTerminal) {
    let fake = FakeTerminal::default();
    let spawn_fake = fake.clone();
    let result = open_terminal_session_for_test(
        registry,
        run_id,
        |id| Ok(fake_plan(id)),
        |_| Ok(true),
        move |_| Ok(SpawnedTerminal::without_reader(spawn_fake.handle())),
    );
    (result, fake)
}

#[test]
fn invalid_session_id_is_rejected_before_resolve() {
    let registry = TerminalSessionRegistry::default();
    let mut resolved = false;
    let result = open_terminal_session_for_test(
        &registry,
        "bad id\u{1b}[2J",
        |_| {
            resolved = true;
            Ok(fake_plan("run"))
        },
        |_| Ok(true),
        |_| panic!("must not spawn on an invalid id"),
    );
    assert!(
        result.is_err(),
        "control-character session id must be rejected"
    );
    assert!(
        !resolved,
        "an invalid id must fail before the read model is queried"
    );
}

#[test]
fn unknown_or_inactive_session_is_rejected_and_spawns_nothing() {
    let registry = TerminalSessionRegistry::default();
    let result = open_terminal_session_for_test(
        &registry,
        "no-such-run",
        |_| Err("not active".to_string()),
        |_| panic!("must not confirm for an inactive session"),
        |_| panic!("must not spawn for an inactive session"),
    );
    assert_eq!(result, Err("not active".to_string()));
    assert!(
        registry.remove("no-such-run").is_none(),
        "nothing must be registered"
    );
}

#[test]
fn confirm_cancel_spawns_nothing() {
    let registry = TerminalSessionRegistry::default();
    let result = open_terminal_session_for_test(
        &registry,
        "run-cancel",
        |id| Ok(fake_plan(id)),
        |_| Ok(false),
        |_| panic!("cancel must spawn nothing"),
    );
    assert_eq!(result, Err(TERMINAL_CANCELED.to_string()));
    assert!(registry.remove("run-cancel").is_none());
}

#[test]
fn concurrent_sessions_are_capped() {
    let registry = TerminalSessionRegistry::default();
    for index in 0..MAX_TERMINAL_SESSIONS {
        let (result, _) = open_fake(&registry, &format!("run-{index}"));
        result.expect("open within cap");
    }
    // The next distinct session is refused before any dialog or spawn.
    let result = open_terminal_session_for_test(
        &registry,
        "run-overflow",
        |id| Ok(fake_plan(id)),
        |_| panic!("cap must be enforced before confirmation"),
        |_| panic!("cap must be enforced before spawn"),
    );
    let error = result.expect_err("cap must hold");
    assert!(error.contains(&MAX_TERMINAL_SESSIONS.to_string()));
}

#[test]
fn duplicate_session_is_rejected() {
    let registry = TerminalSessionRegistry::default();
    open_fake(&registry, "run-dup").0.expect("first open");
    let result = open_terminal_session_for_test(
        &registry,
        "run-dup",
        |id| Ok(fake_plan(id)),
        |_| panic!("a duplicate must be refused before confirmation"),
        |_| panic!("a duplicate must not spawn"),
    );
    assert_eq!(result, Err(TERMINAL_ALREADY_OPEN.to_string()));
}

#[test]
fn write_and_resize_route_to_the_registered_handle() {
    let registry = TerminalSessionRegistry::default();
    let (result, fake) = open_fake(&registry, "run-io");
    result.expect("open");

    registry
        .write_input("run-io", b"ls -la\n")
        .expect("write routes");
    registry.resize("run-io", 120, 40).expect("resize routes");

    let calls = fake.calls.lock().expect("fake calls");
    assert_eq!(calls.writes, vec![b"ls -la\n".to_vec()]);
    assert_eq!(calls.resizes, vec![(120, 40)]);
}

#[test]
fn write_and_resize_fail_closed_for_unknown_sessions() {
    let registry = TerminalSessionRegistry::default();
    assert!(registry.write_input("ghost", b"x").is_err());
    assert!(registry.resize("ghost", 80, 24).is_err());
}

#[test]
fn close_removes_the_entry_and_shuts_the_child_down() {
    let registry = TerminalSessionRegistry::default();
    let (result, fake) = open_fake(&registry, "run-close");
    result.expect("open");

    registry.close("run-close").expect("close");
    assert_eq!(fake.shutdowns(), 1, "close must kill the child");
    assert!(registry.remove("run-close").is_none(), "entry must be gone");
    // Idempotent: closing an already-closed/absent session is not an error.
    registry.close("run-close").expect("close absent is ok");
    assert_eq!(
        fake.shutdowns(),
        1,
        "no second shutdown for an absent session"
    );
}

#[test]
fn close_all_shuts_down_every_live_session() {
    let registry = TerminalSessionRegistry::default();
    let (r1, fake1) = open_fake(&registry, "run-a");
    let (r2, fake2) = open_fake(&registry, "run-b");
    r1.expect("open a");
    r2.expect("open b");

    registry.close_all();
    assert_eq!(fake1.shutdowns(), 1);
    assert_eq!(fake2.shutdowns(), 1);
    assert!(registry.remove("run-a").is_none());
    assert!(registry.remove("run-b").is_none());
}

#[test]
fn close_during_pending_open_shuts_down_the_late_spawn() {
    let registry = TerminalSessionRegistry::default();
    let reservation = registry.reserve("pending-open").expect("reserve");
    registry.close("pending-open").expect("cancel pending open");

    let fake = FakeTerminal::default();
    let spawned = SpawnedTerminal::without_reader(fake.handle());
    let result = register_and_stream(&registry, reservation, spawned);

    assert!(result.is_err(), "a canceled pending open must not register");
    assert_eq!(fake.shutdowns(), 1, "the late child must be shut down");
    assert!(registry.remove("pending-open").is_none());
}

#[test]
fn close_all_during_pending_open_shuts_down_the_late_spawn() {
    let registry = TerminalSessionRegistry::default();
    let reservation = registry.reserve("pending-at-quit").expect("reserve");
    registry.close_all();

    let fake = FakeTerminal::default();
    let spawned = SpawnedTerminal::without_reader(fake.handle());
    let result = register_and_stream(&registry, reservation, spawned);

    assert!(result.is_err(), "app teardown must cancel a pending open");
    assert_eq!(fake.shutdowns(), 1, "the late child must be shut down");
    assert!(registry.remove("pending-at-quit").is_none());
}

#[test]
fn old_reader_generation_cannot_remove_a_reopened_session() {
    let registry = TerminalSessionRegistry::default();
    let first = FakeTerminal::default();
    let first_generation = registry
        .reserve("reused-id")
        .expect("reserve first")
        .activate(first.handle())
        .expect("activate first");
    registry.close("reused-id").expect("close first");

    let second = FakeTerminal::default();
    registry
        .reserve("reused-id")
        .expect("reserve second")
        .activate(second.handle())
        .expect("activate second");

    assert!(
        registry
            .remove_if_generation("reused-id", first_generation)
            .is_none(),
        "an old reader must not remove the replacement"
    );
    registry
        .write_input("reused-id", b"still-live")
        .expect("replacement remains registered");
    assert_eq!(second.calls.lock().expect("calls").writes.len(), 1);
    registry.close("reused-id").expect("close replacement");
}

#[test]
fn reader_spawn_failure_removes_the_entry_and_shuts_down_the_child() {
    // A registered handle whose reader thread never starts would leave a
    // cap-consuming, output-less session. The open path must instead remove the
    // entry, kill the child, and return the error.
    let registry = TerminalSessionRegistry::default();
    let fake = FakeTerminal::default();
    let spawned = SpawnedTerminal {
        handle: fake.handle(),
        start_reader: Some(Box::new(|_registry, _session_id, _generation| {
            Err("could not start the terminal: reader failed".to_string())
        })),
    };
    let reservation = registry.reserve("run-reader-fail").expect("reserve");
    let result = register_and_stream(&registry, reservation, spawned);
    assert!(result.is_err(), "a failed reader spawn must fail the open");
    assert_eq!(
        fake.shutdowns(),
        1,
        "a failed reader spawn must kill the child"
    );
    assert!(
        registry.remove("run-reader-fail").is_none(),
        "a failed reader spawn must leave no registry entry"
    );
}

#[test]
fn close_all_kills_children_even_with_a_poisoned_registry() {
    // F6: a poisoned registry mutex must not orphan children on app quit.
    let registry = TerminalSessionRegistry::default();
    let (r1, fake1) = open_fake(&registry, "run-a");
    let (r2, fake2) = open_fake(&registry, "run-b");
    r1.expect("open a");
    r2.expect("open b");

    // Poison the registry mutex by panicking while it is held.
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        registry.poison_for_test();
    }));
    assert!(poisoned.is_err(), "the panic must poison the lock");
    assert!(
        registry.is_poisoned_for_test(),
        "the registry mutex must now be poisoned"
    );

    registry.close_all();
    assert_eq!(fake1.shutdowns(), 1, "close_all must kill despite poison");
    assert_eq!(fake2.shutdowns(), 1, "close_all must kill despite poison");
}

#[test]
fn gui_attach_plan_builds_a_validated_nonroot_argv() {
    let cache = tempfile::tempdir().expect("cache");
    let _guard = override_cache_root_for_tests(cache.path());
    write_active_run_payload(
        "terminal-argv",
        json!({
            "timestamp": "2026-07-01T00:00:00Z",
            "run_id": "terminal-argv",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-terminal-argv",
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let plan = gui_attach_plan("terminal-argv").expect("resolve attach plan");
    assert_eq!(plan.run_id, "terminal-argv");
    assert_eq!(plan.container_name, "runhaven-codex-terminal-argv");
    assert_eq!(
        plan.args,
        vec![
            "exec".to_string(),
            "--interactive".to_string(),
            "--tty".to_string(),
            "--user".to_string(),
            "agent".to_string(),
            "--workdir".to_string(),
            "/workspace".to_string(),
            "runhaven-codex-terminal-argv".to_string(),
            "/bin/bash".to_string(),
        ]
    );
    // Never root, never a shell-string, never a WebView-supplied name.
    assert!(
        !plan
            .args
            .iter()
            .any(|arg| arg == "root" || arg == "--allow-root-user")
    );
}

#[test]
fn gui_attach_plan_rejects_an_unknown_run() {
    let cache = tempfile::tempdir().expect("cache");
    let _guard = override_cache_root_for_tests(cache.path());
    assert!(gui_attach_plan("not-a-real-run").is_err());
}
