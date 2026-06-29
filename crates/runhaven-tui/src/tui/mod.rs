use anyhow::Result;

#[allow(dead_code)]
pub(crate) mod app_command;
#[allow(dead_code)]
pub(crate) mod app_event;
#[allow(dead_code)]
mod app_event_shared;
#[allow(dead_code)]
pub(crate) mod app_server_approval_conversions;
mod app_shell;
mod runhaven;

#[allow(dead_code)]
pub(crate) mod color;
#[allow(dead_code)]
pub(crate) mod custom_terminal;

pub(crate) use app_event_shared::app;
pub(crate) use app_event_shared::app_server_session;
pub(crate) use app_event_shared::chatwidget;
pub(crate) use app_event_shared::goal_files;
pub(crate) use app_event_shared::hooks_rpc;
pub(crate) use app_event_shared::session_log;

#[allow(dead_code)]
pub(crate) mod app_event_sender;

#[allow(dead_code, unused_imports)]
pub(crate) mod bottom_pane;
#[allow(dead_code)]
pub(crate) mod branch_summary;

#[allow(dead_code)]
pub(crate) mod clipboard_paste;

#[allow(dead_code)]
pub(crate) mod approval_events;
#[allow(dead_code)]
pub(crate) mod diff_model;
#[allow(dead_code)]
pub(crate) mod diff_render;
#[allow(dead_code, unused_imports)]
pub(crate) mod exec_cell;
#[allow(dead_code)]
pub(crate) mod exec_command;

#[allow(dead_code, unused_imports)]
pub(crate) mod key_hint;
#[allow(dead_code)]
pub(crate) mod keymap;
#[allow(dead_code)]
pub(crate) mod line_truncation;
#[allow(dead_code)]
pub(crate) mod live_wrap;
#[allow(dead_code)]
pub(crate) mod markdown;
#[allow(dead_code)]
pub(crate) mod markdown_render;
#[allow(dead_code)]
pub(crate) mod markdown_stream;
#[allow(dead_code)]
pub(crate) mod markdown_text_merge;
#[allow(dead_code)]
pub(crate) mod mention_codec;
#[allow(dead_code)]
pub(crate) mod motion;
#[allow(dead_code)]
pub(crate) mod notifications;
#[allow(dead_code)]
pub(crate) mod onboarding {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    pub(crate) fn mark_underlined_hyperlink(buf: &mut Buffer, area: Rect, url: &str) {
        crate::terminal_hyperlinks::mark_underlined_hyperlink(buf, area, url);
    }
}
#[allow(dead_code, unused_imports)]
pub(crate) mod pets;
#[allow(dead_code)]
pub(crate) mod render;
#[allow(dead_code)]
pub(crate) mod session_state;
#[allow(dead_code)]
pub(crate) mod shimmer;
#[allow(dead_code)]
pub(crate) mod skills_helpers;
#[allow(dead_code)]
pub(crate) mod slash_command;
#[allow(dead_code)]
pub(crate) mod status_indicator_widget;
#[allow(dead_code)]
pub(crate) mod style;
#[allow(dead_code)]
pub(crate) mod table_detect;
#[allow(dead_code)]
pub(crate) mod terminal_hyperlinks;
#[allow(dead_code)]
pub(crate) mod terminal_palette;
#[allow(dead_code)]
pub(crate) mod terminal_probe;
#[allow(dead_code)]
pub(crate) mod terminal_title;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod test_backend;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod test_support;
#[allow(dead_code)]
pub(crate) mod text_formatting;
#[allow(dead_code)]
pub(crate) mod token_usage;
#[allow(dead_code)]
pub(crate) mod tooltips;
#[allow(dead_code)]
pub(crate) mod ui_consts;
#[allow(dead_code)]
pub(crate) mod update_action;
#[allow(dead_code)]
pub(crate) mod version;
#[allow(dead_code)]
pub(crate) mod width;
#[allow(dead_code)]
pub(crate) mod workspace_command;
#[allow(dead_code)]
pub(crate) mod workspace_messages;
#[allow(dead_code)]
pub(crate) mod wrapping;

#[allow(dead_code, unused_imports)]
pub(crate) mod history_cell;
#[allow(dead_code)]
pub(crate) mod insert_history;

#[allow(dead_code)]
#[path = "tui.rs"]
pub(crate) mod codex_runtime;

