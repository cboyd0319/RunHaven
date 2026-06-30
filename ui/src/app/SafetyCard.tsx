import type { RunPlanRequest } from "@/commands/runhaven"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"

import { networkLabels } from "./types"

export function SafetyCard({
  request,
  imageReady,
}: {
  request: RunPlanRequest
  imageReady: boolean
}) {
  const checks = [
    "Workspace is mounted at /workspace only.",
    "Host home and credentials are not mounted.",
    `${networkLabels[request.networkMode]} is selected.`,
    imageReady ? "Image is ready." : "Image needs attention.",
  ]
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          What the agent can access
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
