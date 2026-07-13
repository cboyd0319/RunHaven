use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::support::paths::runhaven_cache_root;

#[derive(Clone, Copy, Debug)]
pub struct ImageAsset {
    pub path: &'static str,
    pub bytes: &'static [u8],
}

pub static IMAGE_ASSETS: &[ImageAsset] = &[
    ImageAsset {
        path: "antigravity/Containerfile",
        bytes: include_bytes!("../../../../images/antigravity/Containerfile"),
    },
    ImageAsset {
        path: "antigravity/runhaven-antigravity-login",
        bytes: include_bytes!("../../../../images/antigravity/runhaven-antigravity-login"),
    },
    ImageAsset {
        path: "base/Containerfile",
        bytes: include_bytes!("../../../../images/base/Containerfile"),
    },
    ImageAsset {
        path: "claude/Containerfile",
        bytes: include_bytes!("../../../../images/claude/Containerfile"),
    },
    ImageAsset {
        path: "claude/runhaven-claude-login",
        bytes: include_bytes!("../../../../images/claude/runhaven-claude-login"),
    },
    ImageAsset {
        path: "claude/package-lock.json",
        bytes: include_bytes!("../../../../images/claude/package-lock.json"),
    },
    ImageAsset {
        path: "claude/package.json",
        bytes: include_bytes!("../../../../images/claude/package.json"),
    },
    ImageAsset {
        path: "codex/Containerfile",
        bytes: include_bytes!("../../../../images/codex/Containerfile"),
    },
    ImageAsset {
        path: "codex/package-lock.json",
        bytes: include_bytes!("../../../../images/codex/package-lock.json"),
    },
    ImageAsset {
        path: "codex/package.json",
        bytes: include_bytes!("../../../../images/codex/package.json"),
    },
    ImageAsset {
        path: "common/apt-snapshot.conf",
        bytes: include_bytes!("../../../../images/common/apt-snapshot.conf"),
    },
    ImageAsset {
        path: "common/create-agent-user.sh",
        bytes: include_bytes!("../../../../images/common/create-agent-user.sh"),
    },
    ImageAsset {
        path: "common/debian-packages.txt",
        bytes: include_bytes!("../../../../images/common/debian-packages.txt"),
    },
    ImageAsset {
        path: "common/debian.sources",
        bytes: include_bytes!("../../../../images/common/debian.sources"),
    },
    ImageAsset {
        path: "copilot/Containerfile",
        bytes: include_bytes!("../../../../images/copilot/Containerfile"),
    },
    ImageAsset {
        path: "copilot/package-lock.json",
        bytes: include_bytes!("../../../../images/copilot/package-lock.json"),
    },
    ImageAsset {
        path: "copilot/package.json",
        bytes: include_bytes!("../../../../images/copilot/package.json"),
    },
];

pub fn assets_for_context(context: &str) -> Vec<ImageAsset> {
    let prefix = format!("{context}/");
    IMAGE_ASSETS
        .iter()
        .copied()
        .filter(|asset| asset.path.starts_with("common/") || asset.path.starts_with(&prefix))
        .collect()
}

pub fn context_digest(context: &str) -> String {
    let mut digest = Sha256::new();
    let mut assets = assets_for_context(context);
    assets.sort_by_key(|asset| asset.path);
    for asset in assets {
        digest.update(asset.path.as_bytes());
        digest.update(b"\0");
        digest.update(asset.bytes);
        digest.update(b"\0");
    }
    hex_digest(&digest.finalize())
}

