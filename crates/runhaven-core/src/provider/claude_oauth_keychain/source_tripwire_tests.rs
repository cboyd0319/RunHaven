//! No-persist / no-log source tripwire for the Claude OAuth Keychain module
//! (design Section 2: "the token is never written to a log, the run-output
//! registry, session records, diagnostics, or any file except the Keychain").
//!
//! This is a source-invariant test: it scans the PRODUCTION text of
//! `claude_oauth_keychain.rs` and fails the build if that module ever gains a
//! file, log, or run-record sink. The only permitted persistence is the
//! Keychain API and the only permitted subprocess is `claude setup-token`;
//! neither appears below. The forbidden strings are assembled at runtime so
//! this test file's own source cannot satisfy the search.

/// The token must reach the Keychain and nowhere else. These are the sinks that
/// would leak it to disk, a log, an emitted event, or a run record.
fn forbidden_sinks() -> Vec<String> {
    let mut markers = vec![
        // Filesystem sinks.
        format!("{}{}", "fs::", "write"),
        format!("{}{}", "fs::", "File"),
        format!("{}{}", "File::", "create"),
        format!("{}{}", "Open", "Options"),
        format!("{}{}", "write_", "all"),
        // Logging / event-emit sinks.
        format!("{}{}", "trac", "ing"),
        format!("{}{}", "eprint", "ln!"),
        format!("{}{}", "print", "ln!"),
        format!("{}{}", ".em", "it("),
        // Run-record / registry / marker persistence sinks (design F1).
        format!("{}{}", "SessionOutput", "Registry"),
        format!("{}{}", "run_", "history"),
        format!("{}{}", "write_active_", "run"),
        format!("{}{}", "markers", "::"),
    ];
    // `log` crate macro paths, enumerated so a marker cannot collide with an
    // unrelated substring.
    for level in ["error", "warn", "info", "debug", "trace"] {
        markers.push(format!("{}{}{}", "log", "::", level));
    }
    markers
}

/// Production text only: strip everything from the first `#[cfg(test)]`
/// attribute onward, which removes both the `tests` module and this tripwire's
/// own `mod source_tripwire_tests;` declaration.
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
fn keychain_module_never_persists_or_logs_the_token() {
    let forbidden = forbidden_sinks();
    let source = production(include_str!("../claude_oauth_keychain.rs"));

    // Prove the scan reaches the capture and store paths, where a leak would
    // most plausibly be introduced.
    assert!(
        source.contains("fn capture_and_store_setup_token"),
        "the scan must cover the capture path"
    );
    assert!(
        source.contains("fn store_token"),
        "the scan must cover the store path"
    );

    assert!(
        first_forbidden(source, &forbidden).is_none(),
        "the Claude OAuth Keychain module must persist the token only to the Keychain; \
         found forbidden sink `{}`",
        first_forbidden(source, &forbidden).unwrap_or_default()
    );
}

#[test]
fn presence_query_reads_attributes_and_never_the_secret_data() {
    // The presence path (`fn present`) must query attributes only. Loading the
    // secret data is the one thing that invokes LocalAuthentication and returns
    // the token, so the whole module must never request it. This proves, at the
    // source level, that checking presence cannot prompt and cannot read the
    // token (design Section 4: "checking presence must NOT trigger the prompt").
    let source = production(include_str!("../claude_oauth_keychain.rs"));
    assert!(
        source.contains("fn present"),
        "the scan must cover the presence path"
    );
    assert!(
        source.contains("load_attributes(true)"),
        "the presence query must request attributes so `search()` returns a result"
    );
    // Assembled at runtime so this test's own source cannot satisfy the search.
    let data_return = format!("{}{}", "load_", "data");
    assert!(
        !source.contains(&data_return),
        "no query in this module may request the secret data (kSecReturnData); \
         that is what prompts and reads the token"
    );
}

#[test]
fn tripwire_fires_on_a_file_or_log_sink() {
    let forbidden = forbidden_sinks();
    let with_file = "fn capture() { let mut f = fs::File::create(p)?; f.write_all(token)?; }";
    assert!(
        first_forbidden(with_file, &forbidden).is_some(),
        "a filesystem sink must trip the tripwire"
    );
    let with_log = "fn capture() { tracing::info!(\"{token}\"); }";
    assert!(
        first_forbidden(with_log, &forbidden).is_some(),
        "a logging sink must trip the tripwire"
    );
}
