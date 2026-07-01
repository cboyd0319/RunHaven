import { isLaunchReady, warningPreview } from "./plan";
import { rememberPreviewManagedSession } from "./preview_sessions";
import type {
  AgentProfile,
  AuthStatusResponse,
  DashboardStatus,
  ImageStatusResponse,
  LaunchRunRequest,
  LaunchRunResponse,
  LaunchStatusResponse,
  RunPlanAccess,
  RunPlanRequest,
  RunPlanResponse,
  RunStatusResponse,
  SetupStatus
} from "./types";

export { previewLogSnapshot } from "./preview_output";
export {
  previewManagedKill,
  previewManagedSessionDiagnostics,
  previewManagedRepair,
  previewManagedSessionLogSnapshot,
  previewManagedSessionStatus,
  previewManagedSessionsList,
  previewManagedStop,
  previewSessionInput
} from "./preview_sessions";

const mockAgents: AgentProfile[] = [
  {
    name: "claude",
    description: "Claude Code with isolated project state.",
    image: "runhaven/claude:0.1.0",
    defaultCommand: ["claude"],
    providerHosts: ["api.anthropic.com"],
    signIn: "runhaven login",
    broker: "yes"
  },
  {
    name: "codex",
    description: "Codex CLI with isolated project state.",
    image: "runhaven/codex:0.1.0",
    defaultCommand: ["codex"],
    providerHosts: ["api.openai.com", "chatgpt.com"],
    signIn: "runhaven login",
    broker: "yes"
  },
  {
    name: "shell",
    description: "Generic shell profile for custom agent images.",
    image: "runhaven/base:0.1.0",
    defaultCommand: ["/bin/bash"],
    providerHosts: [],
    signIn: "n/a",
    broker: "n/a"
  }
];

let previewRunSequence = 0;

export function previewSetupStatus(): SetupStatus {
  return {
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
  };
}

export function previewAgents(): AgentProfile[] {
  return mockAgents;
}

export function previewDashboardStatus(): DashboardStatus {
  return {
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
  };
}

export function previewImageStatus(agent: string): ImageStatusResponse {
  return {
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
  };
}

export function previewRunStatus(runId: string): RunStatusResponse {
  return {
    run: {
      runId,
      profile: "claude",
      workspace: "/tmp/runhaven-preview",
      networkMode: "provider",
      status: "running",
      timestamp: "preview",
      authScope: "project",
      session: "default",
      containerName: "preview"
    },
    container: {
      state: "running",
      image: "runhaven/preview:0.1.0",
      startedAt: "preview",
      resources: {
        cpus: "4",
        memoryBytes: 4 * 1024 ** 3
      },
      networks: [
        {
          network: "preview-network",
          hostname: "preview",
          ipv4Address: "192.0.2.10/24",
          ipv4Gateway: "192.0.2.1",
          ipv6Address: null
        }
      ]
    }
  };
}

export function previewPlanRun(request: RunPlanRequest): RunPlanResponse {
  return {
    profile: request.agent,
    workspace: request.workspacePath || ".",
    workspaceScope: request.workspaceScope,
    workspaceScopeNote: null,
    authScope: request.authScope,
    stateVolume: "preview",
    session: request.sessionName || "default",
    containerName: "preview",
    networkMode: request.networkMode,
    networkName: request.networkMode === "internet" ? null : "preview-network",
    egressSummary: previewEgressSummary(request),
    image: request.image || "runhaven/preview:0.1.0",
    providerAllowedHosts: request.providerHosts,
    preflightCount: 0,
    access: previewAccess(request),
    warnings: warningPreview(request)
  };
}

function previewAccess(request: RunPlanRequest): RunPlanAccess {
  if (request.apiKeyBrokerEnv?.trim()) {
    return {
      status: "ready",
      label: "API key variable found",
      detail: `${request.apiKeyBrokerEnv} is set for this preview.`,
      remedy: "",
      blocking: false,
      apiKeyBrokerEnv: request.apiKeyBrokerEnv,
      apiKeyBrokerEnvPresent: true
    };
  }
  if (request.agent === "shell") {
    return {
      status: "not-needed",
      label: "No sign-in needed",
      detail: "This shell profile does not need RunHaven-managed sign-in.",
      remedy: "",
      blocking: false,
      apiKeyBrokerEnv: null,
      apiKeyBrokerEnvPresent: null
    };
  }
  return {
    status: "setup",
    label: request.authScope === "agent" ? "Shared agent login" : "Project session login",
    detail:
      request.authScope === "agent"
        ? `RunHaven will use the shared ${request.agent} login state across projects on this Mac.`
        : "RunHaven will keep login state separate for this project session.",
    remedy:
      request.authScope === "agent"
        ? `If the agent asks for sign-in, finish it in the session or run runhaven login ${request.agent}.`
        : "If the agent asks for sign-in, finish it inside this session.",
    blocking: false,
    apiKeyBrokerEnv: null,
    apiKeyBrokerEnvPresent: null
  };
}

function previewEgressSummary(request: RunPlanRequest): string {
  if (request.networkMode === "internet") {
    return "unrestricted internet egress"
  }
  if (request.apiKeyBrokerEnv?.trim()) {
    return "restricted preview egress; API key broker enabled"
  }
  return "restricted preview egress"
}

export function previewLaunchRun(request: LaunchRunRequest): LaunchRunResponse {
  const plan = {
    ...request.plan,
    access: previewAccess(request.plan),
    warnings: warningPreview(request.plan)
  };
  if (!isLaunchReady(plan, request.confirmLaunch, new Set(request.confirmedWarnings))) {
    throw new Error("Confirm the launch and every warning before starting a run.");
  }
  const runId = `preview-${Date.now()}-${++previewRunSequence}`;
  const snapshot = {
    runId,
    status: "started",
    profile: request.plan.agent,
    workspace: request.plan.workspacePath || ".",
    stateVolume: "preview",
    authScope: request.plan.authScope,
    session: request.plan.sessionName || "default",
    networkMode: request.plan.networkMode,
    containerName: "preview"
  };
  rememberPreviewManagedSession(snapshot);
  return {
    runId,
    status: "started",
    profile: snapshot.profile,
    workspace: snapshot.workspace,
    stateVolume: snapshot.stateVolume,
    authScope: snapshot.authScope,
    session: snapshot.session,
    networkMode: snapshot.networkMode,
    snapshot
  };
}

export function previewLaunchStatus(runId: string): LaunchStatusResponse {
  if (!runId.trim()) {
    throw new Error("launch status not found");
  }
  return {
    runId,
    state: "running",
    label: "Running",
    detail: "The managed session is running in preview mode.",
    updatedAt: "preview"
  };
}

export function previewAuthStatus(): AuthStatusResponse {
  return {
    status: "preview",
    runtime: "preview",
    profiles: [
      { name: "claude", status: "api-key-broker" },
      { name: "codex", status: "api-key-broker" },
      { name: "gemini", status: "api-key-broker" },
      { name: "shell", status: "n/a" }
    ]
  };
}
