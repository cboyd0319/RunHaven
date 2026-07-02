import { useRef, useState } from "react"
import { CheckCircle2, CircleAlert, Copy, LogIn } from "lucide-react"

import type { AgentProfile } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { agentLabel } from "./sessionIdentity"
import { TerminalPanel } from "./TerminalPanel"
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
// back), so for claude this opens an in-app sign-in terminal (path A) where the
// pasted code round-trips, with running it in your own terminal kept as a
// fallback. Agents with no sign-in at all (like shell) show nothing.
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
      return <ClaudeSignIn />
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

// Claude in-app sign-in (terminal path A): an in-app terminal running the
// in-sandbox `claude auth login`, where the printed link and the pasted-back
// code round-trip in the app. Running it in your own terminal stays as a
// fallback. The terminal only invokes under the desktop runtime; the browser
// preview shows the panel's own placeholder and never signs in.
function ClaudeSignIn() {
  // A stable per-mount login id: the registry key and run label. The target is
  // fixed to Claude in Rust, so this id never steers what runs.
  const loginId = useRef(
    (globalThis.crypto?.randomUUID?.() ?? `claude-login-${Date.now()}`).replaceAll(
      "-",
      "",
    ),
  ).current
  return (
    <div className="flex flex-col gap-3">
      <TerminalPanel login={{ sessionId: loginId }} />
      <Alert>
        <LogIn />
        <AlertTitle>Prefer your own terminal?</AlertTitle>
        <AlertDescription className="flex flex-col items-start gap-2">
          <span>
            You can also sign in from your own terminal. Run{" "}
            <code className="font-mono">{CLAUDE_LOGIN_COMMAND}</code>, then come
            back and refresh.
          </span>
          <CopyCommandButton command={CLAUDE_LOGIN_COMMAND} />
        </AlertDescription>
      </Alert>
    </div>
  )
}

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
