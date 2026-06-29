# Current State

Last updated: 2026-06-29 UTC

## Current Objective

The active slice is the TUI Codex vendor reset plus the Rust workspace
normalization needed to make that TUI a clean reference implementation.

RunHaven is replacing its previous custom TUI with the pinned upstream Codex
TUI source baseline:

```text
repo: https://github.com/openai/codex.git
commit: 5267e805fb830891c0b23376bcd9cbd382c3473c
path: codex-rs/tui/src/
```

The RunHaven TUI is the reference implementation for several sibling projects.
Keep Codex vendoring source-first, RunHaven behavior in thin adapters, shared
data contracts in `runhaven-core`, every culling decision documented, and
user-facing copy plain enough for non-technical users.

Do not publish a release from the interim vendor-reset state. After the TUI is
fully integrated, verified, and confirmed, do a full release bump to `v0.6.0`.

## Startup State Contract

- `AGENTS.md`: root instruction map.
- `feature_list.json`: compact feature status and next product slice.
- `current-state.md`: current facts, blockers, and handoff.

Do not recreate separate root `progress.md` or `session-handoff.md` files.
Load deeper docs only when the task touches that surface.

## Product Facts

- RunHaven is a Rust 1.96.0 workspace for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon.
- The `v0.5.0` CLI-complete pre-release was cut and published on 2026-06-26.
- The CLI remains the complete automation and recovery backend.
- The terminal UI is unreleased and active. A bare interactive `runhaven`
  should open the TUI when the TUI is integrated; pipes, redirection, and
  explicit subcommands stay CLI-first.
- The alpha desktop shell lives under `ui/` and `src-tauri/`. `src-tauri` is a
  Rust workspace member over typed `runhaven-core` commands. The desktop shell
  remains deferred to a later first-class release phase.
- Windows and Linux are not supported runtime or contributor-verification
  targets.
- GitHub Actions CI is disabled during alpha/pre-release. Local macOS 26+
  verification is authoritative until a maintainer explicitly re-enables CI.

## Rust Source Layout

| Area | Path | Owns |
| --- | --- | --- |
| Binary entrypoints | `crates/runhaven/` | `runhaven` and `runhaven-check-pins` startup, including the bare-interactive TUI routing decision. |
| Core library | `crates/runhaven-core/` | Runtime, provider, records, image, doctor, diagnostics, support, harness pin logic, and shared UI contracts. |
| CLI presentation | `crates/runhaven-cli/` | Clap dispatch, setup text, and human CLI output. |
| Terminal UI | `crates/runhaven-tui/` | Codex-vendored TUI source plus RunHaven TUI adapters. |
| Desktop Rust shell | `src-tauri/` | Narrow typed Tauri commands over `runhaven-core`. |
| Frontend | `ui/` | Alpha Svelte desktop UI. |

`crates/runhaven` is binary-only. Do not rebuild a root compatibility facade.
Shared runtime truth belongs in `runhaven-core`; presentation belongs in CLI,
TUI, Tauri, or frontend layers.

## Key Decisions

- Secure defaults must be the easiest path. Supported lower-security choices
  warn and require explicit intent. Unsupported or hard-boundary violations
  fail closed.
- Default runs use task-scoped `container run`, not `container machine`,
  because normal machine workflows can expose host home and credentials.
  Explicit or user-managed machine workflows are not blocked solely for being
  less secure, but any RunHaven machine integration must warn and require
  intent.
- Do not mount host home directories, cloud credential folders, raw SSH keys,
  browser profiles, or arbitrary host environment variables by default.
- `--ssh` remains fail-closed. Apple `container` 1.0.0 exposes the forwarded
  socket to the non-root guest, but `ssh-add -l` returns permission denied.
  Reopen only with a no-secret runtime proof.
- Provider egress stays default-deny in provider mode. Agent provider domains
  can be expressed as reviewed stable domain-family patterns, but data-egress
  hosts stay closed by not being in the allowlist.
- Log sanitization and host-held secret storage are separate non-TUI security
  slices. Because RunHaven is macOS 26+ only, RunHaven-owned host secrets should
  prefer macOS Keychain where practical. This does not authorize reading
  provider-owned Keychain items, browser profiles, cloud credential stores, or
  arbitrary host credentials.
- TUI image and pet rendering must follow Codex source behavior. Use the pinned
  upstream Codex TUI source and local Codex config evidence before writing custom pet,
  terminal image, statusline, bottom-pane, keymap, title, or resume behavior.
  Cubby, pet polish, mascot work, and terminal image polish are final-pass TUI
  work after the core RunHaven TUI is complete. Keep existing pet code as
  parked source-first infrastructure; do not add a live env-gated smoke path
  unless a final-pass pet slice or core terminal-image check explicitly requires
  it.
- TUI implementation slices should use the repo-local
  `.agents/skills/codex-tui` skill first. It requires the Persona Codex TUI
  skill (`/Users/c/Documents/GitHub/persona/content/skills/codex-tui`), then
  `rust` and `adversarial-review` as the end-of-slice gate before commit: Rust
  crate/tooling correctness, Codex source-pattern alignment, then boundary and
  overclaim review.
- Antigravity (`agy`) is research-only in this repo. Do not use it for
  end-of-slice code review, adversarial review, verification, or proof of
  correctness.
- For Codex-vendored TUI and `codex-*` dependencies, preserving the original
  Codex package name, crate name, and module path is the default. Use a local
  bridge only when compiling or activating the real Codex surface would cross a
  RunHaven security boundary that has not been designed and tested.
- User-facing writing is product behavior. UI text, menus, prompts, warnings,
  README/usage docs, and setup instructions target non-technical users at about
  an 8th grade reading level.
- The hidden Zork I easter egg remains wanted. The current reset keeps the
  MIT-licensed `historicalsource/zork1` collection under `third_party/zork1/`.
  The earlier Ferrif-derived TUI engine was removed with the old custom TUI and
  is recoverable from git history. If reintroduced, it must stay TUI-local,
  attributed, offline, and carefully validate save/restore files. Defer it
  until the core TUI is complete and final polish starts.
- The glib advisory GHSA-wrw7-89jp-8q8g remains treated as not affected because
  `glib` enters only through Tauri's Linux GTK backend and is absent from the
  macOS build graph. See `docs/PINNING.md`.

## Latest Verified Work

2026-06-27: Workspace crate split complete. The Rust codebase now uses
workspace crates:

- `crates/runhaven` for binary entrypoints.
- `crates/runhaven-core` for shared runtime truth and UI contracts.
- `crates/runhaven-cli` for CLI presentation.
- `crates/runhaven-tui` for the Codex-vendored TUI.
- `src-tauri` as a workspace member.

This phase also removed the obsolete separate Tauri lockfile, made root Cargo
commands cover Tauri, narrowed public crate exports, kept `runhaven`
binary-only, and refreshed active architecture, harness, pinning, TUI, and
state docs to the new layout.

Follow-up ownership audit fix: `crates/runhaven` now owns the bare-interactive
TUI routing decision, `crates/runhaven-cli` no longer depends on
`crates/runhaven-tui`, the unused `records::history` compatibility alias is
gone, `init.sh` uses `cargo test -p runhaven-tui --locked` as the TUI package
gate, and empty untracked vendored snapshot directories were removed from the
local tree. Dormant vendored Codex test modules remain source-first until their
parent modules are wired back into the RunHaven TUI app shell.

Repo-wide organization audit follow-up: tracked source is now clean of root
`src/`, tracked build output, `.snap` files, `.DS_Store`, and the obsolete
`src-tauri/Cargo.lock`. The largest visible directory clutter was ignored local
build output, not tracked source. Tauri npm scripts now set
`CARGO_TARGET_DIR` to the absolute root `target/` path so desktop builds use
the root workspace target directory. The stale ignored `src-tauri/target/`,
frontend `dist/`, Playwright
reports, test results, and empty `.github/workflows/` directory were removed
locally. `docs/harness/state/clean-state-checklist.md` records which ignored
directories are allowed caches and which should be cleaned when they appear.
Active stale doc paths were corrected in the research and Tauri/TUI design docs;
historical evidence logs were left as records of what happened at the time.

