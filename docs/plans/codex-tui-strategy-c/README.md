# RunHaven Codex TUI Strategy C Plan

Created: 2026-06-27
Split: 2026-06-27

This is the entrypoint for the split Strategy C plan. The old single-file
plan was getting too large, so the details now live in focused documents in
this folder.

## Decision

RunHaven should continue with Strategy C, but define it very narrowly:

RunHaven should become Codex-TUI-client-compatible, not a fork that rewrites the
Codex TUI into a custom launcher.

Direction update, 2026-06-29: the scoped MVP no longer treats native Codex
`App` and `ChatWidget` promotion as the default destination. The near-term
target is the smallest fully working RunHaven TUI on the Codex terminal runtime:
agent and workspace selection, plan review, confirmation, foreground launch,
active-run logs, recovery, and diagnostics. Native `App` and `ChatWidget` remain
dormant unless a later RunHaven scope needs their specific ownership model and a
reviewed redaction, session-recording, and app-server boundary exists.

That means:

- Keep the Codex TUI source layout and module names as close to
  upstream `openai/codex:codex-rs/tui/src/` as possible.
- Keep Codex terminal/runtime ownership active where RunHaven uses it now:
  `Tui` terminal runtime -> scoped RunHaven shell -> `BottomPane` views ->
  RunHaven typed facade -> backend client.
- Treat native Codex `App` and `ChatWidget` as separate future promotions.
  Promote native `App` only if RunHaven needs Codex app-loop ownership beyond
  the current shell. Promote `ChatWidget` only if RunHaven needs source-shaped
  conversation transcript ownership.
- Put RunHaven-specific behavior behind adapters and payloads, mainly under
  `crates/runhaven-tui/src/tui/runhaven/` and `crates/runhaven-core`.
- Do not let `app_shell.rs` become a product screen. For the scoped MVP it is a
  terminal/runtime host while product behavior stays under `tui/runhaven/`.
- Do not expose Codex host-reaching product surfaces by default. Remote
  filesystem, IDE, MCP, plugin, connector, marketplace, cloud feedback, and
  broad account/auth behavior stay disabled or fail-closed unless RunHaven's
  security model explicitly promotes them.
- Keep Codex compatibility state isolated from a user's normal `~/.codex`
  unless the user explicitly chooses to import or share something. RunHaven can
  use Codex-shaped pet, theme, session, and cache directories, but the default
  root should be RunHaven-owned.

The maintenance goal is simple: later upstream Codex TUI changes should be
reviewed with `diff`, not manually rediscovered in a bespoke RunHaven TUI.

## Success Criteria

The finished integration is successful when:

- Bare interactive `runhaven` still routes from `crates/runhaven/src/main.rs`
  to `runhaven_tui::run()` only when stdin and stdout are TTYs.
- `crates/runhaven-tui/src/tui/` remains structurally comparable to Codex
  upstream. Local RunHaven files are obvious and limited.
- The top-level app uses the Codex terminal runtime, event stream, redraw
  scheduling, raw-mode restore, bracketed paste, focus handling, title handling,
  and pet image lifecycle.
- RunHaven launch, active run, history, diff, diagnostics, doctor, image, and
  auth status data come from `runhaven-core`, not CLI prose and not duplicated
  widget logic.
- RunHaven TUI actions use the RunHaven typed facade. If native `App` or
  `ChatWidget` is promoted later, they must use that facade rather than Codex
  host-reaching backend paths.
- User-visible RunHaven actions map to typed backend calls. Widgets do not call
  `container`, inspect files directly, parse logs directly, or rebuild plans.
- Foreground launch is prepared through the typed backend facade but executed by
  the UI loop only after Codex terminal restore. Backend service tasks do not
  own raw terminal state.
- Unsupported Codex surfaces are unavailable in the TUI with clear local
  messages, not half-wired.
- Snapshot and VT100 tests cover each user-visible TUI screen or transcript
  behavior.

## Documents

1. [01-source-inventory.md](01-source-inventory.md): current RunHaven TUI state, copied Codex source, local exceptions, dormant source, and the concrete source checklist.
2. [02-vendoring-and-protocol.md](02-vendoring-and-protocol.md): vendor-first dependency strategy, protocol/client shape, and dependency additions.
3. [03-runtime-wiring.md](03-runtime-wiring.md): target runtime wiring, RunHaven backend interface, Codex concept mapping, method surface, notifications, and launch flow.
4. [04-implementation-and-verification.md](04-implementation-and-verification.md): migration phases, ownership rules, security requirements, verification, sync workflow, and remaining decisions.
5. [05-adversarial-drift-ledger.md](05-adversarial-drift-ledger.md): independent drift audit, severity-ranked remediation backlog, and guardrails against regrowing the temporary shell.

## Bottom Line

RunHaven has already brought over the right Codex TUI source baseline. The
important next move is to finish the RunHaven MVP on the Codex runtime without
porting unrelated Codex product features.

```text
Tui -> scoped RunHaven shell -> BottomPane -> RunHaven facade -> RunHaven service -> runhaven-core
```

Keep RunHaven product logic behind that facade. Keep native `App`,
`ChatWidget`, and unsupported Codex product features dormant until a RunHaven
need and reviewed boundary justify promotion. Keep source paths close enough
that upstream Codex TUI can be diffed and merged later.

## Source Evidence

Primary sources checked:

- `docs/plans/codex-tui-capabilities.md`
- `https://github.com/openai/codex.git`, commit
  `5267e805fb830891c0b23376bcd9cbd382c3473c`
- `openai/codex:codex-rs/tui/src/`
- `openai/codex:codex-rs/tui/Cargo.toml`
- `openai/codex:codex-rs/app-server-protocol/src/`
- `openai/codex:codex-rs/app-server-client/src/lib.rs`
- `AGENTS.md`
- `PROGRESS.md`
- `docs/plans/tui-build-plan.md`
- `docs/plans/tui-architecture.md`
- `crates/runhaven-tui/src/tui/`
- `crates/runhaven-core/src/ui_contracts.rs`
