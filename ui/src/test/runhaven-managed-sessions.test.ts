import { describe, expect, it } from "vitest";
import {
  defaultRunPlanRequest,
  getLogSnapshot,
  getManagedSessionDiagnostics,
  getManagedSessionLogSnapshot,
  getManagedSessionStatus,
  getManagedSessions,
  killManagedSession,
  launchRun,
  repairManagedSession,
  stopManagedSession,
  writeSessionInput
} from "../commands/runhaven";

import { providerAgent } from "./runhaven-fixtures";

describe("runhaven managed session helpers", () => {
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

  it("returns selected managed session runtime status without raw paths", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-status-preview",
      sessionName: "status"
    };
    const started = await launchRun({
      plan: request,
      confirmLaunch: true,
      confirmedWarnings: []
    });
    const sessions = await getManagedSessions();
    const session = sessions.sessions.find((item) => item.runId === started.runId);

    expect(session).toBeDefined();
    await expect(getManagedSessionStatus("missing-session")).rejects.toThrow(
      "managed session not found"
    );

    const status = await getManagedSessionStatus(session!.id);

    expect(status.session.id).toBe(session!.id);
    expect(status.session.runId).toBe(started.runId);
    expect(status.container?.state).toBe("running");
    expect(status.container?.resources.memoryBytes).toBe(4 * 1024 ** 3);
    expect(status.statusUnavailable).toBeNull();
    expect(JSON.stringify(status)).not.toContain("/tmp/runhaven-status-preview");
    expect(JSON.stringify(status)).not.toContain("stateVolume");
    expect(JSON.stringify(status)).not.toContain("write_session_input");
    expect(JSON.stringify(status)).not.toContain("attach_session");
  });

  it("returns selected managed session diagnostics without raw paths", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-diagnostics-preview",
      sessionName: "diagnostics"
    };
    const started = await launchRun({
      plan: request,
      confirmLaunch: true,
      confirmedWarnings: []
    });
    const sessions = await getManagedSessions();
    const session = sessions.sessions.find((item) => item.runId === started.runId);

    expect(session).toBeDefined();
    await expect(getManagedSessionDiagnostics("missing-session")).rejects.toThrow(
      "managed session not found"
    );

    const diagnostics = await getManagedSessionDiagnostics(session!.id);

    expect(diagnostics.session.id).toBe(session!.id);
    expect(diagnostics.authStatus.name).toBe(session!.agent);
    expect(diagnostics.authLogEntries[0]?.runId).toBe(started.runId);
    expect(diagnostics.egressLogEntries[0]?.runId).toBe(started.runId);
    expect(JSON.stringify(diagnostics)).not.toContain("/tmp/runhaven-diagnostics-preview");
    expect(JSON.stringify(diagnostics)).not.toContain("token=");
    expect(JSON.stringify(diagnostics)).not.toContain("write_session_input");
    expect(JSON.stringify(diagnostics)).not.toContain("attach_session");
  });

  it("keeps managed session output refreshes isolated by session", async () => {
    const firstStarted = await launchRun({
      plan: {
        ...defaultRunPlanRequest(providerAgent),
        workspacePath: "/tmp/runhaven-alpha",
        sessionName: "alpha"
      },
      confirmLaunch: true,
      confirmedWarnings: []
    });
    const secondStarted = await launchRun({
      plan: {
        ...defaultRunPlanRequest(providerAgent),
        workspacePath: "/tmp/runhaven-beta",
        sessionName: "beta"
      },
      confirmLaunch: true,
      confirmedWarnings: []
    });
    const sessions = await getManagedSessions();
    const firstSession = sessions.sessions.find((item) => item.runId === firstStarted.runId);
    const secondSession = sessions.sessions.find((item) => item.runId === secondStarted.runId);

    expect(firstSession).toBeDefined();
    expect(secondSession).toBeDefined();
    expect(firstSession!.id).not.toBe(secondSession!.id);

    const firstRefresh = await getManagedSessionLogSnapshot(firstSession!.id, {
      confirmSensitiveOutput: true
    });
    const secondRefresh = await getManagedSessionLogSnapshot(secondSession!.id, {
      confirmSensitiveOutput: true
    });
    const firstFollowUp = await getManagedSessionLogSnapshot(firstSession!.id, {
      confirmSensitiveOutput: true
    });

    expect(firstFollowUp.cursor).not.toBe(firstRefresh.cursor);
    expect(firstFollowUp.entries[0]?.sessionId).toBe(firstSession!.id);
    expect(firstFollowUp.entries[0]?.text).toContain(firstSession!.id);
    expect(secondRefresh.entries[0]?.sessionId).toBe(secondSession!.id);
    expect(secondRefresh.entries[0]?.text).toContain(secondSession!.id);
    expect(JSON.stringify(firstFollowUp)).not.toContain(secondSession!.id);
    expect(JSON.stringify(secondRefresh)).not.toContain(firstSession!.id);
    expect(JSON.stringify(firstFollowUp)).not.toContain("write_session_input");
    expect(JSON.stringify(firstFollowUp)).not.toContain("resize_session");
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

  it("writes preview input through a managed session id", async () => {
    const request = {
      ...defaultRunPlanRequest(providerAgent),
      workspacePath: "/tmp/runhaven-input-preview",
      sessionName: "input"
    };
    const started = await launchRun({
      plan: request,
      confirmLaunch: true,
      confirmedWarnings: []
    });
    const sessions = await getManagedSessions();
    const session = sessions.sessions.find((item) => item.runId === started.runId);

    expect(session).toBeDefined();
    await expect(writeSessionInput("missing-session", "hello")).rejects.toThrow(
      "managed session not found"
    );
    await expect(writeSessionInput(session!.id, "")).rejects.toThrow("session input is required");

    const response = await writeSessionInput(session!.id, "hello agent\n");

    expect(response.accepted).toBe(true);
    expect(response.acceptedBytes).toBe("hello agent\n".length);
    expect(response.session.id).toBe(session!.id);
    expect(response.session.runId).toBe(started.runId);
    expect(response.status).toBe("accepted");
    expect(JSON.stringify(response)).not.toContain("/tmp/runhaven-input-preview");
    expect(JSON.stringify(response)).not.toContain("resize_session");
    expect(JSON.stringify(response)).not.toContain("attach_session");
  });
});
