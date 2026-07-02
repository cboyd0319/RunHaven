import type { DashboardStatus } from "@/commands/runhaven"
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

export function SetupCard({
  loading,
  dashboard,
  onCheckAgain,
}: {
  loading: boolean
  dashboard: DashboardStatus | null
  onCheckAgain?: () => void
}) {
  const setupKnown = dashboard !== null
  const setupReady = setupKnown && dashboard.setup.ok
  const failedChecks = (dashboard?.setup.checks ?? []).filter(
    (check) => !check.ok,
  )
  const firstFailedCheck = failedChecks[0]
  const remainingFailures = Math.max(failedChecks.length - 1, 0)
  const statusLabel = loading
    ? "Checking"
    : !setupKnown
      ? "Unknown"
      : setupReady
        ? "Ready"
        : "Blocked"

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {setupReady ? "Setup ready" : "Finish setup"}
        </CardTitle>
        <CardDescription>
          {loading
            ? "Checking setup..."
            : !setupKnown
              ? "RunHaven could not read setup status."
              : setupReady
              ? "Ready to launch."
              : "Fix this before starting an agent."}
        </CardDescription>
        <CardAction>
          <Badge
            variant={
              loading || !setupKnown
                ? "outline"
                : setupReady
                  ? "success"
                  : "destructive"
            }
          >
            {statusLabel}
          </Badge>
        </CardAction>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {loading ? (
          <p className="text-sm text-muted-foreground">Loading setup...</p>
        ) : null}
        {!loading && !setupKnown ? (
          <p className="text-sm text-muted-foreground">
            Refresh status to try again.
          </p>
        ) : null}
        {setupReady ? (
          <p className="text-sm text-muted-foreground">All checks passed.</p>
        ) : null}
        {firstFailedCheck ? (
          <div className="rounded-lg border p-3">
            <div className="flex items-center justify-between gap-3">
              <div className="font-medium">{firstFailedCheck.name}</div>
              <Badge variant="destructive">Needs attention</Badge>
            </div>
            <p className="mt-2 text-sm text-muted-foreground">
              {firstFailedCheck.detail}
            </p>
            <div className="mt-3 rounded-md bg-muted p-3 text-sm">
              <div className="font-medium">Next step</div>
              <p className="mt-1 text-muted-foreground">
                {firstFailedCheck.remedy}
              </p>
            </div>
          </div>
        ) : null}
        {remainingFailures > 0 ? (
          <p className="text-sm text-muted-foreground">
            {remainingFailures} more setup{" "}
            {remainingFailures === 1 ? "check needs" : "checks need"} attention.
            Open Diagnostics for full details.
          </p>
        ) : null}
        {onCheckAgain ? (
          <div>
            <Button
              type="button"
              variant="outline"
              disabled={loading}
              onClick={onCheckAgain}
            >
              {loading ? "Checking..." : "Check again"}
            </Button>
          </div>
        ) : null}
      </CardContent>
    </Card>
  )
}
