//! Vendored terminal-graphics and pet modules from openai/codex.
//!
//! These modules are copied from openai/codex (`codex-rs/tui` pets and
//! `codex-rs/terminal-detection`), licensed under Apache-2.0, copyright 2025
//! OpenAI. They were modified by RunHaven on 2026-06-26 and 2026-06-27: import
//! paths were adapted to this module layout, the upstream `tracing` logging and
//! TUI / asset-pack couplings were removed, and the ambient placement/rendering
//! path was made asset-agnostic so RunHaven can use it for both the logo and
//! native Cubby pet. The vendored code is a faithful adaptation, not a rewrite.
//!
//! The full license text is in `licenses/codex-Apache-2.0.txt` and the required
//! attribution notice is in `THIRD_PARTY_NOTICES.md` at the repo root.
//!
//! Active TUI code uses the pet animation, frame extraction, protocol detection,
//! and ambient image rendering paths. The per-module `#[allow(...)]` attributes
//! keep the broader vendored foundation compiling cleanly under
//! `cargo clippy --all-targets -- -D warnings` until each module is evaluated
//! and wired intentionally.

use std::path::Path;
use std::path::PathBuf;

#[allow(dead_code, clippy::all, clippy::pedantic)]
pub(crate) mod terminal_detection;

#[allow(dead_code, clippy::all, clippy::pedantic)]
pub(crate) mod sixel;

#[allow(dead_code, clippy::all, clippy::pedantic)]
pub(crate) mod image_protocol;

#[allow(dead_code, clippy::all, clippy::pedantic)]
pub(crate) mod catalog;

#[allow(dead_code, clippy::all, clippy::pedantic)]
pub(crate) mod model;

#[allow(dead_code, clippy::all, clippy::pedantic)]
pub(crate) mod frames;

#[allow(dead_code, clippy::all, clippy::pedantic)]
pub(crate) mod animation;

#[allow(dead_code, clippy::all, clippy::pedantic)]
pub(crate) mod ambient;

// TUI framework foundation vendored from codex-rs/tui (Apache-2.0). See
// THIRD_PARTY_NOTICES.md for sources and modifications. These carry a broader
// allow set than the pet modules: the foundation was written against a ratatui
// git fork, so against published ratatui 0.30.2 it has unused fork-era imports
// and one deprecated `Cell::skip` access that would otherwise fail
// `clippy -D warnings` while the code is unused.
#[allow(dead_code, unused, deprecated, clippy::all, clippy::pedantic)]
pub(crate) mod render;

#[allow(dead_code, unused, deprecated, clippy::all, clippy::pedantic)]
pub(crate) mod key_hint;

#[allow(dead_code, unused, deprecated, clippy::all, clippy::pedantic)]
pub(crate) mod wrapping;

#[allow(dead_code, unused, deprecated, clippy::all, clippy::pedantic)]
pub(crate) mod terminal_hyperlinks;

#[allow(dead_code, unused, deprecated, clippy::all, clippy::pedantic)]
pub(crate) mod selection_list;

#[allow(dead_code, unused, deprecated, clippy::all, clippy::pedantic)]
pub(crate) mod clipboard;

/// Resolve a built-in pet spritesheet path under a Codex/RunHaven home.
///
/// Codex defines this in `tui/src/pets/mod.rs` (which RunHaven does not vendor)
/// as `pub(crate) fn builtin_spritesheet_path(codex_home: &Path, file: &str) ->
/// PathBuf`. `model.rs` calls it through `super::builtin_spritesheet_path`, so
/// the signature is kept exact. RunHaven currently embeds Cubby directly, so
/// this helper remains only for vendored compatibility.
#[allow(dead_code)]
pub(crate) fn builtin_spritesheet_path(codex_home: &Path, file: &str) -> PathBuf {
    codex_home.join("pets").join("assets").join(file)
}
