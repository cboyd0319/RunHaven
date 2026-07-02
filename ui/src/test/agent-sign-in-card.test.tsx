// @vitest-environment happy-dom
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { AgentSignIn } from "../app/AgentSignIn";
import type { AgentProfile } from "../commands/runhaven";

function agent(patch: Partial<AgentProfile> = {}): AgentProfile {
  return {
    name: "codex",
    description: "OpenAI Codex agent.",
    image: "runhaven/codex:0.1.0",
    defaultCommand: ["codex"],
    providerHosts: ["api.openai.com"],
    signIn: "runhaven login",
    broker: "yes",
    supportsInAppSignIn: true,
    ...patch,
  };
}

describe("AgentSignIn", () => {
  it("offers an in-app sign-in button for device-code agents", () => {
    const html = renderToStaticMarkup(<AgentSignIn agent={agent()} />);
    expect(html).toContain("Sign in to Codex");
    // The GUI only triggers sign-in: there is no field to paste a token, code,
    // or password into.
    expect(html).not.toContain("<input");
    expect(html).not.toContain('type="password"');
  });

  it("routes claude to the on-your-Mac default with the sandbox option secondary", () => {
    const html = renderToStaticMarkup(
      <AgentSignIn
        agent={agent({ name: "claude", supportsInAppSignIn: false })}
      />,
    );
    // Path C (secure default) is the primary action.
    expect(html).toContain("Sign in on your Mac");
    // Path A (in-sandbox terminal) is a secondary opt-in, collapsed by default,
    // so its terminal command is not shown until the user chooses it.
    expect(html).toContain("Sign in inside the sandbox instead");
    expect(html).not.toContain("runhaven login claude");
    // Still no field to paste a secret into.
    expect(html).not.toContain("<input");
    expect(html).not.toContain('type="password"');
  });

  it("shows nothing for agents without any sign-in", () => {
    const html = renderToStaticMarkup(
      <AgentSignIn
        agent={agent({ name: "shell", signIn: "n/a", supportsInAppSignIn: false })}
      />,
    );
    expect(html).toBe("");
  });

  it("renders nothing without a selected agent", () => {
    const html = renderToStaticMarkup(<AgentSignIn agent={undefined} />);
    expect(html).toBe("");
  });
});
