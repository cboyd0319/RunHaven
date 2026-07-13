import { describe, expect, it } from "vitest";
import {
  apiKeyBrokerEnvForAgent,
  defaultRunPlanRequest,
  getAuthStatus,
  getDashboardStatus,
  getImageStatus,
  performSetupRecovery,
  getRunStatus,
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

    expect(plan.egressSummary).toBe(
      "The assistant can reach only the services it needs.",
    );
    expect(plan.access.apiKeyBrokerEnv).toBe("OPENAI_API_KEY");
    expect(plan.access.apiKeyBrokerEnvPresent).toBe(true);
    expect(plan.access.blocking).toBe(false);
  });

  it("uses fixed API key broker env names by agent", () => {
    expect(apiKeyBrokerEnvForAgent({ ...providerAgent, name: "codex" })).toBe("OPENAI_API_KEY");
    expect(apiKeyBrokerEnvForAgent({ ...providerAgent, name: "claude" })).toBe("ANTHROPIC_API_KEY");
    expect(apiKeyBrokerEnvForAgent({ ...providerAgent, name: "gemini" })).toBeNull();
    expect(apiKeyBrokerEnvForAgent({ ...providerAgent, name: "other" })).toBeNull();
    expect(supportsApiKeyBroker({ ...providerAgent, name: "other" })).toBe(false);
  });

  it("defaults GUI launches to the shared agent sign-in", () => {
    const request = defaultRunPlanRequest(providerAgent);

    expect(request.authScope).toBe("agent");
  });

  it("defaults GUI launches to an isolated project copy", () => {
    expect(defaultRunPlanRequest(providerAgent).worktree).toBe(true);
  });

  it("warns only when direct mode can change original project files", () => {
    const direct = {
      ...defaultRunPlanRequest(providerAgent),
      worktree: false,
    };

    expect(warningPreview(direct).map((warning) => warning.code)).toContain(
      "direct-workspace",
    );
    expect(
      warningPreview({ ...direct, readOnlyWorkspace: true }).map(
        (warning) => warning.code,
      ),
    ).not.toContain("direct-workspace");
  });

  it("keeps broker fields tied to brokered agents", () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      apiKeyBrokerEnv: "CUSTOM_API_KEY"
    };

    expect(supportsApiKeyBroker(providerAgent)).toBe(true);
    expect(supportsApiKeyBroker(localAgent)).toBe(false);
    expect(apiKeyBrokerEnvForAgent(providerAgent)).toBe("OPENAI_API_KEY");
    expect(reconcileRunPlanRequest(request, providerAgent).apiKeyBrokerEnv).toBe("OPENAI_API_KEY");
    expect(reconcileRunPlanRequest(defaultRunPlanRequest(providerAgent), providerAgent).apiKeyBrokerEnv).toBeNull();
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

  it("returns dashboard preview data outside the Tauri runtime", async () => {
    const dashboard = await getDashboardStatus();

    expect(dashboard.agents.map((agent) => agent.name)).toEqual([
      "claude",
      "codex",
      "antigravity",
      "copilot"
    ]);
    expect(dashboard.agents.find((agent) => agent.name === "codex")?.signIn).toBe("runhaven login");
    expect(dashboard.agents.find((agent) => agent.name === "copilot")?.broker).toBe("no");
    expect(dashboard.warnings[0]).toContain("Desktop runtime commands are unavailable");
  });

  it("keeps setup recovery on an opaque action id in preview", async () => {
    const result = await performSetupRecovery("start-container-system");

    expect(result.actionId).toBe("start-container-system");
    expect(result.detail).not.toContain("container system start");
  });

  it("returns brokered auth status preview data", async () => {
    const authStatus = await getAuthStatus();

    expect(authStatus.profiles).toEqual(
      expect.arrayContaining([
        { name: "claude", status: "api-key-broker" },
        { name: "codex", status: "api-key-broker" }
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
    expect(plan.access.label).toBe("Shared assistant sign-in")
    expect(plan.access.remedy).toContain("If the assistant asks you to sign in")
    expect(plan.egressSummary).toBe("This session can reach any site.")
    expect(plan.warnings.map((warning) => warning.code)).toEqual(["full-internet"]);
  });

  it("uses plain protected-copy and constrained-network preview text", async () => {
    const plan = await planRun({
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview",
    })

    expect(plan.worktreeNote).toBe(
      "RunHaven will work in a protected copy. Your original folder changes only if you choose Apply changes later.",
    )
    expect(plan.egressSummary).toBe(
      "The assistant can reach only the services it needs.",
    )
  })

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

  it("returns a launch preview outside the Tauri runtime", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    };

    const started = await launchRun({
      plan: request,
    });

    expect(started).not.toBeNull();
    if (!started) throw new Error("preview launch was unexpectedly cancelled");
    expect(started.runId).toMatch(/^preview-/);
    expect(started.status).toBe("started");
    expect(started.containerName).toBe("preview");
  });
});
