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

fn tui_source_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tui")
}

fn tui_rust_sources() -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![tui_source_dir()];
    while let Some(path) = pending.pop() {
        let metadata = std::fs::metadata(&path).expect("metadata should be readable");
        if metadata.is_dir() {
            for entry in std::fs::read_dir(&path).expect("directory should be readable") {
                pending.push(entry.expect("directory entry should be readable").path());
            }
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn relative_tui_source(path: &std::path::Path) -> std::path::PathBuf {
    path.strip_prefix(tui_source_dir())
        .expect("path should be under tui source")
        .to_path_buf()
}

fn is_guard_source(path: &std::path::Path) -> bool {
    matches!(path.to_str(), Some("mod.rs") | Some("drift_tests.rs"))
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

fn inline_module_block<'a>(module_source: &'a str, module: &str) -> &'a str {
    let declaration = format!("pub(crate) mod {module} {{");
    let start = module_source
        .find(&declaration)
        .unwrap_or_else(|| panic!("inline module {module} should be declared"));
    let rest = &module_source[start..];
    let mut depth = 0usize;

    for (offset, character) in rest.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth
                    .checked_sub(1)
                    .expect("inline module block should have balanced braces");
                if depth == 0 {
                    return &rest[..=offset];
                }
            }
            _ => {}
        }
    }

    panic!("inline module {module} should have a closing brace");
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
        ["onboarding"],
        "tui/mod.rs may shrink inline staging modules, but must not grow new stand-ins"
    );
}

