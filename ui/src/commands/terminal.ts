import { Channel } from "@tauri-apps/api/core";

import { call, hasTauriRuntime } from "./client";
import type { LaunchRunRequest, TerminalMessage } from "./types";

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
): Promise<boolean> {
  if (!hasTauriRuntime()) {
    return false;
  }
  const onEvent = new Channel<TerminalMessage>(onMessage);
  return call("open_terminal_session", { onEvent, sessionId }, () => false);
}

// Open one supported agent's native sign-in command through a host PTY behind a
// Rust-owned no-filter/no-store confirmation. Rust validates the exact product
// agent set and builds the fixed command; sessionId is only the registry key.
// Streaming, input, resize, and close reuse the same commands as an attach.
export async function openAgentSignInTerminal(
  agent: string,
  sessionId: string,
  onMessage: (message: TerminalMessage) => void
): Promise<boolean> {
  if (!hasTauriRuntime()) {
    return false;
  }
  const onEvent = new Channel<TerminalMessage>(onMessage);
  return call(
    "open_agent_sign_in_terminal",
    { agent, onEvent, sessionId },
    () => false
  );
}

// Open a live interactive agent-run terminal (task #23, Option A). An
// interactive agent (Claude Code today) is a TUI: with no terminal it drops to
// headless --print and errors without a piped prompt, and a RunHaven run carries
// no prompt by design. Rust reuses the normal launch machinery end to end: it
// builds the plan from `request`, shows the native launch confirm (on cancel
// nothing runs), then runs
// the finalized plan under a host PTY streamed over the Channel so the user types
// their prompt in the agent's TUI. Same egress proxy, non-root user, read-only
// rootfs, cap-drop, and workspace mount as a background run; only the execution
// surface (a host PTY, never stored or logged) differs. `sessionId` is only the
// registry key / run label; the plan content comes from `request` and Rust-side
// selection. The browser preview returns without invoking.
export async function openAgentRunTerminal(
  sessionId: string,
  request: LaunchRunRequest,
  onMessage: (message: TerminalMessage) => void
): Promise<boolean> {
  if (!hasTauriRuntime()) {
    return false;
  }
  const onEvent = new Channel<TerminalMessage>(onMessage);
  return call(
    "open_agent_run_terminal",
    { onEvent, sessionId, request },
    () => false
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
