import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { HomeView } from "../app/views/HomeView";
import type { MainViewProps } from "../app/views/props";
import type { LaunchFlow } from "../app/useLaunchFlow";
import type {
  DashboardStatus,
  ManagedSessionSummary,
  RunPlanRequest
} from "../commands/runhaven";
import { providerAgent } from "./runhaven-fixtures";

function request(): RunPlanRequest {
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
    worktree: false
  };
}

// A closed launch flow: static markup renders only the New run trigger, which
// is exactly the Home surface these tests pin.
function launchFlow(): LaunchFlow {
  return {
    sheetOpen: false,
    request: request(),
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
    setLaunchStep: () => {}
  };
}

function dashboard(setupOk: boolean): DashboardStatus {
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
          remedy: setupOk ? "No action needed." : "Install Apple container."
        }
      ]
    },
    agents: [providerAgent],
    activeRuns: [],
    recentRuns: [],
    warnings: []
  };
}

function session(
  patch: Partial<ManagedSessionSummary> = {}
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
    ...patch
  };
}

function props(patch: Partial<MainViewProps> = {}): MainViewProps {
  return {
    error: "",
    launchMessage: "",
    loading: false,
    dashboard: dashboard(true),
    authStatus: {
      status: "ok",
      runtime: "preview",
      profiles: [{ name: "codex", status: "ready" }]
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
      loadSnapshot: async () => {}
    },
    controlBusy: false,
    controlError: "",
    controlMessage: "",
    controlConfirmations: {
      stop: false,
      kill: false,
      repair: false,
      cleanup: false
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
    ...patch
  };
}

describe("HomeView", () => {
  it("shows the one-line health, the New run action, and the first-run note when healthy and empty", () => {
    const html = renderToStaticMarkup(<HomeView {...props()} />);

    expect(html).toContain("Your Mac is ready to run agents.");
    expect(html).toContain("New run");
    expect(html).not.toContain('disabled=""');
    expect(html).toContain("RunHaven runs AI coding agents in a sandbox on your Mac.");
    // The wizard is no longer inline on Home; it lives in the closed sheet.
    expect(html).not.toContain("Choose agent");
    expect(html).not.toContain("Finish setup");
  });

  it("leads with the setup blocker and disables New run with a reason while blocked", () => {
    const html = renderToStaticMarkup(
      <HomeView {...props({ setupReady: false, dashboard: dashboard(false) })} />
    );

    expect(html).toContain("Finish setup");
    expect(html).toContain("Apple container is not installed.");
    expect(html).toContain("Install Apple container.");
    expect(html).toContain("Check again");
    expect(html).toContain('disabled=""');
    expect(html).toContain("Fix the setup problem above before starting a run.");
    expect(html).not.toContain("Your Mac is ready to run agents.");
  });

  it("lists agent readiness with a link to the Agents view", () => {
    const html = renderToStaticMarkup(<HomeView {...props()} />);

    expect(html).toContain("Codex");
    expect(html).toContain("Ready");
    expect(html).toContain("Open Agents");
  });

  it("shows needs-you sessions and recent finished runs as navigable rows", () => {
    const html = renderToStaticMarkup(
      <HomeView
        {...props({
          sessions: [
            session({ id: "s1", title: "Codex - failing", lifecycle: "failed" }),
            session({
              id: "s2",
              title: "Codex - done",
              lifecycle: "finished",
              statusLabel: "Finished"
            }),
            session({ id: "s3", title: "Codex - quiet" })
          ]
        })}
      />
    );

    expect(html).toContain("Needs you");
    expect(html).toContain("Codex - failing");
    expect(html).toContain("Recent finished runs");
    expect(html).toContain("Codex - done");
    // A quiet running session appears in neither Home list.
    expect(html).not.toContain("Codex - quiet");
    // With sessions present, the first-run note goes away.
    expect(html).not.toContain("RunHaven runs AI coding agents in a sandbox");
  });
});