TUI native-pet image smoke follow-up (superseded): the earlier temporary
`app_shell.rs` image-smoke path has been removed from the active shell during
core-completion cleanup. RunHaven still bundles the verified Cubby Codex pet
package from `docs/assets/installed-pet/cubby/` and can materialize it as
`custom:runhaven-cubby` under `$CODEX_HOME/pets/runhaven-cubby/` through the
lower pet modules, avoiding collisions with a user's own
`$CODEX_HOME/pets/cubby/` package. Reintroduce terminal-image smoke only in a
final-pass pet slice or when a core terminal-image check explicitly requires it.

TUI component-seam follow-up: `crates/runhaven-core/src/ui_contracts.rs` now
defines the first tagged RunHaven payload enum with `AgentCatalogData` and
`LaunchPlanData`; `LaunchPlanData` includes the planner's auth scope so the TUI
does not guess whether login state is agent-wide or project-scoped. Fixtures live under
`crates/runhaven-core/tests/fixtures/ui/`. The temporary TUI adapter consumes
`AgentCatalogItemData` for agent display, but the next visual slice should move
toward a Codex-native shell with RunHaven product cards. dbt-wizard is only the
architecture proof for stable domain payloads first and renderer second. The
visual target is closer to native Codex: compact intro and status content,
bottom composer and status line, and no analytics dashboard feel in the default
launcher. Native Cubby behavior is final-pass polish after the core TUI is
complete.

TUI bottom-pane follow-up: `crates/runhaven-tui` now compiles the Codex
`ListSelectionView` family directly from the vendored bottom-pane source through
a narrow staging facade. The facade exposes the Codex-shaped event sender, list
keymap, paste normalization, cancellation, and completion types needed by
`ListSelectionView` and the pet picker. Its default list keymap now mirrors the
upstream Codex list defaults. The upstream Codex list-selection snapshot tests
are gated behind the opt-in `codex-vendored-tests` feature until RunHaven
intentionally vendors or regenerates those snapshot goldens.

TUI launch-picker follow-up: `app_shell.rs` no longer owns a hand-drawn
Ratatui agent list or preview pane. It now hosts a RunHaven launch-wizard view
model under `crates/runhaven-tui/src/tui/runhaven/launch_wizard.rs`, rendered
through Codex `SelectionViewParams` and `ListSelectionView`. The generic picker
logic, side-content layout, cancellation, and list key handling remain
Codex-vendored. RunHaven-owned code maps `AgentCatalogData` and
`LaunchPlanData` into decision rows and a short safety header. The first agent
chooser intentionally does not show a side plan preview, exact command, broker
detail, image detail, or provider-host list. That dense safety information
belongs in the review and confirmation steps, where the user checks it before
launch.

TUI launch-review follow-up: Enter on a ready agent now opens a read-only review
step rendered through the Codex menu-surface style. The review shows the
selected agent, auth scope, network posture, workspace mount, state volume,
non-shared host data, provider hosts, safety notes, and exact `container run`
command. `b`, backspace, or Esc returns to the picker; `q` exits from either
screen. Blocked plans cannot open review. At that review slice, launch and
preflight execution were still disabled; later confirmation and foreground
handoff follow-ups supersede that state.

TUI shell-chrome follow-up: the temporary app shell now reserves a real Codex
footer area around the launch picker and review screen. The footer is rendered
through Codex's vendored `bottom_pane/footer.rs` with a RunHaven status line
showing step, selected agent, network posture, boundary, and `? help`. The
shell also uses Codex's sanitized `terminal_title.rs` writer so terminal tab
titles track the workspace, step, and selected agent, and clears the managed
title on exit. The vendored footer snapshot tests are gated behind
`codex-vendored-tests`, matching the list-selection snapshot policy because
upstream `.snap` goldens are intentionally not tracked.

TUI launch-confirmation follow-up: the launch wizard now has Step 4 confirmation
on top of the current Codex menu-surface review. Enter from review opens
confirmation, and the confirmation screen keeps the exact planner command
visible while using `LaunchPlanData.confirm_required` as the only typed-confirm
gate. Plans that need extra intent now use the vendored Codex `TextArea`
editor primitive for the confirmation phrase. While that text field is focused,
plain `q` and `?` are text input instead of shell shortcuts; Esc returns to
review. Paste is ignored for the lower-security confirmation phrase so the
extra intent still means typing. Secure/default plans still confirm with Enter
and keep `q` as the shell quit shortcut. At that point this was still a
read-only preview; the later foreground launch handoff supersedes the
no-launch behavior.

TUI confirm-composer follow-up: `crates/runhaven-tui` now compiles the vendored
Codex `bottom_pane/textarea.rs` and `bottom_pane/textarea/vim.rs` through the
staging facade. The facade has the Codex editor/Vim keymap defaults, and the
byte-range text element types now come from the real vendored `codex-protocol`
crate. The upstream deterministic textarea tests run by default; the snapshot
and randomized stress tests remain opt-in with the same `codex-vendored-tests`
policy as the other upstream snapshot goldens.

TUI capabilities-doc follow-up: `docs/plans/codex-tui-capabilities.md` now
locks the full local Codex TUI capability survey into repo docs. Use it as the
source map before custom TUI work. It confirms Codex already has mature terminal
runtime, bottom-pane/composer, keymap, selection popup, approval, markdown,
diff, streaming, history-cell, session, status, pet, terminal-title, and
VT100/snapshot-test systems.

TUI architecture correction: the deeper read of
`/Users/c/Downloads/codex-tui-capabilities.md` showed that the full
`ChatComposer` is not the next isolated seam unless RunHaven only uses the small
`public_widgets::ComposerInput` wrapper. Native Codex TUI behavior is built as
`Tui` runtime plus `App` event loop plus `ChatWidget` plus `BottomPane`, with
`app_server_session.rs` owning typed client calls so transport plumbing stays out
of `App` and `ChatWidget`. For RunHaven, the next source-first target is the
terminal runtime/event stream and typed app-server facade pattern, then the full
ChatWidget/BottomPane path, followed by streaming/history cells, approval
surfaces, status, sessions, and terminal UI regression tests. Host-reaching
Codex RPCs such as remote filesystem, MCP, and IDE actions stay fail-closed
unless a RunHaven security design explicitly promotes them.

Strategy decision: RunHaven is following the capability guide's Strategy C path,
a Codex-compatible client, because its domain is close to Codex's
agent/thread/turn/session model. Strategy B, extracting a small TUI kit, is only
a temporary compile bridge for low-coupling modules such as `ComposerInput`,
wrapping, diff rendering, or selection helpers. It is not the product
architecture.

Strategy C Phase 3 runtime-spine follow-up: `crates/runhaven-tui` now compiles
the vendored Codex `tui.rs` runtime spine as `codex_runtime`, including the
event stream, frame requester, frame limiter, Unix job-control hook, terminal
stderr guard, custom terminal, insert-history writer, notifications, and
terminal hyperlinks. The new `runhaven/app_server_session.rs` bridge routes
supported bootstrap, agent-catalog, and workspace-validation calls into the
local RunHaven facade and keeps unsupported Codex method families typed and
fail-closed. Required dependency changes are exact-pinned: crossterm now enables
the upstream event-stream and bracketed-paste features, Ratatui enables
scrolling regions, and `tokio-stream` plus `derive_more` are direct workspace
dependencies for the compiled Codex runtime surface. The Codex runtime is still
dormant; Phase 4 is terminal handoff proof before native `App` loop activation.

Verified:

- `cargo fmt --check`
- `cargo test -p runhaven-tui --locked launch_wizard -- --nocapture`
- `cargo test -p runhaven-tui --locked app_shell -- --nocapture`
- `cargo test -p runhaven-tui --locked --quiet`
- `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`
- `cargo test -p runhaven --locked bare_non_tty_prints_cli_help --quiet`
- `cargo test -p runhaven-cli --locked --quiet`
- `cargo tree -p runhaven-cli --locked` with no `runhaven-tui`, `ratatui`,
  `crossterm`, `tokio`, `reqwest`, or `image` dependency matches
