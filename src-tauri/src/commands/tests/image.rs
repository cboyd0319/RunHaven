use super::*;

#[test]
fn launch_run_blocks_missing_bundled_image() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let request = request(workspace.path().to_path_buf());

    let error = image_readiness_error(
        &request,
        false,
        "missing",
        &request.agent,
        &request.agent,
        Some("runhaven image rebuild codex"),
    )
    .expect("image readiness error");

    assert!(error.contains("Image for codex is missing"));
    assert!(error.contains("runhaven image rebuild codex"));
}

#[test]
fn launch_run_allows_custom_image_without_bundled_image_check() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.image = Some("example/custom:1.0.0".to_string());

    assert!(image_readiness_error(&request, false, "missing", "codex", "codex", None).is_none());
}
