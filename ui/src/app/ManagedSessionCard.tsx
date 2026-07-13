import type {
  ManagedSessionSummary,
  SessionLifecycle,
  SessionNextActionKind,
} from "@/commands/runhaven"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { Metric } from "./Metric"
import { displayValue, sessionDisplayTitle } from "./sessionIdentity"

export function ManagedSessionCard({
  session,
}: {
  session: ManagedSessionSummary
}) {
  const needsAttention = sessionNeedsAttention(session)

  return (
    <Card>
      <CardHeader>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <CardTitle role="heading" aria-level={2}>
              {sessionDisplayTitle(session)}
            </CardTitle>
          </div>
          <Badge variant={sessionBadgeVariant(session)}>
            {needsAttention ? "Needs you" : session.statusLabel}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <Metric label="Assistant" value={displayValue(session.agent)} />
          <Metric label="Project" value={displayValue(session.project)} />
          <Metric label="Started" value={formatStartedAt(session.startedAt)} />
        </div>

      </CardContent>
    </Card>
  )
}

function formatStartedAt(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return displayValue(value)
  }
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date)
}

// The next-action kinds that mean the session needs the user, and the lifecycle
// states that still accept run controls. `satisfies` ties each literal set to
// the closed union at the commands boundary, so a typo or a dropped backend
// value is a compile error; the Set keeps the runtime check a simple lookup.
const ATTENTION_ACTION_KINDS = new Set<string>([
  "auth_attention",
  "recover",
] satisfies SessionNextActionKind[])

const RUN_CONTROL_LIFECYCLES = new Set<string>([
  "running",
  "starting",
  "stop-requested",
  "kill-requested",
] satisfies SessionLifecycle[])

export function sessionNeedsAttention(session: ManagedSessionSummary): boolean {
  return ATTENTION_ACTION_KINDS.has(session.nextAction.kind)
}

export function sessionAllowsRunControl(session: ManagedSessionSummary): boolean {
  return RUN_CONTROL_LIFECYCLES.has(session.lifecycle.trim().toLowerCase())
}

export function sessionAllowsCleanup(session: ManagedSessionSummary): boolean {
  return session.cleanup?.available === true
}

export function sessionBadgeVariant(
  session: ManagedSessionSummary,
): "destructive" | "secondary" | "success" | "warning" {
  if (sessionNeedsAttention(session)) {
    return "destructive"
  }

  switch (session.lifecycle.trim().toLowerCase()) {
    case "running":
      return "success"
    case "starting":
      return "secondary"
    case "failed":
      return "destructive"
    case "stop-requested":
    case "kill-requested":
      return "warning"
    default:
      return "secondary"
  }
}
