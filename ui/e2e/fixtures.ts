import { fileURLToPath } from "node:url"
import {
  expect,
  test as base,
  type Page,
} from "@playwright/test"
import { startLoopbackVite } from "../scripts/loopback-vite.mjs"

type WorkerFixtures = {
  runhavenServer: Awaited<ReturnType<typeof startLoopbackVite>>
}

const test = base.extend<Record<string, never>, WorkerFixtures>({
  runhavenServer: [
    async ({}, use) => {
      const server = await startLoopbackVite({
        root: fileURLToPath(new URL("..", import.meta.url)),
      })

      try {
        await use(server)
      } finally {
        await server.close()
      }
    },
    { scope: "worker" },
  ],
  baseURL: async ({ runhavenServer }, use) => {
    await use(runhavenServer.url)
  },
})

export { expect, test }
export type { Page }
