import { describe, expect, it } from "vitest";
import {
  defaultRunPlanRequest,
  getDashboardStatus,
  getImageStatus,
  getLogSnapshot,
  getManagedSessionLogSnapshot,
  getManagedSessions,
  getRunStatus,
  getSetupStatus,
  isLaunchReady,
  killManagedSession,
  launchRun,
  planRun,
  repairManagedSession,
  secureNetworkDefault,
  stopManagedSession,
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

  it("returns managed session preview data without raw paths", async () => {
    const sessions = await getManagedSessions();

    expect(JSON.stringify(sessions)).not.toContain("/Users/");
    expect(JSON.stringify(sessions)).not.toContain("/tmp/runhaven-preview");
  });

  it("returns launched preview sessions as managed session summaries", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    };
    const started = await launchRun({
      plan: request,
      confirmLaunch: true,
      confirmedWarnings: []
    });

    const sessions = await getManagedSessions();
    const session = sessions.sessions.find((item) => item.runId === started.runId);

    expect(session?.project).toBe("runhaven-preview");
    expect(session?.lifecycle).toBe("running");
    expect(session?.nextAction.kind).toBe("monitor");
    expect(session?.events.map((event) => event.source)).toEqual(["run", "auth", "network"]);
    expect(JSON.stringify(session)).not.toContain("/tmp/runhaven-preview");
  });

  it("requires explicit confirmation before loading raw log preview data", async () => {
    await expect(getLogSnapshot("preview-run", { confirmSensitiveOutput: false })).rejects.toThrow(
      "Confirm raw log viewing"
    );

    const snapshot = await getLogSnapshot("preview-run", { confirmSensitiveOutput: true });

    expect(snapshot.runId).toBe("preview-run");
    expect(snapshot.requestedLines).toBe(200);
    expect(snapshot.text).toContain("Preview log line");
    expect(snapshot.truncated).toBe(false);
  });

  it("loads raw log preview data through a managed session id", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    };
    const started = await launchRun({
      plan: request,
      confirmLaunch: true,
      confirmedWarnings: []
    });
    const sessions = await getManagedSessions();
    const session = sessions.sessions.find((item) => item.runId === started.runId);

    expect(session).toBeDefined();
    await expect(
      getManagedSessionLogSnapshot(session!.id, { confirmSensitiveOutput: false })
    ).rejects.toThrow("Confirm raw log viewing");
    await expect(
      getManagedSessionLogSnapshot("missing-session", { confirmSensitiveOutput: true })
    ).rejects.toThrow("managed session not found");

    const response = await getManagedSessionLogSnapshot(session!.id, {
      confirmSensitiveOutput: true
    });

    expect(response.session.id).toBe(session!.id);
    expect(response.cursor).toBe(response.snapshot.cursor);
    expect(response.entries[0]?.sessionId).toBe(session!.id);
    expect(response.entries[0]?.runId).toBe(started.runId);
    expect(response.entries[0]?.cursor).toBe(response.snapshot.cursor);
    expect(response.entries[0]?.redaction.status).toBe("notApplied");
    expect(response.entries[0]?.redaction.message).toContain("not automatically redacted");
    expect(response.snapshot.runId).toBe(started.runId);
    expect(response.snapshot.entries[0]?.text).toContain("Preview log line");
    expect(JSON.stringify(response)).not.toContain("/tmp/runhaven-preview");
    expect(JSON.stringify(response)).not.toContain("write_session_input");
    expect(JSON.stringify(response)).not.toContain("resize_session");
  });

  it("controls preview runs through a managed session id", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-preview"
    };
    const started = await launchRun({
      plan: request,
      confirmLaunch: true,
      confirmedWarnings: []
    });
    const sessions = await getManagedSessions();
    const session = sessions.sessions.find((item) => item.runId === started.runId);

    expect(session).toBeDefined();
    await expect(stopManagedSession("missing-session")).rejects.toThrow(
      "managed session not found"
    );

    const stop = await stopManagedSession(session!.id);
    expect(stop.session.id).toBe(session!.id);
    expect(stop.result.runId).toBe(started.runId);
    expect(stop.result.status).toBe("stop-requested");

    const kill = await killManagedSession(session!.id);
    expect(kill.result.status).toBe("kill-requested");

    const repair = await repairManagedSession(session!.id);
    expect(repair.result.markerRemoved).toBe(true);
    expect(JSON.stringify(repair)).not.toContain("/tmp/runhaven-preview");
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

    expect(isLaunchReady({ warnings }, true, new Set())).toBe(false);
    expect(isLaunchReady({ warnings }, true, new Set(["active-runs", "resource-memory"]))).toBe(true);
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
