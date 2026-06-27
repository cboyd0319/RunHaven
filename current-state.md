# Current State

Last Updated: 2026-06-27 UTC

## Current Objective

The active slice is the TUI Codex vendor reset. The previous custom RunHaven
TUI implementation is being replaced by the full local Codex TUI source
baseline from `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/`, then
RunHaven-specific integration will continue from that baseline.

The RunHaven TUI setup is the reference implementation for several sibling
projects. Treat reusable TUI structure as a product requirement: keep Codex
vendoring source-first, keep RunHaven-specific behavior in thin adapters, use
shared data contracts, record every culling decision, and keep user-facing text
plain enough for non-technical users.

The `v0.5.0` CLI-complete pre-release was cut and published on 2026-06-26
(first release; pre-1.0, CLI only). Runtime and security hardening,
multi-provider broker work, isolated OAuth login, and release-readiness remain
`passing`. The TUI is now `active` again in `feature_list.json` while the vendor
baseline and integration work are rebuilt.

Do not publish a release from the interim vendor-reset state. After the TUI is
fully integrated, verified, and confirmed, do a full release bump to `v0.6.0`.
Temporary public-doc placeholder statements are not needed; do the full
README/usage/release-doc refresh at the end.

## Startup State Contract

- `AGENTS.md`: root instruction map.
- `feature_list.json`: compact feature status and next product slice.
- `current-state.md`: progress, trusted facts, blockers, and handoff.

Do not recreate separate root `progress.md` or `session-handoff.md` files.
Load deeper docs only when the task touches that surface.

## Product Facts

- RunHaven is a Rust 1.96.0 CLI for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon.
- The CLI is the complete `v0.5.0` pre-release surface and remains the
  explicit automation and recovery backend.
- The terminal UI is unreleased and in an active Codex vendor-reset slice. The
  intended release behavior remains a bare interactive `runhaven` opening the
  TUI while pipes, redirection, and subcommands stay CLI-first.
- The alpha desktop shell lives under `ui/` and `src-tauri/`; it remains
  deferred to a later first-class release phase.
- RunHaven remains alpha/pre-release after `v0.5.0`; stable release scope is
  still open.
- Current sequence: CLI-complete pre-release, completed runtime/security
  hardening slices kept current when touched, active terminal UI vendor reset
  and RunHaven integration, full confirmation and `v0.6.0` release bump, then
  remaining non-UI scope and the first-class desktop app.
- Above all else, secure defaults must be the easiest path. Supported
  lower-security choices should warn and require explicit intent; unsupported
  or hard-boundary violations still fail closed.
- Apple `container machine` is not the default RunHaven boundary, but explicit
  or user-managed machine workflows should not be blocked solely because they
  are less secure. They should warn, require intent, and fail only for concrete
  unsupported or unsafe states.
- All current and future development is DRY and documentation-first. Walk the
  build-necessity ladder (YAGNI, standard library, native platform,
  already-installed dependency, one clear line, then minimum custom code),
  prefer boring over clever, remove meaningful duplication, and ship docs in the
  same slice as the behavior. Every stage also considers file size, modularity,
  crate/component organization, exact current-stable pins, and harness state.
  `AGENTS.md` Working Rules and `docs/harness/boundaries/change-contract.md`
  hold the canonical gate.
- Windows and Linux are not supported runtime or contributor-verification
  targets.
- GitHub Actions CI is disabled during alpha/pre-release. Local verification is
  authoritative until a maintainer explicitly re-enables CI.
- Default safety boundaries remain: no host home mount, no cloud credential
  folder mount, no raw SSH key mount, no arbitrary environment passthrough,
  explicit workspace scope, non-root bundled images, and provider egress
  allowlisting only through reviewed provider mode.

## Key Decisions

Durable rationale that compaction tends to lose. Change these only with new
evidence and a recorded reason.

- Sequencing history: the 2026-06-26 directive deferred GUI/UI behind runtime
  hardening and CLI release readiness; the 2026-06-27 directive pulled the TUI
  forward as a reference implementation for sibling projects. The TUI build plan
  is now complete; the desktop app remains deferred; already-shipped desktop
  slices stay `passing`.
- Default boundary is task-scoped `container run`, not `container machine`,
  because machine workflows map the host user, home, and credentials into the
  guest. Machine use is warned and explicit, not blocked.
- `--ssh` fails closed because Apple `container` 1.0.0 exposed the forwarded
  socket to the non-root guest but `ssh-add -l` returned permission denied;
  enabling it would need a raw key mount or a root default user. Reopen only
  with a no-secret runtime proof.
- CI stays disabled during alpha because local macOS 26 verification is
  authoritative and hosted CI cannot exercise the Apple `container` boundary.
  Re-enable only by explicit maintainer decision.
- The harness is a three-file startup contract because bulk startup context was
  the failure mode; deeper `docs/harness/` material is on-demand reference.
- During active development, exactly one feature should be `active` in
  `feature_list.json` to hold scope; the `active` row is the current slice,
  distinct from `planned` work. After a slice is completed and before the next
  slice is selected, no feature may be active.
- All development is DRY and documentation-first by standing rule (2026-06-24
  user directive), not a per-slice decision. The build-necessity ladder in
  `AGENTS.md` Working Rules and the `change-contract.md` Build Necessity Gate are
  the single canonical source; documentation-is-product means an undocumented
  behavior is treated as not shipped. Boring-over-clever and the edge-case
  tiebreaker (between equally small standard-library options, take the one
  correct on edge cases) resolve style and algorithm choices.
- User-facing writing is product behavior. UI text, menus, prompts, warnings,
  README/usage docs, and setup instructions target non-technical users at
  roughly an 8th grade reading level. Keep exact commands, paths, hosts, and
  safety facts when needed, but explain them with short sentences, plain verbs,
  concrete nouns, and clear next actions.
- The `GitSnapshot`/`GitChange` `available` field is a serde enum tag that
  serializes as the string "true"/"false", never a JSON boolean. Read serialized
  git values with `git_value_available` (or match the typed enum); never
  `Value::as_bool` on `available`. Reading it as a bool silently broke every
  worktree run, diff, and merge until 2026-06-25. The `dirty`, `changed`, and
  `truncated` fields are real booleans.
- The default `--network` mode is profile-aware so the secure path is also the
  default path (2026-06-25 user directive: the secure path must be the easiest
  path, inform rather than block, do not restrict where a restriction cannot be
  made easy). Profiles with bundled provider hosts default to `provider`;
  profiles without them default to `internet` because provider mode would be an
  empty-allowlist dead end. Internet mode is never blocked, only warned. Revisit
  only if a broader safe default becomes both more secure and as usable.
- The `glib` advisory GHSA-wrw7-89jp-8q8g is treated as not-affected because
  `glib` enters only through Tauri's Linux GTK backend and is absent from the
  macOS build graph; it is capped at 0.18.x by `gtk 0.18.2`. Dependabot alert
  was dismissed as "not used" on 2026-06-24. Rationale in `docs/PINNING.md`.
- Provider egress should be low-friction for non-technical users without
  weakening the boundary (2026-06-26 user direction: RunHaven is primarily for
  less-technical people; the user must not manage hosts; exact hosts appear only
  when needed for safety review or troubleshooting). The
  agreed model: trust each agent's own provider expressed as a few stable
  domain-family patterns, not individual hosts; keep default-deny so data-egress
  hosts (`storage.googleapis.com`, etc.) stay closed by simply never being in an
  allow-pattern; degrade gracefully when an optional host is blocked; and
  eventually ship the per-agent policy as signed, auto-updating data so new
  provider endpoints need no release or user action. Step 1 (narrow
  `*-name.domain.tld` family patterns in the egress matcher, anchored to one
  registrable domain) is built and used for Antigravity. Remaining: graceful
  degradation, plain-language foreign-host messaging, and the auto-updating
  policy. This is still an allowlist (default-deny); it just speaks in families.
- Log sanitization and host-held secret storage are a separate non-TUI security
  slice, not part of the TUI build phases (2026-06-27 user direction). Existing
  structured records are intended to be secret-free and raw log views are
  bounded/acknowledged, but the backlog now requires a centralized sanitizer for
  untrusted agent/container stdout/stderr, provider CLI output, raw log
  snapshots, JSONL records, UI state, support bundles, and docs examples. Because
  RunHaven is macOS 26+ only, host-held RunHaven secret material should prefer
  macOS Keychain where practical (for example Claude setup tokens or future
  broker-owned secrets), with a fail-closed or explicit fallback if Keychain is
  unavailable. This does not authorize reading provider-owned Keychain items,
  browser profiles, cloud credential stores, or arbitrary host credentials, and
  it does not reopen the rejected host-side OAuth/subscription-token broker.
- TUI terminal image rendering must follow Codex's protocol choice, not a
  generic crate's auto-detection (2026-06-26 lesson). `ratatui-image` 11.0.6
  auto-selected iTerm2's own OSC 1337 inline-image protocol, which renders blank
  in a full-screen alternate-screen TUI; the image tier was reverted. Codex
  (`codex-rs/tui/src/pets/image_protocol.rs` and `pets/ambient.rs`) instead
  renders via the Kitty graphics protocol on iTerm2 (3.6+) and emits images as
  direct overlays outside the cell buffer, which is TUI-safe. The current reset
  vendors the full Codex TUI source baseline first; future RunHaven image, logo,
  and pet work should adapt Codex native paths instead of rebuilding custom
  renderers.
