import { TriangleAlert } from "lucide-react"

import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

export type TerminalConfig = {
  sessionId: string
  title: string
  description: string
  openLabel: string
  previewText: string
  connectedText: string
  endedText: string
}

type TerminalPhase = "idle" | "opening" | "live" | "ended" | "error"

export function terminalStatusText(
  phase: TerminalPhase,
  connectedText: string,
  endedText: string,
  escapeTarget: string,
): string {
  switch (phase) {
    case "opening":
      return "Confirm the terminal in the dialog to start."
    case "live":
      return `${connectedText} To leave with the keyboard, press Shift+Esc to move to the ${escapeTarget} button.`
    case "ended":
      return endedText
    default:
      return ""
  }
}

export function attachTerminalConfig(
  session: ManagedSessionSummary,
): TerminalConfig {
  return {
    sessionId: session.id,
    title: "Terminal",
    description: "Type directly in this protected session.",
    openLabel: "Open terminal",
    previewText:
      "Terminal needs the desktop app. Open RunHaven on your Mac to type live with this session.",
    connectedText: "Connected. What you type goes directly to this session.",
    endedText: "This terminal ended.",
  }
}

export function loginTerminalConfig(
  agentLabel: string,
  sessionId: string,
): TerminalConfig {
  return {
    sessionId,
    title: `${agentLabel} sign-in terminal`,
    description:
      `Sign in to ${agentLabel} here. Follow the instructions in the terminal and finish in your browser.`,
    openLabel: "Open sign-in terminal",
    previewText:
      `The sign-in terminal needs the desktop app. Open RunHaven on your Mac to sign in to ${agentLabel}.`,
    connectedText:
      "Connected. Follow the instructions in the terminal to finish sign-in.",
    endedText: "The sign-in terminal ended.",
  }
}

export function runTerminalConfig(sessionId: string): TerminalConfig {
  return {
    sessionId,
    title: "Session terminal",
    description:
      "Type your task here. RunHaven starts the assistant in a protected session and connects you to it.",
    openLabel: "Start session",
    previewText:
      "The session terminal needs the desktop app. Open RunHaven on your Mac to start this assistant and type here.",
    connectedText: "Connected. Type your task for the assistant here.",
    endedText: "This session ended.",
  }
}

export function TerminalDisclosure({
  retainForHistory = false,
}: {
  retainForHistory?: boolean
}) {
  if (retainForHistory) {
    return (
      <Alert variant="warning">
        <TriangleAlert />
        <AlertTitle>Live terminal</AlertTitle>
        <AlertDescription>
          RunHaven keeps a filtered copy in memory until you quit, then deletes
          it. It never saves the copy to disk. The assistant can show private
          project information, so review the text before you share it.
        </AlertDescription>
      </Alert>
    )
  }
  return (
    <Alert variant="warning">
      <TriangleAlert />
      <AlertTitle>Live terminal, shown but not saved</AlertTitle>
      <AlertDescription>
        RunHaven shows this live session without reading, filtering, saving, or
        logging it. Anything the assistant displays, including passwords or
        codes, appears in this window.
      </AlertDescription>
    </Alert>
  )
}
