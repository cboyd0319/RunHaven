# 04 Implementation And Verification

## Source Integration Order

### Phase 0: Lock The Vendor Baseline

Before changing behavior, make the vendored Codex baseline explicit:

- Update `crates/runhaven-tui/src/tui/README.md` with the upstream Codex
  GitHub repository and commit used for the snapshot.
- Record that upstream snapshot goldens stay external in the pinned upstream
  Codex source by default, while RunHaven snapshots cover wired RunHaven
  behavior.
- Add a compare command that includes Rust source, helper binaries, tests, and
  snapshot/reference files.
- List every local RunHaven-only file and every copied Codex file with local
  edits.
- Keep `mod.rs` and `app_shell.rs` marked as temporary staging files.

This phase is documentation and source-control hygiene. It prevents later
implementation work from turning the fork boundary into guesswork.

Phase 0 focused checks:

```bash
bash -n scripts/compare-codex-tui.sh
scripts/compare-codex-tui.sh
scripts/compare-codex-tui.sh --list-missing >/tmp/runhaven-codex-tui-missing.txt
cargo run --locked --bin runhaven-check-pins --quiet
jq empty feature_list.json
python3 -m json.tool feature_list.json >/dev/null
! rg -n "[[:blank:]]$" scripts/compare-codex-tui.sh crates/runhaven-tui/src/tui/README.md docs/plans/codex-tui-strategy-c/*.md current-state.md feature_list.json
! LC_ALL=C rg -n "[^ -~]" scripts/compare-codex-tui.sh crates/runhaven-tui/src/tui/README.md docs/plans/codex-tui-strategy-c/*.md current-state.md feature_list.json
git diff --check
```

### Phase 1: Stop Growing The Temporary Shell

Keep `app_shell.rs` working, but do not add new product features to it except
to unblock migration.

Immediate work:

- Move any direct `runhaven-core` calls from `app_shell.rs` into a
  RunHaven TUI service module.
- Keep `launch_wizard.rs` as a UI-owned view model. The service/facade returns
  payloads and events; the temporary shell or future `App` turns them into
  views.
- Add tests for the service mapping independent of Ratatui rendering.

Status:

- Complete as of 2026-06-27. `app_shell.rs` now calls
  `RunHavenTuiService::launch_preview_payload` instead of direct
  `runhaven-core` planner/profile APIs.
- `runhaven/service.rs` returns `AgentLaunchPreview` payloads with typed
  per-agent `LaunchPreviewError` values, and `launch_wizard.rs` remains
  UI-owned.
- Service tests cover profile-name mapping, default network and auth scope,
  provider metadata, shell internet confirmation, shared agent state volumes,
  nested git workspace notes, and missing-workspace errors.

### Phase 2: Build The Codex-Shaped Backend Facade

Create the local client/protocol/service described above.

Minimum requirements:

- `request_typed`
- `next_event`
- `shutdown`
- request handle for background tasks
- bounded event channel
- typed errors that distinguish unsupported method, validation error, and
  backend failure
- lossless delivery for transcript/completion events, best-effort delivery for
  progress/log noise, matching Codex's client design

Tests for this phase must cover more than request mapping. Block this phase on
deterministic Tokio tests for:

- `request_typed`
- `next_event`
- `shutdown`
- request-handle cancellation
- lag signaling
- lossless delivery of transcript, completion, and launch-prepared events
- best-effort dropping of noisy progress or log events
- typed unsupported, validation, and backend-failure errors

Also add a fail-closed method matrix before the full `App` and `ChatWidget`
come online. At minimum, one facade test must prove each disabled family returns
a typed unsupported error:

- `fs/*`
- `mcpServer/*`
- `plugin/*`
- `marketplace/*`
- `app/list`
- `hooks/*`
- `remoteControl/*`
- `environment/add`
- `account/login/*`
- `account/logout`
- `feedback/upload`
- `windowsSandbox/*`
- Codex external-agent import

When a disabled family becomes visible in the UI, add a UI-level test that it is
hidden or shows a clear fail-closed local message.

Status:

- Complete as of 2026-06-27. `runhaven/protocol.rs` now defines the local
  Codex-shaped request, notification, server-request, lag, validation, and
  disabled-method contract.
- `runhaven/app_server_client.rs` mirrors Codex's bounded in-process client
  shape with `request_typed`, `next_event`, `shutdown`, a cloneable request
  handle, server-request resolve/reject methods, and lossless versus
  best-effort event forwarding.
- The worker loop keeps request handling off the command loop so future
  interactive service flows can continue draining events and accepting
  server-request responses.
