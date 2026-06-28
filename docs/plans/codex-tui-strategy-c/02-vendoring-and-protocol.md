# 02 Vendoring And Protocol

## Dependency And Protocol Strategy

Use a vendor-first dependency strategy.

### Recommended For RunHaven: Vendored Codex Shape, RunHaven Backend

Keep Codex TUI source layout and vendor as many Codex crates as practical with
their original crate names. Replace only the active backend path with
RunHaven-owned adapters:

```text
crates/runhaven-tui/src/tui/app_server_session.rs
  -> crates/runhaven-tui/src/tui/runhaven/app_server_client.rs
  -> crates/runhaven-tui/src/tui/runhaven/service.rs
  -> crates/runhaven-core
```

The local client should intentionally look like Codex's
`codex-app-server-client` surface:

```rust
pub enum AppServerEvent {
    Lagged { skipped: usize },
    ServerNotification(ServerNotification),
    ServerRequest(ServerRequest),
    Disconnected { message: String },
}

pub enum AppServerClient {
    InProcess(RunHavenInProcessClient),
}

impl AppServerClient {
    pub async fn request_typed<T>(&self, request: ClientRequest)
        -> Result<T, TypedRequestError>;
    pub async fn next_event(&mut self) -> Option<AppServerEvent>;
    pub async fn shutdown(self) -> std::io::Result<()>;
    pub fn request_handle(&self) -> AppServerRequestHandle;
}
```

This preserves the Codex TUI's mental model while keeping RunHaven's runtime
authority in `runhaven-core`.

Use vendored Codex request and notification types where practical. Add
RunHaven-specific protocol extensions beside the Codex-shaped surface rather
than replacing it. Do not invent direct widget-to-core calls as shortcuts.

### Vendor Priority

Vendor in this order:

- `codex-app-server-protocol`
- `codex-protocol`
- `codex-experimental-api-macros`
- `codex-shell-command`
- `codex-ansi-escape`
- `codex-terminal-detection`
- `codex-utils-absolute-path`
- `codex-utils-approval-presets`
- `codex-utils-cli`
- `codex-utils-elapsed`
- `codex-utils-fuzzy-match`
- `codex-utils-home-dir`
- `codex-utils-oss`
- `codex-utils-path`
- `codex-utils-path-uri`
- `codex-utils-plugins`
- `codex-utils-sandbox-summary`
- `codex-utils-sleep-inhibitor`
- `codex-utils-string`
- specific low-level utility crates required by `codex-protocol`
- Codex TUI-adjacent utility crates whose APIs are referenced directly by
  copied TUI modules
- selected backend-adjacent crates as inert compatibility crates if keeping
  their names removes broad import rewrites

This lets many upstream TUI imports remain unchanged, for example:

```rust
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ServerNotification;
use codex_protocol::ThreadId;
```

`codex-app-server-client` should be treated as source to vendor or mirror
aggressively, but not as an unchanged active dependency unless RunHaven is ready
to vendor its transitive backend stack too. The observed
`codex-app-server-client/Cargo.toml` depends on `codex-app-server`,
`codex-core`, `codex-config`, `codex-exec-server`, and other Codex backend
crates.

The preferred path is:

1. Keep the `codex-app-server-client` public shape.
2. Vendor or copy the request/event plumbing that keeps `AppServerSession`
   unchanged.
3. Swap the transport target to a RunHaven in-process service.
4. Stub or feature-disable methods that would activate Codex backend behavior.

If preserving the unchanged crate is cheaper than copying its shape, vendor its
transitive dependencies too, but keep them disconnected from active RunHaven
actions until each behavior is reviewed.

Progress note, 2026-06-27: `codex-utils-cli`, `codex-utils-elapsed`, and
`codex-utils-sleep-inhibitor` are vendored under original package names and
compile as local workspace members. `codex-utils-sandbox-summary` and
`codex-utils-oss` remain tied to the larger `codex-core` compatibility closure;
do not add local stand-ins for their APIs just to force `chatwidget` forward.

### Codex TUI Manifest Coverage

The Codex TUI manifest references more internal crates than the first protocol
slice. With the vendor-first assumption, classify them instead of omitting them:

