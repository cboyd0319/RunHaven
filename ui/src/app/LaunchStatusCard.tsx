import { CircleAlert, PlayCircle } from "lucide-react"

import type { LaunchStatusResponse } from "@/commands/runhaven"
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

export function LaunchStatusCard({
  status,
  loading,
  error,
}: {
  status: LaunchStatusResponse | null
  loading: boolean
  error: string
}) {
  return (
    <Card>
      <CardHeader>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <CardTitle role="heading" aria-level={2}>
              Launch status
            </CardTitle>
            <CardDescription>Latest launch state.</CardDescription>
          </div>
          {status ? (
            <Badge variant={launchStatusVariant(status.state)}>
              {status.label}
            </Badge>
          ) : null}
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {loading ? (
          <p role="status" className="flex items-center gap-2 text-sm text-muted-foreground">
            <ActivityIndicator />
            Checking launch…
          </p>
        ) : null}
        {error ? (
          <Alert variant="destructive" role="alert">
            <CircleAlert />
            <AlertTitle>Launch status not available</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
        {status ? (
          <Alert
            role={status.state === "failed" ? "alert" : "status"}
            variant={status.state === "failed" ? "destructive" : undefined}
          >
            <PlayCircle />
            <AlertTitle>{status.label}</AlertTitle>
            <AlertDescription>{status.detail}</AlertDescription>
          </Alert>
        ) : null}
      </CardContent>
    </Card>
  )
}

function launchStatusVariant(
  state: string,
): "destructive" | "secondary" | "success" | "warning" {
  switch (state.trim().toLowerCase()) {
    case "running":
    case "finished":
      return "success"
    case "starting":
      return "secondary"
    case "failed":
      return "destructive"
    default:
      return "secondary"
  }
}
