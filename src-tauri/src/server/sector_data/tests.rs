use super::super::test_support::*;
use crate::trading::sector_data::tests::fixture_input;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};

const ROUTE: &str = "/api/strategy/sector-data/validate";

#[tokio::test]
async fn sector_data_rest_equals_ipc_and_does_not_access_account_or_provider() {
    let fixture = Fixture::new(TOKEN).await;
    let input = fixture_input();
    let expected = serde_json::to_value(
        crate::commands::validate_sector_data(input.clone())
            .await
            .unwrap(),
    )
    .unwrap();
    let response = fixture
        .send(
            authorized("POST", ROUTE)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&input).unwrap()))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json_body(response).await, expected);
    fixture.assert_no_effects().await;
}

#[tokio::test]
async fn sector_data_rest_rejects_auth_and_large_body_and_reports_not_ready() {
    let fixture = Fixture::new(TOKEN).await;
    let response = fixture
        .send(
            Request::builder()
                .method("POST")
                .uri(ROUTE)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let response = fixture
        .send(
            authorized("POST", ROUTE)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(" ".repeat(2 * 1024 * 1024 + 1)))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let mut input = fixture_input();
    input.verification.adjustment_verified = false;
    let response = fixture
        .send(
            authorized("POST", ROUTE)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&input).unwrap()))
                .unwrap(),
        )
        .await;
    let report = json_body(response).await;
    assert_eq!(report["ready"], false);
    assert!(report["input"].is_null());
    assert_eq!(
        report["diagnostics"][0]["code"],
        "MANUAL_VERIFICATION_REQUIRED"
    );
    // Authenticated collection and scope getter are intentionally not web routes.
    for route in [
        "/api/strategy/sector-data/collect",
        "/api/strategy/sector-data/scope",
    ] {
        let response = fixture
            .send(authorized("POST", route).body(Body::empty()).unwrap())
            .await;
        // Production SPA fallback is GET-only, so an unknown POST is 405.
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(response.headers()["x-fixture-route"], "<fallback>");
    }
    fixture.assert_no_effects().await;
}
