import { previewLogSnapshot } from "./preview_output";
import { MAX_SESSION_INPUT_BYTES, sessionInputBytes } from "./types";
import type {
  CleanupManagedSessionResponse,
  KillManagedSessionResponse,
  LaunchRunResponse,
  ManagedSessionDiagnosticsResponse,
  ManagedSessionList,
  ManagedSessionLogSnapshotResponse,
  ManagedSessionStatusResponse,
  ManagedSessionSummary,
  RepairManagedSessionResponse,
  StopManagedSessionResponse,
  WriteSessionInputResponse
} from "./types";

let previewManagedSessions: ManagedSessionSummary[] = [];
let previewSessionLogReads = new Map<string, number>();
// Preview runs launched with the isolated-copy (worktree) option. Stopping one
// finishes it with a saved worktree so the review flow is reachable in preview.
const previewWorktreeSessions = new Set<string>();

export function previewManagedSessionLogSnapshot(
  sessionId: string,
  requestedLines: number
): ManagedSessionLogSnapshotResponse {
  const session = previewManagedSessions.find((item) => item.id === sessionId);
  if (!session) {
    throw new Error("managed session not found");
  }
  const readCount = nextPreviewSessionLogRead(session.id);
  const snapshot = previewLogSnapshot(session.runId, requestedLines, {
    capturedAt: `preview:${session.id}:${readCount}`,
    text: `Preview log line ${readCount} for ${session.id}\nPreview log line ${readCount} complete\n`
  });
  // Mirror the desktop backend: the managed path adds the redaction notice to
  // the shared warnings list.
  snapshot.warnings.push(
    "Known secrets and control characters were removed automatically. Review before sharing."
  );
  return {
    session: { ...session },
    snapshot
  };
}

export function previewManagedSessionsList(): ManagedSessionList {
  return {
    sessions: previewManagedSessions.map((session) => ({
      ...session,
      nextAction: { ...session.nextAction },
      events: session.events.map((event) => ({ ...event }))
    }))
  };
}

export function previewManagedSessionStatus(
  sessionId: string
): ManagedSessionStatusResponse {
  const session = previewSession(sessionId);
  const active = ["running", "starting", "stop-requested", "kill-requested"].includes(
    session.lifecycle.trim().toLowerCase()
  );
  return {
    session,
    container: active
      ? {
          state: session.lifecycle,
          image: "runhaven/preview:0.1.0",
          startedAt: session.startedAt === "preview" ? null : session.startedAt,
          resources: {
            cpus: "4",
            memoryBytes: 4 * 1024 ** 3
          },
          networks: [
            {
              network: session.network === "provider" ? "runhaven-provider-preview" : null,
              hostname: session.containerName,
              ipv4Address: null,
              ipv4Gateway: null,
              ipv6Address: null
            }
          ]
        }
      : null,
    statusUnavailable: active
      ? null
      : "Container status is unavailable. The run may have stopped.",
    recovery: active
      ? null
      : {
          status: "attention",
          title: "Runtime status unavailable",
          detail:
            "RunHaven can still see this session record, but Apple container status is unavailable.",
          remedy:
            "Refresh sessions or open diagnostics before taking action.",
          repairAvailable: false
        }
  };
}

export function previewManagedSessionDiagnostics(
  sessionId: string
): ManagedSessionDiagnosticsResponse {
  const session = previewSession(sessionId);
  return {
    session,
    authStatus: {
      name: session.agent,
      status: session.authStatus
    },
    authLogEntries: [
      {
        timestamp: "preview",
        profile: session.agent,
        broker: "preview",
        decision: "allowed",
        method: "POST",
        path: "/v1/responses",
        upstreamStatus: 200,
        count: 1,
        reason: "preview",
        runId: session.runId
      }
    ],
    egressLogEntries: [
      {
        timestamp: "preview",
        profile: session.agent,
        decision: "allowed",
        host: "api.openai.com",
        port: 443,
        count: 1,
        reason: "provider-host",
        matchedRule: "api.openai.com",
        runId: session.runId
      }
    ]
  };
}

