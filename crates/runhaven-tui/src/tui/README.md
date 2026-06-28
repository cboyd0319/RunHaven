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

It also includes Codex terminal detection copied from the same upstream commit
under `crates/codex/terminal-detection/`:

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
codex-context-fragments
codex-exec-server
codex-exec-server-protocol
codex-execpolicy
codex-experimental-api-macros
codex-features
codex-file-search
codex-file-system
codex-git-utils
codex-install-context
codex-memories-read
codex-model-provider-info
codex-network-proxy
codex-otel
codex-plugin
codex-protocol
codex-response-debug-context
codex-sandboxing
codex-shell-command
codex-terminal-detection
codex-utils-absolute-path
codex-utils-approval-presets
codex-utils-cache
codex-utils-cargo-bin
codex-utils-cli
codex-utils-elapsed
codex-utils-fuzzy-match
codex-utils-home-dir
codex-utils-image
codex-utils-output-truncation
codex-utils-path
codex-utils-path-uri
codex-utils-plugins
codex-utils-pty
codex-utils-rustls-provider
codex-utils-sleep-inhibitor
codex-utils-string
codex-utils-stream-parser
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
- RunHaven files under `crates/runhaven-tui/src/tui/`: 368.
- Common file paths: 356.
- Upstream files not vendored: 538, all `.snap` files.
- RunHaven-only files: 12.
- Copied Codex files with local edits: 42.

RunHaven-only files:

```text
README.md
app_event_shared.rs
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
```

Copied Codex files with local edits:

