use std::process::Command;

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
fn plan_ssh_fails_closed_until_runtime_boundary_is_verified() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let output = Command::new(env!("CARGO_BIN_EXE_runhaven"))
        .args(["plan", "shell", "--workspace"])
        .arg(workspace.path())
        .args(["--ssh"])
        .output()
        .expect("run runhaven");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("SSH forwarding is disabled"));
    assert!(stderr.contains("Apple container 1.0.0"));
    assert!(stderr.contains("raw SSH keys"));
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
