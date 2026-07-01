import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { DiagnosticsCard } from "../app/DiagnosticsCard";
import type { DiagnosticsState } from "../app/types";

// The auth tab is the default view. It renders profile status but deliberately
// omits the broker request path/method, so the auth log below carries a real
// request path the card must never surface.
function diagnostics(patch: Partial<DiagnosticsState> = {}): DiagnosticsState {
  return {
    authStatus: {
      status: "ready",
      runtime: "broker",
      profiles: [{ name: "codex", status: "api-key-broker" }]
    },
    authLog: {
      entries: [
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
      ]
    },
    egressLog: {
      entries: [
        {
          timestamp: "2026-07-01T03:00:02Z",
          profile: "codex",
          decision: "allowed",
          host: "api.openai.com",
          port: 443,
          count: 1,
          reason: "provider-host-secret-rule",
          matchedRule: "internal-rule-secret",
          runId: "run-123"
        }
      ]
    },
    ...patch
  };
}

const handlers = { onLoad: async () => {} };

describe("DiagnosticsCard", () => {
  it("prompts to load diagnostics before any data arrives", () => {
    const html = renderToStaticMarkup(
      <DiagnosticsCard diagnostics={null} loading={false} error="" {...handlers} />
    );

    expect(html).toContain("Trust checks");
    expect(html).toContain("Load diagnostics");
    expect(html).not.toContain("Loading diagnostics...");
    expect(html).not.toContain("Runtime:");
    expect(html).not.toContain("Diagnostics unavailable");
  });

  it("shows a loading label on the load button", () => {
    const html = renderToStaticMarkup(
      <DiagnosticsCard diagnostics={null} loading={true} error="" {...handlers} />
    );

    expect(html).toContain("Loading diagnostics...");
  });

  it("surfaces a bounded error without any data", () => {
    const html = renderToStaticMarkup(
      <DiagnosticsCard
        diagnostics={null}
        loading={false}
        error="Diagnostics command failed."
        {...handlers}
      />
    );

    expect(html).toContain("Diagnostics unavailable");
    expect(html).toContain("Diagnostics command failed.");
  });

  it("renders auth broker status without leaking request paths or rules", () => {
    const html = renderToStaticMarkup(
      <DiagnosticsCard
        diagnostics={diagnostics()}
        loading={false}
        error=""
        {...handlers}
      />
    );

    expect(html).toContain("Auth broker status");
    expect(html).toContain("Runtime: broker");
    expect(html).toContain("codex");
    expect(html).toContain("api-key-broker");
    // Auth log entries carry a request path and egress carries internal rule
    // reasons; the trust-check card renders neither.
    expect(html).not.toContain("/v1/responses");
    expect(html).not.toContain("internal-rule-secret");
    expect(html).not.toContain("provider-host-secret-rule");
    expect(html).not.toContain("/Users/");
    // Entries exist, so the empty-state copy must not appear.
    expect(html).not.toContain("No auth broker decisions recorded.");
  });

  it("shows the empty auth-log state when nothing is recorded", () => {
    const html = renderToStaticMarkup(
      <DiagnosticsCard
        diagnostics={diagnostics({ authLog: { entries: [] } })}
        loading={false}
        error=""
        {...handlers}
      />
    );

    expect(html).toContain("No auth broker decisions recorded.");
  });
});
