import { useEffect, useState } from "react"
import { CheckCircle2, CircleAlert, Hammer } from "lucide-react"

import {
  getImageStatus,
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
          onClick={onRefresh}
        >
          {refreshing ? "Checking..." : "Check again"}
        </Button>
      </div>
      {agents.map((agent) => (
        <AgentRow
          key={agent.name}
          agent={agent}
          status={
            authStatus?.profiles.find(
              (profile) => profile.name === agent.name,
            )?.status ?? ""
          }
          onSignedIn={onSignedIn}
        />
      ))}
      <ImageRebuildCard agents={agents.map((agent) => agent.name)} />
    </div>
  )
}

function AgentRow({
  agent,
  status,
  onSignedIn,
}: {
  agent: AgentProfile
  status: string
  onSignedIn: () => void
}) {
  const image = useAgentImageStatus(agent.name)
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

// Read-only image readiness for one agent row, loaded once when the row
// mounts. It reuses the launch flow's ImageReadiness rendering so the Agents
// view and the New Run sheet describe image state with the same words.
function useAgentImageStatus(agentName: string) {
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
  }, [agentName])

  return { loading, status, error }
}

// Rebuilds an agent image through the run-control command, which requires a
// native confirmation in Rust before any change. Moved from the old
// Maintenance accordion; the flow and labels are unchanged.
function ImageRebuildCard({ agents }: { agents: string[] }) {
  const [agent, setAgent] = useState(agents[0] ?? "")
  const { confirm, setConfirm, busy, error, status, rebuild } = useImageRebuild()

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
