import { CircleAlert, FileText } from "lucide-react"

import type { RunPlanResponse } from "@/commands/runhaven"
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
import { Separator } from "@/components/ui/separator"
import { Table, TableBody, TableCell, TableHead, TableRow } from "@/components/ui/table"

import { LaunchSteps } from "./LaunchSteps"

type PlanReviewCardProps = {
  plan: RunPlanResponse
  launchConfirmation: boolean
  confirmedWarnings: string[]
  launchReady: boolean
  launching: boolean
  setupOk: boolean
  launchImageReady: boolean
  onLaunchConfirmationChange: (checked: boolean) => void
  onConfirmWarning: (code: string, checked: boolean) => void
  onBack: () => void
  onStart: () => Promise<void>
}

export function PlanReviewCard({
  plan,
  launchConfirmation,
  confirmedWarnings,
  launchReady,
  launching,
  setupOk,
  launchImageReady,
  onLaunchConfirmationChange,
  onConfirmWarning,
  onBack,
  onStart,
}: PlanReviewCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Review access
        </CardTitle>
        <CardDescription>
          Confirm this is the agent, project, and access you expect.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <LaunchSteps step="review" />

        <Table>
          <TableBody>
            {[
              ["Agent", plan.profile],
              ["Project", plan.workspace],
              ["Session", plan.session],
              ["Auth scope", authScopeLabel(plan.authScope)],
              ["Agent memory", plan.stateVolume],
              ["Network", plan.networkMode],
              ["Image", plan.image],
              ["Preflight steps", String(plan.preflightCount)],
            ].map(([label, value]) => (
              <TableRow key={label}>
                <TableHead className="w-40">{label}</TableHead>
                <TableCell className="break-all">{value}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>

        {plan.workspaceScopeNote ? (
          <Alert>
            <FileText />
            <AlertTitle>Workspace scope</AlertTitle>
            <AlertDescription>{plan.workspaceScopeNote}</AlertDescription>
          </Alert>
        ) : null}

        <p className="text-sm text-muted-foreground">{plan.egressSummary}</p>

        {plan.warnings.length > 0 ? (
          <Alert variant="warning">
            <CircleAlert />
            <AlertTitle>Warnings need confirmation</AlertTitle>
            <AlertDescription>
              <div className="mt-2 flex flex-col gap-2">
                {plan.warnings.map((warning) => (
                  <label key={warning.code} className="flex items-start gap-3">
                    <Checkbox
                      checked={confirmedWarnings.includes(warning.code)}
                      onCheckedChange={(checked) =>
                        onConfirmWarning(warning.code, checked === true)
                      }
                      aria-label={`Confirm ${warning.code} warning`}
                    />
                    <span>{warning.message}</span>
                  </label>
                ))}
              </div>
            </AlertDescription>
          </Alert>
        ) : null}

        <Separator />

        <label className="flex items-start gap-3 rounded-lg border p-3 text-sm">
          <Checkbox
            checked={launchConfirmation}
            onCheckedChange={(checked) =>
              onLaunchConfirmationChange(checked === true)
            }
          />
          <span>I reviewed this plan and want to start this run.</span>
        </label>

        {!setupOk ? (
          <p className="text-sm text-muted-foreground">
            Fix setup blockers before launching a run.
          </p>
        ) : null}
        {!launchImageReady ? (
          <p className="text-sm text-muted-foreground">
            Fix image readiness before launching a run.
          </p>
        ) : null}

        <div className="flex justify-between gap-3">
          <Button type="button" variant="outline" onClick={onBack}>
            Back
          </Button>
          <Button
            type="button"
            disabled={!launchReady || launching}
            onClick={() => void onStart()}
          >
            {launching ? "Launching..." : "Launch run"}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

function authScopeLabel(value: string): string {
  switch (value.trim().toLowerCase()) {
    case "agent":
      return "Shared agent login"
    case "project":
      return "Project session"
    default:
      return value.trim() || "Unknown"
  }
}
