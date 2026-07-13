import type { RunPlanRequest } from "@/commands/runhaven"

import { displayValue, titleCase } from "./sessionIdentity"

// Canonical user-facing label vocabulary for RunHaven. One source per concept so
// the sidebar, launch flow, session cards, and access summaries all speak with
// the same words. The backend contracts feeding these are already secret-free;
// no token, host path, or raw environment value is ever produced here.

// A backend string is only shown when it looks like a plain, bounded label. This
// keeps an unexpected or malformed backend value from rendering as-is; garbage
// falls back to a safe constant instead. Guard only: a value that passes is
// returned unchanged so real labels ("Stop requested", "runhaven login") keep
// their exact text.
const SAFE_LABEL = /^[a-z0-9][a-z0-9 _-]{0,39}$/i

export function safeBackendLabel(value: string, fallback: string): string {
  const trimmed = value.trim()
  return SAFE_LABEL.test(trimmed) ? trimmed : fallback
}

// Prettify an unmapped auth-status code: guard the raw value first, then title
// case it so "custom_pending" reads as "Custom Pending" while unsafe input still
// collapses to "Unknown".
function prettyBackendLabel(value: string): string {
  const safe = safeBackendLabel(value, "")
  return safe ? titleCase(safe.replaceAll("_", " ")) : "Unknown"
}

export const networkLabels: Record<RunPlanRequest["networkMode"], string> = {
  provider: "Provider only",
  internal: "Local only",
  internet: "Internet",
}

export function networkLabel(network: string): string {
  switch (network.trim().toLowerCase()) {
    case "provider":
      return networkLabels.provider
    case "internal":
      return networkLabels.internal
    case "internet":
      return networkLabels.internet
    default:
      return displayValue(network)
  }
}

export const authScopeLabels: Record<RunPlanRequest["authScope"], string> = {
  agent: "Shared agent sign-in",
  project: "This project only",
}

export function authScopeLabel(scope: string): string {
  switch (scope.trim().toLowerCase()) {
    case "agent":
      return authScopeLabels.agent
    case "project":
      return authScopeLabels.project
    default:
      return displayValue(scope)
  }
}

// Canonical auth-status vocabulary shared by the session views and the per-agent
// auth summary, so both speak about auth state with one set of words.
export function authStatusLabel(status: string): string {
  switch (status.trim().toLowerCase()) {
    case "api-key-broker":
    case "brokered":
      return "API key broker"
    case "keychain-oauth":
      return "Legacy Claude Keychain auth"
    case "login-state":
      return "Isolated login"
    case "needs-sign-in":
      return "Needs sign-in"
    case "no-auth-needed":
    case "n/a":
      return "No sign-in needed"
    case "ready":
    case "available":
      return "Ready"
    case "design-only":
      return "Not ready"
    case "":
    case "-":
    case "unknown":
      return "Unknown"
    default:
      return prettyBackendLabel(status)
  }
}
