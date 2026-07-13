import type { RunPlanRequest } from "@/commands/runhaven"

export type RunControlAction = "stop" | "kill" | "repair" | "cleanup"

// New Session asks for an assistant only when opened directly, then asks for a
// project. The trusted native sheet owns the one final review.
export type LaunchStep = "agent" | "project"

export function networkHelp(mode: RunPlanRequest["networkMode"]): string {
  if (mode === "provider") {
    return "The assistant can reach only the services it needs."
  }
  if (mode === "internal") {
    return "No internet access."
  }
  return "The assistant can reach any site. This is less safe."
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
  "Apply changes canceled",
  "Cleanup canceled",
  "Discard changes canceled",
  "Force Stop canceled",
  "Launch canceled",
  "Message send canceled",
  "Network cleanup canceled",
  "Raw output view canceled",
  "Remove canceled",
  "Recovery steps canceled",
  "Save canceled",
  "Setup recovery canceled",
  "Stop canceled",
  "Terminal canceled",
]

function isNativeConfirmationCancellation(message: string): boolean {
  return cancellationPrefixes.some((prefix) => message.startsWith(prefix))
}
