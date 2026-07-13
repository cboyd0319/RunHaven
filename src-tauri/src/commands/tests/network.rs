use crate::commands::network::{managed_network_count_for_test, prune_managed_networks_for_test};
use crate::contracts::NetworkPruneRequest;
use runhaven_core::runtime::network::NetworkPruneOutcome;

#[test]
fn prune_managed_networks_fails_closed_without_confirmation() {
    let error = prune_managed_networks_for_test(
        NetworkPruneRequest { confirm: false },
        3,
        |_| panic!("must not reach confirmation without confirm flag"),
        || panic!("must not prune without confirm flag"),
    )
    .expect_err("unconfirmed prune must fail closed");

    assert!(error.contains("Confirm the cleanup"));
}

#[test]
fn prune_managed_networks_does_not_delete_when_confirmation_denied() {
    let mut shown_count = None;
    let error = prune_managed_networks_for_test(
        NetworkPruneRequest { confirm: true },
        2,
        |count| {
            shown_count = Some(count);
            Ok(false)
        },
        || panic!("denied prune must not delete any network"),
    )
    .expect_err("denied prune must not run");

    assert_eq!(shown_count, Some(2));
    assert!(error.contains("canceled"));
}

#[test]
fn prune_managed_networks_returns_sanitized_counts() {
    let response = prune_managed_networks_for_test(
        NetworkPruneRequest { confirm: true },
        3,
        |_| Ok(true),
        || {
            Ok(NetworkPruneOutcome {
                removed: 2,
                failed: 1,
            })
        },
    )
    .expect("confirmed prune");

    assert_eq!(response.removed, 2);
    assert_eq!(response.failed, 1);
    // The response carries only counts; no field can hold a network name.
    let encoded = serde_json::to_string(&response).expect("serialize");
    assert_eq!(encoded, "{\"removed\":2,\"failed\":1}");
}

#[test]
fn network_preview_collapses_raw_list_failures_to_a_plain_error() {
    let error = managed_network_count_for_test(Err(
        "/Users/alice/private: XPC connection invalid token=secret".to_string(),
    ))
    .expect_err("failed network list");

    assert_eq!(error, "RunHaven could not list its managed networks.");
    assert!(!error.contains("/Users/"));
    assert!(!error.contains("XPC"));
    assert!(!error.contains("secret"));
}

#[test]
fn confirmed_network_cleanup_deletes_only_the_list_shown_for_confirmation() {
    let source = include_str!("../network.rs");

    assert!(
        source.contains("core_prune(&networks)"),
        "the trusted command must pass its pre-confirmation network snapshot to cleanup"
    );
    assert!(
        !source.contains("core_prune()"),
        "cleanup must not re-list after the user confirms a different snapshot"
    );
}
