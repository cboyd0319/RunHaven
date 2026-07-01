export type CheckStatus = {
  name: string;
  ok: boolean;
  detail: string;
  remedy: string;
};

export type SetupStatus = {
  ok: boolean;
  checks: CheckStatus[];
  blockerCount: number;
  sshAvailable: boolean;
};

export type AgentProfile = {
  name: string;
  description: string;
  image: string;
  defaultCommand: string[];
  providerHosts: string[];
  signIn: string;
  broker: string;
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

export type DashboardStatus = {
  setup: SetupStatus;
  agents: AgentProfile[];
  activeRuns: RunSummary[];
  recentRuns: RunSummary[];
  warnings: string[];
};

export type ProfileImageStatus = {
  agent: string;
  image: string;
  status: "ok" | "missing" | "stale" | string;
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

export type LogSnapshotResponse = {
  runId: string;
  capturedAt: string;
  cursor: string;
  requestedLines: number;
  entries: LogSnapshotEntry[];
  sessionEntries?: SessionLogEntry[];
  text: string;
  returnedLines: number;
  truncated: boolean;
  source: string;
  warnings: string[];
};

export type LogSnapshotEntry = {
  source: string;
  timestamp: string;
  text: string;
  lineCount: number;
};

export type SessionLogRedaction = {
  status: "applied" | "notApplied";
  message: string;
};

export type SessionLogEntry = {
  sessionId: string;
  runId: string;
  source: string;
  cursor: string;
  timestamp: string;
  text: string;
  lineCount: number;
  redaction: SessionLogRedaction;
};

export type ManagedSessionLogSnapshotResponse = {
  session: ManagedSessionSummary;
  snapshot: LogSnapshotResponse;
  cursor: string;
  entries: SessionLogEntry[];
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

export type LogSnapshotOptions = {
  lines?: number;
  confirmSensitiveOutput: boolean;
};

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
  preflightCount: number;
  access: RunPlanAccess;
  warnings: PlanWarning[];
  worktree: boolean;
  worktreeNote: string | null;
};

export type LaunchRunRequest = {
  plan: RunPlanRequest;
  confirmLaunch: boolean;
  confirmedWarnings: string[];
};

export type StartedRunSnapshot = {
  runId: string;
  status: string;
  profile: string;
  workspace: string;
  stateVolume: string;
  authScope: string;
  session: string;
  networkMode: string;
  containerName: string;
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
  snapshot: StartedRunSnapshot;
};

export type LaunchStatusResponse = {
  runId: string;
  state: "starting" | "running" | "finished" | "failed" | string;
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

export type EgressLogEntry = {
  timestamp: string;
  profile: string;
  decision: string;
  host: string;
  port: number;
  count: number;
  reason: string;
  matchedRule: string;
  runId: string;
};

export type EgressLogResponse = {
  entries: EgressLogEntry[];
};

export type AuthLogEntry = {
  timestamp: string;
  profile: string;
  broker: string;
  decision: string;
  method: string;
  path: string;
  upstreamStatus: number | null;
  count: number;
  reason: string;
  runId: string;
};

export type AuthLogResponse = {
  entries: AuthLogEntry[];
};

export type AuthProfileStatus = {
  name: string;
  status: string;
};

export type AuthStatusResponse = {
  status: string;
  runtime: string;
  profiles: AuthProfileStatus[];
};

export type AuthExplainResponse = {
  name: string;
  status: string;
  runtime: string;
  supportedAuth: string[];
  hostKeeps: string[];
  guestReceives: string[];
  providerHosts: string[];
  currentSafePath: string;
  notes: string[];
};

export type {
  NetworkExplanationResponse,
  StateExplanationResponse,
  HostExplanationResponse,
  WorkspaceExplanationResponse
} from "./why_types";

export type {
  WorktreeReviewResponse,
  WorktreeActionResult,
  WorktreeActionResponse
} from "./worktree_types";
