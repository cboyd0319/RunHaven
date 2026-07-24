use std::path::PathBuf;

use runhaven_core::ui_contracts::LaunchPlanData;
use serde::{Deserialize, Serialize};

pub(crate) type RequestId = i64;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ClientRequest {
    RunHavenAgentList {
        request_id: RequestId,
    },
    RunHavenActiveRuns {
        request_id: RequestId,
    },
    RunHavenDiagnostics {
        request_id: RequestId,
        limit: usize,
    },
    RunHavenValidateWorkspace {
        request_id: RequestId,
        workspace: PathBuf,
    },
    RunHavenRunLogSnapshot {
        request_id: RequestId,
        run_id: String,
        lines: u32,
        confirm_sensitive_output: bool,
    },
    RunHavenRunDiff {
        request_id: RequestId,
        run_id: String,
        confirm_sensitive_output: bool,
    },
    RunHavenRunStop {
        request_id: RequestId,
        run_id: String,
        confirm_stop: bool,
    },
    RunHavenRunKill {
        request_id: RequestId,
        run_id: String,
        confirm_kill: bool,
    },
    RunHavenRunRepair {
        request_id: RequestId,
        run_id: String,
        confirm_repair: bool,
    },
    Unsupported {
        request_id: RequestId,
        method: UnsupportedMethod,
    },
    #[cfg(test)]
    BackendFailureForTest {
        request_id: RequestId,
        message: String,
    },
}

impl ClientRequest {
    pub(crate) fn request_id(&self) -> RequestId {
        match self {
            Self::RunHavenAgentList { request_id }
            | Self::RunHavenActiveRuns { request_id }
            | Self::RunHavenDiagnostics { request_id, .. }
            | Self::RunHavenValidateWorkspace { request_id, .. }
            | Self::RunHavenRunLogSnapshot { request_id, .. }
            | Self::RunHavenRunDiff { request_id, .. }
            | Self::RunHavenRunStop { request_id, .. }
            | Self::RunHavenRunKill { request_id, .. }
            | Self::RunHavenRunRepair { request_id, .. }
            | Self::Unsupported { request_id, .. } => *request_id,
            #[cfg(test)]
            Self::BackendFailureForTest { request_id, .. } => *request_id,
        }
    }

    pub(crate) fn method(&self) -> &'static str {
        match self {
            Self::RunHavenAgentList { .. } => "runhaven/agent/list",
            Self::RunHavenActiveRuns { .. } => "runhaven/run/active",
            Self::RunHavenDiagnostics { .. } => "runhaven/diagnostics",
            Self::RunHavenValidateWorkspace { .. } => "runhaven/workspace/validate",
            Self::RunHavenRunLogSnapshot { .. } => "runhaven/run/logSnapshot",
            Self::RunHavenRunDiff { .. } => "runhaven/run/diff",
            Self::RunHavenRunStop { .. } => "runhaven/run/stop",
            Self::RunHavenRunKill { .. } => "runhaven/run/kill",
            Self::RunHavenRunRepair { .. } => "runhaven/run/repair",
            Self::Unsupported { method, .. } => method.method(),
            #[cfg(test)]
            Self::BackendFailureForTest { .. } => "runhaven/test/backendFailure",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum UnsupportedFamily {
    Fs,
    McpServer,
    Plugin,
    Marketplace,
    App,
    Hooks,
    RemoteControl,
    Environment,
    AccountLogin,
    AccountLogout,
    Feedback,
    ExternalAgentImport,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum UnsupportedMethod {
    FsReadFile,
    McpServerList,
    PluginList,
    MarketplaceList,
    AppList,
    HooksList,
    RemoteControlStart,
    EnvironmentAdd,
    AccountLoginStart,
    AccountLogout,
    FeedbackUpload,
    ExternalAgentConfigImport,
}

impl UnsupportedMethod {
    pub(crate) const ALL: &'static [Self] = &[
        Self::FsReadFile,
        Self::McpServerList,
        Self::PluginList,
        Self::MarketplaceList,
        Self::AppList,
        Self::HooksList,
        Self::RemoteControlStart,
        Self::EnvironmentAdd,
        Self::AccountLoginStart,
        Self::AccountLogout,
        Self::FeedbackUpload,
        Self::ExternalAgentConfigImport,
    ];

    pub(crate) fn method(self) -> &'static str {
        match self {
            Self::FsReadFile => "fs/readFile",
            Self::McpServerList => "mcpServer/list",
            Self::PluginList => "plugin/list",
            Self::MarketplaceList => "marketplace/list",
            Self::AppList => "app/list",
            Self::HooksList => "hooks/list",
            Self::RemoteControlStart => "remoteControl/start",
            Self::EnvironmentAdd => "environment/add",
            Self::AccountLoginStart => "account/login/start",
            Self::AccountLogout => "account/logout",
            Self::FeedbackUpload => "feedback/upload",
            Self::ExternalAgentConfigImport => "externalAgentConfig/import",
        }
    }

    pub(crate) fn family(self) -> UnsupportedFamily {
        match self {
            Self::FsReadFile => UnsupportedFamily::Fs,
            Self::McpServerList => UnsupportedFamily::McpServer,
            Self::PluginList => UnsupportedFamily::Plugin,
            Self::MarketplaceList => UnsupportedFamily::Marketplace,
            Self::AppList => UnsupportedFamily::App,
            Self::HooksList => UnsupportedFamily::Hooks,
            Self::RemoteControlStart => UnsupportedFamily::RemoteControl,
            Self::EnvironmentAdd => UnsupportedFamily::Environment,
            Self::AccountLoginStart => UnsupportedFamily::AccountLogin,
            Self::AccountLogout => UnsupportedFamily::AccountLogout,
            Self::FeedbackUpload => UnsupportedFamily::Feedback,
            Self::ExternalAgentConfigImport => UnsupportedFamily::ExternalAgentImport,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValidateWorkspaceResponse {
    pub(crate) workspace: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum AppServerEvent {
    Lagged { skipped: usize },
    ServerNotification(ServerNotification),
    ServerRequest(ServerRequest),
    Disconnected { message: String },
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub(crate) enum ServerNotification {
    TranscriptDelta {
        text: String,
    },
    TurnCompleted {
        turn_id: String,
    },
    LaunchPrepared {
        plan_id: String,
        plan: Box<LaunchPlanData>,
    },
    Progress {
        message: String,
    },
    LogDelta {
        text: String,
    },
}

impl ServerNotification {
    pub(crate) fn requires_delivery(&self) -> bool {
        matches!(
            self,
            Self::TranscriptDelta { .. } | Self::TurnCompleted { .. } | Self::LaunchPrepared { .. }
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub(crate) enum ServerRequest {
    Confirm {
        request_id: RequestId,
        prompt: String,
    },
}

impl AppServerEvent {
    pub(crate) fn requires_delivery(&self) -> bool {
        match self {
            Self::Lagged { .. } | Self::Disconnected { .. } | Self::ServerRequest(_) => true,
            Self::ServerNotification(notification) => notification.requires_delivery(),
        }
    }
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
