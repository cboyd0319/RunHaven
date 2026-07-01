use crate::commands::network::prune_managed_networks_for_test;
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
