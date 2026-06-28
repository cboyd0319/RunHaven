# RunHaven TUI Vendor Baseline

This directory maps the upstream Codex TUI source path:

```text
https://github.com/openai/codex.git
commit: 5267e805fb830891c0b23376bcd9cbd382c3473c
path: codex-rs/tui/src/
```

to the local RunHaven vendored path:

```text
crates/runhaven-tui/src/tui/
```

It also includes Codex terminal detection copied from the same upstream commit:

```text
codex-rs/terminal-detection/src/
```

RunHaven also vendors the real Codex crate closures needed by activated
protocol, config, and keymap surfaces under `crates/codex/`, using the original
package and library names:

```text
codex-api
codex-app-server-protocol
codex-async-utils
codex-client
codex-config
codex-connectors
codex-exec-server
codex-exec-server-protocol
codex-execpolicy
codex-experimental-api-macros
codex-features
codex-file-search
codex-file-system
codex-git-utils
codex-model-provider-info
codex-network-proxy
codex-otel
codex-plugin
codex-protocol
codex-sandboxing
codex-shell-command
codex-utils-absolute-path
codex-utils-approval-presets
codex-utils-cache
codex-utils-cargo-bin
codex-utils-home-dir
codex-utils-image
codex-utils-path
codex-utils-path-uri
codex-utils-plugins
codex-utils-pty
codex-utils-rustls-provider
codex-utils-string
codex-windows-sandbox
```

Those vendored crates are Apache-2.0 upstream source. Their local manifests use
explicit `license = "Apache-2.0"`, `version = "0.0.0"`, and
`publish = false` metadata so they do not inherit RunHaven workspace package
metadata or become publishable local forks. Their external exact pins are kept
direct in the vendored manifests. The upstream `runfiles` git rev is preserved
because Codex's schema fixture tests rely on that dependency source, and the
upstream OpenAI fork git revs for `tokio-tungstenite` and `tungstenite` are
preserved because Codex enables proxy features from those forks.
Where Cargo's unified resolver cannot hold two semver-compatible exact versions
in one lockfile, the vendored manifest pin is aligned to RunHaven's existing
workspace pin and recorded as an integration adjustment.

The upstream source is the OpenAI Codex TUI and is licensed under Apache-2.0.
RunHaven keeps attribution in `THIRD_PARTY_NOTICES.md` and
`licenses/codex-Apache-2.0.txt`.

This is a baseline copy before RunHaven product integration. It intentionally
keeps Codex TUI structure first, then RunHaven will adapt the parts it needs.
The active Strategy C integration plan lives in
`docs/plans/codex-tui-strategy-c/`.

Run the vendor audit with:

```bash
scripts/compare-codex-tui.sh
```

That command fetches the pinned upstream Codex source from GitHub into a
temporary checkout and compares all files under `codex-rs/tui/src/`, including
Rust source, helper binaries, nested tests, and upstream snapshot files. It
builds deterministic file manifests for the upstream and RunHaven trees with
relative path, byte size, and SHA-256, then compares those manifests. Use
`scripts/compare-codex-tui.sh --write-manifests <dir>` when you need to keep
the generated manifests and comparison lists as audit artifacts.

Local exclusions in this baseline:

- `.DS_Store` files, because they are local filesystem metadata.
- upstream `*.snap` files, because they are Codex test goldens. They stay
  external in the pinned upstream Codex source by default, while RunHaven
  snapshots cover wired RunHaven behavior.

Current vendor audit summary:

- Upstream files under `codex-rs/tui/src/`: 894.
- RunHaven files under `crates/runhaven-tui/src/tui/`: 369.
- Common file paths: 356.
- Upstream files not vendored: 538, all `.snap` files.
- RunHaven-only files: 13.
- Copied Codex files with local edits: 26.

RunHaven-only files:

```text
README.md
app_shell.rs
mod.rs
pets/bundled_custom.rs
runhaven/app_server_client.rs
runhaven/app_server_session.rs
runhaven/launch_wizard.rs
runhaven/mod.rs
runhaven/protocol.rs
runhaven/service.rs
runhaven/terminal_handoff.rs
terminal_detection.rs
terminal_tests.rs
```

Copied Codex files with local edits:

```text
app/pets.rs
app.rs
bottom_pane/footer.rs
bottom_pane/list_selection_view.rs
bottom_pane/mod.rs
bottom_pane/textarea.rs
chatwidget/pets.rs
custom_terminal.rs
insert_history.rs
markdown_render_tests.rs
motion.rs
pets/image_protocol.rs
pets/mod.rs
pets/model.rs
pets/picker.rs
pets/preview.rs
render/renderable.rs
shimmer.rs
style.rs
terminal_hyperlinks.rs
terminal_palette.rs
terminal_probe.rs
test_backend.rs
tui.rs
tui/event_stream.rs
wrapping.rs
```

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
- `keymap.rs` is now compiled file-backed from the vendored Codex TUI source
  against the real `codex-config` crate, including
  `codex_config::types::{KeybindingsSpec, TuiKeymap, MAX_FUNCTION_KEY}`.
