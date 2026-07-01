import { describe, expect, it } from "vitest";

import {
  decisionBadgeVariant,
  decisionLabel,
  hostLists,
  hostRows,
  networkRows,
  stateRows,
  workspaceRows,
} from "../app/whyExplain";
import type {
  HostExplanationResponse,
  NetworkExplanationResponse,
  StateExplanationResponse,
  WorkspaceExplanationResponse,
} from "../commands/runhaven";

describe("decisionLabel", () => {
  it("maps known verdicts to plain words", () => {
    expect(decisionLabel("allowed")).toBe("Allowed");
    expect(decisionLabel("not-allowed")).toBe("Not allowed");
    expect(decisionLabel("denied")).toBe("Blocked");
  });

  it("treats an empty verdict as unknown", () => {
    expect(decisionLabel("")).toBe("Unknown");
  });

  it("passes an unrecognized verdict through", () => {
    expect(decisionLabel("custom")).toBe("custom");
  });
});

describe("decisionBadgeVariant", () => {
  it("uses success, warning, and destructive for the known verdicts", () => {
    expect(decisionBadgeVariant("allowed")).toBe("success");
    expect(decisionBadgeVariant("not-allowed")).toBe("warning");
    expect(decisionBadgeVariant("denied")).toBe("destructive");
    expect(decisionBadgeVariant("other")).toBe("secondary");
  });
});

describe("networkRows", () => {
  it("drops empty rows so only present fields show", () => {
    const response: NetworkExplanationResponse = {
      mode: "provider",
      behavior: "routes egress through the proxy.",
      boundary: "only bundled hosts are eligible.",
      useWhen: "",
      dnsSafety: "IP literals are denied.",
      inspect: "use why host.",
    };
    const rows = networkRows(response);
    expect(rows.map((row) => row.label)).toEqual([
      "What it does",
      "Boundary",
      "DNS safety",
      "Inspect",
    ]);
  });
});

describe("stateRows", () => {
  it("includes the volume pattern without any host path", () => {
    const response: StateExplanationResponse = {
      profile: "claude",
      defaultSession: "default",
      homeMount: "/home/agent",
      stateVolumePattern: "runhaven-claude-<project-id>-home",
      namedSessionPattern: "runhaven-claude-<project-id>-s-<session>-<digest>-home",
      boundary: "workspace files are mounted separately at /workspace.",
      projectId: "derived from the workspace path; the path is not stored.",
      manage: "runhaven state list.",
    };
    const rows = stateRows(response);
    const volume = rows.find((row) => row.label === "State volume");
    expect(volume?.value).toContain("<project-id>");
  });
});

describe("host helpers", () => {
  function host(overrides: Partial<HostExplanationResponse> = {}): HostExplanationResponse {
    return {
      host: "api.example.com",
      port: 443,
      decision: "not-allowed",
      reason: "",
      nextAction: "review before adding it.",
      agent: "codex",
      profile: "codex",
      matchedRules: [],
      bundledHosts: ["api.openai.com"],
      endpointNotes: [],
      dnsSafety: "checked at runtime.",
      ...overrides,
    };
  }

  it("keeps only non-empty rows", () => {
    const rows = hostRows(host());
    expect(rows.map((row) => row.label)).toEqual(["Next step", "DNS safety"]);
  });

  it("keeps only non-empty lists", () => {
    const lists = hostLists(host());
    expect(lists.map((list) => list.label)).toEqual(["Bundled hosts"]);
  });
});

describe("workspaceRows", () => {
  it("shows scope and boundary but never a raw path", () => {
    const response: WorkspaceExplanationResponse = {
      scope: "current",
      decision: "allowed",
      verdict: "RunHaven can mount this folder safely.",
      nextAction: "",
      boundary: "only the mounted path is exposed at /workspace.",
      sensitiveOverrideUsed: false,
    };
    const rows = workspaceRows(response);
    expect(rows.map((row) => row.label)).toEqual(["Scope", "Boundary"]);
  });
});
