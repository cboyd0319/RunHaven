use std::fs;

use tempfile::TempDir;
use toml::Value;

use super::check_image_pins;

const DEBIAN_DIGEST: &str =
    "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const NODE_DIGEST: &str = "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const ANTIGRAVITY_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

struct Fixture {
    root: TempDir,
    pins: Value,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("tempdir");
        let pins: Value = toml::from_str(&format!(
            r#"
[container_images]
debian_trixie_slim = {{ tag = "trixie-slim", digest = "{DEBIAN_DIGEST}" }}
node_26_trixie_slim = {{ tag = "26.5.0-trixie-slim", digest = "{NODE_DIGEST}" }}
[node_runtime]
npm = "12.0.1"
[debian_snapshot]
timestamp = "20260710T000000Z"
debian_uri = "http://snapshot/debian/20260710T000000Z"
security_uri = "http://snapshot/security/20260710T000000Z"
[debian_trixie_arm64]
curl = "1"
[agent_cli]
claude_code = "2.1.207"
codex = "0.144.1"
antigravity_cli = "1.1.1"
copilot_cli = "1.0.70"
npm = "12.0.1"
[agent_cli_integrity]
claude_code = "sha512-claude"
codex = "sha512-codex"
antigravity_cli = "sha512-{ANTIGRAVITY_SHA}"
copilot_cli = "sha512-copilot"
[agent_cli_source]
antigravity_url = "https://storage.example/antigravity.tar.gz"
[agent_cli_linux_arm64]
claude = {{ package = "@native/claude", version = "2.1.207", integrity = "sha512-native-claude" }}
codex = {{ package = "@native/codex", version = "0.144.1-linux-arm64", integrity = "sha512-native-codex" }}
copilot = {{ package = "@native/copilot", version = "1.0.70", integrity = "sha512-native-copilot" }}
"#
        ))
        .expect("pins");
        let fixture = Self { root, pins };
        fixture.write("images/common/debian-packages.txt", "curl=1\n");
        fixture.write(
            "images/common/debian.sources",
            "URIs: http://snapshot/debian/20260710T000000Z\nURIs: http://snapshot/security/20260710T000000Z\n",
        );
        fixture.write(
            "crates/runhaven-core/src/runtime/plans/types.rs",
            &format!(
                "pub const VOLUME_PREP_IMAGE: &str = \"debian:trixie-slim@{DEBIAN_DIGEST}\";\n"
            ),
        );
        fixture.write("images/base/Containerfile", &containerfile("base", false));
        for (image, package, version, integrity, native, native_version, native_integrity) in [
            (
                "claude",
                "@anthropic-ai/claude-code",
                "2.1.207",
                "sha512-claude",
                "@native/claude",
                "2.1.207",
                "sha512-native-claude",
            ),
            (
                "codex",
                "@openai/codex",
                "0.144.1",
                "sha512-codex",
                "@native/codex",
                "0.144.1-linux-arm64",
                "sha512-native-codex",
            ),
            (
                "copilot",
                "@github/copilot",
                "1.0.70",
                "sha512-copilot",
                "@native/copilot",
                "1.0.70",
                "sha512-native-copilot",
            ),
        ] {
            fixture.write(
                &format!("images/{image}/Containerfile"),
                &containerfile(image, true),
            );
            fixture.write(
                &format!("images/{image}/package.json"),
                &format!(r#"{{"dependencies":{{"{package}":"{version}"}}}}"#),
            );
            fixture.write(
                &format!("images/{image}/package-lock.json"),
                &format!(
                    r#"{{"lockfileVersion":3,"packages":{{"":{{"dependencies":{{"{package}":"{version}"}}}},"node_modules/{package}":{{"version":"{version}","integrity":"{integrity}"}},"node_modules/{native}":{{"version":"{native_version}","integrity":"{native_integrity}"}}}}}}"#
                ),
            );
        }
        fixture.write(
            "images/antigravity/Containerfile",
            &format!(
                "{}ARG ANTIGRAVITY_CLI_VERSION=1.1.1\nARG ANTIGRAVITY_CLI_URL=https://storage.example/antigravity.tar.gz\nARG ANTIGRAVITY_CLI_SHA512={ANTIGRAVITY_SHA}\nRUN echo \"${{ANTIGRAVITY_CLI_SHA512}}  /tmp/antigravity-cli.tar.gz\" | sha512sum -c - \\\n",
                containerfile("antigravity", false)
            ),
        );
        fixture
    }

    fn path(&self, relative: &str) -> std::path::PathBuf {
        self.root.path().join(relative)
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, text).expect("write fixture");
    }

    fn replace(&self, relative: &str, old: &str, new: &str) {
        let path = self.path(relative);
        let text = fs::read_to_string(&path).expect("read fixture");
        assert!(text.contains(old), "fixture does not contain {old:?}");
        fs::write(path, text.replace(old, new)).expect("mutate fixture");
    }

    fn failures(&self) -> Vec<String> {
        check_image_pins(self.root.path(), &self.pins)
    }
}

fn containerfile(image: &str, node: bool) -> String {
    let from = if node {
        format!("FROM node:26.5.0-trixie-slim@{NODE_DIGEST}\n")
    } else {
        format!("FROM debian:trixie-slim@{DEBIAN_DIGEST}\n")
    };
    let npm = node.then(|| {
        format!(
            "COPY {image}/package.json {image}/package-lock.json /opt/runhaven-agent/\nRUN npm install -g npm@12.0.1 && npm ci --strict-allow-scripts=true\n"
        )
    });
    format!(
        "{from}COPY common/debian.sources /etc/apt/sources\nCOPY common/debian-packages.txt /tmp/debian-packages.txt\nRUN apt-get update && xargs -a /tmp/debian-packages.txt apt-get install\n{}USER agent\n",
        npm.as_deref().unwrap_or("")
    )
}