- `crates/codex/protocol`, `crates/codex/app-server-protocol`, and
  `crates/codex/config` are now real vendored package authorities.
  `bottom_pane/textarea.rs` consumes
  `codex_protocol::user_input::{ByteRange, TextElement}` from that vendored
  crate instead of a RunHaven-local staged protocol leaf, and `keymap.rs`
  consumes real Codex config types instead of a RunHaven-local self-alias.
- `crates/codex/connectors`, `crates/codex/file-search`,
  `crates/codex/plugin`, and `crates/codex/utils/approval-presets` are now
  real vendored package authorities for the next `app_event.rs` activation.
  The required plugin namespace closure also vendors `codex-utils-plugins`,
  `codex-exec-server`, `codex-exec-server-protocol`, `codex-sandboxing`,
  `codex-utils-pty`, and `codex-windows-sandbox`. These crates compile as
  source authorities only; RunHaven still does not route active TUI behavior
  through Codex exec-server, filesystem RPC, or sandbox launch paths.
- The local `codex-exec-server` manifest omits Codex's dev-only
  `codex-test-binary-support` dependency until RunHaven intentionally vendors
  and runs that exec-server test surface.
- The local `codex-plugin` manifest allows Clippy's `result_large_err` lint,
  matching the package-level pattern already used by other vendored Codex
  crates. This preserves upstream source shape under RunHaven's stricter
  workspace `-D warnings` gate.
- `lib.rs` no longer aliases `codex_config`; only
  `codex_terminal_detection` remains as a temporary self-alias until the
  terminal-detection crate is promoted into the vendored crate set.
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
  launch picker through `runhaven/launch_wizard.rs`.
- `runhaven/service.rs` is the temporary RunHaven TUI service seam. It turns
  `runhaven-core` profiles and planner output into launch preview payloads, so
  `app_shell.rs` does not call core planner APIs directly and
  `launch_wizard.rs` stays UI-owned.
- `runhaven/protocol.rs` and `runhaven/app_server_client.rs` are the local
  Strategy C backend facade. They mirror Codex's app-server client shape while
  keeping RunHaven runtime authority in `runhaven-core` and fail-closing
  unsupported Codex method families.
- `runhaven/app_server_session.rs` is the local Strategy C session bridge for
  this phase. It routes supported bootstrap, agent-catalog, and workspace
  validation calls into the RunHaven facade and returns typed unsupported errors
  for method families that are not promoted into the RunHaven security model.
- `runhaven/terminal_handoff.rs` is the local Phase 4 smoke hook. It proves
  Codex `Tui::with_restored` can release terminal ownership for a harmless
  foreground child and restore afterward without wiring real agent launch.
- `tui.rs` now compiles as `codex_runtime` under the temporary RunHaven module
  entrypoint. The local edits adapt nested module paths, the Ratatui backend
  error bound, deterministic terminal-handoff tests, and combined ambient plus
  picker-preview pet image cleanup for RunHaven's pinned dependency set.
- `tui/event_stream.rs` uses local `super::job_control` paths because
  `tui.rs` is nested as `codex_runtime` during integration, and keeps a
  deterministic pause/drop/resume regression for foreground handoff.
- `app.rs` uses the combined pet-image cleanup helper during native app
  shutdown so ambient and picker-preview image state are both cleared.
- `custom_terminal.rs`, `insert_history.rs`, `terminal_hyperlinks.rs`, and
  `test_backend.rs` keep Codex runtime behavior with Ratatui 0.30 compatibility
  edits for color conversion, backend error bounds, scrolling-region test
  support, and deprecated `Cell::skip` usage that remains part of the pinned
  upstream source shape.
- Two upstream `insert_history.rs` snapshot tests are opt-in behind
  `codex-vendored-tests` so default RunHaven tests do not create untracked
  `.snap.new` files from external Codex goldens.

Known integration gap:

- The copied Codex crate source still uses Codex crate/module assumptions.
  RunHaven integration will adapt entrypoints, module paths, dependencies, and
  product data in later commits.
- The real Codex protocol and config crates compile as workspace members, and
  `runhaven-tui` depends on them. Wider Codex crate activation is still
  incremental and must keep RunHaven runtime authority in `runhaven-core`.
- The real Codex connector, file-search, plugin, approval-preset, exec-server,
  sandboxing, PTY, and Windows sandbox crates compile as workspace members for
  the event-data dependency closure. `app_event.rs` itself is still dormant
  until the required shared TUI types are exposed without activating
  host-reaching Codex app paths.
- The dormant Codex `Tui` runtime spine now compiles and has focused tests, but
  it is not the active bare-interactive app loop yet.
- The launch picker, read-only review, and confirmation screen are staged in
  `app_shell.rs` plus `runhaven/service.rs`, not the real Codex `App` loop.
  Workspace selection, policy changes, and final foreground launch still need
  to be reattached through the Codex-shaped runtime.
- `tui/mod.rs` has a test guard for dormant host-reaching Codex surfaces. If
  `app`, `app_server_session`, onboarding auth, local ChatGPT auth, external
  editor, clipboard copy, or hooks RPC modules are activated, the test requires
  their risky upstream markers to be removed or fail-closed first.
- Foreground launch must be prepared by the RunHaven service but executed by the
  UI loop only after Codex terminal restore. Backend service tasks must not own
  raw terminal state.
