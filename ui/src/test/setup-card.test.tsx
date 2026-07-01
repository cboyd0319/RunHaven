import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { SetupCard } from "../app/SetupCard";
import type { CheckStatus, DashboardStatus } from "../commands/runhaven";

function check(patch: Partial<CheckStatus> = {}): CheckStatus {
  return {
    name: "Container CLI",
    ok: true,
    detail: "Apple container is installed.",
    remedy: "No action needed.",
    ...patch
  };
}

// A dashboard whose non-setup fields carry host paths and a state volume that
// SetupCard must never render. These stay realistic (RunSummary.workspace is a
// host path) so the secret-safety assertions bite.
function dashboard(patch: Partial<DashboardStatus> = {}): DashboardStatus {
  return {
    setup: {
      ok: true,
      checks: [check()],
      blockerCount: 0
    },
    agents: [],
    activeRuns: [
      {
        runId: "run-9",
        profile: "codex",
        workspace: "/Users/secret/project",
        network: "provider",
        status: "running",
        timestamp: "2026-07-01T00:00:00Z",
        authScope: "project",
        session: "project"
      }
    ],
    recentRuns: [
      {
        runId: "run-8",
        profile: "codex",
        workspace: "/Users/secret/other",
        network: "provider",
        status: "finished",
        timestamp: "2026-06-30T00:00:00Z",
        authScope: "project",
        session: "other"
      }
    ],
    warnings: [],
    ...patch
  };
}

describe("SetupCard", () => {
  it("shows a checking state while loading", () => {
    const html = renderToStaticMarkup(
      <SetupCard loading={true} dashboard={null} />
    );

    expect(html).toContain("Finish setup");
    expect(html).toContain("Checking setup...");
    expect(html).toContain("Loading setup...");
    expect(html).toContain(">Checking</");
    expect(html).not.toContain("Refresh status to try again.");
  });

  it("shows an unknown state when setup status could not be read", () => {
    const html = renderToStaticMarkup(
      <SetupCard loading={false} dashboard={null} />
    );

    expect(html).toContain("RunHaven could not read setup status.");
    expect(html).toContain("Refresh status to try again.");
    expect(html).toContain(">Unknown</");
    expect(html).not.toContain("All checks passed.");
  });

  it("confirms setup readiness without leaking other-run host paths", () => {
    const html = renderToStaticMarkup(
      <SetupCard loading={false} dashboard={dashboard()} />
    );

    expect(html).toContain("Setup ready");
    expect(html).toContain("Ready to launch.");
    expect(html).toContain("All checks passed.");
    expect(html).toContain(">Ready</");
    expect(html).not.toContain("Needs attention");
    // Active and recent run workspaces travel in the dashboard, but this card
    // renders only setup, so those host paths never reach the screen.
    expect(html).not.toContain("/Users/");
  });

  it("surfaces the first failed check with detail and remedy", () => {
    const html = renderToStaticMarkup(
      <SetupCard
        loading={false}
        dashboard={dashboard({
          setup: {
            ok: false,
            blockerCount: 1,
            checks: [
              check({
                name: "Container CLI",
                ok: false,
                detail: "Apple container is not installed.",
                remedy: "Install Apple container and retry."
              })
            ]
          }
        })}
      />
    );

    expect(html).toContain("Finish setup");
    expect(html).toContain(">Blocked</");
    expect(html).toContain("Needs attention");
    expect(html).toContain("Container CLI");
    expect(html).toContain("Apple container is not installed.");
    expect(html).toContain("Install Apple container and retry.");
    expect(html).not.toContain("All checks passed.");
  });

  it("summarizes remaining failures with singular and plural wording", () => {
    const twoFailures = renderToStaticMarkup(
      <SetupCard
        loading={false}
        dashboard={dashboard({
          setup: {
            ok: false,
            blockerCount: 2,
            checks: [
              check({ name: "Container CLI", ok: false }),
              check({ name: "Image cache", ok: false })
            ]
          }
        })}
      />
    );
    expect(twoFailures).toContain("1 more setup check needs");

    const threeFailures = renderToStaticMarkup(
      <SetupCard
        loading={false}
        dashboard={dashboard({
          setup: {
            ok: false,
            blockerCount: 3,
            checks: [
              check({ name: "Container CLI", ok: false }),
              check({ name: "Image cache", ok: false }),
              check({ name: "Network", ok: false })
            ]
          }
        })}
      />
    );
    expect(threeFailures).toContain("2 more setup checks need");
  });
});
