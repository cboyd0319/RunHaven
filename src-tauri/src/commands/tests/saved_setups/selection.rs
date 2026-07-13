use super::*;

#[test]
fn selecting_saved_setup_carries_only_assistant_and_project_into_secure_defaults() {
    let (cache, _cache_home) = isolated_cache();
    let project = project_fixture(cache.path(), "selection-project");
    let canonical = fs::canonicalize(&project).expect("canonical project");
    let id = save_saved_setup_for_test("claude", &project).expect("save setup");

    let request = select_saved_setup_for_test(&id).expect("select saved setup");

    assert_eq!(request.agent, "claude");
    assert_eq!(request.workspace_path, canonical.display().to_string());
    assert_eq!(request.network_mode, "provider");
    assert_eq!(request.workspace_scope, "current");
    assert_eq!(request.auth_scope, "agent");
    assert_eq!(request.session_name, None);
    assert!(!request.read_only_workspace);
    assert_eq!(request.cpus, "4");
    assert_eq!(request.memory, "4g");
    assert!(request.provider_hosts.is_empty());
    assert!(request.env_names.is_empty());
    assert_eq!(request.api_key_broker_env, None);
    assert_eq!(request.image, None);
    assert!(!request.allow_sensitive_workspace);
    assert!(!request.allow_root_user);
    assert_eq!(request.user, "agent");
    assert!(request.worktree);
}
