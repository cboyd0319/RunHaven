import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import {
  previewAgents,
  previewAuthExplain,
  previewAuthStatus,
  previewDashboardStatus,
  previewImageStatus,
  previewLaunchStatus,
  previewLaunchRun,
  previewLogSnapshot,
  previewManagedKill,
  previewManagedCleanup,
  previewManagedSessionDiagnostics,
  previewManagedRepair,
  previewManagedSessionLogSnapshot,
  previewManagedSessionStatus,
  previewManagedSessionsList,
  previewManagedStop,
  previewManagedSessionWorktree,
  previewManagedWorktreeAction,
  previewPlanRun,
  previewRunStatus,
  previewSessionInput,
  previewSetupStatus,
  previewWhyHost,
  previewWhyNetwork,
  previewWhyState,
  previewWhyWorkspace
} from "./preview";
import type {
  AgentProfile,
  AuthExplainResponse,
  AuthLogResponse,
  AuthStatusResponse,
  DashboardStatus,
  EgressLogResponse,
  HostExplanationResponse,
  ImageStatusResponse,
  LaunchRunRequest,
  LaunchRunResponse,
  LaunchStatusResponse,
  LogSnapshotOptions,
  LogSnapshotResponse,
  NetworkExplanationResponse,
  StateExplanationResponse,
  WorkspaceExplanationResponse,
  ManagedSessionList,
  ManagedSessionDiagnosticsResponse,
  ManagedSessionLogSnapshotResponse,
  ManagedSessionStatusResponse,
  RunPlanRequest,
  RunPlanResponse,
  KillRunResponse,
  KillManagedSessionResponse,
  CleanupManagedSessionResponse,
  RepairRunResponse,
  RepairManagedSessionResponse,
  RunStatusResponse,
  SetupStatus,
  StopManagedSessionResponse,
  StopRunResponse,
  WorktreeActionResponse,
  WorktreeReviewResponse,
  WriteSessionInputResponse
} from "./types";

