use super::*;

#[test]
fn ordinary_main_window_destroy_does_not_clean_active_work() {
    let mut effects = LifecycleProbe::hidden();

    main_window_destroyed_for_test(&mut effects);

    assert_eq!(
        effects.calls,
        Vec::<String>::new(),
        "closing the control window must not terminate active sessions or jobs"
    );
}

#[test]
fn routine_main_window_close_is_prevented_and_hides_the_live_webview() {
    let mut effects = LifecycleProbe::hidden();

    main_window_close_requested_for_test(&mut effects).expect("hide main window");

    assert_eq!(effects.calls, ["prevent-close", "hide-main"]);
}

#[test]
fn actual_app_exit_cleans_terminal_and_background_registries() {
    let mut effects = LifecycleProbe::hidden();

    app_exit_for_test(&mut effects);

    assert_eq!(
        effects.calls,
        ["close-terminals", "shutdown-background"],
        "process exit must still terminate every app-owned child"
    );
}
