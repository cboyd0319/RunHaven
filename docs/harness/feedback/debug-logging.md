# Desktop debug logging

Dev/test log trail for the Tauri desktop shell. Every run of the app writes a
reviewable local log so problems can be caught from the log file instead of
screenshots.

The user-facing "Debug logs" section lives in `docs/DESKTOP_APP.md`; this
note is the maintainer-facing detail.

## What gets logged

- Rust side: `tauri-plugin-log` (exact-pinned, see `pins.toml`) initialized
  first in `src-tauri/src/lib.rs`. Level is TRACE in debug builds, INFO in
  release. Targets: stdout and `runhaven.log` under the app log dir.
- WebView side (main window only): `@tauri-apps/plugin-log` forwarding in
  `ui/src/commands/logging.ts`, installed by `ui/src/main.tsx`. Console
  output keeps its normal behavior and is also appended to the same file,
  along with `window.onerror` and unhandled promise rejections (bounded to
  8 KiB per message).
- Invoke tracing: every typed command call through `call` in
  `ui/src/commands/client.ts` logs the command name, duration in ms, and
  ok/error. Never the arguments or results.

## What never gets logged

- The live terminal stream. PTY bytes flow over Tauri Channels, not the log
  pipeline. Per-keystroke commands (`write_terminal_input`,
  `resize_terminal`) are excluded from invoke tracing so even typing cadence
  stays out of the file. xterm.js at its default `logLevel` ("info") never
  sends buffer content through console; do not lower that option.
- The broker's HTTP request bytes. The broker's HTTP client runs IN the app
  process, and the http/tls crates (`ureq`, `ureq_proto`, `rustls`, ...)
  hexdump the raw request -- including the injected `Authorization: Bearer`
  token and the request body -- only at TRACE. The global log level is capped
  at DEBUG in dev (INFO in release), so no crate can wire-dump a secret, and
  the http/tls targets are further pinned to WARN. `scripts/scan-dev-log.sh`
  fails on any token-shaped text as a backstop. Do NOT raise the global level
  to TRACE or un-cap those targets.
- Secrets: no env values, tokens, headers, bodies, or broker payloads. The
  broker token is a Debug-redacted `SecretString`. No log line formats a
  launch request or plan payload. Because the console forwarder would copy
  anything app code prints, `ui/src/test/no-console-guard.test.ts` keeps
  production source console-free outside the reviewed logging module, so the
  promise is enforced, not aspirational.

## Rotation policy

`runhaven.log` rotates at 5 MiB; the five most recent rotated files are kept
(`RotationStrategy::KeepSome(5)`), so the trail is bounded at about 30 MiB.

## Capability boundary

The WebView write path is the log plugin's single `log` command, granted by
`src-tauri/capabilities/logging.json` (`log:allow-log`, main window only).
The grant is pinned by `logging_capability_stays_write_only_and_main_only`
in `src-tauri/src/capability_guard/command_scopes.rs` and vetted in
`VETTED_PLUGIN_PERMISSIONS`. Monitor windows have no log grant.
