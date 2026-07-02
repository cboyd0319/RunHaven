import { useEffect, useRef, useState } from "react"
import { TriangleAlert } from "lucide-react"
import "@xterm/xterm/css/xterm.css"

import {
  closeTerminalSession,
  hasTauriRuntime,
  openClaudeSignInTerminal,
  openTerminalSession,
  resizeTerminal,
  writeTerminalInput,
  type ManagedSessionSummary,
  type TerminalMessage,
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

// The two panel modes share every mechanic (host PTY over a Channel, xterm,
// close-on-unmount) and differ only in the open command, the fixed session key,
// and the surrounding copy. `attach` types live inside a running session's
// sandbox; `login` runs the in-app Claude sign-in terminal (path A).
type TerminalConfig = {
  sessionId: string
  open: (
    sessionId: string,
    onMessage: (message: TerminalMessage) => void,
  ) => Promise<void>
  title: string
  description: string
  openLabel: string
  previewText: string
  connectedText: string
}

function attachConfig(session: ManagedSessionSummary): TerminalConfig {
  return {
    sessionId: session.id,
    open: openTerminalSession,
    title: "Terminal",
    description: "Type live inside this session's sandbox.",
    openLabel: "Open terminal",
    previewText:
      "Terminal needs the desktop app. Open RunHaven on your Mac to type live with this session.",
    connectedText: "Connected. What you type goes straight to the sandbox.",
  }
}

function loginConfig(sessionId: string): TerminalConfig {
  return {
    sessionId,
    open: openClaudeSignInTerminal,
    title: "Claude sign-in terminal",
    description:
      "Sign in to Claude here. RunHaven runs claude auth login in a sandbox, prints a link to open in your browser, and reads the code you paste back.",
    openLabel: "Open sign-in terminal",
    previewText:
      "The sign-in terminal needs the desktop app. Open RunHaven on your Mac to sign in to Claude here.",
    connectedText: "Connected. Open the printed link, approve, then paste the code here.",
  }
}

function TerminalDisclosure() {
  return (
    <Alert variant="warning">
      <TriangleAlert />
      <AlertTitle>Live terminal, shown but not saved</AlertTitle>
      <AlertDescription>{TERMINAL_DISCLOSURE}</AlertDescription>
    </Alert>
  )
}

export type TerminalPanelProps =
  | { session: ManagedSessionSummary; login?: undefined }
  | { login: { sessionId: string }; session?: undefined }

export function TerminalPanel(props: TerminalPanelProps) {
  const config = props.session
    ? attachConfig(props.session)
    : loginConfig(props.login.sessionId)
  const { sessionId } = config
  // A stable module-function reference per mode, so the streaming effect can
  // depend on it without re-running mid-session.
  const open = props.session ? openTerminalSession : openClaudeSignInTerminal
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
        await open(sessionId, (message) => {
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
  }, [active, sessionId, open])

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
          {config.title}
        </CardTitle>
        <CardDescription>{config.description}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <TerminalDisclosure />
        {!hasTauriRuntime() ? (
          <p className="text-sm text-muted-foreground">{config.previewText}</p>
        ) : !active ? (
          <div>
            <Button type="button" variant="outline" onClick={openTerminal}>
              {config.openLabel}
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
                {terminalStatusText(phase, config.connectedText)}
              </span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function terminalStatusText(phase: TerminalPhase, connectedText: string): string {
  switch (phase) {
    case "opening":
      return "Confirm the terminal in the dialog to start."
    case "live":
      return connectedText
    case "ended":
      return "This session ended."
    default:
      return ""
  }
}
