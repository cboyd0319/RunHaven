import { describe, expect, it } from "vitest";

import { loadSessionsUntilRunPresent, sessionIdForRun } from "../app/sessionSelection";
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

  it("waits briefly for a launched run to become a managed session", async () => {
    const loads = [
      sessionList([session("session-old", "run-old")]),
      sessionList([session("session-new", "run-new")])
    ];
    const sleeps: number[] = [];

    const result = await loadSessionsUntilRunPresent(
      async () => loads.shift() ?? sessionList([]),
      "run-new",
      {
        attempts: 3,
        delayMs: 25,
        sleep: async (delayMs) => {
          sleeps.push(delayMs);
        }
      }
    );

    expect(result.sessionId).toBe("session-new");
    expect(result.sessions.sessions.map((item) => item.id)).toEqual(["session-new"]);
    expect(sleeps).toEqual([25]);
  });

  it("returns the last loaded sessions when the launched run never appears", async () => {
    const loads = [
      sessionList([session("session-old", "run-old")]),
      sessionList([session("session-final", "run-final")])
    ];

    const result = await loadSessionsUntilRunPresent(
      async () => loads.shift() ?? sessionList([]),
      "run-new",
      {
        attempts: 2,
        delayMs: 0,
        sleep: async () => {}
      }
    );

    expect(result.sessionId).toBeNull();
    expect(result.sessions.sessions.map((item) => item.id)).toEqual(["session-final"]);
  });
});
