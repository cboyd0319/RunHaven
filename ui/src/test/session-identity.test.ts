import { describe, expect, it } from "vitest"

import {
  agentLabel,
  authLabel,
  authScopeLabel,
  displayValue,
  networkLabel,
  projectLabel,
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

describe("networkLabel", () => {
  it("maps known network modes case-insensitively", () => {
    expect(networkLabel("provider")).toBe("Provider only")
    expect(networkLabel("  INTERNAL ")).toBe("Local only")
    expect(networkLabel("Internet")).toBe("Internet")
  })

  it("falls back to a display value for unknown modes", () => {
    expect(networkLabel("bridge")).toBe("bridge")
    expect(networkLabel("")).toBe("Unknown")
  })
})

describe("authScopeLabel", () => {
  it("maps known scopes", () => {
    expect(authScopeLabel("agent")).toBe("Shared agent login")
    expect(authScopeLabel(" Project ")).toBe("Project")
  })

  it("falls back to a display value for unknown scopes", () => {
    expect(authScopeLabel("session")).toBe("session")
    expect(authScopeLabel("")).toBe("Unknown")
  })
})

describe("authLabel", () => {
  it("maps broker aliases to a single label", () => {
    expect(authLabel("api-key-broker")).toBe("API key broker")
    expect(authLabel("brokered")).toBe("API key broker")
  })

  it("maps no-auth and ready aliases", () => {
    expect(authLabel("no-auth-needed")).toBe("No auth needed")
    expect(authLabel("n/a")).toBe("No auth needed")
    expect(authLabel("ready")).toBe("Ready")
    expect(authLabel("available")).toBe("Ready")
  })

  it("maps empty, dash, and unknown sentinels to Unknown", () => {
    expect(authLabel("")).toBe("Unknown")
    expect(authLabel("-")).toBe("Unknown")
    expect(authLabel("unknown")).toBe("Unknown")
  })

  it("title-cases safe custom statuses with underscores", () => {
    expect(authLabel("custom_pending")).toBe("Custom Pending")
  })

  it("redacts unsafe status strings", () => {
    expect(authLabel("<script>alert(1)</script>")).toBe("Unknown")
    expect(authLabel("has\nnewline")).toBe("Unknown")
  })

  it("redacts over-long status strings", () => {
    expect(authLabel("a".repeat(41))).toBe("Unknown")
    expect(authLabel("a".repeat(40))).toBe(`A${"a".repeat(39)}`)
  })
})

describe("projectLabel", () => {
  it("prefixes a project name", () => {
    expect(projectLabel("web")).toBe("Project web")
  })

  it("uses the final path segment for path-like projects", () => {
    expect(projectLabel("/Users/dev/web")).toBe("Project web")
  })

  it("labels empty projects as Unknown", () => {
    expect(projectLabel("")).toBe("Project Unknown")
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
      "Input ready",
    )
    expect(
      sessionInputLabel(session({ input: { attached: false, status: "detached", detail: "" } })),
    ).toBe("Input blocked")
  })
})
