import { CheckCircle2, CircleAlert, Send } from "lucide-react"

import {
  MAX_SESSION_INPUT_BYTES,
  sessionInputBytes,
  type ManagedSessionSummary,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Label } from "@/components/ui/label"

export function SessionInputCard({
  session,
  draft,
  sending,
  error,
  acceptedMessage,
  onDraftChange,
  onSend,
}: {
  session: ManagedSessionSummary
  draft: string
  sending: boolean
  error: string
  acceptedMessage: string
  onDraftChange: (value: string) => void
  onSend: () => void
}) {
  if (session.nextAction.kind !== "monitor") {
    return null
  }

  const draftBytes = sessionInputBytes(draft)
  const inputAttached = session.input.attached
  const canSend =
    inputAttached && draft.length > 0 && draftBytes <= MAX_SESSION_INPUT_BYTES && !sending

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Message
        </CardTitle>
        <CardDescription>
          {inputAttached
            ? "Send text to this selected session."
            : "Input is not attached for this session."}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className="flex flex-col gap-4"
          onSubmit={(event) => {
            event.preventDefault()
            if (canSend) {
              onSend()
            }
          }}
        >
          <div className="flex flex-col gap-2">
            <Label htmlFor={`session-input-${session.id}`}>Message</Label>
            <textarea
              id={`session-input-${session.id}`}
              className="min-h-24 w-full min-w-0 resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-base outline-none transition-colors placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:bg-input/50 disabled:opacity-50 md:text-sm dark:bg-input/30 dark:disabled:bg-input/80"
              maxLength={MAX_SESSION_INPUT_BYTES}
              value={draft}
              disabled={!inputAttached || sending}
              onChange={(event) => onDraftChange(event.currentTarget.value)}
              onKeyDown={(event) => {
                // Cmd-Return (Ctrl-Return on non-mac keyboards) sends, matching
                // the spec keyboard map. It presses the same confirm-gated Send
                // path the button uses; nothing bypasses the native dialog.
                if (
                  event.key === "Enter" &&
                  (event.metaKey || event.ctrlKey) &&
                  canSend
                ) {
                  event.preventDefault()
                  onSend()
                }
              }}
              placeholder="Type a message for this session."
            />
            <p className="text-sm text-muted-foreground">
              {draftBytes} / {MAX_SESSION_INPUT_BYTES} bytes
            </p>
          </div>
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <Button type="submit" disabled={!canSend}>
              <Send />
              {sending ? "Sending..." : "Send"}
            </Button>
            {acceptedMessage ? (
              <span className="text-sm text-muted-foreground">
                {acceptedMessage}
              </span>
            ) : null}
          </div>
          {error ? (
            <Alert variant="destructive">
              <CircleAlert />
              <AlertTitle>Message not sent</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : null}
          {!inputAttached ? (
            <Alert>
              <CircleAlert />
              <AlertTitle>Input unavailable</AlertTitle>
              <AlertDescription>{session.input.detail}</AlertDescription>
            </Alert>
          ) : null}
          {acceptedMessage ? (
            <Alert>
              <CheckCircle2 />
              <AlertTitle>Message sent</AlertTitle>
              <AlertDescription>
                RunHaven sent the message to {session.title}.
              </AlertDescription>
            </Alert>
          ) : null}
        </form>
      </CardContent>
    </Card>
  )
}
