//! Trust-boundary guard for the development-only logging capability. Release
//! windows never receive it, so raw terminal access and arbitrary persistent
//! log writes cannot coexist in a shipped WebView.

use super::support::{assert_same_permission_set, capability_permissions, capability_windows};

#[test]
fn logging_capability_stays_write_only_and_main_only() {
    // Dev/test log forwarding: the main window may append log lines to the
    // local app log file through the log plugin's single `log` command.
    // Write-only telemetry with no read, list, or delete surface; nothing
    // else may ride along. Main window only: the read-only monitor windows
    // never gain even this. The live terminal stream never passes through
    // here; PTY bytes flow over Channels, and the console forwarding layer
    // (ui/src/main.tsx) wraps console methods, which xterm does not use for
    // its stream.
    let permissions = capability_permissions("logging.json");
    assert_same_permission_set("logging", &permissions, &["log:allow-log"]);
    assert_eq!(
        capability_windows("logging.json"),
        vec!["main".to_string()],
        "logging must stay main-window-only"
    );
}

#[test]
fn release_config_never_grants_webview_logging() {
    let release = std::fs::read_to_string(super::support::manifest_dir().join("tauri.conf.json"))
        .expect("release Tauri config");
    assert!(
        !release.contains("\"logging\""),
        "release WebViews must not combine terminal access with arbitrary log persistence"
    );
}
