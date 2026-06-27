# Current State

Last Updated: 2026-06-26 UTC

## Current Objective

The `v0.5.0` CLI-complete pre-release was cut and published on 2026-06-26 (first
release; pre-1.0, CLI only). The runhaven CLI crate is at `0.5.0`
(`runhaven --version` = 0.5.0), `git tag v0.5.0`, GitHub pre-release; image
template tags stay `0.1.0` (built locally, unchanged). The alpha period has
ended; RunHaven is now a pre-1.0 pre-release. `cli-complete-v0.5.0` and
`release-readiness` are `passing`. The next phase is the terminal UI (TUI), then
the rest of the deferred GUI work.

Next phase (2026-06-26 user directive): all GUI/UI work, both the Tauri desktop
app and the terminal UI (TUI), moves to the very end of the roadmap. Runtime and
security hardening of the Apple `container` boundary leads now, ahead of any new
feature surface. The already-shipped desktop run-control and diagnostics slices
stay `passing`; no GUI work is reverted, only re-sequenced behind the non-UI
hardening, remaining product scope, and release work. The
`runtime-security-hardening` slice is `passing` and was merged to `main`
2026-06-26 (with the four-agent `runhaven login` work, the egress family-pattern
step, the non-technical UX pass, and a full docs and README refresh); no feature
is currently `active`.

## Startup State Contract

- `AGENTS.md`: root instruction map.
- `feature_list.json`: compact feature status and next product slice.
- `current-state.md`: progress, trusted facts, blockers, and handoff.

Do not recreate separate root `progress.md` or `session-handoff.md` files.
Load deeper docs only when the task touches that surface.

## Product Facts

- RunHaven is a Rust 1.96.0 CLI for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon.
- The CLI is the current working product surface.
- The alpha desktop shell lives under `ui/` and `src-tauri/`.
- `v0.5.0` is now the intended CLI-complete release. All CLI product work
  should be done by that tag before broad v1 desktop expansion.
- RunHaven remains alpha/pre-release until after `v0.5.0` is cut.
- All GUI/UI work is deferred to the very end of the roadmap (2026-06-26 user
  directive), superseding the prior `v1.0.0` = first-class-desktop boundary. The
  CLI stays the product surface through the near-term releases; the desktop app
  and TUI are the final roadmap phase. The version label for the desktop release
  is open and not locked to `v1.0.0`.
- The product sequence is CLI-complete (`v0.5.0`), then runtime/security
  hardening of the Apple `container` boundary, then remaining non-UI product
  scope and a CLI-based public release, then (at the very end) the first-class
  desktop app and a terminal UI (TUI) over the same planner and policy objects.
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

- All GUI/UI work (desktop app and TUI) is deferred to the very end of the
  roadmap, and runtime/security hardening leads the near-term work (2026-06-26
  user directive). This reverses the prior sequence where `v1.0.0` was the
  first-class desktop release. Rationale: harden and complete the non-UI product
  and its release before building the heavier desktop/TUI surfaces and the
  signing/notarization path. Already-shipped GUI slices stay `passing`; they are
  re-sequenced, not reverted. Reopen only with a new user directive.
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
- Exactly one feature is `active` in `feature_list.json` to hold scope; the
  `active` row is the current slice, distinct from `planned` work.
- All development is DRY and documentation-first by standing rule (2026-06-24
  user directive), not a per-slice decision. The build-necessity ladder in
  `AGENTS.md` Working Rules and the `change-contract.md` Build Necessity Gate are
  the single canonical source; documentation-is-product means an undocumented
  behavior is treated as not shipped. Boring-over-clever and the edge-case
  tiebreaker (between equally small standard-library options, take the one
  correct on edge cases) resolve style and algorithm choices.
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
  less-technical people; the user must manage no hosts and see no hostnames). The
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

## Latest Verified Work

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
  helpers (now `pub(crate)`) plus `default_network_mode`, so the TUI and
  `runhaven agents` share one source. Input is a testable `handle_key`. Verified:
  cargo fmt, `cargo test --locked` (74 lib incl. 5 TUI tests + 6 integration),
  clippy `-D warnings`, non-TTY help fallback, `git diff --check`.
- 2026-06-26: Started the terminal UI (TUI). Decision: reference the Codex TUI
  (`codex-rs/tui`) for quality and patterns, not fork it; it is ~214k lines,
  Apache-2.0, an agent-chat domain welded to ~30 `codex-*` internal crates (even
  its event loop imports `codex_protocol`), while RunHaven needs a
  launcher/manager, a different domain (the agent's own chat runs inside the
  container). Pinned `ratatui =0.30.2` (current stable). Slice 1 (scaffold):
  `src/runhaven/cli/tui.rs` with `ratatui::init`/`restore` (panic-safe terminal),
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
  framing, and the corrected GUI-and-TUI-last roadmap), and updated the product,
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
  roadmap/planning docs, Tauri planning docs, and harness routing now agree
  that RunHaven remains alpha/pre-release until after `v0.5.0`, `v0.5.0` is
  CLI-complete, and `v1.0.0` is the first-class desktop release. The README now
  names both goals directly. Historical evidence remains historical.
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

- tauri-diagnostics: `src/runhaven/cli/diagnostics.rs` (`auth_status_payload`
  core); `src-tauri/src/commands/diagnostics.rs` (new), `commands/mod.rs`,
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
  `src/runhaven/records/history.rs` and `records/history/diff.rs` (canonical
  reader). Docs: new `docs/CLI_SURFACE_COVERAGE.md`, `docs/harness/evidence/evidence-log.md`,
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
  `docs/ARCHITECTURE.md`, `docs/SECURITY_MODEL.md`, `docs/ROADMAP.md` (deferred
  TUI phase), `docs/RELEASE_GAP_ANALYSIS.md`, `feature_list.json`, and this
  state file.
- Earlier this session (already committed): macOS 27 runtime evidence
  (`current-state.md`, `docs/harness/evidence/evidence-log.md`).

## Next Step

`v0.5.0` is released (pre-release; see `CHANGELOG.md`). The active feature is
`terminal-ui`: the launcher TUI, built fresh in `ratatui`, referencing the Codex
TUI for quality and patterns only (not forked).

Slice 1 (scaffold) is done: bare `runhaven` on a TTY opens a home screen listing
the agents; the CLI stays the complete surface. Remaining TUI slices:

- Agent picker (done, slice 2). Next: workspace picker (choose the project
  directory for the selected agent) and an action to launch.
- Plan and egress review screen (render the run plan and security notices before
  launch, over the existing planner and policy objects).
- Launch and run dashboard (start a run, then live status, attach, stop, kill,
  repair).
- Image, state, and worktree management screens.
- Brand graphics and a mascot sprite (an easter egg). Reference
  `codex-rs/tui/src/pets` for the sprite-animation technique only; build a
  RunHaven sprite fresh (MIT). Full vision in
  `docs/plans/ratatui-brand-graphics.md`.

Non-blocking follow-ups on record: RunHaven runs containers without `--rm`
(killed containers reap asynchronously); a signed auto-updating provider-policy
candidate; and a light docs sweep to retire residual pre-v0.5.0 "alpha" wording
(README and USAGE are already updated).
