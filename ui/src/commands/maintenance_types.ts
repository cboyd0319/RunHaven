// Types for the in-app maintenance surface: agent image rebuild and network
// cleanup. Both mirror the sanitized Tauri command responses; counts and
// bounded, redacted output only, never raw host paths or network names.

// Closed progress states for the two long-running maintenance flows. The wire
// field stays a raw string because the backend contract is open; these unions
// name the values the UI reasons about, and `isTerminalState` is the one place
// that narrows a progress state to "done".
export type ImageBuildState = "building" | "succeeded" | "failed";
export type AgentSignInState =
  | "starting"
  | "needs_browser"
  | "succeeded"
  | "failed";

export function isTerminalState(state: string): boolean {
  return state === "succeeded" || state === "failed";
}

export type ImageBuildStartResponse = {
  buildId: string;
  agent: string;
  tag: string;
  state: string;
  label: string;
  detail: string;
};

export type ImageBuildStatusResponse = {
  buildId: string;
  agent: string;
  tag: string;
  state: string;
  label: string;
  detail: string;
  outputTail: string;
  updatedAt: string;
};

// In-app agent sign-in. The desktop app only triggers sign-in and watches
// sanitized progress; the credential stays with RunHaven core. No token, host
// path, or unbounded log is present in these shapes.

export type AgentSignInStartResponse = {
  signInId: string;
  agent: string;
  state: string;
  label: string;
  detail: string;
};

export type AgentSignInStatusResponse = {
  signInId: string;
  agent: string;
  state: string;
  label: string;
  detail: string;
  // Bounded, secret-redacted tail of the login output, so the sign-in link and
  // short-lived device code show while any token or key is scrubbed.
  outputTail: string;
  updatedAt: string;
};

export type NetworkPrunePreview = {
  count: number;
};

export type NetworkPruneResponse = {
  removed: number;
  failed: number;
};
