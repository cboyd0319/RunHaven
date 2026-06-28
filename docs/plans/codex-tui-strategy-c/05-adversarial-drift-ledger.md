# 05 Adversarial Drift Ledger

Independent adversarial audit of the RunHaven TUI against the Strategy C
vendor-first plan.

- Audit date: 2026-06-27
- Auditor scope: `crates/runhaven-tui/`, the vendored Codex baseline, and
  RunHaven state docs.
- Baseline for drift: local Codex checkout `5267e805fb830891c0b23376bcd9cbd382c3473c`,
  verified to be exactly the commit pinned in
  `crates/runhaven-tui/src/tui/README.md` (0 commits ahead). Every "modified"
  finding below is a RunHaven edit, not upstream drift.

This document is a counterweight to the repo's own self-reported status. The
repo tracks its progress honestly (see Reconciliation, below); this ledger
exists to name what that tracking understates or leaves implicit, with specific
evidence.

Correction applied after audit: the repo copy now restores the canonical
Strategy C phase order. Runtime-spine compile and terminal-handoff proof are
completed Phase 3 gates. Native `App` and `BottomPane` adaptation is Phase 4,
not Phase 5. Findings below still describe the same drift.

---

## Bottom Line

RunHaven has **not** abandoned vendor-first. Measured against the exact pinned
commit, 329 of 355 vendored Rust files are byte-identical and the new
RunHaven-owned facade (`runhaven/protocol.rs`, `runhaven/service.rs`) is a
faithful, secure, Codex-shaped implementation of the plan's boundary. The
security boundary is intact in everything actually compiled.

The drift is real but **early-phase and structural**, not a betrayal of intent:

1. The Codex runtime the whole plan is built around (`App` -> `ChatWidget` ->
   `BottomPane`) is **not wired**. The live app is still `app_shell.rs`, a
   1050-line hand-rolled shell running its own `ratatui::try_init()` loop, which
   is the plan's single most-named anti-goal.
2. `mod.rs` is still a hand-authored compatibility layer, now reduced to 474
   lines after the keymap shim was deleted. It still shadows full-size vendored
   modules with tiny local reimplementations (`app_event`, `app_event_sender`,
   `bottom_pane`, `render`, `status`, and `clipboard_paste`). This is exactly
   the "staged compatibility definitions = debt" that plan rule 6 warned about,
   now shrinking but not gone.
3. The first real `codex-*` crate authority has now been restored:
   `codex-protocol`, `codex-app-server-protocol`, `codex-config`,
   `codex-connectors`, `codex-file-search`, `codex-plugin`, and their required
   utility/config/event-data closure are vendored under original
   package/library names in `crates/codex/`. Remaining drift is narrower:
   `lib.rs` still aliases `codex_terminal_detection`, and `mod.rs` still
   carries staged stand-ins for some TUI modules that have not yet been
   activated.
4. 0 of 538 upstream snapshots were copied. Documented and defensible, but it
   means the vendored test goldens provide no regression signal inside the repo.

Net: the project is at the end of the Phase 3 runtime/handoff gate. The
foundation is faithful; the divergence is that the **temporary scaffolding has
grown product weight** (`app_shell.rs` 1050 lines, `launch_wizard.rs` 1542
lines, `mod.rs` 474 lines) while the Codex app loop it is supposed to be
replaced by does not yet exist. The longer that scaffolding carries real UX, the
harder the Phase 4 swap to the Codex `App` becomes.

Drift rating against the plan's **end state**: substantial (early). Drift
against the plan's **method**: moderate, concentrated in `mod.rs` and the live
shell.

---

## Critical Context: There Were Two TUIs

This reframes "how far have we strayed." `feature_list.json` records the full
history:

1. **Custom TUI (abandoned).** RunHaven first built a complete bespoke TUI:
   Slice 1 scaffold, Slice 2 agent picker, a hand-built Cubby mascot, Phases 0-5
   (foundation, brand, launcher flow, run management, history/diagnostics,
   polish), a four-step launch wizard, run dashboard, bounded log viewer,
   stop/kill/repair screens, history/diff, diagnostics/doctor, and a Zork easter
   egg. This was the original straying from vendor-first.
