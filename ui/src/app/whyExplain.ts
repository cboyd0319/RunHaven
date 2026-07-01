import type {
  HostExplanationResponse,
  NetworkExplanationResponse,
  StateExplanationResponse,
  WorkspaceExplanationResponse,
} from "@/commands/runhaven"

import { safeBackendLabel } from "./labels"

// Pure mapping and label helpers for the read-only "Why is this safe?" panel.
// The backend contracts are already secret-free and path-free; these helpers
// only turn them into beginner-friendly labels and drop empty rows. No secret,
// token, or host path is ever produced here.

export type ExplainRow = {
  label: string
  value: string
}

export type ExplainList = {
  label: string
  items: string[]
}

// Turn a backend decision string into a plain-language verdict word.
export function decisionLabel(decision: string): string {
  switch (decision.trim().toLowerCase()) {
    case "allowed":
      return "Allowed"
    case "not-allowed":
      return "Not allowed"
    case "denied":
      return "Blocked"
    case "":
      return "Unknown"
    default:
      return safeBackendLabel(decision, "Unknown")
  }
}

export function decisionBadgeVariant(
  decision: string,
): "destructive" | "secondary" | "success" | "warning" {
  switch (decision.trim().toLowerCase()) {
    case "allowed":
      return "success"
    case "not-allowed":
      return "warning"
    case "denied":
      return "destructive"
    default:
      return "secondary"
  }
}

// Drop rows whose value is empty so the panel shows only what applies.
function rows(entries: ExplainRow[]): ExplainRow[] {
  return entries.filter((row) => row.value.trim().length > 0)
}

// Drop lists that have no items.
function lists(entries: ExplainList[]): ExplainList[] {
  return entries.filter((list) => list.items.length > 0)
}

export function networkRows(response: NetworkExplanationResponse): ExplainRow[] {
  return rows([
    { label: "What it does", value: response.behavior },
    { label: "Boundary", value: response.boundary },
    { label: "Use when", value: response.useWhen },
    { label: "DNS safety", value: response.dnsSafety },
    { label: "Inspect", value: response.inspect },
  ])
}

export function stateRows(response: StateExplanationResponse): ExplainRow[] {
  return rows([
    { label: "Default session", value: response.defaultSession },
    { label: "Home mount", value: response.homeMount },
    { label: "State volume", value: response.stateVolumePattern },
    { label: "Named session", value: response.namedSessionPattern },
    { label: "Boundary", value: response.boundary },
    { label: "Project id", value: response.projectId },
    { label: "Manage", value: response.manage },
  ])
}

export function hostRows(response: HostExplanationResponse): ExplainRow[] {
  return rows([
    { label: "Reason", value: response.reason },
    { label: "Next step", value: response.nextAction },
    { label: "DNS safety", value: response.dnsSafety },
  ])
}

export function hostLists(response: HostExplanationResponse): ExplainList[] {
  return lists([
    { label: "Matched rules", items: response.matchedRules },
    { label: "Bundled hosts", items: response.bundledHosts },
    { label: "Known endpoints", items: response.endpointNotes },
  ])
}

export function workspaceRows(response: WorkspaceExplanationResponse): ExplainRow[] {
  return rows([
    { label: "Scope", value: response.scope },
    { label: "Next step", value: response.nextAction },
    { label: "Boundary", value: response.boundary },
  ])
}
