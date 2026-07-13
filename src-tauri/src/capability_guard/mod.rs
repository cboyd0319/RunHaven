//! Mechanical guard for Tauri capability scopes.
//!
//! The WebView is untrusted. Capabilities may only grant RunHaven's own typed
//! command permissions (`allow-*`) plus an explicitly vetted allowlist of
//! plugin permissions. These tests fail closed if a capability ever grants a
//! generic host bridge (shell, fs, http, process, os, ...), so widening the
//! frontend trust boundary cannot pass review silently.
//!
//! See `docs/harness/boundaries/component-inventory.md` and
//! `docs/harness/boundaries/security-boundary-map.md`.

mod command_scopes;
mod logging_scope;
mod release_packaging;
mod setup_recovery_scope;
mod support;
mod terminal_scope;

use std::fs;

use support::{
    VETTED_PLUGIN_PERMISSIONS, assert_lacks_forbidden_host_markers, capability_file_identifiers,
    generated_permission_files, invoke_handler_commands, manifest_dir, read_text,
};

/// Pin release and development configs to the capability files. Release omits
/// the one development-only WebView logging grant, so no production window can
/// both receive raw terminal bytes and persist arbitrary strings. Development
/// adds that reviewed write-only grant for the secret-scanned local log trail.
#[test]
fn tauri_configs_pin_release_and_development_capabilities() {
    const DEV_ONLY: &str = "logging";
    let manifest = manifest_dir();
    let path = manifest.join("tauri.conf.json");
    let dev_path = manifest.join("tauri.dev.conf.json");
    let mut file_identifiers = capability_file_identifiers();
    let mut release_identifiers = file_identifiers
        .iter()
        .filter(|identifier| identifier.as_str() != DEV_ONLY)
        .cloned()
        .collect::<Vec<_>>();
    let mut listed_identifiers = config_capabilities(&path);
    let mut dev_identifiers = config_capabilities(&dev_path);
    listed_identifiers.sort();
    release_identifiers.sort();
    dev_identifiers.sort();
    file_identifiers.sort();
    assert_eq!(
        listed_identifiers, release_identifiers,
        "the release config must omit only the development logging capability"
    );
    assert_eq!(
        dev_identifiers, file_identifiers,
        "the development override must reference every reviewed capability file"
    );
    assert!(
        !release_identifiers.is_empty(),
        "no release capability identifiers found"
    );
}

#[test]
fn development_bundle_has_an_unambiguous_identity() {
    let manifest = manifest_dir();
    let release: serde_json::Value =
        serde_json::from_str(&read_text(&manifest.join("tauri.conf.json")))
            .expect("parse release Tauri config");
    let development: serde_json::Value =
        serde_json::from_str(&read_text(&manifest.join("tauri.dev.conf.json")))
            .expect("parse development Tauri config");

    assert_eq!(release["productName"], "RunHaven");
    assert_eq!(release["identifier"], "dev.runhaven.desktop");
    assert_eq!(development["productName"], "RunHaven Debug");
    assert_eq!(development["identifier"], "dev.runhaven.desktop.debug");
    assert_ne!(release["identifier"], development["identifier"]);
}

fn config_capabilities(path: &std::path::Path) -> Vec<String> {
    let conf: serde_json::Value =
        serde_json::from_str(&read_text(path)).expect("parse Tauri config");
    let listed = conf
        .pointer("/app/security/capabilities")
        .and_then(serde_json::Value::as_array)
        .expect("Tauri config must pin app.security.capabilities as an explicit array");
    listed
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| {
                    panic!(
                        "Tauri config declares an inline capability `{entry}`; inline \
                         capabilities bypass the per-file scope guards, so every capability \
                         must be a file under capabilities/ referenced by its identifier"
                    )
                })
                .to_owned()
        })
        .collect()
}

#[test]
fn tauri_bundle_requires_the_supported_macos_version() {
    let path = manifest_dir().join("tauri.conf.json");
    let conf: serde_json::Value =
        serde_json::from_str(&read_text(&path)).expect("parse tauri.conf.json");
    assert_eq!(
        conf.pointer("/bundle/macOS/minimumSystemVersion")
            .and_then(serde_json::Value::as_str),
        Some("26.0"),
        "the desktop bundle must fail before launch on unsupported macOS versions"
    );
    assert_eq!(
        conf.get("version").and_then(serde_json::Value::as_str),
        Some(env!("CARGO_PKG_VERSION")),
        "the desktop bundle version must match the Rust release version"
    );
    assert_eq!(
        conf.pointer("/bundle/active")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "release packaging must not be disabled in tauri.conf.json"
    );
}

#[test]
fn capabilities_grant_only_vetted_scopes() {
    let dir = manifest_dir().join("capabilities");
    let mut checked = 0usize;

    for entry in fs::read_dir(&dir).expect("read capabilities directory") {
        let path = entry.expect("capabilities directory entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let raw = read_text(&path);
        assert_lacks_forbidden_host_markers(&format!("capability {}", path.display()), &raw);
        let value: serde_json::Value = serde_json::from_str(&raw).expect("parse capability json");
        let permissions = value["permissions"]
            .as_array()
            .unwrap_or_else(|| panic!("capability {} has no permissions array", path.display()));

        for permission in permissions {
            let name = match permission.as_str() {
                Some(name) => name,
                None => panic!(
                    "capability {} uses a non-string permission entry `{permission}`; \
                     review it as a trust-boundary change",
                    path.display()
                ),
            };
            let vetted = name.starts_with("allow-") || VETTED_PLUGIN_PERMISSIONS.contains(&name);
            assert!(
                vetted,
                "capability {} grants unvetted scope `{name}`. The WebView is untrusted: \
                 only RunHaven's own `allow-*` command permissions or an explicitly vetted \
                 plugin permission may be granted. If this is intentional, add it to \
                 VETTED_PLUGIN_PERMISSIONS after security review.",
                path.display()
            );
        }

        checked += 1;
    }

    assert!(
        checked > 0,
        "no capability files found under {}",
        dir.display()
    );
}

#[test]
fn invoke_handler_stays_off_forbidden_host_bridges() {
    let commands = invoke_handler_commands();
    for command in &commands {
        assert_lacks_forbidden_host_markers(&format!("Tauri command `{command}`"), command);
    }
    assert!(
        commands.iter().any(|command| command == "launch_run"),
        "expected the checked handler list to include launch_run"
    );
}

#[test]
fn autogenerated_permissions_stay_off_forbidden_host_bridges() {
    let permission_files = generated_permission_files();
    assert!(
        !permission_files.is_empty(),
        "no autogenerated permissions found under src-tauri/permissions/autogenerated"
    );
    for path in permission_files {
        assert_lacks_forbidden_host_markers(
            &format!("permission {}", path.display()),
            &read_text(&path),
        );
    }
}