2. **Vendor-reset pivot (2026-06-27).** RunHaven "replaced the custom
   `crates/runhaven-tui/src/tui` tree with a snapshot of upstream
   `openai/codex:codex-rs/tui/src`" and began re-integrating piece by piece.

Strategy C is the **correction plan** for the first straying. The current state
is the early execution of that correction. So the question is not "did they
stray from Codex" (they reset back to it) but "is the re-integration following
the plan's method." Mostly yes, with the structural exceptions below.

A residue of the abandoned custom TUI survives in the current scaffolding:
`app_shell.rs` and `launch_wizard.rs` re-implement the custom TUI's wizard UX
(four-step picker/review/confirm) rather than waiting for Codex `App`/
`ListSelectionView`. That residue is the main vector of drift.

---

## Reconciliation: Two Plan Copies Diverged

At audit time, the plan existed in two places and was **not** in sync:

| File | Downloads copy (this folder) | In-repo `docs/plans/codex-tui-strategy-c/` |
| --- | --- | --- |
| README.md | baseline | +20/-11 |
| 01-source-inventory.md | baseline | +25/-22 |
| 02-vendoring-and-protocol.md | baseline | +28/-16 |
| 03-runtime-wiring.md | baseline | +15/-7 |
| 04-implementation-and-verification.md | baseline (Phases 0-7) | **+211/-46** (Phases 0-8, with per-phase Status) |

