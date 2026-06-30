import { CircleAlert, ShieldCheck } from "lucide-react"

import type { ManagedSessionSummary } from "@/commands/runhaven"
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

export function ManagedSessionCard({
  session,
}: {
  session: ManagedSessionSummary
}) {
  const needsAttention = sessionNeedsAttention(session)

  return (
    <Card aria-live="polite">
      <CardHeader>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <CardTitle role="heading" aria-level={2}>
              {session.title}
            </CardTitle>
            <CardDescription>
              RunHaven is monitoring this session.
            </CardDescription>
          </div>
          <Badge variant={needsAttention ? "destructive" : "success"}>
            {needsAttention ? "Needs attention" : session.statusLabel}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {needsAttention ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Sign in or auth setup needs attention</AlertTitle>
            <AlertDescription>
              This session is still isolated. Finish auth in RunHaven or choose
              an agent with ready auth.
            </AlertDescription>
          </Alert>
        ) : (
          <Alert>
            <ShieldCheck />
            <AlertTitle>Credentials stay brokered or isolated</AlertTitle>
            <AlertDescription>
              Host home, browser profiles, cloud credentials, and raw SSH keys
              are not part of this session summary.
            </AlertDescription>
          </Alert>
        )}

        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          <Metric label="Agent" value={displayValue(session.agent)} />
          <Metric label="Project" value={displayValue(session.project)} />
          <Metric label="Session" value={displayValue(session.session)} />
          <Metric label="Status" value={displayValue(session.statusLabel)} />
          <Metric label="Auth" value={authLabel(session.authStatus)} />
          <Metric label="Network" value={networkLabel(session.network)} />
          <Metric label="Started" value={displayValue(session.startedAt)} />
        </div>
      </CardContent>
    </Card>
  )
}

export function sessionNeedsAttention(session: ManagedSessionSummary): boolean {
  if (session.agent === "shell") {
    return false
  }

  const status = session.authStatus.trim().toLowerCase()
  return !["api-key-broker", "brokered", "ready", "available", "n/a"].includes(
    status,
  )
}

function networkLabel(network: string): string {
  switch (network.trim().toLowerCase()) {
    case "provider":
      return "Provider only"
    case "internal":
      return "Local only"
    case "internet":
      return "Internet"
    default:
      return displayValue(network)
  }
}

function authLabel(status: string): string {
  if (status.trim().toLowerCase() === "api-key-broker") {
    return "API key broker"
  }
  return titleCase(status)
}

function titleCase(value: string): string {
  const cleaned = value.trim().replaceAll("-", " ")
  if (!cleaned) {
    return "Unknown"
  }
  return cleaned
    .split(/\s+/)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}

function displayValue(value: string): string {
  return value.trim() || "Unknown"
}
