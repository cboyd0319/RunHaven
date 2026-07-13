import type { RunPlanRequest } from "@/commands/runhaven"

export type RunControlAction = "stop" | "kill" | "repair" | "cleanup"

// The New Session page's three intent-capture steps. An interactive agent
// (Claude Code) confirmed at "review" does not add a fourth step: it opens the run
// terminal as the run-terminal main-content destination (a live coding session
// belongs in the bounded content area).
export type LaunchStep = "agent" | "project" | "review"

export function networkHelp(mode: RunPlanRequest["networkMode"]): string {
  if (mode === "provider") {
    return "Only reviewed provider hosts."
  }
  if (mode === "internal") {
    return "No internet from the run."
  }
  return "Allowed only after warning."
}

export function formatMemory(bytes: number | null): string {
  if (bytes === null) {
    return "unknown"
  }
  return `${Math.round(bytes / 1024 ** 2)} MiB`
}

export function errorMessage(cause: unknown): string {
  const message = cause instanceof Error ? cause.message : String(cause)
  return isNativeConfirmationCancellation(message) ? "" : message
}

const cancellationPrefixes = [
  "Agent installation canceled",
  "Cleanup canceled",
  "Discard canceled",
  "Hard-stop canceled",
  "Keep canceled",
  "Launch canceled",
  "Merge canceled",
  "Network cleanup canceled",
  "Raw output view canceled",
  "Recovery review canceled",
  "Repair canceled",
  "Setup recovery canceled",
  "Stop canceled",
  "Terminal canceled",
]

function isNativeConfirmationCancellation(message: string): boolean {
  return cancellationPrefixes.some((prefix) => message.startsWith(prefix))
}
