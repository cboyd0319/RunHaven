# Current State

Last updated: 2026-06-27 UTC

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
- TUI implementation slices should end with the combined `rust`, Persona
  Codex TUI skill
  (`/Users/c/Documents/GitHub/persona/content/skills/codex-tui`), and
  `adversarial-review` skill gate before commit: Rust crate/tooling
  correctness, Codex source-pattern alignment, then boundary and overclaim
  review. The repo-local `.agents/skills/codex-tui` wrapper exists only to make
  that Persona skill discoverable from this project.
- Antigravity (`agy`) is research-only in this repo. Do not use it for
  end-of-slice code review, adversarial review, verification, or proof of
  correctness.
- User-facing writing is product behavior. UI text, menus, prompts, warnings,
  README/usage docs, and setup instructions target non-technical users at about
  an 8th grade reading level.
- The hidden Zork I easter egg remains wanted. The current reset keeps the
  MIT-licensed `historicalsource/zork1` collection under `third_party/zork1/`.
  The earlier Ferrif-derived TUI engine was removed with the old custom TUI and
  is recoverable from git history. If reintroduced, it must stay TUI-local,
  attributed, offline, and carefully validate save/restore files.
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

TUI native-pet image smoke follow-up: the temporary `app_shell.rs` can now run
an opt-in visual check with `RUNHAVEN_TUI_IMAGE_SMOKE=1`. RunHaven now bundles
the verified Cubby Codex pet package from `docs/assets/installed-pet/cubby/`
and materializes it as `custom:runhaven-cubby` under
`$CODEX_HOME/pets/runhaven-cubby/` before calling Codex's vendored
`AmbientPet`, frame cache, Tokio `FrameRequester`, and
`render_ambient_pet_image` writer. This keeps the renderer source-first while
avoiding collisions with a user's own `$CODEX_HOME/pets/cubby/` package. The
smoke path is only for checking terminal image quality before the full Codex
app shell and bottom pane are adapted.

TUI component-seam follow-up: `crates/runhaven-core/src/ui_contracts.rs` now
defines the first tagged RunHaven payload enum with `AgentCatalogData` and
`LaunchPlanData`; `LaunchPlanData` includes the planner's auth scope so the TUI
does not guess whether login state is agent-wide or project-scoped. Fixtures live under
`crates/runhaven-core/tests/fixtures/ui/`. The temporary TUI adapter consumes
`AgentCatalogItemData` for agent display, but the next visual slice should move
toward a Codex-native shell with RunHaven product cards. dbt-wizard is only the
architecture proof for stable domain payloads first and renderer second. The
visual target is closer to native Codex: compact intro and status content,
bottom composer and status line, native Cubby behavior, and no analytics
dashboard feel in the default launcher.

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
`LaunchPlanData` into decision rows, a safety header, and a plan preview that
keeps boundary, host home, credentials, auth scope, network mode, and exact
command visibility near the top. Enter still does not launch.

TUI launch-review follow-up: Enter on a ready agent now opens a read-only review
step rendered through the Codex menu-surface style. The review shows the
selected agent, auth scope, network posture, workspace mount, state volume,
non-shared host data, provider hosts, safety notes, and exact `container run`
command. `b`, backspace, or Esc returns to the picker; `q` exits from either
screen. Blocked plans cannot open review. Launch and preflight execution remain
disabled in the TUI.

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
and keep `q` as the shell quit shortcut. This remains a read-only preview:
confirmation shows an
acknowledgement, but the TUI does not start containers, run preflight commands,
or write launch state yet.

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
- `cargo test -p runhaven-tui --locked app_shell::tests::shell_confirm_enter_shows_disabled_launch_notice_without_launching -- --nocapture`
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
  vendor audit: 894 upstream files, 369 RunHaven files, 356 common paths, 538
  upstream `.snap` files external by default, 13 RunHaven-only files, and 26
  copied Codex files with local edits.

Latest Codex config/keymap crate vendoring:

- 2026-06-27: Continued real `codex-*` crate vendoring under original package
  and library names. Added workspace members for `codex-config`,
  `codex-api`, `codex-client`, `codex-features`, `codex-file-system`,
  `codex-git-utils`, `codex-model-provider-info`, `codex-otel`, and
  `codex-utils-path`, plus their local manifest wiring. `runhaven-tui` now
  depends on the real vendored `codex-config` crate, `lib.rs` no longer aliases
  `codex_config`, and the file-backed vendored `keymap.rs` compiles against
  `codex_config::types::{KeybindingsSpec, TuiKeymap, MAX_FUNCTION_KEY}` instead
  of an inline RunHaven keymap extract. The only remaining `codex_*` self-alias
  is `codex_terminal_detection`, which stays temporary until the terminal
  detection crate is promoted.
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
- Verified so far:
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
- Verified so far:
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

## Blockers

- SSH forwarding remains fail-closed as described above.

## Next Step

Continue TUI integration from `docs/plans/codex-tui-strategy-c/` with Phase 4:
adapt the native `App` and `BottomPane` path. Foreground launch remains
read-only until the native Codex app loop owns terminal restore and
`launch_run_plan` is wired through the UI thread. The next vendor-first slice is
to promote `bottom_pane/mod.rs` after either replacing `app_event_shared.rs`
with real shared modules or proving which inert bridge types can remain until
ChatWidget activation. Do not route host-reaching Codex behavior around the
RunHaven facade.
