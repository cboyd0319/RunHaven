import type { LaunchFlow } from "../app/useLaunchFlow"
import type { MainViewProps } from "../app/views/props"
import type {
  DashboardStatus,
  ManagedSessionSummary,
  RunPlanRequest,
} from "../commands/runhaven"

import { providerAgent } from "./runhaven-fixtures"

// Shared factories for view tests that take the full MainViewProps bag
// (Home, Session). Each returns a fresh object; patch what the case needs.

export function planRequest(): RunPlanRequest {
  return {
    agent: "codex",
    workspacePath: "",
    networkMode: "provider",
    workspaceScope: "current",
    authScope: "project",
    sessionName: null,
    readOnlyWorkspace: false,
    cpus: "4",
    memory: "4g",
    providerHosts: ["api.openai.com"],
    envNames: [],
    apiKeyBrokerEnv: null,
    image: null,
    allowSensitiveWorkspace: false,
    allowRootUser: false,
    user: "runner",
    worktree: false,
  }
}

// A closed launch flow: static markup renders only the New run trigger.
export function launchFlow(): LaunchFlow {
  return {
    sheetOpen: false,
    request: planRequest(),
    launchStep: "agent",
    plan: null,
    planning: false,
    launching: false,
    launchConfirmation: false,
    confirmedWarnings: [],
    launchMessage: "",
    error: "",
    setOpen: () => {},
    applyDashboard: async () => {},
    changeAgent: () => {},
    updateNetworkMode: () => {},
    updateRequest: () => {},
    pickFolder: async () => {},
    reviewPlan: async () => {},
    setWarningConfirmation: () => {},
    startRun: async () => {},
    setLaunchConfirmation: () => {},
    setLaunchStep: () => {},
  }
}

export function dashboard(setupOk: boolean): DashboardStatus {
  return {
    setup: {
      ok: setupOk,
      blockerCount: setupOk ? 0 : 1,
      checks: [
        {
          name: "Container CLI",
          ok: setupOk,
          detail: setupOk
            ? "Apple container is installed."
            : "Apple container is not installed.",
          remedy: setupOk ? "No action needed." : "Install Apple container.",
        },
      ],
    },
    agents: [providerAgent],
    activeRuns: [],
    recentRuns: [],
    warnings: [],
  }
}

export function managedSession(
  patch: Partial<ManagedSessionSummary> = {},
): ManagedSessionSummary {
  return {
    id: "s1",
    runId: "run-s1",
    agent: "codex",
    session: "alpha",
    title: "Codex - alpha",
    project: "alpha",
    status: "running",
    statusLabel: "Running",
    authStatus: "ready",
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-01T01:00:00Z",
    containerName: "runhaven-s1",
    detailCommand: "runhaven runs show run-s1",
    lifecycle: "running",
    input: { attached: true, status: "attached", detail: "" },
    nextAction: { kind: "monitor", label: "Monitor", detail: "" },
    events: [],
    ...patch,
  }
}

export function mainViewProps(
  patch: Partial<MainViewProps> = {},
): MainViewProps {
  return {
    error: "",
    launchMessage: "",
    loading: false,
    dashboard: dashboard(true),
    authStatus: {
      status: "ok",
      runtime: "preview",
      profiles: [{ name: "codex", status: "ready" }],
    },
    setupReady: true,
    selectedSession: null,
    selectedSessionStatus: null,
    selectedSessionStatusLoading: false,
    selectedSessionStatusError: "",
    selectedSessionDiagnostics: null,
    selectedSessionDiagnosticsLoading: false,
    selectedSessionDiagnosticsError: "",
    onLoadSelectedSessionDiagnostics: async () => {},
    launchStatus: null,
    launchStatusLoading: false,
    launchStatusError: "",
    sessionInputDraft: "",
    sessionInputSending: false,
    sessionInputError: "",
    sessionInputAcceptedMessage: "",
    onSessionInputDraftChange: () => {},
    onSendSelectedSessionInput: async () => {},
    runOutput: {
      acknowledged: false,
      loading: false,
      error: "",
      visibleSnapshot: null,
      liveAvailable: false,
      liveEnabled: false,
      liveRefreshing: false,
      setAcknowledged: () => {},
      setLiveEnabled: () => {},
      loadSnapshot: async () => {},
    },
    controlBusy: false,
    controlError: "",
    controlMessage: "",
    controlConfirmations: {
      stop: false,
      kill: false,
      repair: false,
      cleanup: false,
    },
    onControlConfirmationChange: () => {},
    onControl: async () => {},
    onWorktreeResolved: async () => {},
    launchFlow: launchFlow(),
    launchReady: false,
    launchImageReady: true,
    selectedAgent: providerAgent,
    imageLoading: false,
    imageStatus: null,
    imageError: "",
    onSignedIn: () => {},
    sessions: [],
    onNavigate: () => {},
    onRefresh: () => {},
    lastLaunch: null,
    lastLaunchSessionId: null,
    onSelectSession: () => {},
    runStatus: null,
    runStatusLoading: false,
    runStatusError: "",
    diagnostics: null,
    diagnosticsLoading: false,
    diagnosticsError: "",
    onLoadDiagnostics: async () => {},
    ...patch,
  }
}