- Facade tests cover typed requests, request-handle cancellation, next-event
  delivery, shutdown, server-request resolution and rejection, lossless
  transcript, completion, and launch-prepared events, lag signaling,
  best-effort progress and log dropping, validation, backend, and deserialize
  errors, plus the fail-closed disabled-method matrix.

### Phase 3: Switch To Codex Terminal Runtime

Replace the `ratatui::try_init()` loop in `app_shell.rs` with Codex `Tui`.
Bring the Codex runtime spine active with vendor-first crate wiring wherever
practical:

- `tui.rs` and `tui/*`
- `custom_terminal.rs`
- terminal stderr guard
- terminal title cleanup
- pet image cleanup
- `app_server_session.rs` with unsupported backend methods stubbed or routed
  through the RunHaven backend boundary
- enough `App` shape to host the current launch picker inside the Codex runtime

Keep the current visual launch picker as the first screen if needed, but host it
inside the Codex runtime.

Phase gate:

- `cargo check -p runhaven-tui --locked`
- focused tests for the newly compiled adapter surface
- `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`
- `cargo run --locked --bin runhaven-check-pins`
- deterministic tests for `Tui::with_restored(...)` pause/restore/resume
  behavior
- event-stream pause/resume test coverage where practical
- a PTY smoke with a harmless foreground child process, not a real agent launch
- proof that early child errors restore terminal state and surface as a TUI
  error

Do not wire `runhaven/launch/prepare` to foreground
`launch_run_plan` until this gate passes.

Status 2026-06-27: supporting runtime and handoff gates are complete. RunHaven
compiles the vendored Codex `tui.rs` runtime spine as `codex_runtime`,
including `tui/event_stream.rs`, `frame_requester.rs`,
`frame_rate_limiter.rs`, `job_control.rs`, `terminal_stderr.rs`, and
`custom_terminal.rs`. Unsupported Codex method families stay typed and fail
closed. `Tui::with_restored(...)` now has deterministic sequencing tests for
normal and alt-screen handoff, including child-error resume. The event broker
has a pause/drop/resume regression. The env-gated
`RUNHAVEN_TUI_HANDOFF_SMOKE=success|error` path initializes the Codex runtime,
clears managed title and pet image state before handoff, runs only a harmless
foreground child or an intentional missing child, restores terminal ownership,
and exits without wiring real agent launch. This completed gate does not
renumber the canonical plan. The next phase remains Phase 4: adapt `App` and
`BottomPane`.

### Phase 4: Adapt `App` And `BottomPane`

Make the real app loop active and move from the staging shell to Codex terminal
runtime ownership in the same migration. Avoid a separate permanent
`app_shell.rs`-plus-`Tui` architecture.

Progress note, 2026-06-27: `app_event.rs`, `app_event_sender.rs`,
`bottom_pane/mod.rs`, and `workspace_messages.rs` now compile from the real
vendored source under their original module paths. `launch_wizard.rs` now
implements `BottomPaneView` for the current picker/review/confirm flow. The
remaining Phase 4 work is native `App` ownership, replacing bridge types with
real shared modules, showing the launch wizard through `ChatWidget` and the
native bottom pane, and a fail-closed design for host-reaching Codex
app/session/chat surfaces before they become active.

Progress note, 2026-06-27: the next attempted direct `chatwidget` promotion
showed that `ChatWidget`, `history_cell`, and `status` share the same missing
Codex `legacy_core::config::Config` authority. RunHaven added the low-level
Codex utility crate authorities that do not pull the backend stack
(`codex-utils-cli`, `codex-utils-elapsed`, and
`codex-utils-sleep-inhibitor`) and kept `chatwidget` dormant until the
`legacy_core` compatibility decision is handled vendor-first.

Progress note, 2026-06-28: RunHaven added an original-name reduced
`codex-core` crate for the config compatibility path. The crate exposes
config-facing source shapes needed by native `App`/`ChatWidget` promotion and
has guard tests that block backend/runtime dependencies and modules. This does
not activate full Codex core, app-server, login, MCP, filesystem, hooks, tools,
rollout, state, or session behavior.

Progress note, 2026-06-28: RunHaven added an original-name reduced
`codex-app-server-client` crate exposing only the upstream-shaped
`legacy_core` compatibility re-export. A direct real `status`/`history_cell`
activation was tested and reverted because it cascades into `ChatWidget` before
the bottom pane ownership slice is ready. Continue Phase 4 bottom-pane-first:
move `LaunchWizardView` under native `BottomPane` ownership, or add the
smallest source-shaped host hook needed for that, before activating native
`App`, `ChatWidget`, or app-server transport.

