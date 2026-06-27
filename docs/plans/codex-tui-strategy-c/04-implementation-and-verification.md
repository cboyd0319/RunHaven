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

### Phase 3: Compile The Dormant Runtime Spine

Before switching runtime ownership, make the dormant Codex runtime spine compile
with the smallest useful compatibility surface:

- `tui.rs` and `tui/*`
- `custom_terminal.rs`
- `app_server_session.rs` with unsupported backend methods stubbed or routed to
  the local RunHaven facade
- a minimal `App` shell shape, without activating the full product loop

Prefer local adapters and small protocol/utility crates over broad Codex backend
workspace members. Required dependencies and crossterm features belong in this
phase, with exact pins and pin-check evidence.

Phase gate:

- `cargo check -p runhaven-tui --locked`
- focused tests for the newly compiled adapter surface
- `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`
- `cargo run --locked --bin runhaven-check-pins`

### Phase 4: Prove Terminal Handoff

Before wiring real foreground launch, prove terminal restoration independently.
Use Codex `Tui`, but avoid creating a large throwaway custom runtime wrapper
inside `app_shell.rs`; keep `app_shell.rs` working as a staging shell until the
real `App` loop migration.

Bring active enough to test:

- `tui.rs`
- `tui/event_stream.rs`
- `tui/frame_requester.rs`
- `tui/frame_rate_limiter.rs`
- `tui/job_control.rs`
- `custom_terminal.rs`
- terminal stderr guard
- terminal title cleanup
- pet image cleanup

Phase gate:

- deterministic tests for `Tui::with_restored(...)` pause/restore/resume
  behavior
- event-stream pause/resume test coverage where practical
- a PTY smoke with a harmless foreground child process, not a real agent launch
- proof that early child errors restore terminal state and surface as a TUI
  error

Do not wire `runhaven/launch/prepare` to foreground
`launch_run_plan` until this gate passes.

### Phase 5: Adapt `App` And `BottomPane`

Make the real app loop active and move from the staging shell to Codex terminal
runtime ownership in the same migration. Avoid a separate permanent
`app_shell.rs`-plus-`Tui` architecture.

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

### Phase 6: Adapt `ChatWidget` Transcript And Status

Bring active:

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
hidden or clearly unavailable.

### Phase 7: Reattach RunHaven Product Screens

Move existing RunHaven screens into Codex-shaped surfaces:

- Agent picker: `ListSelectionView` or command palette.
- Workspace picker: Codex popup/view pattern, backed by `RunOptions`.
- Plan review: history/status card plus confirmation request.
- Confirm launch: approval/request input path, not one-off shell state.
- Dashboard: status card and active run transcript items.
- Logs: bounded log snapshot item or overlay.
- History/diff: resume picker/history cell/diff renderer patterns.
- Diagnostics/doctor: status card and markdown renderer.
- Pet and logo: Codex pet/image primitives, RunHaven assets.
- Zork easter egg: a RunHaven-owned `BottomPaneView` or equivalent Codex-shaped
  local view, opened through a custom `AppEvent` from the intended hidden `~`
  trigger. It must stay offline, attributed, TUI-local, and constrained to the
  private RunHaven save slot.

### Phase 8: Cull Or Stub Unsupported Codex Product Features

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
