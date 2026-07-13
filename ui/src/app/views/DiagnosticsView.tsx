import { useEffect, useRef, useState } from "react"

import {
  performSetupRecovery,
  type DashboardStatus,
  type SetupActionId,
  type SetupRecoveryResponse,
} from "@/commands/runhaven"
import { CircleAlert, CheckCircle2 } from "lucide-react"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { ActivityIndicator } from "@/components/ui/activity-indicator"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { errorMessage } from "../types"
import { useNetworkCleanup } from "../useMaintenance"

// System details combines setup repair and stale-network cleanup so each
// problem has one next action. Rust owns every mutation and native confirmation;
// the view receives only secret-free status and count data.
export function DiagnosticsView({
  dashboard,
  refreshing,
  onRefresh,
}: {
  dashboard: DashboardStatus | null
  refreshing: boolean
  onRefresh: () => void
}) {
  const cleanup = useNetworkCleanup()
  const setupReady = dashboard?.setup.ok === true
  const previewLoaded = useRef(false)
  useEffect(() => {
    if (!setupReady) {
      previewLoaded.current = false
    } else if (!previewLoaded.current) {
      previewLoaded.current = true
      void cleanup.loadPreview()
    }
  }, [setupReady])

  const busy = refreshing || cleanup.loading

  return (
    <div className="flex flex-col gap-4">
      <div className="flex justify-end">
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={busy}
          onClick={() => {
            onRefresh()
            if (setupReady) {
              void cleanup.loadPreview()
            }
          }}
        >
          {busy ? <ActivityIndicator /> : null}
          {busy ? "Checking…" : "Check again"}
        </Button>
      </div>
      <SystemHealth dashboard={dashboard} />
      <SetupChecksCard
        dashboard={dashboard}
        refreshing={refreshing}
        onRefresh={onRefresh}
      />
      <details className="rounded-lg border bg-card p-4">
        <summary className="cursor-pointer text-sm font-medium">
          Advanced system tools
        </summary>
        <div className="mt-4 flex flex-col gap-4">
          <ContainerServiceCard setupReady={setupReady} onRefresh={onRefresh} />
          <NetworkCleanupCard cleanup={cleanup} setupReady={setupReady} />
          {dashboard ? (
            <section className="flex flex-col gap-3 text-sm text-muted-foreground">
              <h3 className="font-medium text-foreground">Readiness checks</h3>
              <ul aria-label="System readiness checks" className="grid gap-2">
                {dashboard.setup.checks.map((check) => (
                  <li key={check.name} className="rounded-md border p-3">
                    <div className="flex items-center justify-between gap-3">
                      <span className="font-medium text-foreground">{check.name}</span>
                      <span>{check.ok ? "Ready" : "Needs attention"}</span>
                    </div>
                    {check.detail.trim().toLowerCase() !== "ready." ? (
                      <p className="mt-1">{check.detail}</p>
                    ) : null}
                  </li>
                ))}
              </ul>
            </section>
          ) : null}
        </div>
      </details>
    </div>
  )
}

