import { CircleAlert, Network, ShieldCheck } from "lucide-react"
import type { ReactNode } from "react"

import type { ManagedSessionDiagnosticsResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

import { authStatusLabel } from "./labels"
import { agentLabel, displayValue } from "./sessionIdentity"

export function ManagedSessionDiagnosticsCard({
  diagnostics,
  loading,
  error,
  onLoad,
}: {
  diagnostics: ManagedSessionDiagnosticsResponse | null
  loading: boolean
  error: string
  onLoad: () => Promise<void>
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Session diagnostics
        </CardTitle>
        <CardDescription>
          Why this session's sign-in and network requests were allowed or denied.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <Button type="button" variant="outline" onClick={() => void onLoad()}>
          <ShieldCheck className="size-4" />
          {loading
            ? "Loading diagnostics…"
            : diagnostics
              ? "Refresh diagnostics"
              : "Load session diagnostics"}
        </Button>
        {error ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Diagnostics unavailable</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
        {diagnostics ? (
          <>
            <Alert>
              <ShieldCheck />
              <AlertTitle>{agentLabel(diagnostics.authStatus.name)}</AlertTitle>
              <AlertDescription>
                Sign-in method: {authStatusLabel(diagnostics.authStatus.status)}
              </AlertDescription>
            </Alert>
            <SessionLogTable
              title="Sign-in records"
              empty="No sign-in records for this session."
              rows={diagnostics.authLogEntries.map((entry, index) => ({
                // Index first: the log writer stamps one timestamp per batch,
                // so timestamp-path-decision alone can collide within a batch.
                key: `${index}-${entry.timestamp}-${entry.path}-${entry.decision}`,
                first: entry.path,
                second: decisionLabel(entry.decision),
                third: usefulRecordedReason(entry.reason, entry.decision),
              }))}
            />
            <SessionLogTable
              title="Network records"
              empty="No network records for this session."
              rows={diagnostics.egressLogEntries.map((entry, index) => ({
                key: `${index}-${entry.timestamp}-${entry.host}-${entry.decision}`,
                first: entry.host,
                second: decisionLabel(entry.decision),
                third: networkReason(entry.decision, entry.profile),
              }))}
              icon={<Network className="size-4" />}
            />
          </>
        ) : null}
      </CardContent>
    </Card>
  )
}

function SessionLogTable({
  title,
  empty,
  rows,
  icon,
}: {
  title: string
  empty: string
  rows: Array<{ key: string; first: string; second: string; third: string }>
  icon?: ReactNode
}) {
  const showReason = rows.some((row) => row.third !== "")
  return (
    <section>
      <h3 className="mb-2 flex items-center gap-2 text-sm font-medium">
        {icon}
        {title}
      </h3>
      {rows.length === 0 ? (
        <p className="text-sm text-muted-foreground">{empty}</p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Target</TableHead>
              <TableHead>Result</TableHead>
              {showReason ? <TableHead>Reason</TableHead> : null}
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.map((row) => (
              <TableRow key={row.key}>
                <TableCell>{row.first}</TableCell>
                <TableCell>{row.second}</TableCell>
                {showReason ? <TableCell>{row.third}</TableCell> : null}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </section>
  )
}

function decisionLabel(decision: string): string {
  switch (decision.trim().toLowerCase()) {
    case "allowed":
      return "Allowed"
    case "denied":
    case "blocked":
      return "Blocked"
    default:
      return displayValue(decision)
  }
}

function usefulRecordedReason(reason: string, decision: string): string {
  const normalized = reason.trim().toLowerCase()
  if (["", "ok", "allowed", "denied", "blocked"].includes(normalized)) {
    return ""
  }
  if (normalized === decision.trim().toLowerCase()) {
    return ""
  }
  return displayValue(reason)
}

function networkReason(decision: string, agent: string): string {
  if (decision.trim().toLowerCase() === "allowed") {
    return `Allowed because this is a service ${agentLabel(agent)} needs.`
  }
  return "Blocked because it is outside the network access chosen for this session."
}
