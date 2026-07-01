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
import { displayValue } from "./sessionIdentity"

export function RunOutputCard({
  logAcknowledged,
  logLoading,
  logError,
  logSnapshot,
  liveOutputAvailable,
  liveOutputEnabled,
  liveOutputRefreshing,
  onAcknowledgedChange,
  onLoad,
  onLiveOutputChange,
}: {
  logAcknowledged: boolean
  logLoading: boolean
  logError: string
  logSnapshot: LogSnapshotResponse | null
  liveOutputAvailable: boolean
  liveOutputEnabled: boolean
  liveOutputRefreshing?: boolean
  onAcknowledgedChange: (checked: boolean) => void
  onLoad: () => Promise<void>
  onLiveOutputChange: (checked: boolean) => void
}) {
  const sessionEntry = logSnapshot?.sessionEntries?.[0]
  const primaryEntry = sessionEntry ?? logSnapshot?.entries[0]
  const snapshotSource = primaryEntry?.source ?? logSnapshot?.source ?? ""
  const snapshotCaptured =
    sessionEntry?.cursor ??
    logSnapshot?.cursor ??
    primaryEntry?.timestamp ??
    logSnapshot?.capturedAt ??
    ""
  const snapshotLineCount =
    primaryEntry?.lineCount ?? logSnapshot?.returnedLines ?? 0
  const snapshotText = primaryEntry?.text ?? logSnapshot?.text ?? ""
  const warnings = logSnapshot
    ? sessionEntry?.redaction.message
      ? [...logSnapshot.warnings, sessionEntry.redaction.message]
      : logSnapshot.warnings
    : []

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Latest output
        </CardTitle>
        <CardDescription>Load a bounded output snapshot.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <label className="flex items-center gap-3 rounded-lg border p-3 text-sm">
          <Checkbox
            checked={logAcknowledged}
            aria-label="I understand raw output may include private data."
            onCheckedChange={(checked) => {
              const acknowledged = checked === true
              onAcknowledgedChange(acknowledged)
              if (!acknowledged) {
                onLiveOutputChange(false)
              }
            }}
          />
          <span>I understand raw output may include private data.</span>
        </label>
        {liveOutputAvailable ? (
          <label className="flex items-center gap-3 rounded-lg border p-3 text-sm">
            <Checkbox
              checked={liveOutputEnabled}
              disabled={!logAcknowledged}
              aria-label="Keep output updated"
              onCheckedChange={(checked) => onLiveOutputChange(checked === true)}
            />
            <span className="flex min-w-0 flex-col gap-1">
              <span>Keep output updated</span>
              <span className="text-muted-foreground">
                {!logAcknowledged
                  ? "Confirm raw output first."
                  : liveOutputEnabled
                  ? liveOutputRefreshing
                    ? "Checking for new lines."
                    : "Updates every few seconds."
                  : "Refresh while this view stays open."}
              </span>
            </span>
          </label>
        ) : null}
        <Button
          type="button"
          variant="outline"
          disabled={!logAcknowledged || logLoading}
          aria-label={
            logLoading
              ? "Loading latest output"
              : logAcknowledged
              ? "Load latest output"
              : "Load latest output: confirm raw output first"
          }
          onClick={() => void onLoad()}
        >
          {logLoading ? "Loading..." : "Load latest output"}
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
            {warnings.length > 0 ? (
              <ul
                aria-label="Output warnings"
                className="rounded-lg border px-3 py-2 text-sm text-muted-foreground"
              >
                {warnings.map((warning) => (
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
