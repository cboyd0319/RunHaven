import type { AgentProfile, RunPlanRequest } from "../commands/runhaven";

export function agent(patch: Partial<AgentProfile> = {}): AgentProfile {
  return {
    name: "codex",
    description: "OpenAI Codex agent.",
    image: "runhaven/codex:0.7.0",
    defaultCommand: ["codex"],
    providerHosts: ["api.openai.com"],
    signIn: "api-key",
    broker: "codex-api-key",
    supportsInAppSignIn: true,
    ...patch,
  };
}

export function request(patch: Partial<RunPlanRequest> = {}): RunPlanRequest {
  return {
    agent: "codex",
    workspacePath: "/Users/you/project",
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
    ...patch,
  };
}

const handlers = {
  onAgentChange: () => {},
  onPickFolder: async () => {},
  onRequestChange: () => {},
  onNetworkChange: () => {},
  onInstallImage: () => {},
  onBack: () => {},
  onContinue: () => {},
  onStartSession: async () => {},
  onSignedIn: () => {},
};

export function agentProps() {
  return {
    step: "agent" as const,
    agents: [agent()],
    agentReadiness: [],
    request: request(),
    selectedAgent: agent(),
    imageLoading: false,
    imageStatus: {
      agent: "codex",
      image: {
        agent: "codex",
        image: "runhaven/codex:0.7.0",
        status: "ok",
        ready: true,
        expectedSourceDigest: "expected",
        localSourceDigest: "expected",
        fixCommand: null,
      },
      builder: {
        status: "ready",
        detail: "Builder ready",
        image: null,
        cpus: null,
        memory: null,
        rosetta: null,
        startedDate: null,
        ipv4Address: null,
        warning: null,
      },
    },
    imageError: "",
    planning: false,
    setupOk: true,
    ...handlers,
  };
}

export function projectProps() {
  return { ...agentProps(), step: "project" as const };
}

export function reviewProps() {
  return { ...agentProps(), step: "review" as const };
}
