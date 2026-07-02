//! Shared constants and helpers for the capability-scope guard tests.

use std::fs;
use std::path::{Path, PathBuf};

/// Plugin and core permissions intentionally granted to the frontend. Adding
/// an entry here is a deliberate trust-boundary decision and must be reviewed
/// against the security model. The two event permissions let the main window
/// receive menu ids from Rust (listen/unlisten only, never emit).
pub(super) const VETTED_PLUGIN_PERMISSIONS: &[&str] = &[
    "dialog:allow-open",
    "core:event:allow-listen",
    "core:event:allow-unlisten",
];
pub(super) const FORBIDDEN_HOST_MARKERS: &[&str] = &[
    "app_server",
    "filesystem_rpc",
    "filesystem",
    "mcp",
    "shell",
    "exec_server",
    "command_exec",
    "execute_command",
    "attach_session",
    "resize_session",
    "codex",
    "login",
    "logout",
    "set_auth",
    "update_auth",
    "write_auth",
    "clear_auth",
    "delete_auth",
];
pub(super) const MAIN_READ_FORBIDDEN_PERMISSIONS: &[&str] = &[
    "allow-get-log-snapshot",
    "allow-get-managed-session-log-snapshot",
    "allow-open-session-window",
    "allow-launch-run",
    "allow-write-session-input",
    "allow-stop-run",
    "allow-kill-run",
    "allow-repair-run",
    "allow-stop-managed-session",
    "allow-kill-managed-session",
    "allow-repair-managed-session",
    "allow-cleanup-managed-session",
    "allow-keep-managed-session-worktree",
    "allow-merge-managed-session-worktree",
    "allow-recover-managed-session-worktree",
    "allow-discard-managed-session-worktree",
    "allow-rebuild-agent-image",
    "allow-prune-managed-networks",
    "allow-start-agent-sign-in",
    "allow-set-attention-badge",
    "allow-sync-menu-state",
    "allow-open-terminal-session",
    "allow-write-terminal-input",
    "allow-resize-terminal",
    "allow-close-terminal-session",
];
pub(super) const RUN_CONTROL_PERMISSIONS: &[(&str, &str)] = &[
    ("allow-get-log-snapshot", "get_log_snapshot"),
    (
        "allow-get-managed-session-log-snapshot",
        "get_managed_session_log_snapshot",
    ),
    ("allow-write-session-input", "write_session_input"),
    ("allow-stop-run", "stop_run"),
    ("allow-kill-run", "kill_run"),
    ("allow-repair-run", "repair_run"),
    ("allow-stop-managed-session", "stop_managed_session"),
    ("allow-kill-managed-session", "kill_managed_session"),
    ("allow-repair-managed-session", "repair_managed_session"),
    ("allow-cleanup-managed-session", "cleanup_managed_session"),
    (
        "allow-keep-managed-session-worktree",
        "keep_managed_session_worktree",
    ),
    (
        "allow-merge-managed-session-worktree",
        "merge_managed_session_worktree",
    ),
    (
        "allow-recover-managed-session-worktree",
        "recover_managed_session_worktree",
    ),
    (
        "allow-discard-managed-session-worktree",
        "discard_managed_session_worktree",
    ),
    ("allow-rebuild-agent-image", "rebuild_agent_image"),
    ("allow-prune-managed-networks", "prune_managed_networks"),
];

/// The exact grant for the terminal capability (terminal design section 7). A
/// live PTY into a sandbox is the app's most sensitive interactive surface, so
/// it lives in its own main-window-only capability and never widens main-read
/// or the read-only monitor windows.
pub(super) const TERMINAL_PERMISSIONS: &[(&str, &str)] = &[
    ("allow-open-terminal-session", "open_terminal_session"),
    ("allow-write-terminal-input", "write_terminal_input"),
    ("allow-resize-terminal", "resize_terminal"),
    ("allow-close-terminal-session", "close_terminal_session"),
];

pub(super) const ATTENTION_BADGE_PERMISSION: &str = "allow-set-attention-badge";

pub(super) const MENU_STATE_PERMISSION: &str = "allow-sync-menu-state";

/// The exact read-only grant for per-session monitor windows. Widening this
/// set is a trust-boundary change: monitor windows must never gain output
/// snapshots, input, run control, launch, sign-in, events, or window control.
pub(super) const SESSION_MONITOR_PERMISSIONS: &[(&str, &str)] = &[
    (
        "allow-get-session-window-target",
        "get_session_window_target",
    ),
    (
        "allow-get-managed-session-status",
        "get_managed_session_status",
    ),
    (
        "allow-get-managed-session-diagnostics",
        "get_managed_session_diagnostics",
    ),
];

