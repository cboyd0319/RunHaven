import { CircleAlert, FileText, GitBranch, ShieldCheck } from "lucide-react"

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

import { authScopeLabel } from "./labels"
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
              ["Login state", authScopeLabel(plan.authScope)],
              ["Network", plan.networkMode],
              ["Image", plan.image],
            ].map(([label, value]) => (
              <TableRow key={label}>
                <TableHead className="w-40">{label}</TableHead>
                <TableCell className="break-all">{value}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>

        {plan.worktree ? (
          <Alert>
            <GitBranch />
            <AlertTitle>Isolated copy (git worktree)</AlertTitle>
            <AlertDescription>
              {plan.worktreeNote ??
                "RunHaven will run this agent in an isolated copy of your project on a new git branch."}
            </AlertDescription>
          </Alert>
        ) : null}

        {plan.workspaceScopeNote ? (
          <Alert>
            <FileText />
            <AlertTitle>Workspace scope</AlertTitle>
            <AlertDescription>{plan.workspaceScopeNote}</AlertDescription>
          </Alert>
        ) : null}

        <Alert variant={plan.access.blocking ? "destructive" : undefined}>
          {plan.access.blocking ? <CircleAlert /> : <ShieldCheck />}
          <AlertTitle>{plan.access.label}</AlertTitle>
          <AlertDescription>
            <p>{plan.access.detail}</p>
            {plan.access.remedy ? <p>{plan.access.remedy}</p> : null}
          </AlertDescription>
        </Alert>

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
          <p id="launch-reason-setup" className="text-sm text-muted-foreground">
            Fix setup blockers before launching a run.
          </p>
        ) : null}
        {!launchImageReady ? (
          <p id="launch-reason-image" className="text-sm text-muted-foreground">
            Fix image readiness before launching a run.
          </p>
        ) : null}
        {plan.access.blocking ? (
          <p id="launch-reason-access" className="text-sm text-muted-foreground">
            Fix access before launching a run.
          </p>
        ) : null}

        <div className="flex justify-between gap-3">
          <Button type="button" variant="outline" onClick={onBack}>
            Back
          </Button>
          <Button
            type="button"
            disabled={!launchReady || launching}
            aria-describedby={launchBlockedReasonIds(
              setupOk,
              launchImageReady,
              plan.access.blocking,
            )}
            onClick={() => void onStart()}
          >
            {launching ? "Launching..." : "Launch run"}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

// Point the disabled "Launch run" button at whichever blocking reasons are on
// screen, so assistive tech reads why it cannot be pressed.
function launchBlockedReasonIds(
  setupOk: boolean,
  launchImageReady: boolean,
  accessBlocking: boolean,
): string | undefined {
  const ids = [
    setupOk ? null : "launch-reason-setup",
    launchImageReady ? null : "launch-reason-image",
    accessBlocking ? "launch-reason-access" : null,
  ].filter((id): id is string => id !== null)
  return ids.length > 0 ? ids.join(" ") : undefined
}
