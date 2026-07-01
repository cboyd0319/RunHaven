use std::path::Path;

use serde_json::Value;

use super::super::protocol::ClientRequest;
use super::super::protocol::ValidateWorkspaceResponse;
use super::RunHavenServiceError;
use super::RunHavenTuiService;

impl RunHavenTuiService {
    pub(crate) async fn handle_request(
        &self,
        request: ClientRequest,
    ) -> Result<Value, RunHavenServiceError> {
        let method = request.method().to_string();
        match request {
            ClientRequest::RunHavenAgentList { .. } => {
                serde_json::to_value(self.agent_catalog_payload()).map_err(|error| {
                    RunHavenServiceError::Backend {
                        method,
                        message: error.to_string(),
                    }
                })
            }
            ClientRequest::RunHavenActiveRuns { .. } => {
                serde_json::to_value(self.active_runs_payload()).map_err(|error| {
                    RunHavenServiceError::Backend {
                        method,
                        message: error.to_string(),
                    }
                })
            }
            ClientRequest::RunHavenDiagnostics { limit, .. } => {
                serde_json::to_value(self.diagnostics_payload(limit).map_err(|error| {
                    RunHavenServiceError::Backend {
                        method: method.clone(),
                        message: error.to_string(),
                    }
                })?)
                .map_err(|error| RunHavenServiceError::Backend {
                    method,
                    message: error.to_string(),
                })
            }
            ClientRequest::RunHavenValidateWorkspace { workspace, .. } => {
                self.validate_workspace(&workspace, &method)?;
                serde_json::to_value(ValidateWorkspaceResponse {
                    workspace: workspace.display().to_string(),
                })
                .map_err(|error| RunHavenServiceError::Backend {
                    method,
                    message: error.to_string(),
                })
            }
            ClientRequest::RunHavenRunLogSnapshot {
                run_id,
                lines,
                confirm_sensitive_output,
                ..
            } => serde_json::to_value(self.active_run_log_snapshot_data(
                &run_id,
                lines,
                confirm_sensitive_output,
                &method,
            )?)
            .map_err(|error| RunHavenServiceError::Backend {
                method,
                message: error.to_string(),
            }),
            ClientRequest::RunHavenRunDiff {
                run_id,
                confirm_sensitive_output,
                ..
            } => serde_json::to_value(self.run_diff_data(
                &run_id,
                confirm_sensitive_output,
                &method,
            )?)
            .map_err(|error| RunHavenServiceError::Backend {
                method,
                message: error.to_string(),
            }),
            ClientRequest::RunHavenRunStop {
                run_id,
                confirm_stop,
                ..
            } => serde_json::to_value(self.stop_run_data(&run_id, confirm_stop, &method)?).map_err(
                |error| RunHavenServiceError::Backend {
                    method,
                    message: error.to_string(),
                },
            ),
            ClientRequest::RunHavenRunKill {
                run_id,
                confirm_kill,
                ..
            } => serde_json::to_value(self.kill_run_data(&run_id, confirm_kill, &method)?).map_err(
                |error| RunHavenServiceError::Backend {
                    method,
                    message: error.to_string(),
                },
            ),
            ClientRequest::RunHavenRunRepair {
                run_id,
                confirm_repair,
                ..
            } => serde_json::to_value(self.repair_run_data(&run_id, confirm_repair, &method)?)
                .map_err(|error| RunHavenServiceError::Backend {
                    method,
                    message: error.to_string(),
                }),
            ClientRequest::Unsupported { method, .. } => Err(RunHavenServiceError::Unsupported {
                method: method.method().to_string(),
                family: method.family(),
            }),
            #[cfg(test)]
            ClientRequest::BackendFailureForTest { message, .. } => {
                Err(RunHavenServiceError::Backend { method, message })
            }
        }
    }

    fn validate_workspace(
        &self,
        workspace: &Path,
        method: &str,
    ) -> Result<(), RunHavenServiceError> {
        if workspace.exists() {
            return Ok(());
        }

        Err(RunHavenServiceError::Validation {
            method: method.to_string(),
            message: format!("workspace does not exist: {}", workspace.display()),
        })
    }
}
