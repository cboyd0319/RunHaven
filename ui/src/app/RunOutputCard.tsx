import { CircleAlert } from "lucide-react"

import type { LogSnapshotResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { ActivityIndicator } from "@/components/ui/activity-indicator"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { Metric } from "./Metric"
import { displayValue } from "./sessionIdentity"
import { isPastOutputUnavailable } from "./useRunOutput"

export function RunOutputCard({
  logLoading,
  logError,
  logSnapshot,
  onLoad,
}: {
  logLoading: boolean
  logError: string
  logSnapshot: LogSnapshotResponse | null
  onLoad: () => Promise<void>
}) {
  const warnings = logSnapshot?.warnings ?? []
  const outputUnavailable = isPastOutputUnavailable(logError)

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Latest output
        </CardTitle>
        <CardDescription>
          View the latest filtered session output kept in memory.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {!outputUnavailable ? (
          <Button
            type="button"
            variant="outline"
            disabled={logLoading}
            onClick={() => void onLoad()}
          >
            {logLoading ? <ActivityIndicator /> : null}
            {logLoading
              ? "Loading…"
              : logSnapshot
                ? "Refresh output"
                : "Show output…"}
          </Button>
        ) : null}
        {logError ? (
          <Alert
            role={outputUnavailable ? "status" : "alert"}
            variant={outputUnavailable ? "default" : "destructive"}
          >
            <CircleAlert />
            <AlertTitle>Output unavailable</AlertTitle>
            <AlertDescription>{logError}</AlertDescription>
          </Alert>
        ) : null}
        {logSnapshot ? (
          <div className="flex flex-col gap-3">
            <pre
              aria-label="Session output"
              className="max-h-72 overflow-auto rounded-lg border bg-muted p-3 text-sm whitespace-pre-wrap"
            >
              {logSnapshot.text}
            </pre>
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
            <details className="rounded-lg border p-3">
              <summary className="cursor-pointer text-sm font-medium">
                Technical details
              </summary>
              <div className="mt-3 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
                <Metric
                  label="Source"
                  value={outputSourceLabel(logSnapshot.source)}
                />
                <Metric
                  label="Captured"
                  value={displayValue(logSnapshot.capturedAt)}
                />
                <Metric
                  label="Lines shown"
                  value={String(logSnapshot.returnedLines)}
                />
                <Metric
                  label="Shortened"
                  value={logSnapshot.truncated ? "Yes" : "No"}
                />
              </div>
            </details>
          </div>
        ) : null}
      </CardContent>
    </Card>
  )
}

function outputSourceLabel(source: string): string {
  switch (source.trim().toLowerCase()) {
    case "container-stdio":
      return "Session output"
    default:
      return displayValue(source)
  }
}
