use std::path::PathBuf;

use runhaven_core::runtime::plans::AgentRunPlan;
use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::NetworkMode;
use runhaven_core::runtime::plans::default_network_mode;
use runhaven_core::ui_contracts::AgentCatalogData;
use runhaven_core::ui_contracts::AgentCatalogItemData;
use runhaven_core::ui_contracts::LaunchPlanData;

use super::protocol::UnsupportedFamily;

const DIAGNOSTICS_LOG_TAIL_BYTES: u64 = 2 * 1024 * 1024;
const RUN_HISTORY_LOG_TAIL_BYTES: u64 = 4 * 1024 * 1024;
const RUN_DIFF_PREVIEW_LINES: usize = 200;
pub(crate) const CURRENT_DIRECTORY_WORKSPACE_LABEL: &str = "Current directory";
pub(crate) const GIT_REPOSITORY_ROOT_WORKSPACE_LABEL: &str = "Git repository root";
pub(crate) const GIT_REPOSITORY_ROOT_WORKSPACE_DESCRIPTION: &str =
    "Mount the full repository instead of only the nested folder.";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct LaunchPolicySelection {
    pub(crate) network: NetworkPolicySelection,
    pub(crate) auth_scope: AuthScope,
}

impl Default for LaunchPolicySelection {
    fn default() -> Self {
        Self {
            network: NetworkPolicySelection::Default,
            auth_scope: AuthScope::Agent,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum NetworkPolicySelection {
    Default,
    Fixed(NetworkMode),
}

impl NetworkPolicySelection {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Fixed(NetworkMode::Provider) => "provider",
            Self::Fixed(NetworkMode::Internal) => "internal",
            Self::Fixed(NetworkMode::Internet) => "internet",
        }
    }

    fn resolve(self, profile: &runhaven_core::runtime::profiles::AgentProfile) -> NetworkMode {
        match self {
            Self::Default => default_network_mode(profile),
            Self::Fixed(mode) => mode,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct AgentLaunchPreview {
    pub(crate) agent: AgentCatalogItemData,
    pub(crate) plan: Result<PreparedLaunch, LaunchPreviewError>,
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedLaunch {
    pub(crate) data: LaunchPlanData,
    pub(crate) executable: AgentRunPlan,
    pub(crate) policy: LaunchPolicySelection,
}

impl PreparedLaunch {
    fn from_agent_run_plan(executable: AgentRunPlan, policy: LaunchPolicySelection) -> Self {
        let data = LaunchPlanData::from(&executable);
        Self {
            data,
            executable,
            policy,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_parts_for_tests(data: LaunchPlanData, executable: AgentRunPlan) -> Self {
        Self {
            data,
            executable,
            policy: LaunchPolicySelection::default(),
        }
    }
}

impl PartialEq for PreparedLaunch {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for PreparedLaunch {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum LaunchPreviewError {
    PlanBuildFailed { detail: String },
}

impl LaunchPreviewError {
    fn plan_build_failed(error: anyhow::Error) -> Self {
        Self::PlanBuildFailed {
            detail: error.to_string(),
        }
    }

    pub(crate) fn reason(&self) -> &'static str {
        match self {
            Self::PlanBuildFailed { .. } => "Plan could not be built.",
        }
    }

    pub(crate) fn detail(&self) -> &str {
        match self {
            Self::PlanBuildFailed { detail } => detail,
        }
    }
}

impl std::fmt::Display for LaunchPreviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.reason(), self.detail())
    }
}

#[derive(Debug)]
pub(crate) struct LaunchPreviewPayload {
    pub(crate) workspace: PathBuf,
    pub(crate) previews: Vec<AgentLaunchPreview>,
}

#[derive(Debug)]
pub(crate) struct WorkspaceLaunchPreview {
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) payload: LaunchPreviewPayload,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum RunHavenServiceError {
    Unsupported {
        method: String,
        family: UnsupportedFamily,
    },
    Validation {
        method: String,
        message: String,
    },
    Backend {
        method: String,
        message: String,
    },
}

impl std::fmt::Display for RunHavenServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported { method, .. } => {
                write!(f, "{method} is not supported by the RunHaven TUI backend")
            }
            Self::Validation { method, message } => {
                write!(f, "{method} validation error: {message}")
            }
            Self::Backend { method, message } => {
                write!(f, "{method} backend error: {message}")
            }
        }
    }
}

impl std::error::Error for RunHavenServiceError {}

#[derive(Debug, Default, Clone)]
pub(crate) struct RunHavenTuiService;

impl RunHavenTuiService {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[path = "service_launch.rs"]
mod launch;
#[path = "service_requests.rs"]
mod requests;
#[path = "service_runs.rs"]
mod runs;
#[cfg(test)]
#[path = "service_test_fixtures.rs"]
mod test_fixtures;

#[cfg(test)]
pub(crate) use test_fixtures::confirm_required_preview_for_tests;

impl RunHavenTuiService {
    pub(crate) fn agent_catalog_payload(&self) -> AgentCatalogData {
        AgentCatalogData::from_profiles(runhaven_core::runtime::profiles::profiles())
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
