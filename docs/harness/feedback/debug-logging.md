# Desktop debug logging

Diagnostic log trail for the Tauri desktop shell. Development runs write a
reviewable local log so problems can be caught from the log file instead of
screenshots. Release builds keep structured Rust-side lifecycle logging, but
do not grant the untrusted WebView permission to append arbitrary strings. The
user-facing "Debug logs" section lives in
`docs/DESKTOP_APP.md`; this note is the maintainer-facing detail.

Run the app with `npm --prefix ui run tauri:dev:log`
(`scripts/dev-with-logs.sh`): it tees the whole session to a rotated
`~/Library/Logs/RunHaven/dev.log`, and secret-scans every log on exit (Ctrl-C
included) via `scripts/scan-dev-log.sh`, so no session ends unscanned.

## What gets logged

- Rust side: `tauri-plugin-log` (exact-pinned, see `pins.toml`) initialized
  first in `src-tauri/src/lib.rs`. Global level is DEBUG in debug builds, INFO
  in release (capped below TRACE on purpose, see below). Targets: stdout and
  `runhaven.log` under the app log dir. `tao`/`wry` are damped to INFO.
- WebView side (development main window only): `@tauri-apps/plugin-log` forwarding in
  `ui/src/commands/logging.ts`, installed by `ui/src/main.tsx`. Console
  output keeps its normal behavior and is also appended to the same file,
  along with `window.onerror` and unhandled promise rejections (bounded to
  8 KiB per message). The benign `ResizeObserver loop` browser message is
  dropped so it cannot bury real errors.
- Invoke tracing: every typed command call through `call` in
  `ui/src/commands/client.ts` logs the command name, duration in ms, and
  ok/error. Never the arguments or results. A call over 1000 ms is logged at
  WARN (`SLOW ok`) so outliers stand out.
- Run correlation: run-scoped Rust lines carry a greppable `run <id>:` prefix
  (also `image build <id>:`, `sign-in <id>:`), so `grep <run_id> runhaven.log`
  returns one run's whole lifecycle. The four background runners
  (`launch_runner`, `agent_sign_in_runner`, `image_build_runner`,
  `commands/launch/mod.rs`) log through `log::` (they used to print to stderr
  only, invisible in the file).
- Per-run summary: `write_run_record`
  (`crates/runhaven-core/src/records/run_history/summary.rs`) emits one INFO
  line per finished run -- run_id, agent, status, return_code, duration, broker
  label, and provider/auth allowed-denied counts. Non-secret, the same fields
  the JSONL record persists.
- Launch timing: `launch_runner` logs spawn-to-running and total run elapsed
  (via `Instant`), so a slow launch shows spawn cost vs run duration. True
  per-container-step timing needs a core `launch.rs` change (tracked gap).
- Panics: `install_panic_log_hook` logs the panic thread, location, and payload
  at ERROR before chaining to the default hook, so a crash reaches the file.

## What never gets logged (and why it holds)

- The live terminal stream. PTY bytes flow over Tauri Channels, not the log
  pipeline. Per-keystroke commands (`write_terminal_input`, `resize_terminal`)
  are excluded from invoke tracing so even typing cadence stays out of the
  file. xterm.js at its default `logLevel` ("info") never sends buffer content
  through console; do not lower that option.
- The broker's HTTP request bytes. The broker's HTTP client runs IN the app
  process, and the http/tls crates (`ureq`, `ureq_proto`, `rustls`, ...)
  hexdump the raw request -- including the injected `Authorization: Bearer`
  token and the request body -- only at TRACE. The PRIMARY guarantee is the
  global level cap below TRACE, so no crate present or future can wire-dump a
  secret; the http/tls targets are further pinned to WARN. Do NOT raise the
  global level to TRACE or un-cap those targets.
- Secrets: no env values, tokens, headers, bodies, or broker payloads. The
  broker token is a Debug-redacted `SecretString`. No log line formats a launch
  request or plan payload. `ui/src/test/no-console-guard.test.ts` keeps
  production source console-free outside the reviewed logging module.

Defense in depth (belt-and-suspenders, NOT a replacement for the cap):
`src-tauri/src/log_format.rs` scrubs any residual secret shape
(`sk-ant-...`, `bearer <token>`, `x-api-key:`/`authorization:` headers,
`*_TOKEN=`/`*_API_KEY=`) from the final message text to `[REDACTED]` before it
reaches either sink. `scripts/scan-dev-log.sh` greps the same shapes and fails
on any hit; the tee runner runs it automatically on exit.

## Rotation policy

`runhaven.log` rotates at 5 MiB; the five most recent rotated files are kept
(`RotationStrategy::KeepSome(5)`), so the trail is bounded at about 30 MiB.
The tee `dev.log` is capped at 5 MiB with one rotated backup by
`scripts/dev-with-logs.sh`.

## Capability boundary

The development WebView write path is the log plugin's single `log` command,
granted by `src-tauri/capabilities/logging.json` (`log:allow-log`, main window
only) through `src-tauri/tauri.dev.conf.json`. The release config omits that
capability. The boundary is pinned by `logging_capability_stays_write_only_and_main_only`
and `release_config_never_grants_webview_logging`, then vetted in
`VETTED_PLUGIN_PERMISSIONS`. Monitor windows have no log grant.
