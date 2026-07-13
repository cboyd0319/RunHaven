use super::*;

#[test]
fn pre_pty_errors_expose_only_the_fixed_state_conflict_message() {
    let conflict = anyhow::anyhow!(STATE_VOLUME_IN_USE_TERMINAL_ERROR);
    assert_eq!(
        terminal_open_error(&conflict),
        STATE_VOLUME_IN_USE_TERMINAL_ERROR
    );

    let private = anyhow::anyhow!("could not mount /Users/private/workspace");
    assert_eq!(terminal_open_error(&private), GENERIC_TERMINAL_OPEN_ERROR);
}

fn sleep_session() -> GuiPtySession {
    let (pty, pts) = pty_process::blocking::open().expect("open a host pty");
    let child = Command::new("/bin/sleep")
        .arg("30")
        .spawn(pts)
        .expect("spawn a stand-in child");
    GuiPtySession {
        pty: Arc::new(pty),
        child: Arc::new(Mutex::new(Some(child))),
        exit: Arc::new(ExitSignal::new()),
        cleanup: None,
    }
}

#[test]
fn wait_for_exit_unblocks_when_a_reader_reaps_the_child() {
    let session = sleep_session();
    let waiter = session.clone();
    let (code_tx, code_rx) = std::sync::mpsc::channel();
    let waiter_thread = std::thread::spawn(move || {
        let _ = code_tx.send(waiter.wait_for_exit());
    });
    assert!(
        code_rx.recv_timeout(Duration::from_millis(150)).is_err(),
        "wait_for_exit must block while the child is live"
    );
    let reaped = session.kill_and_reap();
    let observed = code_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("waiter releases after reap");
    assert_eq!(observed, reaped);
    waiter_thread.join().expect("waiter joins");
}

#[test]
fn attach_session_has_no_detached_launch_wait() {
    let session = sleep_session();
    assert_eq!(session.wait_for_launch_completion(), None);
    session.shutdown();
}

#[test]
fn channel_death_kill_and_a_concurrent_close_do_not_deadlock() {
    let session = sleep_session();
    let reaper = session.clone();
    let closer = session.clone();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let coordinator = std::thread::spawn(move || {
        let reaper = std::thread::spawn(move || reaper.kill_and_reap());
        let closer = std::thread::spawn(move || closer.shutdown());
        reaper.join().expect("reaper joins");
        closer.join().expect("closer joins");
        let _ = done_tx.send(());
    });
    done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("teardown must not deadlock");
    coordinator.join().expect("coordinator joins");
    assert!(session.child.lock().expect("child slot").is_none());
}

#[test]
fn losing_reaper_observes_the_latched_exit_code() {
    let (pty, pts) = pty_process::blocking::open().expect("open a host pty");
    let child = Command::new("/usr/bin/true")
        .spawn(pts)
        .expect("spawn an exited stand-in child");
    let session = GuiPtySession {
        pty: Arc::new(pty),
        child: Arc::new(Mutex::new(Some(child))),
        exit: Arc::new(ExitSignal::new()),
        cleanup: None,
    };
    let first = session.clone();
    let second = session.clone();
    let first = std::thread::spawn(move || first.reap_after_eof());
    let second = std::thread::spawn(move || second.reap_after_eof());
    assert_eq!(first.join().expect("first reaper"), 0);
    assert_eq!(second.join().expect("second reaper"), 0);
}

#[test]
fn shutdown_waits_for_detached_launch_cleanup() {
    let mut session = sleep_session();
    let cleanup = Arc::new(CleanupSignal::new());
    session.cleanup = Some(cleanup.clone());
    let guard = CleanupGuard::new(cleanup);
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let shutdown = std::thread::spawn(move || {
        session.shutdown();
        let _ = done_tx.send(());
    });
    assert!(
        done_rx.recv_timeout(Duration::from_millis(150)).is_err(),
        "shutdown must wait while detached cleanup is unfinished"
    );
    guard.finish(true);
    done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("shutdown releases after cleanup");
    shutdown.join().expect("shutdown joins");
}

#[test]
fn delayed_launch_completion_blocks_natural_eof_publication() {
    let (pty, pts) = pty_process::blocking::open().expect("open a host pty");
    let child = Command::new("/usr/bin/true")
        .spawn(pts)
        .expect("spawn an exited stand-in child");
    let cleanup = Arc::new(CleanupSignal::new());
    let guard = CleanupGuard::new(cleanup.clone());
    let session = GuiPtySession {
        pty: Arc::new(pty),
        child: Arc::new(Mutex::new(Some(child))),
        exit: Arc::new(ExitSignal::new()),
        cleanup: Some(cleanup),
    };
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let code = session.reap_after_eof();
        let outcome = session.wait_for_launch_completion();
        let _ = done_tx.send((code, outcome));
    });

    assert!(
        done_rx.recv_timeout(Duration::from_millis(150)).is_err(),
        "natural EOF must not publish completion before detached cleanup"
    );
    guard.finish(true);
    assert_eq!(
        done_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("reader releases after cleanup"),
        (0, Some(true))
    );
    reader.join().expect("reader joins");
}
