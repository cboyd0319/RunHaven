// Dashboard read-model types: setup health, per-agent profiles and sign-in
// readiness, and the run summaries the Home dashboard renders. Split out of
// types.ts by ownership. All fields are the sanitized, secret-free backend
// contract; no token, key, or raw host path is present.

export type SetupActionId =
  | "open-apple-container-installer"
  | "start-container-system";

export type SetupAction = {
  id: SetupActionId;
  label: string;
};

export type CheckStatus = {
  name: string;
  ok: boolean;
  detail: string;
  remedy: string;
  action: SetupAction | null;
};

export type SetupStatus = {
  ok: boolean;
  checks: CheckStatus[];
  blockerCount: number;
};

export type AgentProfile = {
  name: string;
  description: string;
  image: string;
  defaultCommand: string[];
  providerHosts: string[];
  signIn: string;
  broker: string;
  // True when the desktop app can sign this agent in fully in-app (device-code
  // login). False agents (like claude) sign in from a terminal. Derived from
  // core in the backend, never re-decided here.
  supportsInAppSignIn: boolean;
};

export type RunSummary = {
  runId: string;
  profile: string;
  workspace: string;
  network: string;
  status: string;
  timestamp: string;
  authScope: string;
  session: string;
};

// Per-agent sign-in state for the Home "Agent setup" panel. Backend-derived and
// secret-free: `category` reflects only what RunHaven can truthfully know
// (Shell needs no auth; product agents use secret-free native sign-in state).
// `label` is an agent-state string, never a claim that RunHaven manages the
// credential.
export type AgentReadiness = {
  name: string;
  category: "ready" | "signed-in" | "needs-sign-in";
  label: string;
};

export type DashboardStatus = {
  setup: SetupStatus;
  agents: AgentProfile[];
  agentReadiness: AgentReadiness[];
  activeRuns: RunSummary[];
  recentRuns: RunSummary[];
  warnings: string[];
  // The RunHaven desktop app version (crate version at build time). A plain,
  // non-secret string the Diagnostics view shows with the pinned runtime
  // versions from the setup checks.
  appVersion: string;
};

export type SetupRecoveryResponse = {
  actionId: SetupActionId;
  status: string;
  label: string;
  detail: string;
};
