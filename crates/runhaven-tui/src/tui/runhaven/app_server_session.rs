//! RunHaven app-server session bridge for the Codex-vendored TUI spine.
//!
//! The upstream Codex TUI keeps typed app-server calls behind an
//! `AppServerSession`. RunHaven keeps the same boundary, but routes supported
//! calls to its local in-process facade and makes unsupported calls fail closed.

use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use runhaven_core::ui_contracts::ActiveRunLogSnapshotData;
use runhaven_core::ui_contracts::AgentCatalogData;

use super::app_server_client::AppServerClient;
use super::app_server_client::AppServerRequestHandle;
use super::app_server_client::TypedRequestError;
use super::protocol::AppServerEvent;
use super::protocol::ClientRequest;
use super::protocol::RequestId;
use super::protocol::UnsupportedMethod;
use super::protocol::ValidateWorkspaceResponse;
use super::service::RunHavenTuiService;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct AppServerBootstrap {
    pub(crate) duration: Duration,
    pub(crate) workspace: PathBuf,
    pub(crate) agents: AgentCatalogData,
}

pub(crate) struct AppServerSession {
    client: AppServerClient,
    next_request_id: RequestId,
}

impl AppServerSession {
    pub(crate) fn start_in_process(service: RunHavenTuiService) -> Self {
        Self {
            client: AppServerClient::start_in_process(service),
            next_request_id: 1,
        }
    }

    pub(crate) async fn bootstrap(
        &mut self,
        workspace: PathBuf,
    ) -> Result<AppServerBootstrap, TypedRequestError> {
        let started_at = Instant::now();
        let validated = self.validate_workspace(workspace).await?;
        let workspace = PathBuf::from(validated.workspace);
        let agents = self.agent_catalog().await?;

        Ok(AppServerBootstrap {
            duration: started_at.elapsed(),
            workspace,
            agents,
        })
    }

    pub(crate) async fn agent_catalog(&mut self) -> Result<AgentCatalogData, TypedRequestError> {
        let request_id = self.alloc_request_id();
        self.client
            .request_typed(ClientRequest::RunHavenAgentList { request_id })
            .await
    }

    pub(crate) async fn validate_workspace(
        &mut self,
        workspace: PathBuf,
    ) -> Result<ValidateWorkspaceResponse, TypedRequestError> {
        let request_id = self.alloc_request_id();
        self.client
            .request_typed(ClientRequest::RunHavenValidateWorkspace {
                request_id,
                workspace,
            })
            .await
    }

    pub(crate) async fn run_log_snapshot(
        &mut self,
        run_id: String,
        lines: u32,
        confirm_sensitive_output: bool,
    ) -> Result<ActiveRunLogSnapshotData, TypedRequestError> {
        let request_id = self.alloc_request_id();
        self.client
            .request_typed(ClientRequest::RunHavenRunLogSnapshot {
                request_id,
                run_id,
                lines,
                confirm_sensitive_output,
            })
            .await
    }

    pub(crate) async fn unsupported(
        &mut self,
        method: UnsupportedMethod,
    ) -> Result<serde_json::Value, TypedRequestError> {
        let request_id = self.alloc_request_id();
        self.client
            .request_typed(ClientRequest::Unsupported { request_id, method })
            .await
    }

    pub(crate) async fn next_event(&mut self) -> Option<AppServerEvent> {
        self.client.next_event().await
    }

    pub(crate) fn request_handle(&self) -> AppServerRequestHandle {
        self.client.request_handle()
    }

    pub(crate) async fn shutdown(self) -> std::io::Result<()> {
        self.client.shutdown().await
    }

    fn alloc_request_id(&mut self) -> RequestId {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        request_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bootstrap_routes_supported_calls_to_runhaven_service() {
        let workspace = std::env::current_dir().expect("current dir");
        let mut session = AppServerSession::start_in_process(RunHavenTuiService::new());

        let bootstrap = session
            .bootstrap(workspace.clone())
            .await
            .expect("bootstrap should succeed");

        assert_eq!(bootstrap.workspace, workspace);
        assert!(!bootstrap.agents.agents.is_empty());
        assert!(bootstrap.duration < Duration::from_secs(30));
        session.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn unsupported_methods_fail_closed() {
        let mut session = AppServerSession::start_in_process(RunHavenTuiService::new());

        for unsupported_method in [
            UnsupportedMethod::FsReadFile,
            UnsupportedMethod::McpServerList,
            UnsupportedMethod::PluginList,
            UnsupportedMethod::AppList,
            UnsupportedMethod::HooksList,
            UnsupportedMethod::AccountLoginStart,
            UnsupportedMethod::EnvironmentAdd,
        ] {
            let error = session
                .unsupported(unsupported_method)
                .await
                .expect_err("unsupported request should fail");

            match error {
                TypedRequestError::Unsupported { method, family } => {
                    assert_eq!(method, unsupported_method.method());
                    assert_eq!(family, unsupported_method.family());
                }
                other => panic!("expected unsupported error, got {other:?}"),
            }
        }
        session.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn run_log_snapshot_rejects_invalid_line_count_before_backend_lookup() {
        let mut session = AppServerSession::start_in_process(RunHavenTuiService::new());

        let error = session
            .run_log_snapshot("not-a-real-run".to_string(), 0, true)
            .await
            .expect_err("invalid line count should fail validation");

        match error {
            TypedRequestError::Validation { method, message } => {
                assert_eq!(method, "runhaven/run/logSnapshot");
                assert!(message.contains("between 1 and 500"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }

        session.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn run_log_snapshot_requires_sensitive_output_confirmation_before_backend_lookup() {
        let mut session = AppServerSession::start_in_process(RunHavenTuiService::new());

        let error = session
            .run_log_snapshot("not-a-real-run".to_string(), 100, false)
            .await
            .expect_err("missing confirmation should fail validation");

        match error {
            TypedRequestError::Validation { method, message } => {
                assert_eq!(method, "runhaven/run/logSnapshot");
                assert!(message.contains("Confirm raw log viewing"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }

        session.shutdown().await.expect("shutdown");
    }
}
