import { useCallback, useEffect, useState } from "react"
import { CheckCircle2, CircleAlert, Hammer } from "lucide-react"

import {
  getImageStatus,
  isTerminalState,
  type AgentProfile,
  type AgentReadiness,
  type ImageBuildStatusResponse,
  type ImageStatusResponse,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { cn } from "@/lib/utils"

import { AgentAuthDetailsCard } from "../AgentAuthDetailsCard"
import { AgentSignIn } from "../AgentSignIn"
import { ImageReadiness } from "../LaunchAccess"
import { agentLabel } from "../sessionIdentity"
import { errorMessage } from "../types"
import { useImageRebuild } from "../useMaintenance"

// The Agents destination uses a compact list and one detail panel. This keeps a
// six-agent install scannable and puts the first agent needing attention first.
export function AgentsView({
  agents,
  agentReadiness,
  setupReady,
  refreshing,
  onSignedIn,
  onRefresh,
}: {
  agents: AgentProfile[]
  agentReadiness: AgentReadiness[]
  setupReady: boolean
  refreshing: boolean
  onSignedIn: () => void
  onRefresh: () => void
}) {
  // Per-row image readiness is loaded inside each AgentRow, so "Check again"
  // and a completed rebuild have to poke those rows to re-read. Bumping this
  // key re-runs every row's image fetch; onRefresh still reloads the shared
  // dashboard and auth status.
  const [imageReloadKey, setImageReloadKey] = useState(0)
  const [selectedName, setSelectedName] = useState("")
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

  const selectedAgent =
    agents.find((agent) => agent.name === selectedName) ??
    agents.find((agent) =>
      needsSignIn(readinessFor(agentReadiness, agent.name)),
    ) ??
    agents[0]

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
      <div className="grid min-w-0 gap-4 lg:grid-cols-[minmax(14rem,0.7fr)_minmax(0,1.5fr)]">
        <Card>
          <CardHeader>
            <CardTitle role="heading" aria-level={2}>
              Available agents
            </CardTitle>
            <CardDescription>
              Choose an agent to check or sign in.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-1">
            {agents.map((agent) => {
              const readiness = readinessFor(agentReadiness, agent.name)
              const selected = agent.name === selectedAgent.name
              return (
                <button
                  key={agent.name}
                  type="button"
                  aria-current={selected ? "true" : undefined}
                  onClick={() => setSelectedName(agent.name)}
                  className={cn(
                    "flex w-full items-center justify-between gap-3 rounded-lg px-3 py-3 text-left transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                    selected ? "bg-accent text-accent-foreground" : "hover:bg-muted",
                  )}
                >
                  <span className="font-medium">{agentLabel(agent.name)}</span>
                  <Badge
                    variant={needsSignIn(readiness) ? "warning" : "outline"}
                  >
                    {readiness?.label ?? "Checking"}
                  </Badge>
                </button>
              )
            })}
          </CardContent>
        </Card>
        <AgentDetail
          key={selectedAgent.name}
          agent={selectedAgent}
          reloadKey={imageReloadKey}
          readiness={readinessFor(agentReadiness, selectedAgent.name)}
          setupReady={setupReady}
          onSignedIn={onSignedIn}
          onImageChanged={reloadImages}
        />
      </div>
      {setupReady ? (
        <details className="rounded-lg border bg-card p-4">
          <summary className="cursor-pointer text-sm font-medium">
            Repair agent installation
          </summary>
          <div className="mt-4">
            <ImageRebuildCard
              agents={agents.map((agent) => agent.name)}
              onRebuilt={reloadImages}
            />
          </div>
        </details>
      ) : null}
    </div>
  )
}

