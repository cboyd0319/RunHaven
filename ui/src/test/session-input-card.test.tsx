import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import {
  SessionInputCard,
  sessionCanAcceptInput
} from "../app/SessionInputCard";
import type { ManagedSessionSummary } from "../commands/runhaven";

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
    expect(sessionCanAcceptInput(session())).toBe(true);

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

    expect(sessionCanAcceptInput(detached)).toBe(false);

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

    expect(sessionCanAcceptInput(blocked)).toBe(false);
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
});
