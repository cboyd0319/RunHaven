import type {
  AgentProfile,
  PlanWarning,
  RunPlanRequest,
  RunPlanResponse
} from "./types";

let launchSessionSequence = 0;
const apiKeyBrokerEnvByAgent: Record<string, string> = {
  claude: "ANTHROPIC_API_KEY",
  codex: "OPENAI_API_KEY"
};

// Pick the agent to preselect on first load. Prefer an agent that can sign in
// inside the app so the happy path never opens with a terminal-only instruction;
// fall back to the first agent when none support in-app sign-in.
export function preferredDefaultAgent(
  agents: AgentProfile[]
): AgentProfile | undefined {
  return agents.find((agent) => agent.supportsInAppSignIn) ?? agents[0];
}

export function secureNetworkDefault(agent: AgentProfile | undefined): RunPlanRequest["networkMode"] {
  return agent && agent.providerHosts.length > 0 ? "provider" : "internal";
}

export function supportsApiKeyBroker(agent: AgentProfile | undefined): boolean {
  return agent?.broker === "yes" && apiKeyBrokerEnvForAgent(agent) !== null;
}

export function apiKeyBrokerEnvForAgent(agent: AgentProfile | undefined): string | null {
  return agent ? (apiKeyBrokerEnvByAgent[agent.name] ?? null) : null;
}

export function reconcileRunPlanRequest(
  current: RunPlanRequest,
  agent: AgentProfile | undefined
): RunPlanRequest {
  if (!agent) {
    return current;
  }
  const agentChanged = current.agent !== agent.name;
  const fixedBrokerEnv = apiKeyBrokerEnvForAgent(agent);
  return {
    ...current,
    agent: agent.name,
    networkMode: agentChanged ? secureNetworkDefault(agent) : current.networkMode,
    apiKeyBrokerEnv: !fixedBrokerEnv || agentChanged || !current.apiKeyBrokerEnv ? null : fixedBrokerEnv
  };
}

export function defaultRunPlanRequest(agent: AgentProfile | undefined): RunPlanRequest {
  return {
    agent: agent?.name ?? "claude",
    workspacePath: "",
    networkMode: secureNetworkDefault(agent),
    workspaceScope: "current",
    authScope: "agent",
    sessionName: null,
    readOnlyWorkspace: false,
    cpus: "4",
    memory: "4g",
    providerHosts: [],
    envNames: [],
    apiKeyBrokerEnv: null,
    image: null,
    allowSensitiveWorkspace: false,
    allowRootUser: false,
    user: "agent",
    worktree: true
  };
}

export function authScopeHelp(scope: RunPlanRequest["authScope"]): string {
  if (scope === "agent") {
    return "Use the sign-in you did in RunHaven for this agent across your projects on this Mac.";
  }
  return "Keep the sign-in separate, just for this project.";
}

export function withLaunchSessionName(request: RunPlanRequest): RunPlanRequest {
  if (request.sessionName?.trim()) {
    return request;
  }
  return {
    ...request,
    sessionName: nextLaunchSessionName()
  };
}

export function nextLaunchSessionName(): string {
  launchSessionSequence += 1;
  return `gui-${Date.now().toString(36)}-${launchSessionSequence.toString(36)}`;
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
  if (!request.worktree && !request.readOnlyWorkspace) {
    warnings.push({
      code: "direct-workspace",
      message: "Direct mode lets the agent change files in your original project folder. Use the recommended isolated copy unless you need direct access."
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
  plan: Pick<RunPlanResponse, "warnings" | "access"> | null,
  confirmLaunch: boolean,
  confirmedWarnings: Set<string>,
  imageReady = true
): boolean {
  if (!plan || !confirmLaunch || !imageReady || plan.access.blocking) {
    return false;
  }
  return plan.warnings.every((warning) => confirmedWarnings.has(warning.code));
}