- `cargo test --workspace --locked --quiet`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo run --locked --bin runhaven-check-pins --quiet`
- `cargo build --workspace --locked --quiet`
- `npm --prefix ui run tauri:build` with `CARGO_TARGET_DIR` resolving to root
  `target/`
- `test ! -d src-tauri/target`
- `jq empty feature_list.json`
- `python3 -m json.tool feature_list.json`
- `python3 -m json.tool ui/package.json`
- active stale-reference scans
- `git diff --check`
- `./init.sh`
- `cargo run --locked --bin runhaven` in a PTY, pressed Enter to open review,
  pressed `b` to return to the picker, then pressed `q` to quit.

Latest TUI smoke verification:

- `cargo fmt --check`
- `cargo test -p runhaven-tui --locked runhaven_cubby --quiet`
- `cargo test -p runhaven-tui --locked picker_ --quiet`
- `cargo test -p runhaven-tui --locked launch_wizard --quiet`
- `cargo test -p runhaven-core --locked ui_contracts --quiet`
- `cargo test -p runhaven-tui --locked app_shell --quiet`
- `cargo test -p runhaven-tui --locked launch_wizard -- --nocapture`
- `cargo test -p runhaven-tui --locked app_shell -- --nocapture`
- `cargo test -p runhaven-tui --locked terminal_title --quiet`
- `cargo test -p runhaven-tui --locked textarea --quiet`
- `cargo test -p runhaven-tui --locked runhaven::app_server_session -- --nocapture`
- `cargo test -p runhaven-tui --locked custom_terminal::tests -- --nocapture`
- `cargo test -p runhaven-tui --locked codex_runtime -- --nocapture`
- `cargo test -p runhaven-tui --locked runhaven::app_server_client -- --nocapture`
- `cargo test -p runhaven-tui --locked runhaven::service -- --nocapture`
- `cargo test -p runhaven-tui --locked runhaven::launch_wizard -- --nocapture`
- `cargo test -p runhaven-tui --locked app_shell -- --nocapture`
- `cargo test -p runhaven-tui --locked codex_runtime::tests::with_restored -- --nocapture`
- `cargo test -p runhaven-tui --locked codex_runtime::event_stream::tests::paused_broker_drops_source_until_resume -- --nocapture`
- `cargo test -p runhaven-tui --locked runhaven::terminal_handoff -- --nocapture`
- `cargo check -p runhaven-tui --locked`
- `cargo test -p runhaven-tui --locked --quiet`
- `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`
- `cargo test -p runhaven-tui --locked pets::image_protocol --quiet`
- `cargo test -p runhaven-tui --locked pets --quiet`
- `cargo test -p runhaven-tui --locked kitty_file_png_transmission_encodes_local_file_reference --quiet`
- `cargo test -p runhaven-tui --locked ambient_pet_image_restores_cursor_after_drawing --quiet`
- `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`
- `cargo clippy -p runhaven-core --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked --quiet`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo build --workspace --locked --quiet`
- `cargo test -p runhaven --locked bare_non_tty_prints_cli_help --quiet`
- `cargo run --locked --bin runhaven-check-pins --quiet`
- `jq empty feature_list.json`
- `python3 -m json.tool feature_list.json`
- `git diff --check`
- `CODEX_HOME=$(mktemp -d) RUNHAVEN_TUI_IMAGE_SMOKE=1 cargo run --locked --bin
  runhaven` in a PTY, quit with `q`; it materialized
  `pets/runhaven-cubby/{pet.json,spritesheet.webp}`, emitted Codex Kitty
  local-file frames from the `custom-runhaven-cubby` frame cache, and exited
  cleanly.
- `cargo run --locked --bin runhaven` in a PTY, pressed `?` to show footer
  help, Enter to open review, `b` to return to the picker, and `q` to quit;
  the terminal title changed between Choose agent and Review plan and cleared
  on exit.
- `cargo run --locked --bin runhaven` in a PTY, pressed Enter to open review,
  Enter to open confirmation, Enter to confirm the read-only notice, `b` to
  return to review, and `q` to quit; the terminal title changed through Choose
  agent, Review plan, and Confirm launch, then cleared on exit.
- `RUNHAVEN_TUI_HANDOFF_SMOKE=success cargo run --locked --bin runhaven` in a
  PTY; Codex runtime initialized, cleared the managed title before handoff,
  restored terminal ownership around `/usr/bin/printf`, printed the harmless
  child marker, and exited 0.
- `RUNHAVEN_TUI_HANDOFF_SMOKE=error cargo run --locked --bin runhaven` in a
  PTY; Codex runtime initialized, cleared the managed title before handoff,
  restored terminal ownership after the missing child failed to start, surfaced
  `terminal handoff child failed to start`, and exited 2.

Latest harness review:

- 2026-06-27: Reviewed
  `/Users/c/Documents/GitHub/learn-harness-engineering/docs/en/resources/`,
  including the minimal templates, reference notes, OpenAI advanced pack, SOPs,
  and repo-template docs. Decision: keep RunHaven's three-file startup contract
  and map external template concepts onto existing RunHaven owners instead of
  adding parallel root progress/handoff, quality, reliability, plan, or product
  spec files. Verified with pin check, JSON validation, typography scan over
  changed files, and `git diff --check`.

Latest TUI strategy review:

- 2026-06-27: Fully read `/Users/c/Downloads/codex-tui-capabilities.md` and
  checked the conclusion against local Codex source entrypoints for
  `app_server_session`, `App`, `ChatWidget`, `BottomPane`, `Tui`,
  `TuiEventStream`, and `FrameRequester`. Decision: RunHaven should follow the
  Strategy C compatible-client path, with Strategy B limited to temporary
  bridges and low-coupling helpers. Verified with `jq empty feature_list.json`,
  `git diff --check`, and a no-em-dash typography scan over changed docs/state
  files.

Latest Strategy C plan import:

- 2026-06-27: Imported the split Strategy C plan from
  `/Users/c/Downloads/runhaven-codex-tui-strategy-c/` into
  `docs/plans/codex-tui-strategy-c/`. Read all five plan files and ran
  read-only adversarial, Rust architecture, and Rust test-architecture reviews.
  The repo copy incorporates the review fixes: do not broadly add Codex backend
  crates as workspace authorities; compile the dormant runtime spine before
  terminal handoff; keep `launch_wizard.rs` UI-owned while the service returns
  payloads/events; keep upstream `.snap` files external by default; add
  deterministic facade, fail-closed, terminal handoff, workspace-gate, and
  snapshot-matrix requirements; prepare foreground launch through the facade but
  execute `launch_run_plan` on the UI thread only after terminal restore; keep
  the hidden Zork easter egg as a future RunHaven-owned Codex-shaped view.

Latest TUI Phase 0 baseline lock:

- 2026-06-27: Completed Phase 0 of the Strategy C plan without runtime behavior
  changes. `crates/runhaven-tui/src/tui/README.md` now records the upstream
  Codex GitHub repo, pinned commit
  `5267e805fb830891c0b23376bcd9cbd382c3473c`, upstream path
  `codex-rs/tui/src/`, RunHaven-only files, and copied Codex files with local
  edits. Added `scripts/compare-codex-tui.sh`, which fetches the pinned
  upstream Codex source from GitHub into a temporary checkout and compares all
  files under `codex-rs/tui/src/`, including Rust files, `src/bin`, nested
  tests, and `.snap` goldens. Current audit: 894 upstream files, 364 RunHaven
  files, 356 common paths, 538 upstream `.snap` files external by default, 8
  RunHaven-only files, and 20 copied Codex files with local edits. Verified:
  `bash -n scripts/compare-codex-tui.sh`,
  `scripts/compare-codex-tui.sh`,
  `scripts/compare-codex-tui.sh --list-missing`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `jq empty feature_list.json`,
  `python3 -m json.tool feature_list.json`, whitespace and ASCII scans, and
  `git diff --check`.

Latest TUI Phase 1 service extraction:

- 2026-06-27: Completed Phase 1 of the Strategy C plan. `app_shell.rs` no
  longer calls `runhaven-core` planner/profile APIs directly. The temporary
  RunHaven TUI service in `crates/runhaven-tui/src/tui/runhaven/service.rs`
  builds launch preview payloads from core profiles and `LaunchPlanData`, keeps
  per-agent planner errors typed, and leaves
  `crates/runhaven-tui/src/tui/runhaven/launch_wizard.rs` as the UI-owned view
  model over Codex `ListSelectionView`. Service tests cover agent-name
  mapping, default network and auth scope, provider metadata, shell internet
  confirmation, shared agent state volumes, nested git workspace notes, and
  fail-per-agent missing-workspace errors. Current vendor audit: 894 upstream
  files, 365 RunHaven files, 356 common paths, 538 upstream `.snap` files
  external by default, 9 RunHaven-only files, and 20 copied Codex files with
  local edits.

Latest TUI Phase 2 backend facade:

