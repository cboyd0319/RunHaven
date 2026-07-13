import { warningPreview } from "./plan";
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
  StateExplanationResponse,
  WorkspaceExplanationResponse
} from "./types";
import type {
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
    image: "runhaven/claude:0.7.0",
    defaultCommand: ["claude"],
    providerHosts: ["api.anthropic.com"],
    signIn: "runhaven login",
    broker: "yes",
    supportsInAppSignIn: true
  },
  {
    name: "codex",
    description: "Codex CLI with isolated project state.",
    image: "runhaven/codex:0.7.0",
    defaultCommand: ["codex"],
    providerHosts: ["api.openai.com", "chatgpt.com"],
    signIn: "runhaven login",
    broker: "yes",
    supportsInAppSignIn: true
  },
  {
    name: "antigravity",
    description: "Google Antigravity CLI with isolated project state.",
    image: "runhaven/antigravity:0.7.0",
    defaultCommand: ["agy"],
    providerHosts: ["accounts.google.com", "oauth2.googleapis.com"],
    signIn: "runhaven login",
    broker: "no",
    supportsInAppSignIn: true
  },
  {
    name: "copilot",
    description: "GitHub Copilot CLI with isolated project state.",
    image: "runhaven/copilot:0.7.0",
    defaultCommand: ["copilot"],
    providerHosts: ["github.com", "api.github.com"],
    signIn: "runhaven login",
    broker: "no",
    supportsInAppSignIn: true
  }
];

let previewRunSequence = 0;

export function previewDashboardStatus(): DashboardStatus {
  return {
    setup: {
      ok: true,
      blockerCount: 0,
      checks: [
        {
          name: "Preview mode",
          ok: true,
          detail: "Using static preview data",
          remedy: "Open the desktop app to read local setup status.",
          action: null
        }
      ]
    },
    agents: mockAgents,
    agentReadiness: [
      { name: "claude", category: "sign-in-saved", label: "Sign-in saved" },
      { name: "codex", category: "sign-in-saved", label: "Sign-in saved" },
      { name: "antigravity", category: "needs-sign-in", label: "Needs sign-in" },
      { name: "copilot", category: "needs-sign-in", label: "Needs sign-in" }
    ],
    activeRuns: [],
    recentRuns: [],
    warnings: ["Desktop runtime commands are unavailable in browser preview."],
    appVersion: "preview"
  };
}

export function previewImageStatus(agent: string): ImageStatusResponse {
  return {
    agent,
    image: {
      agent,
      image: `runhaven/${agent}:0.7.0`,
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
      image: "runhaven/preview:0.7.0",
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
    image: request.image || "runhaven/preview:0.7.0",
    providerAllowedHosts: request.providerHosts,
    access: previewAccess(request),
    warnings: warningPreview(request),
    worktree: request.worktree,
    worktreeNote: request.worktree
      ? "RunHaven will work in a protected copy. Your original folder changes only if you choose Apply changes later."
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
    label: request.authScope === "agent" ? "Shared assistant sign-in" : "This project only",
    detail:
      request.authScope === "agent"
        ? `RunHaven will use the shared ${request.agent} sign-in across projects on this Mac.`
        : "RunHaven will keep the sign-in separate, just for this project.",
    remedy:
      request.authScope === "agent"
        ? "If the assistant asks you to sign in, finish it in the session."
        : "If the assistant asks for sign-in, finish it inside this session.",
    blocking: false,
    apiKeyBrokerEnv: null,
    apiKeyBrokerEnvPresent: null
  };
}

function previewEgressSummary(request: RunPlanRequest): string {
  if (request.networkMode === "internet") {
    return "This session can reach any site."
  }
  return "The assistant can reach only the services it needs."
}

export function previewLaunchRun(request: LaunchRunRequest): LaunchRunResponse {
  const plan = {
    ...request.plan,
    access: previewAccess(request.plan),
    warnings: warningPreview(request.plan)
  };
  if (plan.access.blocking) {
    throw new Error(`${plan.access.label} ${plan.access.detail} ${plan.access.remedy}`.trim());
  }
  const runId = `preview-${Date.now()}-${++previewRunSequence}`;
  const started: LaunchRunResponse = {
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
  rememberPreviewManagedSession(started, request.plan.worktree);
  return started;
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
    currentSafePath: "Use an API key from this Mac for a session without a terminal.",
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
    boundary: "only the mounted path is exposed at /workspace; the assistant’s home folder stays in a state volume.",
    sensitiveOverrideUsed: false
  };
}

export function previewAuthStatus(): AuthStatusResponse {
  // One profile per mock agent, so the preview never advertises an agent the
  // launch flow cannot actually offer.
  return {
    status: "preview",
    runtime: "preview",
    profiles: [
      { name: "claude", status: "api-key-broker" },
      { name: "codex", status: "api-key-broker" },
      { name: "antigravity", status: "needs-sign-in" },
      { name: "copilot", status: "needs-sign-in" }
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

// One stale preview network per page load, so the browser-preview and e2e paths
// can exercise the confirm-gated cleanup flow and see the count drop to zero.
let previewStaleNetworkCount = 1;

export function previewNetworkPrunePreview(): NetworkPrunePreview {
  return { count: previewStaleNetworkCount };
}

export function previewNetworkPrune(): NetworkPruneResponse {
  const removed = previewStaleNetworkCount;
  previewStaleNetworkCount = 0;
  return { removed, failed: 0 };
}
