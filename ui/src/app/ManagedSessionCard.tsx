import { CircleAlert, ShieldCheck } from "lucide-react"

import type {
  ManagedSessionSummary,
  SessionLifecycle,
  SessionNextActionKind,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { Metric } from "./Metric"
import { authScopeLabel, authStatusLabel, networkLabel } from "./labels"
import { useSessionLabels } from "./sessionLabels"
import { displayValue, sessionDisplayTitle } from "./sessionIdentity"

export function ManagedSessionCard({
  session,
}: {
  session: ManagedSessionSummary
}) {
  const needsAttention = sessionNeedsAttention(session)
  const action = session.nextAction
  const { labels } = useSessionLabels()

  return (
    <Card aria-live="polite">
      <CardHeader>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <CardTitle role="heading" aria-level={2}>
              {sessionDisplayTitle(session, labels)}
            </CardTitle>
            <CardDescription>Selected session overview.</CardDescription>
          </div>
          <Badge variant={sessionBadgeVariant(session)}>
            {needsAttention ? "Needs attention" : session.statusLabel}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {needsAttention ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>{displayValue(action.label)}</AlertTitle>
            <AlertDescription>
              {displayValue(action.detail)}
            </AlertDescription>
          </Alert>
        ) : (
          <Alert role="status">
            <ShieldCheck />
            <AlertTitle>{displayValue(action.label)}</AlertTitle>
            <AlertDescription>
              {displayValue(action.detail)}
            </AlertDescription>
          </Alert>
        )}

        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          <Metric label="Agent" value={displayValue(session.agent)} />
          <Metric label="Project" value={displayValue(session.project)} />
          <Metric label="Session name" value={displayValue(session.session)} />
          <Metric label="Status" value={displayValue(session.statusLabel)} />
          <Metric label="Auth" value={authStatusLabel(session.authStatus)} />
          <Metric label="Login state" value={authScopeLabel(session.authScope)} />
          <Metric label="Network" value={networkLabel(session.network)} />
          <Metric label="Started" value={displayValue(session.startedAt)} />
        </div>

        {session.events.length > 0 ? (
          <section aria-labelledby="managed-session-events">
            <h3
              id="managed-session-events"
              className="mb-2 text-sm font-medium"
            >
              Recent events
            </h3>
            <ul className="divide-y rounded-lg border">
              {session.events.map((event) => (
                <li
                  className="flex flex-col gap-1 px-3 py-2 sm:flex-row sm:items-center sm:justify-between"
                  key={`${event.source}-${event.label}-${event.timestamp}`}
                >
                  <div className="flex min-w-0 items-center gap-2">
                    <Badge variant="outline">{displayValue(event.source)}</Badge>
                    <span className="truncate font-medium">
                      {displayValue(event.label)}
                    </span>
                  </div>
                  <span className="text-sm text-muted-foreground">
                    {displayValue(event.detail)}
                  </span>
                </li>
              ))}
            </ul>
          </section>
        ) : null}
      </CardContent>
    </Card>
  )
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

function sessionBadgeVariant(
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