- 2026-06-27: Completed Phase 2 of the Strategy C plan. Added the local
  Codex-shaped request, event, server-request, validation, and disabled-method
  contract in `crates/runhaven-tui/src/tui/runhaven/protocol.rs`. Added the
  bounded in-process client facade in
  `crates/runhaven-tui/src/tui/runhaven/app_server_client.rs` with
  `request_typed`, `next_event`, `shutdown`, a cloneable request handle,
  request cancellation, server-request resolve/reject methods, lossless
  transcript/completion/launch-prepared event delivery, and best-effort
  progress/log dropping with lag markers. `RunHavenTuiService` now dispatches
  neutral facade requests for agent catalog and workspace validation while
  keeping the temporary launch-preview payload for the staging shell. The
  request worker spawns service work off the command loop, matching Codex's
  non-blocking client shape for future interactive flows. Focused facade tests
  cover the Phase 2 matrix, including the fail-closed disabled method families.
  Verified: `cargo fmt --check`,
  `cargo test -p runhaven-tui --locked app_server_client --quiet`,
  `cargo test -p runhaven-tui --locked --quiet`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `scripts/compare-codex-tui.sh`, `cargo run --locked --bin runhaven-check-pins --quiet`,
  JSON validation, ASCII and whitespace scans, and `git diff --check`.

Latest TUI Strategy C drift correction:

- 2026-06-27: Imported
  `docs/plans/codex-tui-strategy-c/05-adversarial-drift-ledger.md` and restored
  the plan's vendor-first wording. The canonical Strategy C phase order is back
  to Phase 4 = adapt native `App` and `BottomPane`; the runtime-spine compile
  and terminal-handoff proof are recorded as completed Phase 3 gates, not a
  separate phase that shifts the target later. `tui/mod.rs` now has a guard
  test that fails if dormant host-reaching Codex surfaces are declared before
  their risky upstream markers are removed or fail-closed.

Latest TUI staging-facade shrink:

- 2026-06-27: Removed the inline `codex_protocol::user_input` shim from
  `crates/runhaven-tui/src/tui/mod.rs` and first replaced it with file-backed
  staged leaves under `crates/runhaven-tui/src/tui/codex_protocol/`. Added drift
  guards so `mod.rs` cannot grow new inline staging modules, new `codex_*`
  self-aliases, or a native `app` declaration that still routes `run()` through
  `app_shell::run()`.
- 2026-06-29: Moved the `tui/mod.rs` drift and security guard tests into
  `tui/drift_tests.rs`. This keeps the same guard coverage while shrinking
  `tui/mod.rs` to declarations plus the TUI entrypoint. The only remaining
  inline staging module in `tui/mod.rs` is the narrow `onboarding` hyperlink
  shim, and the guard still fails if new inline staging modules are added.

Latest Codex protocol crate vendoring:

- 2026-06-27: Began real `codex-*` crate vendoring under original package and
  library names. Added `crates/codex/` workspace members for
  `codex-protocol`, `codex-app-server-protocol`, and their first dependency
  closure: `codex-async-utils`, `codex-execpolicy`,
  `codex-experimental-api-macros`, `codex-network-proxy`,
  `codex-shell-command`, and the required `codex-utils-*` crates. Vendored
  crate manifests use explicit Apache-2.0, `0.0.0`, and `publish = false`
  local metadata, keep internal `codex-*` paths relative, preserve the upstream
  `runfiles` git rev for schema fixture tests, and align only external exact
  pins that Cargo's unified workspace resolver cannot hold twice. `runhaven-tui`
  now depends on the real vendored `codex-protocol` and
  `codex-app-server-protocol` crates, and the active `TextArea` path consumes
  `ByteRange` and `TextElement` from `codex_protocol::user_input`. Deleted the
  local `tui/codex_protocol/` staged leaf. Verified so far:
  `cargo check -p codex-protocol`, `cargo check -p codex-app-server-protocol`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p codex-protocol --locked --quiet`,
  `cargo test -p codex-app-server-protocol --locked --quiet`, and
  `cargo test -p runhaven-tui --locked drift_tests -- --nocapture`.

Latest TUI Phase 3 runtime and handoff gate:

- 2026-06-27: Completed the terminal-handoff proof without wiring real agent
  launch. `Tui::with_restored(...)` now has deterministic sequencing tests
  for normal and alt-screen handoff, including child-error resume. The event
  broker has a pause/drop/resume regression to prove events sent while the
  source is dropped do not leak into the resumed TUI. Added the local
  `runhaven/terminal_handoff.rs` smoke hook, gated by
  `RUNHAVEN_TUI_HANDOFF_SMOKE=success|error`, which initializes the Codex
  runtime, clears managed terminal title and pet image state before handoff,
  runs only a harmless foreground child or an intentional missing child, restores
  terminal ownership, and exits. Ambient and picker-preview pet image state now
  share a combined cleanup helper, including native `App` shutdown. Current
  latest vendor audit: 894 upstream files, 370 RunHaven files, 356 common
  paths, 538 upstream `.snap` files external by default, 14 RunHaven-only files,
  and 53 copied Codex files with local edits.

Latest Codex config/keymap crate vendoring:

- 2026-06-27: Continued real `codex-*` crate vendoring under original package
  and library names. Added workspace members for `codex-config`,
  `codex-api`, `codex-client`, `codex-features`, `codex-file-system`,
  `codex-git-utils`, `codex-model-provider-info`, `codex-otel`, and
  `codex-utils-path`, plus their local manifest wiring. `runhaven-tui` now
  depends on the real vendored `codex-config` crate, `lib.rs` no longer aliases
  `codex_config`, and the file-backed vendored `keymap.rs` compiles against
  `codex_config::types::{KeybindingsSpec, TuiKeymap, MAX_FUNCTION_KEY}` instead
  of an inline RunHaven keymap extract.
- Preserved upstream OpenAI fork git revs for `tokio-tungstenite` and
  `tungstenite` because Codex relies on those forks for proxy-enabled websocket
  behavior. `codex-client` pins `sha2` 0.10 because that source formats the
  digest with the 0.10 trait behavior; RunHaven-owned code keeps its existing
  `sha2` 0.11 pin.
- The active RunHaven launch/security authority is unchanged. These crates are
  vendored source authorities for TUI config/keymap/protocol types, not a
  promotion of Codex auth, filesystem RPC, app-server, or network client
  behavior into the active RunHaven runtime.
- `scripts/compare-codex-tui.sh` now compares deterministic file manifests
  instead of looping over `cmp` calls. Each manifest records relative path, byte
  size, and SHA-256, and `--write-manifests <dir>` writes the upstream/local
  manifests plus missing, local-only, common, and changed lists for audit.
- Verified:
  `cargo metadata --locked --no-deps --format-version 1`,
  `cargo check -p codex-config --locked`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked keymap --quiet`,
  `cargo test -p runhaven-tui --locked drift_tests -- --nocapture`,
  `cargo test -p codex-config --locked --quiet`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `bash -n scripts/compare-codex-tui.sh`, and
  `scripts/compare-codex-tui.sh --write-manifests <tempdir>`.

Latest Codex event-data crate vendoring:

- 2026-06-27: Continued real `codex-*` crate vendoring for the next
  `app_event.rs` activation. Added workspace members for `codex-connectors`,
  `codex-file-search`, `codex-plugin`, and
  `codex-utils-approval-presets`, plus the required plugin namespace closure:
  `codex-utils-plugins`, `codex-exec-server`,
  `codex-exec-server-protocol`, `codex-sandboxing`, `codex-utils-pty`, and
  `codex-windows-sandbox`.
- `runhaven-tui` now depends on the real vendored connector, file-search,
  plugin, and approval-preset crates so the real vendored `app_event.rs`
  imports have crate authority available. `app_event.rs` itself remains
  dormant until its shared TUI types are exposed without activating
  host-reaching Codex app paths.
- Added the same `tokio-tungstenite` and `tungstenite` crates.io patches used
  by upstream Codex, and pinned `codex-exec-server` to upstream Codex's
  `axum` 0.8.8. This avoids carrying an extra registry websocket stack
  (`tokio-tungstenite` 0.29) beside Codex's patched 0.28 fork.
- The active RunHaven launch/security authority is unchanged. These crates are
  vendored source authorities for TUI event-data compatibility, not a promotion
  of Codex exec-server, filesystem RPC, app-server, sandbox launch, plugin
  execution, or connector network behavior into the active RunHaven runtime.
