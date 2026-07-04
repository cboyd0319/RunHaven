// @vitest-environment happy-dom
import { createElement } from "react";
import { createRoot } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { AgentSignIn } from "../app/AgentSignIn";
import type {
  AgentProfile,
  AgentSignInStatusResponse,
} from "../commands/runhaven";
import { act } from "./render-hook";

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>();
  return {
    ...actual,
    startAgentSignIn: vi.fn(),
    getAgentSignInStatus: vi.fn(),
  };
});

const commands = await import("@/commands/runhaven");
const startSignIn = vi.mocked(commands.startAgentSignIn);
const getSignInStatus = vi.mocked(commands.getAgentSignInStatus);

function signInStatus(
  state: string,
  patch: Partial<AgentSignInStatusResponse> = {},
): AgentSignInStatusResponse {
  return {
    signInId: "si1",
    agent: "codex",
    state,
    label: "Signing in",
    detail: "Open the link to finish.",
    outputTail: "",
    updatedAt: "2026-07-01T02:00:00Z",
    ...patch,
  };
}

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

describe("AgentSignIn stop-waiting behavior", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    startSignIn.mockReset();
    getSignInStatus.mockReset();
  });
  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  it("hides the restart while still signing after the user stops waiting", async () => {
    // Once the device login is open, stopping must not offer a restart that
    // would start a second sign-in. The card shows only the honest message.
    startSignIn.mockResolvedValue({
      signInId: "si1",
      agent: "codex",
      state: "starting",
      label: "Signing in",
      detail: "Starting.",
    });
    getSignInStatus.mockResolvedValue(signInStatus("needs_browser"));

    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => {
      root.render(createElement(AgentSignIn, { agent: agent() }));
    });

    const signIn = Array.from(container.querySelectorAll("button")).find((b) =>
      b.textContent?.includes("Sign in"),
    );
    await act(async () => {
      signIn?.click();
    });
    const stop = Array.from(container.querySelectorAll("button")).find((b) =>
      b.textContent?.includes("Stop waiting"),
    );
    expect(stop).toBeDefined();

    await act(async () => {
      stop?.click();
    });

    expect(container.textContent).toContain(
      "You stopped waiting. The sign-in may still finish on its own.",
    );
    // No restart control while the backend is still signing.
    const restart = Array.from(container.querySelectorAll("button")).find((b) =>
      b.textContent?.includes("Sign in"),
    );
    expect(restart).toBeUndefined();
    await act(async () => {
      root.unmount();
    });
  });
});
