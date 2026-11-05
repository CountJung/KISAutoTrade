use super::*;
use crate::broker::{BrokerCurrency, BrokerQuantity};

fn input(interval: &str) -> CandlePageInput {
    CandlePageInput {
        symbol: "SOXQ".into(),
        broker_id: BrokerId::Toss,
        is_overseas: true,
        interval: interval.into(),
        count: Some(200),
        exchange: None,
        cursor: None,
        expected_profile_id: "fixture-profile".into(),
        expected_broker_account_id: Some("fixture-account".into()),
    }
}

fn bar(date: &str) -> BrokerCandle {
    let money = || BrokerMoney {
        amount: "100".into(),
        currency: BrokerCurrency::Usd,
    };
    BrokerCandle {
        symbol: BrokerSymbol("SOXQ".into()),
        market: BrokerMarket::Us,
        date: date.into(),
        open: money(),
        high: money(),
        low: money(),
        close: money(),
        volume: BrokerQuantity("10".into()),
    }
}

fn page(candles: Vec<BrokerCandle>) -> BrokerCandlePage {
    BrokerCandlePage {
        candles,
        next_cursor: Some(CandleCursor::Toss {
            before: "2026-10-15T00:00:00-04:00".into(),
        }),
        end_reason: None,
        timestamp_semantics: "tradingDate",
    }
}

fn utc(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .unwrap()
        .with_timezone(&Utc)
}

#[test]
fn completed_daily_pages_drop_current_and_future_bars_and_deduplicate_before_replay() {
    // UTC/KST are already Oct 16, but New York is still Oct 15.
    let as_of = utc("2026-10-16T02:00:00Z");
    let mut bad = bar("20261013");
    bad.high.amount = "90".into();
    let result = page_view(
        &input("D"),
        "fixture-profile".into(),
        "scope".into(),
        as_of,
        page(vec![
            bar("20261016"),
            bar("20261014"),
            bar("20261015"),
            bar("20261014"),
            bad,
        ]),
    )
    .unwrap();
    assert_eq!(result.candles.len(), 1);
    assert_eq!(result.candles[0].date, "20261014");
    assert_eq!(result.excluded_unfinished_bars, 2);
    assert_eq!(result.excluded_invalid_bars, 1);
    let cursor: PageCursor = serde_json::from_str(result.next_cursor.as_deref().unwrap()).unwrap();
    assert_eq!(cursor.as_of, as_of);
    assert_eq!(cursor.scope_key, "scope");
    assert_eq!(
        result.completion_policy,
        "strictlyBeforeCurrentMarketPeriod"
    );
}

#[test]
fn minute_close_cutoff_compares_instants_without_inventing_start_time_or_missing_bars() {
    let result = page_view(
        &input("1m"),
        "fixture-profile".into(),
        "scope".into(),
        utc("2026-10-16T14:00:00Z"),
        page(vec![
            bar("2026-10-16T10:00:00-04:00"),
            bar("2026-10-16T14:00:00Z"),
            bar("2026-10-16T10:01:00-04:00"),
            bar("2026-10-16T09:58:00-04:00"),
        ]),
    )
    .unwrap();
    assert_eq!(result.candles.len(), 2); // no synthetic 09:59 bar
    assert_eq!(result.candles[0].date, "2026-10-16T09:58:00-04:00");
    assert_eq!(result.excluded_unfinished_bars, 1);
    assert_eq!(result.completion_policy, "barCloseAtOrBeforeAsOf");
}

#[test]
fn current_week_and_month_are_conservatively_unconfirmed() {
    let now = utc("2026-10-16T21:00:00Z");
    assert_eq!(
        completed_before(now, true, "W"),
        NaiveDate::from_ymd_opt(2026, 10, 12).unwrap()
    );
    assert_eq!(
        completed_before(now, true, "M"),
        NaiveDate::from_ymd_opt(2026, 10, 1).unwrap()
    );
    assert_eq!(
        completed_before(utc("2026-03-09T03:30:00Z"), true, "D"),
        NaiveDate::from_ymd_opt(2026, 3, 8).unwrap()
    );
    assert_eq!(
        completed_before(utc("2026-03-09T03:30:00Z"), false, "D"),
        NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()
    );
}