fn assert_failure_contains(fixture: &Fixture, needle: &str) {
    let failures = fixture.failures();
    assert!(
        failures.iter().any(|failure| failure.contains(needle)),
        "missing {needle:?} in {failures:#?}"
    );
}

#[test]
fn valid_minimal_image_ledger_fixture_passes() {
    assert_eq!(Fixture::new().failures(), Vec::<String>::new());
}

#[test]
fn package_and_lock_drift_are_rejected() {
    let fixture = Fixture::new();
    fixture.replace("images/claude/package.json", "2.1.207", "2.1.208");
    assert_failure_contains(&fixture, "package.json: dependencies do not match");

    let fixture = Fixture::new();
    fixture.replace(
        "images/claude/package-lock.json",
        "sha512-claude",
        "sha512-wrong",
    );
    assert_failure_contains(&fixture, "integrity does not match");

    let fixture = Fixture::new();
    fixture.replace(
        "images/claude/package-lock.json",
        "\"dependencies\":{\"@anthropic-ai/claude-code\":\"2.1.207\"}",
        "\"dependencies\":{\"@anthropic-ai/claude-code\":\"2.1.207\",\"extra\":\"1.0.0\"}",
    );
    assert_failure_contains(&fixture, "root @anthropic-ai/claude-code dependency");
}

#[test]
fn missing_lock_and_required_build_steps_are_rejected() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.path("images/claude/package-lock.json")).expect("remove lock");
    assert_failure_contains(&fixture, "package-lock.json: missing or invalid JSON");

    let fixture = Fixture::new();
    fixture.replace("images/claude/Containerfile", "npm ci ", "npm install ");
    assert_failure_contains(&fixture, "missing required build step npm ci");

    let fixture = Fixture::new();
    fixture.replace(
        "images/base/Containerfile",
        "COPY common/debian.sources ",
        "COPY other.sources ",
    );
    assert_failure_contains(
        &fixture,
        "missing required build step COPY common/debian.sources",
    );
}

#[test]
fn base_snapshot_and_debian_package_drift_are_rejected() {
    let fixture = Fixture::new();
    fixture.replace("images/base/Containerfile", DEBIAN_DIGEST, NODE_DIGEST);
    let path = fixture.path("images/base/Containerfile");
    let text = fs::read_to_string(&path).expect("read containerfile");
    fs::write(path, format!("{text}# expected {DEBIAN_DIGEST}\n")).expect("add decoy");
    assert_failure_contains(&fixture, "base image tag or digest");

    let fixture = Fixture::new();
    fixture.write(
        "images/common/debian.sources",
        "URIs: http://snapshot/debian/20260710T000000Z\nURIs: http://snapshot/security/20260710T000000Z\nURIs: http://mutable/debian\n",
    );
    assert_failure_contains(&fixture, "snapshot URIs do not match");

    let fixture = Fixture::new();
    fixture.write("images/common/debian-packages.txt", "curl=2\n");
    assert_failure_contains(&fixture, "curl does not match");
}

#[test]
fn antigravity_native_and_volume_integrity_drift_are_rejected() {
    let fixture = Fixture::new();
    fixture.replace(
        "images/antigravity/Containerfile",
        "sha512sum -c -",
        "sha512sum",
    );
    assert_failure_contains(&fixture, "archive checksum is not enforced");

    let fixture = Fixture::new();
    fixture.replace(
        "images/copilot/package-lock.json",
        "sha512-native-copilot",
        "sha512-wrong",
    );
    assert_failure_contains(&fixture, "Linux arm64 package does not match");

    let fixture = Fixture::new();
    fixture.replace(
        "crates/runhaven-core/src/runtime/plans/types.rs",
        DEBIAN_DIGEST,
        NODE_DIGEST,
    );
    fixture.write(
        "crates/runhaven-core/src/runtime/plans/decoy.txt",
        DEBIAN_DIGEST,
    );
    assert_failure_contains(&fixture, "runtime volume-prep image does not match");
}

#[test]
fn npm_version_and_missing_containerfile_are_rejected() {
    let fixture = Fixture::new();
    fixture.replace("images/codex/Containerfile", "npm@12.0.1", "npm@12.0.2");
    assert_failure_contains(&fixture, "npm version does not match");

    let fixture = Fixture::new();
    fs::remove_file(fixture.path("images/base/Containerfile")).expect("remove containerfile");
    assert_failure_contains(&fixture, "images/base/Containerfile: unreadable");
}

#[test]
fn retired_agent_image_and_pin_entries_are_rejected() {
    let fixture = Fixture::new();
    fixture.write(
        "images/gemini/Containerfile",
        &containerfile("gemini", true),
    );
    assert_failure_contains(&fixture, "unexpected bundled image directory gemini");

    let mut fixture = Fixture::new();
    fixture
        .pins
        .get_mut("agent_cli")
        .and_then(Value::as_table_mut)
        .expect("agent_cli table")
        .insert(
            "gemini_cli".to_string(),
            Value::String("0.50.0".to_string()),
        );
    assert_failure_contains(&fixture, "unexpected agent_cli pin gemini_cli");
}