pub use codex_runtime::FrameRequester;

pub fn run() -> Result<i32> {
    if let Some(exit_code) = runhaven::terminal_handoff::run_smoke_from_env()? {
        return Ok(exit_code);
    }

    app_shell::run()
}

#[cfg(test)]
mod drift_tests {
    fn inline_module_declarations(module_source: &str) -> Vec<String> {
        module_source
            .lines()
            .map(str::trim)
            .filter_map(|line| {
                ["pub(crate) mod ", "pub mod ", "mod "]
                    .iter()
                    .find_map(|prefix| {
                        line.strip_prefix(prefix)
                            .and_then(|rest| rest.strip_suffix(" {"))
                    })
            })
            .map(str::to_string)
            .collect()
    }

    fn top_level_inline_module_declarations(module_source: &str) -> Vec<String> {
        module_source
            .lines()
            .filter_map(|line| {
                ["pub(crate) mod ", "pub mod ", "mod "]
                    .iter()
                    .find_map(|prefix| {
                        line.strip_prefix(prefix)
                            .and_then(|rest| rest.strip_suffix(" {"))
                    })
            })
            .map(str::to_string)
            .collect()
    }

    fn module_declared(module_source: &str, module: &str) -> bool {
        let private_decl = format!("mod {module};");
        let crate_decl = format!("pub(crate) mod {module};");
        let public_decl = format!("pub mod {module};");
        module_source
            .lines()
            .map(str::trim)
            .any(|line| line == private_decl || line == crate_decl || line == public_decl)
    }

    fn workspace_member_declared(root_manifest_source: &str, member: &str) -> bool {
        let quoted = format!("\"{member}\"");
        root_manifest_source
            .lines()
            .map(str::trim)
            .any(|line| line.trim_end_matches(',') == quoted)
    }

    fn assert_risky_markers_absent_when_active(
        module_source: &str,
        module: &str,
        source_path: &str,
        source: &str,
        markers: &[&str],
    ) {
        if !module_declared(module_source, module) {
            return;
        }

        for marker in markers {
            assert!(
                !source.contains(marker),
                "{module} is declared in tui/mod.rs, but {source_path} still contains risky upstream marker {marker:?}; remove or fail-close that behavior before activating the module"
            );
        }
    }

    #[test]
    fn staging_facade_inline_modules_do_not_grow() {
        let module_source = include_str!("mod.rs");
        let inline_modules = inline_module_declarations(module_source);

        assert_eq!(
            inline_modules,
            ["onboarding", "drift_tests",],
            "tui/mod.rs may shrink inline staging modules, but must not grow new stand-ins"
        );
    }

    #[test]
    fn codex_crates_are_vendored_dependencies() {
        let module_source = include_str!("mod.rs");
        let manifest_source = include_str!("../../Cargo.toml");

        assert!(
            !module_declared(module_source, "codex_protocol"),
            "codex_protocol must be consumed from the vendored crate, not staged inside runhaven-tui"
        );
        assert!(
            manifest_source.contains("codex-protocol = { path = \"../codex/protocol\" }")
                && manifest_source.contains(
                    "codex-app-server-protocol = { path = \"../codex/app-server-protocol\" }"
                ),
            "runhaven-tui must depend on the real vendored Codex protocol crates"
        );
        assert!(
            !module_declared(module_source, "codex_config"),
            "codex_config must be consumed from the vendored crate, not staged inside runhaven-tui"
        );
        assert!(
            manifest_source.contains("codex-config = { path = \"../codex/config\" }"),
            "runhaven-tui must depend on the real vendored Codex config crate"
        );
        assert!(
            manifest_source.contains("codex-connectors = { path = \"../codex/connectors\" }")
                && manifest_source.contains("codex-features = { path = \"../codex/features\" }")
                && manifest_source
                    .contains("codex-file-search = { path = \"../codex/file-search\" }")
                && manifest_source.contains("codex-plugin = { path = \"../codex/plugin\" }")
                && manifest_source.contains(
                    "codex-utils-absolute-path = { path = \"../codex/utils/absolute-path\" }"
                )
                && manifest_source.contains(
                    "codex-utils-approval-presets = { path = \"../codex/utils/approval-presets\" }"
                ),
            "runhaven-tui must depend on the real vendored Codex event-data crates needed by app_event.rs"
        );
    }