#[test]
fn cursors_bind_scope_and_cutoff_and_reject_future_or_changed_contract() {
    let now = utc("2026-10-16T21:00:00Z");
    let mut input = input("D");
    let cursor = |as_of, scope: &str| {
        serde_json::to_string(&PageCursor {
            version: 1,
            scope_key: scope.into(),
            as_of,
            provider_cursor: CandleCursor::Toss {
                before: "2026-10-15T00:00:00-04:00".into(),
            },
        })
        .unwrap()
    };
    input.cursor = Some(cursor(now, "same"));
    assert!(decode_cursor(&input, "same", now).is_ok());
    assert!(decode_cursor(&input, "other-account", now).is_err());
    input.cursor = Some(cursor(now + chrono::Duration::seconds(1), "same"));
    assert!(decode_cursor(&input, "same", now).is_err());
    let invalid = |provider_cursor| {
        serde_json::to_string(&PageCursor {
            version: 1,
            scope_key: "same".into(),
            as_of: now,
            provider_cursor,
        })
        .unwrap()
    };
    input.cursor = Some(invalid(CandleCursor::Kis {
        before: "20261015".into(),
    }));
    assert_eq!(
        decode_cursor(&input, "same", now).unwrap_err().code,
        "INVALID_CURSOR"
    );
    input.cursor = Some(invalid(CandleCursor::Toss {
        before: "2026-10-17T00:00:00Z".into(),
    }));
    assert_eq!(
        decode_cursor(&input, "same", now).unwrap_err().code,
        "INVALID_CURSOR"
    );
    input.cursor = Some("x".repeat(2049));
    assert!(normalize_input(&mut input).is_err());
}

#[test]
fn profile_switch_cannot_change_broker_or_account_mid_collection() {
    let mut profile = AccountProfile::new(
        "fixture".into(),
        true,
        "fake-key".into(),
        "fake-secret".into(),
        "fixture-account".into(),
    );
    profile.id = "fixture-profile".into();
    profile.broker_id = BrokerId::Toss;
    let request = input("D");
    assert!(validate_profile(&request, &profile).is_ok());
    let baseline = scope_key(&request, &profile);
    profile.account_no = "other-account".into();
    assert!(validate_profile(&request, &profile).is_err());
    assert_ne!(scope_key(&request, &profile), baseline);
    profile.account_no = "fixture-account".into();
    profile.broker_id = BrokerId::Kis;
    assert!(validate_profile(&request, &profile).is_err());
    profile.broker_id = BrokerId::Toss;
    profile.id = "other-profile".into();
    assert!(validate_profile(&request, &profile).is_err());
}

#[test]
fn daily_timestamps_use_market_date_and_reject_digit_prefix_garbage() {
    let mut request = input("D");
    request.is_overseas = false;
    let domestic = |date| {
        let mut bar = bar(date);
        bar.market = BrokerMarket::Kr;
        for price in [&mut bar.open, &mut bar.high, &mut bar.low, &mut bar.close] {
            price.currency = BrokerCurrency::Krw;
        }
        bar
    };
    let result = page_view(
        &request,
        "p".into(),
        "s".into(),
        utc("2026-10-16T01:00:00Z"),
        page(vec![
            domestic("2026-10-14T15:00:00Z"), // Korean Oct 15 midnight
            domestic("20261014garbage"),
            domestic("2026-02-30"),
        ]),
    )
    .unwrap();
    assert_eq!(result.candles[0].date, "20261015");
    assert_eq!(result.excluded_invalid_bars, 2);
}

