import fs from "node:fs"
import { fileURLToPath } from "node:url"

import config from "../../playwright.config"
import viteConfig from "../../vite.config"
import { describe, expect, it } from "vitest"

const repoRoot = fileURLToPath(new URL("../../..", import.meta.url))

describe("Playwright loopback server", () => {
  it("does not reserve a fixed local port in configuration", () => {
    expect(config.webServer).toBeUndefined()
    expect(config.use?.baseURL).toBeUndefined()
  })

  it("keeps manual and Tauri debug launches off fixed local ports", () => {
    expect(viteConfig.server?.port).toBe(0)
    expect(viteConfig.preview?.port).toBe(0)

    const tauriConfig = JSON.parse(
      fs.readFileSync(`${repoRoot}/src-tauri/tauri.conf.json`, "utf8"),
    ) as { build?: { beforeDevCommand?: unknown; devUrl?: unknown } }
    expect(tauriConfig.build?.beforeDevCommand).toBeUndefined()
    expect(tauriConfig.build?.devUrl).toBeUndefined()

    const launcher = fs.readFileSync(
      `${repoRoot}/scripts/dev-with-logs.sh`,
      "utf8",
    )
    expect(launcher).toContain("ui/scripts/tauri-dev.mjs")
  })
})
