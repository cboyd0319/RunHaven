use super::*;

#[test]
fn launch_run_rejects_unrelated_api_key_broker_env_before_starting() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = request(workspace.path().to_path_buf());
    plan.api_key_broker_env = Some("AWS_SECRET_ACCESS_KEY".to_string());

    let error = build_launch_response(plan, "runhaven-auth-validation".to_string())
        .expect_err("unrelated broker env should fail closed");

    assert!(error.contains("OPENAI_API_KEY"));
}
