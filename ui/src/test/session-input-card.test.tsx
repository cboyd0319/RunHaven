// @vitest-environment happy-dom
import { createElement } from "react";
import { createRoot } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import { SessionInputCard } from "../app/SessionInputCard";
import type { ManagedSessionSummary } from "../commands/runhaven";

import { act } from "./render-hook";

function session(patch: Partial<ManagedSessionSummary> = {}): ManagedSessionSummary {
  return {
    id: "run-123",
    runId: "run-123",
    agent: "codex",
    session: "project",
    title: "Codex - project",
    project: "project",
    status: "running",
    statusLabel: "Running",
    authStatus: "api-key-broker",
    authScope: "project",
    network: "provider",
    startedAt: "2026-06-30T02:00:00Z",
    containerName: "runhaven-codex-project",
    detailCommand: "runhaven runs show run-123",
    lifecycle: "running",
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking status for this session."
    },
    input: {
      attached: true,
      status: "attached",
      detail: "Session input is ready."
    },
    events: [],
    ...patch
  };
}

describe("SessionInputCard", () => {
  it("renders a bounded composer for sessions that can accept input", () => {
    const html = renderToStaticMarkup(
      <SessionInputCard
        session={session()}
        draft=""
        sending={false}
        error=""
        acceptedMessage=""
        onDraftChange={() => undefined}
        onSend={() => undefined}
      />
    );

    expect(html).toContain("Message");
    expect(html).toContain("Send text to this selected session.");
    expect(html).toContain("textarea");
    expect(html).toContain("disabled=\"\"");
    expect(html).not.toContain("/Users/");
    expect(html).not.toContain("resize_session");
    expect(html).not.toContain("attach_session");
  });

  it("shows when a running session cannot accept input in this window", () => {
    const detached = session({
      input: {
        attached: false,
        status: "detached",
        detail: "Session input transport is not attached in this RunHaven window."
      }
    });

    const html = renderToStaticMarkup(
      <SessionInputCard
        session={detached}
        draft="hello"
        sending={false}
        error=""
        acceptedMessage=""
        onDraftChange={() => undefined}
        onSend={() => undefined}
      />
    );

    expect(html).toContain("Message");
    expect(html).toContain("Input unavailable");
    expect(html).toContain("not attached");
    expect(html).toContain("disabled=\"\"");
  });

  it("stays hidden when the session needs auth attention", () => {
    const blocked = session({
      nextAction: {
        kind: "auth_attention",
        label: "Finish auth setup",
        detail: "Auth is not ready."
      }
    });

    expect(
      renderToStaticMarkup(
        <SessionInputCard
          session={blocked}
          draft="hello"
          sending={false}
          error=""
          acceptedMessage=""
          onDraftChange={() => undefined}
          onSend={() => undefined}
        />
      )
    ).toBe("");
  });

  it("shows the sent acknowledgement once, not as a duplicate muted line", () => {
    const html = renderToStaticMarkup(
      <SessionInputCard
        session={session()}
        draft=""
        sending={false}
        error=""
        acceptedMessage="Sent 5 bytes."
        onDraftChange={() => undefined}
        onSend={() => undefined}
      />
    );

    // One unified acknowledgement: the title and the byte-count detail each
    // appear exactly once, not as two separate notices side by side.
    expect(html.split("Message sent").length - 1).toBe(1);
    expect(html.split("Sent 5 bytes.").length - 1).toBe(1);
  });

  it("clears the sent acknowledgement when the composer is focused again", async () => {
    const onDraftChange = vi.fn();
    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => {
      root.render(
        createElement(SessionInputCard, {
          session: session(),
          draft: "next message",
          sending: false,
          error: "",
          acceptedMessage: "Sent 5 bytes.",
          onDraftChange,
          onSend: () => undefined
        })
      );
    });

    // Refocusing to type again clears the stale ack by re-applying the current
    // draft (text unchanged, acknowledgement reset), so it never lingers.
    await act(async () =>
      container
        .querySelector("textarea")
        ?.dispatchEvent(new FocusEvent("focusin", { bubbles: true }))
    );
    expect(onDraftChange).toHaveBeenCalledWith("next message");

    await act(async () => {
      root.unmount();
    });
  });

  it("does not clear on focus when there is no acknowledgement to clear", async () => {
    const onDraftChange = vi.fn();
    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => {
      root.render(
        createElement(SessionInputCard, {
          session: session(),
          draft: "typing",
          sending: false,
          error: "",
          acceptedMessage: "",
          onDraftChange,
          onSend: () => undefined
        })
      );
    });

    await act(async () =>
      container
        .querySelector("textarea")
        ?.dispatchEvent(new FocusEvent("focusin", { bubbles: true }))
    );
    expect(onDraftChange).not.toHaveBeenCalled();

    await act(async () => {
      root.unmount();
    });
  });

  it("sends on Cmd/Ctrl-Return through the same Send path, not plain Enter", async () => {
    const onSend = vi.fn();
    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => {
      root.render(
        createElement(SessionInputCard, {
          session: session(),
          draft: "ready to send",
          sending: false,
          error: "",
          acceptedMessage: "",
          onDraftChange: () => undefined,
          onSend
        })
      );
    });

    const textarea = container.querySelector("textarea");
    expect(textarea).not.toBeNull();

    function press(init: KeyboardEventInit) {
      textarea?.dispatchEvent(
        new KeyboardEvent("keydown", { key: "Enter", bubbles: true, ...init })
      );
    }

    // Plain Enter is a newline in the composer, never a send.
    await act(async () => press({}));
    expect(onSend).not.toHaveBeenCalled();

    // Cmd-Return (and Ctrl-Return) send.
    await act(async () => press({ metaKey: true }));
    await act(async () => press({ ctrlKey: true }));
    expect(onSend).toHaveBeenCalledTimes(2);

    await act(async () => {
      root.unmount();
    });
  });

  it("does not send on Cmd-Return when the draft cannot be sent", async () => {
    const onSend = vi.fn();
    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => {
      root.render(
        createElement(SessionInputCard, {
          session: session(),
          draft: "",
          sending: false,
          error: "",
          acceptedMessage: "",
          onDraftChange: () => undefined,
          onSend
        })
      );
    });

    container
      .querySelector("textarea")
      ?.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: "Enter",
          metaKey: true,
          bubbles: true
        })
      );
    expect(onSend).not.toHaveBeenCalled();

    await act(async () => {
      root.unmount();
    });
  });
});
