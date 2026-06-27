# 01 Source Inventory

## Current RunHaven State

Observed current state:

- RunHaven already contains a broad Codex TUI source snapshot under
  `crates/runhaven-tui/src/tui/`.
- The pinned Codex source has 894 total files under `codex-rs/tui/src`: 355
  Rust files, one non-snapshot instruction file, and 538 upstream `.snap`
  goldens. RunHaven shares the 356 non-snapshot upstream paths and adds seven
  local Rust files:
  - `app_shell.rs`
  - `mod.rs`
  - `pets/bundled_custom.rs`
  - `runhaven/mod.rs`
  - `runhaven/launch_wizard.rs`
  - `terminal_detection.rs`
  - `terminal_tests.rs`
- RunHaven currently does not copy upstream `.snap` files. With the
  vendor-first assumption, keep upstream snapshots and fixtures available as
  reference material, but regenerate authoritative RunHaven snapshots from
  RunHaven-integrated tests.
- RunHaven also added `crates/runhaven-tui/src/tui/README.md`, which records
  the vendor baseline and local integration exceptions.
- The active TUI entrypoint is still temporary:
  - `crates/runhaven-tui/src/lib.rs` exposes `tui::run`.
  - `crates/runhaven-tui/src/tui/mod.rs` defines a staged module facade.
  - `crates/runhaven-tui/src/tui/app_shell.rs` hosts a read-only launch picker,
    review step, confirmation step, footer, terminal title, and opt-in pet image
    smoke path.
  - `crates/runhaven-tui/src/tui/runhaven/launch_wizard.rs` maps
    `AgentCatalogData` and `LaunchPlanData` into Codex `ListSelectionView`.
- The currently wired Codex primitives are only a subset:
  - `bottom_pane/list_selection_view.rs`
  - `bottom_pane/textarea.rs`
  - `bottom_pane/footer.rs`
  - `terminal_title.rs`
  - `pets/*` for the opt-in image smoke path
  - rendering/style/wrapping helpers needed by those pieces
- Full Codex `App`, `ChatWidget`, `BottomPane`, `app_server_session.rs`,
  terminal `Tui`, and async event stream are copied but not the live app shell.

Important implication:

The source copy is mostly done. The remaining work is wiring, backend shape,
and disciplined culling, not bulk file copying.

## What To Bring Over

### Already Brought Over And Should Stay Source-Close

Keep these source families as close to Codex upstream as practical:

| Codex source | RunHaven target | Why it stays |
| --- | --- | --- |
| `tui.rs` and `tui/*` | `crates/runhaven-tui/src/tui/tui.rs`, `tui/tui/*` | Terminal lifecycle, raw mode, bracketed paste, focus events, redraw scheduling, job control, event stream pause/resume. This is hard to get right and should stay Codex-shaped. |
| `custom_terminal.rs` | same path | Codex terminal wrapper and viewport behavior. |
| `insert_history.rs`, `transcript_reflow.rs`, `resize_reflow_cap.rs` | same paths | Codex scrollback/transcript handling. |
| `app.rs` and `app/*` | same paths | Target top-level event loop shape. Adapt, do not replace with a custom loop. |
| `app_event.rs`, `app_event_sender.rs`, `app_command.rs` | same paths | Internal event bus. Rename product-specific variants only when they become active RunHaven behavior. |
| `app_server_session.rs`, `app_server_session/fs.rs` | same paths | Typed backend facade. This is the key Strategy C boundary. |
| `chatwidget.rs` and `chatwidget/*` | same paths | Transcript, active cell, footer state, command routing, status surfaces, streaming integration. Keep the shape, adapt data. |
| `bottom_pane/*` | same paths | Composer, list views, approval overlays, typed input, popups, footer, status line. |
| `public_widgets/composer_input.rs` | same path | Keep as standalone wrapper and testable editor primitive. |
| `history_cell/*` | same paths | Transcript cells for messages, commands, patches, notices, approvals, plans, hooks. Map RunHaven events into these where possible. |
| `streaming/*`, `markdown_stream.rs` | same paths | Stable/tail rendering and markdown streaming behavior. |
| `markdown.rs`, `markdown_render.rs`, `markdown_render/*` | same paths | Markdown renderer for help, diagnostics, logs, and transcript content. |
| `diff_render.rs`, `diff_model.rs` | same paths | Run diff and review rendering. Back it with RunHaven `run_diff_text`, not Codex git helpers. |
| `render/*`, `wrapping.rs`, `width.rs`, `line_truncation.rs`, `text_formatting.rs` | same paths | General Ratatui layout, unicode width, wrapping, truncation. |
| `terminal_hyperlinks.rs` | same path | OSC 8 links for paths, docs, and commands. |
| `clipboard_copy.rs`, `clipboard_paste.rs`, `external_editor.rs` | same paths | User input/output affordances. Activate only with the same terminal restore discipline as Codex. |
| `key_hint.rs`, `keymap.rs`, `keymap_setup/*` | same paths | Shortcut display and keymap structure. Adapt command vocabulary later. |
| `status/*`, `status_indicator_widget.rs`, `goal_display.rs`, `goal_files.rs`, `token_usage.rs` | same paths | Useful status-card structure. Data should become RunHaven run/status data. |
| `terminal_title.rs` | same path | OSC title sanitization and cleanup. Already active. |
| `terminal_palette.rs`, `terminal_probe.rs`, `color.rs`, `style.rs`, `theme_picker.rs`, `motion.rs`, `shimmer.rs` | same paths | Theme, color, terminal probing, animation primitives. |
| `pets/*` | same path, plus `pets/bundled_custom.rs` | Keep Codex pet rendering and terminal image protocol. RunHaven should only swap the selected pet package and copy. |
| `notifications/*` | same paths | Use later for run completion or waiting-for-input notices after safety review. |
| `test_backend.rs`, `test_support.rs`, `app/test_support.rs`, `tui/test_support.rs`, TUI test helpers | same paths | VT100, replay, and snapshot testing. |
| `bin/md-events.rs` | same path or vendor reference path | Useful markdown/event rendering fixture driver. Not a RunHaven product binary by default. |
| `bottom_pane/AGENTS.md` | same path | Local upstream instructions for the most edited TUI subtree. |

