import { CircleAlert } from "lucide-react"

import type { LogSnapshotResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"

import { Metric } from "./Metric"

export function RunOutputCard({
  logAcknowledged,
  logLoading,
  logError,
  logSnapshot,
  onAcknowledgedChange,
  onLoad,
}: {
  logAcknowledged: boolean
  logLoading: boolean
  logError: string
  logSnapshot: LogSnapshotResponse | null
  onAcknowledgedChange: (checked: boolean) => void
  onLoad: () => Promise<void>
}) {
  const primaryEntry = logSnapshot?.entries[0]
  const snapshotSource = primaryEntry?.source ?? logSnapshot?.source ?? ""
  const snapshotCaptured =
    logSnapshot?.cursor ?? primaryEntry?.timestamp ?? logSnapshot?.capturedAt ?? ""
  const snapshotLineCount =
    primaryEntry?.lineCount ?? logSnapshot?.returnedLines ?? 0
  const snapshotText = primaryEntry?.text ?? logSnapshot?.text ?? ""

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Run output
        </CardTitle>
        <CardDescription>
          Raw output can contain secrets. Confirm before loading it.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <label className="flex items-center gap-3 rounded-lg border p-3 text-sm">
          <Checkbox
            checked={logAcknowledged}
            onCheckedChange={(checked) => onAcknowledgedChange(checked === true)}
          />
          <span>Show raw container output for this run.</span>
        </label>
        <Button
          type="button"
          variant="outline"
          disabled={!logAcknowledged || logLoading}
          onClick={() => void onLoad()}
        >
          {logLoading ? "Loading..." : "View latest output"}
        </Button>
        {logError ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Output unavailable</AlertTitle>
            <AlertDescription>{logError}</AlertDescription>
          </Alert>
        ) : null}
        {logSnapshot ? (
          <div className="flex flex-col gap-3">
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
              <Metric
                label="Source"
                value={outputSourceLabel(snapshotSource)}
              />
              <Metric
                label="Captured"
                value={displayValue(snapshotCaptured)}
              />
              <Metric
                label="Returned lines"
                value={String(snapshotLineCount)}
              />
              <Metric
                label="Truncated"
                value={logSnapshot.truncated ? "Yes" : "No"}
              />
            </div>
            {logSnapshot.warnings.length > 0 ? (
              <ul className="rounded-lg border px-3 py-2 text-sm text-muted-foreground">
                {logSnapshot.warnings.map((warning) => (
                  <li key={warning}>{warning}</li>
                ))}
              </ul>
            ) : null}
            <pre
              aria-label="Raw output snapshot"
              className="max-h-72 overflow-auto rounded-lg border bg-muted p-3 text-sm whitespace-pre-wrap"
            >
              {snapshotText}
            </pre>
          </div>
        ) : null}
      </CardContent>
    </Card>
  )
}

function outputSourceLabel(source: string): string {
  switch (source.trim().toLowerCase()) {
    case "container-stdio":
      return "Container output"
    default:
      return displayValue(source)
  }
}

function displayValue(value: string): string {
  return value.trim() || "Unknown"
}
