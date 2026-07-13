//! No-persist / no-log source tripwire for the terminal module (terminal design
//! section 5). This is a source-invariant test, kept separate from the behavior
//! tests in `terminal::tests`: it scans the production text of `mod.rs`,
//! `pty.rs`, `registry.rs`, and the process-local `session_output.rs` sink and
//! fails the build if any of them ever gains a
//! file or logging sink for the live stream. Interactive runs may also append
//! to the bounded, process-local output registry used by History; attach and
//! sign-in streams stay Channel-only. `registry.rs` is scanned
//! explicitly so registry code split out of `mod.rs` cannot escape the guard.

/// Forbidden symbols, built at runtime so this test file's own source cannot
/// satisfy the search (mirroring the menu dispatch-only tripwire). The
/// invariant: the terminal production module never persists terminal bytes to
/// a file or log. A failed worktree launch may persist non-stream
/// recovery metadata through the launch module.
fn forbidden_terminal_markers() -> Vec<String> {
    let mut markers = vec![
        // Persistence sinks.
        format!("{}{}", "write_active_", "run"),
        format!("{}{}", "runs_", "log_path"),
        format!("{}{}", "File::", "create"),
        format!("{}{}", "File::", "options"),
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
fn terminal_module_never_persists_the_stream_to_files_or_logs() {
    let forbidden = forbidden_terminal_markers();
    let mod_production = terminal_production(include_str!("mod.rs"));
    let pty_production = terminal_production(include_str!("pty.rs"));
    // `registry.rs` has no `mod tests;`, so the whole file is production text.
    let registry_production = terminal_production(include_str!("registry.rs"));
    let output_production = terminal_production(include_str!("../session_output.rs"));

    // Proof the scan reaches the command layer in `mod.rs` (below the first
    // `#[cfg(test)]` attribute, the old truncation point) and the registry
    // teardown now living in `registry.rs`.
    assert!(
        mod_production.contains("fn register_and_stream"),
        "the scan must cover the command layer below the first cfg(test) item"
    );
    assert!(
        registry_production.contains("fn close_all"),
        "the scan must cover the registry teardown in registry.rs"
    );
    assert!(
        output_production.contains("impl SessionOutputRegistry"),
        "the scan must cover the process-local History sink"
    );
    // Proof the run terminal (task #23) reuses the scanned reader: its spawn lives
    // in pty.rs under this same no-persist scan, so the interactive-run stream
    // cannot gain a storage or logging sink either.
    assert!(
        pty_production.contains("fn spawn_run_terminal"),
        "the scan must cover the interactive run terminal spawn in pty.rs"
    );
    assert!(
        mod_production.contains("RetainedRunOutput::new(run_id"),
        "the run command must bind History output to the Rust-owned plan run id"
    );
    assert!(
        pty_production.contains("Ok(spawned_run_from_session("),
        "the production run spawn must use the mandatory retained-output wrapper"
    );

    for production in [
        mod_production,
        pty_production,
        registry_production,
        output_production,
    ] {
        assert!(
            first_forbidden_marker(production, &forbidden).is_none(),
            "the terminal module must never persist terminal bytes to a file or log; \
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
    let with_file_options =
        "fn open_terminal_session() { File::options().write(true).open(path); }";
    assert!(
        first_forbidden_marker(with_file_options, &forbidden).is_some(),
        "a writable File::options sink must trip the tripwire"
    );
    // A logging sink slipped into the reader.
    let with_log = "fn stream_pty_output() { tracing::info!(\"chunk\"); }";
    assert!(
        first_forbidden_marker(with_log, &forbidden).is_some(),
        "a logging symbol must trip the tripwire"
    );
}

#[test]
fn interactive_run_output_is_retained_only_in_the_bounded_process_registry() {
    let mod_production = terminal_production(include_str!("mod.rs"));
    let pty_production = terminal_production(include_str!("pty.rs"));

    assert!(mod_production.contains("SessionOutputRegistry"));
    assert!(pty_production.contains("self.registry.append"));
    for forbidden in ["File::create", "OpenOptions", "fs::write", "log::info"] {
        assert!(!pty_production.contains(forbidden));
    }
}