- The RunHaven TUI is a first-class, reference-quality, reusable
  implementation, not a deferred minimal launcher. It is the intended guided
  front door for a bare interactive `runhaven` and the reference TUI for sibling
  projects. The current active plan is the Codex vendor reset in
  `docs/plans/tui-codex-vendor-reset.md`; the older phase build-plan docs remain
  historical/product intent until the new baseline is integrated.
- The hidden Zork I easter egg remains wanted. The current reset keeps the
  MIT-licensed `historicalsource/zork1` collection under `third_party/zork1/`,
  but the earlier Ferrif-derived engine under `src/runhaven/cli/tui/zork/` was
  removed with the old custom TUI tree and is recoverable from git history. If
  Zork is reintroduced, it must stay TUI-local and attributed, add no
  subprocess/network/workspace/credential/container access, validate bundled
  story bytes, and treat disk save/restore as a carefully validated private
  RunHaven cache feature.
- `src/runhaven/` ownership is locked before TUI Phase 4. Shared host readiness
  lives in `doctor.rs`, secret-free diagnostics data in `diagnostics.rs`, auth
  posture metadata in `provider/auth_profiles.rs`, and run history behind the
  `records/` facade (`run_history.rs`, `run_history/`, `io.rs`). `cli/` owns
  argument dispatch and human presentation, not shared runtime truth. Internal
  `src/runhaven` code imports explicit `crate::runhaven::...` module paths; the
  flat `src/lib.rs` exports are compatibility for Tauri and external callers.

## Latest Verified Work

- 2026-06-27: TUI Codex vendor reset baseline. Added
  `docs/plans/tui-codex-vendor-reset.md` as the wishlist and culling ledger,
  replaced the custom `src/runhaven/cli/tui/` tree with a snapshot of
  `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/`, excluded `.DS_Store`, and
  removed 538 copied upstream `*.snap` files as Codex test goldens with culling
  rationale. Added `src/runhaven/cli/tui/README.md`, updated Codex attribution,
  and recorded local Codex evidence from `/Users/c/.codex/config.toml`
  (`[tui] pet = "custom:cubby"`) plus `/Users/c/.codex/pets/` custom pet
  packages. Verification: `jq empty feature_list.json`, `git diff --check`,
  zero `*.snap`, zero `.DS_Store`, and `cargo check --locked --quiet` failed at
  the expected integration boundary (`src/runhaven/cli/mod.rs` still expects
  `tui.rs` or `tui/mod.rs`, while the copied Codex source is crate-shaped with
  `lib.rs` and `main.rs`). Next: integrate the vendored baseline into
  RunHaven's module/dependency/product-data shape before claiming the TUI is
  runnable again.
- 2026-06-27: TUI source-first logo/native-pet polish. Replaced the oversized
  Cubby header hero with the RunHaven logo from `docs/assets/logo.png` and kept
  Cubby as the compact native ambient pet. Vendored an asset-agnostic
  Codex-derived ambient image adapter from `pets/ambient.rs` and `pets/mod.rs`
  so logo and pet overlays share Codex target sizing, right-anchor, composer
  gap, clear-area, cursor save/restore, Kitty deletion, and Sixel clearing
  behavior. Deleted the old hand-built `tui/mascot` sprite module, kept `p`
  scoped to Cubby only, kept `RUNHAVEN_TUI_PET=0` from hiding the logo, renamed
  user-facing terminal capability text from "pet image" to "terminal image",
  and made active TUI copy use plainer non-technical labels such as "safety
  notes", "checks", "network log", and "command". Locked the source-first and
  8th grade user-facing writing rules into `AGENTS.md` and the TUI plans.
  Verified: `cargo fmt --check`, `cargo test --locked tui --quiet` (267
  TUI-filtered tests), `cargo test --locked --quiet` (338 lib + 6 integration
  tests), `cargo clippy --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins`, JSON validation,
  `cargo build --locked --quiet`, snapshot-new scan, and `git diff --check`.
  Branch: `terminal-ui-build-plan`.
- 2026-06-27: TUI Zork easter egg. Added a hidden Home-only `~` screen that runs
  the bundled MIT-licensed Zork I story through an attributed Ferrif-derived
  Z-machine. Vendored the full `historicalsource/zork1` collection under
  `third_party/zork1/`, added Ferrif/Zork license files and notice entries,
  replaced the prior footer easter egg, and added a Zork VT100 snapshot plus
  focused tests for boot, Home-only routing, `q` as game input, exact Quetzal
  save-file shape (`FORM`/`IFZS`, `IFhd`, memory, `Stks`), restore, malformed
  or symlinked save rejection, and private save-file permissions. The easter egg
  adds no new Cargo dependencies and is documented as TUI-local with no
  subprocess, network, workspace, credential, container, or arbitrary save-file
  access. Verified: `cargo fmt --check`, `cargo test --locked zork --quiet` (79
  filtered tests), `cargo test --locked tui --quiet`, full locked cargo tests,
  locked clippy with warnings denied, pin check, JSON validation, security grep,
  typography scan, `cargo build --locked --quiet`, and `git diff --check`.
  Branch: `terminal-ui-build-plan`.
- 2026-06-27: TUI design-review polish. Replaced the old right-side brand copy
  with at-a-glance launch context: four-step wizard, selected agent, network,
  workspace, boundary, and next safe action. This still used Cubby as the header
  hero; the later source-first polish superseded that with the RunHaven logo in
  the header and native Cubby as the ambient pet. Added a compact launch stepper
  to workspace, review, and confirm screens; shortened Home and guide footers
  around screen-local actions; made `p` discoverable from the guide; and
  documented the wizard/user-flow/action model plus stock agent CLI reference
  conventions in the TUI architecture guide. Updated README, USAGE, the brand
  graphics plan, `feature_list.json`, and affected VT100 snapshots. Verified:
  `cargo fmt --check`, `cargo test --locked tui` (189
  TUI-filtered tests), `cargo clippy --all-targets --locked -- -D warnings`,
  JSON validation, typography scan, and `git diff --check`. Branch:
  `terminal-ui-build-plan`.
- 2026-06-27: TUI Phase 5 polish and final build-plan closeout. Added
  `src/runhaven/cli/tui/guide_views.rs` for the RunHaven Guide, opens it first
  when the run-record log is missing or empty, and routes `?`/F1 to it from the
  main screens. Added dashboard notices for status errors, stop/kill transitions,
  stale/done containers, stale/repair markers, and log snapshots that appear to
  be waiting for input or device-code interaction. Added line-mode render
  coverage for guide/history/diagnostics/doctor surfaces, a guide snapshot, and
  the former Home-only footer easter egg. Updated README, USAGE,
  CAPABILITIES, ROADMAP, RELEASE_GAP_ANALYSIS, the TUI build plan, the TUI
  architecture guide, the brand graphics plan, `feature_list.json`, `init.sh`,
  and this state file. Verified: `./init.sh` with its new explicit
  `cargo test --locked tui` lane (187 TUI-filtered tests), full root
  `cargo test --locked` (258 lib tests + 6 integration tests), root clippy, pin
  check, JSON validation, frontend check/test/build/e2e, Tauri fmt/test
  (30 passed, 1 ignored)/clippy/debug no-bundle build, root build, and
  `git diff --check`; post-state stale-reference scan, typography scan, JSON
  validation, and `git diff --check`; and a bounded PTY launch smoke with an
  empty cache that confirmed the first frame renders the RunHaven Guide. Live
  Apple `container` smokes were not rerun because this phase did not change
  runtime boundary behavior. Branch: `terminal-ui-build-plan`.
- 2026-06-27: TUI Phase 4 history and diagnostics. Added
  `src/runhaven/cli/tui/history.rs` and `history_views.rs` for run history,
  per-run diff review, diagnostics, terminal capability reporting, and doctor
  remediation screens. Added a shared `records::run_diff_text` API so the TUI
  consumes diff text as data while `runhaven runs diff` preserves its existing
  output. Home now exposes `h` for history and `g` for diagnostics; diagnostics
  opens doctor with `d`. Split input/navigation handling into `tui/input.rs`,
  bringing `tui/mod.rs` back down to about 604 lines. The diagnostics screen
  uses only the shared
  secret-free diagnostics readers/status payload, and doctor uses shared
  `doctor::collect_checks`. Added VT100 snapshots for history, run diff,
  diagnostics, and doctor screens, plus focused navigation/state tests.
  Verified: `cargo fmt --check`, `cargo test --locked tui` (181 TUI-filtered
  tests), `cargo test --locked` (252 lib tests + 6 integration tests), `cargo
  clippy --all-targets --locked -- -D warnings`, `cargo test --manifest-path
  src-tauri/Cargo.toml --locked` (30 passed, 1 ignored), `cargo clippy
  --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins`, JSON validation, typography
  scan, `git diff --check`, and a bounded PTY launch/key smoke with `h`, `g`,
  `q`. Branch: `terminal-ui-build-plan`. Phase 5 completed in the following
  TUI slice.
- 2026-06-27: Pre-Phase 4 organization/docs lock. Moved shared doctor logic to
  `src/runhaven/doctor.rs`, secret-free diagnostics data to
  `src/runhaven/diagnostics.rs`, run history behind the `records/` facade
  (`run_history.rs`, `run_history/`, and `io.rs`), and shared agent auth posture
  helpers into `provider/auth_profiles.rs`. Internal `src/runhaven` imports now
  use explicit `crate::runhaven::...` ownership paths while `src/lib.rs` remains
  the Tauri/external compatibility facade. Refreshed the README, product docs,
  TUI plans, harness docs, `AGENTS.md`, `feature_list.json`, and this state
  file so the active TUI and later desktop sequencing are consistent. Updated
  `init.sh` to validate `feature_list.json` and run `git diff --check`.
  Refreshed `src-tauri/Cargo.lock` offline so the Tauri crate locks current
  root-crate dependencies. Verified: `./init.sh` (root cargo fmt/test/clippy,
  242 lib tests + 6 integration tests, pin check, JSON validation, frontend
  check/test/build/e2e, Tauri fmt/test/clippy, Tauri debug no-bundle build, root
  build, and `git diff --check`), stale-reference scan, typography scan, and
  read-only code-reviewer approval. Live Apple `container` smokes were not rerun
  because this pass did not change runtime boundary behavior. Branch:
  `terminal-ui-build-plan`. Phase 4 completed in the following TUI slice.