| Codex crate family | RunHaven treatment |
| --- | --- |
| `codex-app-server-protocol`, `codex-protocol`, `codex-shell-command` | Vendor first. These keep request, notification, thread, command, approval, and shell-command types close to upstream. |
| `codex-app-server-client` | Vendor or mirror its public shape aggressively. Swap active transport to RunHaven service unless the full backend is intentionally reviewed. |
| `codex-ansi-escape`, `codex-terminal-detection`, `codex-utils-*` | Vendor first. These are compatibility and terminal/helper crates, not product authority. |
| `codex-config`, `codex-login`, `codex-cloud-config`, `codex-state`, `codex-rollout` | Vendor as compatibility source if needed, but keep active RunHaven config/auth/state paths separate. |
| `codex-core`, `codex-app-server`, `codex-exec-server` | Vendor only as inert or reviewed compatibility source. Do not let TUI actions call these instead of `runhaven-core`. |
| `codex-connectors`, `codex-core-plugins`, `codex-core-skills`, `codex-plugin` | Vendor for source closeness. Keep plugin/app/connector/skill behavior fail-closed until RunHaven designs that boundary. |
| `codex-file-search`, `codex-git-utils`, `codex-message-history` | Vendor where it avoids rewrites. Activate only within RunHaven workspace and record boundaries. |
| `codex-model-provider`, `codex-model-provider-info`, `codex-models-manager` | Keep dormant unless RunHaven adds model/provider selection distinct from agent profiles. |
| `codex-feedback`, `codex-otel`, `codex-install-context` | Vendor for compilation/reference only. Activate only after RunHaven has consent, redaction, and telemetry policy. |
| `codex-sandboxing`, `codex-windows-sandbox` | Vendor as source reference. RunHaven's macOS container boundary remains authoritative. |
| `codex-arg0` | Needed only if RunHaven preserves Codex-style binary dispatch or remote app-server startup paths. |

Current low-coupling support crates already vendored under original names for
the next config closure: `codex-context-fragments`, `codex-install-context`,
`codex-memories-read`, `codex-response-debug-context`,
`codex-utils-output-truncation`, and `codex-utils-stream-parser`. They are
compile/reference authorities only until a later slice promotes the reduced
`codex-core` config compatibility path.

This table is intentionally permissive about vendoring and strict about active
authority. The mistake to avoid is not copying Codex source. The mistake is
letting copied Codex backend paths become the path that launches or mutates
RunHaven runs.

## Dependency Plan

Current `runhaven-tui` has only the dependency subset needed by the staged
facade. Activating more Codex source will require more dependencies from
Codex `tui/Cargo.toml`.

Before activating the full event stream, audit dependency features as well as
crate names. In particular, Codex `TuiEventStream` needs the crossterm
`event-stream` and `bracketed-paste` features, and any module that calls
runtime `rand` APIs needs `rand` as a normal dependency instead of a
dev-dependency. Treat missing features as compile blockers, not optional polish.

Likely additions for full TUI source activation:

- `anyhow`
- `base64`
- `chrono`
- `clap` if RunHaven keeps TUI-specific CLI argument parsing
- `color-eyre`
- `derive_more`
- `diffy`
- `dirs`
- `dunce`
- `image` for terminal image and pet assets
- `itertools`
- `lazy_static`
- `pathdiff`
- `pulldown-cmark`
- `rand`
- `ratatui-macros`
- `reqwest` only for any retained blocking/json metadata checks
- `regex-lite`
- `rmcp` only while MCP protocol forms remain vendored
- `serde`
- `serde_json`
- `sha2`
- `shlex`
- `strum`
- `strum_macros`
- `supports-color`
- `syntect`
- `tempfile`
- `textwrap`
- `thiserror`
- `tokio-stream`
- `tokio-util`
- `toml` with matching needs
- `tracing`
- `tracing-appender`
- `tracing-subscriber`
- `two-face`
- `unicode-segmentation`
- `unicode-width`
- `uuid`
- `url`
- `urlencoding`
- `webbrowser`

Platform-specific:

- `libc` is already present on Unix.
- `windows-sys`, `which`, and `winsplit` only matter if RunHaven keeps Windows
  compile support for copied Codex modules. RunHaven product runtime is macOS
  26+ only, but copied code may still need conditional stubs to compile.
- `arboard` is needed only if RunHaven activates native clipboard support.

Dev/test additions from Codex TUI worth preserving for source-close tests:

- `app_test_support`
- `assert_matches`
- `codex-cli`
- `codex-mcp`
- `codex-utils-cargo-bin`
- `core_test_support`
- `insta`
- `pretty_assertions`
- `serial_test`
- `vt100`
- `wiremock`

Internal Codex crate imports should be satisfied in this order:

- Vendored Codex protocol and utility crates with original crate names.
- Vendored backend-adjacent compatibility crates when keeping the original name
  prevents broad TUI import churn.
- RunHaven-owned facade types at active product boundaries.
- Local compatibility modules only when vendoring would pull in behavior that
  cannot stay inert.
- Focused edits to upstream imports, documented in `tui/README.md`, only when
  a facade or vendored crate would be heavier than the change.

`codex-core` is not forbidden as vendored source. It is forbidden as an
unreviewed active backend path for RunHaven behavior.

RunHaven exact-pins dependencies. Any dependency change should update
`Cargo.lock` and run the normal workspace checks.
