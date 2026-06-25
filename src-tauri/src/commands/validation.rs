use crate::contracts::{LaunchRunRequest, RunPlanRequest};

pub(crate) const MAX_AGENT_NAME_LEN: usize = 64;
pub(crate) const MAX_RUN_ID_LEN: usize = 128;
const MAX_WORKSPACE_PATH_LEN: usize = 4096;
const MAX_SHORT_FIELD_LEN: usize = 64;
const MAX_SESSION_NAME_LEN: usize = 128;
const MAX_IMAGE_REF_LEN: usize = 512;
const MAX_PROVIDER_HOST_LEN: usize = 253;
const MAX_ENV_NAME_LEN: usize = 128;
const MAX_WARNING_CODE_LEN: usize = 64;
const MAX_PROVIDER_HOSTS: usize = 50;
const MAX_ENV_NAMES: usize = 50;
const MAX_CONFIRMED_WARNINGS: usize = 16;

pub(super) fn validate_launch_request_bounds(request: &LaunchRunRequest) -> Result<(), String> {
    validate_plan_request_bounds(&request.plan)?;
    validate_string_list(
        "confirmed warning codes",
        &request.confirmed_warnings,
        MAX_CONFIRMED_WARNINGS,
        MAX_WARNING_CODE_LEN,
    )
}

pub(super) fn validate_plan_request_bounds(request: &RunPlanRequest) -> Result<(), String> {
    validate_text_len("agent", &request.agent, MAX_AGENT_NAME_LEN)?;
    validate_text_len(
        "workspace path",
        &request.workspace_path,
        MAX_WORKSPACE_PATH_LEN,
    )?;
    validate_text_len("network mode", &request.network_mode, MAX_SHORT_FIELD_LEN)?;
    validate_text_len(
        "workspace scope",
        &request.workspace_scope,
        MAX_SHORT_FIELD_LEN,
    )?;
    validate_optional_text_len(
        "session name",
        request.session_name.as_deref(),
        MAX_SESSION_NAME_LEN,
    )?;
    validate_text_len("cpu limit", &request.cpus, MAX_SHORT_FIELD_LEN)?;
    validate_text_len("memory limit", &request.memory, MAX_SHORT_FIELD_LEN)?;
    validate_optional_text_len("image", request.image.as_deref(), MAX_IMAGE_REF_LEN)?;
    validate_text_len("user", &request.user, MAX_SHORT_FIELD_LEN)?;
    validate_string_list(
        "provider hosts",
        &request.provider_hosts,
        MAX_PROVIDER_HOSTS,
        MAX_PROVIDER_HOST_LEN,
    )?;
    validate_string_list(
        "environment variable names",
        &request.env_names,
        MAX_ENV_NAMES,
        MAX_ENV_NAME_LEN,
    )
}

pub(crate) fn validate_text_len(label: &str, value: &str, max_len: usize) -> Result<(), String> {
    if value.len() > max_len {
        return Err(format!("{label} is too long; maximum is {max_len} bytes"));
    }
    Ok(())
}

fn validate_optional_text_len(
    label: &str,
    value: Option<&str>,
    max_len: usize,
) -> Result<(), String> {
    if let Some(value) = value {
        validate_text_len(label, value, max_len)?;
    }
    Ok(())
}

fn validate_string_list(
    label: &str,
    values: &[String],
    max_count: usize,
    max_len: usize,
) -> Result<(), String> {
    if values.len() > max_count {
        return Err(format!(
            "{label} has too many entries; maximum is {max_count}"
        ));
    }
    for value in values {
        validate_text_len(label, value, max_len)?;
    }
    Ok(())
}
