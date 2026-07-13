// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { useImageRebuild } from "../app/useMaintenance"
import type {
  ImageBuildStatusResponse,
} from "../commands/runhaven"
import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return {
    ...actual,
    rebuildAgentImage: vi.fn(),
    getImageBuildStatus: vi.fn(),
  }
})

const commands = await import("@/commands/runhaven")
const rebuildImage = vi.mocked(commands.rebuildAgentImage)
const getBuildStatus = vi.mocked(commands.getImageBuildStatus)

beforeEach(() => {
  vi.useFakeTimers()
  rebuildImage.mockReset()
  getBuildStatus.mockReset()
})

afterEach(() => {
  vi.useRealTimers()
})

function buildStatus(state: string): ImageBuildStatusResponse {
  return {
    buildId: "b1",
    agent: "codex",
    tag: "runhaven/codex:0.7.0",
    state,
    label: state === "succeeded" ? "Succeeded" : "Building",
    detail: state === "succeeded" ? "Done." : "Building.",
    outputTail: "",
    updatedAt: state === "succeeded" ? "2026-07-12T01:00:01Z" : "2026-07-12T01:00:00Z",
  }
}

describe("useImageRebuild", () => {
  it("polls until terminal, guards equal status, and stops after unmount", async () => {
    rebuildImage.mockResolvedValue({
      buildId: "b1",
      agent: "codex",
      tag: "runhaven/codex:0.7.0",
      state: "building",
      label: "Building",
      detail: "Queued.",
    })
    getBuildStatus
      .mockResolvedValueOnce(buildStatus("building"))
      .mockResolvedValueOnce(buildStatus("building"))
      .mockResolvedValue(buildStatus("succeeded"))

    const hook = await renderHook(() => useImageRebuild(), undefined)
    await act(async () => {
      await hook.result.current.rebuild("codex")
    })

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    const buildingStatus = hook.result.current.status
    expect(buildingStatus?.state).toBe("building")

    // Second poll returns identical content: reference stays stable.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    expect(hook.result.current.status).toBe(buildingStatus)

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500) // succeeded
    })
    expect(hook.result.current.status?.state).toBe("succeeded")

    const callsAfterTerminal = getBuildStatus.mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6_000)
    })
    expect(getBuildStatus.mock.calls.length).toBe(callsAfterTerminal)
    await hook.unmount()
  })
})
