use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn bare_non_tty_prints_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .output()
        .expect("run runhaven");

    assert_eq!(
        output.status.code(),
        Some(2),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: runhaven"));
    assert!(stdout.contains("Commands:"));
}

#[cfg(unix)]
#[test]
fn support_creates_one_private_report_and_never_uploads_it() {
    let home = tempfile::tempdir().expect("home");
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .arg("support")
        .env("HOME", home.path())
        .env_remove("RUNHAVEN_CACHE_HOME")
        .output()
        .expect("run support command");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Support report created:"));
    assert!(stdout.contains("Nothing was uploaded or submitted."));
    assert!(stdout.contains(
        "https://github.com/cboyd0319/RunHaven/issues/new?template=runhaven-support.yml"
    ));
    assert!(!stdout.contains("gh issue"));

    let reports = home
        .path()
        .join("Library")
        .join("Application Support")
        .join("RunHaven")
        .join("Support Reports");
    let files = std::fs::read_dir(&reports)
        .expect("support reports directory")
        .map(|entry| entry.expect("report entry").path())
        .collect::<Vec<_>>();
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].extension().and_then(|value| value.to_str()),
        Some("txt")
    );
    assert_eq!(
        std::fs::metadata(&files[0])
            .expect("report metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600,
    );
}

#[test]
fn plan_shell_dry_run_prints_container_boundary() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .args(["plan", "shell", "--workspace"])
        .arg(workspace.path())
        .args(["--", "/bin/bash", "-lc", "pwd"])
        .output()
        .expect("run runhaven");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Workspace:"));
    assert!(stdout.contains("State volume:"));
    assert!(stdout.contains("Egress: unrestricted internet"));
    assert!(stdout.contains("container run"));
    assert!(stdout.contains("/bin/bash -lc pwd"));
}

#[test]
fn run_help_explains_agent_argument_separator() {
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .args(["run", "--help"])
        .output()
        .expect("run runhaven help");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Use -- before flags meant for the agent"));
    assert!(stdout.contains("--workspace-scope"));
    assert!(stdout.contains("runtime allowlist proxy"));
}

#[test]
fn attach_rejects_an_unknown_tty_mode_before_runtime_lookup() {
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .args(["runs", "attach", "missing-run", "--tty", "sometimes"])
        .output()
        .expect("run runhaven attach");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value 'sometimes'"));
    assert!(stderr.contains("auto"));
    assert!(stderr.contains("always"));
    assert!(stderr.contains("never"));
}

#[test]
fn plan_ssh_forwards_the_agent_socket_and_warns() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .args(["plan", "shell", "--workspace"])
        .arg(workspace.path())
        .args(["--ssh"])
        .output()
        .expect("run runhaven");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // The plan forwards the SSH agent socket and warns about the exposure.
    assert!(combined.contains("--ssh"), "plan should forward --ssh");
    assert!(
        combined.contains("SSH agent forwarding"),
        "plan should warn about SSH agent forwarding"
    );
}

#[test]
fn why_network_provider_explains_allowlist_proxy() {
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .args(["why", "network", "provider"])
        .output()
        .expect("run runhaven why network");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Network mode: provider"));
    assert!(stdout.contains("allowlist proxy"));
    assert!(stdout.contains("DNS safety"));
}

#[test]
fn why_workspace_explains_mount_decision() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .args(["why", "workspace"])
        .arg(workspace.path())
        .output()
        .expect("run runhaven why workspace");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Workspace scope: current"));
    assert!(stdout.contains("Mount decision: allowed"));
    assert!(stdout.contains("agent home stays in a RunHaven state volume"));
}

#[test]
fn why_state_explains_volume_isolation() {
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .args(["why", "state", "shell"])
        .output()
        .expect("run runhaven why state");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Profile: shell"));
    assert!(stdout.contains("State volume pattern: runhaven-shell-<project-id>-home"));
    assert!(stdout.contains("workspace files are mounted separately"));
}