- Local manifest integration found by the end-of-slice adversarial pass:
  `codex-plugin` allows Clippy's `result_large_err`, matching existing
  package-level exceptions in other vendored Codex crates and preserving
  upstream source under RunHaven's stricter `-D warnings` gate.
- The same adversarial pass found loose version specs inherited in target/dev
  dependency sections of new vendored manifests; they were tightened to exact
  pins before commit.
- Focused Cargo efficiency note: use one umbrella `cargo check -p runhaven-tui`
  after dependency graph changes, then rerun `cargo check -p runhaven-tui
  --locked`. Avoid parallel Cargo checks; they serialize behind package-cache
  and build-directory locks and are slower than one incremental umbrella check.
- Verified:
  `cargo metadata --locked --no-deps --format-version 1`,
  `cargo check -p runhaven-tui`,
  `cargo check -p runhaven-tui --locked`,
  `cargo check -p codex-plugin --locked`,
  `cargo check -p codex-file-search --locked`,
  `cargo check -p codex-connectors --locked`,
  `cargo test -p runhaven-tui --locked drift_tests -- --nocapture`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `python3 -m json.tool feature_list.json`,
  `find crates/runhaven-tui/src/tui -name '*.snap.new' -print`,
  `git diff --check`,
  `cargo tree -p runhaven-tui --locked -i tokio-tungstenite@0.29.0`
  (expected no package match), and
  `cargo tree -p runhaven-tui --locked -i tokio-tungstenite@0.28.0`.

Latest Codex event-bus activation:

- 2026-06-27: Activated the real vendored `app_event.rs` and
  `app_event_sender.rs` files. The four-variant inline `AppEvent` shim and the
  inline optional-channel `AppEventSender` shim are gone.
- Added `app_event_shared.rs` as a narrow inert leaf-type bridge for
  `AppServerStartedThread`, `UserMessage`, `GoalDraft`, `HistoryCell`,
  `StatusLineGitSummary`, hook trust updates, workspace headline results, and
  no-op session logging while the owning modules remain dormant. This is
  temporary bridge debt, not product behavior.
- `runhaven-tui` now directly depends on `codex-features` and
  `codex-utils-absolute-path` because the real event bus imports those
  authorities directly.
- The temporary launch wizard now gives `ListSelectionView` a real
  `AppEventSender` backed by a scratch channel instead of relying on a local
  `Default` implementation that does not exist upstream.
- Verified:
  `cargo fmt --check`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked drift_tests -- --nocapture`,
  `cargo test -p runhaven-tui --locked launch_wizard --quiet`,
  `cargo test -p runhaven-tui --locked app_shell --quiet`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `python3 -m json.tool feature_list.json`,
  `find crates/runhaven-tui/src/tui -name '*.snap.new' -print`, and
  `git diff --check`.

Latest Codex bottom-pane activation:

- 2026-06-27: Promoted the real vendored `bottom_pane/mod.rs` source under its
  original module path and added the original-name crate authorities needed by
  that surface: `codex-core-skills`, `codex-feedback`,
  `codex-models-manager`, and `codex-utils-fuzzy-match`.
- The default remains original Codex package, crate, and module names. Local
  bridges are exceptions only when activating the real surface would cross an
  unreviewed RunHaven security boundary or pull host-reaching behavior into the
  active TUI. The latest TUI sections below are authoritative for current
  named bridge exceptions; at this point in the history they included
  `app_event_shared.rs`, `status`, `onboarding`, and the narrow exposed
  surfaces inside `codex-core-skills`, `codex-feedback`, and
  `codex-models-manager`.
  `codex-feedback::FeedbackDiagnostics::collect_from_env()` is kept
  shape-compatible but returns no diagnostics until RunHaven has a redaction
  policy for host environment capture.
- Snapshot-heavy upstream bottom-pane tests are gated behind
  `codex-vendored-tests`; default RunHaven tests do not create `.snap.new`
  files from external Codex goldens. The opt-in feature still compiles as a
  vendored-source check.
- `scripts/compare-codex-tui.sh` now reports 894 upstream files, 370 RunHaven
  TUI files, 356 shared paths, 538 external upstream `.snap` goldens, 14
  RunHaven-only files, and 43 copied Codex files with local edits.
- Native Codex `App` remains inactive because its owning app/session/chat
  paths still include host environment, filesystem RPC, onboarding auth,
  external editor, clipboard, and hooks surfaces that need RunHaven-specific
  fail-closed design before activation.
- Verified so far:
  `cargo test -p runhaven-tui --locked --quiet`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo test -p runhaven-tui --locked drift_tests -- --show-output`,
  `cargo test -p runhaven-tui --locked launch_wizard -- --show-output`,
  `cargo test -p runhaven-tui --locked app_shell -- --show-output`,
  `cargo fmt --check`, and
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `python3 -m json.tool feature_list.json`,
  `cargo metadata --locked --no-deps --format-version 1`,
  `find crates/runhaven-tui/src/tui -name '*.snap.new' -print`,
  `git diff --check`, and
  `scripts/compare-codex-tui.sh`.

Latest Codex utility crate vendoring:

- 2026-06-27: Added original-name vendored workspace crates for
  `codex-utils-cli`, `codex-utils-elapsed`, and
  `codex-utils-sleep-inhibitor`, copied from the pinned local Codex source.
  `runhaven-tui` now depends on those authorities for dormant Codex TUI CLI,
  history, exec-cell, and chat turn-lifecycle imports.
- `codex-utils-sleep-inhibitor` has a scoped `unsafe_code = "allow"` lint
  exception because the upstream macOS implementation uses native IOKit power
  assertion FFI. The exception is local to that vendored utility crate and does
  not change RunHaven runtime safety boundaries.
- A direct `chatwidget` module declaration was tested and reverted before this
  commit because it exposed the pending shared closure rather than a clean
  activation point: real `ChatWidget` and `status` still require more of
  Codex's `legacy_core::config::Config` and app-server shape. `history_cell`
  has since been promoted through the reduced config boundary. Do not replace
  that with another custom RunHaven TUI stand-in.
- 2026-06-29: Promoted the real vendored Codex `history_cell/*`,
  `diff_render.rs`, `exec_cell/*`, `markdown*.rs`, `session_state.rs`,
  `tooltips.rs`, `update_action.rs`, and related root-module aliases out of the
  inert `app_event_shared.rs` bridge. Added original-name `codex-ansi-escape`
  plus the upstream markdown/diff/tooltip dependency closure and the upstream
  `tooltips.txt` asset. The reduced `codex-core` config now loads the
  upstream `tui.show_tooltips` field so session tooltip suppression follows the
  same setting. Two small source exceptions remain: update notices use plain
  Ratatui `Line`/`Text` construction because upstream `ratatui-macros` targets
  Ratatui 0.29, and yolo mode reads RunHaven's reduced Codex config shape until
  full Codex core config is promoted. The full upstream `history_cell/tests.rs`
  module remains parked because it currently requires full Codex config/MCP
  surfaces and snapshot goldens that are not promoted; default tests cover the
  reduced tooltip/config seams, yolo-mode mapping, basic diff rendering, decoded
  local-link control stripping, terminal print-boundary control stripping, and
  ANSI-output degradation.
- Verified so far:
  `cargo check -p codex-utils-cli --locked`,
  `cargo check -p codex-utils-elapsed --locked`,
  `cargo check -p codex-utils-sleep-inhibitor --locked`, and
  `cargo check -p runhaven-tui --locked`.
- Latest `history_cell` promotion verification:
  `cargo fmt --check`,
  `cargo check -p runhaven-tui --locked`,
  `cargo check -p codex-ansi-escape --locked`,
  `cargo check -p codex-install-context --locked`,
  `cargo test -p codex-core --locked show_tooltips --quiet`,
  `cargo test -p codex-ansi-escape --locked malformed_ansi_degrades_without_control_bytes --quiet`,
  `cargo test -p runhaven-tui --locked local_link_display_strips_decoded_terminal_controls --quiet`,
  `cargo test -p runhaven-tui --locked safe_print_symbol --quiet`,
  `cargo test -p runhaven-tui --locked drift_tests -- --show-output`,
  `cargo test -p runhaven-tui --locked launch_wizard --quiet`,
  `cargo test -p runhaven-tui --locked app_shell --quiet`,
  `cargo test -p runhaven-tui --locked --quiet`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `scripts/compare-codex-tui.sh`,
  `python3 -m json.tool feature_list.json`,
  `find crates/runhaven-tui/src/tui -name '*.snap.new' -print`, and
  `git diff --check`.

