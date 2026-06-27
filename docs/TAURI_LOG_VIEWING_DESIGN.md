# Tauri Log Viewing Design

Status: first bounded snapshot implemented.

RunHaven should make live agent output visible in the desktop app without
turning the WebView into a generic terminal, filesystem reader, or persistent
log store.

Current release context: this is alpha desktop behavior while RunHaven closes
the `v0.5.0` CLI-complete milestone. The bounded snapshot pattern remains part
of the `v1.0.0` first-class desktop requirement because raw output can contain
workspace content or secrets.

There are two different log surfaces and the UI must keep them separate:

- completed-run records from `runs log`, which join RunHaven run metadata with
  secret-free provider policy and auth broker decisions;
- raw active-container stdio from Apple `container logs`, which is live agent
  output and may contain sensitive workspace content.

## Problem

Users need enough run output to understand whether an agent is starting,
waiting, failing, or finished. The current desktop app shows setup, launch
plan, launch result, image readiness, resource warnings, and a sanitized live
status snapshot, but it does not show container stdio.

Agent output is sensitive. Raw logs can contain prompts, pasted secrets,
workspace file contents, command output, paths, provider errors, and auth
diagnostics. The secure easy path is therefore not automatic log display. The
easy path is status first, then an explicit bounded log snapshot when the user
asks to see output.

## Boundary

Trusted side:

- Rust command code validates the run id, resolves the active marker, validates
  the RunHaven-owned container name, invokes Apple `container logs`, caps the
  returned output, and serializes a typed response.
- Existing RunHaven CLI/runtime code remains the source of truth for active-run
  markers and Apple `container` command construction.

Untrusted side:

- The Tauri WebView requests a fixed log operation for a single run id and
  renders the returned text.
- The frontend may keep only the current visible buffer in component state. It
  must not write raw log text to local storage, IndexedDB, files, telemetry, or
  durable repo docs.

Sensitive data:

- Container stdio is raw user and agent output. Treat it as potentially secret
  even when the selected profile is a bundled non-root image.
- Do not promise automatic redaction. False confidence is worse than an
  explicit sensitive-output warning.

## Accepted Design

Add a bounded snapshot command before any live stream:

| Command | Capability | Request | Response |
| --- | --- | --- | --- |
| `get_log_snapshot` | `run-control` | `run_id`, optional `lines`, `confirm_sensitive_output` | `run_id`, `captured_at`, `requested_lines`, `text`, `returned_lines`, `truncated`, `source`, `warnings` |

Implementation status:

- `get_log_snapshot` is implemented as a typed Tauri command behind the
  `run-control` capability.
- Raw output remains hidden until the user acknowledges sensitive output.
- The Rust path validates the active marker and RunHaven-owned container name,
  calls only `container logs -n`, drains stdout/stderr with bounded memory, and
  caps the returned text.
- The Svelte UI stores only the current visible snapshot in component state.

Default behavior:

- The run detail surface shows live status first and a button to view latest
  output.
- The first log request requires an explicit sensitive-output acknowledgement.
- Default request is the last 200 lines from container stdio.
- Hard limits: no fewer than 1 line, no more than 500 lines, and no more than a
  small fixed byte budget in the returned response.
- The Rust command truncates oversized output and returns `truncated: true`.
- The UI renders the snapshot in a fixed-height scroll region with wrapping
  controlled by the existing app styles.

Out of scope for the first implementation:

- Live streaming through Tauri events or channels.
- Boot logs via `container logs --boot`.
- Attach/terminal behavior.
- Save, export, upload, or persistent search history for raw logs.
- Automatic redaction or secret scanning.
- Reading completed-run `runs log` records into the same raw-output buffer.
- Granting raw-log access through `main-read`.

## Apple Container Constraints

Local evidence on 2026-06-16:

- Host: macOS 26.5.1 on arm64.
- Apple `container`: CLI 1.0.0 build `release`, commit `ee848e3`.
- `container logs --help` supports
  `container logs [--boot] [--follow] [-n <n>] <container-id>`.

Implementation constraints:

- Use `container logs -n <lines> <container-name>` for the first snapshot.
- Do not pass `--follow` until a separate streaming design exists.
- Do not pass `--boot` in the default UI.
- Validate the container name with the existing RunHaven-owned container-name
  validator before calling Apple `container`.
- Confirm `run_id` to active marker to validated RunHaven-owned container name
  on every snapshot request.
- Capture stdout and stderr with size bounds; return clean frontend errors
  rather than raw command dumps.
- Keep Apple `container` install, update, system stop, machine, registry, image
  deletion, and volume deletion operations out of this command.

## Milestones

1. Rust log snapshot primitive
   - Current files: `crates/runhaven-core/src/runtime/active/`.
   - Add a reusable function that validates the active run, calls
     `container logs -n`, and converts bounded stdout into a typed payload.
   - Add unit tests for line validation, active-run container validation, byte
     truncation, and absence of raw command/env/mount data.

2. Tauri command contract
   - Files: `src-tauri/src/contracts.rs`,
     `src-tauri/src/commands/log_snapshot.rs`,
     `src-tauri/src/lib.rs`, `src-tauri/build.rs`, and
     `src-tauri/capabilities/`.
   - Add typed request/response structs and `get_log_snapshot`.
   - Require `confirm_sensitive_output`.
   - Grant only the new command through the narrow run-detail capability.

3. Frontend log panel
   - Files: `ui/src/commands/runhaven.ts`, `ui/src/app/App.svelte`,
     `ui/src/styles/global.css`, `ui/src/test/runhaven.test.ts`, and
     `ui/e2e/app.spec.ts`.
   - Keep logs hidden by default.
   - Add a clear acknowledgement path before loading the snapshot.
   - Render only the current bounded response in memory.
   - Show truncation state and refresh behavior without automatic polling.

4. Docs and harness state
   - Update `feature_list.json`, `current-state.md`,
     `docs/TAURI_UI_GUARDRAILS.md`, and `docs/ROADMAP.md` when their current
     contracts change.
   - Record evidence in `docs/harness/evidence/evidence-log.md`.

## Verification

Focused red checks before implementation:

```bash
cargo test --workspace --locked log_snapshot -- --nocapture
npm --prefix ui test -- --run
```

Required green checks for implementation:

```bash
scripts/apple_container_smoke.sh
cargo fmt --check
cargo test --workspace --locked log_snapshot -- --nocapture
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm --prefix ui test -- --run
npm --prefix ui run check
npm --prefix ui run test:e2e
npm --prefix ui run build
python3 -m json.tool feature_list.json >/dev/null
python3 -m json.tool src-tauri/capabilities/main-read.json >/dev/null
git diff --check
```

Use `./init.sh` when the implementation changes shared launch, active-run,
capability, or package behavior beyond the narrow snapshot command.

## Decision Log

- Snapshot before stream: bounded output is easier to cap, reason about, and
  test than a live event stream.
- Warning before raw output: raw agent output can contain secrets or workspace
  contents, so the secure default is status first and logs only after explicit
  user intent.
- No automatic redaction: redaction misses create a false sense of safety, and
  over-redaction can hide the actual failure.
- No plugin or sidecar: the standard Rust process API and existing Apple
  `container` CLI path are sufficient for the first implementation.
