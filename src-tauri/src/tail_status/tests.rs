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

#[test]
fn sign_in_registry_redacts_and_bounds_output_and_flags_browser_step() {
    let registry = AgentSignInStatusRegistry::default();
    registry
        .mark_starting("agent-sign-in-run", "codex")
        .expect("starting");
    assert_eq!(
        registry.get("agent-sign-in-run").expect("status").state,
        "starting"
    );

    registry
        .append_output(
            "agent-sign-in-run",
            "open https://example.com/device and enter ABCD-1234; token sk-ant-api03-abcdefghij_KLmnopqrstuvwxyz1234\u{1b}[0m",
        )
        .expect("append");

    let status = registry.get("agent-sign-in-run").expect("status");
    // First output flipped the state to needs_browser so the UI tells the
    // user to open the link.
    assert_eq!(status.state, "needs_browser");
    // The short-lived device code stays visible so the user can complete
    // sign-in, but a leaked token and control sequences are scrubbed.
    assert!(status.output_tail.contains("ABCD-1234"));
    assert!(!status.output_tail.contains("sk-ant-api03"));
    assert!(!status.output_tail.contains('\u{1b}'));

    // A long or noisy login cannot grow the tail without bound.
    for line in 0..(MAX_OUTPUT_TAIL_LINES + 50) {
        registry
            .append_output("agent-sign-in-run", &format!("waiting {line}"))
            .expect("append");
    }
    assert_eq!(
        registry
            .get("agent-sign-in-run")
            .expect("status")
            .output_tail
            .lines()
            .count(),
        MAX_OUTPUT_TAIL_LINES,
        "tail must stay bounded"
    );

    registry
        .mark_succeeded("agent-sign-in-run", "codex")
        .expect("done");
    let done = registry.get("agent-sign-in-run").expect("status");
    assert_eq!(done.state, "succeeded");
    assert!(done.detail.contains("signed in"));
}

#[test]
fn sign_in_registry_reports_missing_sign_in() {
    let registry = AgentSignInStatusRegistry::default();
    let error = registry.get("missing-sign-in-run").expect_err("missing");
    assert!(error.contains("not found"));
}

#[test]
fn sign_in_registry_prunes_old_entries() {
    let registry = AgentSignInStatusRegistry::default();
    for idx in 0..=MAX_SIGN_IN_STATUSES {
        let sign_in_id = format!("sign-in-prune-{idx}");
        registry
            .mark_starting(&sign_in_id, "codex")
            .expect("starting");
    }
    assert!(
        registry.get("sign-in-prune-0").is_err(),
        "oldest sign-in must be pruned"
    );
    assert!(
        registry
            .get(&format!("sign-in-prune-{MAX_SIGN_IN_STATUSES}"))
            .is_ok(),
        "newest sign-in must be retained"
    );
}