The repo copy inserted two phases the Downloads copy lacked ("Phase 3: Compile
The Dormant Runtime Spine", "Phase 4: Prove Terminal Handoff"), renumbered
App/BottomPane to Phase 5, ChatWidget to 6, product screens to 7, cull to 8,
and added explicit `Status:` blocks marking Phases 0-4 complete.

> Finding R1 (severity: low, process): Decide which copy is canonical and
> redirect the other, or the next reviewer audits against the wrong target.

Applied fix: `docs/plans/codex-tui-strategy-c/` is the repo-canonical copy and
has been realigned to the original phase order. Runtime compile and handoff are
recorded as completed Phase 3 gates. Everything from "Phase 4: Adapt App and
BottomPane" onward is unstarted.

---

## Phase Status

| Phase | Plan intent | Actual state | Evidence |
| --- | --- | --- | --- |
| 0 Lock vendor baseline | README + compare script + file ledger | **Done** | `tui/README.md`, `scripts/compare-codex-tui.sh` |
| 1 Stop growing temp shell | move core calls into a service seam | **Done** | `app_shell.rs` has zero `runhaven_core::` refs; goes through `runhaven/service.rs` |
| 2 Codex-shaped backend facade | client/protocol/service, typed errors, bounded channel | **Done (minimal)** | `runhaven/{protocol,service,app_server_client,app_server_session}.rs`; 3 active methods only |
| 3 Switch to Codex terminal runtime | `tui.rs` compiles, handoff is proven | **Done as supporting gate** | `tui.rs` as `codex_runtime`; `runhaven/terminal_handoff.rs`, env-gated |
| 4 Adapt `App` + `BottomPane` | real Codex event loop active | **Not started** | `app.rs`/`bottom_pane/mod.rs` dormant; `app_shell.rs` is live shell |
| 5 Adapt `ChatWidget` | transcript/status/history cells | **Not started** | `chatwidget.rs` dormant |
| 6 Reattach product screens | dashboard, logs, history, diff, doctor | **Not started** | none wired; only launch preview exists |
| 7 Cull/stub unsupported surfaces | decide each dormant surface | **Not started** | dormant surfaces sit marker-guarded on disk (see D8) |

---

## Vendor Fidelity Scorecard (what is going right)

These are not throwaway compliments; they bound the size of the problem.

- **F1. Baseline is complete and measurable.** All 355 upstream Rust files are
  present. 329 are byte-identical to the exact pinned commit. The compare tool
  exists and is documented. Upstream diffability is genuinely preserved for the
  dormant majority.
- **F2. The facade boundary is honored where it exists.** `app_shell.rs` makes
  zero direct `runhaven_core::` calls; all domain data flows through
  `AppServerSession::start_in_process(RunHavenTuiService::new())`
  (`app_shell.rs:319-347`). Rule 4 ("widgets draw data") holds in the live path.
- **F3. The protocol facade is genuinely Codex-shaped.**
  `runhaven/protocol.rs` mirrors `AppServerEvent { Lagged, ServerNotification,
  ServerRequest, Disconnected }`, implements the lossless-vs-best-effort
  `requires_delivery()` split (`protocol.rs:170-193`), and enumerates the exact
  13 fail-closed families from doc 03.
- **F4. Unsupported surfaces fail closed, not half-wired.** Every family in
  `UnsupportedMethod::ALL` returns `TypedRequestError::Unsupported`; the test
  `unsupported_method_matrix_fails_closed` iterates all 13
  (`app_server_client.rs:833-859`). There is no partial fs/mcp/plugin/account
  code path.
- **F5. Mutating actions route through the validated core path.**
  `service.rs:167-191` builds plans via
  `runhaven_core::runtime::plans::build_run_plan` with secure defaults
  (`allow_root_user: false`, `allow_sensitive_workspace: false`, `user:
  "agent"`). Nothing executes a plan in the wired code.
- **F6. Terminal handoff uses the real Codex restore sequence.**
  `terminal_handoff.rs:65-69` calls
  `tui.with_restored(RestoreMode::Full, ...)` then `restore_after_exit()`, runs
  only a harmless exact-argv child (`/usr/bin/printf`), and is env-gated. This
  is the launch-safety mechanism the plan requires, proven before real launch.
- **F7. Security-relevant edits are documented.** The compare script now reports
  43 modified vendored files. `tui/README.md` records the known source-format,
  integration, and test-golden exceptions; keep it current as each new
  vendored file is edited.

---

## Drift Findings

Each finding: severity, the plan rule it bends, evidence, risk, recommendation.

### D1 - The live app is `app_shell.rs`, not Codex `App`. (Severity: High)

- Plan rule: README "Do not let `app_shell.rs` and the staged `mod.rs` facade
  become the permanent architecture." Doc 03 target flow is `Tui -> App ->
  ChatWidget -> BottomPane`. Phase 3/4 say host the picker "inside the Codex
  runtime."
- Evidence: `app_shell.rs:44` `let mut terminal = ratatui::try_init()?;`;
  hand-rolled poll/read loop at `app_shell.rs:58-94` (`event::poll`,
  `event::read`, `terminal.draw`). The vendored Codex `Tui`/`custom_terminal`/
  `event_stream` spine **compiles** (`mod.rs:292-294` `codex_runtime`) but
  drives nothing. `mod.rs:298-304` `run()` ends in `app_shell::run()`.
- Risk: the runtime the entire plan is organized around is bypassed by a
  parallel loop. Every screen added to `app_shell.rs` now has to be rebuilt
  against `App` later. The plan explicitly predicted this trap.
- Recommendation: do not add further screens to `app_shell.rs`. Make Phase 4
  (wire `App` + `BottomPane`, host the existing picker inside it) the next
  slice, before any dashboard/history/logs work.

### D2 - `mod.rs` shadows full-size vendored modules with hand-rolled stand-ins. (Severity: High)

- Plan rule: Phase 0 "Keep `mod.rs` marked temporary"; rule 6 "Keep `mod.rs`
  shrinking. If it gains more staged compatibility definitions, that is debt and
  should be called out." Doc 02 wanted vendored `codex-*` crates, not local
  reimplementations.
- Evidence:
  - `app_event.rs` and `app_event_sender.rs` now compile as real vendored
    files, not inline stand-ins.
  - `app_event_shared.rs` is temporary leaf-type bridge debt for shared types
    whose owning modules remain dormant.
  - `mod bottom_pane`, `mod render` Insets/RectExt,
    `mod status::format_tokens_compact`, and `mod clipboard_paste` one-fn are
    still local stand-ins shadowing same-named vendored files.
  - `keymap.rs` is no longer a local inline extract. It is declared as a real
    file-backed vendored module and compiles against the real vendored
    `codex-config` crate.
- Risk: these stand-ins have **smaller, different shapes** than upstream. When
  Phase 4+ activates the real `app_event.rs`/`bottom_pane/mod.rs`,
  every stand-in must be deleted and every consumer rewired. The vendored files
  they shadow cannot be `diff`-merged against a stand-in. This is debt that
  grows with every newly-activated file, exactly as rule 6 predicted.
- Recommendation: treat each stand-in as a tracked debt item with a named
  upgrade path (which vendored module replaces it). Prefer activating the real
  vendored module over extending a stand-in.

Applied fixes after audit: `codex_protocol::user_input` is no longer an inline
`mod.rs` shim or a RunHaven-local staged leaf. `runhaven-tui` now depends on the
real vendored `codex-protocol` crate, and `TextArea` consumes
`codex_protocol::user_input::{ByteRange, TextElement}` from that crate.
The inline keymap extract is gone; `keymap.rs` now compiles against the real
vendored `codex-config` crate and `lib.rs` no longer aliases `codex_config`.
`mod.rs` has guard tests that prevent new inline stand-ins or new `codex_*`
self-aliases from appearing quietly. `app_event.rs` and `app_event_sender.rs`
are now real vendored modules. The remaining D2 debt is `app_event_shared.rs`,
`bottom_pane`, `render`, `status`, and `clipboard_paste`.

### D3 - Codex crate vendoring is now real; broader crate activation remains partial. (Severity: Low/Medium)

- Plan rule: doc 02 "Keep Codex TUI source layout and vendor as many Codex
  crates as practical with their original crate names ... This lets many
  upstream TUI imports remain unchanged" (`use codex_app_server_protocol::...`).
- Evidence: `crates/codex/` now contains real vendored packages for
  `codex-protocol`, `codex-app-server-protocol`, `codex-config`,
  `codex-connectors`, `codex-file-search`, `codex-plugin`,
  `codex-utils-approval-presets`, and the first dependency closures needed by
  protocol, config, keymap, and event-data activation.
  `runhaven-tui/Cargo.toml` depends on
  `codex-protocol = { path = "../codex/protocol" }` and
  `codex-app-server-protocol = { path = "../codex/app-server-protocol" }`,
  plus `codex-config = { path = "../codex/config" }`,
  `codex-connectors = { path = "../codex/connectors" }`,
  `codex-file-search = { path = "../codex/file-search" }`,
  `codex-plugin = { path = "../codex/plugin" }`, and
  `codex-utils-approval-presets = { path = "../codex/utils/approval-presets" }`.
  `cargo check -p codex-protocol`, `cargo check -p codex-app-server-protocol`,
  `cargo check -p codex-config`, `cargo check -p codex-plugin --locked`,
  `cargo check -p codex-file-search --locked`,
  `cargo check -p codex-connectors --locked`, and
  `cargo check -p runhaven-tui --locked` all pass.
- Integration adjustment: vendored manifests keep Codex package/library names
  and Apache-2.0 metadata. External exact pins that conflict with RunHaven's
  unified workspace resolver are aligned to RunHaven's existing pins, while the
  upstream `runfiles`, `tokio-tungstenite`, and `tungstenite` git revs are
  preserved for the same source behavior Codex relies on. RunHaven also carries
  the upstream Codex `tokio-tungstenite` and `tungstenite` crates.io patches,
  and pins `codex-exec-server` to upstream Codex's `axum` 0.8.8 to avoid a
  duplicate registry websocket stack. This is manifest integration drift, not
  source/API drift.
- Remaining risk: the alias for `codex_terminal_detection` remains, and larger
  backend crates such as `codex-app-server` and `codex-core` are not active
  authorities. Activating native `App`, `BottomPane`, or `ChatWidget` may expose
  the next crate authority gap.
- Recommendation: keep advancing crate authority before adding more local
  stand-ins. When a copied TUI module expects a `codex-*` crate, vendor the real
  crate or record the specific security reason for a temporary local boundary.

### D4 - Zero of 538 upstream snapshots copied. (Severity: Medium)

- Plan rule: doc 01/04 "Preserve upstream snapshots and fixtures as reference
  material" and "Add a compare command." RunHaven snapshots should be
  regenerated, not imported wholesale.
- Evidence: `find ... -name '*.snap'` = 538 upstream, 0 in RunHaven. Documented
  in `tui/README.md` ("upstream `*.snap` files ... stay external"). The compare
  script fetches them on demand.
- Risk: inside the repo there is no snapshot regression signal for any vendored
  rendering behavior. The plan accepted external snapshots **if** RunHaven
  regenerates its own; today RunHaven has 0 authoritative snapshots for the
  wired surface either (the live `app_shell` UX is not snapshotted in this
  crate). So the rendering surface is currently untested by golden.
- Recommendation: generate RunHaven snapshots for the live launch-preview
  screens now (cheap, the picker/review/confirm are stable), so Phase 4's swap
  has a before/after reference.

### D5 - `launch_wizard.rs` is a bespoke wizard, not a `ListSelectionView` reuse. (Severity: Medium)

- Plan rule: doc 01 "make it a normal view launched by Codex `App`"; Phase 6
  "Agent picker: `ListSelectionView`"; rule 10 "reuse `BottomPaneView` ...
  before adding a custom rendering loop."
- Evidence (`launch_wizard.rs`, 1542 lines): now implements `BottomPaneView`,
  but remains a 3-screen state machine (`ChooseAgent`/`ReviewPlan`/
  `ConfirmLaunch`, `:71-76`) that hand-builds two full-screen renderers
  (`ReviewPlan` `:628-713`, `ConfirmLaunch` `:715-886`) and a typed-confirmation
  composer rather than reusing `ListSelectionView`. ~1200 non-test lines for a
  "view-model mapper."
- Sub-findings:
  - Hardcoded boundary text (`"Host home" / "not mounted"`, `:586-590`,
    `:681-682`, `:759-760`, `:927-928`) restates trust-boundary claims as static
    strings instead of rendering the core `boundary` contract. If the real
    boundary changes, the UI text will not.
  - Re-derives `network_mode_label` (`:1179-1186`) when the contract already
    supplies `plan.network.summary`. Duplication against the source of truth.
- Risk: a large bespoke renderer that Phase 4/6 wants owned by Codex child
  views; and security text that can silently desync from the real boundary.
- Recommendation: drive boundary/network text from the `LaunchPlanData`
  contract, not literals. Treat the review/confirm screens as Codex child views
  to be migrated, not extended.

### D6 - Parallel `runhaven/app_server_session.rs` shadows the dormant Codex facade. (Severity: Low/Medium)

- Plan rule: success criterion "`app_server_session.rs` remains the only typed
  backend facade used by `App` and `ChatWidget`." The plan meant the **vendored**
  `app_server_session.rs` (96KB) to be that facade.
- Evidence: vendored `src/tui/app_server_session.rs` (96334 bytes) is dormant
  and undeclared. The active facade is `runhaven/app_server_session.rs` (188
  lines), used only by `app_shell` (no `App`/`ChatWidget` exists yet).
- Risk: two files named `app_server_session.rs` with different shapes. When
  `App` is wired, a decision is forced: adapt the 96KB vendored facade, or
  promote the 188-line RunHaven one and accept permanent divergence from
  upstream's session surface. The README frames the small one as "the local
  Strategy C session bridge for this phase," which is honest but defers the
  collision.
- Recommendation: name the intended end state now. If the RunHaven bridge wins,
  say so in the plan and stop implying the vendored facade returns.

### D7 - `tui.rs` `with_restored` was structurally rewritten. (Severity: Low/Medium)

- Plan rule: doc 01 "Terminal lifecycle ... is hard to get right and should stay
  Codex-shaped." `tui.rs` is the highest-churn file (+302/-40, all RunHaven).
- Evidence: upstream's inline `with_restored` body was refactored into a
  `with_restored_terminal_session` helper over new `TerminalHandoffSession` /
  `TerminalHandoffOps` traits plus fakes and tests. Behavior is preserved (tests
  assert the same pause/restore ordering), but the hot path no longer matches
  upstream line-for-line.
- Risk: future upstream changes to `with_restored` will not `diff`-apply cleanly
  to this file. This is the "manually rediscover instead of diff" hazard the
  plan names, localized to one function.
- Recommendation: keep the trait seam if it is earning its tests, but pin a note
  in `tui/README.md` flagging `with_restored` as a known non-mergeable region so
  the next upstream sync expects a manual reconcile.
- Audit correction: an earlier pass speculated `terminal_palette.rs` "predates
  an upstream fix." That is wrong. The local Codex checkout is the exact pinned
  commit, so the `terminal_palette.rs` change is a deliberate RunHaven edit
  (documented: it substitutes the bounded probe because RunHaven did not adopt
  Codex's pinned crossterm fork), not upstream moving ahead.

### D8 - Dangerous vendored capabilities are dormant and marker-guarded. (Severity: Low/Medium, latent)

- Plan rule: security boundary requirements (no env passthrough, no host
  credential login, no clipboard subprocess without restore discipline) and
  Phase 7 "decide each dormant surface."
- Evidence: the active compiled surface is clean (no `container` exec, no host
  mounts, no env passthrough). But dormant on-disk vendored files carry live
  hazards: `app.rs` `std::env::vars().collect()` passthrough; `onboarding/auth.rs`
  `read_openai_api_key_from_env` + login + `webbrowser::open`; `clipboard_copy.rs`
  shell-out paths; `external_editor.rs` `EDITOR` exec. `tui/mod.rs` now has a
  guard test that fails if named risky modules are declared before their risky
  upstream markers are removed or fail-closed.
- Risk: the boundary is still partly enforced by omission from `mod.rs`, and
  the guard is marker-based. A promoted module can still need additional
  RunHaven security review beyond the current marker list.
- Recommendation: keep expanding the guard as each dormant surface is promoted,
  and require a security note when a host-reaching Codex surface becomes active.

### D9 - Minor upstream test-coverage erosion. (Severity: Low)

- Evidence: two `bottom_pane/list_selection_view.rs` tests are hard-disabled via
  `#[cfg(any())]`; an `insert_history.rs` test was rewritten off
  `render_markdown_text` to hand-built lines; two `insert_history` snapshot tests
  are gated behind `codex-vendored-tests` (documented). Net: a few upstream
  behaviors lost their in-repo assertion.
- Risk: small silent parity loss in exactly the widgets being actively edited.
- Recommendation: confirm each disablement was intentional and leave a one-line
  reason at the `#[cfg(any())]` site.

### D10 - Scaffolding mass vs. the thing it scaffolds. (Severity: Medium, trend)

- Evidence: `app_shell.rs` 1050 + `launch_wizard.rs` 1542 + `mod.rs` 474 +
  `runhaven/app_server_client.rs` 925 = 3991 lines of RunHaven-owned
  transitional code, versus a Codex `App` loop that is not yet wired at all.
- Risk: this is not a single bug; it is a trajectory. The plan's whole thesis is
  that the temporary layer shrinks toward the Codex shape. Right now it is
  growing. The `app_server_client.rs` at 925 lines in particular is large for a
  3-method in-process client.
- Recommendation: set an explicit ceiling. Before adding any product screen,
  Phase 4 must reduce `app_shell.rs`/`mod.rs`, not just add beside them.

---

## Modified Vendored Files (all 26, classification)

GLUE = benign integration (paths, Ratatui 0.30 compat, test gating). BEHAVIOR =
real logic change. MIXED = both. All are RunHaven edits (commit-exact baseline).

| File | Class | Note |
| --- | --- | --- |
| `tui.rs` | MIXED | path/Ratatui glue + `with_restored` trait refactor + pet-clear (D7) |
| `style.rs` | MIXED | path glue + net-new RunHaven security-boundary style fns |
| `terminal_palette.rs` | MIXED | path glue + substitutes bounded probe for crossterm focus-requery (documented) |
| `insert_history.rs` | MIXED | Ratatui 0.30 glue + one test rewritten off `render_markdown_text` |
| `app.rs` | BEHAVIOR | combined pet-image clear (`clear_pet_images`); sanctioned pet domain |
| `chatwidget/pets.rs` | BEHAVIOR | bundled-pet selector call; sanctioned pet domain |
| `pets/mod.rs` | BEHAVIOR | bundled Cubby; `DEFAULT_PET_ID` codex->cubby; sanctioned |
| `pets/picker.rs` | BEHAVIOR | Cubby picker entry; preselect idx 2->3; sanctioned |
| `app/pets.rs` | GLUE | call-site rename |
| `bottom_pane/footer.rs` | GLUE | test gated behind `codex-vendored-tests` |
| `bottom_pane/list_selection_view.rs` | GLUE | comments; 2 tests disabled `#[cfg(any())]` (D9) |
| `bottom_pane/mod.rs` | GLUE | re-export `render_menu_surface` |
| `bottom_pane/textarea.rs` | GLUE | real vendored `codex_protocol` crate import; `#[path=vim.rs]`; test gates |
| `custom_terminal.rs` | GLUE | Ratatui 0.30 `Backend<Error=io::Error>`, deprecation allows |
| `markdown_render_tests.rs` | GLUE | `concat!` to keep trailing two-space hard break |
| `motion.rs` | GLUE | `regex_lite`->`regex`; `CARGO_MANIFEST_DIR` |
| `pets/image_protocol.rs` | GLUE | vendored `terminal_detection` path |
| `pets/model.rs` | GLUE | hand-rolled Sha256 hex = identical string; sha2 0.11 pin |
| `pets/preview.rs` | GLUE | module path rewrite |
| `render/renderable.rs` | GLUE | path + Ratatui receiver compat |
| `shimmer.rs` | GLUE | module paths |
| `terminal_hyperlinks.rs` | GLUE | `#![allow(deprecated)]` |
| `terminal_probe.rs` | GLUE | `#[allow(unused_imports)]` |
| `test_backend.rs` | GLUE | `type Error = io::Error` |
| `tui/event_stream.rs` | GLUE | `super::` paths + one added pause/resume test |
| `wrapping.rs` | GLUE | comments + path rewrite |

22 of 26 are pure glue. The 4 BEHAVIOR files are all the sanctioned Cubby
pet/asset swap, which AGENTS.md explicitly permits ("RunHaven domain data",
"asset swaps such as `docs/assets/logo.png`"). No unsanctioned behavioral rewrite
of vendored logic was found. This is the strongest single signal that
vendor-first is being respected at the file level.

---

## Active vs Dormant Surface

- ~51 of 355 vendored Rust files (about 14%) are wired into the compiled module
  tree via `mod.rs` (color, custom_terminal, key_hint, motion, shimmer, style,
  terminal_*, wrapping, insert_history, render helpers, `pets/*`,
  `tui.rs` + `tui/*`, `keymap.rs`, a `bottom_pane` subset, `test_backend`).
  Figure is from `mod.rs` inspection, not a build graph; treat as approximate.
- ~305 files (about 86%) sit on disk byte-identical to upstream but uncompiled,
  including the load-bearing ones the plan's end state needs: `app.rs` (57KB),
  `chatwidget.rs` (85KB), `app_server_session.rs` (96KB),
  `bottom_pane/mod.rs`, `history_cell/*`, `streaming/*`.
- This is consistent with "vendor-first, activate-later." The risk is not the
  dormancy; it is that the active 14% is wired through hand-rolled stand-ins
  (D2) rather than toward the dormant 86%, so activation cost rises over time.

---

## Security Boundary Assessment

Verdict: **intact in compiled code; guarded latent risk remains.**

- No `container` exec, host-home / `.ssh` / `.aws` / gcloud / browser-profile
  mount, arbitrary env passthrough, or boundary `fs::write` exists in the active
  surface. Subprocess hits are benign: env-gated handoff smoke (`printf`),
  `#[cfg(test)]` git, and read-only terminal capability probes
  (`tmux`/`zellij`/`cmd.exe`).
- Launch is genuinely disabled: the confirm step renders "Confirmed. TUI launch
  is still disabled." (`app_shell.rs:987`).
- All 13 unsupported families fail closed with a test matrix (F4).
- Plan defaults preserved: `allow_root_user: false`,
  `allow_sensitive_workspace: false` in the preview path (`service.rs`).
- The standing risk is D8: host-reaching Codex surfaces are dormant and
  marker-guarded, but each promotion still needs a RunHaven security review.

---

## Severity-Ranked Remediation Backlog

1. **(High) Finish Codex `App` ownership for Phase 4 before any new screen.**
   `bottom_pane/mod.rs` now compiles from real vendored source; host the
   existing picker inside native `App`/`Tui` next. Stop growing `app_shell.rs`.
   (D1, D10)
2. **(High) Convert `mod.rs` stand-ins into a tracked debt ledger** with a named
   vendored-module upgrade path for each; prefer activating real modules. (D2)
3. **(Medium) Keep crate authority moving with Phase 4**: protocol, config,
   event-data, and bottom-pane crate closures are vendored under original
   names; the next native `App` slice should vendor required `codex-*` crates
   before adding new local stand-ins. (D3)
4. **(Medium) Expand the security guard** as dormant host-reaching modules are
   promoted. (D8)
5. **(Medium) Drive `launch_wizard.rs` boundary/network text from the
   `LaunchPlanData` contract**, not literals; plan its migration to Codex child
   views. (D5)
6. **(Medium) Generate RunHaven snapshots** for the live launch-preview screens.
   (D4)
7. **(Low/Med) Name the `app_server_session.rs` end state** (vendored 96KB vs
   188-line bridge). (D6)
8. **(Low/Med) Flag `tui.rs` `with_restored` as a non-mergeable region** in the
   vendor README for the next upstream sync. (D7)
9. **(Low) Annotate disabled upstream tests** with intent; reconcile the two
   plan copies. (D9, R1)

---

## Method and Evidence

- Structural diff: `scripts/compare-codex-tui.sh` reports 894 upstream files,
  370 RunHaven files, 356 shared paths, 538 external upstream `.snap` goldens,
  and 14 RunHaven-only files under the mapped roots.
- Content drift: the compare script reports 43 copied Codex files with local
  edits against the commit-exact pinned baseline.
- Wiring: direct read of `lib.rs`, `mod.rs`, `app_shell.rs`,
  `runhaven/{protocol,service}.rs`, plus subagent reads of `app_shell.rs`,
  `launch_wizard.rs`, `app_server_client.rs`, `app_server_session.rs`,
  `terminal_handoff.rs`, and the modified vendored files.
- State reconciliation: `feature_list.json`, `current-state.md`,
  `docs/plans/codex-tui-strategy-c/*` vs this Downloads copy.
- Baseline verification: local Codex `git rev-parse HEAD` == pinned commit.

Confidence: high on structural and security findings (commit-exact diff, direct
reads). The active-file count (~14%) is an inspection estimate, not a build
graph; the ratio, not the exact number, is the finding.
