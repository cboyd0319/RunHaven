use super::*;
use crate::commands::image_build::rebuild_agent_image_for_test;
use crate::commands::image_build_status::get_image_build_status_for_test;
use crate::contracts::{ImageBuildRequest, ImageBuildStatusRequest};
use crate::tail_status::ImageBuildStatusRegistry;

fn rebuild_request(agent: &str, confirm: bool) -> ImageBuildRequest {
    ImageBuildRequest {
        agent: agent.to_string(),
        confirm,
    }
}

#[test]
fn rebuild_agent_image_fails_closed_without_confirmation() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = ImageBuildStatusRegistry::default();

    let error = rebuild_agent_image_for_test(rebuild_request("codex", false), &registry, |_, _| {
        panic!("must not reach confirmation without confirm flag")
    })
    .expect_err("unconfirmed rebuild must fail closed");

    assert!(error.contains("Confirm the rebuild"));
    assert!(
        get_image_build_status_for_test(
            ImageBuildStatusRequest {
                build_id: "any-build".to_string(),
            },
            &registry,
        )
        .is_err(),
        "no build may be recorded when the request is unconfirmed"
    );
}

#[test]
fn rebuild_agent_image_rejects_unknown_agent() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = ImageBuildStatusRegistry::default();

    let error =
        rebuild_agent_image_for_test(rebuild_request("../etc/passwd", true), &registry, |_, _| {
            panic!("must not confirm an unknown agent")
        })
        .expect_err("unknown agent must be rejected");

    assert!(error.contains("known agent"));
}

#[test]
fn rebuild_agent_image_does_not_build_when_confirmation_denied() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = ImageBuildStatusRegistry::default();

    let mut confirmed_target = String::new();
    let error =
        rebuild_agent_image_for_test(rebuild_request("codex", true), &registry, |agent, tag| {
            confirmed_target = format!("{agent}:{tag}");
            Ok(false)
        })
        .expect_err("denied rebuild must not start");

    assert!(error.contains("canceled"));
    // The confirmation named the canonical agent and its image tag, not raw
    // WebView input, and denial left no build behind.
    assert!(confirmed_target.starts_with("codex:"));
    assert!(
        get_image_build_status_for_test(
            ImageBuildStatusRequest {
                build_id: "any-build".to_string(),
            },
            &registry,
        )
        .is_err(),
        "denied rebuild must not record a build"
    );
}
