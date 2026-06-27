# Codex TUI Capabilities And Reuse Guide

Locked into the RunHaven repo on 2026-06-27 from the local reference:

```text
/Users/c/Downloads/codex-tui-capabilities.md
```

Use this as the source map for deciding which Codex TUI capabilities to vendor,
adapt, keep staged, or cull. It is intentionally broader than RunHaven's current
implementation so future TUI work starts from evidence instead of rebuilding
custom behavior.

RunHaven decision: treat Strategy C, a Codex-compatible client, as the target
architecture because RunHaven is itself an agent/session/turn product. Strategy
B, a small TUI kit extraction, is fallback-only for temporary compile bridges or
isolated low-coupling helpers. Do not let Strategy B become the product
architecture unless this plan is explicitly changed. Host-reaching Codex RPCs
such as remote filesystem, MCP, IDE, plugin, connector, and broad app-server
actions stay fail-closed unless RunHaven's security model explicitly promotes
them.

## Scope

This document summarizes the open source `codex-tui` crate under:

`/Users/c/Documents/GitHub/codex/codex-rs/tui`

It is written for someone evaluating whether to reuse the TUI in another project. The short version: this is not just a widget crate. It is a full terminal client for an agent product, with a mature terminal runtime, chat composer, streaming transcript renderer, approval UX, session management, extension surfaces, and app-server integration.

The strongest reusable pieces are the composer, terminal lifecycle, markdown and diff renderers, streaming controller design, keymap system, selection popup patterns, approval modal patterns, clipboard/editor utilities, and vt100/snapshot testing approach. The main adoption risk is coupling to Codex-specific protocol, app-server, auth, model, config, plugin, connector, and state crates.

## License And Project Identity

The repository root describes Codex CLI as a coding agent from OpenAI that runs locally. The root README also states the repository is licensed under Apache-2.0. The Rust workspace package license is `Apache-2.0`, and `codex-tui` inherits `license.workspace = true`.

References:

- `/Users/c/Documents/GitHub/codex/README.md:1`
- `/Users/c/Documents/GitHub/codex/README.md:71`
- `/Users/c/Documents/GitHub/codex/LICENSE:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/Cargo.toml:129`
- `/Users/c/Documents/GitHub/codex/codex-rs/Cargo.toml:136`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:2`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:5`

## Crate Shape

`codex-tui` builds both a binary and a library:

- Binary: `codex-tui`, entrypoint `src/main.rs`.
- Secondary binary: `md-events`, entrypoint `src/bin/md-events.rs`.
- Library: `codex_tui`, entrypoint `src/lib.rs`.

