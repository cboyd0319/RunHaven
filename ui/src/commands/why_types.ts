// Read-only "why is this safe" explanation contracts. Each mirrors a secret-free
// Tauri response. The workspace type carries only the path-free verdict; the
// backend never sends an absolute host path to the WebView.

export type NetworkExplanationResponse = {
  mode: string;
  behavior: string;
  boundary: string;
  useWhen: string;
  dnsSafety: string;
  inspect: string;
};

export type StateExplanationResponse = {
  profile: string;
  defaultSession: string;
  homeMount: string;
  stateVolumePattern: string;
  namedSessionPattern: string;
  boundary: string;
  projectId: string;
  manage: string;
};

export type HostExplanationResponse = {
  host: string;
  port: number;
  decision: string;
  reason: string;
  nextAction: string;
  agent: string;
  profile: string;
  matchedRules: string[];
  bundledHosts: string[];
  endpointNotes: string[];
  dnsSafety: string;
};

export type WorkspaceExplanationResponse = {
  scope: string;
  decision: string;
  verdict: string;
  nextAction: string;
  boundary: string;
  sensitiveOverrideUsed: boolean;
};
