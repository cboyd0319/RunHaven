import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import { isLaunchReady, warningPreview } from "./plan";
import type {
  AgentProfile,
  AuthLogResponse,
  AuthStatusResponse,
  DashboardStatus,
  EgressLogResponse,
  ImageStatusResponse,
  LaunchRunRequest,
  LaunchRunResponse,
  LogSnapshotOptions,
  LogSnapshotResponse,
  ManagedSessionList,
  ManagedSessionLogSnapshotResponse,
  ManagedSessionSummary,
  RunPlanRequest,
  RunPlanResponse,
  KillRunResponse,
  RepairRunResponse,
  RunStatusResponse,
  SetupStatus,
  StopRunResponse
} from "./types";

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

let previewManagedSessions: ManagedSessionSummary[] = [];

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

export async function getRunStatus(runId: string): Promise<RunStatusResponse> {
  return call("get_run_status", { request: { runId } }, () => ({
    run: {
      runId,
      profile: "claude",
      workspace: "/tmp/runhaven-preview",
      networkMode: "provider",
      status: "running",
      timestamp: "preview",
      stateVolume: "preview",
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
  }));
}

export async function getLogSnapshot(
  runId: string,
  options: LogSnapshotOptions
): Promise<LogSnapshotResponse> {
  if (!options.confirmSensitiveOutput) {
    throw new Error("Confirm raw log viewing before loading output that may contain secrets.");
  }
  const requestedLines = options.lines ?? 200;
  return call(
    "get_log_snapshot",
    {
      request: {
        runId,
        lines: options.lines ?? null,
        confirmSensitiveOutput: options.confirmSensitiveOutput
      }
    },
    () => previewLogSnapshot(runId, requestedLines)
  );
}

export async function getManagedSessionLogSnapshot(
  sessionId: string,
  options: LogSnapshotOptions
): Promise<ManagedSessionLogSnapshotResponse> {
  if (!options.confirmSensitiveOutput) {
    throw new Error("Confirm raw log viewing before loading output that may contain secrets.");
  }
  const requestedLines = options.lines ?? 200;
  return call(
    "get_managed_session_log_snapshot",
    {
      request: {
        sessionId,
        lines: options.lines ?? null,
        confirmSensitiveOutput: options.confirmSensitiveOutput
      }
    },
    () => {
      const session = previewManagedSessions.find((item) => item.id === sessionId);
      if (!session) {
        throw new Error("managed session not found");
      }

      return {
        session: { ...session },
        snapshot: previewLogSnapshot(session.runId, requestedLines)
      };
    }
  );
}

function previewLogSnapshot(runId: string, requestedLines: number): LogSnapshotResponse {
  const text = "Preview log line one\nPreview log line two\n";

  return {
    runId,
    capturedAt: "preview",
    cursor: "preview",
    requestedLines,
    entries: [
      {
        source: "container-stdio",
        timestamp: "preview",
        text,
        lineCount: 2
      }
    ],
    text,
    returnedLines: 2,
    truncated: false,
    source: "container-stdio",
    warnings: ["Raw container output can contain secrets or workspace content."]
  };
}

export async function getManagedSessions(): Promise<ManagedSessionList> {
  return call("get_managed_sessions", {}, () => ({
    sessions: previewManagedSessions.map((session) => ({
      ...session,
      nextAction: { ...session.nextAction },
      events: session.events.map((event) => ({ ...event }))
    }))
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
    rememberPreviewManagedSession(snapshot);
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

export async function stopRun(runId: string): Promise<StopRunResponse> {
  return call("stop_run", { request: { runId, confirmStop: true } }, () => ({
    runId,
    containerName: "preview",
    status: "stop-requested"
  }));
}

export async function killRun(runId: string): Promise<KillRunResponse> {
  return call("kill_run", { request: { runId, confirmKill: true } }, () => ({
    runId,
    containerName: "preview",
    status: "kill-requested"
  }));
}

export async function repairRun(runId: string): Promise<RepairRunResponse> {
  return call("repair_run", { request: { runId, confirmRepair: true } }, () => ({
    runId,
    containerName: "preview",
    status: "removed",
    markerRemoved: true
  }));
}

function rememberPreviewManagedSession(
  snapshot: LaunchRunResponse["snapshot"]
) {
  const project = projectLabel(snapshot.workspace);
  const session: ManagedSessionSummary = {
    id: snapshot.runId,
    runId: snapshot.runId,
    agent: snapshot.profile,
    session: snapshot.session,
    title: `${agentLabel(snapshot.profile)} - ${project}`,
    project,
    status: "running",
    statusLabel: "Running",
    authStatus: "api-key-broker",
    network: snapshot.networkMode,
    startedAt: "preview",
    containerName: snapshot.containerName,
    detailCommand: `runhaven runs show ${snapshot.runId}`,
    lifecycle: "running",
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking status for this session."
    },
    events: [
      {
        source: "run",
        label: "Run record",
        detail: "Running",
        timestamp: "preview"
      },
      {
        source: "auth",
        label: "Auth",
        detail: "API key broker",
        timestamp: "preview"
      },
      {
        source: "network",
        label: "Network",
        detail: networkLabel(snapshot.networkMode),
        timestamp: "preview"
      }
    ]
  };

  previewManagedSessions = [
    session,
    ...previewManagedSessions.filter((item) => item.runId !== snapshot.runId)
  ];
}

function projectLabel(workspace: string): string {
  const trimmed = workspace.trim();
  if (!trimmed || trimmed === ".") {
    return "current project";
  }

  const parts = trimmed.split(/[\\/]+/).filter(Boolean);
  return parts[parts.length - 1] || "current project";
}

function agentLabel(agent: string): string {
  const trimmed = agent.trim();
  return trimmed
    ? `${trimmed.charAt(0).toUpperCase()}${trimmed.slice(1)}`
    : "Agent";
}

function networkLabel(network: string): string {
  switch (network.trim().toLowerCase()) {
    case "provider":
      return "Provider only";
    case "internal":
      return "Local only";
    case "internet":
      return "Internet";
    default:
      return network.trim() || "Unknown";
  }
}

export async function getEgressLog(): Promise<EgressLogResponse> {
  return call("get_egress_log", {}, () => ({ entries: [] }));
}

export async function getAuthLog(): Promise<AuthLogResponse> {
  return call("get_auth_log", {}, () => ({ entries: [] }));
}

export async function getAuthStatus(): Promise<AuthStatusResponse> {
  return call("get_auth_status", {}, () => ({
    status: "preview",
    runtime: "preview",
    profiles: [{ name: "codex", status: "api-key-prototype" }]
  }));
}

export async function chooseProjectFolder(): Promise<string | null> {
  if (!hasTauriRuntime()) {
    return null;
  }
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}
