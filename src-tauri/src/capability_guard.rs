//! Mechanical guard for Tauri capability scopes.
//!
//! The WebView is untrusted. Capabilities may only grant RunHaven's own typed
//! command permissions (`allow-*`) plus an explicitly vetted allowlist of
//! plugin permissions. This test fails closed if a capability ever grants a
//! generic host bridge (shell, fs, http, process, os, ...), so widening the
//! frontend trust boundary cannot pass review silently.
//!
//! See `docs/harness/boundaries/component-inventory.md` and
//! `docs/harness/boundaries/security-boundary-map.md`.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    /// Plugin permissions intentionally granted to the frontend. Adding an
    /// entry here is a deliberate trust-boundary decision and must be reviewed
    /// against the security model.
    const VETTED_PLUGIN_PERMISSIONS: &[&str] = &["dialog:allow-open"];

    #[test]
    fn capabilities_grant_only_vetted_scopes() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities");
        let mut checked = 0usize;

        for entry in fs::read_dir(&dir).expect("read capabilities directory") {
            let path = entry.expect("capabilities directory entry").path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let raw = fs::read_to_string(&path).expect("read capability file");
            let value: serde_json::Value =
                serde_json::from_str(&raw).expect("parse capability json");
            let permissions = value["permissions"].as_array().unwrap_or_else(|| {
                panic!("capability {} has no permissions array", path.display())
            });

            for permission in permissions {
                let name = match permission.as_str() {
                    Some(name) => name,
                    None => panic!(
                        "capability {} uses a non-string permission entry `{permission}`; \
                         review it as a trust-boundary change",
                        path.display()
                    ),
                };
                let vetted =
                    name.starts_with("allow-") || VETTED_PLUGIN_PERMISSIONS.contains(&name);
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
}