### Source-Adjacent Test And Fixture Assets

Do not treat tests and helper binaries as disposable just because they do not
ship in the product binary. Snapshot goldens are different: RunHaven already
removed copied upstream `.snap` files from the active tree and gates upstream
snapshot-only tests behind `codex-vendored-tests`.

Keep upstream snapshot goldens external by default in the pinned upstream Codex
source.
Only add local RunHaven snapshots for RunHaven-integrated behavior that is
actually wired and tested.

Preserve or compare against these upstream references:

- `*_tests.rs` files and sibling `tests/` modules
- `bin/md-events.rs`

Recommended layout:

```text
crates/runhaven-tui/src/tui/                 active vendored source
crates/runhaven-tui/src/tui/snapshots/       RunHaven authoritative snapshots
openai/codex:codex-rs/tui/src/**/snapshots/ upstream reference snapshots
```

Do not create `crates/runhaven-tui/upstream-snapshots/` unless a future slice
proves that keeping an in-repo reference copy is worth the noise. RunHaven
product snapshots should be generated from RunHaven behavior, not accepted
wholesale from Codex.

### Local RunHaven Files That Should Stay Local

These files are intentionally not upstream Codex files:

| RunHaven file | Role | Long-term state |
| --- | --- | --- |
| `tui/README.md` | Vendor ledger and local exceptions. | Keep and update every time source-copy rules change. |
| `tui/mod.rs` | Temporary staged facade for compiling selected Codex modules. | Shrink and eventually replace with a Codex-shaped module tree. |
| `tui/app_shell.rs` | Temporary read-only launch preview shell. | Delete or reduce once Codex `App` is adapted. |
| `tui/runhaven/mod.rs` | RunHaven TUI adapter namespace. | Keep. |
| `tui/runhaven/launch_wizard.rs` | RunHaven-owned mapping from planner payloads to Codex picker/review UI. | Keep, but make it a normal view launched by Codex `App`, not the app itself. |
| `tui/pets/bundled_custom.rs` | Materializes the bundled Cubby pet package into Codex custom-pet shape. | Keep. |
| `tui/terminal_detection.rs`, `tui/terminal_tests.rs` | Copied from Codex `terminal-detection` crate. | Prefer moving to a local `codex-terminal-detection` vendor crate when the workspace can absorb it cleanly. |

### Source Not Yet Useful As Active RunHaven Behavior

Keep these copied files dormant or fail-closed until there is a RunHaven design:

- `chatwidget/connectors.rs`
- `chatwidget/plugins.rs`
- `chatwidget/skills.rs`
- `chatwidget/mcp_startup.rs`
- `hooks_rpc.rs`
- `bottom_pane/hooks_browser_view.rs`
- `bottom_pane/mcp_server_elicitation.rs`
- `bottom_pane/app_link_view.rs`
- `ide_context/*`
- `external_agent_config_migration*`
- `local_chatgpt_auth.rs`
- `onboarding/auth/*`
- `workspace_messages.rs`
- `updates*`
- `feedback_view.rs`
- `startup_hooks_review.rs`
- `model_migration.rs`
- `theme_picker.rs` until RunHaven has theme settings
- `windows_sandbox.rs`
- remote filesystem behavior in `app_server_session/fs.rs`

Do not rip these out only because they are not active yet. First wire the
Codex-shaped app shell. Then cull with a recorded reason and tests. Deleting too
early makes upstream comparison harder.

### Source To Vendor But Not Activate As Runtime Backend

Vendor Codex source where practical, including backend-adjacent crates when
that lowers TUI drift or preserves original import paths. Do not make Codex's
full backend stack RunHaven's active backend by default:

- `codex-core`
- `codex-app-server`
- `codex-exec-server`
- `codex-login`
- `codex-config`
- `codex-cloud-config`
- `codex-connectors`
- `codex-core-plugins`
- `codex-core-skills`
- `codex-state`
- `codex-rollout`
- `codex-model-provider*`

