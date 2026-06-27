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
        path: "base/Containerfile",
        bytes: include_bytes!("../../../../images/base/Containerfile"),
    },
    ImageAsset {
        path: "claude/Containerfile",
        bytes: include_bytes!("../../../../images/claude/Containerfile"),
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
    ImageAsset {
        path: "gemini/Containerfile",
        bytes: include_bytes!("../../../../images/gemini/Containerfile"),
    },
    ImageAsset {
        path: "gemini/package-lock.json",
        bytes: include_bytes!("../../../../images/gemini/package-lock.json"),
    },
    ImageAsset {
        path: "gemini/package.json",
        bytes: include_bytes!("../../../../images/gemini/package.json"),
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