pub fn materialize_image_context() -> Result<PathBuf> {
    let digest = all_assets_digest();
    let root = runhaven_cache_root().join("image-context").join(digest);
    for asset in IMAGE_ASSETS {
        let path = root.join(asset.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if fs::read(&path).ok().as_deref() != Some(asset.bytes) {
            fs::write(path, asset.bytes)?;
        }
    }
    Ok(root)
}

pub fn source_root() -> Option<PathBuf> {
    if let Some(value) = std::env::var_os("RUNHAVEN_IMAGE_ROOT") {
        let path = PathBuf::from(value);
        if path.join("common").is_dir() {
            return Some(path);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("images");
    if manifest.join("common").is_dir() {
        return Some(manifest);
    }
    None
}

fn all_assets_digest() -> String {
    let mut digest = Sha256::new();
    for asset in IMAGE_ASSETS {
        digest.update(asset.path.as_bytes());
        digest.update(b"\0");
        digest.update(asset.bytes);
        digest.update(b"\0");
    }
    hex_digest(&digest.finalize())
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes
        .iter()
        .flat_map(|byte| [byte >> 4, byte & 0x0f])
        .map(|n| char::from_digit(n as u32, 16).expect("hex digit"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::IMAGE_ASSETS;

    #[test]
    fn embedded_agent_images_match_the_supported_product_set() {
        let contexts = IMAGE_ASSETS
            .iter()
            .filter_map(|asset| asset.path.split_once('/').map(|(context, _)| context))
            .filter(|context| !matches!(*context, "base" | "common"))
            .collect::<BTreeSet<_>>();

        assert_eq!(
            contexts,
            BTreeSet::from(["antigravity", "claude", "codex", "copilot"])
        );
    }

    #[test]
    fn antigravity_login_wrapper_reports_only_secret_free_readiness() {
        let wrapper = IMAGE_ASSETS
            .iter()
            .find(|asset| asset.path == "antigravity/runhaven-antigravity-login")
            .expect("embedded Antigravity login wrapper");
        let source = std::str::from_utf8(wrapper.bytes).expect("UTF-8 wrapper");

        assert!(source.contains("agy \"$@\""));
        assert!(source.contains("[ \"$status\" -eq 1 ]"));
        assert!(
            source.contains("[ -s \"${HOME}/.gemini/antigravity-cli/antigravity-oauth-token\" ]")
        );
        assert!(source.contains("agy models >/dev/null 2>&1"));
        assert!(source.contains("exit 0"));
        assert!(source.contains("exit \"$status\""));
        for forbidden in ["cat ", "head ", "tail ", "printf ", "echo "] {
            assert!(
                !source.contains(forbidden),
                "the wrapper must never read or print credential bytes"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn antigravity_login_wrapper_preserves_cancellation_and_requires_credentials() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::process::Command;

        let wrapper = IMAGE_ASSETS
            .iter()
            .find(|asset| asset.path == "antigravity/runhaven-antigravity-login")
            .expect("embedded Antigravity login wrapper");
        let root = tempfile::tempdir().expect("temporary wrapper root");
        let bin = root.path().join("bin");
        let home = root.path().join("home");
        fs::create_dir_all(&bin).expect("fake binary directory");
        fs::create_dir_all(&home).expect("fake home");
        let wrapper_path = bin.join("runhaven-antigravity-login");
        let agy_path = bin.join("agy");
        fs::write(&wrapper_path, wrapper.bytes).expect("write wrapper");
        fs::write(
            &agy_path,
            b"#!/bin/sh\nif [ \"${1:-}\" = models ]; then\n    printf 'models-probe\\n'\n    exit \"$FAKE_MODELS_STATUS\"\nfi\nprintf 'agy-output\\n'\nexit \"$FAKE_AGY_STATUS\"\n",
        )
        .expect("write fake agy");
        for path in [&wrapper_path, &agy_path] {
            fs::set_permissions(path, fs::Permissions::from_mode(0o755))
                .expect("make test command executable");
        }
        let run = |status: i32, models_status: i32| {
            Command::new(&wrapper_path)
                .env("PATH", &bin)
                .env("HOME", &home)
                .env("FAKE_AGY_STATUS", status.to_string())
                .env("FAKE_MODELS_STATUS", models_status.to_string())
                .output()
                .expect("run login wrapper")
        };

        assert_eq!(run(1, 0).status.code(), Some(1));
        let token = home.join(".gemini/antigravity-cli/antigravity-oauth-token");
        fs::create_dir_all(token.parent().expect("token parent")).expect("token directory");
        fs::write(&token, b"fake-token-value").expect("fake token");
        for (agy_status, models_status, expected) in [
            (0, 1, 0),
            (1, 0, 0),
            (1, 1, 1),
            (137, 0, 137),
            (143, 0, 143),
        ] {
            let output = run(agy_status, models_status);
            assert_eq!(output.status.code(), Some(expected));
            assert_eq!(output.stdout, b"agy-output\n");
            assert!(!String::from_utf8_lossy(&output.stdout).contains("fake-token-value"));
        }
    }

    #[test]
    fn claude_login_wrapper_requires_claudes_own_auth_status() {
        let wrapper = IMAGE_ASSETS
            .iter()
            .find(|asset| asset.path == "claude/runhaven-claude-login")
            .expect("embedded Claude login wrapper");
        let source = std::str::from_utf8(wrapper.bytes).expect("UTF-8 wrapper");

        assert!(source.contains("claude auth login \"$@\""));
        assert!(source.contains("claude auth status >/dev/null 2>&1"));
        assert!(source.contains("exit \"$status\""));
        for forbidden in ["cat ", "head ", "tail ", "printf ", "echo "] {
            assert!(!source.contains(forbidden));
        }
    }

    #[cfg(unix)]
    #[test]
    fn claude_login_wrapper_preserves_failures_and_rejects_logged_out_status() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::process::Command;

        let wrapper = IMAGE_ASSETS
            .iter()
            .find(|asset| asset.path == "claude/runhaven-claude-login")
            .expect("embedded Claude login wrapper");
        let root = tempfile::tempdir().expect("temporary wrapper root");
        let bin = root.path().join("bin");
        fs::create_dir_all(&bin).expect("fake binary directory");
        let wrapper_path = bin.join("runhaven-claude-login");
        let claude_path = bin.join("claude");
        fs::write(&wrapper_path, wrapper.bytes).expect("write wrapper");
        fs::write(
            &claude_path,
            b"#!/bin/sh\nif [ \"${1:-}\" = auth ] && [ \"${2:-}\" = status ]; then\n    printf 'status-output\\n'\n    exit \"$FAKE_AUTH_STATUS\"\nfi\nprintf 'login-output\\n'\nexit \"$FAKE_LOGIN_STATUS\"\n",
        )
        .expect("write fake Claude");
        for path in [&wrapper_path, &claude_path] {
            fs::set_permissions(path, fs::Permissions::from_mode(0o755))
                .expect("make test command executable");
        }
        let run = |login_status: i32, auth_status: i32| {
            Command::new(&wrapper_path)
                .env("PATH", &bin)
                .env("FAKE_LOGIN_STATUS", login_status.to_string())
                .env("FAKE_AUTH_STATUS", auth_status.to_string())
                .output()
                .expect("run Claude login wrapper")
        };

        for (login_status, auth_status, expected) in
            [(0, 0, 0), (0, 1, 1), (1, 0, 1), (137, 0, 137)]
        {
            let output = run(login_status, auth_status);
            assert_eq!(output.status.code(), Some(expected));
            assert_eq!(output.stdout, b"login-output\n");
            assert!(!String::from_utf8_lossy(&output.stdout).contains("status-output"));
        }
    }
}