export function hasTauriRuntime(): boolean {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function call<T>(command: string, args: Record<string, unknown>, fallback: () => T): Promise<T> {
  if (!hasTauriRuntime()) {
    return fallback();
  }
  return invoke<T>(command, args);
}

export async function getSetupStatus(): Promise<SetupStatus> {
  return call("get_setup_status", {}, previewSetupStatus);
}

export async function listAgents(): Promise<AgentProfile[]> {
  return call("list_agents", {}, previewAgents);
}

export async function getDashboardStatus(): Promise<DashboardStatus> {
  return call("get_dashboard_status", {}, previewDashboardStatus);
}

export async function getImageStatus(agent: string): Promise<ImageStatusResponse> {
  return call("get_image_status", { request: { agent } }, () => previewImageStatus(agent));
}

export async function getRunStatus(runId: string): Promise<RunStatusResponse> {
  return call("get_run_status", { request: { runId } }, () => previewRunStatus(runId));
}

export async function getLogSnapshot(
  runId: string,
  options: LogSnapshotOptions
): Promise<LogSnapshotResponse> {
  if (!options.confirmSensitiveOutput) {
    throw new Error("Confirm raw log viewing before loading output that may contain secrets.");
  }
  const requestedLines = options.lines ?? 200;
  return call(
    "get_log_snapshot",
    {
      request: {
        runId,
        lines: options.lines ?? null,
        confirmSensitiveOutput: options.confirmSensitiveOutput
      }
    },
    () => previewLogSnapshot(runId, requestedLines)
  );
}

export async function getManagedSessionLogSnapshot(
  sessionId: string,
  options: LogSnapshotOptions
): Promise<ManagedSessionLogSnapshotResponse> {
  if (!options.confirmSensitiveOutput) {
    throw new Error("Confirm raw log viewing before loading output that may contain secrets.");
  }
  return call(
    "get_managed_session_log_snapshot",
    {
      request: {
        sessionId,
        lines: options.lines ?? null,
        confirmSensitiveOutput: options.confirmSensitiveOutput
      }
    },
    () => previewManagedSessionLogSnapshot(sessionId, options.lines ?? 200)
  );
}

export async function getManagedSessions(): Promise<ManagedSessionList> {
  return call("get_managed_sessions", {}, previewManagedSessionsList);
}

export async function getManagedSessionStatus(
  sessionId: string
): Promise<ManagedSessionStatusResponse> {
  return call(
    "get_managed_session_status",
    { request: { sessionId } },
    () => previewManagedSessionStatus(sessionId)
  );
}

export async function getManagedSessionDiagnostics(
  sessionId: string
): Promise<ManagedSessionDiagnosticsResponse> {
  return call(
    "get_managed_session_diagnostics",
    { request: { sessionId } },
    () => previewManagedSessionDiagnostics(sessionId)
  );
}

export async function planRun(request: RunPlanRequest): Promise<RunPlanResponse> {
  return call("plan_run", { request }, () => previewPlanRun(request));
}

export async function launchRun(request: LaunchRunRequest): Promise<LaunchRunResponse> {
  return call("launch_run", { request }, () => previewLaunchRun(request));
}

export async function getLaunchStatus(runId: string): Promise<LaunchStatusResponse> {
  return call(
    "get_launch_status",
    { request: { runId } },
    () => previewLaunchStatus(runId)
  );
}

export async function stopRun(runId: string): Promise<StopRunResponse> {
  return call("stop_run", { request: { runId, confirmStop: true } }, () => ({
    runId,
    containerName: "preview",
    status: "stop-requested"
  }));
}

export async function stopManagedSession(
  sessionId: string
): Promise<StopManagedSessionResponse> {
  return call(
    "stop_managed_session",
    { request: { sessionId, confirmStop: true } },
    () => previewManagedStop(sessionId)
  );
}

export async function killRun(runId: string): Promise<KillRunResponse> {
  return call("kill_run", { request: { runId, confirmKill: true } }, () => ({
    runId,
    containerName: "preview",
    status: "kill-requested"
  }));
}

export async function killManagedSession(
  sessionId: string
): Promise<KillManagedSessionResponse> {
  return call(
    "kill_managed_session",
    { request: { sessionId, confirmKill: true } },
    () => previewManagedKill(sessionId)
  );
}

export async function repairRun(runId: string): Promise<RepairRunResponse> {
  return call("repair_run", { request: { runId, confirmRepair: true } }, () => ({
    runId,
    containerName: "preview",
    status: "removed",
    markerRemoved: true
  }));
}

export async function repairManagedSession(
  sessionId: string
): Promise<RepairManagedSessionResponse> {
  return call(
    "repair_managed_session",
    { request: { sessionId, confirmRepair: true } },
    () => previewManagedRepair(sessionId)
  );
}

export async function cleanupManagedSession(
  sessionId: string
): Promise<CleanupManagedSessionResponse> {
  return call(
    "cleanup_managed_session",
    { request: { sessionId, confirmCleanup: true } },
    () => previewManagedCleanup(sessionId)
  );
}

export async function getManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeReviewResponse> {
  return call(
    "get_managed_session_worktree",
    { request: { sessionId } },
    () => previewManagedSessionWorktree(sessionId)
  );
}

export async function keepManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeActionResponse> {
  return call(
    "keep_managed_session_worktree",
    { request: { sessionId, confirm: true } },
    () => previewManagedWorktreeAction(sessionId, "keep")
  );
}

export async function mergeManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeActionResponse> {
  return call(
    "merge_managed_session_worktree",
    { request: { sessionId, confirm: true } },
    () => previewManagedWorktreeAction(sessionId, "merge")
  );
}

export async function recoverManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeActionResponse> {
  return call(
    "recover_managed_session_worktree",
    { request: { sessionId, confirm: true } },
    () => previewManagedWorktreeAction(sessionId, "recover")
  );
}

export async function discardManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeActionResponse> {
  return call(
    "discard_managed_session_worktree",
    { request: { sessionId, confirm: true } },
    () => previewManagedWorktreeAction(sessionId, "discard")
  );
}

export async function writeSessionInput(
  sessionId: string,
  input: string
): Promise<WriteSessionInputResponse> {
  const clientTarget = `session:${sessionId}`;
  return call(
    "write_session_input",
    {
      request: {
        sessionId,
        input,
        confirmSend: true,
        clientTarget
      }
    },
    () => previewSessionInput(sessionId, input)
  );
}

export async function getEgressLog(): Promise<EgressLogResponse> {
  return call("get_egress_log", {}, () => ({ entries: [] }));
}

export async function getAuthLog(): Promise<AuthLogResponse> {
  return call("get_auth_log", {}, () => ({ entries: [] }));
}

export async function getAuthStatus(): Promise<AuthStatusResponse> {
  return call("get_auth_status", {}, previewAuthStatus);
}

export async function getAuthExplain(agent: string): Promise<AuthExplainResponse> {
  return call("get_auth_explain", { agent }, () => previewAuthExplain(agent));
}

export async function getWhyNetwork(mode: string): Promise<NetworkExplanationResponse> {
  return call("get_why_network", { mode }, () => previewWhyNetwork(mode));
}

export async function getWhyState(agent: string): Promise<StateExplanationResponse> {
  return call("get_why_state", { agent }, () => previewWhyState(agent));
}

export async function getWhyHost(
  host: string,
  port: number,
  agent: string | null
): Promise<HostExplanationResponse> {
  return call(
    "get_why_host",
    { request: { host, port, agent } },
    () => previewWhyHost(host)
  );
}

export async function getWhyWorkspace(
  path: string,
  workspaceScope: string,
  allowSensitiveWorkspace: boolean
): Promise<WorkspaceExplanationResponse> {
  return call(
    "get_why_workspace",
    { request: { path, workspaceScope, allowSensitiveWorkspace } },
    () => previewWhyWorkspace(workspaceScope)
  );
}

export async function chooseProjectFolder(): Promise<string | null> {
  if (!hasTauriRuntime()) {
    return null;
  }
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}