Those crates implement the Codex product. RunHaven already has a product
backend in `runhaven-core`. They may be vendored as inert source, compatibility
crates, or disabled feature islands, but the TUI must route authoritative
RunHaven actions through `runhaven-core`.

The rule is vendor-first, activate-later. Prefer preserving Codex crate names
and module paths. Cut or stub only at product-boundary points where direct
activation would bypass RunHaven's container, auth, workspace, or egress model.

## Concrete Source Checklist

Use this as the implementation checklist.

### Keep Copied

- `src/tui.rs`
- `src/tui/*`
- `src/custom_terminal.rs`
- `src/app.rs`
- `src/app/*`
- `src/app_event.rs`
- `src/app_event_sender.rs`
- `src/app_command.rs`
- `src/app_server_session.rs`
- `src/app_server_session/fs.rs`
- `src/chatwidget.rs`
- `src/chatwidget/*`
- `src/bottom_pane/*`
- `src/public_widgets/*`
- `src/history_cell/*`
- `src/exec_cell/*`
- `src/streaming/*`
- `src/markdown.rs`
- `src/markdown_render.rs`
- `src/markdown_render/*`
- `src/markdown_stream.rs`
- `src/diff_model.rs`
- `src/diff_render.rs`
- `src/render/*`
- `src/wrapping.rs`
- `src/width.rs`
- `src/line_truncation.rs`
- `src/text_formatting.rs`
- `src/terminal_hyperlinks.rs`
- `src/terminal_title.rs`
- `src/terminal_palette.rs`
- `src/terminal_probe.rs`
- `src/color.rs`
- `src/style.rs`
- `src/motion.rs`
- `src/shimmer.rs`
- `src/key_hint.rs`
- `src/keymap.rs`
- `src/keymap_setup/*`
- `src/selection_list.rs`
- `src/slash_command.rs`
- `src/tooltips.rs`
- `src/terminal_visualization_instructions.rs`
- `src/config_update.rs`
- `src/cwd_prompt.rs`
- `src/debug_config.rs`
- `src/frames.rs`
- `src/service_tier_resolution.rs`
- `src/status/*`
- `src/status_indicator_widget.rs`
- `src/pager_overlay.rs`
- `src/pets/*`
- `src/notifications/*`
- `src/bin/md-events.rs`
- `src/test_backend.rs`
- `src/test_support.rs`
- `src/app/test_support.rs`
- `src/tui/test_support.rs`
- upstream `src/**/snapshots/*.snap` as external reference material in the
  pinned upstream Codex source, not copied by default

### Keep But Adapt Heavily

- `src/lib.rs`: replace Codex config/auth/app-server bootstrap with RunHaven
  bootstrap while preserving the run-main shape.
- `src/main.rs`: RunHaven probably does not need the `codex-tui` binary, but
  the source is useful as upstream reference.
- `src/cli.rs`: adapt only if RunHaven exposes TUI-specific flags.
- `src/app_server_session.rs`: preserve method facade, replace backend client
  and request types as needed.
- `src/app.rs`: preserve event loop, replace bootstrap/status/model/account
  assumptions with RunHaven state.
- `src/chatwidget.rs`: preserve transcript/composer/status architecture,
  disable or remap Codex model/account/plugin/MCP features.
- `src/status/*`: replace Codex account/model/rate-limit fields with RunHaven
  runtime status.
- `src/resume_picker.rs`, `src/session_resume.rs`, `src/session_state.rs`,
  `src/session_archive_commands.rs`: map to RunHaven run history and active
  run resume/attach concepts.
- `src/onboarding/*`: replace with RunHaven setup, image readiness, login, and
  security boundary guidance.
- `src/startup_hooks_review.rs`: keep shape only if RunHaven adds startup
  checks that can block or warn before launch.
- `src/model_migration.rs`, `src/model_catalog.rs`: adapt only if RunHaven has
  model/provider migration semantics. Otherwise keep as dormant compatibility
  source.

### Keep Dormant Or Fail-Closed First

- `src/hooks_rpc.rs`
- `src/file_search.rs`
- `src/ide_context/*`
- `src/local_chatgpt_auth.rs`
- `src/external_agent_config_migration*`
- `src/model_catalog.rs`
- `src/model_migration.rs`
- `src/multi_agents.rs`
- `src/workspace_messages.rs`
- `src/updates*`
- `src/feedback*`
- `src/oss_selection.rs`
- Codex plugin, app, connector, skill, MCP, and marketplace modules.

### RunHaven-Owned Active Files

- `crates/runhaven-core/src/ui_contracts.rs`
- `crates/runhaven-tui/src/tui/runhaven/mod.rs`
- `crates/runhaven-tui/src/tui/runhaven/launch_wizard.rs`
- future `crates/runhaven-tui/src/tui/runhaven/app_server_client.rs`
- future `crates/runhaven-tui/src/tui/runhaven/protocol.rs`
- future `crates/runhaven-tui/src/tui/runhaven/service.rs`
- future `crates/runhaven-tui/src/tui/runhaven/mapper.rs`
- `crates/runhaven-tui/src/tui/pets/bundled_custom.rs`
