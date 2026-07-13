import { CircleAlert } from "lucide-react"

import type { ManagedSessionStatusResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { ActivityIndicator } from "@/components/ui/activity-indicator"
import { Badge } from "@/components/ui/badge"
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

import { Metric } from "./Metric"
import { formatMemory } from "./types"

export function ManagedSessionStatusCard({
  status,
  loading,
  error,
}: {
  status: ManagedSessionStatusResponse | null
  loading: boolean
  error: string
}) {
  const container = status?.container ?? null

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Runtime status
        </CardTitle>
        <CardDescription>Current state for this session.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {loading ? (
          <p role="status" className="flex items-center gap-2 text-sm text-muted-foreground">
            <ActivityIndicator />
            Checking status…
          </p>
        ) : null}
        {error ? (
          <Alert variant="destructive" role="alert">
            <CircleAlert />
            <AlertTitle>Status unavailable</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
        {status?.statusUnavailable ? (
          <Alert role="status">
            <CircleAlert />
            <AlertTitle>Live status not available</AlertTitle>
            <AlertDescription>{status.statusUnavailable}</AlertDescription>
          </Alert>
        ) : null}
        {status?.recovery ? (
          <Alert
            role={status.recovery.repairAvailable ? "alert" : "status"}
            variant={status.recovery.repairAvailable ? "warning" : undefined}
          >
            <CircleAlert />
            <AlertTitle>{status.recovery.title}</AlertTitle>
            <AlertDescription>
              <p>{status.recovery.detail}</p>
              <p>{status.recovery.remedy}</p>
              {status.recovery.repairAvailable ? (
                <Badge variant="warning">Repair available</Badge>
              ) : null}
            </AlertDescription>
          </Alert>
        ) : null}
        {status ? (
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            <Metric label="State" value={status.session.statusLabel} />
            <Metric label="Input" value={status.session.input.status} />
            <Metric label="Container" value={container?.state ?? "Unavailable"} />
            <Metric label="Image" value={container?.image ?? "Unknown"} />
            <Metric label="CPU" value={container?.resources.cpus ?? "Unknown"} />
            <Metric
              label="Memory"
              value={formatMemory(container?.resources.memoryBytes ?? null)}
            />
          </div>
        ) : null}
        {container && container.networks.length > 0 ? (
          <div className="overflow-x-auto">
            <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Network</TableHead>
                <TableHead>Host</TableHead>
                <TableHead>IPv4</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {container.networks.map((network, index) => (
                <TableRow key={`${network.network ?? "network"}-${index}`}>
                  <TableCell>{network.network ?? "network"}</TableCell>
                  <TableCell>{network.hostname ?? "-"}</TableCell>
                  <TableCell>{network.ipv4Address ?? "-"}</TableCell>
                </TableRow>
              ))}
            </TableBody>
            </Table>
          </div>
        ) : null}
      </CardContent>
    </Card>
  )
}
