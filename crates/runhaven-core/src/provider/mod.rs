pub mod auth_broker;
pub mod auth_profiles;
/// macOS Keychain custody for the Claude OAuth broker (design C, Slice 1). The
/// data-protection keychain and `SecAccessControl` are macOS-only, so the whole
/// custody surface is gated to macOS; the workspace still builds elsewhere.
#[cfg(target_os = "macos")]
pub mod claude_oauth_keychain;
pub mod egress;
pub mod endpoints;
pub mod observability;
pub mod runtime;