pub(super) fn capability_windows(file_name: &str) -> Vec<String> {
    let path = manifest_dir().join("capabilities").join(file_name);
    let raw = read_text(&path);
    let value: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()));
    value["windows"]
        .as_array()
        .unwrap_or_else(|| panic!("capability {} has no windows array", path.display()))
        .iter()
        .map(|window| {
            window.as_str().unwrap_or_else(|| {
                panic!(
                    "capability {} uses a non-string windows entry `{window}`",
                    path.display()
                )
            })
        })
        .map(str::to_owned)
        .collect()
}

/// The `identifier` of every capability file under `capabilities/`.
pub(super) fn capability_file_identifiers() -> Vec<String> {
    let dir = manifest_dir().join("capabilities");
    let mut identifiers = Vec::new();
    for entry in fs::read_dir(&dir).expect("read capabilities directory") {
        let path = entry.expect("capabilities directory entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(&read_text(&path))
            .unwrap_or_else(|_| panic!("parse {}", path.display()));
        let identifier = value["identifier"]
            .as_str()
            .unwrap_or_else(|| panic!("capability {} has no string identifier", path.display()));
        identifiers.push(identifier.to_owned());
    }
    identifiers
}

pub(super) fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn normalize(raw: &str) -> String {
    raw.to_ascii_lowercase().replace('-', "_")
}

pub(super) fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()))
}

pub(super) fn assert_lacks_forbidden_host_markers(label: &str, raw: &str) {
    let normalized = normalize(raw);
    for marker in FORBIDDEN_HOST_MARKERS {
        assert!(
            !normalized.contains(marker),
            "{label} exposes forbidden host-reaching marker `{marker}` without an explicit reviewed capability"
        );
    }
}

pub(super) fn capability_permissions(file_name: &str) -> Vec<String> {
    let path = manifest_dir().join("capabilities").join(file_name);
    let raw = read_text(&path);
    let value: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()));
    value["permissions"]
        .as_array()
        .unwrap_or_else(|| panic!("capability {} has no permissions array", path.display()))
        .iter()
        .map(|permission| {
            permission.as_str().unwrap_or_else(|| {
                panic!(
                    "capability {} uses a non-string permission entry `{permission}`",
                    path.display()
                )
            })
        })
        .map(str::to_owned)
        .collect()
}

pub(super) fn assert_same_permission_set(label: &str, actual: &[String], expected: &[&str]) {
    let mut actual_sorted = actual.to_owned();
    let mut expected_sorted = expected
        .iter()
        .map(|permission| (*permission).to_owned())
        .collect::<Vec<_>>();
    actual_sorted.sort();
    expected_sorted.sort();
    assert_eq!(
        actual_sorted, expected_sorted,
        "{label} capability permission set changed; review GUI trust boundary before widening it"
    );
}

pub(super) fn assert_generated_permission_allows_only(command: &str) {
    let path = manifest_dir()
        .join("permissions")
        .join("autogenerated")
        .join(format!("{command}.toml"));
    let raw = read_text(&path);
    assert!(
        raw.contains(&format!("commands.allow = [\"{command}\"]")),
        "{} permission must allow only the fixed `{command}` command",
        path.display()
    );
}

pub(super) fn generated_permission_files() -> Vec<PathBuf> {
    let dir = manifest_dir().join("permissions").join("autogenerated");
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).expect("read generated permissions directory") {
        let path = entry.expect("generated permission entry").path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            files.push(path);
        }
    }
    files
}

pub(super) fn invoke_handler_commands() -> Vec<String> {
    let lib_rs = manifest_dir().join("src").join("lib.rs");
    let raw = read_text(&lib_rs);
    let mut commands = Vec::new();
    let mut in_handler = false;

    for line in raw.lines() {
        if line.contains("tauri::generate_handler![") {
            in_handler = true;
            continue;
        }
        if !in_handler {
            continue;
        }
        if line.contains("])") {
            break;
        }
        let trimmed = line.trim().trim_end_matches(',');
        if trimmed.is_empty() {
            continue;
        }
        commands.push(
            trimmed
                .rsplit("::")
                .next()
                .expect("handler command name")
                .to_owned(),
        );
    }

    assert!(
        !commands.is_empty(),
        "no commands found in {} invoke handler",
        lib_rs.display()
    );
    commands
}
