import { describe, expect, it } from "vitest"

import {
  activeTerminalWorkspace,
  terminalWorkspacePageTitle,
  terminalWorkspaceTitle,
} from "../app/terminalWorkspace"

const terminals = [
  {
    sessionId: "terminal-a",
    runId: "run-a",
    label: "Claude · RunHaven",
    startedAt: "2026-07-13T18:00:00Z",
  },
  {
    sessionId: "terminal-b",
    request: {
      plan: { agent: "codex", workspacePath: "/private/pro\u202eject" },
    },
    startedAt: "2026-07-13T18:01:00Z",
  },
]

describe("terminal workspace", () => {
  it("selects the requested live terminal or the newest terminal", () => {
    expect(activeTerminalWorkspace(terminals, "run-a")?.sessionId).toBe(
      "terminal-a",
    )
    expect(activeTerminalWorkspace(terminals, "terminal-b")?.sessionId).toBe(
      "terminal-b",
    )
    expect(activeTerminalWorkspace(terminals)?.sessionId).toBe("terminal-b")
    expect(activeTerminalWorkspace(terminals, "missing")).toBeNull()
  })

  it("uses assistant and project identity instead of a generic page title", () => {
    expect(terminalWorkspaceTitle(terminals[0])).toBe("Claude · RunHaven")
    expect(terminalWorkspaceTitle(terminals[1])).toBe("Codex · project")
    expect(
      terminalWorkspacePageTitle(
        { kind: "run-terminal", id: "run-a" },
        terminals,
        "Terminal workspace",
      ),
    ).toBe("Claude · RunHaven")
    expect(
      terminalWorkspacePageTitle(
        { kind: "new-session" },
        terminals,
        "New session",
      ),
    ).toBe("New session")
  })
})