The manifest shows the crate is built on `crossterm`, `ratatui`, `tokio`, `pulldown-cmark`, `syntect`, `two-face`, `image`, `arboard`, and a large set of Codex workspace crates for app-server, protocol, config, auth, plugins, skills, connectors, state, rollout, sandboxing, file search, and model management.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:8`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:18`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:24`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:71`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:82`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:116`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/Cargo.toml:147`

## High-Level Architecture

The TUI is organized around these layers:

1. `main.rs`: parses top-level CLI flags, invokes `run_main`, and prints final token usage or resume hints after the TUI exits.
2. `lib.rs`: config loading, auth/bootstrap, embedded or remote app-server startup, onboarding, resume/fork startup, terminal initialization, and main app launch.
3. `app.rs`: top-level app event loop and orchestration, coordinating app-server events, widgets, background requests, config persistence, session lifecycle, and platform actions.
4. `chatwidget.rs`: main chat surface, protocol event handling, active/committed history cells, overlays, status headers, streaming state, queued input, slash command dispatch, settings, approvals, plugins, skills, MCP, and usage/status output.
5. `bottom_pane/*`: composer, popups, selection lists, approval overlay, prompt overlays, file/skill/app mentions, status-line setup, keymap capture, and other interactive footer surfaces.
6. `history_cell/*`, `markdown_render.rs`, `diff_render.rs`, `streaming/*`: display and transcript rendering.
7. `app_server_session.rs`: typed JSON-RPC facade over embedded or remote app-server.
8. `tui/*`: terminal lifecycle, event streams, frame scheduling, raw mode, focus, paste, job-control, and stderr handling.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/main.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/lib.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/lib.rs:849`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mod.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs:1`

## Terminal Runtime Capabilities

The terminal layer handles many edge cases that are easy to miss in custom TUIs:

- Raw mode setup and restore.
- Bracketed paste enable/disable.
- Focus change events.
- Keyboard enhancement flags for distinguishing modified keys.
- Alternate scroll behavior.
- Panic hook restoration so the shell is not left broken.
- Inline viewport mode and alternate screen mode.
- Cursor position and default color probing.
- Windows virtual terminal handling and stdin flush.
- Unix job-control support for suspend/resume.
- A shared event broker that can pause and recreate the crossterm event stream, which avoids stealing stdin from spawned editors or other subprocesses.
- Frame requester and frame-rate limiting.
- Desktop notification support through OSC 9 or BEL.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs:175`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs:282`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs:302`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs:376`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs:513`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs:633`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs:690`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/event_stream.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/event_stream.rs:51`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/event_stream.rs:139`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/notifications/mod.rs:14`

## CLI And Launch Capabilities

The TUI supports:

- Optional startup prompt.
- Strict config mode.
- Approval policy override.
- Live web search toggle through `--search`.
- Inline mode via `--no-alt-screen`.
- Internal resume and fork launch modes.
- Remote app-server endpoints via WebSocket or Unix socket.
- Embedded app-server startup.
- Implicit local daemon reuse when safe.
- Config override handling.
- Login/onboarding and trust-directory screens.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/cli.rs:10`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/cli.rs:13`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/cli.rs:17`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/cli.rs:62`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/cli.rs:66`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/cli.rs:72`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/lib.rs:347`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/lib.rs:849`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/lib.rs:868`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/lib.rs:1723`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/lib.rs:1865`

## App-Server Integration

The app-server facade is a major part of the TUI. It owns typed client calls and keeps transport plumbing out of `App` and `ChatWidget`.

It can:

- Bootstrap account and model metadata.
- Start, resume, fork, read, list, archive, delete, unarchive, unsubscribe, compact, and roll back threads.
- Start, steer, and interrupt turns.
- Update thread settings.
- Rename threads.
- Set thread memory mode.
- Reset memories.
- Get, set, update, pause, resume, and clear thread goals.
- Start inline reviews.
- List skills.
- Trigger shell commands.
- Approve guardian-denied actions.
- Clean background terminals.
- Reload user config.
- Perform remote or local filesystem RPCs for create directory, read, write, and remove.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:229`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:252`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:421`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:493`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:519`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:749`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:813`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:827`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:916`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:1074`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs:1093`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session/fs.rs:1`

## Main Chat Surface

`ChatWidget` is the primary chat UI. It consumes protocol events, maintains committed transcript cells plus an in-flight active cell, drives rendering, manages overlays, routes slash commands, tracks task-running state, and coordinates active streaming output.

Important behaviors include:

- Active-cell/live-tail transcript overlay.
- Agent turn running state independent from MCP startup state.
- Status header updates.
- Queued input while work is running.
- Interrupt and rollback handling.
- Settings popups.
- Model, reasoning, personality, service tier, permission, and collaboration-mode updates.
- Side conversations and multi-agent thread switching.
- Plugin, app, skill, hook, MCP, memory, feedback, usage, status, and review surfaces.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget.rs:203`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_event.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_event.rs:139`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_event.rs:250`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_event.rs:520`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_event.rs:753`

## Composer And Bottom Pane

The bottom pane is a reusable UI pattern: it owns a persistent composer plus a stack of transient views that replace the composer for focused interactions.

The composer can:

- Edit multiline input.
- Preserve placeholder elements for attachments.
- Route keys to slash, file, skill, app, and mention popups.
- Promote slash commands into atomic elements.
- Submit with Enter.
- Insert newlines with Shift+Enter or configured keys.
- Queue input with Tab while a task is running.
- Preserve in-session and persistent history.
- Open reverse history search with `Ctrl+R`.
- Rehydrate local image paths and remote image URLs from history.
- Represent large pastes with placeholders.
- Detect non-bracketed paste bursts, especially on Windows.
- Support remote image rows that can be selected and deleted.
- Temporarily disable input while retaining UI state.

The easiest standalone component is `public_widgets::ComposerInput`, which wraps the internal `ChatComposer` and exposes a smaller API for other crates.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mod.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mod.rs:79`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mod.rs:85`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mod.rs:208`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/chat_composer.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/chat_composer.rs:12`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/chat_composer.rs:19`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/chat_composer.rs:62`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/chat_composer.rs:73`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/chat_composer.rs:92`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/chat_composer.rs:127`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/public_widgets/composer_input.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/public_widgets/composer_input.rs:28`

## Slash Commands

Slash commands expose most product capabilities from the TUI. The enum order is also the presentation order in the popup.

Available commands include:

- Model and reasoning: `/model`.
- IDE context: `/ide`.
- Permissions: `/permissions`, `/approve`, `/setup-default-sandbox`, `/sandbox-add-read-dir`.
- Keymaps and editing: `/keymap`, `/vim`.
- Experimental features: `/experimental`.
- Memory: `/memories`.
- Skills and setup import: `/skills`, `/import`.
- Hooks: `/hooks`.
- Review: `/review`.
- Session lifecycle: `/rename`, `/new`, `/archive`, `/delete`, `/resume`, `/fork`, `/compact`, `/clear`.
- App handoff: `/app`.
- Project setup: `/init`.
- Collaboration: `/plan`, `/goal`, `/agent`, `/subagents`, `/side`, `/btw`.
- Output utility: `/copy`, `/raw`, `/diff`, `/mention`.
- Status and usage: `/status`, `/usage`, `/debug-config`.
- UI customization: `/title`, `/statusline`, `/theme`, `/pets`.
- Integration surfaces: `/mcp`, `/apps`, `/plugins`.
- Auth and feedback: `/logout`, `/feedback`.
- Background terminals: `/ps`, `/stop`.
- Exit: `/quit`, `/exit`.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/slash_command.rs:12`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/slash_command.rs:65`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/slash_command.rs:133`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/slash_command.rs:162`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/slash_command.rs:258`

## Selection Views And Popups

The bottom pane uses a generic selection-list model for many UI surfaces:

- Searchable list popups.
- Column-width modes.
- Side content panels.
- Toggleable rows.
- Multi-select pickers.
- Selection tabs.
- Custom prompt views.
- Approval overlays.
- Request-user-input overlays.
- MCP elicitation forms.
- Feedback flows.
- Hooks browser.
- Status-line and terminal-title setup.
- Skill enable/disable picker.

This is a good source of patterns for building dense, keyboard-driven operational TUIs.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mod.rs:35`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mod.rs:111`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mod.rs:122`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/list_selection_view.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/multi_select_picker.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mcp_server_elicitation.rs`

## Approval And Permission UX

The approval overlay converts high-risk operations into explicit user decisions. It supports:

- Command execution approvals.
- Permission profile approvals.
- Apply-patch/file-change approvals.
- MCP elicitation approval/cancel behavior.
- Queued approval requests.
- Custom approval keybindings.
- Fullscreen details for large payloads.

This is a useful model for any app that needs human-in-the-loop approval of risky actions.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/approval_overlay.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/approval_overlay.rs:72`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/approval_overlay.rs:159`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_approval_conversions.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/permission_compat.rs:1`

## Transcript And History Cells

Conversation output is represented as `HistoryCell` objects. A cell can render itself as:

- Rich display lines for the main viewport.
- Raw copy-friendly lines.
- Hyperlink-aware display lines.
- Transcript overlay lines.
- Height measurements for wrapped viewport rendering.

Cell modules cover approvals, exec output, hook cells, MCP, messages, notices, patches, plans, request-user-input, search, separators, and session info.

This design is valuable if your app has multiple output types and needs consistent scrollback, transcript, and copy behavior.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/history_cell/mod.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/history_cell/mod.rs:111`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/history_cell/mod.rs:145`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/history_cell/mod.rs:189`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/thread_transcript.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/thread_transcript.rs:23`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/thread_transcript.rs:35`

## Markdown Rendering

The markdown renderer is built on `pulldown-cmark` and emits styled ratatui lines. Notable features:

- Styled headings, code, emphasis, strong text, strikethrough, links, lists, and blockquotes.
- Local file links displayed using the real destination path rather than only the markdown label.
- Width-aware wrapping.
- URL-aware wrapping through helper functions.
- Hyperlink metadata carried separately from visible text so OSC 8 sequences do not affect layout.
- Table rendering that normalizes rows, computes column widths, and falls back to key/value records when narrow.
- Heuristics to avoid broken table rendering from lenient markdown parsing.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown_render.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown_render.rs:13`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown_render.rs:298`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/terminal_hyperlinks.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/terminal_hyperlinks.rs:33`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/terminal_hyperlinks.rs:147`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/wrapping.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/wrapping.rs:508`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/wrapping.rs:528`

## Syntax Highlighting And Themes

The syntax highlighter wraps syntect with two-face grammars and themes. It provides:

- Roughly 250-language syntax highlighting through bundled grammars.
- Bundled themes.
- Custom `.tmTheme` discovery under `$CODEX_HOME/themes`.
- Runtime theme swapping for previews.
- Adaptive default theme selection from terminal background lightness.
- Guardrails that skip highlighting for very large inputs.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/render/highlight.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/render/highlight.rs:81`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/render/highlight.rs:136`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/render/highlight.rs:199`

## Diff Rendering

The diff renderer is one of the more polished standalone pieces. It renders unified diffs with:

- File-change blocks for add, delete, and update.
- Line numbers and gutter signs.
- Syntax-highlighted content.
- Wrapped long lines with style preservation.
- Terminal background-aware palettes.
- Rich color, ANSI 256, and ANSI 16 handling.
- Theme-provided inserted/deleted scope background overrides when available.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/diff_render.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/diff_render.rs:86`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/diff_render.rs:189`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/diff_render.rs:199`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/diff_render.rs:215`

## Streaming Output

The streaming controller is designed for incremental agent output:

- It buffers raw markdown deltas.
- It commits only newline-complete source.
- It partitions output into a stable region and a mutable tail.
- It holds back markdown tables because adding rows can reshape earlier rows.
- It re-renders from source on resize.
- It consolidates finalized output into source-backed transcript cells.
- It uses adaptive chunking to drain one line at a time during normal output but catch up when queue depth or age grows.

This design is worth studying if your project streams model output, logs, test output, or long-running command output into a terminal UI.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/controller.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/controller.rs:12`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/controller.rs:22`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/controller.rs:73`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/controller.rs:462`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/controller.rs:574`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/chunking.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/chunking.rs:119`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/chunking.rs:137`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/chunking.rs:157`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown_stream.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown_stream.rs:30`

## Keymap System

The keymap system resolves config into a runtime snapshot. It handles:

- Context-specific bindings.
- Global fallback.
- Built-in defaults.
- Explicit unbinding.
- Duplicate-key validation.
- User-facing config errors.
- App, chat, composer, editor, Vim normal, Vim operator, Vim text object, pager, list, and approval contexts.
- Guided remapping UI through `/keymap`.
- Key capture and debug inspection.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap.rs:44`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap.rs:58`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap.rs:146`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap.rs:255`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap_setup.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap_setup.rs:70`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap_setup.rs:74`

## File Search And Mentions

The TUI supports `@` file search and a unified mention popup. File search is session-based and updates as the user types. Mention bindings can represent files, skills, apps, plugins, or tool targets depending on enabled features.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/file_search.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/file_search.rs:16`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/file_search.rs:29`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mentions_v2/mod.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/mention_codec.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/plugin_mentions.rs`

## Apps, Skills, Plugins, MCP, And Hooks

The TUI has real extension surfaces, but they are mostly backed by Codex app-server APIs and Codex workspace crates.

Apps/connectors:

- Prefetch connector state.
- Show installed and available apps.
- Open app install/manage links.
- Insert installed app mentions into prompts.

Skills:

- List skills.
- Enable or disable skills.
- Convert protocol skill metadata to core skill metadata for mentions.
- Annotate `SKILL.md` reads in parsed commands.

Plugins:

- Browse plugin marketplaces.
- Add, remove, and upgrade marketplace sources.
- Fetch plugin detail.
- Install, uninstall, enable, and disable plugins.
- Advance post-install app-auth flow.

MCP:

- Fetch MCP inventory.
- Render MCP server status.
- Track MCP startup by server.
- Surface startup failures and cancellations.

Hooks:

- Fetch lifecycle hook inventory.
- Enable or disable hooks.
- Trust hook definitions by hash.
- Review startup hooks.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/connectors.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/connectors.rs:16`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/connectors.rs:57`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/connectors.rs:142`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/skills.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/skills.rs:73`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/skills.rs:154`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/plugins.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/plugins.rs:42`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/plugins.rs:49`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/plugins.rs:64`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/plugins.rs:202`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_event.rs:638`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_event.rs:644`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/mcp_startup.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/mcp_startup.rs:18`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/hooks_browser_view.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/hooks_rpc.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/startup_hooks_review.rs`

## Sessions, Resume, Fork, Archive, And Delete

The session layer supports:

- Startup resume picker.
- In-session resume picker.
- Fork picker.
- Resume or fork by UUID or exact name.
- Latest-session lookup.
- CWD-based filtering.
- Remote-workspace-aware CWD behavior.
- Dense and comfortable list modes.
- Transcript previews and full transcript loading.
- Archive, delete, and unarchive commands.
- Interactive delete confirmation.
- CWD conflict prompt before resume/fork.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/resume_picker.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/resume_picker.rs:66`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/resume_picker.rs:97`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/resume_picker.rs:105`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/resume_picker.rs:209`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/session_archive_commands.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/session_archive_commands.rs:216`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/session_resume.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/thread_transcript.rs:1`

## Onboarding And Auth

The onboarding UI includes:

- Welcome screen.
- Authentication step UI.
- ChatGPT browser login.
- Device-code login state.
- API-key entry.
- Trust-directory flow.
- Keyboard-driven onboarding state transitions.
- OSC 8 hyperlink marking for auth URLs.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/onboarding/mod.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/onboarding/auth.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/onboarding/auth.rs:48`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/onboarding/auth.rs:74`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/onboarding/auth/headless_chatgpt_login.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/onboarding/trust_directory.rs`

## Clipboard, Images, External Editor, And Terminal Extras

The TUI has robust local terminal utilities:

- Text copy through native clipboard, WSL PowerShell, tmux, or OSC 52.
- SSH-aware clipboard routing so copies reach the local terminal emulator.
- OSC 52 payload size cap.
- Clipboard image paste to PNG.
- WSL image-paste fallback.
- External editor launch via `VISUAL` or `EDITOR`.
- Windows command parsing and `.cmd`/`.bat` resolution.
- Temporary markdown file editing.
- Terminal title setup.
- Status line setup.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/clipboard_copy.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/clipboard_copy.rs:26`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/clipboard_copy.rs:40`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/clipboard_paste.rs:51`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/clipboard_paste.rs:121`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/external_editor.rs:33`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/external_editor.rs:54`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/title_setup.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/status_line_setup.rs`

## Terminal Pets

The TUI includes ambient terminal pets configured by `/pets`. This is an optional visual system with:

- Built-in and custom pets.
- Built-in asset caching under `CODEX_HOME`.
- Custom user-owned pets under `$CODEX_HOME/pets/<pet-id>/pet.json` or legacy avatar directories.
- Kitty graphics support.
- Kitty local file graphics support.
- Sixel support.
- Terminal capability detection.
- Picker previews.
- Clear/delete handling for previously rendered images.

This is less relevant to most operational apps, but it is a useful reference for terminal image protocols.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/pets/mod.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/pets/mod.rs:50`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/pets/mod.rs:51`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/pets/mod.rs:60`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/pets/mod.rs:100`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/pets/image_protocol.rs:27`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/pets/image_protocol.rs:34`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/pets/image_protocol.rs:112`

## Status, Usage, Rate Limits, Goals, And Plans

The TUI includes product surfaces for:

- Session configuration output.
- Account display.
- Token usage.
- Rate-limit display.
- Reset-credit flow.
- Workspace status-line headline refresh.
- Thread goals, including token budgets and status.
- Plan-mode handoff and proposed-plan display.

These are mostly Codex-specific, but the display patterns are reusable for any terminal app that needs account/status cards.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/status/mod.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/token_usage.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/goal_display.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/goal_files.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/goal_status.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/plan_implementation.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/history_cell/plans.rs`

## Testing And Verification Infrastructure

The crate has meaningful test infrastructure:

- One integration-test binary that aggregates tests under `tests/suite`.
- VT100 backend tests for terminal output.
- Snapshot tests with `insta`.
- Snapshot fixtures for diff rendering, markdown rendering, resume picker, history cells, keymap setup, status widgets, startup hooks, model migration, and more.
- Bazel test integration with test data that includes snapshot files.

References:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/tests/all.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/tests/suite/mod.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/test_backend.rs:21`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/tests/suite/vt100_history.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/tests/suite/vt100_live_commit.rs:1`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/BUILD.bazel:3`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/BUILD.bazel:25`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/BUILD.bazel:27`

Typical local commands from the repo instructions:

- `just fmt`
- `just test -p codex-tui`
- `just fix -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`
- `cargo insta accept -p codex-tui`

Note: no tests were run while creating this document. The task was a read-only survey and documentation pass.

## Best Reuse Targets

### 1. ComposerInput

Best if you want a polished prompt input without inheriting the full Codex app. It is already public and intentionally minimal.

Source:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/public_widgets/composer_input.rs`

### 2. Markdown, Hyperlink, Wrapping, And Diff Rendering

Best if your app renders model output, logs, diffs, findings, file links, or tables.

Sources:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown_render.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/terminal_hyperlinks.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/wrapping.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/diff_render.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/render/highlight.rs`

### 3. Streaming Controller Design

Best if your app streams generated text or command output. The stable/tail split and table holdback are especially useful.

Sources:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/controller.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/chunking.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown_stream.rs`

### 4. Terminal Runtime

Best if you need robust terminal setup/restore and event handling.

Sources:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/event_stream.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/frame_requester.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/frame_rate_limiter.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/job_control.rs`

### 5. Keymap And Picker Patterns

Best if your app needs user-customizable keyboard shortcuts and searchable command/list popups.

Sources:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/keymap_setup.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/list_selection_view.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/selection_tabs.rs`

### 6. Approval Overlay Pattern

Best if your app executes commands, edits files, or performs risky operations that need explicit approval.

Source:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/approval_overlay.rs`

### 7. Testing Approach

Best if you want reliable terminal UI regression tests.

Sources:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/test_backend.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/tests/suite/vt100_history.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/tests/suite/vt100_live_commit.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/snapshots`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/history_cell/snapshots`

## Harder Reuse Areas

These are valuable but coupled:

- `App` and `ChatWidget`: deeply tied to Codex app-server events, protocol objects, config, model settings, auth, permissions, plugins, skills, memory, and session state.
- App-server facade: useful if you adopt Codex's app-server protocol, otherwise it becomes an interface you need to replace.
- Plugins, apps/connectors, skills, hooks, MCP: strong product surfaces, but tied to Codex workspace crates and app-server RPCs.
- Auth/onboarding: mostly Codex-specific.
- Status/usage/rate-limit/account surfaces: mostly Codex-specific.
- Windows sandbox and permission compatibility: useful reference, but coupled to Codex permission models.

## Adoption Strategies

### Strategy A: Use It As A Reference

Use the source as a design reference and copy patterns into your own ratatui app. This is the lowest-risk path if your project is not itself a Codex-compatible agent client.

Good targets:

- Terminal setup/restore.
- Event stream pause/resume.
- Composer behavior.
- Markdown/diff rendering.
- Streaming stable/tail model.
- Approval modal UX.
- Keymap conflict resolution.
- vt100 and snapshot testing.

### Strategy B: Extract A Small TUI Kit

Create a small internal crate in your project and port selected modules. Start with low-coupling pieces:

1. `public_widgets::ComposerInput`.
2. `wrapping`, `terminal_hyperlinks`, and markdown rendering.
3. `diff_render` and `render/highlight`.
4. `key_hint`, `keymap` concepts, and list selection.
5. `tui/event_stream` concepts if your app spawns editors or subprocesses.

Expect to replace Codex-specific types with your own application events and data models.

### Strategy C: Build A Codex-Compatible Client

If your project can speak the Codex app-server protocol, you can reuse much more of the app. This is powerful but brings along many workspace crates and product assumptions.

You would need to preserve or adapt:

- `codex-app-server-client`
- `codex-app-server-protocol`
- `codex-protocol`
- `codex-config`
- `codex-login`
- `codex-state`
- `codex-rollout`
- `codex-plugin`
- `codex-core-skills`
- `codex-connectors`
- `codex-file-search`

This path makes sense only if your app's domain is close to Codex's agent/thread/turn/session model.

## Practical Recommendation

For a different project, do not start by forking the whole TUI unless you want a Codex-like agent client. Start by extracting or porting the stable UI subsystems:

1. Composer input.
2. Markdown/hyperlink/wrapping renderer.
3. Diff renderer.
4. Streaming controller.
5. Keymap and picker model.
6. Terminal lifecycle and event broker.
7. Approval overlay pattern.
8. VT100/snapshot tests.

That gives you the most value while avoiding the heavy Codex-specific app-server and product coupling.

## Source Map

Core entrypoints:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/main.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/lib.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget.rs`

Terminal runtime:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/event_stream.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/frame_requester.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/tui/frame_rate_limiter.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/custom_terminal.rs`

Composer and bottom pane:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/public_widgets/composer_input.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/mod.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/chat_composer.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/list_selection_view.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/bottom_pane/approval_overlay.rs`

Rendering:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/history_cell/mod.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown_render.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/diff_render.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/render/highlight.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/terminal_hyperlinks.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/wrapping.rs`

Streaming:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/controller.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/chunking.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/streaming/table_holdback.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/markdown_stream.rs`

App-server and sessions:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/app_server_session/fs.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/resume_picker.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/session_resume.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/session_archive_commands.rs`

Extensions:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/connectors.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/plugins.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/skills.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/chatwidget/mcp_startup.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/hooks_rpc.rs`

Utilities:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/clipboard_copy.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/clipboard_paste.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/external_editor.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/pets/mod.rs`

Tests:

- `/Users/c/Documents/GitHub/codex/codex-rs/tui/tests/all.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/tests/suite/mod.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/test_backend.rs`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/snapshots`
- `/Users/c/Documents/GitHub/codex/codex-rs/tui/src/history_cell/snapshots`
