import type {
  AuthLogEntry,
  AuthProfileStatus,
  EgressLogEntry
} from "./diagnostics_types";

// Known image states. The wire field stays a raw string (open backend
// contract); the union names the values the UI understands.
export type ProfileImageState = "ok" | "missing" | "stale";

export type ProfileImageStatus = {
  agent: string;
  image: string;
  status: string;
  ready: boolean;
  expectedSourceDigest: string;
  localSourceDigest: string | null;
  fixCommand: string | null;
};

export type BuilderStatus = {
  status: string;
  detail: string;
  image: string | null;
  cpus: string | null;
  memory: string | null;
  rosetta: boolean | null;
  startedDate: string | null;
  ipv4Address: string | null;
  warning: string | null;
};

export type ImageStatusResponse = {
  agent: string;
  image: ProfileImageStatus;
  builder: BuilderStatus;
};

export type RunStatusRun = {
  runId: string;
  profile: string;
  workspace: string;
  networkMode: string;
  status: string;
  timestamp: string;
  authScope: string;
  session: string;
  containerName: string;
};

export type RunStatusResources = {
  cpus: string | null;
  memoryBytes: number | null;
};

export type RunStatusNetwork = {
  network: string | null;
  hostname: string | null;
  ipv4Address: string | null;
  ipv4Gateway: string | null;
  ipv6Address: string | null;
};

export type RunStatusContainer = {
  state: string;
  image: string | null;
  startedAt: string | null;
  resources: RunStatusResources;
  networks: RunStatusNetwork[];
};

export type RunStatusResponse = {
  run: RunStatusRun;
  container: RunStatusContainer;
};

// One bounded, redacted output snapshot. `capturedAt` is the capture timestamp
// and doubles as the change marker the live poll compares between refreshes.
export type LogSnapshotResponse = {
  runId: string;
  capturedAt: string;
  requestedLines: number;
  text: string;
  returnedLines: number;
  truncated: boolean;
  source: string;
  warnings: string[];
};

export type ManagedSessionLogSnapshotResponse = {
  session: ManagedSessionSummary;
  snapshot: LogSnapshotResponse;
};

export type ManagedSessionStatusResponse = {
  session: ManagedSessionSummary;
  container: RunStatusContainer | null;
  statusUnavailable: string | null;
  recovery: ManagedSessionRecovery | null;
};

export type ManagedSessionDiagnosticsResponse = {
  session: ManagedSessionSummary;
  authStatus: AuthProfileStatus;
  authLogEntries: AuthLogEntry[];
  egressLogEntries: EgressLogEntry[];
};

export type ManagedSessionRecovery = {
  status: string;
  title: string;
  detail: string;
  remedy: string;
  repairAvailable: boolean;
};

export type StopManagedSessionResponse = {
  session: ManagedSessionSummary;
  result: StopRunResponse;
};

export type KillManagedSessionResponse = {
  session: ManagedSessionSummary;
  result: KillRunResponse;
};

export type RepairManagedSessionResponse = {
  session: ManagedSessionSummary;
  result: RepairRunResponse;
};

export type CleanupManagedSessionResult = {
  runId: string;
  target: string;
  status: string;
};

export type CleanupManagedSessionResponse = {
  session: ManagedSessionSummary;
  result: CleanupManagedSessionResult;
};

export type WriteSessionInputResponse = {
  session: ManagedSessionSummary;
  accepted: boolean;
  acceptedBytes: number;
  cursor: string;
  status: string;
};

// Known managed-session lifecycle and next-action values. The wire fields stay
// raw strings (open backend contract); these unions name the values the UI
// reasons about, and the session guards in ManagedSessionCard validate their
// literal sets against them with `satisfies`.
export type SessionLifecycle =
  | "starting"
  | "running"
  | "stop-requested"
  | "kill-requested"
  | "finished"
  | "failed"
  | "ended"
  | "removed"
  | "cleaned-up";

export type SessionNextActionKind =
  | "monitor"
  | "auth_attention"
  | "recover";

export type ManagedSessionSummary = {
  id: string;
  runId: string;
  agent: string;
  session: string;
  title: string;
  project: string;
  status: string;
  statusLabel: string;
  authStatus: string;
  authScope: string;
  network: string;
  startedAt: string;
  containerName: string;
  detailCommand: string;
  lifecycle: string;
  input: ManagedSessionInput;
  cleanup?: ManagedSessionCleanup;
  nextAction: ManagedSessionNextAction;
  events: ManagedSessionEvent[];
};

export type ManagedSessionInput = {
  attached: boolean;
  status: string;
  detail: string;
};

export type ManagedSessionCleanup = {
  available: boolean;
  target: string;
  label: string;
  detail: string;
};

