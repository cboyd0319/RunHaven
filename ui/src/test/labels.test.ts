import { describe, expect, it } from "vitest"

import {
  authScopeLabel,
  authStatusLabel,
  networkLabel,
  safeBackendLabel,
} from "../app/labels"

describe("networkLabel", () => {
  it("maps known network modes case-insensitively", () => {
    expect(networkLabel("provider")).toBe("Provider only")
    expect(networkLabel("  INTERNAL ")).toBe("Local only")
    expect(networkLabel("Internet")).toBe("Internet")
  })

  it("falls back to a display value for unknown modes", () => {
    expect(networkLabel("bridge")).toBe("bridge")
    expect(networkLabel("")).toBe("Unknown")
  })
})

describe("authScopeLabel", () => {
  it("maps known scopes to the canonical login wording", () => {
    expect(authScopeLabel("agent")).toBe("Shared agent login")
    expect(authScopeLabel(" Project ")).toBe("Project session")
  })

  it("falls back to a display value for unknown scopes", () => {
    expect(authScopeLabel("session")).toBe("session")
    expect(authScopeLabel("")).toBe("Unknown")
  })
})

describe("authStatusLabel", () => {
  it("maps broker aliases to a single label", () => {
    expect(authStatusLabel("api-key-broker")).toBe("API key broker")
    expect(authStatusLabel("brokered")).toBe("API key broker")
  })

  it("names the Claude Keychain OAuth broker distinctly", () => {
    expect(authStatusLabel("keychain-oauth")).toBe(
      "Claude subscription (Keychain)",
    )
  })

  it("maps no-auth, ready, and design-only states", () => {
    expect(authStatusLabel("no-auth-needed")).toBe("No auth needed")
    expect(authStatusLabel("n/a")).toBe("No auth needed")
    expect(authStatusLabel("ready")).toBe("Ready")
    expect(authStatusLabel("available")).toBe("Ready")
    expect(authStatusLabel("design-only")).toBe("Not ready")
  })

  it("maps empty, dash, and unknown sentinels to Unknown", () => {
    expect(authStatusLabel("")).toBe("Unknown")
    expect(authStatusLabel("-")).toBe("Unknown")
    expect(authStatusLabel("unknown")).toBe("Unknown")
  })

  it("title-cases safe custom statuses with underscores", () => {
    expect(authStatusLabel("custom_pending")).toBe("Custom Pending")
  })

  it("redacts unsafe status strings", () => {
    expect(authStatusLabel("<script>alert(1)</script>")).toBe("Unknown")
    expect(authStatusLabel("has\nnewline")).toBe("Unknown")
  })

  it("redacts over-long status strings", () => {
    expect(authStatusLabel("a".repeat(41))).toBe("Unknown")
    expect(authStatusLabel("a".repeat(40))).toBe(`A${"a".repeat(39)}`)
  })
})

describe("safeBackendLabel", () => {
  it("passes bounded plain labels through unchanged", () => {
    expect(safeBackendLabel("Stop requested", "Unknown")).toBe("Stop requested")
    expect(safeBackendLabel("  runhaven login  ", "Unknown")).toBe("runhaven login")
  })

  it("falls back for unsafe or over-long input", () => {
    expect(safeBackendLabel("/Users/secret/TOKEN=value", "Unknown")).toBe("Unknown")
    expect(safeBackendLabel("has\nnewline", "Unknown")).toBe("Unknown")
    expect(safeBackendLabel("a".repeat(41), "Unknown")).toBe("Unknown")
  })
})
