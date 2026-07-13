use super::*;

#[test]
fn main_targeted_menu_command_recreates_shows_and_focuses_missing_main_window() {
    let mut effects = LifecycleProbe::missing();

    dispatch_main_menu_for_test(&mut effects, "new-session").expect("dispatch menu command");

    assert_eq!(
        effects.calls,
        ["create-main", "show-main", "focus-main", "emit:new-session"]
    );
}

#[test]
fn main_targeted_menu_command_shows_and_focuses_hidden_main_window() {
    let mut effects = LifecycleProbe::hidden();

    dispatch_main_menu_for_test(&mut effects, "help").expect("dispatch menu command");

    assert_eq!(effects.calls, ["show-main", "focus-main", "emit:help"]);
}

#[test]
fn dock_reopen_restores_missing_main_window() {
    let mut effects = LifecycleProbe::missing();

    dock_reopen_for_test(&mut effects).expect("restore main window from Dock");

    assert_eq!(effects.calls, ["create-main", "show-main", "focus-main"]);
}
