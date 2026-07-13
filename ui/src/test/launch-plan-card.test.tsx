import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { LaunchPlanCard } from "../app/LaunchPlanCard";
import { orderProductAgents } from "../app/sessionIdentity";
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
  onStartSession: async () => {},
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

    expect(html).toContain("Choose assistant");
    expect(html).toContain("Pick the coding assistant you want to use.");
    expect(html).not.toContain("OpenAI Codex agent.");
    expect(html).not.toContain("API key handling");
    expect(html).not.toContain("Sign-in details");
    expect(html).toContain("Continue");
    expect(html).toContain(">Cancel</");
    expect(html).not.toContain(">Back</");
    expect(html).not.toContain("Review access");
  });

  it("uses the same assistant order as Home", () => {
    const ordered = orderProductAgents([
      agent({ name: "copilot" }),
      agent({ name: "antigravity" }),
      agent({ name: "claude" }),
      agent({ name: "codex" }),
    ]);

    expect(ordered.map((item) => item.name)).toEqual([
      "claude",
      "codex",
      "antigravity",
      "copilot",
    ]);
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

    expect(html).toContain("Checking assistant setup…");
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

    expect(html).toContain("Install this assistant before starting a session.");
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
    expect(html).toContain("Pick the project folder Codex can use.");
    expect(html).toContain(">Back</");
    expect(html).toContain("Start session");
    expect(html).toContain("Choose folder");
    expect(html).toContain("project");
    expect(html).toContain("Protected copy (recommended)");
    expect(html).toContain("Read-only original folder");
    expect(html).toContain("Original folder with changes (less safe)");
    expect(html).not.toContain("Why is this safe?");
    expect(html).not.toContain("Host check");
    expect(html).toContain('title="Enter a CPU count such as 4 or 2.5."');
    expect(html).toContain(">Memory<");
    expect(html).toContain('title="Enter memory such as 4g for 4 GB or 4096m."');
  });

  it("keeps the protected-copy implementation out of the ordinary project step", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);

    expect(html).not.toContain("Protect my project with an isolated copy");
    expect(html).not.toContain("git repo");
    expect(html).not.toContain("new git branch");
    expect(html.indexOf("Advanced options")).toBeLessThan(
      html.indexOf("Protected copy (recommended)"),
    );
  });

  it("uses the native folder picker instead of asking users to type a path", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);

    expect(html).toContain("Choose folder")
    expect(html).not.toContain("Project folder path")
    expect(html).not.toContain("/Users/you/project")
    expect(html.indexOf("Advanced options")).toBeLessThan(
      html.indexOf("Sign-in sharing"),
    );
  });

  it("shows selected-agent preparation on the project step without asking again", () => {
    const html = renderToStaticMarkup(
      <LaunchPlanCard
        {...projectProps()}
        agentReadiness={[
          { name: "codex", category: "needs-sign-in", label: "Sign in needed" },
        ]}
      />,
    );

    expect(html).toContain("Codex sign-in terminal")
    expect(html).not.toContain("Select agent")
    expect(html).not.toContain("Pick the agent you want to run")
  });

  it("warns that full internet access is only allowed after a warning", () => {
    const html = renderToStaticMarkup(<LaunchPlanCard {...projectProps()} />);

    expect(html).toContain("Full internet");
    expect(html).toContain("The assistant can reach any site. This is less safe.");
    expect(html).toContain("Assistant services only");
    expect(html).toContain("The assistant can reach only the services it needs.");
    expect(html).toContain("@container")
    expect(html).toContain("@[42rem]:grid-cols-3")
    expect(html).not.toContain("sm:grid-cols-3")
  });

  it("explains the consequence without blocking an offline cloud assistant", () => {
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

    expect(html).toContain("This assistant will be offline")
    expect(html).toContain("Claude can’t connect while Local only is selected.")
    expect(html).toContain("Choose Assistant services only to let it connect safely")
    expect(html).not.toContain("may ask you to sign in inside the sandbox")
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
      "RunHaven can’t share your saved sign-in with this network choice."
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

    expect(html).toContain("Preparing…");
  });

  it("explains Mac API-key access without broker jargon", () => {
    const brokerAgent = agent({ broker: "yes" })
    const html = renderToStaticMarkup(
      <LaunchPlanCard
        {...projectProps()}
        agents={[brokerAgent]}
        selectedAgent={brokerAgent}
        request={request({ apiKeyBrokerEnv: "OPENAI_API_KEY" })}
      />
    )

    expect(html).toContain("Use an API key from this Mac")
    expect(html).toContain('aria-label="Use OPENAI_API_KEY from this Mac"')
    expect(html).toContain(
      "RunHaven keeps <code class=\"font-mono\">OPENAI_API_KEY</code> on this Mac",
    )
    expect(html).toContain("temporary access for this session")
    expect(html).toContain("Assistant services only")
    expect(html).not.toContain("Use API key broker")
  })

  it("names assistant setup errors plainly", () => {
    const html = renderToStaticMarkup(
      <LaunchPlanCard
        {...agentProps()}
        imageStatus={null}
        imageError="The check failed."
      />
    )

    expect(html).toContain("Couldn’t check assistant setup")
  })
});
