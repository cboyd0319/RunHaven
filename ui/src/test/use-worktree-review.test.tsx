// @vitest-environment happy-dom
import { describe, expect, it, vi } from "vitest"

import { useWorktreeReview } from "../app/useWorktreeReview"
import type { WorktreeReviewResponse } from "../commands/runhaven"

import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return { ...actual, getManagedSessionWorktree: vi.fn() }
})

const { getManagedSessionWorktree } = await import("@/commands/runhaven")
const getReview = vi.mocked(getManagedSessionWorktree)

const review: WorktreeReviewResponse = {
  runId: "run-1",
  branch: "runhaven/codex/run-1",
  statusSummary: "One file change is ready to review.",
  worktreeChanges: 1,
  sourceChanges: 0,
  clean: false,
  fileChanges: [{ path: "README.md", kind: "changed" }],
  fileChangesTruncated: false,
  sourceFileChanges: [],
  sourceFileChangesTruncated: false,
  reviewId: "0".repeat(64),
  availableActions: ["merge", "recover", "discard"],
}

async function flushReviewLoad() {
  await act(async () => {
    await Promise.resolve()
    await Promise.resolve()
  })
}

describe("useWorktreeReview", () => {
  it("refreshes the session summary after an explicit retry succeeds", async () => {
    getReview.mockReset()
    getReview
      .mockRejectedValueOnce(new Error("Could not read the protected copy."))
      .mockResolvedValueOnce(review)
    const onResolved = vi.fn().mockResolvedValue(undefined)
    const hook = await renderHook(
      (sessionId: string) => useWorktreeReview(sessionId, onResolved),
      "run-1",
    )
    await flushReviewLoad()
    expect(hook.result.current.loadError).toContain("Could not read")
    expect(onResolved).not.toHaveBeenCalled()

    await act(async () => hook.result.current.retry())
    await flushReviewLoad()

    expect(getReview).toHaveBeenCalledTimes(2)
    expect(onResolved).toHaveBeenCalledTimes(1)
    expect(hook.result.current.review).toEqual(review)
    await hook.unmount()
  })
})