    #[test]
    fn codex_self_aliases_do_not_grow() {
        let lib_source = include_str!("../lib.rs");
        let aliases = lib_source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("extern crate self as codex_"))
            .collect::<Vec<_>>();

        assert!(
            aliases.is_empty(),
            "do not add codex_* self-aliases; vendor real Codex crates or shrink local shims"
        );
    }

    #[test]
    fn legacy_core_boundary_stays_vendor_first() {
        let module_source = include_str!("mod.rs");
        let root_lib_source = include_str!("../lib.rs");
        let root_manifest_source = include_str!("../../../../Cargo.toml");
        let bridge_source = include_str!("app_event_shared.rs");
        let runhaven_sources = [
            include_str!("runhaven/app_server_client.rs"),
            include_str!("runhaven/app_server_session.rs"),
            include_str!("runhaven/protocol.rs"),
            include_str!("runhaven/service.rs"),
            include_str!("runhaven/launch_wizard.rs"),
            include_str!("runhaven/terminal_handoff.rs"),
        ];

        assert!(
            !module_declared(module_source, "legacy_core")
                && !module_declared(root_lib_source, "legacy_core"),
            "do not add a local legacy_core shim; vendor the real Codex compatibility path"
        );
        assert!(
            workspace_member_declared(root_manifest_source, "crates/codex/core"),
            "codex-core config compatibility authority must be a real original-name workspace crate"
        );
        assert!(
            !bridge_source.contains("legacy_core"),
            "app_event_shared.rs must not grow legacy_core compatibility behavior"
        );
        for source in runhaven_sources {
            assert!(
                !source.contains("crate::legacy_core") && !source.contains("legacy_core::"),
                "RunHaven-owned TUI adapters must not import legacy_core directly"
            );
        }
    }

    #[test]
    fn runhaven_adapters_do_not_import_codex_core_runtime_surfaces() {
        let runhaven_sources = [
            include_str!("runhaven/app_server_client.rs"),
            include_str!("runhaven/app_server_session.rs"),
            include_str!("runhaven/protocol.rs"),
            include_str!("runhaven/service.rs"),
            include_str!("runhaven/launch_wizard.rs"),
            include_str!("runhaven/terminal_handoff.rs"),
        ];

        for source in runhaven_sources {
            for forbidden in [
                "codex_core::session",
                "codex_core::exec",
                "codex_core::mcp",
                "codex_core::shell",
                "codex_core::spawn",
                "codex_core::thread_manager",
                "codex_core::tools",
                "codex_core::rollout",
                "codex_core::state",
                "codex_core::client",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "RunHaven-owned TUI adapters must not import runtime Codex core surface {forbidden}"
                );
            }
        }
    }

    #[test]
    fn chatwidget_branch_summary_uses_source_first_boundary() {
        let module_source = include_str!("mod.rs");
        let bridge_source = include_str!("app_event_shared.rs");
        let branch_summary_source = include_str!("branch_summary.rs");
        let workspace_command_source = include_str!("workspace_command.rs");
        let app_shell_source = include_str!("app_shell.rs");
        let runhaven_sources = [
            include_str!("runhaven/app_server_client.rs"),
            include_str!("runhaven/app_server_session.rs"),
            include_str!("runhaven/protocol.rs"),
            include_str!("runhaven/service.rs"),
            include_str!("runhaven/launch_wizard.rs"),
            include_str!("runhaven/terminal_handoff.rs"),
        ];

        assert!(
            module_declared(module_source, "branch_summary")
                && module_declared(module_source, "workspace_command"),
            "ChatWidget status promotion must use the real branch_summary.rs plus workspace_command.rs source boundary"
        );
        assert!(
            !bridge_source.contains("StatusLineGitSummary"),
            "app_event_shared.rs must not keep the StatusLineGitSummary bridge once branch_summary.rs is active"
        );
        for marker in [
            "std::process::Command",
            "use tokio::process",
            "tokio::process::Command::new",
            "std::env",
            "std::fs",
            "runhaven_core::",
        ] {
            assert!(
                !branch_summary_source.contains(marker),
                "branch_summary.rs must stay best-effort metadata over WorkspaceCommandExecutor, not direct host access marker {marker:?}"
            );
        }
        assert!(
            workspace_command_source
                .contains("#[cfg(any())]\npub(crate) struct AppServerWorkspaceCommandRunner")
                && workspace_command_source.contains("ClientRequest::OneOffCommandExec"),
            "workspace_command.rs may carry the upstream app-server runner only while compiled dormant"
        );
        assert!(
            !app_shell_source.contains("WorkspaceCommandRunner")
                && !app_shell_source.contains("AppServerWorkspaceCommandRunner"),
            "the temporary app_shell must not start app-server workspace command execution"
        );
        for source in runhaven_sources {
            assert!(
                !source.contains("AppServerWorkspaceCommandRunner"),
                "RunHaven-owned adapters must not activate app-server workspace command execution in this slice"
            );
        }
    }

    #[test]
    fn token_usage_uses_source_first_boundary_before_full_status() {
        let module_source = include_str!("mod.rs");
        let root_lib_source = include_str!("../lib.rs");
        let footer_source = include_str!("bottom_pane/footer.rs");
        let hooks_browser_source = include_str!("bottom_pane/hooks_browser_view.rs");

        assert!(
            module_declared(module_source, "token_usage"),
            "ChatWidget/status integration must use the real token_usage.rs source model"
        );
        assert!(
            !module_declared(module_source, "status"),
            "Keep full status/ dormant until its config, model-provider, remote app-server, and status-card closure is promoted"
        );
        assert!(
            !root_lib_source.contains("pub(crate) use tui::status;"),
            "do not re-export a fake root status surface before full status/ is intentionally promoted"
        );
        for source in [footer_source, hooks_browser_source] {
            assert!(
                !source.contains("crate::status::") && !source.contains("use crate::status"),
                "active bottom-pane code must use the small RunHaven status_format helper instead of a root status bridge"
            );
        }
    }

    #[test]
    fn app_event_shared_shrinks_only() {
        let source = include_str!("app_event_shared.rs");
        let inline_modules = top_level_inline_module_declarations(source);

        assert_eq!(
            inline_modules,
            [
                "app",
                "app_server_session",
                "chatwidget",
                "goal_files",
                "hooks_rpc",
                "session_log",
            ],
            "app_event_shared.rs may shrink as real Codex modules activate, but must not grow new bridge modules"
        );
        for marker in [
            "std::env",
            "std::fs",
            "std::process::Command",
            "reqwest",
            "runhaven_core::",
            "legacy_core",
        ] {
            assert!(
                !source.contains(marker),
                "app_event_shared.rs must stay an inert type bridge and not grow host-reaching marker {marker:?}"
            );
        }
    }

    #[test]
    fn native_app_entrypoint_cannot_share_temporary_shell() {
        let module_source = include_str!("mod.rs");

        if module_declared(module_source, "app") {
            assert!(
                !module_source.contains("app_shell::run()"),
                "native app activation must move run() off the temporary app_shell entrypoint"
            );
        }
    }

    #[test]
    fn host_reaching_codex_surfaces_stay_dormant_until_sanitized() {
        let module_source = include_str!("mod.rs");

        assert_risky_markers_absent_when_active(
            module_source,
            "app",
            "app.rs",
            include_str!("app.rs"),
            &["std::env::vars().collect"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "app_server_session",
            "app_server_session.rs",
            include_str!("app_server_session.rs"),
            &["mod fs;"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "chatwidget",
            "chatwidget.rs",
            include_str!("chatwidget.rs"),
            &[
                "crate::clipboard_copy::ClipboardLease",
                "ExternalEditorState",
            ],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "onboarding",
            "onboarding/auth.rs",
            include_str!("onboarding/auth.rs"),
            &["read_openai_api_key_from_env", "webbrowser::open"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "local_chatgpt_auth",
            "local_chatgpt_auth.rs",
            include_str!("local_chatgpt_auth.rs"),
            &["OPENAI_API_KEY", "ChatGPT"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "external_editor",
            "external_editor.rs",
            include_str!("external_editor.rs"),
            &["std::process::Command", "EDITOR"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "clipboard_copy",
            "clipboard_copy.rs",
            include_str!("clipboard_copy.rs"),
            &["std::process::Command"],
        );
        assert_risky_markers_absent_when_active(
            module_source,
            "hooks_rpc",
            "hooks_rpc.rs",
            include_str!("hooks_rpc.rs"),
            &["hook", "Hook"],
        );
    }
}
