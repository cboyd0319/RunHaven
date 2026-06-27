# Current State

Last updated: 2026-06-27 UTC

## Current Objective

The active slice is the TUI Codex vendor reset plus the Rust workspace
normalization needed to make that TUI a clean reference implementation.

RunHaven is replacing its previous custom TUI with the local Codex TUI source
baseline from:

```text
/Users/c/Documents/GitHub/codex/codex-rs/tui/src/
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
- TUI image and pet rendering must follow Codex source behavior. Use the local
  Codex TUI source and local Codex config evidence before writing custom pet,
  terminal image, statusline, bottom-pane, keymap, title, or resume behavior.
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
staging facade. The facade has the Codex editor/Vim keymap defaults and a tiny
local `codex_protocol::user_input` compatibility module for the byte-range text
element types used by the textarea. The upstream deterministic textarea tests
run by default; the snapshot and randomized stress tests remain opt-in with the
same `codex-vendored-tests` policy as the other upstream snapshot goldens.

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

## Blockers

- SSH forwarding remains fail-closed as described above.

## Next Step

Continue TUI integration from the Codex-vendored source baseline. Keep using
source-first Codex modules for app shell, bottom pane, status line, native pet,
resume/session, keymap, tooltips, and terminal-title behavior before writing
custom RunHaven TUI code. The next practical slice is the Codex terminal
runtime/event-stream alignment plus the typed app-server facade pattern for
planner, launch, status, interrupt, history, diagnostics, and session calls.
Treat this as a Strategy C Codex-compatible-client slice, not a permanent
Strategy B widget-kit extraction. After that, adapt the native
`App`/`ChatWidget`/`BottomPane` path and then wire real launch execution from
the confirmation step once the command path stays planner-owned and inspectable.
Feed RunHaven-specific surfaces from the shared `RunHavenComponentPayload`
contracts instead of ad hoc screen structs.
