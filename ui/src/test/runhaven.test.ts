import { describe, expect, it } from "vitest";
import {
  defaultRunPlanRequest,
  getDashboardStatus,
  getSetupStatus,
  planRun,
  secureNetworkDefault,
  warningPreview,
  type AgentProfile
} from "../commands/runhaven";

const providerAgent: AgentProfile = {
  name: "codex",
  description: "Codex",
  image: "runhaven/codex:0.1.0",
  defaultCommand: ["codex"],
  providerHosts: ["api.openai.com"]
};

const localAgent: AgentProfile = {
  name: "shell",
  description: "Shell",
  image: "runhaven/base:0.1.0",
  defaultCommand: ["/bin/bash"],
  providerHosts: []
};

describe("runhaven command helpers", () => {
  it("chooses provider-only when reviewed provider hosts exist", () => {
    expect(secureNetworkDefault(providerAgent)).toBe("provider");
  });

  it("chooses local-only when no provider hosts exist", () => {
    expect(secureNetworkDefault(localAgent)).toBe("internal");
  });

  it("keeps advanced supported choices as warnings", () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      networkMode: "internet" as const,
      allowSensitiveWorkspace: true,
      envNames: ["OPENAI_API_KEY"],
      image: "example/custom:1.0.0",
      providerHosts: ["example.com"]
    };
    expect(warningPreview(request).map((warning) => warning.code)).toEqual([
      "full-internet",
      "sensitive-workspace",
      "environment",
      "custom-image",
      "provider-host"
    ]);
  });

  it("returns setup preview data outside the Tauri runtime", async () => {
    const setup = await getSetupStatus();

    expect(setup.ok).toBe(true);
    expect(setup.checks[0]?.name).toBe("Preview mode");
  });

  it("returns dashboard preview data outside the Tauri runtime", async () => {
    const dashboard = await getDashboardStatus();

    expect(dashboard.agents.map((agent) => agent.name)).toContain("codex");
    expect(dashboard.warnings[0]).toContain("Desktop runtime commands are unavailable");
  });

  it("returns a read-only launch plan preview outside the Tauri runtime", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview",
      networkMode: "internet" as const
    };

    const plan = await planRun(request);

    expect(plan.workspace).toBe("/tmp/runhaven-preview");
    expect(plan.networkMode).toBe("internet");
    expect(plan.warnings.map((warning) => warning.code)).toEqual(["full-internet"]);
  });
});
