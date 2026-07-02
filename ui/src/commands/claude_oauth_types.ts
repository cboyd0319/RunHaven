// Types for the Claude Keychain sign-in surface (design C, path C). The desktop
// app only triggers sign-in, polls a sanitized status, and can remove the saved
// token; the credential lives only in the Mac's Keychain and never crosses to
// the WebView. No token, token prefix, or path appears in any of these shapes.

// The sanitized status the backend derives from Keychain PRESENCE plus the
// in-flight sign-in phase, never from the token value. The wire field stays a
// raw string; this union names the values the UI reasons about.
export type ClaudeOAuthStatus =
  | "not_signed_in"
  | "signing"
  | "signed_in"
  | "failed";

// Acknowledgement that the background host capture started. Carries only an id
// and fixed presentation strings; never a token.
export type ClaudeOAuthStartResponse = {
  signInId: string;
  state: string;
  label: string;
  detail: string;
};

export type ClaudeOAuthStatusResponse = {
  status: string;
  // A fixed, path-free failure message, present only when status is "failed".
  reason: string | null;
  // Correlates the last sign-in attempt.
  signInId: string | null;
  // Whole days since the token was saved, from the item's own non-secret
  // creation date. Present only when signed in; drives the re-sign-in cadence
  // prompt. A plain day count, never a token-derived value.
  ageDays: number | null;
};

export type ClaudeOAuthClearResponse = {
  // "cleared" when RunHaven's own item was removed, or "already_absent".
  outcome: string;
  detail: string;
};
