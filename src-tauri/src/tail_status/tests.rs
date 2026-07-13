use super::*;

#[test]
fn image_build_registry_redacts_and_bounds_output() {
    let registry = ImageBuildStatusRegistry::default();
    registry
        .mark_building("image-build-run", "codex", "runhaven-codex")
        .expect("building");

    registry
        .append_output(
            "image-build-run",
            "step: sk-ant-api03-abcdefghij_KLmnopqrstuvwxyz1234 leaked\u{1b}[0m",
        )
        .expect("append");
    for line in 0..(MAX_OUTPUT_TAIL_LINES + 50) {
        registry
            .append_output("image-build-run", &format!("layer {line}"))
            .expect("append");
    }

    let status = registry.get("image-build-run").expect("status");
    assert_eq!(status.state, "building");
    assert_eq!(status.tag, "runhaven-codex");
    assert!(!status.output_tail.contains("sk-ant-api03"));
    assert!(!status.output_tail.contains('\u{1b}'));
    assert_eq!(
        status.output_tail.lines().count(),
        MAX_OUTPUT_TAIL_LINES,
        "tail must stay bounded"
    );

    registry.mark_succeeded("image-build-run").expect("done");
    assert_eq!(
        registry.get("image-build-run").expect("status").state,
        "succeeded"
    );
}

#[test]
fn image_build_registry_reports_missing_build() {
    let registry = ImageBuildStatusRegistry::default();
    let error = registry.get("missing-build-run").expect_err("missing");
    assert!(error.contains("not found"));
}
