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
pub(crate) mod workspace_messages;
#[allow(dead_code)]
pub(crate) mod status {
    use std::path::Path;

    use unicode_width::UnicodeWidthStr;

    pub(crate) fn format_tokens_compact(value: i64) -> String {
        let value = value.max(0);
        if value == 0 {
            return "0".to_string();
        }
        if value < 1_000 {
            return value.to_string();
        }

        let value_f64 = value as f64;
        let (scaled, suffix) = if value >= 1_000_000_000_000 {
            (value_f64 / 1_000_000_000_000.0, "T")
        } else if value >= 1_000_000_000 {
            (value_f64 / 1_000_000_000.0, "B")
        } else if value >= 1_000_000 {
            (value_f64 / 1_000_000.0, "M")
        } else {
            (value_f64 / 1_000.0, "K")
        };

        let decimals = if scaled < 10.0 {
            2
        } else if scaled < 100.0 {
            1
        } else {
            0
        };

        let mut formatted = format!("{scaled:.decimals$}");
        if formatted.contains('.') {
            while formatted.ends_with('0') {
                formatted.pop();
            }
            if formatted.ends_with('.') {
                formatted.pop();
            }
        }

        format!("{formatted}{suffix}")
    }

    pub(crate) fn format_directory_display(directory: &Path, max_width: Option<usize>) -> String {
        let formatted = if let Some(rel) = crate::exec_command::relativize_to_home(directory) {
            if rel.as_os_str().is_empty() {
                "~".to_string()
            } else {
                format!("~{}{}", std::path::MAIN_SEPARATOR, rel.display())
            }
        } else {
            directory.display().to_string()
        };

        if let Some(max_width) = max_width {
            if max_width == 0 {
                return String::new();
            }
            if UnicodeWidthStr::width(formatted.as_str()) > max_width {
                return crate::text_formatting::center_truncate_path(&formatted, max_width);
            }
        }

        formatted
    }
}
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
            ["onboarding", "status", "drift_tests",],
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
