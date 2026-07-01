import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { LaunchPlanCard } from "../app/LaunchPlanCard";
import type { AgentProfile, RunPlanRequest } from "../commands/runhaven";

function agent(patch: Partial<AgentProfile> = {}): AgentProfile {
  return {
    name: "codex",
    description: "OpenAI Codex agent.",
    image: "runhaven/codex:0.1.0",
    defaultCommand: ["codex"],
    providerHosts: ["api.openai.com"],
    signIn: "api-key",
    broker: "codex-api-key",
    ...patch
  };
}

function request(patch: Partial<RunPlanRequest> = {}): RunPlanRequest {
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
    ...patch
  };
}

const handlers = {
  onAgentChange: () => {},
  onPickFolder: async () => {},
  onRequestChange: () => {},
  onNetworkChange: () => {},
  onBack: () => {},
  onContinue: () => {},
  onReviewPlan: async () => {}
};

function agentProps() {
  return {
    step: "agent" as const,
    agents: [agent()],
    request: request(),
    selectedAgent: agent(),
    authStatus: null,
    imageLoading: false,
    imageStatus: null,
    imageError: "",
    planning: false,
    ...handlers
  };
}

function projectProps() {
  return {
    ...agentProps(),
    step: "project" as const
  };
}

describe("LaunchPlanCard", () => {
  it("renders the agent step with a continue action and no back button", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...agentProps()} />);

    expect(html).toContain("Choose agent");
    expect(html).toContain("Pick the agent you want to run.");
    expect(html).toContain("OpenAI Codex agent.");
    expect(html).toContain("Continue");
    expect(html).not.toContain(">Back</");
    expect(html).not.toContain("Review access");
  });

  it("shows a checking-image state on the agent step", () => {
    const html = renderToStaticMarkup(
      <LaunchPlanCard {...agentProps()} imageLoading={true} />
    );

    expect(html).toContain("Checking image...");
  });

  it("blocks continue until an agent is chosen", () => {
    // The disabled *attribute* renders as `disabled=""`; Tailwind `disabled:`
    // class names are always present, so match the attribute specifically.
    const withAgent = renderToStaticMarkup(<LaunchPlanCard {...agentProps()} />);
    expect(withAgent).not.toContain('disabled=""');

    const withoutAgent = renderToStaticMarkup(
      <LaunchPlanCard
        {...agentProps()}
        request={request({ agent: "" })}
        selectedAgent={undefined}
      />
    );
    expect(withoutAgent).toContain('disabled=""');
  });

  it("renders the project step with back and review actions", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);

    expect(html).toContain("Choose project");
    expect(html).toContain("Pick the project folder the agent can use.");
    expect(html).toContain(">Back</");
    expect(html).toContain("Review access");
    expect(html).toContain("Read-only project folder");
  });

  it("offers the isolated-copy toggle on the project step", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);

    expect(html).toContain("Run in an isolated copy (git worktree)");
    expect(html).toContain("Your project folder must be a clean git repo.");
  });

  it("warns that full internet access is only allowed after a warning", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);

    expect(html).toContain("Full internet");
    expect(html).toContain("Allowed only after warning.");
    expect(html).toContain("Provider allowlist");
    expect(html).toContain("Only reviewed provider hosts.");
  });

  it("blocks review until a workspace path is set", () => {
    const filled = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);
    expect(filled).not.toContain('disabled=""');

    const empty = renderToStaticMarkup(
      <LaunchPlanCard {...projectProps()} request={request({ workspacePath: "" })} />
    );
    expect(empty).toContain('disabled=""');
  });

  it("shows a reviewing label while planning is in flight", () => {
    const html = renderToStaticMarkup(
      <LaunchPlanCard {...projectProps()} planning={true} />
    );

    expect(html).toContain("Reviewing...");
  });
});
