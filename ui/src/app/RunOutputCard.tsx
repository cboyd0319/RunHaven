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
          <pre className="max-h-72 overflow-auto rounded-lg border bg-muted p-3 text-sm whitespace-pre-wrap">
            {logSnapshot.text}
          </pre>
        ) : null}
      </CardContent>
    </Card>
  )
}
