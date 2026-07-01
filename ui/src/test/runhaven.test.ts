import { describe, expect, it } from "vitest";
import {
  apiKeyBrokerEnvPlaceholder,
  defaultRunPlanRequest,
  getAuthStatus,
  getDashboardStatus,
  getImageStatus,
  getRunStatus,
  getSetupStatus,
  isLaunchReady,
  launchRun,
  planRun,
  reconcileRunPlanRequest,
  secureNetworkDefault,
  supportsApiKeyBroker,
  warningPreview,
  withLaunchSessionName
} from "../commands/runhaven";

import { localAgent, providerAgent } from "./runhaven-fixtures";

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

  it("keeps API key broker env separate from raw env passthrough", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview",
      apiKeyBrokerEnv: "OPENAI_API_KEY"
    };

    expect(defaultRunPlanRequest(providerAgent).apiKeyBrokerEnv).toBeNull();
    expect(warningPreview(request).map((warning) => warning.code)).toEqual([]);

    const plan = await planRun(request);

    expect(plan.egressSummary).toContain("API key broker enabled");
    expect(plan.access.apiKeyBrokerEnv).toBe("OPENAI_API_KEY");
    expect(plan.access.apiKeyBrokerEnvPresent).toBe(true);
    expect(plan.access.blocking).toBe(false);
  });

  it("defaults GUI launches to shared agent login state", () => {
    const request = defaultRunPlanRequest(providerAgent);

    expect(request.authScope).toBe("agent");
  });

  it("keeps broker fields tied to brokered agents", () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      apiKeyBrokerEnv: "OPENAI_API_KEY"
    };

    expect(supportsApiKeyBroker(providerAgent)).toBe(true);
    expect(supportsApiKeyBroker(localAgent)).toBe(false);
    expect(apiKeyBrokerEnvPlaceholder(providerAgent)).toBe("OPENAI_API_KEY");
    expect(reconcileRunPlanRequest(request, providerAgent).apiKeyBrokerEnv).toBe("OPENAI_API_KEY");
    expect(reconcileRunPlanRequest(request, localAgent).apiKeyBrokerEnv).toBeNull();
  });

  it("previews active-run resource warnings when context has active runs", () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview",
      memory: "4g"
    };

    expect(warningPreview(request, { activeRunCount: 1 }).map((warning) => warning.code)).toEqual([
      "active-runs",
      "resource-memory"
    ]);
  });

  it("adds a fresh GUI session name when the launch request has none", () => {
    const first = withLaunchSessionName({
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    });
    const second = withLaunchSessionName({
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    });
    const named = withLaunchSessionName({
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview",
      sessionName: "custom-session"
    });

    expect(first.sessionName).toMatch(/^gui-[a-z0-9]+-[a-z0-9]+$/);
    expect(second.sessionName).toMatch(/^gui-[a-z0-9]+-[a-z0-9]+$/);
    expect(first.sessionName).not.toBe(second.sessionName);
    expect(named.sessionName).toBe("custom-session");
  });

  it("returns setup preview data outside the Tauri runtime", async () => {
    const setup = await getSetupStatus();

    expect(setup.ok).toBe(true);
    expect(setup.checks[0]?.name).toBe("Preview mode");
  });

  it("returns dashboard preview data outside the Tauri runtime", async () => {
    const dashboard = await getDashboardStatus();

    expect(dashboard.agents.map((agent) => agent.name)).toContain("codex");
    expect(dashboard.agents.find((agent) => agent.name === "codex")?.signIn).toBe("runhaven login");
    expect(dashboard.agents.find((agent) => agent.name === "shell")?.broker).toBe("n/a");
    expect(dashboard.warnings[0]).toContain("Desktop runtime commands are unavailable");
  });

  it("returns brokered auth status preview data", async () => {
    const authStatus = await getAuthStatus();

    expect(authStatus.profiles).toEqual(
      expect.arrayContaining([
        { name: "claude", status: "api-key-broker" },
        { name: "codex", status: "api-key-broker" },
        { name: "gemini", status: "api-key-broker" }
      ])
    );
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
    expect(plan.authScope).toBe("agent");
    expect(plan.access.detail).toContain("across projects on this Mac");
    expect(plan.warnings.map((warning) => warning.code)).toEqual(["full-internet"]);
  });

  it("returns image readiness preview data outside the Tauri runtime", async () => {
    const status = await getImageStatus("codex");

    expect(status.agent).toBe("codex");
    expect(status.image.ready).toBe(true);
    expect(status.image.status).toBe("ok");
    expect(status.builder.status).toBe("preview");
  });

  it("returns run status preview data outside the Tauri runtime", async () => {
    const status = await getRunStatus("preview-run");

    expect(status.run.runId).toBe("preview-run");
    expect(status.container.state).toBe("running");
    expect(status.container.resources.memoryBytes).toBe(4 * 1024 ** 3);
  });

  it("requires explicit confirmation before launch is available", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    };
    const plan = await planRun(request);

    expect(isLaunchReady(plan, false, new Set())).toBe(false);
    expect(isLaunchReady(plan, true, new Set())).toBe(true);
  });

  it("blocks launch when the bundled image is not ready", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    };
    const plan = await planRun(request);

    expect(isLaunchReady(plan, true, new Set(), false)).toBe(false);
  });

  it("requires every warning to be acknowledged before launch is available", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview",
      networkMode: "internet" as const
    };
    const plan = await planRun(request);

    expect(isLaunchReady(plan, true, new Set())).toBe(false);
    expect(isLaunchReady(plan, true, new Set(["full-internet"]))).toBe(true);
  });

  it("requires active-run warnings to be acknowledged before launch is available", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    };
    const warnings = warningPreview(request, { activeRunCount: 1 });

    expect(isLaunchReady(planWithWarnings(warnings), true, new Set())).toBe(false);
    expect(isLaunchReady(planWithWarnings(warnings), true, new Set(["active-runs", "resource-memory"]))).toBe(true);
  });

  it("blocks launch when plan access is blocked", () => {
    const warnings: Array<{ code: string; message: string }> = [];

    expect(isLaunchReady(planWithWarnings(warnings, true), true, new Set())).toBe(false);
  });

  it("returns a launch preview outside the Tauri runtime", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    };

    const started = await launchRun({
      plan: request,
      confirmLaunch: true,
      confirmedWarnings: []
    });

    expect(started.runId).toMatch(/^preview-/);
    expect(started.status).toBe("started");
    expect(started.snapshot.runId).toBe(started.runId);
    expect(started.snapshot.containerName).toBe("preview");
  });
});

function planWithWarnings(warnings: Array<{ code: string; message: string }>, blocking = false) {
  return {
    warnings,
    access: {
      status: blocking ? "blocked" : "setup",
      label: blocking ? "API key variable missing" : "Shared agent login",
      detail: "",
      remedy: "",
      blocking,
      apiKeyBrokerEnv: null,
      apiKeyBrokerEnvPresent: null
    }
  };
}
