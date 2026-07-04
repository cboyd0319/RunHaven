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
          Auth and network records for the selected session only.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <Button type="button" variant="outline" onClick={() => void onLoad()}>
          <ShieldCheck className="size-4" />
          {loading
            ? "Loading diagnostics..."
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
              <AlertTitle>{diagnostics.authStatus.name}</AlertTitle>
              <AlertDescription>
                Auth state: {diagnostics.authStatus.status}
              </AlertDescription>
            </Alert>
            <SessionLogTable
              title="Auth records"
              empty="No auth broker records for this session."
              rows={diagnostics.authLogEntries.map((entry, index) => ({
                // Index first: the log writer stamps one timestamp per batch,
                // so timestamp-path-decision alone can collide within a batch.
                key: `${index}-${entry.timestamp}-${entry.path}-${entry.decision}`,
                first: entry.path,
                second: entry.decision,
                third: entry.reason,
              }))}
            />
            <SessionLogTable
              title="Network records"
              empty="No provider network records for this session."
              rows={diagnostics.egressLogEntries.map((entry, index) => ({
                key: `${index}-${entry.timestamp}-${entry.host}-${entry.decision}`,
                first: entry.host,
                second: entry.decision,
                third: entry.reason,
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
              <TableHead>Decision</TableHead>
              <TableHead>Reason</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.map((row) => (
              <TableRow key={row.key}>
                <TableCell>{row.first}</TableCell>
                <TableCell>{row.second}</TableCell>
                <TableCell>{row.third}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </section>
  )
}
