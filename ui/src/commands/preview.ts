import { isLaunchReady, warningPreview } from "./plan";
import { rememberPreviewManagedSession } from "./preview_sessions";
import type {
  AgentProfile,
  AuthExplainResponse,
  AuthStatusResponse,
  DashboardStatus,
  HostExplanationResponse,
  ImageStatusResponse,
  LaunchRunRequest,
  LaunchRunResponse,
  LaunchStatusResponse,
  NetworkExplanationResponse,
  RunPlanAccess,
  RunPlanRequest,
  RunPlanResponse,
  RunStatusResponse,
  SetupStatus,
  StateExplanationResponse,
  WorkspaceExplanationResponse
} from "./types";
import type {
  AgentSignInStartResponse,
  AgentSignInStatusResponse,
  ImageBuildStartResponse,
  ImageBuildStatusResponse,
  NetworkPrunePreview,
  NetworkPruneResponse
} from "./maintenance_types";

export { previewLogSnapshot } from "./preview_output";
export {
  previewManagedKill,
  previewManagedCleanup,
  previewManagedSessionDiagnostics,
  previewManagedRepair,
  previewManagedSessionLogSnapshot,
  previewManagedSessionStatus,
  previewManagedSessionsList,
  previewManagedStop,
  previewSessionInput
} from "./preview_sessions";
export {
  previewManagedSessionWorktree,
  previewManagedWorktreeAction
} from "./preview_worktree";

const mockAgents: AgentProfile[] = [
  {
    name: "claude",
    description: "Claude Code with isolated project state.",
    image: "runhaven/claude:0.1.0",
    defaultCommand: ["claude"],
    providerHosts: ["api.anthropic.com"],
    signIn: "runhaven login",
    broker: "yes",
    supportsInAppSignIn: false
  },
  {
    name: "codex",
    description: "Codex CLI with isolated project state.",
    image: "runhaven/codex:0.1.0",
    defaultCommand: ["codex"],
    providerHosts: ["api.openai.com", "chatgpt.com"],
    signIn: "runhaven login",
    broker: "yes",
    supportsInAppSignIn: true
  },
  {
    name: "shell",
    description: "Generic shell profile for custom agent images.",
    image: "runhaven/base:0.1.0",
    defaultCommand: ["/bin/bash"],
    providerHosts: [],
    signIn: "n/a",
    broker: "n/a",
    supportsInAppSignIn: false
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
    warnings: warningPreview(request),
    worktree: request.worktree,
    worktreeNote: request.worktree
      ? "RunHaven will make an isolated copy of your project on a new git branch. Your original files change only if you merge later."
      : null
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

export function previewAuthExplain(agent: string): AuthExplainResponse {
  return {
    name: agent,
    status: "api-key-broker",
    runtime: "preview",
    supportedAuth: ["API key through the host-side broker"],
    hostKeeps: ["the host environment variable value you name"],
    guestReceives: ["a placeholder API key value"],
    providerHosts: ["api.example.com"],
    currentSafePath: "Use the API key broker for a headless run.",
    notes: ["Preview data only."]
  };
}

export function previewWhyNetwork(mode: string): NetworkExplanationResponse {
  return {
    mode,
    behavior: "routes egress through RunHaven's allowlist proxy.",
    boundary: "only bundled provider hosts and explicit hosts are eligible.",
    useWhen: "",
    dnsSafety: "IP literals and single-label names are denied.",
    inspect: "Preview data only."
  };
}

export function previewWhyState(agent: string): StateExplanationResponse {
  return {
    profile: agent,
    defaultSession: "default",
    homeMount: "/home/agent",
    stateVolumePattern: `runhaven-${agent}-<project-id>-home`,
    namedSessionPattern: `runhaven-${agent}-<project-id>-s-<session>-<digest>-home`,
    boundary: "workspace files are mounted separately at /workspace.",
    projectId: "derived from the workspace path; the path is not stored in the name.",
    manage: "Preview data only."
  };
}

export function previewWhyHost(host: string): HostExplanationResponse {
  return {
    host,
    port: 443,
    decision: "allowed",
    reason: "",
    nextAction: "",
    agent: "",
    profile: "",
    matchedRules: [`${host} (preview)`],
    bundledHosts: [],
    endpointNotes: [],
    dnsSafety: "Preview data only."
  };
}

export function previewWhyWorkspace(scope: string): WorkspaceExplanationResponse {
  return {
    scope,
    decision: "allowed",
    verdict: "RunHaven can mount this folder safely.",
    nextAction: "",
    boundary: "only the mounted path is exposed at /workspace; agent home stays in a state volume.",
    sensitiveOverrideUsed: false
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

export function previewImageBuildStart(agent: string): ImageBuildStartResponse {
  return {
    buildId: `preview-build-${agent}`,
    agent,
    tag: `runhaven/${agent}:preview`,
    state: "building",
    label: "Building",
    detail: "Preview mode: no image is rebuilt without the desktop app."
  };
}

export function previewImageBuildStatus(buildId: string): ImageBuildStatusResponse {
  return {
    buildId,
    agent: "preview",
    tag: "runhaven/preview:preview",
    state: "succeeded",
    label: "Succeeded",
    detail: "Preview mode: rebuild status is simulated.",
    outputTail: "Preview build output.",
    updatedAt: "1970-01-01T00:00:00Z"
  };
}

export function previewAgentSignInStart(agent: string): AgentSignInStartResponse {
  return {
    signInId: `preview-sign-in-${agent}`,
    agent,
    state: "starting",
    label: "Starting",
    detail: "Preview mode: no sign-in runs without the desktop app."
  };
}

export function previewAgentSignInStatus(signInId: string): AgentSignInStatusResponse {
  return {
    signInId,
    agent: "preview",
    state: "needs_browser",
    label: "Open your browser",
    detail: "Preview mode: sign-in progress is simulated.",
    outputTail: "Open https://example.com/device and enter CODE-1234.",
    updatedAt: "1970-01-01T00:00:00Z"
  };
}

export function previewNetworkPrunePreview(): NetworkPrunePreview {
  return { count: 0 };
}

export function previewNetworkPrune(): NetworkPruneResponse {
  return { removed: 0, failed: 0 };
}