Progress note, 2026-06-28: The live staging shell now hosts
`LaunchWizardView` inside the real vendored `BottomPane`. Key events, paste,
rendering, cursor placement, frame scheduling, selected-index lookup, terminal
title, footer status, text-input routing, and footer help now flow through
`BottomPane` or defaulted `BottomPaneView` contracts instead of direct
launch-wizard ownership. At that point, confirmation only set a notice and the
view completed only on cancel; the later foreground launch handoff supersedes
that read-only behavior. Native `App` and `ChatWidget` remain dormant until
their host-reaching surfaces are fail-closed or routed through reviewed
RunHaven boundaries.

Progress note, 2026-06-29: The live staging shell now initializes and restores
the real vendored Codex `Tui` runtime. Its loop consumes `TuiEventStream`,
draws through `Tui::draw`, and shares the Codex `FrameRequester` with
`BottomPane` and the temporary Cubby image smoke path. This removes the
previous `ratatui::try_init()` plus raw `crossterm::event::poll/read` loop from
the active bare-interactive path. Native `App`, `ChatWidget`, real
`app_server_session`, and app-server transport remain dormant until
host-reaching surfaces are removed, fail-closed, or routed through reviewed
RunHaven boundaries.

Bring active:

- `app.rs`
- `app/*`
- `app_event.rs`
- `app_event_sender.rs`
- `bottom_pane/mod.rs`
- `bottom_pane/chat_composer.rs`
- `public_widgets/composer_input.rs`
- `bottom_pane/footer.rs`
- `bottom_pane/list_selection_view.rs`
- `bottom_pane/textarea.rs`
- approval and request-user-input overlays as needed

Replace Codex product initialization with RunHaven bootstrap data.

Preserve the Codex loop inputs:

- `AppEvent` from widgets and background UI work
- active thread events
- `TuiEventStream`
- `AppServerSession::next_event`

Before this phase finishes, define the authoritative RunHaven TUI snapshot
matrix. Include at least `80x24` and `120x48` coverage for picker, review,
confirm, transition-clearing cases, transcript, history, diff, log, diagnostics,
and any visible fail-closed unsupported surface.

### Phase 5: Adapt `ChatWidget` Transcript And Status

Bring active only where needed for RunHaven's MVP transcript, status, and log
surfaces:

- `chatwidget.rs`
- `chatwidget/input_*`
- `chatwidget/turn_*`
- `chatwidget/status_surfaces.rs`
- `history_cell/*`
- `streaming/*`
- `markdown_stream.rs`
- `exec_cell/*`
- `diff_render.rs`

Map RunHaven service events into history cells. Keep unsupported Codex commands
hidden or clearly unavailable. Do not port non-RunHaven Codex product features
for parity; leave them dormant, fail-closed, stubbed, or deleted with a
documented reason.

### Phase 6: Reattach RunHaven MVP Product Screens

Move only the RunHaven MVP screens into Codex-shaped surfaces:

- Agent picker: `ListSelectionView` or command palette.
- Workspace picker: Codex popup/view pattern, backed by `RunOptions`.
- Plan review: history/status card plus confirmation request.
- Confirm launch: approval/request input path, not one-off shell state.
- Foreground launch: validated plan from the facade, terminal restore on the UI
  thread, and active run tracking after handoff.
- Active run transcript/logs: bounded transcript items, log snapshot item, or
  overlay.
- Diagnostics/doctor: status card and markdown renderer.

Defer Cubby/pet polish, mascot work, terminal image polish, Zork, dashboard
breadth, rich history/diff, and Codex-native product affordances until the core
RunHaven TUI is fully working. Existing pet/image code remains source-first
infrastructure and opt-in smoke coverage, not near-term product scope.

Progress note, 2026-06-29: The MVP workspace picker became active inside the
BottomPane-owned `LaunchWizardView` without expanding `app_shell.rs`. The
RunHaven service offers current directory and git repository root choices when
the current directory is nested inside a repo, and the agent preview list is
rebuilt from the selected workspace before review. At that point foreground
launch was still disabled; the next progress note supersedes that launch state.