#[test]
fn candle_page_wire_query_preserves_camel_case_and_encoded_cursor() {
    use axum::extract::Query;
    let cursor = r#"{"before":"2026-10-15T00:00:00+09:00","token":"a+/=&한글"}"#;
    let mut url = reqwest::Url::parse("http://localhost/api/candles/page").unwrap();
    url.query_pairs_mut().extend_pairs([
        ("symbol", "005930"),
        ("brokerId", "toss"),
        ("isOverseas", "false"),
        ("interval", "D"),
        ("count", "200"),
        ("expectedProfileId", "fixture-profile"),
        ("expectedBrokerAccountId", "fixture-account"),
        ("cursor", cursor),
    ]);
    let uri = url.as_str().parse().unwrap();
    let Query(request) = Query::<CandlePageInput>::try_from_uri(&uri).unwrap();
    assert_eq!(request.cursor.as_deref(), Some(cursor));
    assert_eq!(request.count, Some(200));
    assert!(!request.is_overseas);
    assert_eq!(request.broker_id, BrokerId::Toss);
    let result = page_view(
        &input("D"),
        "p".into(),
        "s".into(),
        utc("2026-10-16T21:00:00Z"),
        page(vec![bar("20261015")]),
    )
    .unwrap();
    let json = serde_json::to_value(result).unwrap();
    assert_eq!(json["brokerId"], "toss");
    assert!(json.get("nextCursor").is_some());
    assert!(json.get("next_cursor").is_none());
    assert_eq!(
        serde_json::to_value(page_error("SCOPE_MISMATCH", "changed")).unwrap(),
        serde_json::json!({"code":"SCOPE_MISMATCH","message":"changed"})
    );
}

#[test]
fn unsupported_provider_intervals_and_oversized_pages_fail_explicitly() {
    let mut request = input("W");
    assert_eq!(
        normalize_input(&mut request).unwrap_err().code,
        "INVALID_INTERVAL"
    );
    request.broker_id = BrokerId::Kis;
    request.interval = "1m".into();
    assert_eq!(
        normalize_input(&mut request).unwrap_err().code,
        "INVALID_INTERVAL"
    );
    assert_eq!(
        page_view(
            &input("D"),
            "p".into(),
            "s".into(),
            utc("2026-10-16T21:00:00Z"),
            page(vec![bar("20261015"); 201])
        )
        .unwrap_err()
        .code,
        "CANDLE_PAGE_FAILED"
    );
}

#[test]
fn completed_provider_minute_page_runs_through_production_replay() {
    use crate::commands::{preview_strategy_from_candles, StrategyPreviewInput};
    let completed = page_view(
        &input("1m"),
        "p".into(),
        "s".into(),
        utc("2026-10-16T14:00:00Z"),
        page(vec![
            bar("2026-10-16T09:58:00-04:00"),
            bar("2026-10-16T14:00:00Z"),
            bar("2026-10-16T10:01:00-04:00"),
        ]),
    )
    .unwrap();
    assert_eq!(completed.excluded_unfinished_bars, 1);
    let result = preview_strategy_from_candles(StrategyPreviewInput {
        strategy_id: "price_condition_page_fixture".into(), strategy_name: "page fixture".into(),
        symbol: "SOXQ".into(), is_overseas: true, order_quantity: 1,
        params: serde_json::json!({"symbols":[{"symbol":"SOXQ","quantity":1,"buy_trigger_price":110.0,"sell_trigger_price":120.0,"take_profit_pct":5,"stop_loss_pct":3,"is_overseas":true}]}),
        candles: completed.candles, history_candles: vec![], warmup_count: Some(0),
        interval: Some("1m".into()), data_source: Some(completed.source), strategy_version: None,
        broker_id: Some(BrokerId::Toss), broker_account_id: Some("fixture".into()),
        assumptions: Default::default(),
    }).unwrap();
    assert_eq!(result.candles.len(), 2); // no synthetic bar for 09:59
    assert_eq!(result.preparation.evaluated_bars, Some(2));
    assert!(result.replay.deterministic);
}

#[cfg(test)]
#[path = "dependency_contract_tests.rs"]
mod dependency_contract_tests;
