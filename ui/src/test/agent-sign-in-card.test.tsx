// @vitest-environment happy-dom
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { AgentSignIn } from "../app/AgentSignIn"
import type { AgentProfile } from "../commands/runhaven"

function agent(name: string): AgentProfile {
  return {
    name,
    description: `${name} agent.`,
    image: `runhaven/${name}:0.7.0`,
    defaultCommand: [name],
    providerHosts: [],
    signIn: name === "shell" ? "n/a" : "runhaven login",
    broker: name === "shell" ? "n/a" : "yes",
    supportsInAppSignIn: name !== "shell",
  }
}

describe("AgentSignIn", () => {
  it.each(["claude", "codex", "antigravity", "copilot"])(
    "uses the native sign-in terminal for %s",
    (name) => {
      const html = renderToStaticMarkup(<AgentSignIn agent={agent(name)} />)
      expect(html).toContain("sign-in terminal")
      expect(html).toContain("The sign-in terminal needs the desktop app")
      expect(html).not.toContain("Sign in on your Mac")
      expect(html).not.toContain("continues in the background")
      expect(html).not.toContain("<input")
    },
  )

  it("shows nothing when the selected profile has no sign-in", () => {
    expect(renderToStaticMarkup(<AgentSignIn agent={agent("shell")} />)).toBe("")
  })

  it("shows nothing without a selected agent", () => {
    expect(renderToStaticMarkup(<AgentSignIn agent={undefined} />)).toBe("")
  })
})
