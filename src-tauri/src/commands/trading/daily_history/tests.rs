use super::*;
use crate::api::rest::ChartCandle;

fn ticket(overseas: bool) -> DailyHistoryTicket {
    DailyHistoryTicket {
        generation: 1,
        strategy_id: "fifty_two_week_high".into(),
        symbol: if overseas { "SOXQ" } else { "005930" }.into(),
        profile_id: "fixture-profile".into(),
        broker_id: BrokerId::Kis,
        broker_account_id: Some("fixture-account".into()),
        config_fingerprint: "fixture".into(),
        market_date: NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(),
        required_bars: 252,
    }
}
fn bar(age: i64) -> ChartCandle {
    ChartCandle {
        date: (ticket(false).market_date - chrono::Duration::days(age + 1))
            .format("%Y%m%d")
            .to_string(),
        open: "100".into(),
        high: "200".into(),
        low: "90".into(),
        close: "110".into(),
        volume: "10".into(),
    }
}
fn page(
    input: &CandlePageInput,
    candles: Vec<ChartCandle>,
    cursor: Option<String>,
) -> CandlePageView {
    CandlePageView {
        candles,
        next_cursor: cursor,
        end_reason: None,
        broker_id: input.broker_id,
        profile_id: input.expected_profile_id.clone(),
        scope_key: "fixture-scope".into(),
        symbol: input.symbol.clone(),
        interval: "D".into(),
        exchange: input.exchange.clone(),
        source: "fixture:D".into(),
        timestamp_semantics: "tradingDate".into(),
        completion_policy: "strictlyBeforeCurrentMarketPeriod".into(),
        as_of: "2026-10-19T13:30:00Z".into(),
        excluded_unfinished_bars: 0,
        excluded_invalid_bars: 0,
    }
}

#[test]
fn startup_skips_52_week_only_symbols_but_preserves_other_strategy_history() {
    use crate::trading::strategy::{build_strategy, StrategyConfig, StrategyManager};
    let mut manager = StrategyManager::new();
    manager.add(build_strategy(StrategyConfig::new(
        "fifty_two_week_high",
        "fixture",
        true,
        vec!["SOXQ".into(), "SPY".into()],
        1,
        serde_json::json!({}),
    )));
    assert!(super::super::history::legacy_history_symbols(&manager).is_empty());
    manager.add(build_strategy(StrategyConfig::new(
        "ma_cross",
        "fixture",
        true,
        vec!["SPY".into()],
        1,
        serde_json::json!({}),
    )));
    assert_eq!(
        super::super::history::legacy_history_symbols(&manager),
        ["SPY"]
    );
}

#[tokio::test]
async fn collects_252_actual_completed_highs_without_dropping_the_last_completed_bar() {
    let mut requests = Vec::new();
    let result = collect_pages(&ticket(false), |input| {
        let index = requests.len();
        requests.push(input.clone());
        // Inclusive overlap on both page boundaries.
        let start = index as i64 * 99;
        std::future::ready(Ok(page(
            &input,
            (start..start + 100).map(bar).collect(),
            Some(format!("next{index}")),
        )))
    })
    .await
    .unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(result.candles.len(), 252);
    assert_eq!(result.candles.last().unwrap().ohlc.high, 200);
    assert_eq!(result.candles.last().unwrap().ohlc.close, 110);
    assert_eq!(result.last_completed_date.as_deref(), Some("2026-10-18"));
    assert!(requests
        .iter()
        .all(|request| request.count == Some(200)
            && request.expected_profile_id == "fixture-profile"));
    assert_eq!(requests[1].cursor.as_deref(), Some("next0"));
}

#[tokio::test]
async fn empty_filtered_page_can_advance_and_full_512_live_window_is_bounded() {
    let mut ticket = ticket(false);
    ticket.required_bars = 512;
    let mut calls = 0;
    let result = collect_pages(&ticket, |input| {
        calls += 1;
        let candles = if calls == 1 {
            vec![]
        } else {
            ((calls - 2) * 100..(calls - 1) * 100)
                .map(|age| bar(age as i64))
                .collect()
        };
        std::future::ready(Ok(page(&input, candles, Some(format!("next{calls}")))))
    })
    .await
    .unwrap();
    assert_eq!(calls, 7);
    assert_eq!(result.candles.len(), 512);
    assert_eq!(result.last_completed_date.as_deref(), Some("2026-10-18"));
}

