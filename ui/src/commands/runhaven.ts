import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

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

const mockAgents: AgentProfile[] = [
  {
    name: "claude",
    description: "Claude Code with isolated project state.",
    image: "runhaven/claude:0.1.0",
    defaultCommand: ["claude"],
    providerHosts: ["api.anthropic.com"]
  },
  {
    name: "codex",
    description: "Codex CLI with isolated project state.",
    image: "runhaven/codex:0.1.0",
    defaultCommand: ["codex"],
    providerHosts: ["api.openai.com", "chatgpt.com"]
  },
  {
    name: "shell",
    description: "Generic shell profile for custom agent images.",
    image: "runhaven/base:0.1.0",
    defaultCommand: ["/bin/bash"],
    providerHosts: []
  }
];

export function hasTauriRuntime(): boolean {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function call<T>(command: string, args: Record<string, unknown>, fallback: () => T): Promise<T> {
  if (!hasTauriRuntime()) {
    return fallback();
  }
  return invoke<T>(command, args);
}

export async function getSetupStatus(): Promise<SetupStatus> {
  return call("get_setup_status", {}, () => ({
    ok: true,
    blockerCount: 0,
    sshAvailable: false,
    checks: [
      {
        name: "Preview mode",
        ok: true,
        detail: "Tauri runtime is not attached",
        remedy: "Open the desktop app to read local setup status."
      }
    ]
  }));
}

export async function listAgents(): Promise<AgentProfile[]> {
  return call("list_agents", {}, () => mockAgents);
}

export async function getDashboardStatus(): Promise<DashboardStatus> {
  return call("get_dashboard_status", {}, () => ({
    setup: {
      ok: true,
      blockerCount: 0,
      sshAvailable: false,
      checks: [
        {
          name: "Preview mode",
          ok: true,
          detail: "Using static preview data",
          remedy: "Open the desktop app to read local setup status."
        }
      ]
    },
    agents: mockAgents,
    activeRuns: [],
    recentRuns: [],
    warnings: ["Desktop runtime commands are unavailable in browser preview."]
  }));
}

export async function getImageStatus(agent: string): Promise<ImageStatusResponse> {
  return call("get_image_status", { request: { agent } }, () => ({
    agent,
    image: {
      agent,
      image: `runhaven/${agent}:0.1.0`,
      status: "ok",
      ready: true,
      expectedSourceDigest: "preview",
      localSourceDigest: "preview",
      fixCommand: null
    },
    builder: {
      status: "preview",
      detail: "Using static preview data",
      image: "preview",
      cpus: "2",
      memory: "2048 MiB",
      rosetta: null,
      startedDate: null,
      ipv4Address: null,
      warning: null
    }
  }));
}

export async function planRun(request: RunPlanRequest): Promise<RunPlanResponse> {
  return call("plan_run", { request }, () => ({
    profile: request.agent,
    workspace: request.workspacePath || ".",
    workspaceScope: request.workspaceScope,
    workspaceScopeNote: null,
    stateVolume: "preview",
    session: request.sessionName || "default",
    containerName: "preview",
    networkMode: request.networkMode,
    networkName: request.networkMode === "internet" ? null : "preview-network",
    egressSummary: request.networkMode === "internet" ? "unrestricted internet egress" : "restricted preview egress",
    image: request.image || "runhaven/preview:0.1.0",
    providerAllowedHosts: request.providerHosts,
    preflightCount: 0,
    warnings: warningPreview(request)
  }));
}

export async function launchRun(request: LaunchRunRequest): Promise<LaunchRunResponse> {
  return call("launch_run", { request }, () => {
    const plan = {
      ...request.plan,
      warnings: warningPreview(request.plan)
    };
    if (!isLaunchReady(plan, request.confirmLaunch, new Set(request.confirmedWarnings))) {
      throw new Error("Confirm the launch and every warning before starting a run.");
    }
    const runId = `preview-${Date.now()}`;
    const snapshot = {
      runId,
      status: "started",
      profile: request.plan.agent,
      workspace: request.plan.workspacePath || ".",
      stateVolume: "preview",
      session: request.plan.sessionName || "default",
      networkMode: request.plan.networkMode,
      containerName: "preview"
    };
    return {
      runId,
      status: "started",
      profile: snapshot.profile,
      workspace: snapshot.workspace,
      stateVolume: snapshot.stateVolume,
      session: snapshot.session,
      networkMode: snapshot.networkMode,
      snapshot
    };
  });
}

export async function chooseProjectFolder(): Promise<string | null> {
  if (!hasTauriRuntime()) {
    return null;
  }
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}

export function secureNetworkDefault(agent: AgentProfile | undefined): RunPlanRequest["networkMode"] {
  return agent && agent.providerHosts.length > 0 ? "provider" : "internal";
}

export function defaultRunPlanRequest(agent: AgentProfile | undefined): RunPlanRequest {
  return {
    agent: agent?.name ?? "claude",
    workspacePath: "",
    networkMode: secureNetworkDefault(agent),
    workspaceScope: "current",
    sessionName: null,
    readOnlyWorkspace: false,
    cpus: "4",
    memory: "4g",
    providerHosts: [],
    envNames: [],
    image: null,
    allowSensitiveWorkspace: false,
    allowRootUser: false,
    user: "agent"
  };
}

export type WarningPreviewContext = {
  activeRunCount?: number;
};

export function warningPreview(request: RunPlanRequest, context: WarningPreviewContext = {}): PlanWarning[] {
  const warnings: PlanWarning[] = [];
  const activeRunCount = context.activeRunCount ?? 0;
  if (activeRunCount > 0) {
    warnings.push({
      code: "active-runs",
      message: activeRunsWarningMessage(activeRunCount)
    });
  }
  if (activeRunCount > 0 && materialMemoryRequest(request.memory || "4g")) {
    warnings.push({
      code: "resource-memory",
      message: "This memory limit plus active runs may be material on the host. macOS memory pressure is not measured yet."
    });
  }
  if (request.networkMode === "internet") {
    warnings.push({
      code: "full-internet",
      message: "Full internet lets the agent reach unrestricted network destinations from inside the container."
    });
  }
  if (request.allowSensitiveWorkspace) {
    warnings.push({
      code: "sensitive-workspace",
      message: "The selected folder may contain private files. The agent can read files inside that folder."
    });
  }
  if (request.allowRootUser || request.user === "root" || request.user === "0") {
    warnings.push({
      code: "root-user",
      message: "The agent will run as root inside the container, weakening normal container guardrails."
    });
  }
  if (request.envNames.length > 0) {
    warnings.push({
      code: "environment",
      message: "Environment variable names are passed into the run. Values are never shown in the UI."
    });
  }
  if (request.image) {
    warnings.push({
      code: "custom-image",
      message: "Custom images are outside the bundled RunHaven image set."
    });
  }
  if (request.providerHosts.length > 0) {
    warnings.push({
      code: "provider-host",
      message: "Additional provider hosts allow that host and its subdomains in provider-only mode."
    });
  }
  return warnings;
}

function activeRunsWarningMessage(activeRunCount: number): string {
  const noun = activeRunCount === 1 ? "run" : "runs";
  const verb = activeRunCount === 1 ? "exists" : "exist";
  return `${activeRunCount} active RunHaven ${noun} already ${verb}. Starting another run starts another Apple container VM.`;
}

function materialMemoryRequest(memory: string): boolean {
  const bytes = memoryBytes(memory);
  return bytes !== null && bytes >= 2 * 1024 ** 3;
}

function memoryBytes(memory: string): number | null {
  const trimmed = memory.trim();
  if (!trimmed) {
    return null;
  }
  const suffix = trimmed.at(-1) ?? "";
  const multipliers: Record<string, number> = {
    k: 1024,
    m: 1024 ** 2,
    g: 1024 ** 3,
    t: 1024 ** 4
  };
  const multiplier = multipliers[suffix.toLowerCase()] ?? 1;
  const digits = multiplier === 1 ? trimmed : trimmed.slice(0, -1);
  if (!/^[1-9][0-9]*$/.test(digits)) {
    return null;
  }
  return Number(digits) * multiplier;
}

export function isLaunchReady(
  plan: Pick<RunPlanResponse, "warnings"> | null,
  confirmLaunch: boolean,
  confirmedWarnings: Set<string>,
  imageReady = true
): boolean {
  if (!plan || !confirmLaunch || !imageReady) {
    return false;
  }
  return plan.warnings.every((warning) => confirmedWarnings.has(warning.code));
}
