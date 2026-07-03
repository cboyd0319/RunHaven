pub mod auth_broker;
pub mod auth_profiles;
/// Non-secret age of the stored Claude token, from the Keychain item's own
/// creation date (design C, Section 1 Hardening 4). Used only by the macOS
/// custody surface, so it is gated to macOS to match.
#[cfg(target_os = "macos")]
mod claude_oauth_age;
/// Host `claude setup-token` PTY capture for the Claude OAuth broker (design C,
/// Section 2). Split from the custody surface by ownership; macOS-only to match
/// the Keychain sink it feeds.
#[cfg(target_os = "macos")]
pub mod claude_oauth_capture;
/// macOS Keychain custody for the Claude OAuth broker (design C, Slice 1). The
/// data-protection keychain and `SecAccessControl` are macOS-only, so the whole
/// custody surface is gated to macOS; the workspace still builds elsewhere.
#[cfg(target_os = "macos")]
pub mod claude_oauth_keychain;
pub mod egress;
pub mod endpoints;
pub mod observability;
pub mod runtime;
