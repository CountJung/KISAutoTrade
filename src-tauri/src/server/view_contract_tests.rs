use super::test_support::*;
use crate::trading::strategy::{build_strategy, StrategyConfig};
use axum::{
    body::Body,
    http::{header, StatusCode},
};
use serde_json::{json, Value};

#[tokio::test]
async fn profiles_risk_and_strategy_views_match_production_ipc_builders_exactly() {
    let fixture = Fixture::new(TOKEN).await;
    {
        let mut profiles = fixture.state.profiles.write().await;
        profiles.active_id = Some(fixture.profile.id.clone());
        profiles.profiles.push(fixture.profile.clone());
    }
    let expected = serde_json::to_value(crate::commands::profile_to_view(
        &fixture.profile,
        &Some(fixture.profile.id.clone()),
    ))
    .unwrap();
    let response = fixture
        .send(request("GET", "/api/profiles").body(Body::empty()).unwrap())
        .await;
    assert_eq!(json_body(response).await, json!([expected]));
    let expected = {
        let risk = fixture.state.risk_manager.lock().await;
        serde_json::to_value(crate::commands::build_risk_view(&risk)).unwrap()
    };
    let response = fixture
        .send(
            request("GET", "/api/risk-config")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(json_body(response).await, expected);
    let cfg = StrategyConfig::new(
        "price_condition_server_fixture",
        "fixture",
        true,
        vec!["005930".into()],
        2,
        json!({"symbols":[{"symbol":"005930","quantity":2,"buy_trigger_price":10000}]}),
    );
    fixture
        .state
        .strategy_manager
        .lock()
        .await
        .add(build_strategy(cfg));
    let cfg = fixture.state.strategy_manager.lock().await.all_configs()[0].clone();
    let expected = serde_json::to_value(
        crate::trading::build_strategy_view(&cfg, &fixture.state.stock_store).await,
    )
    .unwrap();
    let response = fixture
        .send(
            request("GET", "/api/strategies")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let actual = json_body(response).await;
    assert_eq!(actual, json!([expected]));
    assert_eq!(actual[0]["orderQuantity"], 2);
    assert!(actual[0].get("order_quantity").is_none());
    assert!(fixture.requests.lock().await.is_empty());
}

#[tokio::test]
async fn pending_view_matches_the_ipc_projection_without_losing_scope_or_partial_quantity() {
    use crate::{
        storage::order_store::{OrderRecord, OrderSide},
        trading::order::PendingOrder,
    };
    let fixture = Fixture::new(TOKEN).await;
    let scope = crate::trading::order::test_support::scope(&fixture.profile);
    let mut record = OrderRecord::new(
        "005930".into(),
        "fixture".into(),
        OrderSide::Buy,
        3,
        10000,
        "Limit".into(),
    )
    .with_broker_scope(&scope);
    record.kis_order_id = Some("fixture-pending".into());
    record.timestamp = "2026-10-29T00:00:00Z".into();
    let pending = PendingOrder {
        record,
        signal_reason: "fixture".into(),
        strategy_id: Some("fixture-owner".into()),
        signal_price: 10000,
        order_price: 10000,
        exchange: None,
        broker_scope: scope,
        filled_quantity: 1,
        filled_notional: 10000,
        confirmed_filled_quantity: 1,
        confirmed_avg_price: 10000,
        confirmed_filled_notional: Some(10000),
        application_started: false,
        application_execution_date: None,
        application_exchange_rate: None,
        application_pnl: None,
        client_order_id: None,
        provider_status: Some("partially_filled".into()),
        ownership_key: None,
        ownership_generation: None,
        strategy_position_metadata: None,
        submitted_cost_basis: None,
    };
    let expected = serde_json::to_value(crate::commands::pending_order_to_view(&pending)).unwrap();
    fixture
        .state
        .order_manager
        .lock()
        .await
        .track_pending_order("fixture-pending".into(), pending);
    let response = fixture
        .send(
            request("GET", "/api/pending-orders")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let value = json_body(response).await;
    assert_eq!(value, json!([expected]));
    assert_eq!(value[0]["remainingQuantity"], 2);
    assert_eq!(value[0]["brokerAccountId"], "1234567801");
    assert!(fixture.requests.lock().await.is_empty());
}

#[tokio::test]
async fn local_views_keep_casing_types_and_never_return_the_web_api_token() {
    let fixture = Fixture::new(TOKEN).await;
    for (uri, expected) in [
        ("/api/positions", json!([])),
        ("/api/pending-orders", json!([])),
        ("/api/profiles", json!([])),
        ("/api/refresh-interval", json!(30)),
        ("/api/exchange-rate", json!(1450.0)),
        (
            "/api/archive-config",
            json!({"retention_days":90,"max_size_mb":500}),
        ),
    ] {
        let response = fixture
            .send(request("GET", uri).body(Body::empty()).unwrap())
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(json_body(response).await, expected, "{uri}");
    }
    let response = fixture
        .send(
            request("GET", "/api/exchange-rate/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let expected =
        serde_json::to_value(fixture.state.exchange_rate_status.read().await.clone()).unwrap();
    assert_eq!(json_body(response).await, expected);
    let response = fixture
        .send(
            request("GET", "/api/web-config")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let value = json_body(response).await;
    assert_eq!(value["apiTokenConfigured"], true);
    assert_eq!(value["lanEnabled"], false);
    assert!(!value.to_string().contains(TOKEN));
    let response = fixture
        .send(
            request("GET", "/api/app-config")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let value = json_body(response).await;
    assert_eq!(value["active_broker_id"], "kis");
    assert_eq!(value["active_broker_configured"], false);
    assert!(value.get("activeBrokerId").is_none());
    fixture.assert_no_effects().await;
}

fn preview_input() -> Value {
    json!({"strategyId":"price_condition_server_preview","strategyName":"fixture","symbol":"005930","isOverseas":false,"orderQuantity":1,
        "params":{"symbols":[{"symbol":"005930","quantity":1,"buy_trigger_price":10000,"sell_trigger_price":11000}]},
        "candles":[{"date":"20261001","open":"9900","high":"9900","low":"9900","close":"9900","volume":"1"},
                   {"date":"20261002","open":"12000","high":"12000","low":"12000","close":"12000","volume":"1"}],
        "interval":"1d","dataSource":"syntheticServerContract"})
}

#[tokio::test]
async fn supplied_candle_preview_rest_matches_the_same_ipc_calculation() {
    let fixture = Fixture::new(TOKEN).await;
    let input = preview_input();
    let parsed = serde_json::from_value(input.clone()).unwrap();
    let mut expected =
        serde_json::to_value(crate::commands::preview_strategy_from_candles(parsed).unwrap())
            .unwrap();
    let response = fixture
        .send(
            authorized("POST", "/api/strategy/preview")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(input.to_string()))
                .unwrap(),
        )
        .await;
    // Compare the same JSON wire representation as IPC, including f64 parsing.
    expected = serde_json::from_slice(&serde_json::to_vec(&expected).unwrap()).unwrap();
    let mut actual = json_body(response).await;
    actual.as_object_mut().unwrap().remove("generatedAt");
    expected.as_object_mut().unwrap().remove("generatedAt");
    assert_eq!(actual, expected);
    assert!(actual["replay"]["inputHash"].is_string());
    fixture.assert_no_effects().await;
}

#[tokio::test]
async fn application_error_status_and_envelopes_remain_distinct() {
    let fixture = Fixture::new(TOKEN).await;
    let response = fixture
        .send(
            request("GET", "/api/candles/page?symbol=005930&brokerId=kis&isOverseas=false&interval=1d&count=1&expectedProfileId=missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(response).await["code"], "CONFIG_NOT_READY");
    let response = fixture
        .send(
            request("GET", "/api/stats?from=invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(json_body(response).await["error"]
        .as_str()
        .unwrap()
        .contains("from"));
    let payload = json!({"expectedProfileId":"missing","expectedBrokerId":"kis","expectedBrokerAccountId":null});
    let response = fixture
        .send(
            authorized("POST", "/api/strategies/missing-fixture")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let value = json_body(response).await;
    assert_eq!(value["code"], "STRATEGY_NOT_FOUND");
    assert!(value["error"].is_string());
    fixture.assert_no_effects().await;
}
