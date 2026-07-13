#[test]
fn native_confirmations_explain_consequences_without_internal_metadata() {
    let mut messages = vec![
        crate::commands::session_cleanup::cleanup_confirmation_message_for_test().to_string(),
        crate::commands::managed_session_input::session_input_confirmation_message_for_test(
            "Session", "hello",
        ),
        crate::commands::image_build::image_install_confirmation_message_for_test("Copilot"),
        crate::commands::network::network_cleanup_confirmation_message_for_test(3),
        crate::terminal::sign_in_confirmation_message_for_test("Copilot"),
        crate::terminal::terminal_confirmation_message_for_test().to_string(),
    ];
    messages.extend(
        crate::commands::session_worktree::worktree_confirmation_messages_for_test()
            .into_iter()
            .map(str::to_string),
    );

    for message in messages {
        assert!(!message.contains("Run ID:"));
        assert!(!message.contains("Action:"));
        assert!(!message.contains("Status:"));
        assert!(!message.contains("Branch:"));
        assert!(!message.contains("Image tag:"));
        assert!(!message.contains("Container:"));
        assert!(!message.contains("runhaven-"));
    }
}

#[test]
fn destructive_confirmation_copy_names_what_will_be_lost() {
    let cleanup = crate::commands::session_cleanup::cleanup_confirmation_message_for_test();
    assert!(cleanup.contains("permanently delete"));
    assert!(cleanup.contains("original project folder won’t be deleted"));

    let worktree = crate::commands::session_worktree::worktree_confirmation_messages_for_test();
    assert!(
        worktree
            .iter()
            .any(|message| message.contains("permanently delete"))
    );
    assert!(
        worktree
            .iter()
            .any(|message| message.contains("add the assistant’s changes"))
    );

    let terminal = crate::terminal::terminal_confirmation_message_for_test();
    assert!(terminal.contains("not filtered or saved"));
    assert!(terminal.contains("Passwords or codes"));
}
