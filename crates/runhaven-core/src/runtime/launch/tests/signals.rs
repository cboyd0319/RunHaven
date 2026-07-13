use super::*;
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;

#[test]
fn status_runner_survives_first_sigint_until_its_child_exits() {
    let temp = tempfile::tempdir().expect("temp");
    let ready = temp.path().join("ready");
    let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"));
    child
        .args([
            "--ignored",
            "--exact",
            "runtime::launch::tests::signals::status_runner_sigint_fixture",
        ])
        .env("RUNHAVEN_SIGNAL_TEST_READY", &ready)
        .process_group(0);
    let mut child = child.spawn().expect("fixture process");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !ready.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if !ready.exists() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("signal fixture did not become ready");
    }

    let signal_status = std::process::Command::new("/bin/kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("send SIGINT");
    assert!(signal_status.success());

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().expect("fixture status") {
            assert!(status.success(), "fixture failed with {status}");
            break;
        }
        if std::time::Instant::now() >= deadline {
            let process_group = format!("-{}", child.id());
            let _ = std::process::Command::new("/bin/kill")
                .args(["-KILL", &process_group])
                .status();
            let _ = child.wait();
            panic!("signal fixture did not exit after SIGINT");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn second_sigint_uses_the_default_handler_during_stuck_cleanup() {
    let temp = tempfile::tempdir().expect("temp");
    let child_ready = temp.path().join("child-ready");
    let cleanup_ready = temp.path().join("cleanup-ready");
    let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"));
    child
        .args([
            "--ignored",
            "--exact",
            "runtime::launch::tests::signals::status_runner_second_sigint_fixture",
        ])
        .env("RUNHAVEN_SIGNAL_TEST_READY", &child_ready)
        .env("RUNHAVEN_SIGNAL_CLEANUP_READY", &cleanup_ready)
        .process_group(0);
    let mut child = child.spawn().expect("fixture process");

    wait_for_fixture_path(&mut child, &child_ready, "child ready");
    send_sigint(child.id());
    wait_for_fixture_path(&mut child, &cleanup_ready, "cleanup ready");
    send_sigint(child.id());

    let status = child.wait().expect("fixture status");
    assert_eq!(status.signal(), Some(signal_hook::consts::SIGINT));
}

fn wait_for_fixture_path(child: &mut std::process::Child, path: &std::path::Path, label: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !path.exists() && std::time::Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("fixture status") {
            panic!("fixture exited before {label}: {status}");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if !path.exists() {
        let process_group = format!("-{}", child.id());
        let _ = std::process::Command::new("/bin/kill")
            .args(["-KILL", &process_group])
            .status();
        let _ = child.wait();
        panic!("fixture did not reach {label}");
    }
}

fn send_sigint(pid: u32) {
    let status = std::process::Command::new("/bin/kill")
        .args(["-INT", &pid.to_string()])
        .status()
        .expect("send SIGINT");
    assert!(status.success());
}

#[test]
#[ignore = "subprocess fixture for the SIGINT lifecycle test"]
fn status_runner_sigint_fixture() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = test_plan(workspace.path());
    let ready = std::env::var("RUNHAVEN_SIGNAL_TEST_READY").expect("ready path");
    plan.command = [
        "/bin/sh",
        "-c",
        "touch \"$RUNHAVEN_SIGNAL_TEST_READY\"; sleep 30",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    let mut runner = StatusAgentCommandRunner::new().expect("signal-aware runner");
    let cleanup_calls = std::cell::Cell::new(0);

    let code = run_standard_launch_with_ops(
        &plan,
        &mut runner,
        |_| Ok(()),
        |_| {
            cleanup_calls.set(cleanup_calls.get() + 1);
            Ok(json!({
                "provider_network": "not-created",
                "run_container": "removed",
            }))
        },
    )
    .expect("interrupted run result");

    assert!(!ready.is_empty());
    assert_eq!(code, 130);
    assert_eq!(cleanup_calls.get(), 1);
    assert!(crate::runtime::active::find_active_run_record("runner-seam").is_err());
    let record = crate::records::find_run_record("runner-seam").expect("run record");
    assert_eq!(
        record
            .get("return_code")
            .and_then(serde_json::Value::as_i64),
        Some(130)
    );
    assert_eq!(record["cleanup"]["provider_network"], "not-created");
    assert_eq!(record["cleanup"]["run_container"], "removed");
}

#[test]
#[ignore = "subprocess fixture for the second SIGINT cleanup test"]
fn status_runner_second_sigint_fixture() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = test_plan(workspace.path());
    std::env::var("RUNHAVEN_SIGNAL_TEST_READY").expect("child ready path");
    let cleanup_ready = std::env::var("RUNHAVEN_SIGNAL_CLEANUP_READY").expect("cleanup ready path");
    plan.command = [
        "/bin/sh",
        "-c",
        "touch \"$RUNHAVEN_SIGNAL_TEST_READY\"; sleep 30",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let mut runner = StatusAgentCommandRunner::new().expect("signal-aware runner");

    let _ = run_standard_launch_with_ops(
        &plan,
        &mut runner,
        |_| Ok(()),
        |_| {
            std::fs::write(&cleanup_ready, b"ready").expect("mark cleanup ready");
            std::thread::sleep(std::time::Duration::from_secs(30));
            Ok(json!({"provider_network": "not-created"}))
        },
    );

    panic!("second SIGINT did not terminate blocked cleanup");
}
