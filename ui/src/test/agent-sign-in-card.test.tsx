// @vitest-environment happy-dom
import { createElement } from "react";
import { createRoot } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { AgentSignIn } from "../app/AgentSignIn";
import type { AgentProfile } from "../commands/runhaven";

import { act } from "./render-hook";

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

  it("directs claude to the terminal instead of an in-app button", () => {
    const html = renderToStaticMarkup(
      <AgentSignIn
        agent={agent({ name: "claude", supportsInAppSignIn: false })}
      />,
    );
    expect(html).toContain("runhaven login claude");
    expect(html).toContain("terminal");
    expect(html).toContain("Copy command");
    expect(html).not.toContain("Sign in to claude");
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

  it("copies the exact terminal command and confirms with the button label", async () => {
    const written: string[] = [];
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: async (text: string) => {
          written.push(text);
        }
      }
    });

    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => {
      root.render(
        createElement(AgentSignIn, {
          agent: agent({ name: "claude", supportsInAppSignIn: false })
        })
      );
    });

    const copyButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Copy command")
    );
    expect(copyButton).toBeDefined();
    await act(async () => {
      copyButton?.click();
    });

    expect(written).toEqual(["runhaven login claude"]);
    expect(copyButton?.textContent).toContain("Copied");

    await act(async () => {
      root.unmount();
    });
  });
});
