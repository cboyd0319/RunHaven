use std::path::PathBuf;

use runhaven_core::ui_contracts::LaunchPlanData;
use serde::{Deserialize, Serialize};

pub(crate) type RequestId = i64;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ClientRequest {
    RunHavenAgentList {
        request_id: RequestId,
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
            | Self::RunHavenValidateWorkspace { request_id, .. }
            | Self::RunHavenRunLogSnapshot { request_id, .. }
            | Self::Unsupported { request_id, .. } => *request_id,
            #[cfg(test)]
            Self::BackendFailureForTest { request_id, .. } => *request_id,
        }
    }

    pub(crate) fn method(&self) -> &'static str {
        match self {
            Self::RunHavenAgentList { .. } => "runhaven/agent/list",
            Self::RunHavenValidateWorkspace { .. } => "runhaven/workspace/validate",
            Self::RunHavenRunLogSnapshot { .. } => "runhaven/run/logSnapshot",
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
    WindowsSandbox,
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
    WindowsSandboxStatus,
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
        Self::WindowsSandboxStatus,
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
            Self::WindowsSandboxStatus => "windowsSandbox/status",
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
            Self::WindowsSandboxStatus => UnsupportedFamily::WindowsSandbox,
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
mod tests {
    use super::*;
    use crate::tui::runhaven::service::confirm_required_preview_for_tests;

    #[test]
    fn launch_prepared_notification_serializes_display_plan_only() {
        let launch = confirm_required_preview_for_tests()
            .plan
            .expect("prepared launch");
        let notification = ServerNotification::LaunchPrepared {
            plan_id: "codex-plan".to_string(),
            plan: Box::new(launch.data),
        };

        let value = serde_json::to_value(&notification).expect("notification json");
        let plan = value
            .get("params")
            .and_then(|params| params.get("plan"))
            .expect("plan payload");

        assert_eq!(
            value.get("method").and_then(|method| method.as_str()),
            Some("launchPrepared")
        );
        assert!(plan.get("command").expect("command").is_string());
        assert!(
            plan.get("preflightCommands")
                .and_then(|commands| commands.as_array())
                .expect("preflight commands")
                .iter()
                .all(serde_json::Value::is_string)
        );
        assert!(plan.get("executable").is_none());
        assert!(plan.get("runId").is_none());
        assert!(plan.get("command").is_some());

        let round_trip: ServerNotification =
            serde_json::from_value(value).expect("round-trip notification");
        assert_eq!(round_trip, notification);

        let protocol_source = include_str!("protocol.rs");
        let prepared_launch_marker = ["Prepared", "Launch"].concat();
        let executable_plan_marker = ["Agent", "Run", "Plan"].concat();
        let display_plan_field = ["plan", ": Box<", "LaunchPlanData", ">"].concat();
        assert!(protocol_source.contains(&display_plan_field));
        assert!(!protocol_source.contains(&prepared_launch_marker));
        assert!(!protocol_source.contains(&executable_plan_marker));
    }

    #[test]
    fn run_log_snapshot_request_uses_runhaven_method() {
        let request = ClientRequest::RunHavenRunLogSnapshot {
            request_id: 42,
            run_id: "run-123".to_string(),
            lines: 100,
            confirm_sensitive_output: true,
        };

        assert_eq!(request.request_id(), 42);
        assert_eq!(request.method(), "runhaven/run/logSnapshot");
    }
}
