use super::*;

#[test]
fn launch_run_rejects_missing_api_key_broker_env_before_starting() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = request(workspace.path().to_path_buf());
    let missing_env = missing_broker_env_name();
    plan.api_key_broker_env = Some(missing_env.clone());

    let error = launch_run_with_registries(
        LaunchRunRequest {
            plan,
            confirm_launch: true,
            confirmed_warnings: Vec::new(),
        },
        SessionInputRegistry::default(),
        SessionOutputRegistry::default(),
    )
    .expect_err("missing broker env should fail closed");

    assert!(error.contains(&missing_env));
    assert!(error.contains("not set on the host"));
}
