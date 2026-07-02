import { Channel } from "@tauri-apps/api/core";

import { call, hasTauriRuntime } from "./client";
import type { TerminalMessage } from "./types";

// The WebView terminal command surface. These wrappers route through the shared
// `call` helper in client.ts, so `invoke` stays centralized and the browser
// preview no-ops. The raw PTY stream they carry is rendered in xterm's in-memory
// scrollback only; nothing here writes it to disk or logs it.

// Open a live terminal for a running session. Rust resolves the target from
// sessionId (the WebView never names a container), shows the native no-filter/
// no-store confirm dialog, then spawns a host-PTY-wrapped `container exec` and
// streams raw output over the Channel. onMessage receives each TerminalMessage.
// The Channel is constructed only under the desktop runtime because its callback
// registration needs Tauri internals; the browser preview returns without it.
export async function openTerminalSession(
  sessionId: string,
  onMessage: (message: TerminalMessage) => void
): Promise<void> {
  if (!hasTauriRuntime()) {
    return;
  }
  const onEvent = new Channel<TerminalMessage>(onMessage);
  return call("open_terminal_session", { onEvent, sessionId }, () => undefined);
}

// Open a live Claude sign-in terminal (terminal path A). Rust runs the in-sandbox
// `claude auth login` through a host PTY behind its own native no-filter/no-store
// confirm, so the printed OAuth link and the pasted-back code round-trip in-app.
// The target is fixed to Claude in Rust; sessionId is only the registry key, so
// the WebView cannot steer what runs. Streaming, input, resize, and close reuse
// the same commands as an attach. The browser preview returns without invoking.
export async function openClaudeSignInTerminal(
  sessionId: string,
  onMessage: (message: TerminalMessage) => void
): Promise<void> {
  if (!hasTauriRuntime()) {
    return;
  }
  const onEvent = new Channel<TerminalMessage>(onMessage);
  return call(
    "open_claude_sign_in_terminal",
    { onEvent, sessionId },
    () => undefined
  );
}

export async function writeTerminalInput(
  sessionId: string,
  data: number[]
): Promise<void> {
  return call("write_terminal_input", { sessionId, data }, () => undefined);
}

export async function resizeTerminal(
  sessionId: string,
  cols: number,
  rows: number
): Promise<void> {
  return call("resize_terminal", { sessionId, cols, rows }, () => undefined);
}

// Tear down the host PTY and container child for this terminal. Called on view
// unmount and on window unload so navigating away or reloading kills the child;
// the backend send-error kill is only a backstop.
export async function closeTerminalSession(sessionId: string): Promise<void> {
  return call("close_terminal_session", { sessionId }, () => undefined);
}
