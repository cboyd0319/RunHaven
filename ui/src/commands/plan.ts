import type { AgentProfile, PlanWarning, RunPlanRequest, RunPlanResponse } from "./types";

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
