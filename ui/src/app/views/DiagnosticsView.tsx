import { useEffect } from "react"

import type { DashboardStatus } from "@/commands/runhaven"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { DiagnosticsCard } from "../DiagnosticsCard"
import { DiagnosticsSystemPanels } from "../DiagnosticsSystemPanels"
import type { DiagnosticsState } from "../types"
import { useDiagnosticsReadModels } from "../useDiagnosticsReadModels"

// The Diagnostics destination: read-only support and trust evidence. It never
// mutates anything, so the trust data loads as soon as the view opens; Check
// again re-reads both the setup checks and the trust checks. Everything shown
// here is already secret-free at the commands boundary.
export function DiagnosticsView({
  dashboard,
  refreshing,
  diagnostics,
  diagnosticsLoading,
  diagnosticsError,
  onLoadDiagnostics,
  onRefresh,
}: {
  dashboard: DashboardStatus | null
  refreshing: boolean
  diagnostics: DiagnosticsState | null
  diagnosticsLoading: boolean
  diagnosticsError: string
  onLoadDiagnostics: () => Promise<void>
  onRefresh: () => void
}) {
  // Read-only system panels (health rollup, versions, image matrix, stale
  // networks) built from existing commands. The hook loads on entry and
  // reloads when the agent set changes; Check again reloads it too.
  const readModels = useDiagnosticsReadModels(
    dashboard?.agents ?? [],
    dashboard?.setup.ok === true,
  )
  // Load on entry, once per visit. Background polls never remount this view,
  // so quiet refresh stays quiet; re-entering the destination reloads.
  useEffect(() => {
    void onLoadDiagnostics()
  }, [])

  const busy = refreshing || diagnosticsLoading || readModels.loading

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
            void onLoadDiagnostics()
            void readModels.reload()
          }}
        >
          {busy ? "Checking..." : "Check again"}
        </Button>
      </div>
      <DiagnosticsSystemPanels dashboard={dashboard} readModels={readModels} />
      <SetupChecksCard dashboard={dashboard} refreshing={refreshing} />
      <DiagnosticsCard
        diagnostics={diagnostics}
        loading={diagnosticsLoading}
        error={diagnosticsError}
      />
    </div>
  )
}

// Full doctor detail: every setup check with its result. Home shows only the
// first blocker; this is the place that shows all of them, including the ones
// that passed, so the trust story is inspectable end to end.
function SetupChecksCard({
  dashboard,
  refreshing,
}: {
  dashboard: DashboardStatus | null
  refreshing: boolean
}) {
  const checks = dashboard?.setup.checks ?? []

  return (
    <Card aria-live="polite">
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Setup checks
        </CardTitle>
        <CardDescription>
          Everything RunHaven checks on this Mac before a run.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {checks.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            {refreshing
              ? "Checking your Mac's setup..."
              : "No setup checks recorded yet. Use Check again to load them."}
          </p>
        ) : (
          <ul className="flex flex-col gap-3">
            {checks.map((check) => (
              <li key={check.name} className="rounded-lg border p-3">
                <div className="flex items-center justify-between gap-3">
                  <div className="font-medium">{check.name}</div>
                  <Badge variant={check.ok ? "success" : "destructive"}>
                    {check.ok ? "Passed" : "Needs attention"}
                  </Badge>
                </div>
                <p className="mt-2 text-sm text-muted-foreground">
                  {check.detail}
                </p>
                {!check.ok ? (
                  <div className="mt-3 rounded-md bg-muted p-3 text-sm">
                    <div className="font-medium">Next step</div>
                    <p className="mt-1 text-muted-foreground">{check.remedy}</p>
                  </div>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}
