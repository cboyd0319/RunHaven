import type { LaunchFlow } from "../app/useLaunchFlow"
import type { MainViewProps } from "../app/views/props"
import type {
  DashboardStatus,
  ManagedSessionSummary,
  RunPlanRequest,
} from "../commands/runhaven"

import { providerAgent } from "./runhaven-fixtures"

// Shared factories for view tests that take the full MainViewProps bag
// (New session, Session). Each returns a fresh object; patch what the case needs.

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

// A launch flow fixture with no launch in progress.
export function launchFlow(): LaunchFlow {
  return {
    request: planRequest(),
    launchStep: "agent",
    planning: false,
    launchMessage: "",
    reviewFocusRevision: 0,
    error: "",
    runTerminal: null,
    runTerminals: [],
    setOpen: () => {},
    applyDashboard: async () => {},
    changeAgent: () => {},
    updateNetworkMode: () => {},
    updateRequest: () => {},
    pickFolder: async () => {},
    startSession: async () => {},
    retryWithoutProtectedCopy: async () => {},
    closeRunTerminal: async () => {},
    cancelRunTerminal: () => {},
    markRunTerminalOpened: () => {},
    applySavedSetup: () => {},
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
          action: null,
        },
      ],
    },
    agents: [providerAgent],
    agentReadiness: [
      { name: "codex", category: "sign-in-saved", label: "Sign-in saved" },
    ],
    activeRuns: [],
    recentRuns: [],
    warnings: [],
    appVersion: "0.5.0",
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
    lastRefreshedAt: null,
    dashboard: dashboard(true),
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
      loading: false,
      error: "",
      visibleSnapshot: null,
      loadSnapshot: async () => {},
    },
    controlBusy: false,
    controlError: "",
    controlMessage: "",
    onControl: async () => {},
    onWorktreeResolved: async () => {},
    launchFlow: launchFlow(),
    savedSetups: {
      pinnedSetups: [],
      error: "",
      message: "",
      refresh: async () => {},
      selectSetup: async () => {},
      repairUnavailable: async () => {},
      pinDraft: async () => {},
      unpin: async () => {},
    },
    supportReport: {
      busy: false,
      message: "",
      error: "",
      create: async () => {},
    },
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
    ...patch,
  }
}
