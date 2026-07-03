import type {
  AuthLogResponse,
  AuthStatusResponse,
  EgressLogResponse,
  RunPlanRequest,
} from "@/commands/runhaven"

export type DiagnosticsState = {
  authStatus: AuthStatusResponse
  authLog: AuthLogResponse
  egressLog: EgressLogResponse
}

export type RunControlAction = "stop" | "kill" | "repair" | "cleanup"

// The New Run sheet's three intent-capture steps. An interactive agent (Claude
// Code) confirmed at "review" does not add a fourth step: it opens the run
// terminal as the run-terminal main-content destination (a live coding session
// belongs in the bounded content area, not inside the modal).
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
  return cause instanceof Error ? cause.message : String(cause)
}
