import { describe, expect, it } from "vitest";

import { sessionIdForRun } from "../app/sessionSelection";
import type { ManagedSessionList, ManagedSessionSummary } from "../commands/runhaven";

function session(id: string, runId: string): ManagedSessionSummary {
  return {
    id,
    runId,
    agent: "codex",
    session: "project",
    title: "Codex - project",
    project: "project",
    status: "running",
    statusLabel: "Running",
    authStatus: "ready",
    authScope: "project",
    network: "provider",
    startedAt: "2026-06-30T02:00:00Z",
    containerName: `runhaven-${id}`,
    detailCommand: `runhaven runs show ${runId}`,
    lifecycle: "running",
    input: {
      attached: true,
      status: "attached",
      detail: "Session input is ready."
    },
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking this session."
    },
    events: []
  };
}

function sessionList(sessions: ManagedSessionSummary[]): ManagedSessionList {
  return { sessions };
}

describe("sessionIdForRun", () => {
  it("selects the managed session owned by the launched run id", () => {
    const sessions = sessionList([
      session("session-old", "run-old"),
      session("session-new", "run-new")
    ]);

    expect(sessionIdForRun(sessions, "run-new")).toBe("session-new");
  });

  it("does not select a session when only display fields match", () => {
    const sessions = sessionList([session("session-collision", "run-other")]);

    expect(sessionIdForRun(sessions, "run-new")).toBeNull();
  });

});
