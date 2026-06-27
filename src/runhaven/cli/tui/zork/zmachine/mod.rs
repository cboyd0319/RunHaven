//! Vendored Z-machine core adapted from `moosepod/ferrif-zmachine`.
//!
//! The upstream engine is MIT licensed. RunHaven keeps the engine isolated under
//! the hidden TUI Zork easter egg so the copied compatibility code does not leak
//! into the broader CLI boundary. See `THIRD_PARTY_NOTICES.md` for attribution,
//! source revision, and modification notes.

#![allow(warnings)]
#![allow(clippy::all)]

pub mod instructions;
pub mod interfaces;
pub mod quetzal;
pub mod story;
pub mod vm;
