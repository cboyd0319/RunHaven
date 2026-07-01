use super::*;
use crate::runtime::profiles::{get_profile, profile_names, profiles};

#[test]
fn from_profiles_builds_one_item_per_profile_in_order() {
    let data = AgentCatalogData::from_profiles(profiles());

    let names: Vec<&str> = data.agents.iter().map(|item| item.name.as_str()).collect();
    assert_eq!(names, profile_names());
}

#[test]
fn from_profile_derives_labels_for_brokered_networked_agent() {
    let codex = get_profile("codex").expect("codex profile");
    let item = AgentCatalogItemData::from_profile(&codex);

    assert_eq!(item.name, "codex");
    assert_eq!(item.image, "runhaven/codex:0.1.0");
    // Codex signs in through the host and is covered by the API-key broker.
    assert_eq!(item.sign_in, "runhaven login");
    assert_eq!(item.broker, "yes");
    // Bundled provider hosts make the secure "provider" network the default.
    assert_eq!(item.default_network, "provider");
    assert_eq!(item.provider_host_count, codex.provider_hosts.len());
    assert!(item.provider_host_count > 0);
}

#[test]
fn from_profile_marks_shell_as_hostless_internet_default() {
    let shell = get_profile("shell").expect("shell profile");
    let item = AgentCatalogItemData::from_profile(&shell);

    assert_eq!(item.name, "shell");
    // The generic shell profile has no bundled hosts, so provider mode would be
    // an empty allowlist; the default falls back to internet.
    assert_eq!(item.provider_host_count, 0);
    assert_eq!(item.default_network, "internet");
    // No agent to sign in or broker for the generic shell.
    assert_eq!(item.sign_in, "n/a");
    assert_eq!(item.broker, "n/a");
}

#[test]
fn catalog_serializes_camel_case_and_round_trips() {
    let data = AgentCatalogData::from_profiles(profiles());
    let encoded = serde_json::to_value(&data).expect("serialize catalog");

    let first = encoded
        .pointer("/agents/0")
        .expect("first catalog agent object");
    // Public UI contract keys are camelCase, never snake_case.
    assert!(first.get("providerHostCount").is_some());
    assert!(first.get("defaultNetwork").is_some());
    assert!(first.get("signIn").is_some());
    assert!(first.get("provider_host_count").is_none());
    assert!(first.get("default_network").is_none());
    assert!(first.get("sign_in").is_none());

    let decoded: AgentCatalogData = serde_json::from_value(encoded).expect("deserialize catalog");
    assert_eq!(decoded, data);
}
