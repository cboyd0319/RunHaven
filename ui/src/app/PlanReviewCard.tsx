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

import { authScopeLabel, networkLabel } from "./labels"
import { LaunchSteps } from "./LaunchSteps"
import { agentLabel } from "./sessionIdentity"

type PlanReviewCardProps = {
  plan: RunPlanResponse
  confirmedWarnings: string[]
  launchReady: boolean
  launching: boolean
  setupOk: boolean
  launchImageReady: boolean
  onConfirmWarning: (code: string, checked: boolean) => void
  onBack: () => void
  onStart: () => Promise<void>
}

export function PlanReviewCard({
  plan,
  confirmedWarnings,
  launchReady,
  launching,
  setupOk,
  launchImageReady,
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

        <dl className="grid gap-3 sm:grid-cols-2">
          <ReviewItem label="Agent" value={agentLabel(plan.profile)} />
          <ReviewItem label="Project" value={projectName(plan.workspace)} />
        </dl>

        <div className="rounded-lg border bg-muted/40 p-3 text-sm">
          <div className="font-medium">Project folder</div>
          <div className="mt-1 break-all text-muted-foreground">{plan.workspace}</div>
        </div>

        {plan.worktree ? (
          <Alert>
            <GitBranch />
            <AlertTitle>Recommended protection: isolated copy</AlertTitle>
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

        <details className="rounded-lg border p-3">
          <summary className="cursor-pointer text-sm font-medium">
            Technical details
          </summary>
          <dl className="mt-3 grid gap-3 text-sm sm:grid-cols-2">
            <ReviewItem label="Run name" value={plan.session} />
            <ReviewItem
              label="Sign-in sharing"
              value={authScopeLabel(plan.authScope)}
            />
            <ReviewItem label="Network" value={networkLabel(plan.networkMode)} />
            <ReviewItem label="Agent image" value={plan.image} />
          </dl>
          {plan.providerAllowedHosts.length > 0 ? (
            <p className="mt-3 text-sm text-muted-foreground">
              Can reach: {plan.providerAllowedHosts.join(", ")} only.
            </p>
          ) : null}
          <p className="mt-2 text-sm text-muted-foreground">{plan.egressSummary}</p>
        </details>

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
            {launching ? "Starting..." : "Start run"}
          </Button>
        </div>
        <p className="text-right text-xs text-muted-foreground">
          RunHaven will ask for final confirmation in a Mac dialog.
        </p>
      </CardContent>
    </Card>
  )
}

function ReviewItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 rounded-lg border p-3">
      <dt className="text-xs font-medium text-muted-foreground">{label}</dt>
      <dd className="mt-1 break-words font-medium">{value}</dd>
    </div>
  )
}

function projectName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean)
  return parts.at(-1) ?? path
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
