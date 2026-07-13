use super::support::{manifest_dir, read_text};

#[test]
fn shareable_bundle_command_uses_the_fail_closed_release_wrapper() {
    let manifest = manifest_dir();
    let root = manifest.parent().expect("repository root");
    let package: serde_json::Value =
        serde_json::from_str(&read_text(&root.join("ui/package.json")))
            .expect("parse ui/package.json");
    assert_eq!(
        package
            .pointer("/scripts/tauri:bundle")
            .and_then(serde_json::Value::as_str),
        Some("cd .. && ./scripts/release-desktop.sh"),
        "the shareable bundle command must not bypass release verification"
    );
}

#[test]
fn release_wrapper_requires_signing_notarization_and_provenance() {
    let manifest = manifest_dir();
    let root = manifest.parent().expect("repository root");
    let source = format!(
        "{}\n{}",
        read_text(&root.join("scripts/release-desktop.sh")),
        read_text(&root.join("scripts/lib/macos-signing.sh"))
    );
    for required in [
        "the release worktree must be clean",
        "HEAD must have the exact release tag",
        "RUNHAVEN_SIGNING_IDENTITY",
        "RUNHAVEN_PROVISIONING_PROFILE",
        "RUNHAVEN_NOTARY_PROFILE",
        "CARGO_PROFILE_RELEASE_STRIP=none",
        "com.apple.application-identifier",
        "ApplicationIdentifierPrefix",
        "./init.sh",
        "scripts/apple_container_smoke.sh",
        "scripts/gui_tauri_live_smoke.sh --skip-tauri-build",
        "codesign --verify --deep --strict",
        "com.apple.security.get-task-allow",
        "notarytool submit",
        "stapler validate",
        "spctl --assess",
        "verify_release_dmg_layout",
        "hdiutil attach -quiet -readonly -nobrowse",
        "CFBundleShortVersionString",
        "cargo cyclonedx",
        "npm --prefix",
        "SHA256SUMS",
        "provenance.json",
        "RunHaven-$version-macos-arm64.zip",
        "-- --locked",
        "vtool -show-build",
        "pwd -P",
    ] {
        assert!(
            source.contains(required),
            "release wrapper lost required fail-closed control: {required}"
        );
    }
    assert!(
        !source.contains("--no-sign"),
        "shareable releases must never disable signing"
    );
    assert!(
        !source.contains("$publish_tmp/RunHaven.app"),
        "the release must publish a checksummed archive, not a raw app directory"
    );
}

#[test]
fn desktop_bundle_enables_the_hardened_runtime() {
    let path = manifest_dir().join("tauri.conf.json");
    let conf: serde_json::Value = serde_json::from_str(&read_text(&path)).expect("parse config");
    assert_eq!(
        conf.pointer("/bundle/macOS/hardenedRuntime")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "a shareable desktop bundle must enable the hardened runtime"
    );
}
