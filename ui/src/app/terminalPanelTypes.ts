import type {
  LaunchRunRequest,
  ManagedSessionSummary,
  RunTerminalOpenResponse,
  TerminalMessage,
} from "@/commands/runhaven"
import type { Ref } from "react"

// Bounded in-WebView scrollback. Raw terminal bytes live only in memory for
// the life of the view and are never written to disk or logged.
export const SCROLLBACK_LINES = 1000

export type TerminalPhase = "idle" | "opening" | "live" | "ended" | "error"

export type TerminalInteractionError = {
  title: string
  detail: string
}

export type TerminalOpen = (
  sessionId: string,
  onMessage: (message: TerminalMessage) => void,
) => Promise<boolean | RunTerminalOpenResponse | null>

export type TerminalPanelProps =
  | { session: ManagedSessionSummary; login?: undefined; run?: undefined }
  | {
      login: {
        agent: string
        agentLabel: string
        sessionId: string
        onSignedIn?: () => void
      }
      session?: undefined
      run?: undefined
    }
  | {
      run: {
        sessionId: string
        request: LaunchRunRequest
        label?: string
      }
      session?: undefined
      login?: undefined
      fill?: boolean
      onClose?: () => void
      onOpenCancelled?: () => void
      onOpened?: (opened: RunTerminalOpenResponse) => void
      onEnded?: () => void
      visible?: boolean
      headingRef?: Ref<HTMLDivElement>
    }
