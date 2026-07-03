//! No-persist / no-log source tripwire for the Claude OAuth capture module
//! (design Section 2: the token is never written to a log, the run-output
//! registry, session records, diagnostics, or any file except the Keychain).
//!
//! This is a source-invariant test: it scans the PRODUCTION text of
//! `claude_oauth_capture.rs` and fails the build if the capture ever gains a
//! file, log, or run-record sink. The only permitted persistence is the Keychain
//! store (in the sibling custody module) and the only permitted subprocess is
//! `claude setup-token`. It also pins the F8 no-TOCTOU property at the source
//! level: exactly one PATH lookup, in the resolver, never in the capture. The
//! forbidden strings are assembled at runtime so this file's own source cannot
//! satisfy the search.

/// The token must reach the Keychain and nowhere else. These are the sinks that
/// would leak it to disk, a log, an emitted event, or a run record.
fn forbidden_sinks() -> Vec<String> {
    let mut markers = vec![
        format!("{}{}", "fs::", "write"),
        format!("{}{}", "fs::", "File"),
        format!("{}{}", "File::", "create"),
        format!("{}{}", "Open", "Options"),
        format!("{}{}", "write_", "all"),
        format!("{}{}", "trac", "ing"),
        format!("{}{}", "eprint", "ln!"),
        format!("{}{}", "print", "ln!"),
        format!("{}{}", ".em", "it("),
        format!("{}{}", "SessionOutput", "Registry"),
        format!("{}{}", "run_", "history"),
        format!("{}{}", "write_active_", "run"),
        format!("{}{}", "markers", "::"),
    ];
    for level in ["error", "warn", "info", "debug", "trace"] {
        markers.push(format!("{}{}{}", "log", "::", level));
    }
    markers
}

/// Production text only: strip everything from the first `#[cfg(test)]` attribute
/// onward, which removes the `tests` module and this tripwire's own declaration.
fn production(source: &str) -> &str {
    let cfg_test = format!("#[{}]", "cfg(test)");
    match source.split_once(&cfg_test) {
        Some((before, _)) => before,
        None => source,
    }
}

fn first_forbidden(production: &str, forbidden: &[String]) -> Option<String> {
    forbidden
        .iter()
        .find(|marker| production.contains(marker.as_str()))
        .cloned()
}

#[test]
fn capture_module_never_persists_or_logs_the_token() {
    let forbidden = forbidden_sinks();
    let source = production(include_str!("../claude_oauth_capture.rs"));

    // Prove the scan reaches the capture and the PTY reader, where a leak would
    // most plausibly be introduced.
    assert!(
        source.contains("fn capture_and_store_setup_token"),
        "the scan must cover the capture path"
    );
    assert!(
        source.contains("fn read_until_token"),
        "the scan must cover the PTY reader path"
    );

    assert!(
        first_forbidden(source, &forbidden).is_none(),
        "the Claude OAuth capture module must persist the token only to the Keychain; \
         found forbidden sink `{}`",
        first_forbidden(source, &forbidden).unwrap_or_default()
    );
}

#[test]
fn capture_runs_the_resolved_path_with_a_single_path_lookup() {
    // F8: the capture must run the exact resolved `claude` string, with no second
    // PATH lookup between the shown and the run binary. Assert the only
    // `find_on_path` call is the resolver, and the child is spawned from the
    // passed `claude` path with the exact `setup-token` argument.
    let source = production(include_str!("../claude_oauth_capture.rs"));
    let lookup = format!("{}{}", "find_on_path", "(");
    assert_eq!(
        source.matches(lookup.as_str()).count(),
        1,
        "exactly one PATH lookup, in the resolver only (no re-lookup in the capture)"
    );
    assert!(
        source.contains("Command::new(claude)"),
        "the capture must run the caller-resolved path, not a re-looked-up one"
    );
    assert!(
        source.contains(".arg(\"setup-token\")"),
        "the capture must pass the exact `setup-token` argument, never a shell string"
    );
}

#[test]
fn tripwire_fires_on_a_file_or_log_sink() {
    let forbidden = forbidden_sinks();
    let with_file = "fn read() { let mut f = fs::File::create(p)?; f.write_all(token)?; }";
    assert!(
        first_forbidden(with_file, &forbidden).is_some(),
        "a filesystem sink must trip the tripwire"
    );
    let with_log = "fn read() { tracing::info!(\"{token}\"); }";
    assert!(
        first_forbidden(with_log, &forbidden).is_some(),
        "a logging sink must trip the tripwire"
    );
}