function ContainerServiceCard({
  setupReady,
  onRefresh,
}: {
  setupReady: boolean
  onRefresh: () => void
}) {
  const [busy, setBusy] = useState(false)
  const [result, setResult] = useState<SetupRecoveryResponse | null>(null)
  const [error, setError] = useState("")

  async function restart() {
    if (busy) {
      return
    }
    setBusy(true)
    setResult(null)
    setError("")
    try {
      setResult(await performSetupRecovery("restart-container-system"))
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      onRefresh()
      setBusy(false)
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>Fix session internet</CardTitle>
        <CardDescription>
          Restart Apple container when new sessions can’t connect to the
          internet. This ends current sessions.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col items-start gap-3">
        <Button
          type="button"
          variant="outline"
          disabled={!setupReady || busy}
          onClick={() => void restart()}
        >
          {busy ? <ActivityIndicator /> : null}
          {busy ? "Restarting…" : "Restart container service…"}
        </Button>
        {result ? (
          <Alert role="status">
            {result.status === "succeeded" ? <CheckCircle2 /> : null}
            <AlertTitle>
              {result.status === "succeeded"
                ? "Internet service refreshed"
                : result.label}
            </AlertTitle>
            <AlertDescription>{result.detail}</AlertDescription>
          </Alert>
        ) : null}
        {error ? (
          <Alert variant="destructive" role="alert">
            <CircleAlert />
            <AlertTitle>Could not restart the service</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
      </CardContent>
    </Card>
  )
}

// Passing checks do not ask the user to do anything. Show only setup problems,
// with one plain next action for each problem.
function SetupChecksCard({
  dashboard,
  refreshing,
  onRefresh,
}: {
  dashboard: DashboardStatus | null
  refreshing: boolean
  onRefresh: () => void
}) {
  const failedChecks =
    dashboard?.setup.checks.filter((check) => !check.ok) ?? []
  const [busyAction, setBusyAction] = useState<SetupActionId | null>(null)
  const [message, setMessage] = useState("")
  const [error, setError] = useState("")

  async function recover(actionId: SetupActionId) {
    if (busyAction) {
      return
    }
    setBusyAction(actionId)
    setMessage("")
    setError("")
    try {
      const result = await performSetupRecovery(actionId)
      setMessage(result.detail)
      onRefresh()
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setBusyAction(null)
    }
  }

  if (dashboard && failedChecks.length === 0) {
    return null
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {dashboard ? "Setup problems" : "Setup status unavailable"}
        </CardTitle>
        <CardDescription>
          {dashboard
            ? "Fix each problem below, then check again."
            : "Choose Check again to check this Mac."}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {!dashboard ? (
          <p
            role={refreshing ? "status" : undefined}
            className="flex items-center gap-2 text-sm text-muted-foreground"
          >
            {refreshing ? <ActivityIndicator /> : null}
            {refreshing
              ? "Checking your Mac's setup…"
              : "RunHaven could not load setup status."}
          </p>
        ) : (
          <ul className="flex flex-col gap-3">
            {failedChecks.map((check) => (
              <li key={check.name} className="rounded-lg border p-3">
                <div className="font-medium">{check.name}</div>
                <p className="mt-2 text-sm text-muted-foreground">
                  {check.detail}
                </p>
                <div className="mt-3 flex flex-col items-start gap-2 rounded-md bg-muted p-3 text-sm">
                  <div className="font-medium">Next step</div>
                  <p className="mt-1 text-muted-foreground">{check.remedy}</p>
                  {check.action ? (
                    <Button
                      type="button"
                      size="sm"
                      disabled={busyAction !== null}
                      onClick={() => void recover(check.action!.id)}
                    >
                      {busyAction === check.action.id ? <ActivityIndicator /> : null}
                      {busyAction === check.action.id
                        ? "Working…"
                        : `${check.action.label}…`}
                    </Button>
                  ) : null}
                </div>
              </li>
            ))}
          </ul>
        )}
        {message ? (
          <Alert role="status">
            <CheckCircle2 />
            <AlertTitle>Setup updated</AlertTitle>
            <AlertDescription>{message}</AlertDescription>
          </Alert>
        ) : null}
        {error ? (
          <Alert variant="destructive" role="alert">
            <CircleAlert />
            <AlertTitle>Could not fix setup</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
      </CardContent>
    </Card>
  )
}

function SystemHealth({ dashboard }: { dashboard: DashboardStatus | null }) {
  if (!dashboard) {
    return null
  }
  const blockers = dashboard.setup.blockerCount
  return (
    <Card>
      <CardContent>
        <p className="text-sm">
          {dashboard.setup.ok
            ? "Your Mac is ready for RunHaven."
            : `${blockers} setup ${blockers === 1 ? "problem needs" : "problems need"} your attention.`}
        </p>
      </CardContent>
    </Card>
  )
}

type NetworkCleanup = ReturnType<typeof useNetworkCleanup>

function NetworkCleanupCard({
  cleanup,
  setupReady,
}: {
  cleanup: NetworkCleanup
  setupReady: boolean
}) {
  const count = cleanup.count
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>Old session networks</CardTitle>
        <CardDescription>
          RunHaven can remove networks left behind by ended sessions.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col items-start gap-3">
        <p
          role={cleanup.loading ? "status" : undefined}
          className="flex items-center gap-2 text-sm text-muted-foreground"
        >
          {cleanup.loading ? <ActivityIndicator /> : null}
          {!setupReady
            ? "Finish setup before checking networks."
            : cleanup.loading
              ? "Checking for old session networks…"
              : count === null
                ? "The network count is unavailable."
                : count === 0
                  ? "No old session networks found."
                  : `${count} old session ${count === 1 ? "network" : "networks"} found.`}
        </p>
        <Button
          type="button"
          variant="outline"
          className="!text-foreground"
          disabled={!setupReady || count === null || count === 0 || cleanup.busy}
          onClick={() => void cleanup.prune()}
        >
          {cleanup.busy ? <ActivityIndicator /> : null}
          {cleanup.busy ? "Removing old networks…" : "Remove old networks…"}
        </Button>
        {cleanup.result && cleanup.result.failed > 0 ? (
          <Alert variant="destructive" role="alert">
            <CircleAlert />
            <AlertTitle>
              Could not remove {cleanup.result.failed} {cleanup.result.failed === 1 ? "network" : "networks"}
            </AlertTitle>
            <AlertDescription>
              RunHaven left anything it could not safely remove. Try again after
              current sessions finish.
            </AlertDescription>
          </Alert>
        ) : cleanup.result ? (
          <Alert role="status">
            <CheckCircle2 />
            <AlertTitle>Cleanup complete</AlertTitle>
            <AlertDescription>
              Removed {cleanup.result.removed} RunHaven networks.
            </AlertDescription>
          </Alert>
        ) : null}
        {cleanup.loadError || cleanup.actionError ? (
          <Alert variant="destructive" role="alert">
            <CircleAlert />
            <AlertTitle>Could not clean up networks</AlertTitle>
            <AlertDescription>
              {cleanup.actionError || cleanup.loadError}
            </AlertDescription>
          </Alert>
        ) : null}
      </CardContent>
    </Card>
  )
}
