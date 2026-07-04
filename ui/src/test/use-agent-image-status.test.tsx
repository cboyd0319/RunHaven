// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from "vitest"

import { useAgentImageStatus } from "../app/views/AgentsView"
import type { ImageStatusResponse } from "../commands/runhaven"

import { renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return { ...actual, getImageStatus: vi.fn() }
})

const { getImageStatus } = await import("@/commands/runhaven")
const getImage = vi.mocked(getImageStatus)

function imageStatus(ready: boolean): ImageStatusResponse {
  return {
    agent: "codex",
    image: {
      agent: "codex",
      image: "runhaven/codex:0.1.0",
      status: ready ? "ready" : "missing",
      ready,
      expectedSourceDigest: "sha256:expected",
      localSourceDigest: ready ? "sha256:expected" : null,
      fixCommand: ready ? null : "runhaven images build codex",
    },
    builder: {
      status: "running",
      detail: "",
      image: null,
      cpus: null,
      memory: null,
      rosetta: null,
      startedDate: null,
      ipv4Address: null,
      warning: null,
    },
  }
}

beforeEach(() => {
  getImage.mockReset()
})

describe("useAgentImageStatus", () => {
  it("reads image status on mount and re-reads when the reload key bumps", async () => {
    getImage.mockResolvedValueOnce(imageStatus(false))
    const hook = await renderHook(
      (reloadKey: number) => useAgentImageStatus("codex", reloadKey),
      0 as number,
    )
    expect(getImage).toHaveBeenCalledTimes(1)
    expect(hook.result.current.status?.image.ready).toBe(false)

    // A rebuild finished: "Check again" bumps the key, so the stale row re-reads
    // and now sees the ready image.
    getImage.mockResolvedValueOnce(imageStatus(true))
    await hook.rerender(1)
    expect(getImage).toHaveBeenCalledTimes(2)
    expect(hook.result.current.status?.image.ready).toBe(true)
    await hook.unmount()
  })
})
