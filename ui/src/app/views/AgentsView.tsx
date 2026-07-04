import { useCallback, useEffect, useState } from "react"
import { CheckCircle2, CircleAlert, Hammer } from "lucide-react"

import {
  getImageStatus,
  isTerminalState,
  type AgentProfile,
  type AuthStatusResponse,
  type ImageStatusResponse,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"

import { AgentAuthDetailsCard } from "../AgentAuthDetailsCard"
import { AgentSignIn } from "../AgentSignIn"
import { authStatusLabel } from "../labels"
import { ImageReadiness } from "../LaunchAccess"
import { agentLabel } from "../sessionIdentity"
import { errorMessage } from "../types"
import { useImageRebuild } from "../useMaintenance"

// The Agents destination: one row per agent with its sign-in state, the
// sign-in flow, read-only auth details, and the image rebuild job (moved here
// from the old Maintenance accordion). Sign-in and rebuild internals are the
// same components and hooks as before; this view only composes them.
export function AgentsView({
  agents,
  authStatus,
  refreshing,
  onSignedIn,
  onRefresh,
}: {
  agents: AgentProfile[]
  authStatus: AuthStatusResponse | null
  refreshing: boolean
  onSignedIn: () => void
  onRefresh: () => void
}) {
  // Per-row image readiness is loaded inside each AgentRow, so "Check again"
  // and a completed rebuild have to poke those rows to re-read. Bumping this
  // key re-runs every row's image fetch; onRefresh still reloads the shared
  // dashboard and auth status.
  const [imageReloadKey, setImageReloadKey] = useState(0)
  const reloadImages = useCallback(
    () => setImageReloadKey((key) => key + 1),
    [],
  )

  if (agents.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle role="heading" aria-level={2}>
            Agents
          </CardTitle>
          <CardDescription>
            No agents found yet. Refresh status to try again.
          </CardDescription>
        </CardHeader>
      </Card>
    )
  }

  return (
    <div className="flex flex-col gap-4">
      {/* Toolbar approximation until the native menu bar lands: Check again
          refreshes agent readiness. No prominent action lives on Agents. */}
      <div className="flex justify-end">
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={refreshing}
          onClick={() => {
            reloadImages()
            onRefresh()
          }}
        >
          {refreshing ? "Checking..." : "Check again"}
        </Button>
      </div>
      {agents.map((agent) => (
        <AgentRow
          key={agent.name}
          agent={agent}
          reloadKey={imageReloadKey}
          status={
            authStatus?.profiles.find(
              (profile) => profile.name === agent.name,
            )?.status ?? ""
          }
          onSignedIn={onSignedIn}
        />
      ))}
      <ImageRebuildCard
        agents={agents.map((agent) => agent.name)}
        onRebuilt={reloadImages}
      />
    </div>
  )
}

function AgentRow({
  agent,
  status,
  reloadKey,
  onSignedIn,
}: {
  agent: AgentProfile
  status: string
  reloadKey: number
  onSignedIn: () => void
}) {
  const image = useAgentImageStatus(agent.name, reloadKey)
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {agentLabel(agent.name)}
        </CardTitle>
        <CardDescription>{agent.description}</CardDescription>
        <CardAction>
          <Badge variant="outline">{authStatusLabel(status)}</Badge>
        </CardAction>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <ImageReadiness
          imageLoading={image.loading}
          imageStatus={image.status}
          imageError={image.error}
        />
        <AgentSignIn agent={agent} onSignedIn={onSignedIn} />
        <AgentAuthDetailsCard agentName={agent.name} />
      </CardContent>
    </Card>
  )
}

// Read-only image readiness for one agent row. It re-reads when `reloadKey`
// changes, so "Check again" and a completed rebuild refresh a stale row. It
// reuses the launch flow's ImageReadiness rendering so the Agents view and the
// New Run sheet describe image state with the same words. Exported so a focused
// test can drive the reload without the whole view's other command calls.
export function useAgentImageStatus(agentName: string, reloadKey = 0) {
  const [loading, setLoading] = useState(true)
  const [status, setStatus] = useState<ImageStatusResponse | null>(null)
  const [error, setError] = useState("")

  useEffect(() => {
    let cancelled = false
    setLoading(true)
    setStatus(null)
    setError("")
    getImageStatus(agentName)
      .then((next) => {
        if (!cancelled) {
          setStatus(next)
        }
      })
      .catch((cause: unknown) => {
        if (!cancelled) {
          setError(errorMessage(cause))
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false)
        }
      })
    return () => {
      cancelled = true
    }
  }, [agentName, reloadKey])

  return { loading, status, error }
}

// Rebuilds an agent image through the run-control command, which requires a
// native confirmation in Rust before any change. Moved from the old
// Maintenance accordion; the flow and labels are unchanged.
function ImageRebuildCard({
  agents,
  onRebuilt,
}: {
  agents: string[]
  onRebuilt: () => void
}) {
  const [agent, setAgent] = useState(agents[0] ?? "")
  const { confirm, setConfirm, busy, error, status, rebuild } = useImageRebuild()

  // Once a rebuild reaches a terminal state the on-disk image changed, so tell
  // the parent to re-read the per-agent readiness rows. Keyed on the terminal
  // state string so it fires once per finished build, not on every poll tick.
  const rebuildState = status?.state ?? ""
  useEffect(() => {
    if (isTerminalState(rebuildState)) {
      onRebuilt()
    }
  }, [onRebuilt, rebuildState])

  return (
    <Card aria-live="polite">
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          <span className="flex items-center gap-2">
            <Hammer className="size-4" aria-hidden />
            Rebuild agent image
          </span>
        </CardTitle>
        <CardDescription>
          Use this if an agent image is missing or out of date. RunHaven
          rebuilds it from the bundled template. Running sessions keep working
          until you launch a new one.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <label className="flex flex-col gap-1 text-sm">
          <span>Agent</span>
          <select
            className="rounded-md border bg-background p-2"
            value={agent}
            onChange={(event) => setAgent(event.target.value)}
            aria-label="Agent to rebuild"
          >
            {agents.map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </label>
        <label className="flex items-start gap-2 text-sm">
          <Checkbox
            checked={confirm}
            onCheckedChange={(checked) => setConfirm(checked === true)}
            aria-label="Confirm agent image rebuild"
          />
          <span>Yes, rebuild this agent image.</span>
        </label>
        <div>
          <Button
            type="button"
            variant="outline"
            disabled={busy || !confirm || !agent}
            onClick={() => void rebuild(agent)}
          >
            {busy ? "Starting..." : "Rebuild image"}
          </Button>
        </div>
        {error ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Rebuild failed to start</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
        {status ? (
          <Alert>
            <CheckCircle2 />
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
      </CardContent>
    </Card>
  )
}
