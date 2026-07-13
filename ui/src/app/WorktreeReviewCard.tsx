import { CheckCircle2, CircleAlert } from "lucide-react"

import type {
  WorktreeFileChange,
  WorktreeReviewResponse,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { WORKTREE_ACTIONS, type WorktreeAction } from "./worktreeReview"

export type WorktreeReviewCardProps = {
  review: WorktreeReviewResponse | null
  loading: boolean
  loadError: string
  busy: boolean
  message: string
  nextSteps: string[]
  actionError: string
  onAction: (action: WorktreeAction) => void
}

export function WorktreeReviewCard({
  review,
  loading,
  loadError,
  busy,
  message,
  nextSteps,
  actionError,
  onAction,
}: WorktreeReviewCardProps) {
  return (
    <section className="flex flex-col gap-4 border-t pt-5">
      <header className="grid gap-1">
        <h2 className="text-base font-semibold">Review changes</h2>
        <p className="text-sm text-muted-foreground">
          The assistant worked in a protected copy. Review the files, then
          choose what happens to the changes.
        </p>
      </header>

      {loading ? (
        <p role="status" aria-live="polite" className="text-sm text-muted-foreground">
          Loading changes…
        </p>
      ) : null}
      {loadError ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Changes unavailable</AlertTitle>
          <AlertDescription>{loadError}</AlertDescription>
        </Alert>
      ) : null}

      {review ? (
        review.clean ? (
          <p className="text-sm">The assistant didn’t change any files.</p>
        ) : (
          <>
            <FileChangeList
              title="Assistant changes"
              changes={review.fileChanges}
              total={review.worktreeChanges}
              truncated={review.fileChangesTruncated}
            />

            {review.sourceChanges > 0 ? (
              <>
                <Alert variant="destructive">
                  <CircleAlert />
                  <AlertTitle>Your project also changed</AlertTitle>
                  <AlertDescription>
                    Finish or undo these changes before you apply the
                    assistant&apos;s changes.
                  </AlertDescription>
                </Alert>
                <FileChangeList
                  title="Changes already in your project"
                  changes={review.sourceFileChanges}
                  total={review.sourceChanges}
                  truncated={review.sourceFileChangesTruncated}
                />
              </>
            ) : null}

            <div className="flex flex-wrap gap-2 border-t pt-4">
              {WORKTREE_ACTIONS.filter((definition) =>
                review.availableActions.includes(definition.action),
              ).map((definition) => (
                <Button
                  key={definition.action}
                  type="button"
                  variant={
                    definition.action === "discard" ? "destructive" : "default"
                  }
                  disabled={
                    busy ||
                    (definition.action === "merge" && review.sourceChanges > 0)
                  }
                  onClick={() => onAction(definition.action)}
                >
                  {definition.button}
                </Button>
              ))}
            </div>
          </>
        )
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
          <AlertTitle>Couldn’t update these changes</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>{actionError}</span>
            {review?.availableActions.includes("recover") ? (
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={busy}
                onClick={() => onAction("recover")}
              >
                Show recovery steps
              </Button>
            ) : null}
          </AlertDescription>
        </Alert>
      ) : null}
    </section>
  )
}

function FileChangeList({
  title,
  changes,
  total,
  truncated,
}: {
  title: string
  changes: WorktreeFileChange[]
  total: number
  truncated: boolean
}) {
  return (
    <section aria-label={title} className="grid gap-2">
      <h3 className="text-sm font-medium">{title}</h3>
      <ul className="divide-y border-y" aria-label={`${title} file list`}>
        {changes.map((change) => (
          <li
            key={`${change.kind}:${change.path}`}
            className="grid grid-cols-[5rem_minmax(0,1fr)] gap-3 py-2 text-sm"
          >
            <span className="font-medium">{changeKindLabel(change.kind)}</span>
            <span className="break-words text-muted-foreground [overflow-wrap:anywhere]">
              {change.path}
            </span>
          </li>
        ))}
      </ul>
      {truncated ? (
        <p className="text-xs text-muted-foreground">
          Showing {changes.length} of {total} file changes.
        </p>
      ) : null}
    </section>
  )
}

function changeKindLabel(kind: WorktreeFileChange["kind"]): string {
  return `${kind.charAt(0).toUpperCase()}${kind.slice(1)}`
}
