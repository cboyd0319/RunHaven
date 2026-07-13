use super::*;

use std::path::PathBuf;

use crate::commands::validation::{validate_plan_request_bounds, validate_text_len};

// The plan/launch request comes from the untrusted WebView, so these bounds are
// the first trust gate. They must reject oversized input before any field
// reaches the planner.

fn valid_request() -> RunPlanRequest {
    request(PathBuf::from("/tmp/runhaven-validation"))
}

fn rejects_with(label: &str, mutate: impl FnOnce(&mut RunPlanRequest)) {
    let mut req = valid_request();
    mutate(&mut req);
    let err = validate_plan_request_bounds(&req).expect_err(label);
    assert!(
        err.contains(label),
        "expected a `{label}` bounds error, got: {err}"
    );
}

#[test]
fn plan_bounds_accept_a_normal_request() {
    assert!(validate_plan_request_bounds(&valid_request()).is_ok());
}

#[test]
fn validate_text_len_is_byte_bounded_at_the_limit() {
    assert!(validate_text_len("field", &"a".repeat(64), 64).is_ok());
    let err = validate_text_len("field", &"a".repeat(65), 64).expect_err("over limit");
    assert!(err.contains("too long"));
    assert!(err.contains("64"));
}

#[test]
fn plan_bounds_reject_overlong_scalar_fields() {
    rejects_with("agent", |r| r.agent = "a".repeat(65));
    rejects_with("workspace path", |r| r.workspace_path = "w".repeat(4097));
    rejects_with("network mode", |r| r.network_mode = "n".repeat(65));
    rejects_with("workspace scope", |r| r.workspace_scope = "s".repeat(65));
    rejects_with("auth scope", |r| r.auth_scope = "a".repeat(65));
    rejects_with("cpu limit", |r| r.cpus = "c".repeat(65));
    rejects_with("memory limit", |r| r.memory = "m".repeat(65));
    rejects_with("user", |r| r.user = "u".repeat(65));
}

#[test]
fn plan_bounds_reject_overlong_optional_fields() {
    rejects_with("session name", |r| r.session_name = Some("s".repeat(129)));
    rejects_with("image", |r| r.image = Some("i".repeat(513)));
    rejects_with("API key broker environment variable", |r| {
        r.api_key_broker_env = Some("E".repeat(129))
    });
}

#[test]
fn plan_bounds_accept_optional_fields_when_absent() {
    let mut req = valid_request();
    req.session_name = None;
    req.image = None;
    req.api_key_broker_env = None;
    assert!(validate_plan_request_bounds(&req).is_ok());
}

#[test]
fn plan_bounds_cap_list_length_and_entry_length() {
    rejects_with("provider hosts", |r| {
        r.provider_hosts = vec!["h".to_string(); 51]
    });
    rejects_with("provider hosts", |r| {
        r.provider_hosts = vec!["h".repeat(254)]
    });
    rejects_with("environment variable names", |r| {
        r.env_names = vec!["E".to_string(); 51]
    });
    rejects_with("environment variable names", |r| {
        r.env_names = vec!["E".repeat(129)]
    });
}