export function previewManagedStop(sessionId: string): StopManagedSessionResponse {
  const session = previewSession(sessionId);
  const result = {
    runId: session.runId,
    containerName: session.containerName,
    status: "stop-requested"
  };
  if (previewWorktreeSessions.has(sessionId)) {
    finishPreviewWorktreeSession(sessionId);
  } else {
    updatePreviewSessionStatus(sessionId, "stop-requested", "Stop requested");
  }
  return {
    session: previewSession(sessionId),
    result
  };
}

function finishPreviewWorktreeSession(sessionId: string) {
  updatePreviewSessionStatus(sessionId, "finished", "Finished");
  previewManagedSessions = previewManagedSessions.map((session) =>
    session.id === sessionId
      ? {
          ...session,
          cleanup: {
            available: true,
            target: "worktree",
            label: "Discard saved copy",
            detail: "This session saved an isolated copy of your project."
          }
        }
      : session
  );
}

// Merge and discard remove the isolated copy, so the preview session stops
// offering worktree review afterward.
export function resolvePreviewSessionWorktree(sessionId: string) {
  previewManagedSessions = previewManagedSessions.map((session) =>
    session.id === sessionId ? { ...session, cleanup: undefined } : session
  );
}

export function previewManagedKill(sessionId: string): KillManagedSessionResponse {
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

export function previewManagedRepair(sessionId: string): RepairManagedSessionResponse {
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

export function previewManagedCleanup(sessionId: string): CleanupManagedSessionResponse {
  const session = previewSession(sessionId);
  if (session.cleanup?.available !== true) {
    throw new Error("Managed session cleanup is only available for saved worktree sessions.");
  }
  const result = {
    runId: session.runId,
    target: session.cleanup.target || "worktree",
    status: "discarded"
  };
  updatePreviewSessionStatus(sessionId, "cleaned-up", "Cleaned up");
  return {
    session: previewSession(sessionId),
    result
  };
}

export function previewSessionInput(
  sessionId: string,
  input: string
): WriteSessionInputResponse {
  validatePreviewSessionInput(input);
  const session = previewSession(sessionId);
  if (session.nextAction.kind === "auth_attention") {
    throw new Error("Session sign-in is not ready. Finish sign-in before sending input.");
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

export function rememberPreviewManagedSession(
  started: LaunchRunResponse,
  worktree = false
) {
  if (worktree) {
    previewWorktreeSessions.add(started.runId);
  } else {
    previewWorktreeSessions.delete(started.runId);
  }
  const project = projectLabel(started.workspace);
  const session: ManagedSessionSummary = {
    id: started.runId,
    runId: started.runId,
    agent: started.profile,
    session: started.session,
    title: `${agentLabel(started.profile)} - ${project}`,
    project,
    status: "running",
    statusLabel: "Running",
    authStatus: "api-key-broker",
    authScope: started.authScope,
    network: started.networkMode,
    startedAt: "preview",
    containerName: started.containerName,
    detailCommand: `runhaven runs show ${started.runId}`,
    lifecycle: "running",
    input: {
      attached: true,
      status: "attached",
      detail: "Session input is ready in this preview window."
    },
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
        label: "Sign-in",
        detail: "API key broker",
        timestamp: "preview"
      },
      {
        source: "network",
        label: "Network",
        detail: networkLabel(started.networkMode),
        timestamp: "preview"
      }
    ]
  };
  previewManagedSessions = [
    session,
    ...previewManagedSessions.filter((item) => item.runId !== started.runId)
  ];
  previewSessionLogReads.delete(started.runId);
}

function nextPreviewSessionLogRead(sessionId: string): number {
  const nextRead = (previewSessionLogReads.get(sessionId) ?? 0) + 1;
  previewSessionLogReads.set(sessionId, nextRead);
  return nextRead;
}

export function previewSession(sessionId: string): ManagedSessionSummary {
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
