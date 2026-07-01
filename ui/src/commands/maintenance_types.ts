// Types for the in-app maintenance surface: agent image rebuild and network
// cleanup. Both mirror the sanitized Tauri command responses; counts and
// bounded, redacted output only, never raw host paths or network names.

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
  state: "building" | "succeeded" | "failed" | string;
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
  state: "starting" | "needs_browser" | "succeeded" | "failed" | string;
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
