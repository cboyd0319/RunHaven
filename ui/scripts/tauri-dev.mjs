#!/usr/bin/env node
import { spawn } from "node:child_process"
import path from "node:path"
import { fileURLToPath } from "node:url"

import { startLoopbackVite } from "./loopback-vite.mjs"

const uiRoot = fileURLToPath(new URL("..", import.meta.url))
const repoRoot = fileURLToPath(new URL("../..", import.meta.url))
const tauri = path.join(uiRoot, "node_modules", ".bin", "tauri")
const server = await startLoopbackVite({ root: uiRoot, logLevel: "info" })
const override = JSON.stringify({
  build: {
    beforeDevCommand: null,
    devUrl: server.url,
  },
})

console.log(`===== RunHaven Debug UI ${server.url} (OS-assigned loopback port) =====`)

let child
let relayedSignal
const signalHandlers = new Map()

function relay(signal) {
  relayedSignal = signal
  if (child && child.exitCode === null && child.signalCode === null) {
    child.kill(signal)
  }
}

try {
  child = spawn(
    tauri,
    [
      "dev",
      "--config",
      "src-tauri/tauri.conf.json",
      "--config",
      "src-tauri/tauri.dev.conf.json",
      "--config",
      override,
    ],
    {
      cwd: repoRoot,
      env: process.env,
      stdio: "inherit",
    },
  )

  for (const signal of ["SIGINT", "SIGTERM"]) {
    const handler = () => relay(signal)
    signalHandlers.set(signal, handler)
    process.once(signal, handler)
  }

  const result = await new Promise((resolve, reject) => {
    child.once("error", reject)
    child.once("exit", (code, signal) => resolve({ code, signal }))
  })

  if (result.code !== null) {
    process.exitCode = result.code
  } else if (result.signal || relayedSignal) {
    process.exitCode = relayedSignal === "SIGTERM" ? 143 : 130
  }
} finally {
  for (const [signal, handler] of signalHandlers) {
    process.off(signal, handler)
  }
  await server.close()
}
