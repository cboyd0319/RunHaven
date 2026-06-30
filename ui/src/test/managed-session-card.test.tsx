import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import {
  ManagedSessionCard,
  sessionNeedsAttention
} from "../app/ManagedSessionCard";
import type { ManagedSessionSummary } from "../commands/runhaven";

function session(patch: Partial<ManagedSessionSummary> = {}): ManagedSessionSummary {
  return {
    id: "run-123",
    runId: "run-123",
    agent: "codex",
    session: "project",
    title: "Codex - project",
    project: "project",
    status: "running",
    statusLabel: "Running",
    authStatus: "api-key-broker",
    network: "provider",
    startedAt: "2026-06-30T02:00:00Z",
    containerName: "runhaven-codex-project",
    detailCommand: "runhaven runs show run-123",
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
        timestamp: "2026-06-30T02:00:00Z"
      },
      {
        source: "auth",
        label: "Auth",
        detail: "API key broker",
        timestamp: "2026-06-30T02:00:00Z"
      },
      {
        source: "network",
        label: "Network",
        detail: "Provider only",
        timestamp: "2026-06-30T02:00:00Z"
      }
    ],
    ...patch
  };
}

describe("ManagedSessionCard", () => {
  it("renders a read-only selected-session summary without host paths", () => {
    const html = renderToStaticMarkup(
      <ManagedSessionCard session={session({ project: "secret-project" })} />
    );

    expect(html).toContain("Codex - project");
    expect(html).toContain("secret-project");
    expect(html).toContain("Provider only");
    expect(html).toContain("RunHaven is monitoring this session.");
    expect(html).toContain("Monitor session");
    expect(html).toContain("Run record");
    expect(html).not.toContain("/Users/");
    expect(html).not.toContain("/sensitive/");
    expect(html).not.toContain("state_volume");
    expect(html).not.toContain("write_session_input");
  });

  it("shows auth attention when the broker state is not ready", () => {
    const blocked = session({
      authStatus: "unknown",
      nextAction: {
        kind: "auth_attention",
        label: "Finish auth setup",
        detail: "Auth is not ready for Codex."
      },
      events: [
        {
          source: "auth",
          label: "Auth",
          detail: "Unknown",
          timestamp: "2026-06-30T02:00:00Z"
        }
      ]
    });

    expect(sessionNeedsAttention(blocked)).toBe(true);

    const html = renderToStaticMarkup(<ManagedSessionCard session={blocked} />);

    expect(html).toContain("Finish auth setup");
    expect(html).toContain("Auth is not ready for Codex.");
    expect(html).not.toContain("Start session");
  });
});
