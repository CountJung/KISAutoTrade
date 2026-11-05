use super::test_support::*;
use axum::{
    body::Body,
    http::{header, StatusCode},
};
use serde_json::json;

#[tokio::test]
async fn production_json_query_and_path_rejections_stop_before_provider_or_mutation() {
    let fixture = Fixture::new(TOKEN).await;
    let invalid_side =
        json!({"symbol":"005930","side":"hold","order_type":"Limit","quantity":1,"price":10000});
    for (content_type,payload,status) in [
        (None,"{}".to_string(),StatusCode::UNSUPPORTED_MEDIA_TYPE),
        (Some("text/plain"),"{}".to_string(),StatusCode::UNSUPPORTED_MEDIA_TYPE),
        (Some("application/json"),"{".to_string(),StatusCode::BAD_REQUEST),
        (Some("application/json"),"{}".to_string(),StatusCode::UNPROCESSABLE_ENTITY),
        (Some("application/json"),json!({"symbol":"005930","side":"Buy","order_type":"Limit","quantity":"1","price":10000}).to_string(),StatusCode::UNPROCESSABLE_ENTITY),
        (Some("application/json"),invalid_side.to_string(),StatusCode::BAD_REQUEST),
        (Some("application/json")," ".repeat(2*1024*1024+1),StatusCode::PAYLOAD_TOO_LARGE),
    ] {
        let mut builder=authorized("POST","/api/order");
        if let Some(content_type)=content_type { builder=builder.header(header::CONTENT_TYPE,content_type); }
        let response=fixture.send(builder.body(Body::from(payload)).unwrap()).await;
        assert_eq!(response.status(),status);
    }
    let response = fixture
        .send(
            request("GET", "/api/chart/005930?count=not-a-number")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let response = fixture
        .send(
            request("GET", "/api/search/%FF")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    // Valid token and invalid body prove that the .env-writing handler is never entered.
    let response = fixture
        .send(
            authorized("POST", "/api/web-config/save")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    fixture.assert_no_effects().await;
}

#[tokio::test]
async fn path_values_decode_without_changing_the_route_segment_contract() {
    let fixture = Fixture::new(TOKEN).await;
    fixture
        .state
        .stock_list
        .write()
        .await
        .push(crate::api::rest::StockSearchItem {
            pdno: "005930".into(),
            prdt_name: "한글 fixture".into(),
            market: None,
        });
    let response = fixture
        .send(
            request("GET", "/api/search/%ED%95%9C%EA%B8%80")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(json_body(response).await[0]["pdno"], "005930");
    let response = fixture
        .send(
            authorized("POST", "/api/profiles/missing%2Ffixture/toss-accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(json_body(response).await["error"]
        .as_str()
        .unwrap()
        .contains("missing/fixture"));
    let response = fixture
        .send(
            request("GET", "/api/price/005930/extra")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.headers()["x-fixture-route"], "<fallback>");
    fixture.assert_no_effects().await;
}

#[tokio::test]
async fn trailing_json_is_rejected_by_the_framework_before_order_validation() {
    let fixture = Fixture::new(TOKEN).await;
    let payload = r#"{"symbol":"005930","side":"hold","order_type":"Limit","quantity":1,"price":10000} trailing"#;
    let response = fixture
        .send(
            authorized("POST", "/api/order")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/plain; charset=utf-8"
    );
    let text = String::from_utf8(body(response).await).unwrap();
    // The 0.7/0.8 comparison is preserved in phase logs. Final regression
    // protection must require the newer framework's rejection before the handler.
    assert!(text.contains("trailing characters"), "{text}");
    assert!(!text.contains("INVALID_SIDE"));
    eprintln!("trailing JSON rejection observed: {text}");
    fixture.assert_no_effects().await;
}
