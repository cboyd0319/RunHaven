import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { LaunchPlanCard } from "../app/LaunchPlanCard";
import type { AgentProfile, RunPlanRequest } from "../commands/runhaven";

function agent(patch: Partial<AgentProfile> = {}): AgentProfile {
  return {
    name: "codex",
    description: "OpenAI Codex agent.",
    image: "runhaven/codex:0.7.0",
    defaultCommand: ["codex"],
    providerHosts: ["api.openai.com"],
    signIn: "api-key",
    broker: "codex-api-key",
    supportsInAppSignIn: true,
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
  onInstallImage: () => {},
  onCancel: () => {},
  onBack: () => {},
  onContinue: () => {},
  onReviewPlan: async () => {},
  onSignedIn: () => {}
};

function agentProps() {
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
        fixCommand: null
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
        warning: null
      }
    },
    imageError: "",
    planning: false,
    setupOk: true,
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
  it("renders the agent step with cancel leading, continue trailing, no back", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...agentProps()} />);

    expect(html).toContain("Choose agent");
    expect(html).toContain("Pick the agent you want to run.");
    expect(html).toContain("OpenAI Codex agent.");
    expect(html).not.toContain("API key handling");
    expect(html).not.toContain("Sign-in details");
    expect(html).toContain("Continue");
    expect(html).toContain(">Cancel</");
    expect(html).not.toContain(">Back</");
    expect(html).not.toContain("Review access");
  });

  it("leaves the contextual setup explanation to New Session and disables continue", () => {
    const html = renderToStaticMarkup(
      <LaunchPlanCard {...agentProps()} setupOk={false} />
    );

    expect(html).not.toContain("Setup needs attention");
    expect(html).not.toContain("Fix the setup problem on Home before starting a run.");
    expect(html).toContain('disabled=""');
  });

  it("shows a checking-image state on the agent step", () => {
    const html = renderToStaticMarkup(
      <LaunchPlanCard {...agentProps()} imageLoading={true} />
    );

    expect(html).toContain("Checking image...");
  });

  it("blocks continue and gives a nontechnical next step when the agent is not installed", () => {
    const html = renderToStaticMarkup(
      <LaunchPlanCard
        {...agentProps()}
        imageStatus={{
          ...agentProps().imageStatus!,
          image: {
            ...agentProps().imageStatus!.image,
            status: "missing",
            ready: false,
            localSourceDigest: null,
            fixCommand: "runhaven image build codex"
          }
        }}
      />
    );

    expect(html).toContain("Install this agent before starting a run.");
    expect(html).toContain("Install Codex");
    expect(html).not.toContain("runhaven image build codex");
    expect(html).toContain('disabled=""');
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
    expect(html).toContain("Choose folder");
    expect(html).toContain("project");
    expect(html).toContain("Read-only project folder");
  });

  it("offers the isolated-copy toggle on the project step", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);

    expect(html).toContain("Protect my project with an isolated copy");
    expect(html).toContain("Recommended");
    expect(html).toContain("Your project folder must be a clean git repo.");
    expect(html.indexOf("Protect my project with an isolated copy")).toBeLessThan(
      html.indexOf("Advanced options"),
    );
  });

  it("keeps the raw folder path and expert controls inside Advanced options", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);

    expect(html.indexOf("Advanced options")).toBeLessThan(
      html.indexOf("Project folder path"),
    );
    expect(html.indexOf("Advanced options")).toBeLessThan(
      html.indexOf("Sign-in sharing"),
    );
  });

  it("warns that full internet access is only allowed after a warning", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);

    expect(html).toContain("Internet");
    expect(html).toContain("Allowed only after warning.");
    expect(html).toContain("Provider only");
    expect(html).toContain("Only reviewed provider hosts.");
  });

  it("warns without blocking alternate networks for Claude", () => {
    const claude = agent({
      name: "claude",
      description: "Claude",
      broker: "yes",
      providerHosts: ["api.anthropic.com"],
      supportsInAppSignIn: true
    });
    const html = renderToStaticMarkup(
      <LaunchPlanCard
        {...projectProps()}
        agents={[claude]}
        selectedAgent={claude}
        request={request({ agent: "claude", networkMode: "internal" })}
      />
    );

    expect(html).toContain("This choice does not use RunHaven")
    expect(html).toContain("host API key broker")
    expect(html).toContain("may ask you to sign in inside the sandbox")
    expect(html.match(/name="network"/g) ?? []).toHaveLength(3);
    expect(html).not.toContain('name="network" disabled=""');
  });

  it("explains that alternate networks turn off the host API key broker", () => {
    const codex = agent({ broker: "yes" });
    const html = renderToStaticMarkup(
      <LaunchPlanCard
        {...projectProps()}
        agents={[codex]}
        selectedAgent={codex}
        request={request({ networkMode: "internet" })}
      />
    );

    expect(html).toContain("Sign-in method changes");
    expect(html).toContain(
      "This choice does not use RunHaven&#x27;s host API key broker."
    );
    expect(html).not.toContain('name="network" disabled=""');
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
