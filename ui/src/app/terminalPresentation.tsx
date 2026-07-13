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

export function attachTerminalConfig(
  session: ManagedSessionSummary,
): TerminalConfig {
  return {
    sessionId: session.id,
    title: "Terminal",
    description: "Type live inside this run's sandbox.",
    openLabel: "Open terminal",
    previewText:
      "Terminal needs the desktop app. Open RunHaven on your Mac to type live with this run.",
    connectedText: "Connected. What you type goes straight to the sandbox.",
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
    title: "Run terminal",
    description:
      "This agent is interactive, so you type your task right here in its terminal. RunHaven starts the run in a sandbox and connects it to this terminal.",
    openLabel: "Start run",
    previewText:
      "The run terminal needs the desktop app. Open RunHaven on your Mac to start this agent and type here.",
    connectedText: "Connected. Type your task for the agent here.",
    endedText: "This run ended.",
  }
}

export function TerminalDisclosure() {
  return (
    <Alert variant="warning">
      <TriangleAlert />
      <AlertTitle>Live terminal, shown but not saved</AlertTitle>
      <AlertDescription>
        This is a live terminal inside the sandbox. RunHaven does not filter,
        read, store, or log it. It just shows what the sandbox prints. So
        anything here, including passwords or codes, is visible on this screen.
      </AlertDescription>
    </Alert>
  )
}
