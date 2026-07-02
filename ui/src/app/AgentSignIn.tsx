import { useState } from "react"
import { CheckCircle2, CircleAlert, Copy, LogIn } from "lucide-react"

import type { AgentProfile } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { agentLabel } from "./sessionIdentity"
import { useAgentSignIn } from "./useMaintenance"

// Per-agent sign-in affordance shown next to the read-only auth summary.
//
// For device-code agents (codex, copilot, antigravity) it starts sign-in in the
// app: the backend shows a native confirmation, then RunHaven opens the agent's
// device login and this card shows the link and short-lived code to finish in a
// browser. When it succeeds, `onSignedIn` refreshes auth status so the agent
// shows as signed in. RunHaven never sees or stores the token.
//
// Claude signs in from a terminal (its in-sandbox sign-in needs a code pasted
// back into the terminal, which the app does not host), so for claude this shows
// a clear terminal instruction instead of a button. Agents with no sign-in at
// all (like shell) show nothing.
export function AgentSignIn({
  agent,
  onSignedIn,
}: {
  agent: AgentProfile | undefined
  onSignedIn?: () => void
}) {
  const { busy, error, status, signIn } = useAgentSignIn(onSignedIn)
  if (!agent) {
    return null
  }

  if (!agent.supportsInAppSignIn) {
    if (agent.name === "claude") {
      return (
        <Alert>
          <LogIn />
          <AlertTitle>Sign in to Claude from a terminal</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>
              Claude sign-in needs a code pasted in a terminal, so it does not
              run in the app. Open your terminal and run{" "}
              <code className="font-mono">{CLAUDE_LOGIN_COMMAND}</code>, then
              come back and refresh.
            </span>
            <CopyCommandButton command={CLAUDE_LOGIN_COMMAND} />
          </AlertDescription>
        </Alert>
      )
    }
    return null
  }

  return (
    <section
      className="flex flex-col gap-3 rounded-lg border p-3"
      aria-live="polite"
    >
      <div className="flex items-center gap-2 text-sm font-medium">
        <LogIn className="size-4" aria-hidden />
        Sign in to {agentLabel(agent.name)}
      </div>
      <p className="text-sm text-muted-foreground">
        RunHaven opens a sign-in for {agentLabel(agent.name)}. You confirm first, then finish
        in your browser. RunHaven never sees or stores your password or token.
      </p>
      <div>
        <Button
          type="button"
          disabled={busy}
          onClick={() => void signIn(agent.name)}
        >
          <LogIn />
          {busy ? "Starting..." : "Sign in"}
        </Button>
      </div>
      {error ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Sign-in did not start</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      ) : null}
      {status ? (
        <Alert variant={status.state === "failed" ? "destructive" : "default"}>
          {status.state === "succeeded" ? <CheckCircle2 /> : <LogIn />}
          <AlertTitle>
            {status.label} ({status.agent})
          </AlertTitle>
          <AlertDescription className="flex flex-col gap-2">
            <span>{status.detail}</span>
            {status.outputTail ? (
              <pre className="max-h-48 overflow-auto rounded bg-muted p-2 text-xs whitespace-pre-wrap">
                {status.outputTail}
              </pre>
            ) : null}
          </AlertDescription>
        </Alert>
      ) : null}
    </section>
  )
}

const CLAUDE_LOGIN_COMMAND = "runhaven login claude"

// Copies the terminal sign-in command with the browser clipboard API. The
// button label doubles as the copied confirmation; if the WebView refuses
// clipboard access, a short note tells the user to copy the text by hand.
function CopyCommandButton({ command }: { command: string }) {
  const [copyState, setCopyState] = useState<"idle" | "copied" | "failed">(
    "idle",
  )

  async function copy() {
    try {
      await navigator.clipboard.writeText(command)
      setCopyState("copied")
    } catch {
      setCopyState("failed")
    }
  }

  return (
    <span className="flex flex-wrap items-center gap-2" aria-live="polite">
      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={() => void copy()}
      >
        <Copy aria-hidden />
        {copyState === "copied" ? "Copied" : "Copy command"}
      </Button>
      {copyState === "failed" ? (
        <span className="text-sm">
          Copy did not work here. Select the command text and copy it yourself.
        </span>
      ) : null}
    </span>
  )
}
