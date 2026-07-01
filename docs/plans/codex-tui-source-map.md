# Codex TUI Source Map

Pinned upstream source references for the RunHaven Codex TUI adaptation.

## Core Entrypoints

- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/main.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/lib.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/app.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/chatwidget.rs`

## Terminal Runtime

- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/tui.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/tui/event_stream.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/tui/frame_requester.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/tui/frame_rate_limiter.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/custom_terminal.rs`

## Composer And Bottom Pane

- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/public_widgets/composer_input.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/bottom_pane/mod.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/bottom_pane/chat_composer.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/bottom_pane/list_selection_view.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/bottom_pane/approval_overlay.rs`

## Rendering

- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/history_cell/mod.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/markdown_render.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/markdown.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/diff_render.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/render/highlight.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/terminal_hyperlinks.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/wrapping.rs`

## Streaming

- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/streaming/controller.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/streaming/chunking.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/streaming/table_holdback.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/markdown_stream.rs`

## App Server And Sessions

- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/app_server_session.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/app_server_session/fs.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/resume_picker.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/session_resume.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/session_archive_commands.rs`

## Extensions

- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/chatwidget/connectors.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/chatwidget/plugins.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/chatwidget/skills.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/chatwidget/mcp_startup.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/hooks_rpc.rs`

## Utilities

- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/clipboard_copy.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/clipboard_paste.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/external_editor.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/pets/mod.rs`

## Tests

- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/tests/all.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/tests/suite/mod.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/test_backend.rs`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/snapshots`
- `openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui/src/history_cell/snapshots`
