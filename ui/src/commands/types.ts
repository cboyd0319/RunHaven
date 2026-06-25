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
};

export type RunSummary = {
  runId: string;
  profile: string;
  workspace: string;
  network: string;
  status: string;
  timestamp: string;
  stateVolume: string;
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
  stateVolume: string;
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
  requestedLines: number;
  text: string;
  returnedLines: number;
  truncated: boolean;
  source: string;
  warnings: string[];
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
  sessionName: string | null;
  readOnlyWorkspace: boolean;
  cpus: string;
  memory: string;
  providerHosts: string[];
  envNames: string[];
  image: string | null;
  allowSensitiveWorkspace: boolean;
  allowRootUser: boolean;
  user: string;
};

export type PlanWarning = {
  code: string;
  message: string;
};

export type RunPlanResponse = {
  profile: string;
  workspace: string;
  workspaceScope: string;
  workspaceScopeNote: string | null;
  stateVolume: string;
  session: string;
  containerName: string;
  networkMode: string;
  networkName: string | null;
  egressSummary: string;
  image: string;
  providerAllowedHosts: string[];
  preflightCount: number;
  warnings: PlanWarning[];
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
  session: string;
  networkMode: string;
  snapshot: StartedRunSnapshot;
};
