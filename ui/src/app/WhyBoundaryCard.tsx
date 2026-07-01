import { CircleAlert, ShieldQuestion } from "lucide-react"
import { useCallback, useState, type ReactNode } from "react"

import {
  getWhyHost,
  getWhyNetwork,
  getWhyState,
  getWhyWorkspace,
  type HostExplanationResponse,
  type NetworkExplanationResponse,
  type StateExplanationResponse,
  type WorkspaceExplanationResponse,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"

import { errorMessage } from "./types"
import {
  decisionBadgeVariant,
  decisionLabel,
  hostLists,
  hostRows,
  networkRows,
  stateRows,
  workspaceRows,
  type ExplainList,
  type ExplainRow,
} from "./whyExplain"

// Read-only "Why is this safe?" panel. Each section loads on demand from a
// secret-free backend command and shows plain-language boundary details. No
// input is sent that could change a run, and no secret or host path is shown.

type Loader<T> = () => Promise<T>

type ExplainState<T> = {
  data: T | null
  error: string
  loading: boolean
  run: () => void
}

function useExplain<T>(loader: Loader<T>): ExplainState<T> {
  const [data, setData] = useState<T | null>(null)
  const [error, setError] = useState("")
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    setError("")
    void loader()
      .then((value) => setData(value))
      .catch((cause) => {
        setData(null)
        setError(errorMessage(cause))
      })
      .finally(() => setLoading(false))
  }, [loader])

  return { data, error, loading, run }
}

function RowList({ rows }: { rows: ExplainRow[] }) {
  return (
    <div className="flex flex-col gap-1.5 text-sm">
      {rows.map((row) => (
        <div key={row.label}>
          <span className="font-medium">{row.label}: </span>
          <span className="text-muted-foreground">{row.value}</span>
        </div>
      ))}
    </div>
  )
}

function BulletLists({ lists }: { lists: ExplainList[] }) {
  return (
    <div className="flex flex-col gap-2 text-sm">
      {lists.map((list) => (
        <div key={list.label}>
          <div className="font-medium">{list.label}</div>
          <ul className="ml-4 list-disc text-muted-foreground">
            {list.items.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  )
}

function DecisionBadge({ decision }: { decision: string }) {
  return (
    <Badge variant={decisionBadgeVariant(decision)}>{decisionLabel(decision)}</Badge>
  )
}

function ExplainSection({
  title,
  description,
  buttonLabel,
  state,
  children,
}: {
  title: string
  description: string
  buttonLabel: string
  state: ExplainState<unknown>
  children: ReactNode
}) {
  return (
    <section className="rounded-lg border p-3">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h4 className="text-sm font-medium">{title}</h4>
          <p className="text-sm text-muted-foreground">{description}</p>
        </div>
        <Button type="button" variant="outline" size="sm" onClick={state.run}>
          {state.loading ? "Loading..." : state.data ? "Refresh" : buttonLabel}
        </Button>
      </div>
      {state.error ? (
        <Alert variant="destructive" className="mt-3">
          <CircleAlert />
          <AlertTitle>Details unavailable</AlertTitle>
          <AlertDescription>{state.error}</AlertDescription>
        </Alert>
      ) : null}
      {state.data ? <div className="mt-3">{children}</div> : null}
    </section>
  )
}

type WhyBoundaryCardProps = {
  agentName?: string
  networkMode: string
  workspacePath: string
  workspaceScope: string
  allowSensitiveWorkspace: boolean
}

export function WhyBoundaryCard({
  agentName,
  networkMode,
  workspacePath,
  workspaceScope,
  allowSensitiveWorkspace,
}: WhyBoundaryCardProps) {
  const network = useExplain<NetworkExplanationResponse>(
    useCallback(() => getWhyNetwork(networkMode), [networkMode]),
  )
  const workspace = useExplain<WorkspaceExplanationResponse>(
    useCallback(
      () => getWhyWorkspace(workspacePath, workspaceScope, allowSensitiveWorkspace),
      [workspacePath, workspaceScope, allowSensitiveWorkspace],
    ),
  )
  const state = useExplain<StateExplanationResponse>(
    useCallback(() => getWhyState(agentName ?? ""), [agentName]),
  )
  const [host, setHost] = useState("")
  const hostCheck = useExplain<HostExplanationResponse>(
    useCallback(() => getWhyHost(host, 443, agentName ?? null), [host, agentName]),
  )

  return (
    <section className="flex flex-col gap-3 rounded-lg border p-3">
      <div className="flex items-center gap-2">
        <ShieldQuestion className="size-4" />
        <h3 className="text-sm font-medium">Why is this safe?</h3>
      </div>
      <p className="text-sm text-muted-foreground">
        Read-only explanations of the network, workspace, host, and state
        boundaries. Nothing here changes your run, and no secrets are shown.
      </p>

      <ExplainSection
        title="Network boundary"
        description={`How the "${networkMode}" network setting limits what the agent can reach.`}
        buttonLabel="Explain network"
        state={network}
      >
        {network.data ? <RowList rows={networkRows(network.data)} /> : null}
      </ExplainSection>

      <ExplainSection
        title="Workspace boundary"
        description="Whether this folder can be mounted, and what stays private."
        buttonLabel="Explain workspace"
        state={workspace}
      >
        {workspace.data ? (
          <div className="flex flex-col gap-2">
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-muted-foreground">Verdict</span>
              <DecisionBadge decision={workspace.data.decision} />
              <span className="text-sm">{workspace.data.verdict}</span>
            </div>
            <RowList rows={workspaceRows(workspace.data)} />
          </div>
        ) : null}
      </ExplainSection>

      {agentName ? (
        <ExplainSection
          title="State boundary"
          description="How this agent's saved state is kept separate from your files."
          buttonLabel="Explain state"
          state={state}
        >
          {state.data ? <RowList rows={stateRows(state.data)} /> : null}
        </ExplainSection>
      ) : null}

      <section className="rounded-lg border p-3">
        <h4 className="text-sm font-medium">Host check</h4>
        <p className="text-sm text-muted-foreground">
          Check whether a provider host would be allowed before you add it.
        </p>
        <div className="mt-3 flex gap-2">
          <Input
            value={host}
            onChange={(event) => setHost(event.currentTarget.value)}
            placeholder="api.example.com"
            aria-label="Provider host to check"
          />
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => {
              if (host.trim()) {
                hostCheck.run()
              }
            }}
          >
            {hostCheck.loading ? "Checking..." : "Check host"}
          </Button>
        </div>
        {hostCheck.error ? (
          <Alert variant="destructive" className="mt-3">
            <CircleAlert />
            <AlertTitle>Host check unavailable</AlertTitle>
            <AlertDescription>{hostCheck.error}</AlertDescription>
          </Alert>
        ) : null}
        {hostCheck.data ? (
          <div className="mt-3 flex flex-col gap-2">
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-sm">
                {hostCheck.data.host}:{hostCheck.data.port}
              </span>
              <DecisionBadge decision={hostCheck.data.decision} />
            </div>
            <RowList rows={hostRows(hostCheck.data)} />
            <BulletLists lists={hostLists(hostCheck.data)} />
          </div>
        ) : null}
      </section>
    </section>
  )
}
