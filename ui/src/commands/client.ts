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
  KillManagedSessionResponse,
  RepairRunResponse,
  RepairManagedSessionResponse,
  RunStatusResponse,
  SessionLogEntry,
  SetupStatus,
  StopManagedSessionResponse,
  StopRunResponse,
  WriteSessionInputResponse
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
let previewRunSequence = 0;
let previewSessionLogReads = new Map<string, number>();
const MAX_SESSION_INPUT_BYTES = 8 * 1024;

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
      const sessionSnapshot = { ...session };
      const readCount = nextPreviewSessionLogRead(session.id);
      const snapshot = previewLogSnapshot(session.runId, requestedLines, {
        cursor: `preview:${session.id}:${readCount}`,
        text: `Preview log line ${readCount} for ${session.id}\nPreview log line ${readCount} complete\n`
      });

      return {
        session: sessionSnapshot,
        snapshot,
        cursor: snapshot.cursor,
        entries: sessionLogEntries(sessionSnapshot, snapshot)
      };
    }
  );
}

function sessionLogEntries(
  session: ManagedSessionSummary,
  snapshot: LogSnapshotResponse
): SessionLogEntry[] {
  return snapshot.entries.map((entry) => ({
    sessionId: session.id,
    runId: session.runId,
    source: entry.source,
    cursor: snapshot.cursor,
    timestamp: entry.timestamp,
    text: entry.text,
    lineCount: entry.lineCount,
    redaction: {
      status: "notApplied",
      message: "Raw output is not automatically redacted."
    }
  }));
}

function nextPreviewSessionLogRead(sessionId: string): number {
  const nextRead = (previewSessionLogReads.get(sessionId) ?? 0) + 1;
  previewSessionLogReads.set(sessionId, nextRead);
  return nextRead;
}

function previewLogSnapshot(
  runId: string,
  requestedLines: number,
  patch: Partial<Pick<LogSnapshotResponse, "cursor" | "capturedAt" | "text">> = {}
): LogSnapshotResponse {
  const text = patch.text ?? "Preview log line one\nPreview log line two\n";
  const capturedAt = patch.capturedAt ?? patch.cursor ?? "preview";
  const cursor = patch.cursor ?? capturedAt;

  return {
    runId,
    capturedAt,
    cursor,
    requestedLines,
    entries: [
      {
        source: "container-stdio",
        timestamp: capturedAt,
        text,
        lineCount: text.split("\n").filter(Boolean).length
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
    const runId = `preview-${Date.now()}-${++previewRunSequence}`;
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

export async function stopManagedSession(
  sessionId: string
): Promise<StopManagedSessionResponse> {
  return call(
    "stop_managed_session",
    { request: { sessionId, confirmStop: true } },
    () => {
      const session = previewSession(sessionId);
      const result = {
        runId: session.runId,
        containerName: session.containerName,
        status: "stop-requested"
      };
      updatePreviewSessionStatus(sessionId, "stop-requested", "Stop requested");
      return {
        session: previewSession(sessionId),
        result
      };
    }
  );
}

export async function killRun(runId: string): Promise<KillRunResponse> {
  return call("kill_run", { request: { runId, confirmKill: true } }, () => ({
    runId,
    containerName: "preview",
    status: "kill-requested"
  }));
}

export async function killManagedSession(
  sessionId: string
): Promise<KillManagedSessionResponse> {
  return call(
    "kill_managed_session",
    { request: { sessionId, confirmKill: true } },
    () => {
      const session = previewSession(sessionId);
      const result = {
        runId: session.runId,
        containerName: session.containerName,
        status: "kill-requested"
      };
      updatePreviewSessionStatus(sessionId, "kill-requested", "Hard stop requested");
      return {
        session: previewSession(sessionId),
        result
      };
    }
  );
}

export async function repairRun(runId: string): Promise<RepairRunResponse> {
  return call("repair_run", { request: { runId, confirmRepair: true } }, () => ({
    runId,
    containerName: "preview",
    status: "removed",
    markerRemoved: true
  }));
}

export async function repairManagedSession(
  sessionId: string
): Promise<RepairManagedSessionResponse> {
  return call(
    "repair_managed_session",
    { request: { sessionId, confirmRepair: true } },
    () => {
      const session = previewSession(sessionId);
      const result = {
        runId: session.runId,
        containerName: session.containerName,
        status: "removed",
        markerRemoved: true
      };
      updatePreviewSessionStatus(sessionId, "removed", "Marker removed");
      return {
        session: previewSession(sessionId),
        result
      };
    }
  );
}

export async function writeSessionInput(
  sessionId: string,
  input: string
): Promise<WriteSessionInputResponse> {
  const clientTarget = `session:${sessionId}`;
  return call(
    "write_session_input",
    {
      request: {
        sessionId,
        input,
        confirmSend: true,
        clientTarget
      }
    },
    () => {
      validatePreviewSessionInput(input);
      const session = previewSession(sessionId);
      if (session.nextAction.kind === "auth_attention") {
        throw new Error("Session auth is not ready. Finish auth setup before sending input.");
      }
      if (session.nextAction.kind !== "monitor") {
        throw new Error(`Session cannot accept input while action is ${session.nextAction.kind}.`);
      }
      const acceptedBytes = sessionInputBytes(input);
      return {
        session,
        accepted: true,
        acceptedBytes,
        cursor: `session:${session.id}:input:${acceptedBytes}`,
        status: "accepted"
      };
    }
  );
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
  previewSessionLogReads.delete(snapshot.runId);
}

function previewSession(sessionId: string): ManagedSessionSummary {
  const session = previewManagedSessions.find((item) => item.id === sessionId);
  if (!session) {
    throw new Error("managed session not found");
  }
  return {
    ...session,
    nextAction: { ...session.nextAction },
    events: session.events.map((event) => ({ ...event }))
  };
}

function updatePreviewSessionStatus(
  sessionId: string,
  status: string,
  statusLabel: string
) {
  previewManagedSessions = previewManagedSessions.map((session) => {
    if (session.id !== sessionId) {
      return session;
    }
    return {
      ...session,
      status,
      statusLabel,
      lifecycle: status,
      events: session.events.map((event) =>
        event.source === "run"
          ? { ...event, detail: statusLabel, timestamp: "preview" }
          : { ...event }
      )
    };
  });
}

function validatePreviewSessionInput(input: string) {
  if (input.length === 0) {
    throw new Error("session input is required");
  }
  if (sessionInputBytes(input) > MAX_SESSION_INPUT_BYTES) {
    throw new Error(
      `session input is too long; keep each send at or below ${MAX_SESSION_INPUT_BYTES} bytes`
    );
  }
  if (input.includes("\0")) {
    throw new Error("session input contains a NUL byte");
  }
  if (containsTerminalEscape(input)) {
    throw new Error("session input contains a terminal escape sequence");
  }
}

function sessionInputBytes(input: string): number {
  return new TextEncoder().encode(input).length;
}

function containsTerminalEscape(input: string): boolean {
  return [...input].some((char) => char === "\u001b" || char === "\u009b");
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
