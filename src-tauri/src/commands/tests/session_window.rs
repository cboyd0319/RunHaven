//! Validation tests for the monitor window open path. Window creation itself
//! needs a live Tauri app (covered by the shared `tauri dev` smoke); these
//! tests pin the Rust-side gate every open request passes first, and the
//! label-keyed target lookup a monitor window uses to learn its session.

use super::*;
use crate::commands::session_window::{monitor_session_for_test, session_window_target_for_test};
use crate::session_windows::{SessionWindowRegistry, SessionWindowSlot};

#[test]
fn open_rejects_malformed_session_id_shapes_before_any_lookup() {
    let (_cache, _cache_home) = isolated_cache();
    for bad in [
        "",
        "-leading-dash",
        "has whitespace",
        "path/traversal",
        "back\\slash",
        "control\u{1b}chars",
        "tab\tseparated",
    ] {
        let error = monitor_session_for_test(bad).expect_err("malformed id must be rejected");
        assert!(
            error.contains("invalid run id") || error.contains("required"),
            "unexpected error for {bad:?}: {error}"
        );
    }
}

#[test]
fn open_rejects_session_ids_the_backend_does_not_know() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run("session-window-known");

    let missing = monitor_session_for_test("session-window-unknown")
        .expect_err("unknown session must be rejected");
    assert!(missing.contains("managed session not found"));
}

#[test]
fn open_resolves_backend_truth_and_the_title_carries_no_paths() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "session-window-open",
        json!({
            "timestamp": "2026-07-01T02:00:00Z",
            "run_id": "session-window-open",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/deep/project-x",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-open",
            "state_volume": "runhaven-codex-open-home",
            "session": "open"
        }),
    )
    .expect("active run payload");

    let session = monitor_session_for_test("session-window-open").expect("known session");
    assert_eq!(session.id, "session-window-open");
    // The session title becomes the monitor window title: project label only,
    // never the workspace path.
    assert!(session.title.contains("project-x"));
    assert!(!session.title.contains("/sensitive"));
    assert!(!session.title.contains("/workspaces"));
}

#[test]
fn target_lookup_answers_only_registered_monitor_labels() {
    let registry = SessionWindowRegistry::default();
    let label = match registry.reserve("run-target", |_| true) {
        Ok(SessionWindowSlot::Create(label)) => label,
        other => panic!("expected fresh label, got {other:?}"),
    };

    let target = session_window_target_for_test(&registry, &label).expect("registered label");
    assert_eq!(target.session_id, "run-target");

    // Every other label fails closed, including the main window's own.
    assert!(session_window_target_for_test(&registry, "main").is_err());
    assert!(session_window_target_for_test(&registry, "session-99").is_err());
}
