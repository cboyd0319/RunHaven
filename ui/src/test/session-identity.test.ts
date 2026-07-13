import { describe, expect, it } from "vitest"

import {
  agentLabel,
  displayValue,
  sessionIdentityLabel,
  sessionInputLabel,
  titleCase,
} from "../app/sessionIdentity"
import type { ManagedSessionSummary } from "../commands/runhaven"

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
    startedAt: "2026-07-01T02:00:00Z",
    containerName: "runhaven-codex-project",
    detailCommand: "runhaven runs show run-123",
    lifecycle: "running",
    input: {
      attached: true,
      status: "attached",
      detail: "Session input is ready.",
    },
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking this session.",
    },
    events: [],
    ...patch,
  }
}

describe("displayValue", () => {
  it("keeps trimmed non-empty values", () => {
    expect(displayValue("  codex  ")).toBe("codex")
  })

  it("falls back to Unknown for empty or whitespace input", () => {
    expect(displayValue("")).toBe("Unknown")
    expect(displayValue("   ")).toBe("Unknown")
  })
})

describe("titleCase", () => {
  it("title-cases hyphen and space separated words", () => {
    expect(titleCase("api-key broker")).toBe("Api Key Broker")
  })

  it("collapses repeated whitespace", () => {
    expect(titleCase("first    second")).toBe("First Second")
  })

  it("returns Unknown for empty input", () => {
    expect(titleCase("   ")).toBe("Unknown")
  })

  it("agentLabel delegates to titleCase", () => {
    expect(agentLabel("claude-code")).toBe("Claude Code")
  })
})

describe("sessionIdentityLabel", () => {
  it("shows project only when the session name is unknown", () => {
    expect(sessionIdentityLabel(session({ project: "web", session: "" }))).toBe(
      "Project web",
    )
  })

  it("shows project only when the session matches the project case-insensitively", () => {
    expect(sessionIdentityLabel(session({ project: "web", session: "WEB" }))).toBe(
      "Project web",
    )
  })

  it("shows both segments when project and session differ", () => {
    expect(
      sessionIdentityLabel(session({ project: "web", session: "feature" })),
    ).toBe("Project web / Session feature")
  })

  it("reduces path-like project and session values to their last segment", () => {
    expect(
      sessionIdentityLabel(
        session({ project: "/repos/web", session: "branches/feature" }),
      ),
    ).toBe("Project web / Session feature")
  })
})

describe("sessionInputLabel", () => {
  it("reports readiness from the attached flag", () => {
    expect(sessionInputLabel(session({ input: { attached: true, status: "attached", detail: "" } }))).toBe(
      "Messages available",
    )
    expect(
      sessionInputLabel(session({ input: { attached: false, status: "detached", detail: "" } })),
    ).toBe("Messages unavailable")
  })
})
