import type { Terminal } from "@xterm/xterm"

import { writeTerminalInput } from "@/commands/runhaven"

type ReportError = (title: string, cause: unknown) => void

export function installTerminalKeyHandler(
  terminal: Terminal,
  sessionId: string,
  leaveTerminal: () => void,
  reportError: ReportError,
) {
  terminal.attachCustomKeyEventHandler((event) => {
    // Shift-Escape leaves xterm without swallowing Escape from guest apps.
    if (event.key === "Escape" && event.shiftKey) {
      if (event.type === "keydown") {
        leaveTerminal()
      }
      return false
    }
    // The native Edit menu targets NSText, not the xterm canvas. Keep Ctrl-C
    // available to the guest and pass Cmd-C through when nothing is selected.
    if (event.metaKey && event.key === "c") {
      const selection = terminal.getSelection()
      if (selection && event.type === "keydown") {
        void navigator.clipboard
          .writeText(selection)
          .catch((cause) => reportError("Copy failed", cause))
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
                reportError("Terminal input failed", cause),
              )
            }
          })
          .catch((cause) => reportError("Paste failed", cause))
      }
      return false
    }
    return true
  })
}
