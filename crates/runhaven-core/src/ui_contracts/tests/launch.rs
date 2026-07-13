use super::*;
use crate::runtime::profiles::agent_profiles;

#[test]
fn launch_plan_contract_maps_safe_plan_fields() {
    let plan = test_plan(NetworkMode::Internal);
    let data = LaunchPlanData::from(&plan);

    assert_eq!(data.profile_name, "shell");
    assert_eq!(data.workspace_scope, "current");
    assert_eq!(data.auth_scope, "project");
    assert_eq!(data.network.mode, "internal");
    assert_eq!(data.state_volume, plan.state_volume);
    assert!(data.boundary.mounted_workspace.ends_with(" -> /workspace"));
    assert!(
        data.boundary
            .not_shared
            .iter()
            .any(|item| item == "host home folder")
    );
    assert!(
        data.command.contains("container run"),
        "command should be copyable CLI text"
    );
    assert!(!data.confirm_required);
}

#[test]
fn launch_plan_contract_marks_lower_security_plan_for_confirm() {
    let plan = test_plan(NetworkMode::Internet);
    let data = LaunchPlanData::from(&plan);

    assert_eq!(data.network.mode, "internet");
    assert!(data.confirm_required);
    assert!(
        data.safety_notes
            .iter()
            .any(|note| note.contains("Unrestricted internet egress"))
    );
}

#[test]
fn launch_plan_contract_round_trips_json() {
    let data = LaunchPlanData::from(&test_plan(NetworkMode::Internal));

    let encoded = serde_json::to_string(&data).expect("serialize");
    let decoded: LaunchPlanData = serde_json::from_str(&encoded).expect("deserialize");

    assert_eq!(decoded, data);
}

#[test]
fn agent_catalog_contract_maps_profiles() {
    let data = AgentCatalogData::from_profiles(agent_profiles());

    let codex = data
        .agents
        .iter()
        .find(|agent| agent.name == "codex")
        .expect("codex agent");
    assert_eq!(codex.sign_in, "runhaven login");
    assert_eq!(codex.default_network, "provider");
    assert!(codex.provider_host_count > 0);
}

#[test]
fn component_payload_round_trips_json() {
    let payload = RunHavenComponentPayload::LaunchPlan(Box::new(LaunchPlanData::from(&test_plan(
        NetworkMode::Internal,
    ))));

    let encoded = serde_json::to_string(&payload).expect("serialize");
    let decoded: RunHavenComponentPayload = serde_json::from_str(&encoded).expect("deserialize");

    assert_eq!(decoded, payload);
}

#[test]
fn component_payload_fixtures_decode() {
    let catalog = include_str!("../../../tests/fixtures/ui/agent-catalog.json");
    let launch_plan = include_str!("../../../tests/fixtures/ui/launch-plan-provider.json");

    let RunHavenComponentPayload::AgentCatalog(catalog) =
        serde_json::from_str::<RunHavenComponentPayload>(catalog).expect("agent catalog")
    else {
        panic!("expected agent catalog fixture");
    };
    let names = catalog
        .agents
        .iter()
        .map(|agent| agent.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["antigravity", "claude", "codex", "copilot"]);
    assert!(matches!(
        serde_json::from_str::<RunHavenComponentPayload>(launch_plan).expect("launch plan"),
        RunHavenComponentPayload::LaunchPlan(_)
    ));
}
