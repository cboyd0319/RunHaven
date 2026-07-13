import { fileURLToPath } from "node:url"

import { describe, expect, it } from "vitest"

import { startLoopbackVite } from "../../scripts/loopback-vite.mjs"

const uiRoot = fileURLToPath(new URL("../..", import.meta.url))

describe("OS-assigned loopback server", () => {
  it("gives concurrent servers distinct live URLs", async () => {
    const [first, second] = await Promise.all([
      startLoopbackVite({ root: uiRoot }),
      startLoopbackVite({ root: uiRoot }),
    ])

    try {
      expect(new URL(first.url).hostname).toBe("127.0.0.1")
      expect(new URL(second.url).hostname).toBe("127.0.0.1")
      expect(first.url).not.toBe(second.url)
      expect((await fetch(first.url)).ok).toBe(true)
      expect((await fetch(second.url)).ok).toBe(true)
    } finally {
      await Promise.all([first.close(), second.close()])
    }
  })
})