- 2026-06-27: TUI Phase 0 foundation. Added the reusable TUI settings/theme
  layer (`NO_COLOR`, `RUNHAVEN_TUI_REDUCED_MOTION=1`,
  `RUNHAVEN_TUI_LINE_MODE=1`, dark/light palette seam), a synchronous
  `event::poll` tick loop with deterministic ticker tests, Codex-derived
  `color.rs`, a Codex-derived VT100 test backend, and an `insta` snapshot harness
  with accepted home/detail snapshots at 80x24 and 120x36. The current screens
  now render through the palette, Cubby has a no-color shape fallback, and the
  80-column agent list truncates with an ASCII affordance instead of clipping
  mid-word. New exact-pinned pure-Rust dev deps: `insta =1.48.0`
  (`default-features = false`) and `vt100 =0.16.2`; pin checking now covers them.
  Docs updated in `USAGE.md`, `THIRD_PARTY_NOTICES.md`, and
  `docs/plans/tui-build-plan.md`. Verified: `cargo fmt --check`, `cargo test
  --locked tui` (135 TUI-filtered tests), `cargo clippy --all-targets --locked
  -- -D warnings`, `cargo run --locked --bin runhaven-check-pins`, and `git diff
  --check`. Branch: `terminal-ui-build-plan`. Next: Phase 1 brand complete.
