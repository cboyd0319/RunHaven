use super::*;
use crate::runtime::profiles::profiles;

// RunHavenComponentPayload is an adjacently-tagged serde enum with no logic of
// its own. These tests lock the envelope contract the UI depends on: the
// {"type", "data"} shape, camelCase variant tags, and that boxed variants add
// no extra wrapping around their inner data.

#[test]
fn component_payload_wraps_data_in_typed_envelope() {
    let inner = AgentCatalogData::from_profiles(profiles());
    let payload = RunHavenComponentPayload::AgentCatalog(inner.clone());

    let encoded = serde_json::to_value(&payload).expect("serialize payload");

    assert_eq!(
        encoded.get("type").and_then(serde_json::Value::as_str),
        Some("agentCatalog")
    );
    // The content lives under "data" and matches the inner value verbatim,
    // proving the envelope adds no reshaping.
    assert_eq!(
        encoded.get("data"),
        Some(&serde_json::to_value(&inner).expect("serialize inner"))
    );

    let decoded: RunHavenComponentPayload =
        serde_json::from_value(encoded).expect("deserialize payload");
    assert_eq!(decoded, payload);
}

#[test]
fn boxed_variant_data_is_transparent() {
    let diff = RunDiffData::from_run_diff_text("run-1", "+added\n", 10);
    let payload = RunHavenComponentPayload::RunDiff(Box::new(diff.clone()));

    let encoded = serde_json::to_value(&payload).expect("serialize boxed payload");

    assert_eq!(
        encoded.get("type").and_then(serde_json::Value::as_str),
        Some("runDiff")
    );
    // The Box wrapper is invisible in the JSON: "data" is the inner struct.
    assert_eq!(
        encoded.get("data"),
        Some(&serde_json::to_value(&diff).expect("serialize inner diff"))
    );
}

#[test]
fn unknown_variant_tag_is_rejected() {
    let encoded = serde_json::json!({"type": "unknownComponent", "data": {}});
    let decoded: Result<RunHavenComponentPayload, _> = serde_json::from_value(encoded);
    assert!(decoded.is_err());
}
