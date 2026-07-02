import { useEffect, useRef, useState } from "react"
import { TriangleAlert } from "lucide-react"
import "@xterm/xterm/css/xterm.css"

import {
  closeTerminalSession,
  hasTauriRuntime,
  openTerminalSession,
  resizeTerminal,
  writeTerminalInput,
  type ManagedSessionSummary,
} from "@/commands/runhaven"

import { errorMessage } from "./types"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

// The exact scoped disclosure from the terminal design ("what RunHaven can and
// cannot see"). It is a standing warning, always visible before and during the
// terminal, and applies only to the live terminal stream, not the app.
const TERMINAL_DISCLOSURE =
  "This is a live terminal inside the sandbox. RunHaven does not filter, read, " +
  "store, or log it. It just shows what the sandbox prints. So anything here, " +
  "including passwords or codes, is visible on this screen."

// Bounded in-WebView scrollback (design F5). The raw stream lives only here, in
// memory, for the life of the view; it is never written to disk or logged.
const SCROLLBACK_LINES = 1000

type TerminalPhase = "idle" | "opening" | "live" | "ended" | "error"

function TerminalDisclosure() {
  return (
    <Alert variant="warning">
      <TriangleAlert />
      <AlertTitle>Live terminal, shown but not saved</AlertTitle>
      <AlertDescription>{TERMINAL_DISCLOSURE}</AlertDescription>
    </Alert>
  )
}

export function TerminalPanel({
  session,
}: {
  session: ManagedSessionSummary
}) {
  const sessionId = session.id
  const containerRef = useRef<HTMLDivElement | null>(null)
  const [active, setActive] = useState(false)
  const [phase, setPhase] = useState<TerminalPhase>("idle")
  const [errorText, setErrorText] = useState("")

  // Kill the backend child when the user navigates away or reloads. The unmount
  // path (the effect below) covers navigation; these listeners cover window
  // reload/close, which does not unmount React first. Both call the same close.
  useEffect(() => {
    if (!active || !hasTauriRuntime()) {
      return
    }
    const teardown = () => {
      void closeTerminalSession(sessionId)
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
      term.loadAddon(fitAddon)
      term.open(container)
      fitAddon.fit()
      term.onData((data) => {
        void writeTerminalInput(sessionId, [...new TextEncoder().encode(data)])
      })
      term.onResize(({ cols, rows }) => {
        void resizeTerminal(sessionId, cols, rows)
      })
      if (typeof ResizeObserver !== "undefined") {
        observer = new ResizeObserver(() => fitAddon.fit())
        observer.observe(container)
      }
      try {
        await openTerminalSession(sessionId, (message) => {
          if (disposed || !term) {
            return
          }
          if (message.event === "output") {
            term.write(new Uint8Array(message.data))
          } else {
            term.writeln("")
            term.writeln("This session ended")
            term.options.disableStdin = true
            setPhase("ended")
          }
        })
        if (disposed || !term) {
          return
        }
        // Make sure the guest starts at the current viewport size.
        void resizeTerminal(sessionId, term.cols, term.rows)
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
      // Teardown contract: killing the backend child on unmount is required,
      // not optional. The backend send-error kill is only a backstop.
      void closeTerminalSession(sessionId)
      term?.dispose()
    }
  }, [active, sessionId])

  function openTerminal() {
    setErrorText("")
    setPhase("opening")
    setActive(true)
  }

  function closeTerminal() {
    setActive(false)
    setPhase("idle")
    setErrorText("")
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Terminal
        </CardTitle>
        <CardDescription>
          Type live inside this session's sandbox.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <TerminalDisclosure />
        {!hasTauriRuntime() ? (
          <p className="text-sm text-muted-foreground">
            Terminal needs the desktop app. Open RunHaven on your Mac to type
            live with this session.
          </p>
        ) : !active ? (
          <div>
            <Button type="button" variant="outline" onClick={openTerminal}>
              Open terminal
            </Button>
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            <div
              ref={containerRef}
              role="group"
              aria-label="Sandbox terminal"
              className="min-h-72 overflow-hidden rounded-lg border bg-black p-2"
            />
            {phase === "error" ? (
              <Alert variant="destructive">
                <TriangleAlert />
                <AlertTitle>Terminal did not open</AlertTitle>
                <AlertDescription>{errorText}</AlertDescription>
              </Alert>
            ) : null}
            <div className="flex flex-wrap items-center gap-3">
              <Button type="button" variant="outline" onClick={closeTerminal}>
                Close terminal
              </Button>
              <span className="text-sm text-muted-foreground" aria-live="polite">
                {terminalStatusText(phase)}
              </span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function terminalStatusText(phase: TerminalPhase): string {
  switch (phase) {
    case "opening":
      return "Confirm the terminal in the dialog to start."
    case "live":
      return "Connected. What you type goes straight to the sandbox."
    case "ended":
      return "This session ended."
    default:
      return ""
  }
}
