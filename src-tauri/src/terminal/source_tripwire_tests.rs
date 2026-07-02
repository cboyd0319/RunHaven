//! No-persist / no-log source tripwire for the terminal module (terminal design
//! section 5). This is a source-invariant test, kept separate from the behavior
//! tests in `terminal::tests`: it scans the production text of `mod.rs` and
//! `pty.rs` and fails the build if either ever gains a persistence or logging
//! sink for the live stream. The terminal production module must relay to the
//! Tauri `Channel` only.

/// Forbidden symbols, built at runtime so this test file's own source cannot
/// satisfy the search (mirroring the menu dispatch-only tripwire). The
/// invariant: the terminal production module never references the captured-output
/// registry, writes run records, touches any file, or logs/emits the stream.
fn forbidden_terminal_markers() -> Vec<String> {
    let mut markers = vec![
        // Persistence sinks.
        format!("{}{}", "SessionOutput", "Registry"),
        format!("{}{}", "session_", "output"),
        format!("{}{}", "write_active_", "run"),
        format!("{}{}", "runs_", "log_path"),
        format!("{}{}", "File::", "create"),
        format!("{}{}", "Open", "Options"),
        format!("{}{}", "fs::", "write"),
        // Logging / event-emit sinks.
        format!("{}{}", "trac", "ing"),
        format!("{}{}", "eprint", "ln!"),
        format!("{}{}", "print", "ln!"),
        format!("{}{}", ".em", "it("),
    ];
    // `log` crate macro paths, enumerated so the marker cannot collide with an
    // unrelated path like `dialog::` (a bare `log::` substring would).
    for level in ["error", "warn", "info", "debug", "trace"] {
        markers.push(format!("{}{}{}", "log", "::", level));
    }
    markers
}

/// Strip the trailing `#[cfg(test)]`-annotated `mod tests;` inclusion so the
/// scan covers every production line. The old code truncated at the *first*
/// `#[cfg(test)]` attribute, which in `mod.rs` sits on `without_reader` near the
/// top, so the whole command/registry layer below it went unscanned. `pty.rs`
/// has no test module and is scanned in full.
fn terminal_production(source: &str) -> &str {
    match source.split_once("\nmod tests;") {
        Some((before, _)) => before,
        None => source,
    }
}

fn first_forbidden_marker(production: &str, forbidden: &[String]) -> Option<String> {
    forbidden
        .iter()
        .find(|marker| production.contains(marker.as_str()))
        .cloned()
}

#[test]
fn terminal_module_never_persists_or_logs_the_stream() {
    let forbidden = forbidden_terminal_markers();
    let mod_production = terminal_production(include_str!("mod.rs"));
    let pty_production = terminal_production(include_str!("pty.rs"));

    // Proof the scan now reaches the command/registry layer, not just the head
    // above the first `#[cfg(test)]` attribute (the old truncation point).
    assert!(
        mod_production.contains("fn register_and_stream"),
        "the scan must cover the command layer below the first cfg(test) item"
    );
    assert!(
        mod_production.contains("fn close_all"),
        "the scan must cover the registry teardown"
    );

    for production in [mod_production, pty_production] {
        assert!(
            first_forbidden_marker(production, &forbidden).is_none(),
            "the terminal module must relay to the Channel only and never persist or log the \
             stream; found forbidden marker `{}`",
            first_forbidden_marker(production, &forbidden).unwrap_or_default()
        );
    }
}

/// Negative check: the tripwire must FAIL if a storage or logging symbol is
/// added anywhere in production (including below the old truncation point).
#[test]
fn no_persist_tripwire_fires_on_a_storage_or_logging_symbol() {
    let forbidden = forbidden_terminal_markers();
    // A storage sink slipped into the command layer.
    let with_storage = "fn open_terminal_session() { fs::write(path, stream); }";
    assert!(
        first_forbidden_marker(with_storage, &forbidden).is_some(),
        "a storage symbol must trip the tripwire"
    );
    // A logging sink slipped into the reader.
    let with_log = "fn stream_pty_output() { tracing::info!(\"chunk\"); }";
    assert!(
        first_forbidden_marker(with_log, &forbidden).is_some(),
        "a logging symbol must trip the tripwire"
    );
}
