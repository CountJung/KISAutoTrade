use super::*;
#[test]
fn toss_error_display_preserves_structured_and_fallback_literals() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Request-Id", "header-fixture".parse().unwrap());
    headers.insert("Retry-After", "2".parse().unwrap());
    let body = r#"{"error":{"requestId":"body-fixture","code":"synthetic-rejection","message":"합성 거절"}}"#;
    let error = format_toss_error(StatusCode::BAD_REQUEST, &headers, body, "synthetic order");
    assert_eq!(
        error.to_string(),
        r#"synthetic order: HTTP 400 Bad Request; code=synthetic-rejection; message=합성 거절; request_id=Some("body-fixture"); header_request_id=Some("header-fixture"); retry_after=Some("2"); data=None"#
    );
    assert_eq!(
        error.downcast_ref::<TossHttpError>().unwrap().status,
        StatusCode::BAD_REQUEST
    );
    assert!(definitively_rejected(&error.context("synthetic context")));
    let fallback = format_toss_error(
        StatusCode::SERVICE_UNAVAILABLE,
        &headers,
        "synthetic unavailable",
        "synthetic order",
    );
    assert_eq!(
        fallback.to_string(),
        r#"synthetic order: HTTP 503 Service Unavailable; request_id=Some("header-fixture"); retry_after=Some("2"); body=synthetic unavailable"#
    );
    assert!(!definitively_rejected(&fallback));
}
#[test]
fn toss_typed_rejections_keep_the_existing_status_boundary() {
    for code in [400, 401, 403, 404, 405, 422, 408, 409, 429, 500, 502, 503] {
        let error = format_toss_error(
            StatusCode::from_u16(code).unwrap(),
            &HeaderMap::new(),
            "synthetic",
            "synthetic order",
        );
        assert_eq!(
            definitively_rejected(&error),
            matches!(code, 400 | 401 | 403 | 404 | 405 | 422)
        );
    }
    assert!(!definitively_rejected(&anyhow::anyhow!(
        "HTTP 400 transport outcome unknown"
    )));
}
