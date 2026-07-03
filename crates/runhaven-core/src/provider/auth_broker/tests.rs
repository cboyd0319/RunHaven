use super::*;

#[test]
fn sanitize_broker_request_path_strips_query_and_fragment() {
    assert_eq!(
        sanitize_broker_request_path("/v1/responses?token=secret#frag"),
        "/v1/responses"
    );
    assert_eq!(
        sanitize_broker_request_path("/v1/responses#fragment"),
        "/v1/responses"
    );
    assert_eq!(sanitize_broker_request_path("?token=secret"), "-");
}

/// Type-enforced no-leak guarantee for the held broker credential. The token is
/// held for the run as `BrokerState.api_key: SecretString` (see this module's
/// parent). This proves that formatting that exact type with `{:?}` renders
/// `[REDACTED]` and never the token, so a stray debug format of `BrokerState` or
/// this field cannot leak the credential. `SecretString` also has no `Display`
/// impl, so `{}` on the token would fail to compile; the only path to the
/// plaintext is the single audited `expose_secret()` call at header injection.
#[test]
fn held_broker_token_type_is_redacted_in_debug_output() {
    let token = "sk-ant-oat01-SECRETTOKENVALUE";
    let held: SecretString = SecretString::from(token);
    // Sanity: the value is really present behind the audited accessor.
    assert_eq!(held.expose_secret(), token);

    let debug = format!("{held:?}");
    assert!(
        !debug.contains(token) && !debug.contains("sk-ant"),
        "the held credential type must never render its plaintext in Debug, got {debug:?}"
    );
    assert!(
        debug.contains("REDACTED"),
        "Debug of the held credential type must be redacted, got {debug:?}"
    );
}
