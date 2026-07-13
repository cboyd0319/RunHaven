import { useEffect, useMemo, useRef, useState } from "react"
import { TriangleAlert } from "lucide-react"
import "@xterm/xterm/css/xterm.css"

import {
  closeTerminalSession,
  hasTauriRuntime,
  openAgentRunTerminal,
  openAgentSignInTerminal,
  openTerminalSession,
  resizeTerminal,
  writeTerminalInput,
  type LaunchRunRequest,
  type ManagedSessionSummary,
  type TerminalMessage,
} from "@/commands/runhaven"

import { errorMessage } from "./types"
import { cn } from "@/lib/utils"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  attachTerminalConfig,
  loginTerminalConfig,
  runTerminalConfig,
  TerminalDisclosure,
  terminalStatusText,
} from "./terminalPresentation"

// Bounded in-WebView scrollback (design F5). The raw stream lives only here, in
// memory, for the life of the view; it is never written to disk or logged.
const SCROLLBACK_LINES = 1000

type TerminalPhase = "idle" | "opening" | "live" | "ended" | "error"

type TerminalInteractionError = {
  title: string
  detail: string
}

// The three panel modes share every mechanic (host PTY over a Channel, xterm,
// close-on-unmount) and differ only in the open command, the fixed session key,
// and the surrounding copy. `attach` types live inside a running session's
// sandbox; `login` runs a supported agent's native sign-in terminal; `run`
// launches an interactive agent and lets the user type their task in its TUI
// (task #23, Option A).
type TerminalOpen = (
  sessionId: string,
  onMessage: (message: TerminalMessage) => void,
) => Promise<boolean>

export type TerminalPanelProps =
  | { session: ManagedSessionSummary; login?: undefined; run?: undefined }
  | {
      login: { agent: string; agentLabel: string; sessionId: string; onSignedIn?: () => void }
      session?: undefined
      run?: undefined
    }
  | {
      run: { sessionId: string; request: LaunchRunRequest }
      session?: undefined
      login?: undefined
      // The run terminal in its main-content destination. `fill` makes the card
      // and the xterm container flex-fill a bounded parent (no min-height growth
      // loop); `onClose` ends the run and leaves the destination. `visible` is
      // false while the run is live but its destination is not shown (the panel
      // stays mounted to keep the PTY alive); it drives a re-fit when the
      // terminal comes back into view.
      fill?: boolean
      onClose?: () => void
      onOpenCancelled?: () => void
      visible?: boolean
    }

