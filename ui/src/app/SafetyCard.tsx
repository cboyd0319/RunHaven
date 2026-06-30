import type { RunPlanRequest } from "@/commands/runhaven"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"

import { networkLabels } from "./types"

export function SafetyCard({
  request,
}: {
  request: RunPlanRequest
  imageReady: boolean
}) {
  const checks = [
    "Project folder only",
    "Home and credentials not shared",
    networkLabels[request.networkMode],
  ]
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Access summary
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {checks.map((check) => (
          <div key={check} className="flex items-start gap-3 rounded-lg border p-3">
            <Checkbox checked aria-label={check} />
            <span className="text-sm">{check}</span>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