Latest Codex terminal-detection crate vendoring:

- 2026-06-28: Promoted `codex-terminal-detection` from the temporary
  `runhaven-tui` self-alias to a real original-name vendored crate under
  `crates/codex/terminal-detection`. The copied source and tests are
  byte-identical to the pinned local Codex source. `runhaven-tui` now depends
  on the crate directly, and the duplicate local `terminal_detection.rs` and
  `terminal_tests.rs` files were deleted. This removes the last `codex_*`
  self-alias from `runhaven-tui` without changing active TUI behavior.
- Verified:
  `cargo check -p codex-terminal-detection --locked`,
  `cargo test -p codex-terminal-detection --locked --quiet`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked drift_tests -- --show-output`,
  `cargo test -p runhaven-tui --locked terminal_palette --quiet`,
  `cargo test -p runhaven-tui --locked pets::image_protocol --quiet`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo fmt --check`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `python3 -m json.tool feature_list.json`,
  `find crates/runhaven-tui/src/tui -name '*.snap.new' -print`,
  `scripts/compare-codex-tui.sh`, and
  `git diff --check`.

Latest Codex config support crate vendoring:

- 2026-06-28: Added original-name vendored workspace crates for
  `codex-context-fragments`, `codex-install-context`,
  `codex-memories-read`, `codex-response-debug-context`,
  `codex-utils-output-truncation`, and `codex-utils-stream-parser`. These are
  low-coupling upstream support crates for the next reduced `codex-core`
  config compatibility closure. They are not active RunHaven product authority,
  and `runhaven-tui` does not route launch, install, memory, response-debug,
  app-server, login, MCP, or filesystem behavior through them in this slice.
- Added two drift guards: local `legacy_core` compatibility shims are blocked,
  and `app_event_shared.rs` may shrink but cannot grow new bridge modules or
  host-reaching behavior.
- Full upstream `codex-app-server-client` and full upstream `codex-core` remain
  intentionally inactive. The next useful step is the reduced original-name
  `codex-core` config compatibility closure, not promoting the app-server
  backend stack.
- Verified:
  `cargo check -p codex-utils-output-truncation -p codex-utils-stream-parser -p codex-context-fragments -p codex-install-context -p codex-memories-read -p codex-response-debug-context --offline`,
  `cargo check -p codex-utils-output-truncation -p codex-utils-stream-parser -p codex-context-fragments -p codex-install-context -p codex-memories-read -p codex-response-debug-context --locked`,
  `cargo test -p codex-utils-output-truncation -p codex-utils-stream-parser -p codex-context-fragments -p codex-install-context -p codex-memories-read -p codex-response-debug-context --locked --quiet`,
  `cargo test -p runhaven-tui --locked drift_tests -- --show-output`,
  `cargo fmt --check`, `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked --quiet`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `scripts/compare-codex-tui.sh`, JSON validation, snap-new scan, metadata
  check, typography scan for changed files, and `git diff --check`.

Latest Codex reduced core config authority:

- 2026-06-28: Added `crates/codex/core` as an original-name reduced
  `codex-core` workspace crate for the config compatibility path needed by
  native `App`/`ChatWidget` promotion. It exposes config-facing source-shaped
  surfaces, including terminal resize reflow, bootstrap keyring resolution,
  exec-policy warning/loading placeholders, Windows sandbox config helpers, and
  small path/unified-exec constants.
- This is not full Codex backend activation. The crate deliberately omits
  app-server, login, MCP, filesystem RPC, hooks, tools, rollout, state, and
  session runtime modules until RunHaven designs and verifies those boundaries.
  Guard tests prevent those backend dependencies/modules and block
  RunHaven-owned TUI adapters from importing `codex_core` runtime surfaces.
- Verified so far:
  `cargo fmt --check`,
  `cargo test -p codex-core --locked --quiet`,
  `cargo check -p codex-core --locked`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked drift_tests -- --show-output`,
  `cargo test -p runhaven-tui --locked --quiet`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `scripts/compare-codex-tui.sh`,
  `python3 -m json.tool feature_list.json`,
  `find crates/runhaven-tui/src/tui -name '*.snap.new' -print`, and
  `git diff --check`.

Latest Codex reduced app-server client compatibility authority:

- 2026-06-28: Added `crates/codex/app-server-client` as an original-name
  reduced `codex-app-server-client` workspace crate. It exposes only the
  upstream-shaped `codex_app_server_client::legacy_core` re-export backed by
  reduced `codex-core`. It deliberately omits app-server transport, remote
  clients, in-process client startup, login, MCP, filesystem RPC, exec-server,
  rollout, state, and thread-store behavior.
- A direct real `status`/`history_cell` activation was tested and reverted in
  the working tree because it cascaded into `ChatWidget` and richer config
  methods before the bottom pane ownership slice was ready. `history_cell` has
  since been promoted through the reduced config boundary. The next Phase 4
  slice should continue toward native `ChatWidget` ownership without activating
  native `App`, real app-server session, or app-server transport host-reaching
  behavior.
- Verified so far:
  `cargo check -p codex-app-server-client --offline`,
  `cargo test -p codex-app-server-client --locked --quiet`,
  `cargo check -p runhaven-tui --offline`, and
  `cargo test -p runhaven-tui --locked drift_tests --quiet`.

Latest TUI native bottom-pane ownership:

- 2026-06-28: The live staging `app_shell.rs` now hosts `LaunchWizardView`
  inside the real vendored `BottomPane`. Key events, paste, render, cursor
  placement, frame scheduling, selected-index lookup, terminal title, footer
  status, text-input routing, and footer help now flow through `BottomPane` or
  defaulted `BottomPaneView` contracts instead of direct launch-wizard
  ownership.
- At that point, confirmation was still read-only and the view completed only
  on cancel. The later foreground launch handoff supersedes that behavior.
  Native `App`, `ChatWidget`, real `app_server_session`, and app-server
  transport remain dormant until host-reaching surfaces are removed,
  fail-closed, or routed through reviewed RunHaven boundaries.
- Verified so far:
  `cargo fmt --check`,
  `cargo test -p runhaven-tui --locked app_shell --quiet`,
  `cargo test -p runhaven-tui --locked launch_wizard --quiet`,
  `cargo test -p runhaven-tui --locked --quiet`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `python3 -m json.tool feature_list.json`,
  `find crates/runhaven-tui/src/tui -name '*.snap.new' -print`,
  `scripts/compare-codex-tui.sh`, and
  `git diff --check`.

Latest TUI Codex runtime ownership:

- 2026-06-29: The live staging `app_shell.rs` now initializes and restores the
  real vendored Codex `Tui` runtime instead of using `ratatui::try_init()` and
  raw `crossterm::event::poll/read`. Its active loop consumes
  `TuiEventStream`, draws through `Tui::draw`, and shares the Codex
  `FrameRequester` with the hosted `BottomPane`. The earlier Cubby image-smoke
  path is no longer active.
- This preserved the launch picker, review, and confirmation behavior from
  that earlier disabled-launch phase. Native `App`, `ChatWidget`, real
  `app_server_session`, and app-server transport remain dormant until
  host-reaching surfaces are removed, fail-closed, or routed through reviewed
  RunHaven boundaries.
- Verified so far:
  `cargo fmt --check`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked app_shell --quiet`, and
  `cargo test -p runhaven-tui --locked launch_wizard --quiet`,
  `cargo test -p runhaven-tui --locked drift_tests --quiet`,
  `cargo test -p runhaven-tui --locked --quiet`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `python3 -m json.tool feature_list.json`,
  `find crates/runhaven-tui/src/tui -name '*.snap.new' -print`,
  `scripts/compare-codex-tui.sh`, and
  `git diff --check`.

Latest ChatWidget status-source promotion:

- 2026-06-29: Promoted the real vendored Codex `branch_summary.rs` and the
  `workspace_command.rs` contract for the next ChatWidget status-line path.
  `app_event_shared.rs` no longer owns the `StatusLineGitSummary` bridge; it
  re-exports the real upstream type from `branch_summary.rs`.
