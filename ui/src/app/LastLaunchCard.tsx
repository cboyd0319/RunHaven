import type { LaunchRunResponse } from "@/commands/runhaven"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { authScopeLabel, networkLabel } from "./labels"
import { Metric } from "./Metric"

export function LastLaunchCard({ lastLaunch }: { lastLaunch: LaunchRunResponse }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Last launch
        </CardTitle>
        <CardDescription>{lastLaunch.runId}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid gap-3 sm:grid-cols-3">
          <Metric label="Agent" value={lastLaunch.profile} />
          <Metric label="Auth scope" value={authScopeLabel(lastLaunch.authScope)} />
          <Metric label="Network" value={networkLabel(lastLaunch.networkMode)} />
          <Metric label="Container" value={lastLaunch.snapshot.containerName} />
        </div>
      </CardContent>
    </Card>
  )
}
