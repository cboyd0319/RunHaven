use super::*;

#[test]
fn native_app_and_chatwidget_stay_dormant_under_runhaven_shell() {
    let module_source = include_str!("../mod.rs");
    let app_shell_source = include_str!("../app_shell.rs");
    let readme_source = include_str!("../README.md");

    assert!(
        module_source.contains("pub(crate) use app_event_shared::app;")
            && module_source.contains("pub(crate) use app_event_shared::chatwidget;"),
        "the active RunHaven shell may keep only inert app/chatwidget bridge exports"
    );
    assert!(
        !module_declared(module_source, "app")
            && !module_declared(module_source, "chatwidget")
            && !source_file_declared_by_path(module_source, "app.rs")
            && !source_file_declared_by_path(module_source, "chatwidget.rs"),
        "native Codex App and ChatWidget must stay dormant while app_shell.rs hosts the RunHaven view, including path-aliased source activation"
    );
    assert!(
        app_shell_source.contains("let mvp_view = RunHavenMvpView::new(workspace.clone());")
            && app_shell_source.contains("BottomPane::new(BottomPaneParams"),
        "the active shell should construct the RunHaven view and the real BottomPane"
    );
    assert!(
        app_shell_source.contains("bottom_pane.show_view(Box::new(mvp_view));"),
        "the active shell must install the RunHaven view into BottomPane"
    );
    for marker in [
        "crate::tui::app::App",
        "crate::tui::chatwidget::ChatWidget",
        "App::run(",
        "ChatWidget::",
    ] {
        assert!(
            !app_shell_source.contains(marker),
            "app_shell.rs must not activate native App/ChatWidget marker {marker:?}"
        );
    }
    assert!(
        readme_source.contains("The runtime entrypoint is `app_shell::run`")
            && readme_source.contains("launch, recovery,")
            && readme_source.contains("active-run, history, and diagnostic behavior")
            && readme_source.contains("delegates product truth to `runhaven-core`")
            && readme_source.contains("alternate owner for container policy"),
        "README.md must retain the RunHaven shell and product-truth ownership boundary"
    );
}

#[test]
fn native_app_entrypoint_cannot_share_temporary_shell() {
    let module_source = include_str!("../mod.rs");

    if module_declared(module_source, "app") {
        assert!(
            !module_source.contains("app_shell::run()"),
            "native app activation must move run() off the temporary app_shell entrypoint"
        );
    }
}

#[test]
fn host_reaching_codex_surfaces_stay_dormant_until_sanitized() {
    let module_source = include_str!("../mod.rs");

    assert_risky_markers_absent_when_active(
        module_source,
        "app_server_session",
        "app_server_session.rs",
        include_str!("../app_server_session.rs"),
        &["mod fs;"],
    );
    assert_risky_markers_absent_when_active(
        module_source,
        "onboarding",
        "onboarding/auth.rs",
        include_str!("../onboarding/auth.rs"),
        &["read_openai_api_key_from_env", "webbrowser::open"],
    );
    assert_risky_markers_absent_when_active(
        module_source,
        "local_chatgpt_auth",
        "local_chatgpt_auth.rs",
        include_str!("../local_chatgpt_auth.rs"),
        &["OPENAI_API_KEY", "ChatGPT"],
    );
    assert_risky_markers_absent_when_active(
        module_source,
        "hooks_rpc",
        "hooks_rpc.rs",
        include_str!("../hooks_rpc.rs"),
        &["hook", "Hook"],
    );
}