export type ManagedSessionNextAction = {
  kind: string;
  label: string;
  detail: string;
};

export type ManagedSessionEvent = {
  source: string;
  label: string;
  detail: string;
  timestamp: string;
};

export type ManagedSessionList = {
  sessions: ManagedSessionSummary[];
};

// Messages streamed from the host over the terminal Channel. Mirrors the Rust
// TerminalMessage enum (serde tag "event", snake_case): raw PTY bytes as a
// number array, or a final exit code. This is the only terminal payload the
// WebView ever sees; it is rendered in xterm's in-memory scrollback and never
// stored or logged in the frontend.
export type TerminalMessage =
  | { event: "output"; data: number[] }
  | { event: "exit"; code: number; completionRecorded?: boolean };

// Which selection-scoped menu items apply right now, sent to Rust so the
// native menu grays out actions that would do nothing (navigation invariant 5).
// Booleans only: no session ids, paths, counts, or secrets cross this boundary.
// Mirrors MenuState in src-tauri/src/menu/state.rs.
export type MenuState = {
  showAttentionBadge: boolean;
  sidebarOpen: boolean;
  selectedSessionCanRunControl: boolean;
  selectedSessionCanSendInput: boolean;
  selectedSessionHasCleanup: boolean;
};

export type LogSnapshotOptions = {
  lines?: number;
  confirmSensitiveOutput: boolean;
};

// Bounded session-input contract, shared by the composer and the browser
// preview so the byte ceiling has a single source of truth.
export const MAX_SESSION_INPUT_BYTES = 8 * 1024;

export function sessionInputBytes(input: string): number {
  return new TextEncoder().encode(input).length;
}

export type RunPlanRequest = {
  agent: string;
  workspacePath: string;
  networkMode: "provider" | "internal" | "internet";
  workspaceScope: "current" | "git-root";
  authScope: "agent" | "project";
  sessionName: string | null;
  readOnlyWorkspace: boolean;
  cpus: string;
  memory: string;
  providerHosts: string[];
  envNames: string[];
  apiKeyBrokerEnv: string | null;
  image: string | null;
  allowSensitiveWorkspace: boolean;
  allowRootUser: boolean;
  user: string;
  worktree: boolean;
};

export type PlanWarning = {
  code: string;
  message: string;
};

export type RunPlanAccess = {
  status: string;
  label: string;
  detail: string;
  remedy: string;
  blocking: boolean;
  apiKeyBrokerEnv: string | null;
  apiKeyBrokerEnvPresent: boolean | null;
};

export type RunPlanResponse = {
  profile: string;
  workspace: string;
  workspaceScope: string;
  workspaceScopeNote: string | null;
  authScope: string;
  stateVolume: string;
  session: string;
  containerName: string;
  networkMode: string;
  networkName: string | null;
  egressSummary: string;
  image: string;
  providerAllowedHosts: string[];
  access: RunPlanAccess;
  warnings: PlanWarning[];
  worktree: boolean;
  worktreeNote: string | null;
};

export type LaunchRunRequest = {
  plan: RunPlanRequest;
};

export type LaunchRunResponse = {
  runId: string;
  status: "started";
  profile: string;
  workspace: string;
  stateVolume: string;
  authScope: string;
  session: string;
  networkMode: string;
  containerName: string;
};

// Known launch states. The wire field stays a raw string (open backend
// contract); the union names the values the UI understands.
export type LaunchState = "starting" | "running" | "finished" | "failed";

export type LaunchStatusResponse = {
  runId: string;
  state: string;
  label: string;
  detail: string;
  updatedAt: string;
};

export type StopRunResponse = {
  runId: string;
  containerName: string;
  status: string;
};

export type KillRunResponse = {
  runId: string;
  containerName: string;
  status: string;
};

export type RepairRunResponse = {
  runId: string;
  containerName: string;
  status: string;
  markerRemoved: boolean;
};

export type {
  EgressLogEntry,
  EgressLogResponse,
  AuthLogEntry,
  AuthLogResponse,
  AuthProfileStatus,
  AuthStatusResponse,
  AuthExplainResponse
} from "./diagnostics_types";

export type {
  NetworkExplanationResponse,
  StateExplanationResponse,
  HostExplanationResponse,
  WorkspaceExplanationResponse
} from "./why_types";

export type {
  WorktreeFileChange,
  WorktreeReviewResponse,
  WorktreeActionResult,
  WorktreeActionResponse
} from "./worktree_types";

export type {
  CheckStatus,
  SetupActionId,
  SetupAction,
  SetupStatus,
  AgentProfile,
  RunSummary,
  AgentReadiness,
  DashboardStatus,
  SetupRecoveryResponse
} from "./dashboard_types";
