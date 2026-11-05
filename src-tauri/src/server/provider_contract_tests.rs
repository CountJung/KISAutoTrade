use super::test_support::*;
use axum::{
    body::Body,
    http::{header, StatusCode},
};
use serde_json::json;

#[tokio::test]
async fn production_kis_routes_preserve_capture_query_headers_and_response_mapping() {
    let fixture = Fixture::new(TOKEN).await;
    let response = fixture
        .send(
            request("GET", "/api/price/0005A0")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(json_body(response).await["stck_prpr"], "10000");
    let response = fixture
        .send(
            request("GET", "/api/overseas-price/NAS/SOXQ")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let value = json_body(response).await;
    assert_eq!(value["last"], "100.25");
    assert_eq!(value["exchange"], "NAS");
    assert_eq!(value["symbol"], "SOXQ");
    let response = fixture
        .send(
            request("GET", "/api/chart/0005A0?period=W&count=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let candles = json_body(response).await;
    assert_eq!(candles.as_array().unwrap().len(), 1);
    assert_eq!(candles[0]["date"], "20261002");
    let response = fixture
        .send(
            request("GET", "/api/overseas-chart/NYS/SOXQ?period=M&count=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(json_body(response).await[0]["close"], "100.25");
    let requests = fixture.requests.lock().await;
    assert_eq!(requests.len(), 4);
    for (_, headers) in requests.iter() {
        assert_eq!(
            headers[header::AUTHORIZATION],
            "Bearer synthetic-localhost-token"
        );
        assert_eq!(headers["appkey"], "fixture-key-1234567801");
    }
    let query = |index: usize| {
        reqwest::Url::parse(&format!("http://127.0.0.1{}", requests[index].0))
            .unwrap()
            .query_pairs()
            .into_owned()
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    assert_eq!(query(0)["fid_input_iscd"], "0005A0");
    assert_eq!(requests[0].1["tr_id"], "FHKST01010100");
    assert_eq!(query(1)["EXCD"], "NAS");
    assert_eq!(query(1)["SYMB"], "SOXQ");
    assert_eq!(query(2)["FID_PERIOD_DIV_CODE"], "W");
    assert_eq!(query(2)["FID_INPUT_ISCD"], "0005A0");
    assert_eq!(requests[2].1["tr_id"], "FHKST03010100");
    assert_eq!(query(3)["EXCD"], "NYS");
    assert_eq!(query(3)["SYMB"], "SOXQ");
    assert_eq!(query(3)["GUBN"], "2");
    assert_eq!(requests[3].1["tr_id"], "HHDFS76240000");
    eprintln!("loopback KIS production route wire cases checked: 4");
}

#[tokio::test]
async fn provider_http_failure_keeps_the_legacy_200_error_envelope() {
    let fixture = Fixture::new(TOKEN).await;
    let response = fixture
        .send(
            request("GET", "/api/price/FAIL")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let value = json_body(response).await;
    assert!(value["error"].as_str().unwrap().contains("502"));
    assert!(value["error"]
        .as_str()
        .unwrap()
        .contains("synthetic provider failure"));
    assert_eq!(fixture.requests.lock().await.len(), 1);
}

#[tokio::test]
async fn toss_and_control_routes_keep_safe_errors_without_provider_or_engine_start() {
    let fixture = Fixture::new(TOKEN).await;
    for uri in [
        "/api/toss-market-snapshot/SOXQ",
        "/api/toss-stock-safety/SOXQ",
        "/api/toss-market-calendar",
        "/api/toss-chart/SOXQ",
    ] {
        let response = fixture
            .send(request("GET", uri).body(Body::empty()).unwrap())
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(json_body(response).await["code"], "CONFIG_NOT_READY");
    }
    for (uri, payload) in [
        (
            "/api/toss-order-preflight",
            json!({"symbol":"SOXQ","side":"Buy","quantity":"1"}),
        ),
        ("/api/toss-open-orders", json!({})),
        (
            "/api/toss-order-modify",
            json!({"orderId":"fixture","orderType":"LIMIT"}),
        ),
        (
            "/api/toss-small-buy-verification",
            json!({"symbol":"SOXQ","expectedAccountSeq":"fixture","maxNotionalAmount":"1","confirmed":false}),
        ),
        (
            "/api/strategy/leveraged-trend-hold/preview",
            json!({"symbol":"SOXQ","params":{}}),
        ),
    ] {
        let response = fixture
            .send(
                authorized("POST", uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        assert_eq!(
            json_body(response).await["code"],
            "CONFIG_NOT_READY",
            "{uri}"
        );
    }
    for uri in ["/api/trading/start", "/api/test-discord"] {
        let response = fixture
            .send(authorized("POST", uri).body(Body::empty()).unwrap())
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let value = json_body(response).await;
        assert!(value.get("error").is_some() || value["code"] == "CONFIG_NOT_READY");
    }
    let response = fixture
        .send(
            authorized("POST", "/api/profiles/missing-fixture/toss-diagnostic")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert!(json_body(response).await["error"]
        .as_str()
        .unwrap()
        .contains("missing-fixture"));
    fixture.assert_no_effects().await;
}
