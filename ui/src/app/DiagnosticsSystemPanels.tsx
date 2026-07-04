import { CheckCircle2, TriangleAlert } from "lucide-react"

import type { DashboardStatus } from "@/commands/runhaven"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { agentLabel } from "./sessionIdentity"
import type { DiagnosticsReadModels } from "./useDiagnosticsReadModels"

// The setup checks that carry a pinned version or runtime pin, surfaced as a
// focused "versions" lens. The names match the doctor checks in
// runhaven-core; if one is renamed the panel simply shows fewer rows (it never
// breaks), and the full check with its fix still lives in the Setup checks
// card above.
const VERSION_CHECK_NAMES = new Set<string>([
  "rust",
  "Apple container version",
  "Apple container runtime pins",
  "Apple container runtime commit",
])

// Read-only system panels for the Diagnostics view: a one-line health rollup,
// versions and pins, an image readiness matrix, and the stale-network count.
// Everything here re-surfaces data the app already reads; nothing mutates.
export function DiagnosticsSystemPanels({
  dashboard,
  readModels,
}: {
  dashboard: DashboardStatus | null
  readModels: DiagnosticsReadModels
}) {
  return (
    <>
      <HealthRollup dashboard={dashboard} />
      <VersionsPanel dashboard={dashboard} />
      <ImageReadinessPanel readModels={readModels} />
      <StaleNetworksPanel readModels={readModels} />
    </>
  )
}

// One green or amber status line for the whole Mac, derived from the setup
// checks. It summarizes, it does not re-list; the Setup checks card is the
// place that shows every check.
function HealthRollup({ dashboard }: { dashboard: DashboardStatus | null }) {
  if (!dashboard) {
    return null
  }
  const blockers = dashboard.setup.blockerCount
  const ok = dashboard.setup.ok
  return (
    <Card aria-live="polite">
      <CardContent>
        <p className="flex items-center gap-2 text-sm">
          {ok ? (
            <CheckCircle2 className="size-4 text-success" aria-hidden />
          ) : (
            <TriangleAlert className="size-4 text-warning" aria-hidden />
          )}
          {ok
            ? "Your Mac passes every setup check."
            : `${blockers} setup ${
                blockers === 1 ? "check needs" : "checks need"
              } attention. See the setup checks below.`}
        </p>
      </CardContent>
    </Card>
  )
}

// The app version (the one new value) plus the pinned runtime versions already
// verified in the setup checks, shown as a compact reference.
function VersionsPanel({ dashboard }: { dashboard: DashboardStatus | null }) {
  const versionChecks = (dashboard?.setup.checks ?? []).filter((check) =>
    VERSION_CHECK_NAMES.has(check.name),
  )
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Versions and pins
        </CardTitle>
        <CardDescription>
          The RunHaven app version and the runtime versions it expects.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2 text-sm">
        <div className="flex justify-between gap-3">
          <span className="font-medium">RunHaven app</span>
          <span className="text-muted-foreground">
            {dashboard?.appVersion ?? "Unknown"}
          </span>
        </div>
        {versionChecks.map((check) => (
          <div key={check.name} className="flex justify-between gap-3">
            <span className="font-medium">{check.name}</span>
            <span className="text-right text-muted-foreground">
              {check.detail}
            </span>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}

// A built/ready row per agent, read from the existing image status command.
function ImageReadinessPanel({
  readModels,
}: {
  readModels: DiagnosticsReadModels
}) {
  return (
    <Card aria-live="polite">
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Agent images
        </CardTitle>
        <CardDescription>
          Whether each agent's sandbox image is built and ready. Rebuild lives
          in the Agents view.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        {readModels.images.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            {readModels.loading
              ? "Checking agent images..."
              : "No agent images to check yet."}
          </p>
        ) : (
          <ul className="flex flex-col gap-2 text-sm">
            {readModels.images.map((row) => (
              <li key={row.agent} className="flex justify-between gap-3">
                <span className="font-medium">{agentLabel(row.agent)}</span>
                <ImageStateBadge row={row} />
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}

function ImageStateBadge({
  row,
}: {
  row: DiagnosticsReadModels["images"][number]
}) {
  if (!row.available) {
    return <Badge variant="secondary">Unavailable</Badge>
  }
  if (row.ready) {
    return <Badge variant="success">Ready</Badge>
  }
  return <Badge variant="warning">{row.state || "Not ready"}</Badge>
}

// The count of RunHaven's own leftover networks that could be pruned. Read-only
// here; the prune action itself stays in the Maintenance view.
function StaleNetworksPanel({
  readModels,
}: {
  readModels: DiagnosticsReadModels
}) {
  const count = readModels.staleNetworks
  return (
    <Card aria-live="polite">
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Stale networks
        </CardTitle>
        <CardDescription>
          Leftover RunHaven networks that could be cleaned up in Maintenance.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-muted-foreground">
          {count === null
            ? readModels.loading
              ? "Checking for stale networks..."
              : "Stale network count is unavailable."
            : count === 0
              ? "No stale networks to clean up."
              : `${count} stale ${
                  count === 1 ? "network" : "networks"
                } can be cleaned up in Maintenance.`}
        </p>
      </CardContent>
    </Card>
  )
}
