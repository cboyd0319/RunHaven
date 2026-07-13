import { CheckCircle2, CircleAlert } from "lucide-react"

import type { RunStatusResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

import { Metric } from "./Metric"
import { formatMemory, type RunControlAction } from "./types"

type RunStatusCardProps = {
  title?: string
  showStatus?: boolean
  controlsAvailable?: boolean
  cleanupAvailable?: boolean
  runStatus: RunStatusResponse | null
  runStatusLoading: boolean
  runStatusError: string
  controlBusy: boolean
  controlError: string
  controlMessage: string
  onControl: (action: RunControlAction) => Promise<void>
}

export function RunStatusCard({
  title = "Run status",
  showStatus = true,
  controlsAvailable = true,
  cleanupAvailable = false,
  runStatus,
  runStatusLoading,
  runStatusError,
  controlBusy,
  controlError,
  controlMessage,
  onControl,
}: RunStatusCardProps) {
  const showControls =
    (controlsAvailable || cleanupAvailable) && (!showStatus || runStatus !== null)

  return (
    <Card aria-live="polite">
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {title}
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {showStatus && runStatusLoading ? (
          <p className="text-sm text-muted-foreground">Refreshing status...</p>
        ) : null}
        {showStatus && runStatusError ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Status unavailable</AlertTitle>
            <AlertDescription>{runStatusError}</AlertDescription>
          </Alert>
        ) : null}
        {showStatus && runStatus ? (
          <>
            <div className="grid gap-3 sm:grid-cols-3">
              <Metric label="Run record" value={runStatus.run.status} />
              <Metric label="Container state" value={runStatus.container.state} />
              <Metric label="Image" value={runStatus.container.image ?? "-"} />
              <Metric
                label="Started"
                value={runStatus.container.startedAt ?? "-"}
              />
              <Metric
                label="CPU"
                value={runStatus.container.resources.cpus ?? "unknown"}
              />
              <Metric
                label="Memory"
                value={formatMemory(runStatus.container.resources.memoryBytes)}
              />
            </div>
            {runStatus.container.networks.length > 0 ? (
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
                  {runStatus.container.networks.map((network, index) => (
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

          </>
        ) : null}
        {showControls ? (
          <div className="flex flex-col gap-3">
            {controlsAvailable ? (
              <div className="flex flex-wrap items-center gap-3">
                <Button
                  type="button"
                  variant="outline"
                  aria-label="Stop run"
                  disabled={controlBusy}
                  onClick={() => void onControl("stop")}
                >
                  Stop run
                </Button>
                <span className="text-xs text-muted-foreground">
                  RunHaven asks for confirmation in a Mac dialog.
                </span>
              </div>
            ) : null}
            {cleanupAvailable ? (
              <div className="flex flex-wrap items-center gap-3">
                <Button
                  type="button"
                  variant="outline"
                  aria-label="Clean up run"
                  disabled={controlBusy}
                  onClick={() => void onControl("cleanup")}
                >
                  Clean up run
                </Button>
                <span className="text-xs text-muted-foreground">
                  RunHaven asks for confirmation in a Mac dialog.
                </span>
              </div>
            ) : null}
            {controlsAvailable ? (
              <details className="rounded-lg border p-3">
                <summary className="cursor-pointer text-sm font-medium">
                  More actions
                </summary>
                <p className="mt-2 text-sm text-muted-foreground">
                  Use hard stop only when a normal stop does not work.
                </p>
                <div className="mt-3 flex flex-wrap gap-2">
                  <Button
                    type="button"
                    variant="destructive"
                    aria-label="Hard stop"
                    disabled={controlBusy}
                    onClick={() => void onControl("kill")}
                  >
                    Hard stop
                  </Button>
                </div>
              </details>
            ) : null}
          </div>
        ) : null}
        {controlMessage ? (
          <Alert role="status">
            <CheckCircle2 />
            <AlertTitle>{controlMessage}</AlertTitle>
          </Alert>
        ) : null}
        {controlError ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Run control failed</AlertTitle>
            <AlertDescription>{controlError}</AlertDescription>
          </Alert>
        ) : null}
      </CardContent>
    </Card>
  )
}
