//! Trust-boundary guard for the dev/test logging capability: the WebView's
//! only log grant stays the log plugin's single write-only `log` command, on
//! the main window only.

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