#[test]
fn inline_onboarding_shim_stays_link_only_until_auth_boundary_exists() {
    let module_source = include_str!("mod.rs");
    let onboarding_source = tui_rust_sources()
        .into_iter()
        .filter(|path| relative_tui_source(path).starts_with("onboarding"))
        .map(|path| std::fs::read_to_string(path).expect("onboarding source should be readable"))
        .collect::<Vec<_>>()
        .join("\n");
    let expected_shim = r#"pub(crate) mod onboarding {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    pub(crate) fn mark_underlined_hyperlink(buf: &mut Buffer, area: Rect, url: &str) {
        crate::terminal_hyperlinks::mark_underlined_hyperlink(buf, area, url);
    }
}"#;

    assert!(
        !module_declared(module_source, "onboarding"),
        "do not activate the full onboarding module while onboarding/auth.rs still carries login, browser, app-server, and environment-key behavior"
    );
    assert_eq!(
        inline_module_block(module_source, "onboarding"),
        expected_shim,
        "the temporary onboarding shim may expose only the hyperlink helper needed by active vendored widgets"
    );
    for marker in [
        "read_openai_api_key_from_env",
        "webbrowser::open",
        "AppServerRequestHandle",
        "OPENAI_API_KEY",
        "headless_chatgpt_login",
        "codex_login",
    ] {
        assert!(
            !module_source.contains(marker),
            "the inline onboarding shim must not grow host-reaching auth marker {marker:?}"
        );
    }
    for marker in [
        "read_openai_api_key_from_env",
        "webbrowser::open",
        "AppServerRequestHandle",
        "OPENAI_API_KEY",
        "codex_login",
    ] {
        assert!(
            onboarding_source.contains(marker),
            "update the onboarding dormancy guard if risky onboarding marker {marker:?} moves outside onboarding/ or is removed"
        );
    }
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
            && manifest_source.contains("codex-file-search = { path = \"../codex/file-search\" }")
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
        include_str!("runhaven/mvp.rs"),
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
        include_str!("runhaven/launch_handoff.rs"),
        include_str!("runhaven/protocol.rs"),
        include_str!("runhaven/service.rs"),
        include_str!("runhaven/launch_wizard.rs"),
        include_str!("runhaven/mvp.rs"),
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
fn active_run_log_snapshot_route_stays_in_runhaven_facade() {
    let active_snapshot_marker = ["active_run_", "log_snapshot_payload"].concat();
    let method_marker = ["runhaven/run/", "logSnapshot"].concat();
    let service_source = include_str!("runhaven/service.rs");
    let protocol_source = include_str!("runhaven/protocol.rs");
    let session_source = include_str!("runhaven/app_server_session.rs");
    let app_shell_source = include_str!("app_shell.rs");
    let owners = tui_rust_sources()
        .into_iter()
        .filter_map(|path| {
            let relative = relative_tui_source(&path);
            if is_guard_source(&relative) {
                return None;
            }
            let source = std::fs::read_to_string(&path).expect("source should be readable");
            source
                .contains(&active_snapshot_marker)
                .then(|| relative.display().to_string())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        owners,
        ["runhaven/service.rs"],
        "only the RunHaven TUI service facade may call the active-run log snapshot runtime API"
    );
    assert!(
        protocol_source.contains(&method_marker)
            && session_source.contains("run_log_snapshot")
            && service_source.contains("self.active_run_log_snapshot_data(")
            && service_source.contains(
                "self.validate_sensitive_log_confirmation(confirm_sensitive_output, method)?;"
            )
            && service_source.contains("self.validate_log_snapshot_lines(lines, method)?;")
            && service_source
                .contains("self.active_run_log_snapshot_payload(run_id, lines, method)"),
        "log snapshots must stay a typed RunHaven method with sensitive-output confirmation and validation before backend lookup"
    );
    for marker in [
        method_marker.as_str(),
        active_snapshot_marker.as_str(),
        "run_log_snapshot",
    ] {
        assert!(
            !app_shell_source.contains(marker),
            "the temporary app_shell must not grow active-run log product behavior or direct runtime calls"
        );
    }
}

#[test]
fn visible_run_log_ui_stays_confirmation_gated_in_runhaven_mvp() {
    let mvp_source = include_str!("runhaven/mvp.rs");
    let app_shell_source = include_str!("app_shell.rs");
    let log_text_marker = ["snapshot", ".text.lines().take"].concat();
    let owners = tui_rust_sources()
        .into_iter()
        .filter_map(|path| {
            let relative = relative_tui_source(&path);
            if is_guard_source(&relative) {
                return None;
            }
            let source = std::fs::read_to_string(&path).expect("source should be readable");
            source
                .contains(&log_text_marker)
                .then(|| relative.display().to_string())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        owners,
        ["runhaven/mvp.rs"],
        "raw log text rendering must stay in the RunHaven MVP view"
    );
    assert!(
        mvp_source.contains("LOG_CONFIRM_PHRASE")
            && mvp_source.contains("Raw container output can contain secrets")
            && mvp_source.contains("typed.trim() != LOG_CONFIRM_PHRASE")
            && mvp_source.contains("active_run_log_snapshot_data(")
            && mvp_source.contains("\"runhaven/run/logSnapshot\""),
        "visible log UI must require typed confirmation before loading raw output"
    );
    for marker in [
        "Raw container output",
        log_text_marker.as_str(),
        "active_run_log_snapshot_data",
    ] {
        assert!(
            !app_shell_source.contains(marker),
            "app_shell.rs must not render or load active-run logs"
        );
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
        include_str!("runhaven/launch_handoff.rs"),
        include_str!("runhaven/protocol.rs"),
        include_str!("runhaven/service.rs"),
        include_str!("runhaven/launch_wizard.rs"),
        include_str!("runhaven/mvp.rs"),
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
fn session_log_uses_source_first_boundary_without_active_recording() {
    let module_source = include_str!("mod.rs");
    let bridge_source = include_str!("app_event_shared.rs");

    assert!(
        module_declared(module_source, "session_log"),
        "ChatWidget/AppEvent promotion must use the real vendored session_log.rs source"
    );
    assert!(
        !bridge_source.contains("mod session_log"),
        "app_event_shared.rs must not keep a session_log bridge once real session_log.rs is active"
    );

    let maybe_init_marker = ["session_log::", "maybe_init"].concat();
    let recording_env_marker = ["CODEX_TUI_", "RECORD_SESSION"].concat();
    for path in tui_rust_sources() {
        let relative = relative_tui_source(&path);
        if matches!(relative.to_str(), Some("session_log.rs") | Some("lib.rs")) {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("source should be readable");
        assert!(
            !source.contains(&maybe_init_marker) && !source.contains(&recording_env_marker),
            "only session_log.rs and dormant tui/lib.rs may mention Codex session recording markers; found one in {}",
            relative.display()
        );
    }
}

#[test]
fn foreground_runtime_launch_call_stays_in_ui_thread_handoff_owner() {
    let app_event_sender_source = include_str!("app_event_sender.rs");
    let launch_handoff_source = include_str!("runhaven/launch_handoff.rs");
    let runtime_launch_marker = ["launch", "_run", "_plan"].concat();
    let owners = tui_rust_sources()
        .into_iter()
        .filter_map(|path| {
            let relative = relative_tui_source(&path);
            let source = std::fs::read_to_string(&path).expect("source should be readable");
            source
                .contains(&runtime_launch_marker)
                .then(|| relative.display().to_string())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        owners,
        ["runhaven/launch_handoff.rs"],
        "only the UI-thread handoff owner may call the foreground runtime launch function"
    );
    assert!(
        launch_handoff_source.contains(&runtime_launch_marker)
            && launch_handoff_source.contains("with_restored("),
        "RunHaven foreground launch must have one UI-thread handoff owner that restores the Codex terminal before launch"
    );
    assert!(
        app_event_sender_source.contains("AppEvent::RunHavenLaunchPrepared { .. }")
            && app_event_sender_source.contains("session_log::log_inbound_app_event(&event)"),
        "RunHaven launch-prepared events carry full plan data and must stay excluded from Codex session logging"
    );
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
