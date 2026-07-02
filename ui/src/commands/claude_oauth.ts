// Claude Keychain sign-in command wrappers (path C). Split out of client.ts by
// ownership; they route through the same typed `call` helper, so `invoke` itself
// stays centralized in client.ts. No wrapper accepts or returns a token: the
// token lives only in the Mac's Keychain and never crosses to the WebView.

import { call } from "./client";
import {
  previewClaudeOAuthClear,
  previewClaudeOAuthStart,
  previewClaudeOAuthStatus
} from "./preview";
import type {
  ClaudeOAuthClearResponse,
  ClaudeOAuthStartResponse,
  ClaudeOAuthStatusResponse
} from "./claude_oauth_types";

// Path C: start the host Claude Keychain sign-in. `confirm` rides the request,
// but Rust still shows its own native confirmation before anything runs, and no
// token is ever returned. The browser preview never invokes Tauri.
export async function startClaudeOAuthSignIn(): Promise<ClaudeOAuthStartResponse> {
  return call(
    "start_claude_oauth_signin",
    { request: { confirm: true } },
    previewClaudeOAuthStart
  );
}

// Sanitized Claude sign-in status derived from Keychain presence plus the
// non-secret token age. Never returns a token, prefix, or path.
export async function getClaudeOAuthStatus(): Promise<ClaudeOAuthStatusResponse> {
  return call("get_claude_oauth_status", {}, previewClaudeOAuthStatus);
}

// Remove RunHaven's own saved Claude token. Rust shows a native confirmation
// before deleting and reports only whether an item was removed.
export async function clearClaudeOAuth(): Promise<ClaudeOAuthClearResponse> {
  return call(
    "clear_claude_oauth",
    { request: { confirm: true } },
    previewClaudeOAuthClear
  );
}
