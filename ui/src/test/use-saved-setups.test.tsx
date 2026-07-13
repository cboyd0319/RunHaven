// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from "vitest"

import { useSavedSetups } from "../app/useSavedSetups"
import { act, renderHook } from "./render-hook"
import { planRequest } from "./main-view-props"

const commands = vi.hoisted(() => ({
  chooseProjectFolder: vi.fn(),
  getSavedSetups: vi.fn(),
  pinSavedSetup: vi.fn(),
  selectSavedSetup: vi.fn(),
  setSavedSetupPinned: vi.fn(),
}))

vi.mock("@/commands/runhaven", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../commands/runhaven")>()),
  ...commands,
}))

describe("useSavedSetups", () => {
  beforeEach(() => {
    vi.clearAllMocks()
    commands.getSavedSetups.mockResolvedValue([
      {
        id: "setup-a",
        assistant: "codex",
        project: "RunHaven",
        name: "Codex · RunHaven",
        available: true,
        pinned: true,
      },
    ])
    commands.selectSavedSetup.mockResolvedValue({
      ...planRequest(),
      workspacePath: "/private/project",
      worktree: true,
    })
    commands.pinSavedSetup.mockResolvedValue("setup-a")
    commands.chooseProjectFolder.mockResolvedValue("/private/repaired-project")
    commands.setSavedSetupPinned.mockResolvedValue(undefined)
  })

  it("loads pinned rows and selecting one only applies the secure draft", async () => {
    const applyRequest = vi.fn()
    const navigate = vi.fn()
    const hook = await renderHook(
      () => useSavedSetups({ applyRequest, navigate }),
      undefined,
    )
    await act(async () => {})

    expect(hook.result.current.pinnedSetups).toHaveLength(1)
    await act(async () => hook.result.current.selectSetup("setup-a"))
    expect(commands.selectSavedSetup).toHaveBeenCalledWith("setup-a")
    expect(applyRequest).toHaveBeenCalledWith(
      expect.objectContaining({
        workspacePath: "/private/project",
        worktree: true,
        sessionName: null,
      }),
    )
    expect(navigate).toHaveBeenCalledWith({ kind: "new-session" })
  })

  it("pins the current assistant and project, then can unpin it", async () => {
    const hook = await renderHook(
      () => useSavedSetups({ applyRequest: vi.fn(), navigate: vi.fn() }),
      undefined,
    )
    await act(async () => {})

    await act(async () => hook.result.current.pinDraft("codex", "/private/project"))
    expect(commands.pinSavedSetup).toHaveBeenCalledWith(
      "codex",
      "/private/project",
    )

    await act(async () => hook.result.current.unpin("setup-a"))
    expect(commands.setSavedSetupPinned).toHaveBeenCalledWith("setup-a", false)
  })

  it("repairs an unavailable setup with its assistant and clears the dead-end error", async () => {
    commands.getSavedSetups.mockResolvedValueOnce([
      {
        id: "missing-setup",
        assistant: "claude",
        project: "Moved project",
        name: "Claude · Moved project",
        available: false,
        pinned: true,
      },
    ])
    commands.pinSavedSetup.mockResolvedValue("repaired-setup")
    commands.selectSavedSetup.mockResolvedValue({
      ...planRequest(),
      agent: "claude",
      workspacePath: "/private/repaired-project",
    })
    const applyRequest = vi.fn()
    const hook = await renderHook(
      () => useSavedSetups({ applyRequest, navigate: vi.fn() }),
      undefined,
    )
    await act(async () => {})

    await act(async () => hook.result.current.selectSetup("missing-setup"))
    expect(hook.result.current.error).toContain("Project folder can’t be found")

    await act(async () => hook.result.current.repairUnavailable())
    expect(commands.pinSavedSetup).toHaveBeenCalledWith(
      "claude",
      "/private/repaired-project",
    )
    expect(commands.setSavedSetupPinned).toHaveBeenCalledWith(
      "missing-setup",
      false,
    )
    expect(commands.selectSavedSetup).toHaveBeenCalledWith("repaired-setup")
    expect(applyRequest).toHaveBeenCalledWith(
      expect.objectContaining({
        agent: "claude",
        workspacePath: "/private/repaired-project",
      }),
    )
    expect(hook.result.current.error).toBe("")
    expect(hook.result.current.message).toBe("Setup updated.")
  })
})