Progress note, 2026-06-29: The confirmation step now prepares a typed launch
intent instead of only showing a disabled notice. `LaunchWizardView` emits
`AppEvent::RunHavenLaunchPrepared` with a RunHaven `PreparedLaunch`, which
keeps the selected `LaunchPlanData` for display and the original
`AgentRunPlan` for execution. The staging shell now exits its draw loop with
that intent and calls `runhaven/launch_handoff.rs`, the single TUI owner allowed
to invoke `launch_run_plan`, after Codex `Tui::with_restored` has released
terminal ownership. The Codex-shaped `ServerNotification::LaunchPrepared` still
carries the display plan payload for the later App/ChatWidget event path. A
guard keeps widgets and backend facade code from calling `launch_run_plan` and
keeps full launch plans out of Codex session logging until RunHaven owns a
redaction policy.

Progress note, 2026-06-29: The first active-run transcript/log backing seam is
promoted without expanding `app_shell.rs`: `runhaven/run/logSnapshot` now flows
through the local protocol, in-process client, service, and session bridge into
`runhaven_core::runtime::active::active_run_log_snapshot_payload`. The UI
contract maps the existing snake_case core payload into
`ActiveRunLogSnapshotData` camelCase display data. Guard coverage keeps the
direct active-run log snapshot runtime call owned by `runhaven/service.rs`, and
requests must carry explicit raw-output confirmation before line validation and
backend lookup. This is not a visible ChatWidget transcript/log screen yet; the
raw payload can contain secrets or workspace content, so future visible UI must
define redaction and session-recording policy before rendering, caching, or
logging it. App-server transport, filesystem RPC, MCP, login, workspace command
execution, Codex session recording, native `App`, and `ChatWidget` remain
dormant or fail-closed.

### Phase 7: Cull Or Stub Unsupported Codex Product Features

After the Codex-shaped app shell is active, decide each dormant Codex product
surface:

- Keep active.
- Keep dormant with a fail-closed message.
- Delete with a documented reason.

Record decisions in:

- `crates/runhaven-tui/src/tui/README.md`
- `docs/plans/tui-build-plan.md`
- `docs/plans/tui-architecture.md`

Do not delete large copied modules before the active app shell exists, because
their dependencies may become useful once `App` and `ChatWidget` are wired.

## File-Level Ownership Rules

Use these rules while editing:

1. If a file exists upstream in Codex TUI, keep the file path and public shape
   close to upstream.
2. Prefer import adapters and type mappers over rewriting the file.
3. RunHaven product logic goes under `tui/runhaven/` or `runhaven-core`.
4. Widgets draw data. They do not build run plans, inspect active runs, call
   `container`, or parse CLI output.
5. `app_server_session.rs` is the facade. If a widget needs backend data, add a
   typed facade method instead of reaching around it.
   For foreground launch, the facade returns a validated plan and the UI loop
   owns terminal restoration plus `launch_run_plan`.
6. Keep `mod.rs` shrinking. If it gains more staged compatibility definitions,
   that is debt and should be called out.
7. Keep local changes to copied files documented in `tui/README.md`.
8. Preserve access to upstream snapshots and fixtures as external reference
   material where practical.
9. Regenerate RunHaven snapshots. Do not import upstream Codex `.snap` files as
   authoritative RunHaven behavior.
10. New RunHaven views should implement or reuse `BottomPaneView`,
   `ChatWidget`, history cells, or status surfaces before adding a custom
   rendering loop.
11. New RunHaven UI commands should go through `AppEventSender` or
   `AppServerSession`, not direct widget-to-service calls.
12. For `bottom_pane/chat_composer.rs` and `bottom_pane/paste_burst.rs`, keep
   module docs and implementation behavior aligned. The upstream bottom-pane
   instructions call this out specifically for Enter/newline paths and
   `disable_paste_burst` semantics.

## Security Boundary Requirements

The Codex-shaped TUI must not weaken RunHaven's boundary:

- No host home mount by default.
- No cloud credential folder mount by default.
- No raw SSH key mount by default.
- No browser profile mount by default.
- No arbitrary environment passthrough by default.
- No root agent by default.
- No bypass around `build_run_plan` validation.
- No direct widget calls to `container`.
- No background filesystem RPC that can mutate host files outside RunHaven's
  selected workspace model.
- No Codex account login path that reads host Codex credentials.
- No MCP, plugin, connector, IDE, or remote-control feature without a
  RunHaven security design and explicit promotion.

Every mutating RunHaven action in the TUI should be backed by the same core
function used by CLI or Tauri. Examples:

- Launch: `launch_run_plan`
- Stop: `stop_active_run`
- Kill: `kill_active_run`
- Repair: `repair_active_run`
- Log snapshot: `active_run_log_snapshot_payload`
- Status: `active_run_status_payload`
- Diff: `run_diff_text`
- Diagnostics: `read_egress_policy_log`, `read_auth_broker_log`,
  `auth_status_payload`

