use super::*;
use crate::tui::bottom_pane::BottomPaneView;
use crossterm::event::KeyCode;

#[test]
fn post_run_recovery_screen_can_return_to_launch() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut view = RunHavenMvpView::new(workspace.path().to_path_buf());
    let launch = view
        .service
        .launch_preview_payload(workspace.path())
        .previews
        .into_iter()
        .find(|preview| preview.agent.name == "codex")
        .expect("codex preview")
        .plan
        .expect("codex plan");

    view.show_post_run(PostRunOutcome::from_launch(&launch, 0, None));
    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Run finished"));
    assert!(output.contains("The terminal is back in RunHaven."));

    view.handle_key_event(key(KeyCode::Enter));
    let output = render_to_text(&mut view, 120, 32);
    assert!(output.contains("Choose agent"));
}

#[test]
fn post_run_recovery_preserves_effective_workspace_and_policy() {
    let root = tempfile::tempdir().expect("root");
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested).expect("nested workspace");
    let mut view = RunHavenMvpView::new(nested.clone());
    let policy = LaunchPolicySelection {
        network: NetworkPolicySelection::Fixed(NetworkMode::Internet),
        auth_scope: AuthScope::Project,
    };
    let launch = view
        .service
        .launch_preview_payload_with_policy(root.path(), policy)
        .previews
        .into_iter()
        .find(|preview| preview.agent.name == "codex")
        .expect("codex preview")
        .plan
        .expect("codex plan");

    view.show_post_run(PostRunOutcome::from_launch(&launch, 0, None));
    assert_eq!(view.workspace, launch.executable.workspace);
    assert_eq!(view.policy, policy);

    view.handle_key_event(key(KeyCode::Enter));
    assert_eq!(view.workspace, launch.executable.workspace);
    assert_eq!(view.policy, policy);
}