export function TerminalPanel(props: TerminalPanelProps) {
  const config = props.session
    ? attachTerminalConfig(props.session)
    : props.login
      ? loginTerminalConfig(props.login.agentLabel, props.login.sessionId)
      : runTerminalConfig(props.run.sessionId)
  const { sessionId } = config
  const mode: "attach" | "login" | "run" = props.session
    ? "attach"
    : props.login
      ? "login"
      : "run"
  // fill/onClose only exist on the run variant; attach and login keep their
  // bounded-card layout and their own idle/close behavior untouched.
  const runProps = props.run ? props : null
  const fill = runProps?.fill ?? false
  const onClose = runProps?.onClose
  const onOpenCancelled = runProps?.onOpenCancelled
  const visible = runProps?.visible ?? true
  const onOpenCancelledRef = useRef(onOpenCancelled)
  useEffect(() => {
    onOpenCancelledRef.current = onOpenCancelled
  }, [onOpenCancelled])
  // A stable open per mode, so the streaming effect can depend on it without
  // re-running mid-session. `run` binds the captured launch request (fixed for
  // this terminal's life) into the uniform (sessionId, onMessage) open shape, so
  // the plan the backend launches is the reviewed request, not a WebView value.
  const runRequest = props.run?.request
  const loginAgent = props.login?.agent
  const open = useMemo<TerminalOpen>(() => {
    switch (mode) {
      case "attach":
        return openTerminalSession
      case "login":
        return (sessionId, onMessage) =>
          openAgentSignInTerminal(loginAgent as string, sessionId, onMessage)
      case "run": {
        const request = runRequest as LaunchRunRequest
        return (sessionId, onMessage) =>
          openAgentRunTerminal(sessionId, request, onMessage)
      }
    }
  }, [loginAgent, mode, runRequest])
  const onSignedIn = props.login?.onSignedIn
  const onSignedInRef = useRef(onSignedIn)
  useEffect(() => {
    onSignedInRef.current = onSignedIn
  }, [onSignedIn])
  const containerRef = useRef<HTMLDivElement | null>(null)
  const fitRef = useRef<import("@xterm/addon-fit").FitAddon | null>(null)
  // The primary control the keyboard escape lands on. xterm captures Tab and
  // most keys, so without this a keyboard-only user who focuses the terminal
  // cannot Tab back out (WCAG 2.1.2, audit A11Y-1). The key handler moves DOM
  // focus here so there is always a keyboard way out.
  const primaryControlRef = useRef<HTMLButtonElement | null>(null)
  // "End run" on the run destination, "Close terminal" for attach/login. Names
  // both the visible button and the discoverable escape hint from one source.
  const primaryControlLabel = onClose ? "End session" : "Close terminal"
  const autoStart = mode === "run" && hasTauriRuntime()
  const [active, setActive] = useState(autoStart)
  const [phase, setPhase] = useState<TerminalPhase>(autoStart ? "opening" : "idle")
  const [errorText, setErrorText] = useState("")
  const [interactionError, setInteractionError] =
    useState<TerminalInteractionError | null>(null)
  // Kill the backend child when the user navigates away or reloads. The unmount
  // path (the effect below) covers navigation; these listeners cover window
  // reload/close, which does not unmount React first. Both call the same close.
  useEffect(() => {
    if (!active || !hasTauriRuntime()) {
      return
    }
    const teardown = () => {
      void closeTerminalSession(sessionId).catch(() => undefined)
    }
    window.addEventListener("beforeunload", teardown)
    window.addEventListener("pagehide", teardown)
    return () => {
      window.removeEventListener("beforeunload", teardown)
      window.removeEventListener("pagehide", teardown)
    }
  }, [active, sessionId])

  useEffect(() => {
    if (!active || !hasTauriRuntime()) {
      return
    }
    const container = containerRef.current
    if (!container) {
      return
    }
    let disposed = false
    let term: import("@xterm/xterm").Terminal | undefined
    let observer: ResizeObserver | undefined

    function reportInteractionError(title: string, cause: unknown) {
      if (!disposed) {
        setInteractionError({ title, detail: errorMessage(cause) })
      }
    }

    async function start() {
      const [{ Terminal }, { FitAddon }] = await Promise.all([
        import("@xterm/xterm"),
        import("@xterm/addon-fit"),
      ])
      if (disposed || !container) {
        return
      }
      term = new Terminal({
        scrollback: SCROLLBACK_LINES,
        screenReaderMode: true,
        cursorBlink: false,
      })
      const fitAddon = new FitAddon()
      fitRef.current = fitAddon
      term.loadAddon(fitAddon)
      term.open(container)
      // Keyboard escape from the terminal trap (WCAG 2.1.2). Shift+Esc blurs the
      // terminal and moves focus to the primary control, so a keyboard-only user
      // is never stuck behind xterm's key capture. A bare Esc and a double-Esc
      // are both meaningful to guest TUIs (Claude Code uses both), so the chord
      // adds Shift to keep the "escape" mnemonic without swallowing them. The
      // chord is kept out of the PTY (returns false); every other key still
      // reaches the terminal (returns true).
      term.attachCustomKeyEventHandler((event) => {
        if (event.key === "Escape" && event.shiftKey) {
          if (event.type === "keydown") {
            term?.blur()
            primaryControlRef.current?.focus()
          }
          return false
        }
        // Mac clipboard on the xterm canvas (audit NAT-8): the native Edit menu
        // targets NSText, not this canvas selection. Only Cmd(meta) is caught, so
        // Ctrl+C still interrupts the guest; a no-selection Cmd-C passes through.
        if (event.metaKey && event.key === "c") {
          const selection = term?.getSelection() ?? ""
          if (selection && event.type === "keydown") {
            void navigator.clipboard
              .writeText(selection)
              .catch((cause) => reportInteractionError("Copy failed", cause))
          }
          return !selection
        }
        if (event.metaKey && event.key === "v") {
          if (event.type === "keydown") {
            void navigator.clipboard
              .readText()
              .then((text) => {
                const bytes = [...new TextEncoder().encode(text)]
                if (bytes.length > 0) {
                  void writeTerminalInput(sessionId, bytes).catch((cause) =>
                    reportInteractionError("Terminal input failed", cause),
                  )
                }
              })
              .catch((cause) => reportInteractionError("Paste failed", cause))
          }
          return false
        }
        return true
      })
      fitAddon.fit()
      term.onData((data) => {
        void writeTerminalInput(sessionId, [...new TextEncoder().encode(data)]).catch(
          (cause) => reportInteractionError("Terminal input failed", cause),
        )
      })
      term.onResize(({ cols, rows }) => {
        void resizeTerminal(sessionId, cols, rows).catch(() => undefined)
      })
      if (typeof ResizeObserver !== "undefined") {
        observer = new ResizeObserver(() => {
          // Skip while hidden (0-size): fitting a display:none container would
          // push a 0x0 resize to the guest. The visibility effect re-fits on
          // re-show.
          if (container.clientWidth > 0 && container.clientHeight > 0) {
            fitAddon.fit()
          }
        })
        observer.observe(container)
      }
      try {
        const opened = await open(sessionId, (message) => {
          if (disposed || !term) {
            return
          }
          if (message.event === "output") {
            term.write(new Uint8Array(message.data))
          } else {
            term.writeln("")
            term.writeln(config.endedText)
            term.options.disableStdin = true
            setPhase("ended")
            if (mode === "login" && message.code === 0) {
              if (message.completionRecorded === false) {
                reportInteractionError(
                  "Sign-in status was not saved",
                  "The assistant may still be signed in, but RunHaven couldn’t update its status. Close this terminal, make sure your Mac has free space, then try again.",
                )
              } else {
                onSignedInRef.current?.()
              }
            }
          }
        })
        if (disposed || !term) {
          return
        }
        if (!opened) {
          setErrorText("")
          setPhase("idle")
          setActive(false)
          if (mode === "run") {
            onOpenCancelledRef.current?.()
          }
          return
        }
        // Make sure the guest starts at the current viewport size.
        void resizeTerminal(sessionId, term.cols, term.rows).catch(() => undefined)
        term.focus()
        setPhase("live")
      } catch (cause) {
        if (disposed) {
          return
        }
        setErrorText(errorMessage(cause))
        setPhase("error")
      }
    }

    void start()
    return () => {
      disposed = true
      observer?.disconnect()
      fitRef.current = null
      // Teardown contract: killing the backend child on unmount is required,
      // not optional. The backend send-error kill is only a backstop. The run
      // terminal reaches here only when the panel truly unmounts (End run
      // clears it); navigating away keeps it mounted, so the run is not killed.
      void closeTerminalSession(sessionId).catch(() => undefined)
      term?.dispose()
    }
  }, [active, config.endedText, mode, open, sessionId])

  // Re-fit when the run terminal returns to view. While its destination is not
  // shown the panel stays mounted but hidden (0-size), so the ResizeObserver is
  // suppressed; on re-show the container is laid out again and xterm must match
  // the real size and resize the guest. rAF waits for layout before measuring.
  useEffect(() => {
    if (!visible || !active) {
      return
    }
    const frame = requestAnimationFrame(() => fitRef.current?.fit())
    return () => cancelAnimationFrame(frame)
  }, [visible, active])

  function openTerminal() {
    setErrorText("")
    setInteractionError(null)
    setPhase("opening")
    setActive(true)
  }

  function closeTerminal() {
    // Run-terminal destination: closing ends the run and leaves the view; the
    // parent navigates Home, unmounting this panel so close-on-unmount kills the
    // child. Attach/login keep returning to their idle "open" button.
    if (onClose) {
      onClose()
      return
    }
    setActive(false)
    setPhase("idle")
    setErrorText("")
    setInteractionError(null)
  }

  return (
    <Card className={cn(fill && "min-h-0 flex-1")}>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {config.title}
        </CardTitle>
        <CardDescription>{config.description}</CardDescription>
      </CardHeader>
      <CardContent className={cn("flex flex-col gap-4", fill && "min-h-0 flex-1")}>
        <TerminalDisclosure retainForHistory={mode === "run"} />
        {!hasTauriRuntime() ? (
          <div className="flex flex-col gap-3">
            <p className="text-sm text-muted-foreground">{config.previewText}</p>
            {onClose ? (
              // Even in browser preview (nothing to invoke) the run-terminal
              // destination stays leaveable, so a preview user is never stranded.
              <div>
                <Button type="button" variant="ghost" onClick={onClose}>
                  Close
                </Button>
              </div>
            ) : null}
          </div>
        ) : !active ? (
          <div className="flex flex-wrap items-center gap-3">
            <Button type="button" variant="outline" onClick={openTerminal}>
              {config.openLabel}
            </Button>
            {onClose ? (
              // Always reachable: a way out before the run is even started, so
              // the user is never stranded on the destination.
              <Button type="button" variant="ghost" onClick={onClose}>
                Close
              </Button>
            ) : null}
          </div>
        ) : (
          <div className={cn("flex flex-col gap-3", fill && "min-h-0 flex-1")}>
            <div
              ref={containerRef}
              role="group"
              aria-label="Session terminal"
              className={cn(
                "overflow-hidden rounded-lg border bg-black p-2",
                fill ? "min-h-0 flex-1" : "h-72",
              )}
            />
            {phase === "error" ? (
              <Alert variant="destructive">
                <TriangleAlert />
                <AlertTitle>Terminal did not open</AlertTitle>
                <AlertDescription>{errorText}</AlertDescription>
              </Alert>
            ) : null}
            {interactionError ? (
              <Alert variant="destructive" role="alert">
                <TriangleAlert />
                <AlertTitle>{interactionError.title}</AlertTitle>
                <AlertDescription>{interactionError.detail}</AlertDescription>
              </Alert>
            ) : null}
            <div className="flex flex-wrap items-center gap-3">
              <Button
                ref={primaryControlRef}
                type="button"
                variant="outline"
                onClick={closeTerminal}
              >
                {primaryControlLabel}
              </Button>
              <span className="text-sm text-muted-foreground" aria-live="polite">
                {terminalStatusText(
                  phase,
                  config.connectedText,
                  config.endedText,
                  primaryControlLabel,
                )}
              </span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
