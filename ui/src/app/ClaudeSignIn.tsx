import { useState, type ComponentProps } from "react"
import {
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  CircleAlert,
  Copy,
  LogIn,
  Trash2,
  TriangleAlert,
} from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

import { TerminalPanel } from "./TerminalPanel"
import { loadSettings } from "./uiStatePersistence"
import { useClaudeOAuthSignIn } from "./useClaudeOAuthSignIn"

// Approved user-facing strings from the design (Section 5). These are the ONLY
// wording allowed for these controls: each was checked against the mechanism, so
// they must ship verbatim and must not be paraphrased or extended with new
// security claims. Tests assert them character for character.
export const PATH_C_TOOLTIP =
  "Sign in on your Mac. RunHaven saves your Claude token to your Mac's Keychain and uses it to sign your runs. The token never goes inside the sandbox."
export const PATH_A_TOOLTIP =
  "Sign in inside the sandbox instead. Your Claude login is saved in the agent's own space, not on your Mac's Keychain."
export const CLEAR_TOOLTIP =
  "Remove the Claude token RunHaven saved. This deletes only RunHaven's Keychain item, not your other logins."

const CLAUDE_LOGIN_COMMAND = "runhaven login claude"

// Claude sign-in control. Path C (sign in on your Mac, the secure default) is the
// primary action: RunHaven saves the token to the Mac's Keychain and the broker
// adds it only to your runs' requests to Anthropic; the token never enters the
// sandbox. Path A (sign in inside the sandbox) stays available as an explicit
// opt-in and runs the existing in-app terminal login unchanged. The WebView
// never receives the token in either path.
export function ClaudeSignIn({
  onSignedIn,
  reauthDays,
}: {
  onSignedIn?: () => void
  reauthDays?: number
}) {
  const cadence = reauthDays ?? loadSettings().claudeReauthDays
  const {
    status,
    busy,
    error,
    statusError,
    signing,
    polling,
    stalled,
    canceled,
    cancel,
    signIn,
    clearing,
    clearError,
    clearMessage,
    clear,
  } = useClaudeOAuthSignIn(onSignedIn)
  const state = status?.status ?? "not_signed_in"
  const ageDays = status?.ageDays ?? null
  const tokenIsOld =
    state === "signed_in" && ageDays !== null && ageDays >= cadence

  return (
    <TooltipProvider>
      <section
        className="flex flex-col gap-3 rounded-lg border p-3"
        aria-live="polite"
      >
        <div className="flex items-center gap-2 text-sm font-medium">
          <LogIn className="size-4" aria-hidden />
          Sign in to Claude
        </div>

        {state === "signed_in" ? (
          <SignedInState
            ageDays={ageDays}
            tokenIsOld={tokenIsOld}
            busy={busy}
            onReauth={() => void signIn()}
            clearing={clearing}
            onClear={() => void clear()}
            clearMessage={clearMessage}
            clearError={clearError}
          />
        ) : signing ? (
          <SigningState stalled={stalled} onCancel={cancel} />
        ) : canceled && polling ? (
          // The user stopped waiting, but the host capture is still running. Do
          // not offer a restart: the backend refuses a concurrent capture, so a
          // second "Sign in" would just fail. Say plainly that it may still
          // finish; the poll flips this to signed in on its own if it does.
          <p className="text-sm text-muted-foreground">
            You stopped waiting. The sign-in may still finish on its own.
            You&apos;ll see it here if it does.
          </p>
        ) : (
          <div className="flex flex-col gap-2">
            <p className="text-sm text-muted-foreground">
              RunHaven saves your Claude token to your Mac&apos;s Keychain and
              uses it to sign your runs. The token never goes inside the sandbox.
            </p>
            {canceled ? (
              <p className="text-sm text-muted-foreground">
                You stopped waiting. You can start over below.
              </p>
            ) : null}
            <div>
              <TipButton tip={PATH_C_TOOLTIP} disabled={busy} onClick={() => void signIn()}>
                <LogIn />
                {busy ? "Starting..." : "Sign in on your Mac"}
              </TipButton>
            </div>
          </div>
        )}

        {error ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Sign-in did not start</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
        {statusError ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Could not check sign-in status</AlertTitle>
            <AlertDescription>{statusError}</AlertDescription>
          </Alert>
        ) : null}
        {state === "failed" && status?.reason ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Sign-in did not finish</AlertTitle>
            <AlertDescription>{status.reason}</AlertDescription>
          </Alert>
        ) : null}

        <SandboxOptIn />
      </section>
    </TooltipProvider>
  )
}

// The in-flight view while the host capture runs. It always offers a way to stop
// waiting, and after a reasonable wait adds a calm hint that starting over is
// fine. Stopping is client-side: it frees the UI even if the host capture is
// still open, and a real cancel command is a backend follow-up.
function SigningState({
  stalled,
  onCancel,
}: {
  stalled: boolean
  onCancel: () => void
}) {
  return (
    <div className="flex flex-col gap-2">
      <p className="text-sm text-muted-foreground">
        Signing in on your Mac. Finish in your browser, then approve saving the
        token. Keep RunHaven open until it finishes.
      </p>
      {stalled ? (
        <p className="text-sm text-muted-foreground">
          Still waiting. If the browser step did not open or you closed it, you
          can stop and start over.
        </p>
      ) : null}
      <div>
        <Button type="button" variant="outline" onClick={onCancel}>
          Stop waiting
        </Button>
      </div>
    </div>
  )
}

