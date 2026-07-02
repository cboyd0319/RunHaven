//! Behavior tests for the terminal session boundary. The real host-PTY spawn is
//! injected, so these cover id validation, unknown/inactive rejection, the
//! concurrent cap, confirm-cancel, register/remove/write/resize/close, reader
//! spawn failure, channel-death teardown, poisoned-registry teardown, and the
//! validated attach argv. The no-persist/no-log source tripwire lives in
//! `terminal::source_tripwire_tests`; the live PTY behavior itself is the
//! slice-4 `tauri dev` smoke.

use std::sync::{Arc, Mutex};

use runhaven_core::runtime::active::{gui_attach_plan, write_active_run_payload};
use runhaven_core::support::paths::override_cache_root_for_tests;
use serde_json::json;

use super::*;

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
fn losing_the_insert_race_shuts_down_the_orphan_spawn() {
    // Fill the cap, then drive the flow with a resolver/approver that pass but a
    // spawn whose handle must be shut down when the authoritative insert fails.
    let registry = TerminalSessionRegistry::default();
    for index in 0..MAX_TERMINAL_SESSIONS {
        open_fake(&registry, &format!("filled-{index}"))
            .0
            .expect("fill cap");
    }
    // Bypass precheck by inserting directly is not possible here, so exercise
    // register_and_stream at the cap with a fresh id: insert must fail and the
    // handle must be shut down.
    let fake = FakeTerminal::default();
    let spawned = SpawnedTerminal::without_reader(fake.handle());
    let result = register_and_stream(&registry, "orphan", spawned);
    assert!(result.is_err(), "insert past the cap must fail");
    assert_eq!(fake.shutdowns(), 1, "the orphan spawn must be shut down");
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
        start_reader: Some(Box::new(|_registry, _session_id| {
            Err("could not start the terminal: reader failed".to_string())
        })),
    };
    let result = register_and_stream(&registry, "run-reader-fail", spawned);
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
fn channel_death_kills_the_child_without_deadlocking_a_concurrent_close() {
    // F1: the reader's channel-death path must kill a still-live child (so a
    // dead Channel tears the session down instead of orphaning `/bin/bash`), and
    // an explicit close racing it must never block on a lock held across
    // `wait()`. A real long-lived child stands in for the guest shell: `wait()`
    // on it blocks forever unless something kills it, so completion proves the
    // kill happened and the reap paths are lock-safe.
    let stand_in = std::process::Command::new("/bin/sleep")
        .arg("30")
        .spawn()
        .expect("spawn a stand-in child");
    let child = Arc::new(Mutex::new(Some(stand_in)));

    let reaper_child = Arc::clone(&child);
    let closer_child = Arc::clone(&child);
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let coordinator = std::thread::spawn(move || {
        // The reader's channel-death backstop and a concurrent explicit close.
        let reaper = std::thread::spawn(move || super::pty::kill_and_reap_child(&reaper_child));
        let closer = std::thread::spawn(move || super::pty::kill_child(&closer_child));
        reaper.join().expect("reaper joins");
        closer.join().expect("closer joins");
        let _ = done_tx.send(());
    });

    done_rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("channel-death teardown and a concurrent close must not deadlock");
    coordinator.join().expect("coordinator joins");

    assert!(
        child.lock().expect("child slot").is_none(),
        "the child must be reaped exactly once and left removed"
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
        let _guard = registry.sessions.lock().expect("hold to poison");
        panic!("poison the registry mutex");
    }));
    assert!(poisoned.is_err(), "the panic must poison the lock");
    assert!(
        registry.sessions.lock().is_err(),
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
