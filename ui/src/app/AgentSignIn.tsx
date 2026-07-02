import { CheckCircle2, CircleAlert, LogIn } from "lucide-react"

import type { AgentProfile } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { ClaudeSignIn } from "./ClaudeSignIn"
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
// Claude uses its own control (`ClaudeSignIn`): path C (sign in on your Mac, the
// secure default) saves the token to the Mac's Keychain, with path A (sign in
// inside the sandbox) as an explicit opt-in. Agents with no sign-in at all (like
// shell) show nothing.
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
      return <ClaudeSignIn onSignedIn={onSignedIn} />
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
