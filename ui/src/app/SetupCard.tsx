import { useState } from "react"
import {
  performSetupRecovery,
  type DashboardStatus,
  type SetupActionId,
} from "@/commands/runhaven"
import { ArrowUpRight, CheckCircle2, CircleAlert } from "lucide-react"
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
import { errorMessage } from "./types"

export function SetupCard({
  loading,
  dashboard,
  onCheckAgain,
  onOpenDiagnostics,
}: {
  loading: boolean
  dashboard: DashboardStatus | null
  onCheckAgain?: () => void
  onOpenDiagnostics?: () => void
}) {
  const [recoveryBusy, setRecoveryBusy] = useState(false)
  const [recoveryError, setRecoveryError] = useState("")
  const [recoveryMessage, setRecoveryMessage] = useState("")
  const setupKnown = dashboard !== null
  const setupReady = setupKnown && dashboard.setup.ok
  const failedChecks = (dashboard?.setup.checks ?? []).filter(
    (check) => !check.ok,
  )
  const firstFailedCheck = failedChecks[0]
  const remainingFailures = Math.max(failedChecks.length - 1, 0)

  async function recover(actionId: SetupActionId) {
    if (recoveryBusy) {
      return
    }
    setRecoveryBusy(true)
    setRecoveryError("")
    setRecoveryMessage("")
    try {
      const result = await performSetupRecovery(actionId)
      setRecoveryMessage(result.detail)
      onCheckAgain?.()
    } catch (cause) {
      setRecoveryError(errorMessage(cause))
    } finally {
      setRecoveryBusy(false)
    }
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {loading
            ? "Checking your Mac"
            : !setupKnown
              ? "Setup unavailable"
              : setupReady
                ? "Ready to run"
                : "Setup needs attention"}
        </CardTitle>
        <CardDescription>
          {loading
            ? "Checking setup..."
            : !setupKnown
              ? "RunHaven could not read setup status."
              : setupReady
                ? "Everything RunHaven needs is ready."
                : "Fix this one issue first, then RunHaven can start an agent."}
        </CardDescription>
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
          <div className="rounded-lg border border-l-4 border-l-destructive p-4">
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
            Open System details for the rest.
          </p>
        ) : null}
        {recoveryMessage ? (
          <Alert role="status">
            <CheckCircle2 />
            <AlertTitle>Setup updated</AlertTitle>
            <AlertDescription>{recoveryMessage}</AlertDescription>
          </Alert>
        ) : null}
        {recoveryError ? (
          <Alert variant="destructive" role="alert">
            <CircleAlert />
            <AlertTitle>Setup could not be fixed</AlertTitle>
            <AlertDescription>{recoveryError}</AlertDescription>
          </Alert>
        ) : null}
        {onCheckAgain || onOpenDiagnostics || firstFailedCheck?.action ? (
          <div className="flex flex-wrap gap-2">
            {firstFailedCheck?.action ? (
              <Button
                type="button"
                disabled={loading || recoveryBusy}
                onClick={() => void recover(firstFailedCheck.action!.id)}
              >
                {recoveryBusy ? "Starting..." : firstFailedCheck.action.label}
              </Button>
            ) : null}
            {onOpenDiagnostics ? (
              <Button
                type="button"
                variant={firstFailedCheck?.action ? "outline" : "default"}
                onClick={onOpenDiagnostics}
              >
                <ArrowUpRight />
                System details
              </Button>
            ) : null}
            {onCheckAgain ? (
              <Button
                type="button"
                variant="outline"
                disabled={loading || recoveryBusy}
                onClick={onCheckAgain}
              >
                {loading ? "Checking..." : "Check again"}
              </Button>
            ) : null}
          </div>
        ) : null}
      </CardContent>
    </Card>
  )
}