- This does not activate Codex app-server transport or host command execution.
  The upstream `AppServerWorkspaceCommandRunner` remains compiled dormant in
  `workspace_command.rs`, and a drift guard blocks `app_shell.rs` plus
  RunHaven-owned adapters from using it in this slice. `branch_summary.rs`
  remains best-effort metadata over an injected `WorkspaceCommandExecutor` and
  has no direct host process, environment, filesystem, or RunHaven core access.
- Verified so far:
  `cargo test -p runhaven-tui --locked` as the baseline,
  `cargo fmt --check`,
  `cargo check -p runhaven-tui --locked --tests`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked drift_tests -- --show-output`,
  `cargo test -p runhaven-tui --locked branch_summary -- --show-output`,
  `cargo test -p runhaven-tui --locked unsupported_methods_fail_closed -- --show-output`,
  `cargo test -p runhaven-tui --locked unsupported_method_matrix_fails_closed -- --show-output`,
  `cargo test -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `scripts/compare-codex-tui.sh`,
  `python3 -m json.tool feature_list.json >/dev/null`,
  `find crates/runhaven-tui/src/tui -name '*.snap.new' -print`, and
  `git diff --check`.

Latest repo-local agent harness audit:

- 2026-06-29: Deep review of `.agents/` found that
  `.agents/skills/codex-tui` is now a full repo-local TUI routing skill.
  `AGENTS.md`, this file, repo-local skills, and focused harness docs now
  describe `.agents/skills/` as on-demand instruction surfaces while
  preserving the three-file startup contract.
- Verified with repo-local skill validation for `codex-tui` and `harness`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `python3 -m json.tool feature_list.json >/dev/null`, local Markdown link
  check over touched docs, and `git diff --check`.

Latest TUI status-bridge reduction:

- 2026-06-29: Removed the inline root `status` bridge from
  `crates/runhaven-tui/src/tui/mod.rs` without activating the full Codex
  `status/` module. Active footer and hook-browser call sites now use
  `tui/runhaven/status_format.rs` for the two helper functions they need.
  `token_usage.rs` is active from real source, and a drift guard keeps full
  `status/` dormant until its config, model-provider, remote-app-server, and
  status-card closure is intentionally promoted.
- `AppEvent::StatusLineGitSummaryUpdated` now uses
  `branch_summary::StatusLineGitSummary` directly, so `app_event_shared.rs`
  no longer re-exports that type through the `chatwidget` bridge.
- Full `status/`, native `App`, `ChatWidget`, real `app_server_session`,
  app-server transport, filesystem RPC, MCP, login, workspace command
  execution, and other host-reaching Codex paths remain dormant or
  fail-closed.
- Verified so far:
  baseline `cargo test -p runhaven-tui --locked`,
  `cargo fmt --check`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked status_format -- --show-output`,
  `cargo test -p runhaven-tui --locked drift_tests -- --show-output`, and
  `cargo test -p runhaven-tui --locked footer -- --show-output`.
  Final gate:
  `cargo test -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `scripts/compare-codex-tui.sh`,
  `python3 -m json.tool feature_list.json >/dev/null`, snap-new scan, and
  `git diff --check`.

Latest TUI session-log source promotion:

- 2026-06-29: Promoted the real vendored Codex `session_log.rs` source and
  removed the no-op `session_log` bridge from `app_event_shared.rs`. This is a
  ChatWidget/AppEvent support surface only; a source-tree guard allows
  `session_log::maybe_init` and `CODEX_TUI_RECORD_SESSION` only in
  `session_log.rs` itself and the parked dormant vendored `tui/lib.rs`
  launcher, not the active temporary `app_shell` path or RunHaven adapters.
- Reduced `codex-core` now exposes `Config::model_provider_id` with the
  default `openai` id, derives known built-in ids from `model_provider`
  overrides, resolves built-in id-only overrides to their matching provider,
  rejects unknown id-only overrides, and preserves explicit custom provider ids
  only when a provider override is supplied. This keeps the upstream session-log
  source compiling without adding full Codex core, app-server transport, login,
  MCP, filesystem, hooks, tools, rollout, state, or thread-store behavior.
- Full `status/`, native `App`, `ChatWidget`, real `app_server_session`,
  app-server transport, filesystem RPC, MCP, login, workspace command
  execution, and other host-reaching Codex paths remain dormant or
  fail-closed. Codex session recording initialization is not active; before any
  native `App` startup promotes it, RunHaven needs a reviewed env/path and
  redaction policy.
- Verified so far:
  baseline `cargo test -p runhaven-tui --locked`,
  fail-first
  `cargo test -p runhaven-tui --locked session_log_uses_source_first_boundary_without_active_recording -- --show-output`,
  green rerun of the same guard,
  `cargo check -p runhaven-tui --locked`,
  `cargo fmt --check`,
  focused `codex-core` config tests,
  `cargo test -p runhaven-tui --locked drift_tests -- --show-output`,
  `cargo test -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `cargo test -p codex-core --locked`,
  `scripts/compare-codex-tui.sh`,
  `python3 -m json.tool feature_list.json >/dev/null`, snap-new scan, and
  `git diff --check`.

Latest TUI MVP workspace picker:

- 2026-06-29: Added the missing MVP workspace picker as Step 1 inside the
  existing BottomPane-owned `LaunchWizardView`, without adding product screens
  to `app_shell.rs`. `RunHavenTuiService` now offers the current directory plus
  the git repository root when the selected directory is nested inside a repo.
  Selecting the git root rebuilds the agent preview list for that workspace
  before review, so the mounted `/workspace` path and exact command match the
  chosen workspace.
- Security boundary is unchanged: the picker only changes the validated
  workspace path sent to the existing planner. It does not mount host home,
  credentials, SSH keys, browser profiles, cloud credential folders, arbitrary
  host environment variables, or activate launch execution. Foreground launch
  remains disabled after confirmation.
- Verified so far:
  red compile failures for missing workspace-choice API and wizard path,
  `cargo test -p runhaven-tui --locked launch_workspace_choices_offer_current_and_git_root_for_nested_repo -- --show-output`,
  `cargo test -p runhaven-tui --locked workspace_picker_selects_git_root_before_agent_review -- --show-output`,
  `cargo test -p runhaven-tui --locked launch_wizard -- --show-output`,
  `cargo test -p runhaven-tui --locked service -- --show-output`,
  `cargo test -p runhaven-tui --locked app_shell -- --show-output`,
  `cargo fmt --check`,
  `cargo test -p runhaven-tui --locked`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `scripts/compare-codex-tui.sh`, JSON validation, snap-new scan, and
  `git diff --check`.

Latest TUI foreground launch handoff:

- 2026-06-29: Confirmation now carries a typed RunHaven `PreparedLaunch`:
  display-only `LaunchPlanData` plus the original executable `AgentRunPlan`.
  `LaunchWizardView` emits `AppEvent::RunHavenLaunchPrepared` with that value,
  and the staging shell exits the draw loop with `ShellExit::Launch` instead of
  starting work from the widget.
- `runhaven/launch_handoff.rs` is the only active TUI file allowed to call
  `runhaven_core::runtime::launch::launch_run_plan`. It clears TUI-owned title
  and terminal image state, clears the TUI screen before handoff, calls Codex
  `Tui::with_restored(RestoreMode::Full, ...)`, and then launches from the
  stored `AgentRunPlan`. It does not reconstruct execution from
  `LaunchPlanData.command`.
- Security boundary is unchanged for Codex host-reaching surfaces:
  app-server transport, filesystem RPC, MCP, login, workspace command
  execution, and Codex session recording remain dormant or fail-closed. The
  full RunHaven launch intent remains excluded from Codex session logging until
  RunHaven owns a redaction policy.
- Verified so far:
  baseline `cargo test -p runhaven-tui --locked`,
  red compile failures for the missing handoff owner, missing
  `ShellAction::Launch`, and display-only `LaunchPlanData` still being
  stored in shell state,
  `cargo test -p runhaven-tui --locked shell_confirm_enter_requests_foreground_launch_handoff -- --show-output`,
  `cargo test -p runhaven-tui --locked secure_plan_confirm_enter_prepares_foreground_launch_handoff -- --show-output`,
  `cargo test -p runhaven-tui --locked prepared_launch_handoff_restores_terminal_before_launcher -- --show-output`,
  `cargo test -p runhaven-tui --locked prepared_launch_handoff_uses_executable_plan_not_display_command -- --show-output`,
  `cargo test -p runhaven-tui --locked foreground_runtime_launch_call_stays_in_ui_thread_handoff_owner -- --show-output`,
  `cargo test -p runhaven-tui --locked launch_wizard -- --show-output`,
  `cargo test -p runhaven-tui --locked service -- --show-output`,
  `cargo test -p runhaven-tui --locked app_shell -- --show-output`,
  `cargo test -p runhaven-tui --locked runhaven::app_server_client -- --show-output`,
  and full `cargo test -p runhaven-tui --locked --quiet` coverage for typed
  confirmation handoff, display-only protocol notification, session-log
  exclusion, and `PreparedLaunch` display/executable consistency.
  Final gate passed:
  `cargo fmt --check`,
  `cargo check -p runhaven-tui --locked`,
  `cargo test -p runhaven-tui --locked --quiet` (761 passed, 5 ignored),
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`,
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins --quiet`,
  `scripts/compare-codex-tui.sh`,
  `python3 -m json.tool feature_list.json >/dev/null`, snap-new scan, and
  `git diff --check`.

Latest TUI active-run log snapshot seam:

- 2026-06-29: Added the first active-run transcript/log backing route without
  adding product UI to `app_shell.rs`. `runhaven/run/logSnapshot` now flows
  through the local RunHaven protocol, in-process client, service, and session
  bridge into `runhaven_core::runtime::active::active_run_log_snapshot_payload`.
  The shared UI contract exposes `ActiveRunLogSnapshotData` as camelCase display
  data converted from the existing core snake_case payload.
- Guard posture: only `runhaven/service.rs` may call the active-run log snapshot
  runtime API, raw-output requests require explicit confirmation before
  validation or backend lookup, malformed line counts fail validation before
  active-run or container log lookup, and `app_shell.rs` must not grow active-run
  log product behavior. This does not activate native `App`, `ChatWidget`,
  app-server transport, filesystem RPC, MCP, login, workspace command execution,
  Codex session recording, or host-reaching Codex execution. Future visible log
  UI must add a redaction and session-recording policy before it renders,
  caches, or logs raw container output.
- Verified:
  red compile failures for the missing `ActiveRunLogSnapshotData`,
  `RunHavenRunLogSnapshot`, and `AppServerSession::run_log_snapshot` APIs;
  `cargo test -p runhaven-core --locked active_run_log_snapshot_contract -- --show-output`;
  `cargo test -p runhaven-tui --locked run_log_snapshot_request_uses_runhaven_method -- --show-output`;
  `cargo test -p runhaven-tui --locked log_snapshot_requires_sensitive_output_confirmation_before_backend_lookup -- --show-output`;
  `cargo test -p runhaven-tui --locked run_log_snapshot_requires_sensitive_output_confirmation_before_backend_lookup -- --show-output`;
  `cargo test -p runhaven-tui --locked log_snapshot_rejects_invalid_line_count_before_backend_lookup -- --show-output`;
  `cargo test -p runhaven-tui --locked active_run_log_snapshot_route_stays_in_runhaven_facade -- --show-output`;
  `cargo check -p runhaven-core --locked`;
  `cargo check -p runhaven-tui --locked`;
  `cargo test -p runhaven-core --locked --quiet` (78 passed);
  `cargo test -p runhaven-tui --locked --quiet` (767 passed, 5 ignored);
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run`;
  `cargo clippy -p runhaven-core --all-targets --locked -- -D warnings`;
  `cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings`;
  `cargo run --locked --bin runhaven-check-pins --quiet`;
  `scripts/compare-codex-tui.sh`;
  `cargo fmt --check`;
  `python3 -m json.tool feature_list.json >/dev/null`;
  snap-new scan;
  `git diff --check`;
  codex-tui-expert review, adversarial-reviewer review, and rust-expert
  re-review after the sensitive-output confirmation fix.