#[tokio::test]
async fn rejects_future_and_invalid_rows_without_fabricating_replacements() {
    let result = collect_pages(&ticket(false), |input| {
        let mut invalid = bar(2);
        invalid.high = "80".into();
        let mut future = bar(0);
        future.date = "20261019".into();
        std::future::ready(Ok(page(
            &input,
            vec![bar(0), bar(1), invalid, future],
            None,
        )))
    })
    .await
    .unwrap();
    assert_eq!(result.candles.len(), 2);
    assert!(result.end_reason.contains("자료 끝"));
}

#[tokio::test]
async fn limits_requests_and_stops_on_repeat_or_no_progress() {
    for repeat in [false, true] {
        let mut calls = 0;
        let result = collect_pages(&ticket(false), |input| {
            calls += 1;
            std::future::ready(Ok(page(
                &input,
                vec![bar(0)],
                Some(if repeat {
                    "same".into()
                } else {
                    calls.to_string()
                }),
            )))
        })
        .await
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(result.candles.len(), 1);
        assert!(result.end_reason.contains(if repeat {
            "커서 반복"
        } else {
            "유효 완료봉 없음"
        }));
    }
    let mut calls = 0;
    let result = collect_pages(&ticket(false), |input| {
        calls += 1;
        std::future::ready(Ok(page(&input, vec![bar(calls)], Some(calls.to_string()))))
    })
    .await
    .unwrap();
    assert_eq!(calls, MAX_PAGES as i64);
    assert_eq!(result.candles.len(), MAX_PAGES);
    assert!(result.end_reason.contains("8회"));
}

#[tokio::test]
async fn only_first_provider_failure_changes_exchange_and_later_failure_is_explicit_partial() {
    let mut requests = Vec::new();
    let result = collect_pages(&ticket(true), |input| {
        requests.push(input.exchange.clone());
        let result = match requests.len() {
            1 | 3 => Err(CmdError {
                code: "CANDLE_PAGE_FAILED".into(),
                message: "fixture provider failure".into(),
            }),
            _ => Ok(page(&input, vec![bar(0)], Some("next".into()))),
        };
        std::future::ready(result)
    })
    .await
    .unwrap();
    assert_eq!(
        requests,
        [Some("NAS".into()), Some("NYS".into()), Some("NYS".into())]
    );
    assert_eq!(result.candles[0].ohlc.high, 20_000);
    assert!(result.end_reason.contains("일부 페이지 조회 실패"));
    let mut calls = 0;
    assert!(collect_pages(&ticket(true), |_| {
        calls += 1;
        std::future::ready(Err(CmdError {
            code: "SCOPE_MISMATCH".into(),
            message: "fixture".into(),
        }))
    })
    .await
    .is_err());
    assert_eq!(calls, 1);
}

#[tokio::test]
async fn scope_or_cutoff_drift_discards_partial_history_and_oversized_pages_fail() {
    for kind in ["scope", "asOf", "date", "size"] {
        let mut calls = 0;
        let result = collect_pages(&ticket(false), |input| {
            calls += 1;
            let mut value = page(&input, vec![bar(calls)], Some(calls.to_string()));
            if calls == 2 {
                match kind {
                    "scope" => value.scope_key = "changed".into(),
                    "asOf" => value.as_of = "2026-10-19T13:31:00Z".into(),
                    "date" => value.as_of = "2026-10-20T13:30:00Z".into(),
                    _ => value.candles = vec![bar(0); 201],
                }
            }
            std::future::ready(Ok(value))
        })
        .await;
        assert!(result.is_err(), "{kind}");
        assert_eq!(calls, 2);
    }
}

#[tokio::test]
async fn collector_preserves_raw_volume_on_its_exact_completed_ohlc_row() {
    let mut t = ticket(false);
    t.strategy_id = "failed_breakout_volume".into();
    t.required_bars = 3;
    let result = collect_pages(&t, |input| {
        let mut rows = vec![bar(0), bar(1), bar(2)];
        rows[0].volume = "0".into();
        rows[0].high = "999".into();
        rows[1].volume = "invalid".into();
        rows[1].high = "888".into();
        rows[2].volume = "1,000".into();
        rows[2].high = "777".into();
        std::future::ready(Ok(page(&input, rows, None)))
    })
    .await
    .unwrap();
    assert_eq!(
        result
            .candles
            .iter()
            .map(|b| (b.ohlc.high, b.volume))
            .collect::<Vec<_>>(),
        [(777, Some(1000)), (888, None), (999, Some(0))]
    );
}
