// Browser-preview fallbacks for the Claude Keychain sign-in surface (path C).
// The preview never invokes Tauri and never touches a real Keychain, so it just
// reports "not signed in" and describes that sign-in runs only in the desktop
// app. No token or path is ever produced here.

import type {
  ClaudeOAuthClearResponse,
  ClaudeOAuthStartResponse,
  ClaudeOAuthStatusResponse
} from "./claude_oauth_types";

export function previewClaudeOAuthStatus(): ClaudeOAuthStatusResponse {
  // Nothing is saved on this Mac in preview, so the control shows its
  // signed-out placeholder and the sign-in button.
  return { status: "not_signed_in", reason: null, signInId: null, ageDays: null };
}

export function previewClaudeOAuthStart(): ClaudeOAuthStartResponse {
  // The real host sign-in cannot run in the browser preview. Reporting a
  // "signing" acknowledgement keeps the flow honest: the status poll then
  // settles back to "not signed in".
  return {
    signInId: "preview-claude-oauth",
    state: "signing",
    label: "Signing in",
    detail: "Preview mode: signing in on your Mac runs only in the desktop app."
  };
}

export function previewClaudeOAuthClear(): ClaudeOAuthClearResponse {
  return {
    outcome: "already_absent",
    detail: "Preview mode: nothing is saved on your Mac to remove."
  };
}