- 2026-06-27: TUI Phase 1 brand. Embedded the validated Cubby Codex pet package
  at `src/runhaven/cli/tui/assets/cubby/` and copied QA evidence to
  `docs/assets/cubby-pet/` (validation ok, contact sheet, preview GIFs, review
  ok with one accepted jumping `stable-slots` warning). The home banner now uses
  the real Cubby atlas: Codex animation timing selects idle frames, reduced
  motion pins the first idle frame, the portable fallback renders the current
  frame as half-blocks, and graphics terminals use the Codex Kitty/iTerm2/Sixel
  overlay path after the ratatui draw. `p` toggles Cubby for the session and
  `RUNHAVEN_TUI_PET=0` starts hidden. Added RunHaven-authored rotating footer
  tooltips. Verified after implementation: `cargo fmt --check`, `cargo test
  --locked tui`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo
  test --locked`, `cargo run --locked --bin runhaven-check-pins`, Cubby atlas
  validation, copied metadata path sanitation, iTerm2 3.6.11 PTY launch/quit
  smoke, and `git diff --check`. Branch:
  `terminal-ui-build-plan`. Next: Phase 2 launcher flow.
- 2026-06-27: TUI Phase 2 launcher flow. Added `tui/launcher.rs` for
  workspace-picker and plan-review state, `tui/widgets.rs` for shared drawing
  helpers, and `tui/tests.rs` to keep `tui/mod.rs` small enough to review. The
  TUI now opens a workspace picker with simple fuzzy filtering and typed paths
  (`w`), keeps the agent picker as the provider selector, builds a review from
  the shared `AgentRunPlan`, shows the workspace mount, state volume, network
  mode, provider egress posture, explicit non-mounts, and equivalent CLI command,
  requires typing `run` only when the shared planner emits security notices,
  restores the terminal, and launches through `launch_run_plan`. Accepted `.snap`
  files stay tracked as golden baselines; `.gitignore` now ignores only
  `*.snap.new`. Verified after implementation: `cargo fmt --check`, `cargo test
  --locked tui` (159 TUI-filtered tests), `cargo clippy --all-targets --locked
  -- -D warnings`, `cargo test --locked`, `cargo run --locked --bin
  runhaven-check-pins`, iTerm2 3.6.11 PTY review-screen smoke, and `git diff
  --check`. Branch: `terminal-ui-build-plan`. Next: Phase 3 run management.
- 2026-06-27: TUI Phase 3 run management. Added `tui/runs.rs` and
  `tui/run_views.rs` for active-run dashboard state and rendering while keeping
  `tui/mod.rs` focused on orchestration. The dashboard opens with `d`, lists
  active runs, shows sanitized status/resource/network details, filters provider
  egress log entries by run id, and opens an explicit bounded log viewer with
  search, scroll, tail-following, and ANSI parsing through `vt100` so escape
  sequences are not replayed into the terminal. Stop, hard-stop, and stale-marker
  repair use typed-confirm screens over the existing validated run-control
  cores. Provider-mode runs now write egress decision deltas while active instead
  of only at run-record finalization. `vt100 =0.16.2` moved from dev-dependency
  to runtime dependency for the log renderer; pin checking covers the new
  manifest shape. Verified: `cargo fmt`, `cargo test --locked tui` (171
  TUI-filtered tests), `cargo test --locked
  provider_decision_deltas_only_emit_new_counts`, `cargo clippy --all-targets
  --locked -- -D warnings`, `cargo test --locked` (241 lib + 6 integration),
  `cargo run --locked --bin runhaven-check-pins`, and a PTY smoke that opened
  `runhaven`, pressed `d`, rendered the no-active-runs dashboard, and exited
  cleanly. Branch: `terminal-ui-build-plan`. Phase 4 completed in a later
  slice.
- 2026-06-26: Vendored codex's pet/image rendering stack under
  `src/runhaven/cli/tui/codex/` (Apache-2.0, with attribution), covering the
  three pillars: the high-fidelity hero/image tier (`terminal_detection.rs`,
  `image_protocol.rs` with the iTerm2 3.6+ Kitty `t=f` path, `sixel.rs`), the
  pet system (`model.rs`, `frames.rs`, `catalog.rs`), and pet animation timing
  (`animation.rs`, the decoupled `current_animation_frame` extracted from
  `ambient.rs`). UI/runtime-coupled codex code (the ambient state machine,
  `FrameRequester`/tokio, picker/preview, asset_pack) was deliberately not
  vendored; RunHaven supplies its own tick loop and run-state mapping. New
  pure-Rust deps: `base64` `=0.22.1`, `image` `=0.25.10` (png+webp), no C.
  Attribution: `licenses/codex-Apache-2.0.txt` + `THIRD_PARTY_NOTICES.md`
  (carries OpenAI copyright + the Ratatui MIT note). Not yet wired into the app.
  Text-motion polish (codex `shimmer.rs`/`motion.rs`, needs `color.rs` +
  `supports_color`) deferred as optional. Verified: cargo build, clippy
  `-D warnings`, `cargo fmt --check`, `cargo test --locked` (120 lib incl. 40
  vendored + 6 integration), pin check.
- 2026-06-26: Reverted the `ratatui-image` high-resolution image tier (it
  rendered blank on iTerm2; see the Key Decision above). The home banner is back
  to the reliable xterm-256 half-block Cubby hero, which renders on every
  terminal. `ratatui-image` and `image` deps removed. High-resolution rendering
  will be redone via codex's Kitty-graphics overlay approach. Verified: cargo
  build, `cargo test --locked` (10 TUI tests), pin check.
- 2026-06-27: Generated Terminal.app-safe Cubby header/hero mascot assets from
  the reference cube image. Added exact pixel-grid PNGs plus half-block ANSI
  renderings for 16x18, 24x26, 32x36, 40x44, and 48x52 under
  `docs/assets/terminal-mascot/`. The generator removed the dark background with
  edge-connected flood fill, added a one-pixel safety inset, quantized every
  opaque pixel to stable xterm 256-color indices 16-255 (avoiding profile-
  dependent base colors 0-15), and wrote a contact sheet plus manifest. Verified:
  PNG dimensions, half-block cell dimensions, ANSI escape/index constraints,
  xterm-palette membership, visual contact sheet, `magick identify`, and
  `git diff --check`.
- 2026-06-26: TUI slices 2b/2c (Cubby mascot). Moved `tui.rs` to `tui/mod.rs`
  and added a `tui/mascot.rs` + `tui/mascot/sprites.rs` submodule so branding
  stays separate from the functional cards. The mascot is **Cubby**: a glass
  container cube with a gold agent spark inside, on a layered base. Slice 2b was
  a hand-built placeholder; slice 2c replaced it with the finished art, real
  pixel renders quantized to xterm-256 (indices 16-255, avoiding 0-15 for macOS
  Terminal.app stability) at sizes 16x18/24x26/32x36/40x44, embedded as index
  grids and rendered via `Color::Indexed` half-blocks (the portable rendering
  floor, no image protocol). `hero_for_banner` shows the largest hero that fits
  the terminal, so detail scales up on big windows and degrades to 16x18 on an
  80x24 floor; the brand is vertically centered beside it. Source renders are in
  `docs/assets/terminal-mascot/`, the 1024px master in
  `docs/assets/cubby-hero-1024.png`. The animated pet (codex-pet-style atlas,
  validated and backed up at `codex-pet`) is a separate future slice. Verified:
  cargo fmt, `cargo test --locked` (79 lib incl. 10 TUI tests + 6 integration),
  clippy `-D warnings`, pin check, `git diff --check`.
- 2026-06-26: Captured TUI architecture patterns in
  `docs/plans/tui-architecture.md` (study of the Codex `ratatui` TUI): the
  planner and policy objects are the single source of truth; adapters build and
  widgets only draw; fixed-size vs width-aware cards with bounded lists; shared
  draw helpers; a palette and color-mode module; the TUI and the Tauri app share
  data rather than duplicated logic; branding stays separate from functional
  cards; per-card `TestBackend` fixtures. These guide the upcoming slices.
- 2026-06-26: TUI slice 2 (agent picker). The home screen is a navigable agent
  list (up/down or j/k, clamped) via a ratatui `ListState`; enter opens a
  per-agent detail screen (description, image, support tiers) and esc/backspace
  returns home. The detail tiers reuse the shared `agent_sign_in`/`agent_broker`
  helpers from `provider/auth_profiles.rs` plus `default_network_mode`, so the
  TUI and `runhaven agents` share one source. Input is a testable `handle_key`. Verified:
  cargo fmt, `cargo test --locked` (74 lib incl. 5 TUI tests + 6 integration),
  clippy `-D warnings`, non-TTY help fallback, `git diff --check`.
- 2026-06-26: Started the terminal UI (TUI). Decision: reference the Codex TUI
  (`codex-rs/tui`) for quality and patterns, not fork it; it is ~214k lines,
  Apache-2.0, an agent-chat domain welded to ~30 `codex-*` internal crates (even
  its event loop imports `codex_protocol`), while RunHaven needs a
  launcher/manager, a different domain (the agent's own chat runs inside the
  container). Pinned `ratatui =0.30.2` (current stable). Slice 1 (scaffold):
  `src/runhaven/cli/tui/mod.rs` with `ratatui::init`/`restore` (panic-safe terminal),
  a draw/key-event loop, and a home screen listing the agents from `profiles()`.
  Bare `runhaven` opens the TUI only when both stdin and stdout are a terminal
  (`should_launch_tui`); piped, redirected, or subcommand invocations keep the
  CLI. Tracked as the active `terminal-ui` feature. Also fixed the v0.5.0
  pin-check miss (`pins.toml [runhaven] version` was left at 0.1.0; bumped to
  0.5.0, fixed forward on `main`, not re-tagged). Verified: cargo fmt, `cargo
  test --locked` (71 lib incl. 2 TUI tests + 6 integration), clippy `-D
  warnings`, pin check passes, non-TTY falls back to help, `git diff --check`.
- 2026-06-26: Cut and published the `v0.5.0` CLI-complete pre-release. Closed
  V05-G4 (Apple container smokes, default plus `--with-provider` and
  `--with-ssh`, passed on the current code, validating the egress wildcard
  matcher and SSH fail-closed) and V05-G8 (`CHANGELOG.md` release notes, README
  pre-release caution, feature_list and current-state closure). Bumped the
  runhaven CLI crate `0.1.0` to `0.5.0` (`runhaven --version` = 0.5.0); src-tauri
  rebuilt against it (30 Tauri tests pass); both `Cargo.lock` files updated; image
  template tags stay `0.1.0` (built locally, unchanged). Pushed `main` to origin,
  tagged `v0.5.0`, pushed the tag, and published a GitHub pre-release. This closes
  all eight v0.5.0 gap-analysis blockers. Verified: cargo fmt, `cargo test
  --locked` (69 lib + 6 integration), clippy `-D warnings`, src-tauri tests (30),
  Apple container smokes, `cli_surface_check.sh` 39/39, doc scans, `git diff
  --check`.
- 2026-06-26: Closed the last two `v0.5.0` CLI-complete items (on `main`,
  committed locally), so all four closure items are now done. JSON and local-data
  lifecycle: recorded an explicit decision in `V1_RELEASE_PLAN.md` (Data, Storage,
  And Recovery) with a `USAGE.md` pointer, through `v0.5.0` every CLI `--json`
  output and local record file (`runs.jsonl`, `egress-policy.jsonl`,
  `auth-broker.jsonl`, active-run markers) is best-effort and unversioned; OAuth
  tokens, locks, the login workspace, and volumes are internal; stability needs
  an explicit `schema_version` field first (audit/log outputs are the first
  candidates). CLI maintainability check (lean-reviewer pass against the
  ~500-line guard): deleted dead `ApiKeyBrokerProxy::bind` and deduped
  `logout_shared_volume` onto the shared existence-aware `delete_volume` (logout
  now gets the same existence check and retry). `egress.rs` (525),
  `auth_broker.rs` (512), and `app.rs` (508) are slightly over the guard but
  cohesive surfaces left whole; `egress.rs` has a policy-vs-proxy seam to split
  only if it grows. Verified: cargo fmt, `cargo test --locked` (69 lib + 6
  integration), clippy `-D warnings`, doc scans and `git diff --check` clean.
- 2026-06-26: CLI command + docs contract audit (a `v0.5.0` closure item, on
  `main`, committed locally). Cross-checked the full command tree (14 top-level
  commands plus all subcommand groups: runs 15, image 3, network 2, state 3,
  auth 3, egress 1, why 4) against the clap help, `CLI_SURFACE_COVERAGE.md`, and
  `USAGE.md`. Fixed one doc gap (`runhaven agents` was missing from USAGE; added
  a List Agents section). The breadth surface check then caught a real regression
  from this session's `--auth-scope agent` default: `runhaven run --session X`
  now uses the shared per-agent volume, so `state reset --session X` (always
  project-scoped) targeted a session volume the run no longer creates and failed
  on a non-existent volume. Fixes: `state reset`/`state prune` deletion is now
  existence-aware (a missing volume is reported, not an error) and retries while
  a volume is transiently held after a stop or kill (Apple container does not
  auto-remove the container); the surface check's active run uses `--auth-scope
  project` so the session-volume reset path is actually exercised. Result:
  `cli_surface_check.sh` 39/39. Verified: cargo fmt, `cargo test --locked` (69
  lib incl. a new `volume_in_list` test + 6 integration), clippy `-D warnings`,
  `git diff --check`. Follow-up: RunHaven runs without `--rm`, so killed
  containers reap asynchronously; a runtime-lifecycle review is a separate item.
- 2026-06-26: Profile support tiers (a `v0.5.0` CLI-complete closure item, on
  `main`, committed locally). Made the per-agent support matrix code-derived so
  it cannot drift: `runhaven agents` now prints sign-in path (`runhaven login`
  vs in-sandbox vs n/a), default network (provider/internet), and API-key broker
  (yes/no) per agent, sourced from `login::supports_login`,
  `default_network_mode`, and `auth_profiles::is_brokered`. Fixed the stale
  `CAPABILITIES.md` matrix (codex now lists `auth.openai.com`; copilot lists the
  `githubcopilot.com` hosts plus `github.com`/`api.github.com`; added the
  `runhaven login` mentions) and pointed it at `runhaven agents`; updated
  `CLI_SURFACE_COVERAGE.md` and marked the `NON_UI_BACKLOG.md` item done. Tests
  assert the login set {claude, codex, copilot, antigravity} and the broker set
  {claude, codex, gemini}. Verified: cargo fmt, `cargo test --locked` (68 lib +
  6 integration), clippy `-D warnings`, doc scans and `git diff --check` clean.
- 2026-06-26: Full repo docs and README refresh, then merged the
  `runtime-security-hardening` branch to `main`. Rewrote `README.md` (added the
  `runhaven login` sign-in story for all four agents, plain-language egress
  framing, and the then-current UI-last roadmap), and updated the product,
  roadmap, and policy docs to match current state (PROVIDER_ENDPOINTS host sets
  and the domain-family pattern, USAGE/CAPABILITIES/SECURITY_MODEL/ARCHITECTURE
  broker-and-egress wording, ROADMAP/V1_RELEASE_PLAN/NON_UI_BACKLOG re-sequencing
  and login-done, CLI_SURFACE_COVERAGE login row, CONTRIBUTING/SECURITY/RESEARCH).
  Verified: `git diff --check`, em-dash/emoji/canned-phrase scans clean, relative
  Markdown link check (32 README links + the rest, 0 broken), pin check passed,
  and a final `cargo fmt`/`test --locked` (66 lib + 6 integration)/clippy
  `-D warnings` green. Pushed to `origin/main`.
- 2026-06-26: Non-technical UX pass on login/run output (the "much easier"
  thread, after the four-agent set). (1) `launch_run_plan` now preflights the
  agent image (`image_doctor::image_is_built`) and fails with "Build it once
  with: runhaven image build <agent>" instead of the cryptic
  `registry-1.docker.io 401`. (2) Login guidance anticipates the two verified
  friction points: Codex points to ChatGPT Settings then Security for device-code
  login; Copilot pre-warns that the plaintext-keychain prompt should be answered
  `y` because the file lives in the isolated volume. (3) The end-of-run
  blocked-host output is now a calm two-line plain-language notice ("RunHaven
  kept <agent> inside its provider's network and blocked N other destinations to
  protect your data; run `runhaven egress log` ...") instead of a per-host
  technical dump; detail stays in `runhaven egress log`, and the now-dead
  `provider_denial_next_action` was removed. Verified: fmt, `cargo test --locked`
  (66 lib + 6 integration), clippy `-D warnings`, `git diff --check`. Remaining
  UX: the auto-updating signed provider-policy file (post-alpha, the bigger
  lift), and a live look at the new messages on the user's machine.
- 2026-06-26: Added narrow domain-family allowlist patterns (step 1 of the
  lower-friction egress design). The egress matcher now accepts maintainer-
  curated `*-name.domain.tld` wildcard patterns, anchored so the wildcard can
  only expand a subdomain label inside one registrable domain (must start with
  `-` or `.` and carry a >=2-dot tail; `*-foo.com` is rejected at construction).
  Applied to Antigravity: the exact `daily-cloudcode-pa` pin became
  `*-cloudcode-pa.googleapis.com`, so any Google Cloud Code channel/region prefix
  is covered without a re-pin while `storage` and other googleapis.com services
  stay denied. Tests: positive (daily-/us- allowed), negative-exfil (storage
  denied), construction guard (cross-domain rejected). Verified: fmt, `cargo test
  --locked` (65 lib + 6 integration), clippy `-D warnings`, `git diff --check`.
- 2026-06-26: Built and live-verified `runhaven login antigravity`, completing
  the user's four-agent set (Claude, Codex, Copilot, Antigravity). agy has no
  login subcommand, so `runhaven login antigravity` runs agy, whose first run
  triggers a Google OAuth sign-in (the user approves in the host browser, then
  types /exit); the login persists in the shared home volume. Observing the real
  egress corrected the reverse-engineered research: the flow is Google
  auth-code with a redirect to `antigravity.google` (not device-code, not
  localhost), and the live model endpoint is `daily-cloudcode-pa.googleapis.com`
  (not `cloudcode-pa` alone). Pinned 4 bundled hosts from the egress ledger:
  `oauth2.googleapis.com`, `www.googleapis.com`, `cloudcode-pa.googleapis.com`,
  `daily-cloudcode-pa.googleapis.com`. `accounts.google.com` and
  `antigravity.google` are browser-side only, not bundled. Live-verified: the
  model answered prompts ("Kermit is green"). agy's startup eligibility check
  fetches `lh3.googleusercontent.com` (profile pic); blocked it is a cosmetic
  "Eligibility check failed" line and the agent still works, so it is left out of
  the default (least privilege) with a documented `--provider-host` opt-in.
  antigravity now defaults to provider mode (it has bundled hosts), a security
  improvement over the prior internet default. Decision: keep tight host pins,
  not `*.googleapis.com` (that would open `storage.googleapis.com` as an
  exfil channel). Verified: fmt, `cargo test --locked` (63 lib incl. an updated
  default-network test + 6 integration), clippy `-D warnings`, `git diff
  --check`. Open question from the user: geo/endpoint variation of the
  cloudcode-pa hosts; a narrow `*-cloudcode-pa.googleapis.com` matcher (Google
  controls googleapis.com, so it cannot open storage/gmail) is the candidate fix.
- 2026-06-26: Built `runhaven login` for Codex and Copilot (in-sandbox device
  flow). Each runs the CLI's own device-code login (`codex login --device-auth`;
  `copilot login`) once inside the sandbox on the agent's shared home volume
  (`--auth-scope agent`), reusing `launch_run_plan` in provider mode. The
  credential stays in the isolated volume; RunHaven never sees the token.
  Allowlisted the login/refresh hosts in `endpoints.rs`: `auth.openai.com` for
  Codex; `github.com` and `api.github.com` for Copilot (a deliberate, documented
  egress widening, flipped from `candidate` to `bundled` in the endpoint matrix).
  `--clear` deletes that agent's shared home volume. Added
  `paths::login_workspace_dir` (a stable read-only login workspace), command +
  allowlist unit tests, and `AUTH_BROKER`/`USAGE` docs. Verified: cargo fmt,
  `cargo test --locked` (63 lib incl. 2 new login tests + 6 integration), clippy
  `-D warnings`, `git diff --check`. Live-verified 2026-06-26: `runhaven login
  codex` reached `auth.openai.com/codex/device` and returned "Successfully
  logged in"; `runhaven login copilot` reached `github.com/login/device` and
  returned "Signed in successfully", both persisted in the shared home volume.
  Two verified gotchas: Codex needs the ChatGPT account "device code
  authorization" setting on (OpenAI gates it, not RunHaven); Copilot has no
  in-container keychain so it prompts "Store token in plaintext config file?
  (y/N)" defaulting to N, the user must answer y (the token lands in the
  isolated volume, the same model as every other in-container login). Prereqs
  that bit during verification: the agent image must be built first
  (`runhaven image build <agent>`; an unbuilt image fails with a cryptic
  registry-1.docker.io 401), and Apple container DNS had gone stale after a host
  network change (fixed with `container system stop && container system start`).
  The user flagged that these friction points need much friendlier UX/phrasing
  (image-not-built message, Copilot keychain heads-up, Codex toggle heads-up) as
  the next thread. Antigravity login is the last of the four (its hosts are
  reverse-engineered, so observe real egress before pinning an allowlist).
- 2026-06-26: Built `runhaven login claude`, the Claude setup-token opt-in (the
  zero-friction path the user chose; Claude has no in-container device login at
  the pinned version). It runs Anthropic's `claude setup-token` on the host
  (needs host Claude Code), stores the token `0600` in the RunHaven cache, and
  `runhaven run claude` injects it at run time as a name-only
  `--env CLAUDE_CODE_OAUTH_TOKEN` (value from the RunHaven process env, never on
  the argv or in the printed `plan`). A run-time notice marks the injection;
  provider-mode egress confines the token to Anthropic's hosts;
  `runhaven login claude --clear` removes it. New
  `src/runhaven/runtime/login.rs` + `paths::oauth_token_path`, wired into the
  standard and provider run paths; docs in `AUTH_BROKER`/`USAGE`. Verified: cargo
  fmt, `cargo test --locked` (61 lib incl. 2 new login tests + 6 integration),
  clippy `-D warnings`, Tauri builds and clippy. Live-verified 2026-06-26:
  `runhaven login claude` stored the token and `runhaven run claude` came up
  authenticated with no login prompt (zero friction).
- 2026-06-26: Started the `oauth-isolated-login` slice (easy OAuth; the product's
  target audience uses subscription/OAuth logins, not API keys). Live-verified
  that a Claude Max subscription OAuth login works end to end inside a RunHaven
  sandbox: provider mode with the login hosts (`api.anthropic.com`, `claude.ai`,
  `platform.claude.com`) allowlisted, no host credentials mounted; telemetry and
  registry hosts were correctly blocked and the agent still ran. The OAuth path
  is the isolated in-container login (not a broker; brokering is ToS-forbidden).
  Built the first piece: `--auth-scope <agent|project>` (default `agent`) shares
  one per-agent home volume (`runhaven-<agent>-shared-home`) across all
  workspaces so the login is done once; `project` keeps the per-workspace volume.
  Documented the shared-login tradeoff in `SECURITY_MODEL`/`USAGE`/`CAPABILITIES`.
  Verified: cargo fmt, `cargo test --locked` (59 lib incl. a new auth-scope test
  + 6 integration), clippy `-D warnings`, Tauri test (30) and clippy. One UX
  friction the user flagged: the in-sandbox login requires copy/pasting the URL
  to the browser and the code back; remaining work cuts that (see Next Step).
- 2026-06-26: Completed the `multi-provider-broker` slice (branch
  `runtime-security-hardening`, not yet merged). Generalized the Codex API-key
  broker into a provider-agnostic core (`ProviderBrokerProfile`: upstream host,
  path matcher, credential-injection strategy, guest redirect) and wired Codex,
  Claude, and Gemini brokers into the run orchestration. The real host key stays
  host-side for every provider; the guest gets only a placeholder plus a base-URL
  redirect (Codex custom-provider config; Claude `ANTHROPIC_BASE_URL` with
  `x-api-key`; Gemini `GOOGLE_GEMINI_BASE_URL` with `x-goog-api-key`). Renamed
  `--codex-api-key-broker-env` to `--api-key-broker-env` (old name kept as an
  alias) and the field/types; flipped Claude/Gemini `auth_profiles` from
  design-only to the api-key-broker status; Copilot stays design-only (token
  exchange + dynamic API host cannot be brokered without TLS interception). OAuth
  and subscription logins stay out of broker scope and use isolated in-container
  state; RunHaven never reads host `~/.claude.json` or the Keychain. Docs updated:
  `AUTH_BROKER.md`, `SECURITY_MODEL.md`, `ARCHITECTURE.md`, `CAPABILITIES.md`,
  `USAGE.md`. Verified: cargo fmt, `cargo test --locked` (58 lib incl. 3 new
  broker-config tests + 6 integration), clippy `-D warnings`, Tauri `cargo test`
  (30) and clippy. Claude/Gemini live redirect needs a real provider key on the
  target CLI version (unit and fail-closed tested here). Commits 4f11716,
  b5cf193, 2d696e2 plus docs.
- 2026-06-26: Completed the `runtime-security-hardening` audit-and-fix slice
  (branch `runtime-security-hardening`, not yet merged). Ran a two-lens audit
  (apple-container-expert + security-audit) cross-referenced against the upstream
  Apple source clone; the core boundary verified sound. Empirically confirmed
  audit finding #1 (Medium) with a scoped live probe on macOS 27.0: a hostOnly
  guest opened raw TCP to a host listener on the gateway while direct internet
  egress was refused. Apple `container` 1.0.0 has no per-port guest-to-host
  firewalling, so #1 is remediated by a `SECURITY_MODEL.md` caveat plus guidance,
  with an in-guest eBPF egress filter logged design-first. Fixed under test: #2
  the state-volume-prep root container now drops all caps and re-adds only
  CHOWN/FOWNER/DAC_OVERRIDE (live smoke confirmed prep still works); #4 added
  `/usr`, `/bin`, `/sbin`, `/opt` to `sensitive_workspace_paths`; #5 the provider
  proxy and Codex broker warn on the `0.0.0.0` bind fallback. Also ran a
  competitive landscape scan (clones under `~/Documents/GitHub/`: `sand`,
  `container-use`, `agent-sandbox`) and recorded borrowed ideas in
  `NON_UI_BACKLOG.md`. Verified: cargo fmt, `cargo test --locked` (52 unit incl.
  3 new + 6 integration), clippy `-D warnings`, `git diff --check`, and
  `scripts/apple_container_smoke.sh` on macOS 27.0.
- 2026-06-25: Added desktop diagnostics (partial V1-G5): read-only, secret-free
  `get_egress_log`, `get_auth_log`, and `get_auth_status` behind the `main-read`
  capability. `read_egress_policy_log`/`read_auth_broker_log` back the logs; a new
  shared `auth_status_payload` core backs both the CLI `auth status` and the Tauri
  command. Responses map only metadata (host/port/decision/reason/count,
  method/path/upstream-status) and intentionally omit workspace paths, mirroring
  the CLI text output. A self-contained `DiagnosticsPanel` fetches and renders
  them on demand, keeping `App.svelte` lean. `why` explanations, blocked-host
  review, and auth explain remain CLI-only. Verified: main `cargo test` (49),
  Tauri `cargo test` (30, incl. 3 diagnostics mapper tests + `capability_guard`)/
  clippy, svelte-check, 15 unit tests, build, and Playwright e2e (3, incl. a
  diagnostics load test).
- 2026-06-25: Completed the V1-G3 desktop run-control surface by adding GUI
  `kill_run` and `repair_run`, mirroring `stop_run`. Shared library cores
  `kill_active_run` and `repair_active_run` validate the run id, active marker,
  and RunHaven-owned container before `container kill` / stale-marker repair, and
  back both the CLI `runs_kill`/`runs_repair` and the typed Tauri commands. Both
  require confirmation and sit behind the `run-control` capability
  (`allow-kill-run`, `allow-repair-run`). `RunStatusPanel` now has Hard stop and
  Repair marker controls with per-action confirm checkboxes over consolidated
  `controlBusy`/`controlError`/`controlMessage` state and a shared `runControl`
  helper in `App.svelte` (DRY across stop/kill/repair). Verified: main
  `cargo test` (49), Tauri `cargo test` (27, incl. 4 new kill/repair tests +
  `capability_guard`)/clippy, svelte-check, 15 unit tests, build, and Playwright
  e2e (2, extended to stop+kill+repair a preview run).
- 2026-06-25: Implemented `tauri-stop-run-control`, the first v1 desktop GUI
  feature. Added a shared library core `stop_active_run` that validates the run
  id, active marker, and RunHaven-owned container before `container stop`, and
  backs both the CLI `runs_stop` and the typed Tauri `stop_run` command. The
  command requires `confirm_stop`, lives behind the narrow `run-control`
  capability (`allow-stop-run` added to `build.rs` and `run-control.json`;
  `capability_guard` confirms it is an `allow-*` scope), and `RunStatusPanel`
  shows a confirm checkbox plus a Stop button whose success state is set only
  after the command returns. Verified: main `cargo test` (49), Tauri `cargo test`
  (23, incl. 4 new `stop_run` tests + `capability_guard`)/clippy, svelte-check,
  15 unit tests, build, and Playwright e2e (2, extended to stop a preview run).
- 2026-06-25: Completed the v1 desktop maintainability split (milestone 1 /
  V1-G10) before adding GUI controls. Split `src-tauri/src/commands/mod.rs`
  (528 -> 342) into `validation.rs` (bounds validators) and `warnings.rs` (plan
  warnings); split `ui/src/commands/runhaven.ts` (543) into `types.ts`,
  `client.ts`, and `plan.ts` behind a 6-line barrel so importers are unchanged;
  split `ui/src/app/App.svelte` (569 -> 409) into `SetupChecksPanel`,
  `PlanReviewPanel`, `LastLaunchPanel`, `RunStatusPanel`, and `RunOutputPanel`
  components, keeping the launch form and container state in `App.svelte`.
  Behavior preserved: Tauri tests (19)/clippy, svelte-check (0 errors), 15 unit
  tests, build, and Playwright e2e (2) pass. `tauri-stop-run-control` is now the
  active slice.
- 2026-06-25: Produced documented evidence that every CLI surface is tested.
  Added `scripts/cli_surface_check.sh`, a repeatable live breadth check that
  exercises every command family (agents, doctor, setup, plan, run,
  run --worktree, image build/rebuild/doctor, network list/prune,
  state list/reset/prune, runs list/show/log/diff/keep/recover/merge/discard/
  active/status/attach/kill/repair, egress log, auth status/explain/log, why
  host/workspace/network/state) and self-cleans. It found a real bug: the
  `GitSnapshot`/`GitChange` `available` serde tag serializes as the string
  "true"/"false", but four sites read it with `Value::as_bool` (always None),
  silently breaking `run --worktree`, `runs diff`, and `runs merge` for every
  clean repo. Fixed with a canonical `git_value_available` reader plus typed
  `GitSnapshot` enum matches in the worktree code, with a regression test.
  Coverage is indexed in `docs/CLI_SURFACE_COVERAGE.md`. Final evidence on macOS
  27.0: `cli_surface_check.sh` 39/39, `apple_container_smoke.sh --with-provider
  --with-ssh` passed, and both crates green on fmt/test/clippy plus pins, JSON,
  and diff checks.
- 2026-06-25: Made the network mode secure-by-default per the user directive
  that the secure path must be the easiest path. `--network` is now optional and
  resolves profile-aware in `make_run_plan` via `default_network_mode`: provider
  for profiles with bundled provider hosts (claude, codex, copilot, gemini) and
  internet for those without (shell, antigravity), where provider would be an
  empty allowlist. A provider-default run reaches the agent's own API but not
  arbitrary hosts; `plan` and `run` inform the `--provider-host` / `--network
  internet` escape hatch and never block. Updated CLI plus `CAPABILITIES`,
  `USAGE`, `ARCHITECTURE`, `SECURITY_MODEL`, `README`, `V1_RELEASE_PLAN`, and the
  harness security-boundary map; added a focused `default_network_mode` test.
  Verified with cargo fmt/test (48 unit + 6 integration)/clippy, Tauri test
  (19, 1 ignored)/clippy, and live plan checks (claude defaults to provider,
  shell to internet, explicit `--network internet` override works). This resolves
  the last open v0.5.0 decision, so `cli-complete-v0.5.0` is now `passing`.
- 2026-06-24: Closed the `cli-complete-v0.5.0` contract gaps (G1-G7) in one
  pass. Added plain-language security notices to standard error for every
  lower-security run choice (internet default, `--env`, custom or root `--user`,
  extra `--provider-host`, `--allow-sensitive-workspace`, `--image`), computed
  once in `build_run_plan`, carried on `AgentRunPlan`, and emitted at plan and
  run time; secure defaults stay silent (G6). Documented the previously
  undocumented `--user` and `runs attach` `--user`/`--workdir`/`--tty` overrides
  (G1), published the per-profile support matrix (G3), documented the
  runs/egress/auth JSONL records as best-effort/pre-stable, append-only,
  metadata-only with `--json` as the supported read path (G2), confirmed `--ssh`
  fail-closed posture is consistent across docs, behavior, and tests with no
  raw-key workaround (G5), and kept touched modules under the size guard with no
  new duplication (G7; `auth_broker.rs` 499 and `egress.rs` 495 remain watched,
  untouched). Verified with root cargo fmt/test (47 unit + 6 integration)/clippy,
  Tauri test (19, 1 ignored)/clippy, and a live `plan` security-notice check. One
  product decision is open: whether to flip the warned `internet` default to a
  stricter mode before `v1.0.0`. Tagged release notes are deferred to the
  release-readiness step. Status in `docs/RELEASE_GAP_ANALYSIS.md`.
- 2026-06-24: Proved the Apple `container` runtime on the current host. The
  session host moved to macOS 27.0 (build 26A5368g); prior runtime evidence was
  macOS 26.5.1. Started the Apple `container` system service (it was stopped at
  session start), confirmed `runhaven doctor` is green for every pinned
  prerequisite on macOS 27.0 (Rust 1.96.0, container CLI/apiserver 1.0.0 commit
  ee848e3, builder 0.12.0, vminit 0.33.3, Kata kernel 6.18.15-186), verified the
  bundled shell image (`runhaven/base:0.1.0`, digest `818ed6181723`), and ran
  `scripts/apple_container_smoke.sh --with-provider --with-ssh` to completion.
  The smoke exercised an internal read-only `/workspace` run with the full
  active/status/logs-follow/stop/show lifecycle, the live provider allowlist
  (allowed `example.com`; denied non-allowlisted host, proxied IP literal,
  direct egress, and direct IP egress), and SSH fail-closed at plan and run,
  then cleaned up with no stale active marker. No code bug surfaced; the only
  friction was the stopped system service, which `doctor` reports. This refreshes
  the V05-G4 runtime evidence on the current host. Static baseline (root and
  Tauri fmt/test/clippy, pin check, UI check/test) was green first. Command
  detail is in `docs/harness/evidence/evidence-log.md`.
- 2026-06-24: Locked the DRY and documentation-first development rule into the
  harness per user directive. Reframed the `AGENTS.md` build-necessity bullet as
  the named DRY ladder (YAGNI, standard library, native platform, installed
  dependency, one line, then minimum custom code) and added
  documentation-is-product and boring-over-clever principles; aligned the
  `change-contract.md` Build Necessity Gate rungs plus acceptance criteria with
  the same wording and the edge-case tiebreaker; added a doc-ships-with-behavior
  and ladder-applied check to the `AGENTS.md` Definition Of Done; recorded the
  standing rule in Product Facts and Key Decisions. Kept one canonical gate
  (no ladder copy/paste) so the change is itself DRY. Docs-only change: verified
  with `git diff --check` (clean) and confirmed the referenced gate path
  resolves; no code, JSON, or pins changed, so cargo/JSON/pin checks were not
  run.
- 2026-06-24: Ran a repo-wide docs accuracy audit across all 54 tracked
  Markdown files against the canonical current state. Root, core-product, and
  most harness docs were already accurate. Fixed: `docs/RESEARCH.md` reframed
  its 2026-06-18 "current image pins" line as dated and added a 2026-06-24
  current-pins note plus a no-workflows qualifier on the Actions source;
  `docs/harness/state/modularization-plan.md` dropped the removed
  `cli/lock.rs` pointer (locking lives in `runtime/lock.rs`) and refreshed the
  largest-file line counts; `.agents/skills/harness/references/repo-harness.md`
  corrected relative paths that were one level too shallow. Verified with a
  repo-wide relative Markdown link check (0 broken), path-resolution checks,
  pin check, JSON validation, and `git diff --check`.
- 2026-06-24: Ran a full dependency pin audit triggered by the `glib`
  Dependabot alert. Confirmed every Cargo, npm, image-CLI, base-image, and
  Debian pin is hard-pinned and that `.github/workflows/` is empty (no actions
  to pin). Brought the 10 pins behind latest stable to current: `time`
  0.3.49->0.3.51; ui `@lucide/svelte` 1.21.0, `svelte` 5.56.4,
  `@playwright/test` 1.61.1, `@tauri-apps/cli` 2.11.3, `svelte-check` 4.7.1,
  `vite` 8.1.0; bundled CLIs Claude Code 2.1.190, Codex 0.142.0, Copilot
  1.0.64, Gemini CLI 0.47.0 (integrity hashes regenerated). Refreshed Cargo and
  npm lockfiles. The `glib` alert was dismissed as not-affected (macOS-only;
  Linux-GTK-only transitive dep capped at 0.18.x). Verified with root and Tauri
  fmt/test/clippy, image dry-run builds, ui ci/check/test/build/e2e,
  `tauri:build`, pin check, JSON validation, and `git diff --check`.
- 2026-06-24: Applied a harness gap-analysis pass against the
  learn-harness-engineering course. Added a `feature_list.json` status_legend
  and marked the single current slice `active`; added a startup baseline gate to
  `AGENTS.md`; added this Key Decisions section; added a boundary-journey table
  and independent-evaluator routing to `security-boundary-map.md`; added
  verify-before-refactor and agent-oriented-error gates to `change-contract.md`;
  defined a representative task set in `quality-document.md` and linked it from
  `roadmap.md`; and added a mechanical `capability_guard` test that fails closed
  if a Tauri capability grants a host bridge. Tauri tests (incl. the capability
  guard), pin check, JSON validation, Markdown link check, and diff checks
  passed.
- 2026-06-18: Clarified the Container Machine policy across active docs and
  harness state. Task-scoped `container run` remains the secure-easy default,
  while explicit or user-managed Apple `container machine` workflows should be
  warned and require intent rather than blocked solely because they are less
  secure.
- 2026-06-18: Ran a full active-doc release-status pass. User-facing docs,
  roadmap/planning docs, Tauri planning docs, and harness routing were aligned
  to the then-current release ladder: alpha through the CLI-complete milestone,
  `v0.5.0` as CLI-complete, and `v1.0.0` as the first-class desktop release.
  Later sequencing changes are recorded above. Historical evidence remains
  historical.
- 2026-06-18: Added `docs/RELEASE_GAP_ANALYSIS.md` as the active v0.5/v1 gap
  tracker. It records observed CLI command coverage, current desktop command
  coverage, maintainability pressure, v0.5 blockers, v1 blockers, v1.x
  deferrals, and immediate next actions. Linked it from README, roadmap,
  release plan, non-UI backlog, feature state, and harness routing.
- 2026-06-18: Locked secure-easy and maintainability gates into `AGENTS.md`,
  `docs/V1_RELEASE_PLAN.md`, `docs/SECURITY_MODEL.md`, and focused harness
  docs. Future slices must make secure defaults the easiest path, warn and
  require intent for supported lower-security choices, fail closed on hard
  boundary violations, avoid deferred large-file debt, remove meaningful
  duplication, prefer standard/native/installed solutions, keep exact
  current-stable pins, and update harness state when scope changes.
- 2026-06-18: Added and revised `docs/V1_RELEASE_PLAN.md` as the proposed
  durable release ladder. The plan now sets `v0.5.0` as CLI-complete, makes
  `v1.0.0` a first-class desktop release for the safe beginner workflow, keeps
  the CLI as the stable backend and automation surface, records missing
  runtime/data/storage/network/auth/UX/accessibility/performance edge cases,
  and defines release milestones and verification gates. Linked it from
  `README.md` and `docs/ROADMAP.md`; added `cli-complete-v0.5.0` and
  `desktop-first-class-v1` to `feature_list.json`.
- 2026-06-18: Refreshed direct package pins and lockfiles to current stable
  package-manager releases. Tauri Rust pins moved to `tauri` 2.11.3 and
  `tauri-build` 2.6.3; frontend `@tauri-apps/api` moved to 2.11.1; bundled
  image CLIs moved to Claude Code 2.1.181, Codex 0.140.0, and Copilot 1.0.63.
  Cargo and npm lockfiles were refreshed. Playwright now starts an isolated
  strict-port RunHaven dev server instead of reusing an unrelated process on
  port 5173.
- 2026-06-18: Implemented OWASP-informed local hardening from the Cheat Sheet
  review. Tauri commands now reject oversized IPC fields before planning or
  launch confirmation, and RunHaven cache markers, logs, and locks are created
  with owner-only permissions on Unix.
- 2026-06-17: Simplified the repo harness to the lightweight five-subsystem
  model from the referenced harness-learning material. Startup now routes
  through only `AGENTS.md`, `feature_list.json`, and `current-state.md`;
  harness docs are on-demand reference material.
- 2026-06-16: Implemented the first raw-log snapshot slice. `get_log_snapshot`
  lives behind `run-control`, requires sensitive-output acknowledgement,
  validates the run id and RunHaven-owned active container marker, calls only
  bounded `container logs -n`, and keeps raw output out of durable frontend
  state.
- 2026-06-16: Tauri launch flow can confirm launch, check image readiness,
  show resource warnings, render sanitized run snapshots, and refresh live run
  status without exposing raw logs or raw Apple inspect payloads.

## Trusted Verification

- 2026-06-24 Apple container runtime smoke (macOS 27.0):
  - `sw_vers` reported macOS 27.0 build 26A5368g; `uname -m` reported `arm64`.
  - `container --version` reported Apple `container` CLI 1.0.0 commit `ee848e3`.
  - `container system status` reported the apiserver not running at session
    start; `container system start` brought it to `status running`.
  - `runhaven doctor` returned ok for Rust 1.96.0, macOS 27.0, arm64, the
    container CLI/runtime commit ee848e3, builder image 0.12.0, vminit 0.33.3,
    and the Kata 6.18.15-186 kernel.
  - `runhaven image doctor shell` reported `ok shell: runhaven/base:0.1.0`.
  - `scripts/apple_container_smoke.sh --with-provider --with-ssh` printed
    "Apple container smoke checks passed." and exited 0.
  - Static baseline before runtime work passed: root `cargo fmt --check`,
    `cargo build --locked`, `cargo test --locked` (46 unit + 6 integration),
    `cargo clippy --all-targets --locked -- -D warnings`, and
    `cargo run --locked --bin runhaven-check-pins`; Tauri `cargo fmt --check`,
    `cargo test --locked` (19 passed, 1 ignored), and
    `cargo clippy --all-targets --locked -- -D warnings`; and
    `npm --prefix ui run check` (0 errors) with `npm --prefix ui test`
    (15 passed).
- 2026-06-18 README and Container Machine policy docs checks:
  - `sw_vers` reported macOS 26.5.1 build 25F80.
  - `uname -m` reported `arm64`.
  - `container --version` reported Apple `container` CLI 1.0.0 commit
    `ee848e3`.
  - `container machine --help` passed and showed create, delete, inspect,
    list, logs, run, set, set-default, and stop subcommands.
  - Stale hard-block wording scan for old Container Machine policy phrasing
    passed.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 54 Markdown files passed.
  - `git diff --check` passed.
- 2026-06-18 v0.5.0/v1.0.0 gap-analysis docs checks:
  - CLI help smokes passed for top-level `runhaven`, `runs`, `image`,
    `network`, `state`, `egress`, `auth`, and `why`.
  - Tauri command/capability scan and source file-size scan completed for
    gap-analysis evidence.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 54 Markdown files passed.
  - Stale wording scan for old pre-Tauri/release-boundary/package-evidence
    phrasing passed with only intentional README release-plan link text.
  - Explicit trailing-whitespace check over changed docs/state files passed.
  - `git diff --check` passed.
- 2026-06-18 active-doc release-status checks:
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 53 Markdown files passed.
  - Stale wording scans for old pre-Tauri, release-boundary, alpha, `v0.5.0`,
    and `v1.0.0` phrasing passed with only intentional historical/evidence
    matches.
  - Explicit trailing-whitespace check over changed docs/state files passed.
  - `git diff --check` passed.
- 2026-06-18 secure-easy and maintainability docs/harness checks:
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 53 Markdown files passed.
  - Explicit trailing-whitespace check over the changed docs/state files
    passed.
  - `git diff --check` passed.
- 2026-06-18 v0.5.0/v1.0.0 release-ladder docs checks:
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 53 Markdown files passed.
  - Explicit trailing-whitespace check over the changed docs/state files
    passed.
  - `git diff --check` passed.
- 2026-06-18 package pin refresh checks:
  - `rustup check` reported stable `1.96.0` up to date.
  - `cargo info`, `cargo search`, and `npm view` checked current stable direct
    package versions.
  - `cargo update` and `cargo update --manifest-path src-tauri/Cargo.toml`
    refreshed Cargo lockfiles to the latest Rust 1.96-compatible versions.
  - `npx -y npm@11.17.0 --prefix <package> install --package-lock-only
    --ignore-scripts` refreshed UI and bundled-image npm lockfiles.
  - `npx -y npm@11.17.0 --prefix <package> audit --audit-level=moderate`
    passed for the UI and bundled-image npm packages.
  - `cargo update --dry-run --verbose` reported zero remaining root Cargo
    lockfile updates.
  - `cargo update --manifest-path src-tauri/Cargo.toml --dry-run --verbose`
    reported zero remaining Tauri lockfile updates; remaining newer transitive
    releases are outside upstream semver constraints.
  - `cargo tree --manifest-path src-tauri/Cargo.toml --locked --target
    aarch64-apple-darwin -i glib` found no macOS dependency path for `glib`.
  - `cargo fmt --check` passed.
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check` passed.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - `git diff --check` passed.
  - `cargo test --locked` passed.
  - `cargo test --manifest-path src-tauri/Cargo.toml --locked` passed.
  - `cargo clippy --all-targets --locked -- -D warnings` passed.
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked
    -- -D warnings` passed.
  - `npx -y npm@11.17.0 --prefix ui test -- --run` passed.
  - `npx -y npm@11.17.0 --prefix ui run check` passed.
  - `npx -y npm@11.17.0 --prefix ui run build` passed.
  - `npx -y npm@11.17.0 --prefix ui run test:e2e` passed after Playwright was
    isolated from the unrelated JobSentinel dev server on port 5173.
  - `cargo build --locked` passed.
  - `npx -y npm@11.17.0 --prefix ui run tauri:build` passed.
  - `cargo run --locked --bin runhaven -- image build <agent> --dry-run`
    passed for Claude, Codex, Copilot, and Gemini.
- 2026-06-18 security hardening checks:
  - Red checks first failed for oversized IPC payloads and default active-run
    marker permissions.
  - `cargo fmt --check` passed.
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check` passed.
  - `cargo test --locked` passed.
  - `cargo test --manifest-path src-tauri/Cargo.toml --locked` passed.
  - `cargo clippy --all-targets --locked -- -D warnings` passed.
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked
    -- -D warnings` passed.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `npm --prefix ui test -- --run` passed.
  - `npm --prefix ui run check` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - `git diff --check` passed.
- 2026-06-17 harness simplification checks:
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 52 tracked Markdown files passed.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git diff --check` passed.
  - Stale-reference scans for retired root `progress.md`/`session-handoff.md`,
    old Python pin-check commands, and old mandatory harness roadmap routing
    found only intentional archive or historical evidence references.
  - `./init.sh` was not run because this pass changed documentation, harness
    instructions, and state only; no runtime code, lockfile, package, image, or
    Tauri capability behavior changed.
