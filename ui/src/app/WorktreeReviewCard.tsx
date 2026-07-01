import { CheckCircle2, CircleAlert, GitBranch } from "lucide-react"

import type { WorktreeReviewResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"

import { Metric } from "./Metric"
import {
  WORKTREE_ACTIONS,
  type WorktreeAction,
} from "./worktreeReview"

export type WorktreeReviewCardProps = {
  review: WorktreeReviewResponse | null
  loading: boolean
  loadError: string
  confirmations: Record<WorktreeAction, boolean>
  busy: boolean
  message: string
  nextSteps: string[]
  actionError: string
  onConfirmationChange: (action: WorktreeAction, checked: boolean) => void
  onAction: (action: WorktreeAction) => void
}

export function WorktreeReviewCard({
  review,
  loading,
  loadError,
  confirmations,
  busy,
  message,
  nextSteps,
  actionError,
  onConfirmationChange,
  onAction,
}: WorktreeReviewCardProps) {
  return (
    <Card aria-live="polite">
      <CardHeader>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <CardTitle role="heading" aria-level={2}>
              Review the isolated copy
            </CardTitle>
            <CardDescription>
              This session ran in a separate git copy. Choose what to do with its
              changes.
            </CardDescription>
          </div>
          {review ? (
            <Badge variant={review.clean ? "secondary" : "warning"}>
              {review.clean ? "No changes" : "Has changes"}
            </Badge>
          ) : null}
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {loading ? (
          <p className="text-sm text-muted-foreground">Loading the copy...</p>
        ) : null}
        {loadError ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Copy details unavailable</AlertTitle>
            <AlertDescription>{loadError}</AlertDescription>
          </Alert>
        ) : null}

        {review ? (
          <>
            <Alert role="status">
              <GitBranch />
              <AlertTitle>Branch: {review.branch}</AlertTitle>
              <AlertDescription>{review.statusSummary}</AlertDescription>
            </Alert>

            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              <Metric
                label="Changes in the copy"
                value={String(review.worktreeChanges)}
              />
              <Metric
                label="Changes in your project"
                value={String(review.sourceChanges)}
              />
              <Metric label="Clean" value={review.clean ? "Yes" : "No"} />
            </div>

            <div className="grid gap-3">
              {WORKTREE_ACTIONS.filter((definition) =>
                review.availableActions.includes(definition.action),
              ).map((definition) => (
                <WorktreeActionControl
                  key={definition.action}
                  button={definition.button}
                  confirmLabel={definition.confirmLabel}
                  description={definition.description}
                  checked={confirmations[definition.action]}
                  disabled={busy}
                  onCheckedChange={(checked) =>
                    onConfirmationChange(definition.action, checked)
                  }
                  onClick={() => onAction(definition.action)}
                />
              ))}
            </div>
          </>
        ) : null}

        {message ? (
          <Alert role="status">
            <CheckCircle2 />
            <AlertTitle>{message}</AlertTitle>
            {nextSteps.length > 0 ? (
              <AlertDescription>
                <ol className="mt-1 list-decimal pl-5">
                  {nextSteps.map((step, index) => (
                    <li key={`${index}-${step}`}>{step}</li>
                  ))}
                </ol>
              </AlertDescription>
            ) : null}
          </Alert>
        ) : null}

        {actionError ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>That action did not finish</AlertTitle>
            <AlertDescription>{actionError}</AlertDescription>
          </Alert>
        ) : null}
      </CardContent>
    </Card>
  )
}

function WorktreeActionControl({
  button,
  confirmLabel,
  description,
  checked,
  disabled,
  onCheckedChange,
  onClick,
}: {
  button: string
  confirmLabel: string
  description: string
  checked: boolean
  disabled: boolean
  onCheckedChange: (checked: boolean) => void
  onClick: () => void
}) {
  const buttonLabel = !checked
    ? `${button}: confirmation required`
    : disabled
      ? `${button}: action in progress`
      : button

  return (
    <div className="grid gap-2 rounded-lg border p-3 sm:grid-cols-[1fr_auto] sm:items-center">
      <label className="flex items-start gap-3 text-sm">
        <Checkbox
          checked={checked}
          aria-label={confirmLabel}
          onCheckedChange={(value) => onCheckedChange(value === true)}
        />
        <span className="flex flex-col gap-1">
          <span className="font-medium">{confirmLabel}</span>
          <span className="text-muted-foreground">{description}</span>
        </span>
      </label>
      <Button
        type="button"
        variant="outline"
        disabled={!checked || disabled}
        aria-label={buttonLabel}
        onClick={onClick}
      >
        {button}
      </Button>
    </div>
  )
}
