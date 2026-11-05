use super::super::test_support::*;
use axum::{
    body::Body,
    http::{header, StatusCode},
};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde_json::{json, Value};

fn supplied_input() -> Value {
    let mut date = NaiveDate::from_ymd_opt(2025, 9, 1).unwrap();
    let mut days = vec![];
    while days.len() < 25 {
        if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
            let prices = crate::trading::sector_portfolio::ALL_SYMBOLS
                .iter()
                .enumerate()
                .map(|(index, symbol)| {
                    let price = 100.0 + days.len() as f64 * (index as f64 + 1.0) / 10.0;
                    (
                        symbol.to_string(),
                        json!({"adjustedClose":price,"open":price,"close":price}),
                    )
                })
                .collect::<serde_json::Map<_, _>>();
            days.push(json!({"date":date.to_string(),"prices":prices}));
        }
        date += Duration::days(1);
    }
    json!({"source":{"name":"synthetic REST contract","retrievedAt":"2026-01-01T00:00:00Z",
        "synthetic":true,"adjustmentPolicy":"adjustedSignalRawExecution","calendarSource":"synthetic weekdays",
        "coverageVerified":true,"corporateActionsVerified":true},
        "expectedSessionDates":days.iter().map(|day| day["date"].clone()).collect::<Vec<_>>(),
        "history":days[..22],"evaluation":days[22..]})
}

#[tokio::test]
async fn sector_portfolio_rest_equals_ipc_without_state_or_provider_access() {
    let fixture = Fixture::new(TOKEN).await;
    let input = supplied_input();
    let parsed = serde_json::from_value(input.clone()).unwrap();
    let report = crate::commands::preview_sector_portfolio(parsed)
        .await
        .unwrap();
    let expected: Value = serde_json::from_slice(&serde_json::to_vec(&report).unwrap()).unwrap();
    let response = fixture
        .send(
            authorized("POST", "/api/strategy/sector-portfolio/preview")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(input.to_string()))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let actual = json_body(response).await;
    assert_eq!(actual, expected);
    assert_eq!(actual["portfolios"].as_array().unwrap().len(), 5);
    fixture.assert_no_effects().await;
}

#[tokio::test]
async fn sector_portfolio_missing_price_rejects_the_whole_comparison_and_large_body() {
    let fixture = Fixture::new(TOKEN).await;
    let mut input = supplied_input();
    input["evaluation"][0]["prices"]
        .as_object_mut()
        .unwrap()
        .remove("XLP");
    let response = fixture
        .send(
            authorized("POST", "/api/strategy/sector-portfolio/preview")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(input.to_string()))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error = json_body(response).await;
    assert_eq!(error["code"], "INVALID_SECTOR_PORTFOLIO_INPUT");
    assert!(error.get("portfolios").is_none());
    let response = fixture
        .send(
            authorized("POST", "/api/strategy/sector-portfolio/preview")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(" ".repeat(2 * 1024 * 1024 + 1)))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    fixture.assert_no_effects().await;
}
