import type { DashboardStatus } from "@/commands/runhaven"
import { Badge } from "@/components/ui/badge"
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
}: {
  loading: boolean
  dashboard: DashboardStatus | null
}) {
  const failedChecks = (dashboard?.setup.checks ?? []).filter(
    (check) => !check.ok,
  )
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Status
        </CardTitle>
        <CardDescription>
          {dashboard?.setup.ok
            ? "Ready to launch."
            : "Fix blockers before launch."}
        </CardDescription>
        <CardAction>
          <Badge variant={dashboard?.setup.ok ? "success" : "destructive"}>
            {dashboard?.setup.ok ? "Ready" : "Blocked"}
          </Badge>
        </CardAction>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {loading ? (
          <p className="text-sm text-muted-foreground">Loading setup...</p>
        ) : null}
        {dashboard?.setup.ok && !loading ? (
          <p className="text-sm text-muted-foreground">All checks passed.</p>
        ) : null}
        {failedChecks.map((check) => (
          <div key={check.name} className="rounded-lg border p-3">
            <div className="flex items-center justify-between gap-3">
              <div className="font-medium">{check.name}</div>
              <Badge variant="destructive">Fix</Badge>
            </div>
            <p className="mt-1 text-sm text-muted-foreground">{check.detail}</p>
            <p className="mt-1 text-sm text-muted-foreground">{check.remedy}</p>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
