import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import {
  ManagedSessionCard,
  sessionAllowsCleanup,
  sessionAllowsRunControl,
  sessionNeedsAttention
} from "../app/ManagedSessionCard";
import { sameManagedSessionList } from "../app/sessionList";
import { authLabel } from "../app/sessionIdentity";
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
    authScope: "project",
    network: "provider",
    startedAt: "2026-06-30T02:00:00Z",
    containerName: "runhaven-codex-project",
    detailCommand: "runhaven runs show run-123",
    lifecycle: "running",
    input: {
      attached: true,
      status: "attached",
      detail: "Session input is ready."
    },
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking status for this session."
    },
    events: [
      {
        source: "run",
        label: "Run record",
        detail: "Running",
        timestamp: "2026-06-30T02:00:00Z"
      },
      {
        source: "auth",
        label: "Auth",
        detail: "API key broker",
        timestamp: "2026-06-30T02:00:00Z"
      },
      {
        source: "network",
        label: "Network",
        detail: "Provider only",
        timestamp: "2026-06-30T02:00:00Z"
      }
    ],
    ...patch
  };
}

describe("ManagedSessionCard", () => {
  it("renders a read-only selected-session summary without host paths", () => {
    const html = renderToStaticMarkup(
      <ManagedSessionCard session={session({ project: "secret-project" })} />
    );

    expect(html).toContain("Codex - project");
    expect(html).toContain("secret-project");
    expect(html).toContain("Provider only");
    expect(html).toContain("Login state");
    expect(html).toContain("Project");
    expect(html).toContain("Selected session overview.");
    expect(html).toContain("Monitor session");
    expect(html).toContain("Run record");
    expect(html).not.toContain("/Users/");
    expect(html).not.toContain("/sensitive/");
    expect(html).not.toContain("state_volume");
    expect(html).not.toContain("write_session_input");
  });

  it("shows auth attention when the broker state is not ready", () => {
    const blocked = session({
      authStatus: "unknown",
      nextAction: {
        kind: "auth_attention",
        label: "Finish auth setup",
        detail: "Auth is not ready for Codex."
      },
      events: [
        {
          source: "auth",
          label: "Auth",
          detail: "Unknown",
          timestamp: "2026-06-30T02:00:00Z"
        }
      ]
    });

    expect(sessionNeedsAttention(blocked)).toBe(true);

    const html = renderToStaticMarkup(<ManagedSessionCard session={blocked} />);

    expect(html).toContain("Finish auth setup");
    expect(html).toContain("Auth is not ready for Codex.");
    expect(html).toContain("Needs attention");
    expect(html).not.toContain("Start session");
  });

  it("flags a stale session for recovery", () => {
    const stale = session({
      statusLabel: "Not responding",
      nextAction: {
        kind: "recover",
        label: "Repair session",
        detail:
          "This session's container is no longer running. Repair the stale marker to clear it."
      }
    });

    expect(sessionNeedsAttention(stale)).toBe(true);

    const html = renderToStaticMarkup(<ManagedSessionCard session={stale} />);

    expect(html).toContain("Repair session");
    expect(html).toContain("Not responding");
    expect(html).toContain("Needs attention");
  });

  it("renders explicit auth labels for managed-session statuses", () => {
    const cases: Array<[authStatus: string, label: string]> = [
      ["api-key-broker", "API key broker"],
      ["login-state", "Login state"],
      ["needs-sign-in", "Needs sign-in"],
      ["no-auth-needed", "No auth needed"],
      ["unknown", "Unknown"]
    ];

    for (const [authStatus, label] of cases) {
      const html = renderToStaticMarkup(
        <ManagedSessionCard session={session({ authStatus, events: [] })} />
      );

      expect(authLabel(authStatus)).toBe(label);
      expect(html).toContain(
        `class="mt-1 break-all font-medium">${label}</div>`
      );
    }
  });

  it("allows run controls only while a session is active", () => {
    expect(sessionAllowsRunControl(session({ lifecycle: "running" }))).toBe(true);
    expect(sessionAllowsRunControl(session({ lifecycle: "starting" }))).toBe(true);
    expect(sessionAllowsRunControl(session({ lifecycle: "stop-requested" }))).toBe(true);
    expect(sessionAllowsRunControl(session({ lifecycle: "kill-requested" }))).toBe(true);
    expect(sessionAllowsRunControl(session({ lifecycle: "finished" }))).toBe(false);
    expect(sessionAllowsRunControl(session({ lifecycle: "failed" }))).toBe(false);
  });

  it("allows cleanup only when a session advertises a cleanup target", () => {
    expect(sessionAllowsCleanup(session({ lifecycle: "finished" }))).toBe(false);
    expect(
      sessionAllowsCleanup(
        session({
          lifecycle: "finished",
          cleanup: {
            available: false,
            target: "worktree",
            label: "Clean up worktree",
            detail: "Delete the recorded worktree and branch for this session."
          }
        })
      )
    ).toBe(false);
    expect(
      sessionAllowsCleanup(
        session({
          lifecycle: "finished",
          cleanup: {
            available: true,
            target: "worktree",
            label: "Clean up worktree",
            detail: "Delete the recorded worktree and branch for this session."
          }
        })
      )
    ).toBe(true);
  });

  it("detects real managed-session status changes for quiet refresh", () => {
    const current = { sessions: [session()] };

    expect(sameManagedSessionList(current, { sessions: [session()] })).toBe(true);
    expect(
      sameManagedSessionList(current, {
        sessions: [
          session({
            status: "stop-requested",
            statusLabel: "Stop requested",
            lifecycle: "stop-requested",
            events: [
              {
                source: "run",
                label: "Run record",
                detail: "Stop requested",
                timestamp: "2026-06-30T02:00:01Z"
              }
            ]
          })
        ]
      })
    ).toBe(false);
  });

  it("detects managed-session display field changes for quiet refresh", () => {
    const current = { sessions: [session()] };

    expect(
      sameManagedSessionList(current, {
        sessions: [
          session({
            title: "Codex - renamed",
            project: "renamed",
            session: "renamed",
            startedAt: "2026-06-30T02:00:01Z",
            containerName: "runhaven-codex-renamed",
            detailCommand: "runhaven runs show renamed"
          })
        ]
      })
    ).toBe(false);
  });

  it("detects managed-session input and next-step changes for quiet refresh", () => {
    const current = { sessions: [session()] };

    expect(
      sameManagedSessionList(current, {
        sessions: [
          session({
            input: {
              attached: false,
              status: "detached",
              detail: "Input is not attached for this session."
            },
            nextAction: {
              kind: "auth_attention",
              label: "Finish auth setup",
              detail: "Auth is not ready."
            }
          })
        ]
      })
    ).toBe(false);
  });
});
