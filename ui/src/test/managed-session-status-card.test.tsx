import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { ManagedSessionStatusCard } from "../app/ManagedSessionStatusCard";
import { sameManagedSessionStatus } from "../app/useManagedSessionStatus";
import type {
  ManagedSessionStatusResponse,
  ManagedSessionSummary
} from "../commands/runhaven";

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
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-01T02:00:00Z",
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
    events: [],
    ...patch
  };
}

function status(
  patch: Partial<ManagedSessionStatusResponse> = {}
): ManagedSessionStatusResponse {
  return {
    session: session(),
    container: {
      state: "running",
      image: "runhaven/codex:0.7.0",
      startedAt: "2026-07-01T02:00:03Z",
      resources: {
        cpus: "4",
        memoryBytes: 4 * 1024 ** 3
      },
      networks: [
        {
          network: "runhaven-provider-preview",
          hostname: "runhaven-codex-project",
          ipv4Address: "192.168.64.20/24",
          ipv4Gateway: "192.168.64.1",
          ipv6Address: null
        }
      ]
    },
    statusUnavailable: null,
    recovery: null,
    ...patch
  };
}

describe("ManagedSessionStatusCard", () => {
  it("renders selected-session runtime status without raw paths", () => {
    const html = renderToStaticMarkup(
      <ManagedSessionStatusCard status={status()} loading={false} error="" />
    );

    expect(html).toContain("Runtime status");
    expect(html).toContain("Container");
    expect(html).toContain("Current state for this session.");
    expect(html).toContain("runhaven/codex:0.7.0");
    expect(html).toContain("4096 MiB");
    expect(html).toContain("runhaven-provider-preview");
    expect(html).not.toContain("/Users/");
    expect(html).not.toContain("/tmp/runhaven-status-preview");
    expect(html).not.toContain("stateVolume");
    expect(html).not.toContain("write_session_input");
    expect(html).not.toContain("attach_session");
  });

  it("shows a bounded unavailable state", () => {
    const html = renderToStaticMarkup(
      <ManagedSessionStatusCard
        status={status({
          container: null,
          statusUnavailable:
            "Container status is unavailable. The run may have stopped.",
          recovery: {
            status: "attention",
            title: "Runtime status unavailable",
            detail:
              "RunHaven can still see this session record, but Apple container status is unavailable.",
            remedy:
              "Refresh sessions or open diagnostics before taking action.",
            repairAvailable: false
          }
        })}
        loading={false}
        error=""
      />
    );

    expect(html).toContain("Live status not available");
    expect(html).toContain("may have stopped");
    expect(html).toContain("Runtime status unavailable");
    expect(html).toContain("Refresh sessions");
    expect(html).not.toContain("Repair marker");
    expect(html).not.toContain("Repair available");
    expect(html).not.toContain("container inspect");
  });

  it("treats recovery guidance changes as status changes", () => {
    const current = status({
      container: null,
      statusUnavailable: "Container status is unavailable. The run may have stopped.",
      recovery: {
        status: "attention",
        title: "Runtime status unavailable",
        detail:
          "RunHaven can still see this session record, but Apple container status is unavailable.",
        remedy: "Refresh sessions or open diagnostics before taking action.",
        repairAvailable: false
      }
    });
    const next = {
      ...current,
      recovery: {
        ...current.recovery!,
        remedy: "Open diagnostics before taking action."
      }
    };

    expect(sameManagedSessionStatus(current, next)).toBe(false);
  });
});
