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

import { Metric } from "./Metric"
import { displayValue } from "./sessionIdentity"

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

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Latest output
        </CardTitle>
        <CardDescription>Load a bounded output snapshot.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <Button
          type="button"
          variant="outline"
          disabled={logLoading}
          aria-label={logLoading ? "Loading latest output" : "Show latest output"}
          onClick={() => void onLoad()}
        >
          {logLoading ? "Loading..." : "Show latest output"}
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
                value={outputSourceLabel(logSnapshot.source)}
              />
              <Metric
                label="Captured"
                value={displayValue(logSnapshot.capturedAt)}
              />
              <Metric
                label="Returned lines"
                value={String(logSnapshot.returnedLines)}
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
              {logSnapshot.text}
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
