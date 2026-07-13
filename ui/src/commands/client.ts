import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import { hasTauriRuntime, traceInvoke } from "./logging";
import {
  previewAuthExplain,
  previewAuthStatus,
  previewDashboardStatus,
  previewImageStatus,
  previewImageBuildStart,
  previewImageBuildStatus,
  previewNetworkPrunePreview,
  previewNetworkPrune,
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
  previewPlanRun,
  previewRunStatus,
  previewSessionInput,
  previewWhyHost,
  previewWhyNetwork,
  previewWhyState,
  previewWhyWorkspace
} from "./preview";
import type {
  AuthExplainResponse,
  AuthLogResponse,
  AuthStatusResponse,
  DashboardStatus,
  SetupActionId,
  SetupRecoveryResponse,
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
  MenuState,
  RunPlanRequest,
  RunPlanResponse,
  KillRunResponse,
  KillManagedSessionResponse,
  CleanupManagedSessionResponse,
  RepairRunResponse,
  RepairManagedSessionResponse,
  RunStatusResponse,
  StopManagedSessionResponse,
  StopRunResponse,
  WriteSessionInputResponse
} from "./types";
import type {
  ImageBuildStartResponse,
  ImageBuildStatusResponse,
  NetworkPrunePreview,
  NetworkPruneResponse
} from "./maintenance_types";

// Runtime detection lives in logging.ts (which must not import this module);
// re-exported here so existing consumers keep one import site.
export { hasTauriRuntime } from "./logging";

// A successful invoke slower than this is traced at warn (SLOW ok) so it stands
// out in the log; faster ones stay at debug. One real session showed a 15648ms
// invoke that was only findable by grepping every "ok in Xms" line.
const SLOW_INVOKE_MS = 1000;

// The one typed invoke touchpoint; domain files (terminal.ts) route through it.
// Every call is traced into the dev log by name, duration, and outcome only;
// args and results never leave this function (they may carry workspace paths
// and user text).
export async function call<T>(command: string, args: Record<string, unknown>, fallback: () => T): Promise<T> {
  if (!hasTauriRuntime()) {
    return fallback();
  }
  const startedAt = performance.now();
  try {
    const result = await invoke<T>(command, args);
    const durationMs = performance.now() - startedAt;
    traceInvoke(command, durationMs, undefined, durationMs > SLOW_INVOKE_MS);
    return result;
  } catch (failure) {
    traceInvoke(command, performance.now() - startedAt, failure ?? "unknown error");
    throw failure;
  }
}
export async function getDashboardStatus(): Promise<DashboardStatus> {
  return call("get_dashboard_status", {}, previewDashboardStatus);
}

export async function performSetupRecovery(
  actionId: SetupActionId
): Promise<SetupRecoveryResponse> {
  return call(
    "perform_setup_recovery",
    { request: { actionId } },
    () => ({
      actionId,
      status: "preview",
      label: "Desktop app required",
      detail: "Open RunHaven on your Mac to repair setup."
    })
  );
}

export async function getImageStatus(agent: string): Promise<ImageStatusResponse> {
  return call("get_image_status", { request: { agent } }, () => previewImageStatus(agent));
}

export async function rebuildAgentImage(agent: string): Promise<ImageBuildStartResponse> {
  return call(
    "rebuild_agent_image",
    { request: { agent, confirm: true } },
    () => previewImageBuildStart(agent)
  );
}

export async function getImageBuildStatus(buildId: string): Promise<ImageBuildStatusResponse> {
  return call(
    "get_image_build_status",
    { request: { buildId } },
    () => previewImageBuildStatus(buildId)
  );
}

export async function getNetworkPrunePreview(): Promise<NetworkPrunePreview> {
  return call("get_network_prune_preview", {}, previewNetworkPrunePreview);
}

export async function pruneManagedNetworks(): Promise<NetworkPruneResponse> {
  return call("prune_managed_networks", { request: { confirm: true } }, previewNetworkPrune);
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

export async function launchRun(
  request: LaunchRunRequest
): Promise<LaunchRunResponse | null> {
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

export async function writeSessionInput(
  sessionId: string,
  input: string
): Promise<WriteSessionInputResponse> {
  return call(
    "write_session_input",
    {
      request: {
        sessionId,
        input
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

export async function syncMenuState(state: MenuState): Promise<void> {
  // Menu enable-state only: booleans and one count in, native menu gray-out
  // out. The browser preview has no menu bar, so the fallback is a no-op.
  return call("sync_menu_state", { state }, () => undefined);
}

export async function chooseProjectFolder(): Promise<string | null> {
  if (!hasTauriRuntime()) {
    const previewPath = new URLSearchParams(window.location.search).get("project");
    return previewPath || "/tmp/runhaven-preview";
  }
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}
