import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { ManagedSessionDiagnosticsCard } from "../app/ManagedSessionDiagnosticsCard";
import type { ManagedSessionDiagnosticsResponse } from "../commands/runhaven";

function diagnostics(): ManagedSessionDiagnosticsResponse {
  return {
    session: {
      id: "run-123",
      runId: "run-123",
      agent: "codex",
      session: "project",
      title: "Codex - project",
      project: "project",
      status: "running",
      statusLabel: "Running",
      authStatus: "api-key-broker",
      authScope: "project",
      network: "provider",
      startedAt: "2026-07-01T03:00:00Z",
      containerName: "runhaven-codex-project",
      detailCommand: "runhaven runs show run-123",
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
    },
    authStatus: {
      name: "codex",
      status: "api-key-broker"
    },
    authLogEntries: [
      {
        timestamp: "2026-07-01T03:00:01Z",
        profile: "codex",
        broker: "codex-api-key",
        decision: "allowed",
        method: "POST",
        path: "/v1/responses",
        upstreamStatus: 200,
        count: 1,
        reason: "ok",
        runId: "run-123"
      }
    ],
    egressLogEntries: [
      {
        timestamp: "2026-07-01T03:00:02Z",
        profile: "codex",
        decision: "allowed",
        host: "api.openai.com",
        port: 443,
        count: 1,
        reason: "provider-host",
        matchedRule: "api.openai.com",
        runId: "run-123"
      }
    ]
  };
}

describe("ManagedSessionDiagnosticsCard", () => {
  it("renders session-scoped diagnostics without raw paths", () => {
    const html = renderToStaticMarkup(
      <ManagedSessionDiagnosticsCard
        diagnostics={diagnostics()}
        loading={false}
        error=""
        onLoad={async () => {}}
      />
    );

    expect(html).toContain("Session diagnostics");
    expect(html).toContain("Sign-in method: API key from this Mac");
    expect(html).toContain("Sign-in records");
    expect(html).toContain(">Result<");
    expect(html).not.toContain("Auth records");
    expect(html).not.toContain(">Decision<");
    expect(html).toContain(">Codex<");
    expect(html).not.toContain(">codex<");
    expect(html).toContain("/v1/responses");
    expect(html).toContain("api.openai.com");
    expect(html).toContain("Allowed because this is a service Codex needs.");
    expect(html).not.toContain(">allowed<");
    expect(html).not.toContain(">provider-host<");
    expect(html).not.toContain("/Users/");
    expect(html).not.toContain("/tmp/");
    expect(html).not.toContain("token=");
    expect(html).not.toContain("stateVolume");
  });
});
