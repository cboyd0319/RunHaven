import { CheckCircle2, CircleAlert } from "lucide-react"

import type { RunStatusResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"
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
  runStatus: RunStatusResponse | null
  runStatusLoading: boolean
  runStatusError: string
  controlBusy: boolean
  controlError: string
  controlMessage: string
  controlConfirmations: Record<RunControlAction, boolean>
  onConfirmationChange: (action: RunControlAction, checked: boolean) => void
  onControl: (action: RunControlAction) => Promise<void>
}

export function RunStatusCard({
  runStatus,
  runStatusLoading,
  runStatusError,
  controlBusy,
  controlError,
  controlMessage,
  controlConfirmations,
  onConfirmationChange,
  onControl,
}: RunStatusCardProps) {
  return (
    <Card aria-live="polite">
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Run status
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {runStatusLoading ? (
          <p className="text-sm text-muted-foreground">Refreshing status...</p>
        ) : null}
        {runStatusError ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Status unavailable</AlertTitle>
            <AlertDescription>{runStatusError}</AlertDescription>
          </Alert>
        ) : null}
        {runStatus ? (
          <>
            <div className="grid gap-3 sm:grid-cols-3">
              <Metric label="Marker status" value={runStatus.run.status} />
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
            ) : null}

            <div className="grid gap-3">
              <RunControl
                label="Confirm stopping this run."
                button="Stop run"
                checked={controlConfirmations.stop}
                disabled={controlBusy}
                onCheckedChange={(checked) =>
                  onConfirmationChange("stop", checked)
                }
                onClick={() => void onControl("stop")}
              />
              <RunControl
                label="Confirm hard-stopping this run."
                button="Hard stop"
                checked={controlConfirmations.kill}
                disabled={controlBusy}
                onCheckedChange={(checked) =>
                  onConfirmationChange("kill", checked)
                }
                onClick={() => void onControl("kill")}
              />
              <RunControl
                label="Confirm clearing this run's stale marker."
                button="Repair marker"
                checked={controlConfirmations.repair}
                disabled={controlBusy}
                onCheckedChange={(checked) =>
                  onConfirmationChange("repair", checked)
                }
                onClick={() => void onControl("repair")}
              />
            </div>
          </>
        ) : null}
        {controlMessage ? (
          <Alert>
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

function RunControl({
  label,
  button,
  checked,
  disabled,
  onCheckedChange,
  onClick,
}: {
  label: string
  button: string
  checked: boolean
  disabled: boolean
  onCheckedChange: (checked: boolean) => void
  onClick: () => void
}) {
  return (
    <div className="grid gap-2 rounded-lg border p-3 sm:grid-cols-[1fr_auto] sm:items-center">
      <label className="flex items-center gap-3 text-sm">
        <Checkbox
          checked={checked}
          onCheckedChange={(value) => onCheckedChange(value === true)}
        />
        <span>{label}</span>
      </label>
      <Button
        type="button"
        variant="outline"
        disabled={!checked || disabled}
        onClick={onClick}
      >
        {button}
      </Button>
    </div>
  )
}
