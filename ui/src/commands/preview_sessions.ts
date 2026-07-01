import { previewLogSnapshot } from "./preview_output";
import type {
  KillManagedSessionResponse,
  LaunchRunResponse,
  ManagedSessionList,
  ManagedSessionLogSnapshotResponse,
  ManagedSessionSummary,
  RepairManagedSessionResponse,
  SessionLogEntry,
  StopManagedSessionResponse,
  WriteSessionInputResponse
} from "./types";

let previewManagedSessions: ManagedSessionSummary[] = [];
let previewSessionLogReads = new Map<string, number>();
const MAX_SESSION_INPUT_BYTES = 8 * 1024;

export function previewManagedSessionLogSnapshot(
  sessionId: string,
  requestedLines: number
): ManagedSessionLogSnapshotResponse {
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

export function previewManagedSessionsList(): ManagedSessionList {
  return {
    sessions: previewManagedSessions.map((session) => ({
      ...session,
      nextAction: { ...session.nextAction },
      events: session.events.map((event) => ({ ...event }))
    }))
  };
}

export function previewManagedStop(sessionId: string): StopManagedSessionResponse {
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

export function previewSessionInput(
  sessionId: string,
  input: string
): WriteSessionInputResponse {
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

export function rememberPreviewManagedSession(snapshot: LaunchRunResponse["snapshot"]) {
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

function sessionLogEntries(
  session: ManagedSessionSummary,
  snapshot: ReturnType<typeof previewLogSnapshot>
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
