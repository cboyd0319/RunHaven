use runhaven_core::diagnostics::auth_status_payload;
use runhaven_core::diagnostics::read_auth_broker_log_tail_bounded;
use runhaven_core::diagnostics::read_egress_policy_log_tail_bounded;
use runhaven_core::doctor::collect_checks;
use runhaven_core::records::read_run_records_tail_bounded;
use runhaven_core::records::run_diff_text;
use runhaven_core::runtime::active::active_run_log_snapshot_payload;
use runhaven_core::runtime::active::kill_active_run;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::runtime::active::repair_active_run;
use runhaven_core::runtime::active::stop_active_run;
use runhaven_core::runtime::active::validate_log_snapshot_lines;
use runhaven_core::ui_contracts::ActiveRunListData;
use runhaven_core::ui_contracts::ActiveRunLogSnapshotData;
use runhaven_core::ui_contracts::RunControlResultData;
use runhaven_core::ui_contracts::RunDiffData;
use runhaven_core::ui_contracts::RunHavenDiagnosticsData;
use runhaven_core::ui_contracts::RunHistoryListData;

use super::DIAGNOSTICS_LOG_TAIL_BYTES;
use super::RUN_DIFF_PREVIEW_LINES;
use super::RUN_HISTORY_LOG_TAIL_BYTES;
use super::RunHavenServiceError;
use super::RunHavenTuiService;

impl RunHavenTuiService {
    pub(crate) fn active_runs_payload(&self) -> ActiveRunListData {
        ActiveRunListData::from_active_run_records(read_active_run_records())
    }

    pub(crate) fn run_history_payload(&self, limit: usize) -> anyhow::Result<RunHistoryListData> {
        Ok(RunHistoryListData::from_run_records(
            read_run_records_tail_bounded(limit, RUN_HISTORY_LOG_TAIL_BYTES)?,
        ))
    }

    pub(crate) fn diagnostics_payload(
        &self,
        limit: usize,
    ) -> anyhow::Result<RunHavenDiagnosticsData> {
        Ok(RunHavenDiagnosticsData::from_payloads(
            collect_checks(),
            auth_status_payload(),
            read_egress_policy_log_tail_bounded(limit, DIAGNOSTICS_LOG_TAIL_BYTES)?,
            read_auth_broker_log_tail_bounded(limit, DIAGNOSTICS_LOG_TAIL_BYTES)?,
        ))
    }

    pub(crate) fn active_run_log_snapshot_data(
        &self,
        run_id: &str,
        lines: u32,
        confirm_sensitive_output: bool,
        method: &str,
    ) -> Result<ActiveRunLogSnapshotData, RunHavenServiceError> {
        self.validate_sensitive_log_confirmation(confirm_sensitive_output, method)?;
        self.validate_log_snapshot_lines(lines, method)?;
        self.active_run_log_snapshot_payload(run_id, lines, method)
    }

    pub(crate) fn run_diff_data(
        &self,
        run_id: &str,
        confirm_sensitive_output: bool,
        method: &str,
    ) -> Result<RunDiffData, RunHavenServiceError> {
        self.validate_sensitive_diff_confirmation(confirm_sensitive_output, method)?;
        let text = run_diff_text(run_id).map_err(|error| RunHavenServiceError::Backend {
            method: method.to_string(),
            message: error.to_string(),
        })?;
        Ok(RunDiffData::from_run_diff_text(
            run_id,
            &text,
            RUN_DIFF_PREVIEW_LINES,
        ))
    }

    pub(crate) fn stop_run_data(
        &self,
        run_id: &str,
        confirmed: bool,
        method: &str,
    ) -> Result<RunControlResultData, RunHavenServiceError> {
        self.validate_run_control_confirmation(
            confirmed,
            method,
            "Confirm stop before stopping this run.",
        )?;
        let payload = stop_active_run(run_id).map_err(|error| RunHavenServiceError::Backend {
            method: method.to_string(),
            message: error.to_string(),
        })?;
        RunControlResultData::from_stop_payload(payload).map_err(|error| {
            RunHavenServiceError::Backend {
                method: method.to_string(),
                message: error.to_string(),
            }
        })
    }

    pub(crate) fn kill_run_data(
        &self,
        run_id: &str,
        confirmed: bool,
        method: &str,
    ) -> Result<RunControlResultData, RunHavenServiceError> {
        self.validate_run_control_confirmation(
            confirmed,
            method,
            "Confirm hard stop before killing this run.",
        )?;
        let payload = kill_active_run(run_id).map_err(|error| RunHavenServiceError::Backend {
            method: method.to_string(),
            message: error.to_string(),
        })?;
        RunControlResultData::from_kill_payload(payload).map_err(|error| {
            RunHavenServiceError::Backend {
                method: method.to_string(),
                message: error.to_string(),
            }
        })
    }

    pub(crate) fn repair_run_data(
        &self,
        run_id: &str,
        confirmed: bool,
        method: &str,
    ) -> Result<RunControlResultData, RunHavenServiceError> {
        self.validate_run_control_confirmation(
            confirmed,
            method,
            "Confirm repair before changing this active-run marker.",
        )?;
        let payload = repair_active_run(run_id).map_err(|error| RunHavenServiceError::Backend {
            method: method.to_string(),
            message: error.to_string(),
        })?;
        RunControlResultData::from_repair_payload(payload).map_err(|error| {
            RunHavenServiceError::Backend {
                method: method.to_string(),
                message: error.to_string(),
            }
        })
    }

    fn validate_log_snapshot_lines(
        &self,
        lines: u32,
        method: &str,
    ) -> Result<(), RunHavenServiceError> {
        validate_log_snapshot_lines(lines).map_err(|error| RunHavenServiceError::Validation {
            method: method.to_string(),
            message: error.to_string(),
        })
    }

    fn validate_sensitive_log_confirmation(
        &self,
        confirmed: bool,
        method: &str,
    ) -> Result<(), RunHavenServiceError> {
        if confirmed {
            return Ok(());
        }

        Err(RunHavenServiceError::Validation {
            method: method.to_string(),
            message: "Confirm raw log viewing before loading output that may contain secrets."
                .to_string(),
        })
    }

    fn validate_sensitive_diff_confirmation(
        &self,
        confirmed: bool,
        method: &str,
    ) -> Result<(), RunHavenServiceError> {
        if confirmed {
            return Ok(());
        }

        Err(RunHavenServiceError::Validation {
            method: method.to_string(),
            message: "Confirm diff viewing before loading workspace file contents.".to_string(),
        })
    }

    fn validate_run_control_confirmation(
        &self,
        confirmed: bool,
        method: &str,
        message: &'static str,
    ) -> Result<(), RunHavenServiceError> {
        if confirmed {
            return Ok(());
        }

        Err(RunHavenServiceError::Validation {
            method: method.to_string(),
            message: message.to_string(),
        })
    }

    fn active_run_log_snapshot_payload(
        &self,
        run_id: &str,
        lines: u32,
        method: &str,
    ) -> Result<ActiveRunLogSnapshotData, RunHavenServiceError> {
        let payload = active_run_log_snapshot_payload(run_id, lines).map_err(|error| {
            RunHavenServiceError::Backend {
                method: method.to_string(),
                message: error.to_string(),
            }
        })?;
        let data = ActiveRunLogSnapshotData::from_active_log_snapshot_payload(payload).map_err(
            |error| RunHavenServiceError::Backend {
                method: method.to_string(),
                message: error.to_string(),
            },
        )?;

        Ok(data)
    }
}
