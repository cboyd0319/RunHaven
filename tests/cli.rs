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