function AgentDetail({
  agent,
  readiness,
  setupReady,
  reloadKey,
  onSignedIn,
  onImageChanged,
}: {
  agent: AgentProfile
  readiness: AgentReadiness | undefined
  setupReady: boolean
  reloadKey: number
  onSignedIn: () => void
  onImageChanged: () => void
}) {
  const image = useAgentImageStatus(agent.name, reloadKey, setupReady)
  const install = useImageRebuild()
  const installState = install.status?.state ?? ""
  const installing =
    install.busy || (!!install.status && !isTerminalState(installState))
  useEffect(() => {
    if (isTerminalState(installState)) {
      onImageChanged()
    }
  }, [installState, onImageChanged])
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {agentLabel(agent.name)}
        </CardTitle>
        <CardDescription>{agent.description}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <div className="flex items-center justify-between gap-3 rounded-lg border p-3 text-sm">
          <span>Sign-in status</span>
          <Badge variant={needsSignIn(readiness) ? "warning" : "success"}>
            {readiness?.label ?? "Checking"}
          </Badge>
        </div>
        {setupReady ? (
          <ImageReadiness
            imageLoading={image.loading}
            imageStatus={image.status}
            imageError={image.error}
            installing={installing}
            onInstall={
              image.status && !image.status.image.ready
                ? () => void install.rebuild(agent.name)
                : undefined
            }
          />
        ) : (
          <p className="text-sm text-muted-foreground">
            Finish setup before checking or repairing the agent image.
          </p>
        )}
        <ImageBuildFeedback error={install.error} status={install.status} />
        {needsSignIn(readiness) ? (
          <AgentSignIn agent={agent} onSignedIn={onSignedIn} />
        ) : (
          <p className="text-sm text-muted-foreground">
            This agent is ready. You can choose it when you start a new run.
          </p>
        )}
        <details className="rounded-lg border p-3">
          <summary className="cursor-pointer text-sm font-medium">
            Sign-in details
          </summary>
          <div className="mt-3">
            <AgentAuthDetailsCard agentName={agent.name} />
          </div>
        </details>
      </CardContent>
    </Card>
  )
}

// Read-only image readiness for one agent row. It re-reads when `reloadKey`
// changes, so "Check again" and a completed rebuild refresh a stale row. It
// reuses the launch flow's ImageReadiness rendering so the Agents view and the
// New Run sheet describe image state with the same words. Exported so a focused
// test can drive the reload without the whole view's other command calls.
export function useAgentImageStatus(
  agentName: string,
  reloadKey = 0,
  setupReady = true,
) {
  const [loading, setLoading] = useState(setupReady)
  const [status, setStatus] = useState<ImageStatusResponse | null>(null)
  const [error, setError] = useState("")

  useEffect(() => {
    let cancelled = false
    if (!setupReady) {
      setLoading(false)
      setStatus(null)
      setError("")
      return
    }
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
  }, [agentName, reloadKey, setupReady])

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
  const { busy, error, status, rebuild } = useImageRebuild()

  // Once a rebuild reaches a terminal state the on-disk image changed, so tell
  // the parent to re-read the per-agent readiness rows. Keyed on the terminal
  // state string so it fires once per finished build, not on every poll tick.
  const rebuildState = status?.state ?? ""
  const rebuilding = busy || (!!status && !isTerminalState(rebuildState))
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
            Reinstall agent
          </span>
        </CardTitle>
        <CardDescription>
          Replace an installed agent with a fresh copy from RunHaven's bundled
          template. Running sessions keep working until you launch a new one.
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
        <div>
          <Button
            type="button"
            variant="outline"
            disabled={rebuilding || !agent}
            onClick={() => void rebuild(agent)}
          >
            {rebuilding ? "Reinstalling..." : "Reinstall agent"}
          </Button>
        </div>
        <ImageBuildFeedback error={error} status={status} />
      </CardContent>
    </Card>
  )
}

export function ImageBuildFeedback({
  error,
  status,
}: {
  error: string
  status: ImageBuildStatusResponse | null
}) {
  const failed = status?.state === "failed"
  return (
    <>
      {error ? (
        <Alert variant="destructive" role="alert">
          <CircleAlert />
          <AlertTitle>Agent installation could not start</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      ) : null}
      {status ? (
        <Alert variant={failed ? "destructive" : undefined} role={failed ? "alert" : "status"}>
          {failed ? <CircleAlert /> : <CheckCircle2 />}
          <AlertTitle>
            {status.label} ({agentLabel(status.agent)})
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
    </>
  )
}

function readinessFor(
  readiness: AgentReadiness[],
  agentName: string,
): AgentReadiness | undefined {
  return readiness.find((item) => item.name === agentName)
}

function needsSignIn(readiness: AgentReadiness | undefined): boolean {
  return readiness?.category === "needs-sign-in"
}
