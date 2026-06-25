use runhaven::active::stop_active_run;
use serde_json::Value;

use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{StopRunRequest, StopRunResponse};

#[tauri::command]
pub(crate) fn stop_run(request: StopRunRequest) -> Result<StopRunResponse, String> {
    validate_text_len("run id", &request.run_id, MAX_RUN_ID_LEN)?;
    if !request.confirm_stop {
        return Err("Confirm the stop before stopping this run.".to_string());
    }
    let payload = stop_active_run(&request.run_id).map_err(|error| error.to_string())?;
    stop_run_response(&payload)
}

fn stop_run_response(payload: &Value) -> Result<StopRunResponse, String> {
    let run_id = required_string(payload, "run_id")?;
    let container_name = required_string(payload, "container_name")?;
    let return_code = payload
        .get("return_code")
        .and_then(Value::as_i64)
        .ok_or_else(|| "stop payload is missing return_code".to_string())?;
    if return_code != 0 {
        return Err(format!(
            "could not stop run {run_id} ({container_name}); container stop exited {return_code}"
        ));
    }
    Ok(StopRunResponse {
        run_id,
        container_name,
        status: "stop-requested".to_string(),
    })
}

fn required_string(payload: &Value, name: &str) -> Result<String, String> {
    payload
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("stop payload is missing {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_run_requires_explicit_confirmation() {
        let error = stop_run(StopRunRequest {
            run_id: "a".repeat(32),
            confirm_stop: false,
        })
        .expect_err("stop without confirmation should fail");
        assert!(error.contains("Confirm the stop"));
    }

    #[test]
    fn stop_run_rejects_oversized_run_id() {
        let error = stop_run(StopRunRequest {
            run_id: "a".repeat(MAX_RUN_ID_LEN + 1),
            confirm_stop: true,
        })
        .expect_err("oversized run id should fail");
        assert!(error.contains("run id is too long"));
    }

    #[test]
    fn stop_run_response_maps_success_payload() {
        let payload = serde_json::json!({
            "run_id": "abc",
            "container_name": "runhaven-shell-run",
            "return_code": 0,
        });
        let response = stop_run_response(&payload).expect("success payload");
        assert_eq!(response.run_id, "abc");
        assert_eq!(response.container_name, "runhaven-shell-run");
        assert_eq!(response.status, "stop-requested");
    }

    #[test]
    fn stop_run_response_reports_nonzero_stop() {
        let payload = serde_json::json!({
            "run_id": "abc",
            "container_name": "runhaven-shell-run",
            "return_code": 1,
        });
        let error = stop_run_response(&payload).expect_err("nonzero stop should fail");
        assert!(error.contains("container stop exited 1"));
    }
}
