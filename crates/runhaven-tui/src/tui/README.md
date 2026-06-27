# RunHaven TUI Vendor Baseline

This directory is a source snapshot from:

```text
/Users/c/Documents/GitHub/codex/codex-rs/tui/src/
```

It also includes Codex terminal detection copied from:

```text
/Users/c/Documents/GitHub/codex/codex-rs/terminal-detection/src/
```

The upstream source is the OpenAI Codex TUI and is licensed under Apache-2.0.
RunHaven keeps attribution in `THIRD_PARTY_NOTICES.md` and
`licenses/codex-Apache-2.0.txt`.

This is a baseline copy before RunHaven product integration. It intentionally
keeps Codex TUI structure first, then RunHaven will adapt the parts it needs.
The active Strategy C integration plan lives in
`docs/plans/codex-tui-strategy-c/`.

Local exclusions in this baseline:

- `.DS_Store` files, because they are local filesystem metadata.
- upstream `*.snap` files, because they are Codex test goldens and must be
  regenerated from integrated RunHaven tests if those tests are kept.

Local source-format exception:

- `markdown_render_tests.rs` uses `concat!` for one Markdown hard-break fixture
  so the runtime test input still contains two trailing spaces, while the source
  file satisfies RunHaven's whitespace check.

Local integration exceptions:

- `mod.rs` is the temporary RunHaven module entrypoint during integration. It
  keeps the crate buildable and fails closed for interactive TUI launch until
  the vendored Codex entrypoint is adapted.
- `terminal_detection.rs` and `terminal_tests.rs` are copied from the Codex
  terminal-detection crate because the native pet image protocol depends on the
  same iTerm2, Kitty, Sixel, tmux, and Zellij decisions as Codex.
- `pets/picker.rs` and `pets/preview.rs` remain vendored but are not compiled
  against the full Codex bottom-pane view yet. They compile against the staged
  `bottom_pane` selection contract in `mod.rs` until the full bottom-pane view
  is adapted.
- `pets/model.rs` formats the SHA-256 cache key bytes explicitly because
  RunHaven is pinned to `sha2` 0.11. The produced cache key string stays the
  same shape as Codex.
- `app_event`, `app_event_sender`, and `bottom_pane` in `mod.rs` are staged
  contracts for compiled vendored surfaces. Replace them with full Codex
  adapters as those surfaces come online.
- `render/renderable.rs` is now compiled through the RunHaven adapter with one
  Ratatui 0.30 compatibility tweak: `Line` renders through the borrowed
  `WidgetRef` implementation.
- `terminal_title.rs` is compiled as a vendored app-shell helper. It keeps
  Codex's OSC title sanitization rules intact.
- `motion.rs`, `shimmer.rs`, `color.rs`, `terminal_palette.rs`, and
  `terminal_probe.rs` are compiled as the Codex motion and terminal-theme
  foundation. Their module paths point through `crate::tui`.
- `motion.rs` keeps the Codex animation-primitive boundary test, pointed at
  `crates/runhaven-tui/src/tui/` and using RunHaven's existing `regex` dependency
  instead of Codex's crate-root test helper.
- `terminal_palette.rs` uses the vendored bounded terminal probe for Unix
  default-color requery because RunHaven has not adopted Codex's pinned
  crossterm fork with color-query helpers.
- `style.rs`, `line_truncation.rs`, `text_formatting.rs`, `wrapping.rs`, and
  `render/line_utils.rs` are compiled as shared Codex UI helpers. Their module
  paths point through `crate::tui`.
- `serde_json` enables `preserve_order` at the RunHaven crate level to match
  Codex TUI's compact JSON formatting behavior.
- `app_shell.rs` is temporary RunHaven-owned shell glue. It restores bare
  interactive `runhaven` to a read-only launch preview while the full Codex app
  shell is still being adapted. It now hosts a Codex `ListSelectionView`
  launch picker through `runhaven/launch_wizard.rs`, which consumes
  `LaunchPlanData` from `crates/runhaven-core/src/ui_contracts.rs`.

Known integration gap:

- The copied Codex crate source still uses Codex crate/module assumptions.
  RunHaven integration will adapt entrypoints, module paths, dependencies, and
  product data in later commits.
- The launch picker, read-only review, and confirmation screen are staged in
  `app_shell.rs`, not the real Codex `App` loop. Workspace selection, policy
  changes, and final foreground launch still need to be reattached through the
  Codex-shaped runtime and typed backend facade.
- Foreground launch must be prepared by the RunHaven service but executed by the
  UI loop only after Codex terminal restore. Backend service tasks must not own
  raw terminal state.