Latest RunHaven-only TUI MVP surface:

- 2026-06-29: Added the RunHaven-owned MVP root view in
  `crates/runhaven-tui/src/tui/runhaven/mvp.rs` and made the staging shell host
  that view inside the real vendored `BottomPane`. The active TUI now covers
  the scoped MVP path: workspace picker, agent picker, policy changes for
  network mode and auth scope, plan review, typed launch confirmation,
  foreground launch handoff, post-run recovery back into the TUI, active-run
  list, confirmation-gated raw log snapshot display, and secret-free diagnostics
  for auth broker status/log plus provider egress decisions.
- 2026-06-29: Simplified the initial agent chooser for non-technical users.
  The chooser now shows plain guidance, agent descriptions, workspace, current
  network mode, and the short `/workspace only` safety summary. It no longer
  renders the side `Plan Preview`, exact `container run` command, provider-host
  list, or broker/image/auth detail on the first screen. Review and confirm
  still show auth scope, network posture, not-shared host data, provider hosts,
  safety notes, and the exact command before launch.
- 2026-06-29: Removed the active `RUNHAVEN_TUI_IMAGE_SMOKE` hook from the live
  `app_shell.rs` path so core TUI completion stays free of Cubby/pet polish
  scope. The bundled Cubby package and lower pet/image modules remain parked
  for final-pass work. Source scan confirms no active image-smoke symbols remain
  in `app_shell.rs` or the RunHaven TUI view modules.
- Shared UI contracts now include `ActiveRunListData` and
  `RunHavenDiagnosticsData`. Active-run summaries intentionally omit workspace
  paths, diagnostics map only metadata fields, auth broker request paths are
  scrubbed of query strings and fragments at producer, reader, and UI-contract
  boundaries, and raw container output remains hidden until the user types
  `logs`. Raw log text is kept in live view state only; the active path still
  does not initialize Codex session recording. Post-run recovery preserves the
  effective launch workspace and selected policy, and TUI diagnostics use
  bounded tail reads for log files.
- Guard posture: direct container log backend access still belongs only to
  `runhaven/service.rs`; visible raw-log rendering is guarded to
  `runhaven/mvp.rs`; `app_shell.rs` only owns terminal runtime, foreground
  launch handoff/recovery routing, and process exit-code tracking. Native
  `App`, `ChatWidget`, full `status/`, real app-server transport, filesystem
  RPC, MCP, login, workspace command execution, Codex session recording
  initialization, and host-reaching Codex execution remain dormant or
  fail-closed.
- Verified: `cargo test -p runhaven-core --locked ui_contracts --quiet`;
  `cargo test -p runhaven-core --locked --quiet` (84 passed);
  `cargo test -p runhaven-tui --locked runhaven::mvp -- --nocapture`;
  `cargo test -p runhaven-tui --locked runhaven::service -- --nocapture`;
  `cargo test -p runhaven-tui --locked runhaven::app_server_session --
  --nocapture`; `cargo test -p runhaven-tui --locked app_shell --
  --nocapture`; `cargo check -p runhaven-tui --locked`;
  `cargo test -p runhaven-tui --locked drift_tests -- --show-output`;
  `cargo test -p runhaven-tui --locked --quiet` (781 passed, 5 ignored);
  `cargo test -p runhaven-tui --locked --features codex-vendored-tests
  --no-run`; `cargo clippy -p runhaven-core --all-targets --locked --
  -D warnings`; `cargo clippy -p runhaven-tui --all-targets --locked --
  -D warnings`; `cargo run --locked --bin runhaven-check-pins --quiet`;
  `cargo test -p runhaven-tui --locked launch_wizard -- --nocapture`;
  `cargo test -p runhaven-tui --locked runhaven::mvp -- --nocapture`;
  `scripts/compare-codex-tui.sh` (372 RunHaven files, 16 RunHaven-only files,
  53 copied Codex files with local edits); `cargo fmt --check`;
  `python3 -m json.tool feature_list.json >/dev/null`; snap-new scan; and
  `git diff --check`; local Rust/Codex/adversarial review of this cleanup found
  no blocker.

## Blockers

- SSH forwarding remains fail-closed as described above.

## Next Step

The RunHaven-only TUI MVP surface is now present in the staging Codex runtime.
Next TUI work should be core completion, cleanup, and hardening, not final
polish or new product scope. Defer Cubby/pet polish, mascot work, terminal image
polish, and Zork until the end. Reduce module-path debt, decide whether native
`App`/`ChatWidget` ownership is still needed for RunHaven after the MVP root
view, and keep unrelated Codex product features dormant, fail-closed, or
deleted. If native `App` startup is promoted later, first replace raw Codex
env/path session-recording behavior with a reviewed RunHaven redaction boundary.