```text
app.rs
app/pets.rs
bottom_pane/app_link_view.rs
bottom_pane/approval_overlay.rs
bottom_pane/chat_composer.rs
bottom_pane/command_popup.rs
bottom_pane/feedback_view.rs
bottom_pane/footer.rs
bottom_pane/hooks_browser_view.rs
bottom_pane/list_selection_view.rs
bottom_pane/mcp_server_elicitation.rs
bottom_pane/mod.rs
bottom_pane/pending_input_preview.rs
bottom_pane/request_user_input/mod.rs
bottom_pane/skill_popup.rs
bottom_pane/skills_toggle_view.rs
bottom_pane/status_line_setup.rs
bottom_pane/textarea.rs
bottom_pane/title_setup.rs
bottom_pane/unified_exec_footer.rs
chatwidget/pets.rs
custom_terminal.rs
insert_history.rs
markdown_render_tests.rs
motion.rs
pets/mod.rs
pets/model.rs
pets/picker.rs
pets/preview.rs
render/highlight.rs
render/renderable.rs
shimmer.rs
status_indicator_widget.rs
style.rs
terminal_hyperlinks.rs
terminal_palette.rs
terminal_probe.rs
test_backend.rs
test_support.rs
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
- `pets/picker.rs` and `pets/preview.rs` remain vendored and now compile
  against the real vendored `bottom_pane` module path. They are still not
  exposed as active product flows until the native app shell owns those views.
- `pets/model.rs` formats the SHA-256 cache key bytes explicitly because
  RunHaven is pinned to `sha2` 0.11. The produced cache key string stays the
  same shape as Codex.
- `app_event.rs` and `app_event_sender.rs` are now compiled as the real
  vendored Codex files. `app_event_shared.rs` is a temporary inert type bridge
  for shared leaves whose owning modules remain dormant. Remove that bridge as
  the real shared modules are promoted.
- `bottom_pane/mod.rs` is now the real vendored Codex source. RunHaven keeps
  small re-exports for the temporary shell and gates snapshot-heavy upstream
  tests behind `codex-vendored-tests` until RunHaven intentionally tracks
  those goldens.
- `keymap.rs` is now compiled file-backed from the vendored Codex TUI source
  against the real `codex-config` crate, including
  `codex_config::types::{KeybindingsSpec, TuiKeymap, MAX_FUNCTION_KEY}`.
- `crates/codex/protocol`, `crates/codex/app-server-protocol`, and
  `crates/codex/config` are now real vendored package authorities.
  `bottom_pane/textarea.rs` consumes
  `codex_protocol::user_input::{ByteRange, TextElement}` from that vendored
  crate instead of a RunHaven-local staged protocol leaf, and `keymap.rs`
  consumes real Codex config types instead of a RunHaven-local self-alias.
- `crates/codex/connectors`, `crates/codex/features`,
  `crates/codex/file-search`, `crates/codex/plugin`,
  `crates/codex/utils/absolute-path`, and
  `crates/codex/utils/approval-presets` are now real vendored package
  authorities for the active vendored `app_event.rs` and
  `app_event_sender.rs`.
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
- `crates/codex/core-skills`, `crates/codex/feedback`,
  `crates/codex/models-manager`, and
  `crates/codex/utils/fuzzy-match` are now original-name crate authorities for
  the real bottom-pane source. The first three expose only the inert model or
  diagnostic surfaces needed by the active TUI compile path; upstream
  host-skill loading, feedback upload/logging, remote model cache, login, and
  telemetry behavior remain dormant. The feedback diagnostics env collector is
  shape-compatible but returns no diagnostics until RunHaven has a redaction
  policy for host environment capture.
- `crates/codex/utils/cli`, `crates/codex/utils/elapsed`, and
  `crates/codex/utils/sleep-inhibitor` are now original-name crate authorities
  for dormant Codex TUI CLI, history, exec-cell, and chat turn-lifecycle
  imports. The sleep inhibitor keeps its native FFI unsafe allowance scoped to
  that vendored utility crate; it is not active RunHaven backend authority.
- `crates/codex/terminal-detection` is now the original-name crate authority
  for terminal identification. `runhaven-tui` no longer aliases itself as
  `codex_terminal_detection`, and the duplicate local
  `terminal_detection.rs` plus `terminal_tests.rs` files were deleted.
- `crates/codex/context-fragments`, `crates/codex/install-context`,
  `crates/codex/memories/read`, `crates/codex/response-debug-context`,
  `crates/codex/utils/output-truncation`, and
  `crates/codex/utils/stream-parser` are original-name support crate
  authorities for the next `legacy_core::config` compatibility closure. They
  compile as source authorities only; `runhaven-tui` does not call their
  install, memory, or response-debug helpers as active product behavior in this
  slice.
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
- The real Codex event, sender, and bottom-pane files compile. The temporary
  `app_event_shared.rs` leaf-type bridge plus the inline `status` and
  `onboarding` shims must be removed as real `chatwidget`, `history_cell`,
  `goal_files`, `session_log`, status, onboarding, and app-server-session
  surfaces are promoted without activating host-reaching Codex app paths.
- Direct `chatwidget` activation is blocked on replacing the temporary
  `legacy_core::config` gap with a vendor-first compatibility path. The real
  `history_cell`, `status`, and `chatwidget` modules all depend on Codex's
  core config shape, so promoting only `chatwidget.rs` produces root-module and
  config-surface errors instead of a useful intermediate state.
- Full upstream `codex-app-server-client` and full upstream `codex-core` are
  not active yet. `codex-app-server-client` brings the app-server transport
  closure, and full `codex-core` brings login, MCP, filesystem, hooks, tools,
  rollout, model-provider, and app-server-adjacent behavior. The next useful
  step is the reduced original-name `codex-core` config compatibility closure,
  with host-reaching behavior kept inert or fail-closed.
- The dormant Codex `Tui` runtime spine now compiles and has focused tests, but
  it is not the active bare-interactive app loop yet.
- The launch picker, read-only review, and confirmation screen still run from
  `app_shell.rs` plus `runhaven/service.rs`, not the real Codex `App` loop.
  Workspace selection, policy changes, and final foreground launch still need
  to be reattached through the Codex-shaped runtime and native app ownership.
- `tui/mod.rs` has a test guard for dormant host-reaching Codex surfaces. If
  `app`, `app_server_session`, onboarding auth, local ChatGPT auth, external
  editor, clipboard copy, or hooks RPC modules are activated, the test requires
  their risky upstream markers to be removed or fail-closed first.
- Foreground launch must be prepared by the RunHaven service but executed by the
  UI loop only after Codex terminal restore. Backend service tasks must not own
  raw terminal state.
