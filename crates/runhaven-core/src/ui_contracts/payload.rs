use serde::{Deserialize, Serialize};

use super::{
    ActiveRunListData, ActiveRunLogSnapshotData, AgentCatalogData, LaunchPlanData,
    ManagedSessionListData, RunControlResultData, RunDiffData, RunHavenDiagnosticsData,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum RunHavenComponentPayload {
    AgentCatalog(AgentCatalogData),
    LaunchPlan(Box<LaunchPlanData>),
    ActiveRunList(ActiveRunListData),
    ActiveRunLogSnapshot(Box<ActiveRunLogSnapshotData>),
    ManagedSessionList(ManagedSessionListData),
    RunDiff(Box<RunDiffData>),
    RunControlResult(Box<RunControlResultData>),
    Diagnostics(Box<RunHavenDiagnosticsData>),
}
