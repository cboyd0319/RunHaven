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
- `app_event`, `app_event_sender`, `bottom_pane`, and `render` in `mod.rs` are
  staged contracts for compiled vendored surfaces. Replace them with full Codex
  adapters as those surfaces come online.

Known integration gap:

- The copied Codex crate source still uses Codex crate/module assumptions.
  RunHaven integration will adapt entrypoints, module paths, dependencies, and
  product data in later commits.