## Testing And Verification

For source-only planning docs, no Rust checks are required. For implementation,
state, or script work, use the smallest focused checks.

After TUI code changes:

```bash
cargo fmt --check
cargo test -p runhaven-tui --locked
cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings
```

If `runhaven-core` UI contracts or runtime paths change:

```bash
cargo test -p runhaven-core --locked
cargo test -p runhaven-tui --locked
```

If snapshots change:

```bash
cargo insta pending-snapshots -p runhaven-tui
cargo insta show -p runhaven-tui <path-to-snap.new>
cargo insta accept -p runhaven-tui
```

For terminal image behavior:

```bash
RUNHAVEN_TUI_IMAGE_SMOKE=1 cargo run --locked --bin runhaven
```

For substantial integration slices on macOS 26+:

```bash
./init.sh
```

When vendored dependencies, crate wiring, terminal runtime ownership, or
workspace-level paths change, add the workspace gates:

```bash
cargo test -p runhaven --locked bare_non_tty_prints_cli_help --quiet
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
cargo run --locked --bin runhaven-check-pins
```

When a new vendored module family is activated, also compile the gated upstream
test surface:

```bash
cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run
```

Add or update tests for:

- `RunHavenTuiService` request-to-core mapping.
- `RunHavenTuiService` request/event ordering, lag/drop behavior, cancellation,
  shutdown, and typed unsupported/validation/backend errors.
- a fail-closed unsupported-method matrix covering disabled Codex method
  families before corresponding UI paths are visible.
- Event emission for launch, stop, kill, repair, status, log snapshot, diff,
  diagnostics.
- Terminal restore before interactive launch, including early launch errors.
- Composer and paste-burst state-machine behavior when RunHaven changes input
  handling.
- Fail-closed unsupported Codex methods.
- Lower-security confirmation prompts.
- VT100 snapshots for each active screen.
- No secret-bearing fields in diagnostics/log/status TUI payloads.

For docs-only changes in this plan set:

```bash
rg -n "[[:blank:]]$" docs/plans/codex-tui-strategy-c/*.md
LC_ALL=C rg -n "[^ -~]" docs/plans/codex-tui-strategy-c/*.md
```

## Upstream Sync Workflow

Use the pinned upstream source comparison first:

```bash
scripts/compare-codex-tui.sh
```

This command fetches `https://github.com/openai/codex.git` at the pinned commit
recorded in `crates/runhaven-tui/src/tui/README.md` and compares
`codex-rs/tui/src/` against `crates/runhaven-tui/src/tui/`.

To list upstream files missing from RunHaven:

```bash
scripts/compare-codex-tui.sh --list-missing
```

To include upstream snapshots and tests in the audit, do not filter by file
extension. The compare command intentionally includes `.rs`, `.snap`, test
modules, helper binaries under `src/bin`, and local instruction files under
`src/`.

To list RunHaven-local files:

```bash
scripts/compare-codex-tui.sh
```

Expected RunHaven-local files today:

```text
README.md
app_shell.rs
mod.rs
pets/bundled_custom.rs
runhaven/launch_wizard.rs
runhaven/mod.rs
terminal_detection.rs
terminal_tests.rs
```

When syncing:

1. Sync upstream Codex TUI source first.
2. Reapply only documented RunHaven changes.
3. Update `tui/README.md` with any new local exception.
4. Preserve upstream snapshots as external reference material or record why a
   specific local copy is necessary.
5. Regenerate RunHaven snapshots rather than accepting Codex snapshots as
   RunHaven behavior.
6. Run focused checks.

## Open Design Decisions

These need explicit choices before full Strategy C implementation:

1. Whether RunHaven wants a chat composer as a command surface, a command
   palette, or both.
2. Whether a RunHaven "thread" is always a run id, or whether the TUI session
   has its own thread id that can contain multiple run actions.
3. Whether launch is always terminal-restoring and foreground, or whether
   RunHaven adds a background launch mode.
4. Which dormant Codex surfaces stay as disabled menu entries versus removed
   code after the app shell is active.

Recommended defaults:

- Vendor Codex protocol, utility, and TUI-adjacent crates first, preserving
  original crate names where practical.
- Keep active RunHaven product behavior behind RunHaven-owned adapters even
  when the source is vendored from Codex.
- Keep the Codex composer, but make it command/action oriented for RunHaven.
- Use a TUI session thread id, and attach run ids as items/actions under it.
- Foreground launch restores the terminal before starting the agent.
- Keep dormant Codex surfaces fail-closed until the full app shell works.