- 2026-06-16 Tauri raw-log snapshot checks passed:
  - `cargo test --manifest-path src-tauri/Cargo.toml --locked`
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings`
  - `cargo test --locked`
  - `cargo clippy --all-targets --locked -- -D warnings`
  - `npm --prefix ui test -- --run`
  - `npm --prefix ui run check`
  - `npm --prefix ui run test:e2e`
  - `npm --prefix ui run build`
  - `scripts/apple_container_smoke.sh`

## Blockers

- `--ssh` remains fail-closed. Apple `container` 1.0.0 exposes an SSH agent
  socket to the non-root guest user, but `ssh-add -l` returns permission
  denied. Do not re-enable SSH forwarding, mount raw SSH keys, or switch the
  default agent user to root without explicit security review and no-secret
  runtime proof.

## Touched Surfaces In This Pass

- tauri-diagnostics: `src/runhaven/diagnostics.rs` (`auth_status_payload`
  core and secret-free log readers), `src/runhaven/cli/diagnostics.rs` (CLI adapter); `src-tauri/src/commands/diagnostics.rs` (new), `commands/mod.rs`,
  `lib.rs`, `build.rs`, `contracts.rs`, `capabilities/main-read.json`;
  `ui/src/commands/{types.ts,client.ts}`,
  `ui/src/components/DiagnosticsPanel.svelte` (new, self-contained),
  `ui/src/app/App.svelte`, `ui/e2e/app.spec.ts`.
- tauri-kill-repair-run-control: `src/runhaven/runtime/active/mod.rs`
  (`kill_active_run` + thin `runs_kill`), `active/repair.rs` (`repair_active_run`);
  `src-tauri/src/commands/run_control.rs` (kill_run/repair_run + tests),
  `lib.rs`, `build.rs`, `contracts.rs`, `capabilities/run-control.json`;
  `ui/src/commands/{types.ts,client.ts}`, `ui/src/components/RunStatusPanel.svelte`,
  `ui/src/app/App.svelte` (shared `runControl`), `ui/e2e/app.spec.ts`.
- tauri-stop-run-control: `src/runhaven/runtime/active/mod.rs` (`stop_active_run`
  core + thin `runs_stop`); `src-tauri/src/commands/run_control.rs` (new),
  `commands/mod.rs`, `lib.rs`, `build.rs`, `contracts.rs`,
  `capabilities/run-control.json`; `ui/src/commands/{types.ts,client.ts}`,
  `ui/src/components/RunStatusPanel.svelte`, `ui/src/app/App.svelte`,
  `ui/e2e/app.spec.ts`.
- Desktop maintainability split (v1 milestone 1): `src-tauri/src/commands/`
  (`mod.rs`, new `validation.rs`, new `warnings.rs`, sibling import updates);
  `ui/src/commands/` (`runhaven.ts` barrel, new `types.ts`/`client.ts`/`plan.ts`);
  `ui/src/app/App.svelte` plus new `ui/src/components/{SetupChecksPanel,
  PlanReviewPanel,LastLaunchPanel,RunStatusPanel,RunOutputPanel}.svelte`.
- CLI surface verification + git-availability bug fix (code): new
  `scripts/cli_surface_check.sh`; `src/runhaven/support/git.rs`
  (`git_value_available`); `src/runhaven/runtime/worktrees/mod.rs` and
  `merge.rs` (typed `GitSnapshot` matches + regression test);
  `src/runhaven/records/run_history.rs`, `records/run_history/diff.rs`,
  and the `records/` facade (canonical reader). Docs: new `docs/CLI_SURFACE_COVERAGE.md`, `docs/harness/evidence/evidence-log.md`,
  `docs/harness/feedback/verification-matrix.md`, `README.md` (docs index),
  `feature_list.json`, and this state file.
- Secure-by-default network (code): `src/runhaven/cli/args.rs` (`--network`
  optional), `src/runhaven/runtime/plans/validation.rs` (`default_network_mode`),
  `mod.rs` (re-export + focused test), `src/runhaven/cli/app.rs` (resolve default
  + provider escape-hatch info line). Docs: `CAPABILITIES`, `USAGE`,
  `ARCHITECTURE`, `SECURITY_MODEL`, `README`, `V1_RELEASE_PLAN`,
  `docs/harness/boundaries/security-boundary-map.md`, `docs/RELEASE_GAP_ANALYSIS.md`,
  `feature_list.json`, and this state file.
- v0.5.0 CLI contract closure (code): `src/runhaven/runtime/plans/types.rs`
  (`security_notices` field), `validation.rs` (`security_notices` function),
  `mod.rs` (compute + carry + focused tests), and `src/runhaven/cli/app.rs`
  (`eprint_security_notices` emitted from `plan` and `run`).
- v0.5.0 CLI contract closure (docs/state): `docs/CAPABILITIES.md` (profile
  support matrix, lower-security overrides, local record files), `docs/USAGE.md`,
  `docs/ARCHITECTURE.md`, `docs/SECURITY_MODEL.md`, `docs/ROADMAP.md` (active TUI plus later desktop), `docs/RELEASE_GAP_ANALYSIS.md`, `feature_list.json`, and this
  state file.
- Earlier this session (already committed): macOS 27 runtime evidence
  (`current-state.md`, `docs/harness/evidence/evidence-log.md`).

## Next Step

`v0.5.0` is released (pre-release; see `CHANGELOG.md`). The active slice is
`terminal-ui`: integrate the full Codex-vendored TUI source baseline into
RunHaven, then adapt it to the wishlist in
`docs/plans/tui-codex-vendor-reset.md`.

Immediate next step: adapt the copied Codex crate-root assumptions into
RunHaven entrypoint, dependency, and product adapters without culling product
surfaces prematurely. `src/runhaven/cli/tui/mod.rs` currently keeps the crate
buildable and fails closed for bare interactive TUI launch until that
integration is complete. For each removal, record why removal is better than
leaving and adapting. Keep the reference-implementation requirement in view
because this TUI setup will guide several sibling projects.

Do not publish a release from the interim vendor-reset state. After the TUI is
fully integrated, verified, and confirmed, do a full release bump to `v0.6.0`
with a full public docs refresh.

Separate non-TUI follow-up already called out by the user: logs sanitization and
secure secrets/OAuth-token handling, with macOS Keychain preferred for
RunHaven-owned host secrets where practical. Other non-blocking follow-ups on
record: RunHaven runs containers without `--rm` (killed containers reap
asynchronously) and a signed auto-updating provider-policy candidate.