// The signed-in view: token age, the re-sign-in reminder once the token is past
// the cadence (a prompt, never a forced expiry), and the confirm-gated remove
// action. The age is a non-secret day count; no token is ever shown.
function SignedInState({
  ageDays,
  tokenIsOld,
  busy,
  onReauth,
  clearing,
  onClear,
  clearMessage,
  clearError,
}: {
  ageDays: number | null
  tokenIsOld: boolean
  busy: boolean
  onReauth: () => void
  clearing: boolean
  onClear: () => void
  clearMessage: string
  clearError: string
}) {
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-2 text-sm">
        <CheckCircle2 className="size-4 text-success" aria-hidden />
        <span>Signed in on your Mac. {savedAgeLabel(ageDays)}</span>
      </div>

      {tokenIsOld ? (
        <Alert>
          <TriangleAlert />
          <AlertTitle>Time to sign in again</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>
              Your saved Claude token is old. Signing in again keeps it fresh.
              This is only a reminder, not a cutoff.
            </span>
            <TipButton
              tip={PATH_C_TOOLTIP}
              disabled={busy}
              onClick={onReauth}
            >
              <LogIn />
              {busy ? "Starting..." : "Sign in again"}
            </TipButton>
          </AlertDescription>
        </Alert>
      ) : null}

      <div>
        <TipButton
          tip={CLEAR_TOOLTIP}
          variant="outline"
          disabled={clearing}
          onClick={onClear}
        >
          <Trash2 />
          {clearing ? "Removing..." : "Remove saved token"}
        </TipButton>
      </div>
      {clearMessage ? (
        <p className="text-sm text-muted-foreground">{clearMessage}</p>
      ) : null}
      {clearError ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Could not remove the token</AlertTitle>
          <AlertDescription>{clearError}</AlertDescription>
        </Alert>
      ) : null}
    </div>
  )
}

// Path A opt-in: a secondary disclosure that reveals the existing in-sandbox
// terminal login unchanged. Collapsed by default so path C stays the primary,
// obvious action.
function SandboxOptIn() {
  const [open, setOpen] = useState(false)
  return (
    <div className="flex flex-col gap-2 border-t pt-3">
      <div>
        <TipButton
          tip={PATH_A_TOOLTIP}
          variant="ghost"
          size="sm"
          aria-expanded={open}
          onClick={() => setOpen((value) => !value)}
        >
          {open ? <ChevronDown /> : <ChevronRight />}
          Sign in inside the sandbox instead
        </TipButton>
      </div>
      {open ? <SandboxTerminalSignIn /> : null}
    </div>
  )
}

// The in-app Claude sign-in terminal (path A), unchanged from before: an in-app
// terminal running the in-sandbox `claude auth login`, where the printed link
// and pasted-back code round-trip in the app, with running it in your own
// terminal kept as a copy fallback. Only invokes under the desktop runtime; the
// browser preview shows the panel's own placeholder and never signs in.
function SandboxTerminalSignIn() {
  const [loginId] = useState(() =>
    (globalThis.crypto?.randomUUID?.() ?? `claude-login-${Date.now()}`).replaceAll(
      "-",
      "",
    ),
  )
  return (
    <div className="flex flex-col gap-3">
      <TerminalPanel login={{ sessionId: loginId }} />
      <Alert>
        <LogIn />
        <AlertTitle>Prefer your own terminal?</AlertTitle>
        <AlertDescription className="flex flex-col items-start gap-2">
          <span>
            You can also sign in from your own terminal. Run{" "}
            <code className="font-mono">{CLAUDE_LOGIN_COMMAND}</code>, then come
            back and refresh.
          </span>
          <CopyCommandButton command={CLAUDE_LOGIN_COMMAND} />
        </AlertDescription>
      </Alert>
    </div>
  )
}

// Copies the terminal sign-in command with the browser clipboard API. The button
// label doubles as the copied confirmation; if the WebView refuses clipboard
// access, a short note tells the user to copy the text by hand.
function CopyCommandButton({ command }: { command: string }) {
  const [copyState, setCopyState] = useState<"idle" | "copied" | "failed">(
    "idle",
  )

  async function copy() {
    try {
      await navigator.clipboard.writeText(command)
      setCopyState("copied")
    } catch {
      setCopyState("failed")
    }
  }

  return (
    <span className="flex flex-wrap items-center gap-2" aria-live="polite">
      <Button type="button" variant="outline" size="sm" onClick={() => void copy()}>
        <Copy aria-hidden />
        {copyState === "copied" ? "Copied" : "Copy command"}
      </Button>
      {copyState === "failed" ? (
        <span className="text-sm">
          Copy did not work here. Select the command text and copy it yourself.
        </span>
      ) : null}
    </span>
  )
}

// A button that carries one of the approved verbatim tooltips. The visible label
// is always clear on its own; the tooltip adds the plain-language detail.
function TipButton({
  tip,
  children,
  ...props
}: ComponentProps<typeof Button> & { tip: string }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button type="button" {...props}>
          {children}
        </Button>
      </TooltipTrigger>
      <TooltipContent>{tip}</TooltipContent>
    </Tooltip>
  )
}

function savedAgeLabel(ageDays: number | null): string {
  if (ageDays === null) {
    return "RunHaven is signing your runs with it."
  }
  if (ageDays <= 0) {
    return "Saved today."
  }
  return `Saved ${ageDays} day${ageDays === 1 ? "" : "s"} ago.`
}
