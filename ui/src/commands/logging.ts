import { debug, error, info, trace, warn } from "@tauri-apps/plugin-log";

// Development-only diagnostic log trail for the WebView. The release WebView
// has no logging capability because it also owns the interactive terminal.
// Everything here is write-only telemetry into the local app log file (main
// development window only, `logging` capability): the original console behavior is
// preserved, forwarding failures are swallowed so logging can never break or
// loop the app, and no invoke payload, secret, or terminal PTY byte is ever
// sent. The no-console source guard (ui/src/test/no-console-guard.test.ts)
// keeps app code from console-logging objects that could carry sensitive
// values into this file. In the browser preview (no Tauri runtime) nothing is
// installed and the console stays native.

export function hasTauriRuntime(): boolean {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

// Bound every forwarded message so a runaway console.log of a huge object
// cannot bloat the log file or the IPC channel.
const MAX_MESSAGE_LENGTH = 8192;

function bounded(message: string): string {
  if (message.length <= MAX_MESSAGE_LENGTH) {
    return message;
  }
  return `${message.slice(0, MAX_MESSAGE_LENGTH)} [truncated ${message.length - MAX_MESSAGE_LENGTH} chars]`;
}

function formatValue(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  if (value instanceof Error) {
    return `${value.name}: ${value.message}\n${value.stack ?? ""}`;
  }
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

// Fire-and-forget: a denied or failed forward must never surface as a new
// console error (that would loop back through the wrapper) or break callers.
// The latch also blocks the synchronous path: if a sink ever emitted console
// output while dispatching (e.g. under a denied capability), the wrapped
// console would re-enter forward(), which is then a no-op instead of a loop.
let forwarding = false;
function forward(sink: (message: string) => Promise<void>, message: string): void {
  if (forwarding) {
    return;
  }
  forwarding = true;
  try {
    void sink(bounded(message)).catch(() => undefined);
  } finally {
    forwarding = false;
  }
}

// Commands whose calls are never traced. A timestamped log entry per terminal
// keystroke or resize would record typing cadence and input length of the live
// terminal stream, and the standing user disclosure promises that stream is
// never read, stored, or logged.
const UNTRACED_COMMANDS = new Set(["write_terminal_input", "resize_terminal"]);

// Invoke tracing choke point, called from `call` in client.ts: command name,
// duration, and outcome only. Never the args or the result; both may carry
// workspace paths and user text. A slow ok forwards at warn instead of debug
// so an unusually long invoke is greppable, not buried in the DEBUG "ok" lines.
export function traceInvoke(
  command: string,
  durationMs: number,
  failure?: unknown,
  slow = false
): void {
  if (!import.meta.env.DEV || !hasTauriRuntime() || UNTRACED_COMMANDS.has(command)) {
    return;
  }
  const duration = durationMs.toFixed(1);
  if (failure !== undefined) {
    forward(error, `invoke ${command}: failed in ${duration}ms: ${formatValue(failure)}`);
  } else if (slow) {
    forward(warn, `invoke ${command}: SLOW ok in ${duration}ms`);
  } else {
    forward(debug, `invoke ${command}: ok in ${duration}ms`);
  }
}

// The official console mapping from the plugin docs: console.log is the
// noisiest so it forwards at trace, the rest map one-to-one.
const CONSOLE_SINKS = {
  log: trace,
  debug,
  info,
  warn,
  error,
} as const;

function forwardConsole(level: keyof typeof CONSOLE_SINKS): void {
  const original = console[level].bind(console);
  const sink = CONSOLE_SINKS[level];
  console[level] = (...args: unknown[]) => {
    original(...args);
    forward(sink, args.map(formatValue).join(" "));
  };
}

// Route console output, window errors, and stray promise rejections into the
// app log file so every run leaves a reviewable trail without screenshots.
// Called once in development from main.tsx, main window only (the logging
// capability does not reach monitor windows or release builds).
export function installDiagnosticLogging(): void {
  if (!hasTauriRuntime()) {
    return;
  }
  for (const level of Object.keys(CONSOLE_SINKS) as (keyof typeof CONSOLE_SINKS)[]) {
    forwardConsole(level);
  }
  window.addEventListener("error", (event) => {
    // A benign, non-actionable browser quirk: the observer just reschedules to
    // the next frame. Chromium fires it as an uncaught error, so it would spam
    // the log (264 lines in one real session) and bury real errors. Drop it.
    if (event.message.includes("ResizeObserver loop")) {
      return;
    }
    const stack = event.error instanceof Error ? `\n${event.error.stack ?? ""}` : "";
    forward(
      error,
      `window.onerror: ${event.message} @ ${event.filename}:${event.lineno}:${event.colno}${stack}`,
    );
  });
  window.addEventListener("unhandledrejection", (event) => {
    forward(error, `unhandled rejection: ${formatValue(event.reason)}`);
  });
}
