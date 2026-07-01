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
