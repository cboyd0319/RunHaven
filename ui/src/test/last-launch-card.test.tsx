import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { LastLaunchCard } from "../app/LastLaunchCard";
import type { LaunchRunResponse } from "../commands/runhaven";

// A launch response carrying a host workspace path and a state-volume name that
// the card must not render. Only sanitized display fields belong on screen.
function lastLaunch(patch: Partial<LaunchRunResponse> = {}): LaunchRunResponse {
  return {
    runId: "run-123",
    status: "started",
    profile: "codex",
    workspace: "/Users/secret/project",
    stateVolume: "runhaven-state-secret",
    authScope: "project",
    session: "project",
    networkMode: "provider",
    snapshot: {
      runId: "run-123",
      status: "started",
      profile: "codex",
      workspace: "/Users/secret/project",
      stateVolume: "runhaven-state-secret",
      authScope: "project",
      session: "project",
      networkMode: "provider",
      containerName: "runhaven-codex-project"
    },
    ...patch
  };
}

describe("LastLaunchCard", () => {
  it("renders sanitized launch metrics without host paths or state volume", () => {
    const html = renderToStaticMarkup(
      <LastLaunchCard lastLaunch={lastLaunch()} />
    );

    expect(html).toContain("Last launch");
    expect(html).toContain("run-123");
    expect(html).toContain("codex");
    expect(html).toContain("provider");
    expect(html).toContain("runhaven-codex-project");
    // Workspace path and state volume travel in the response but never render.
    expect(html).not.toContain("/Users/");
    expect(html).not.toContain("runhaven-state-secret");
  });

  it("maps the agent auth scope to a plain-language login label", () => {
    const html = renderToStaticMarkup(
      <LastLaunchCard lastLaunch={lastLaunch({ authScope: "agent" })} />
    );

    expect(html).toContain("Shared agent login");
    expect(html).not.toContain("Project session");
  });

  it("maps the project auth scope to a plain-language login label", () => {
    const html = renderToStaticMarkup(
      <LastLaunchCard lastLaunch={lastLaunch({ authScope: "project" })} />
    );

    expect(html).toContain("Project session");
    expect(html).not.toContain("Shared agent login");
  });

  it("falls back to Unknown for an empty auth scope", () => {
    const html = renderToStaticMarkup(
      <LastLaunchCard lastLaunch={lastLaunch({ authScope: "  " })} />
    );

    expect(html).toContain("Unknown");
  });
});
