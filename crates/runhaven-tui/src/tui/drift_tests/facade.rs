use super::*;

#[test]
fn staging_facade_inline_modules_do_not_grow() {
    let module_source = include_str!("../mod.rs");
    let inline_modules = inline_module_declarations(module_source);

    assert_eq!(
        inline_modules,
        ["onboarding"],
        "tui/mod.rs may shrink inline staging modules, but must not grow new stand-ins"
    );
}

#[test]
fn inline_onboarding_shim_stays_link_only_until_auth_boundary_exists() {
    let module_source = include_str!("../mod.rs");
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
    let module_source = include_str!("../mod.rs");
    let manifest_source = include_str!("../../../Cargo.toml");

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
    let lib_source = include_str!("../../lib.rs");
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
    let module_source = include_str!("../mod.rs");
    let root_lib_source = include_str!("../../lib.rs");
    let vendor_workspace_manifest_source = include_str!("../../../../codex/Cargo.toml");
    let bridge_source = include_str!("../app_event_shared.rs");
    let runhaven_sources = [
        include_str!("../runhaven/app_server_client.rs"),
        include_str!("../runhaven/app_server_session.rs"),
        include_str!("../runhaven/protocol.rs"),
        include_str!("../runhaven/service.rs"),
        include_str!("../runhaven/launch_wizard.rs"),
        include_str!("../runhaven/mvp.rs"),
        include_str!("../runhaven/terminal_handoff.rs"),
    ];

    assert!(
        !module_declared(module_source, "legacy_core")
            && !module_declared(root_lib_source, "legacy_core"),
        "do not add a local legacy_core shim; vendor the real Codex compatibility path"
    );
    assert!(
        workspace_member_declared(vendor_workspace_manifest_source, "core"),
        "codex-core config compatibility authority must be a real member of the isolated vendored workspace"
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
        include_str!("../runhaven/app_server_client.rs"),
        include_str!("../runhaven/app_server_session.rs"),
        include_str!("../runhaven/launch_handoff.rs"),
        include_str!("../runhaven/protocol.rs"),
        include_str!("../runhaven/service.rs"),
        include_str!("../runhaven/launch_wizard.rs"),
        include_str!("../runhaven/mvp.rs"),
        include_str!("../runhaven/terminal_handoff.rs"),
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
