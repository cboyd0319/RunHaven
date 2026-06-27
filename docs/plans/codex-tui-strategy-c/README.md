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

That means:

- Keep the Codex TUI source layout and module names as close to
  `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/` as possible.
- Keep the Codex runtime ownership model:
  `Tui` terminal runtime -> `App` event loop -> `ChatWidget` and `BottomPane`
  -> `AppServerSession` typed facade -> backend client.
- Put RunHaven-specific behavior behind adapters and payloads, mainly under
  `crates/runhaven-tui/src/tui/runhaven/` and `crates/runhaven-core`.
- Do not let `app_shell.rs` and the staged `mod.rs` facade become the permanent
  architecture. They are compile bridges.
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
- `app_server_session.rs` remains the only typed backend facade used by `App`
  and `ChatWidget`.
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

## Bottom Line

RunHaven has already brought over the right Codex TUI source baseline. The
important next move is to stop treating individual widgets as the architecture.

Wire the copied source back into the Codex shape:

```text
Tui -> App -> ChatWidget/BottomPane -> AppServerSession -> RunHaven service -> runhaven-core
```

Keep RunHaven product logic behind that facade. Keep unsupported Codex product
features disabled. Keep source paths close enough that upstream Codex TUI can be
diffed and merged later.

## Source Evidence

Primary sources checked:

- `/Users/c/Downloads/codex-tui-capabilities.md`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml`
- `/Users/c/Documents/GitHub/codex/codex-rs/app-server-protocol/src/`
- `/Users/c/Documents/GitHub/codex/codex-rs/app-server-client/src/lib.rs`
- `/Users/c/Documents/GitHub/RunHaven/AGENTS.md`
- `/Users/c/Documents/GitHub/RunHaven/current-state.md`
- `/Users/c/Documents/GitHub/RunHaven/docs/plans/tui-build-plan.md`
- `/Users/c/Documents/GitHub/RunHaven/docs/plans/tui-architecture.md`
- `/Users/c/Documents/GitHub/RunHaven/crates/runhaven-tui/src/tui/`
- `/Users/c/Documents/GitHub/RunHaven/crates/runhaven-core/src/ui_contracts.rs`
