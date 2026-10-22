use super::*;
use crate::broker::{BrokerMarket, BrokerMoney, BrokerQuantity};

fn candle(date: &str, price: &str) -> BrokerCandle {
    BrokerCandle {
        symbol: BrokerSymbol("SOXL".into()),
        market: BrokerMarket::Us,
        date: date.into(),
        open: BrokerMoney::usd(price),
        high: BrokerMoney::usd(price),
        low: BrokerMoney::usd(price),
        close: BrokerMoney::usd(price),
        volume: BrokerQuantity("1000".into()),
    }
}

fn as_of() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339("2026-03-09T13:32:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc)
}

#[test]
fn assessment_uses_prepared_lexical_tail_and_distinguishes_price_exclusions() {
    let source = vec![
        candle("2026-03-09T13:31:00Z", "100"),
        candle("2026-03-09T09:33:00-04:00", "100"),
        candle("2026-03-09T13:32:00Z", "bad"),
        candle("2026-03-09T13:33:00Z", "0"),
        candle("invalid", "100"),
    ];
    let timed = broker_candles_to_timed_ohlc(&source, 4, "1m", 0, 0, 0);
    assert_eq!(
        timed.iter().map(|c| c.time.as_str()).collect::<Vec<_>>(),
        ["2026-03-09T13:31:00Z", "invalid"]
    );
    let before = timed
        .iter()
        .map(|row| (&row.time, row.candle.close))
        .collect::<Vec<_>>();
    let assessment = lth_preview_assessment("1m", 252, source.len(), 4, &timed, true, as_of());
    let view = assessment.minute_timing.as_ref().unwrap();
    assert_eq!(
        (
            view.source_rows,
            view.selected_rows,
            view.used_rows,
            view.price_excluded_rows
        ),
        (5, 4, 2, 2)
    );
    assert_eq!((view.parsed_rows, view.invalid_rows), (1, 1));
    assert_eq!(assessment.performance_status, "sampleOnly");
    assert_eq!(assessment.timestamp_status, "unverified");
    assert_eq!(assessment.session_status, "unverified");
    assert_eq!(
        before,
        timed
            .iter()
            .map(|row| (&row.time, row.candle.close))
            .collect::<Vec<_>>()
    );
}

#[test]
fn daily_omits_timing_and_empty_minute_is_not_evaluable() {
    let daily = lth_preview_assessment("1d", 252, 10, 20, &[], true, as_of());
    assert!(daily.minute_timing.is_none());
    assert!(serde_json::to_value(daily)
        .unwrap()
        .get("minuteTiming")
        .is_none());
    let minute = lth_preview_assessment("1m", 252, 10, 20, &[], false, as_of());
    assert_eq!(minute.performance_status, "notEvaluable");
    let json = serde_json::to_value(minute).unwrap();
    assert_eq!(json["minuteTiming"]["market"], "kr");
    assert_eq!(json["minuteTiming"]["labelConvention"], "barClose");
    assert_eq!(
        json["minuteTiming"]["diagnosticVersion"],
        "lth-minute-timing-v1"
    );
    assert_eq!(json["minuteTiming"]["priceExcludedRows"], 10);
}

#[test]
fn legacy_selection_stays_at_200_and_does_not_refill_rejected_price_rows() {
    let source = (0..240)
        .map(|index| {
            candle(
                &format!("20260309{:02}{:02}00", 10 + index / 60, index % 60),
                if index == 100 { "bad" } else { "100" },
            )
        })
        .collect::<Vec<_>>();
    let timed = broker_candles_to_timed_ohlc(&source, 200, "1m", 0, 0, 0);
    let assessment = lth_preview_assessment("1m", 252, 240, 200, &timed, true, as_of());
    let view = assessment.minute_timing.unwrap();
    assert_eq!(
        (
            view.selected_rows,
            view.used_rows,
            view.price_excluded_rows,
            view.processed_rows
        ),
        (200, 199, 1, 199)
    );
    assert_eq!(timed[0].time, "20260309104000");
}

#[test]
fn shared_parser_wrapper_preserves_public_canonical_output() {
    assert_eq!(
        generic::normalize_minute_time("202603092231").as_deref(),
        Some("2026-03-09T13:31:00.000000000Z")
    );
    assert_eq!(
        generic::normalize_minute_time("2026-03-09T09:31:00.1-04:00").as_deref(),
        Some("2026-03-09T13:31:00.100000000Z")
    );
    assert!(generic::normalize_minute_time("20260309").is_none());
}

#[test]
fn timing_assessment_cannot_change_production_signals_settlement_or_input_hash() {
    use crate::trading::strategy::{LeveragedTrendHoldEntry, LeveragedTrendHoldParams};
    let timed = [100, 96, 97, 98, 99, 90]
        .iter()
        .enumerate()
        .map(|(index, price)| LeveragedTrendHoldTimedCandle {
            time: format!("2026070713{:02}00", index + 11),
            candle: OhlcCandle {
                open: price * 100,
                high: price * 100,
                low: price * 100,
                close: price * 100,
            },
        })
        .collect::<Vec<_>>();
    let params = LeveragedTrendHoldParams {
        entries: vec![LeveragedTrendHoldEntry {
            leveraged_symbol: "SOXL".into(),
            leveraged_symbol_name: "Fixture".into(),
            inverse_leveraged_symbol: "SOXS".into(),
            inverse_leveraged_symbol_name: "Fixture inverse".into(),
            base_symbols: vec!["SOXX".into()],
            base_symbol_names: Default::default(),
            base_symbol_roles: Default::default(),
            quantity: 3,
            inverse_quantity: 2,
            is_overseas: true,
        }],
        rapid_rebound_enabled: true,
        rapid_rebound_lookback_ticks: 4,
        rapid_rebound_drop_pct: 3.0,
        rapid_rebound_recovery_pct: 1.5,
        rapid_rebound_max_low_age_ticks: 3,
        no_trade_adx_below: 90.0,
        ..Default::default()
    };
    let run = || {
        let events = timed
            .iter()
            .map(|row| SimulationEvent {
                time: row.time.clone(),
                chart_time: row.time.clone(),
                open_units: row.candle.open,
                high_units: row.candle.high,
                low_units: row.candle.low,
                close_units: row.candle.close,
                signal: None,
            })
            .collect::<Vec<_>>();
        let mut session = ReplaySession::new(
            "leveraged_trend_hold",
            "SOXL",
            true,
            SimulationAssumptions::default(),
            &[],
            events.len(),
        );
        let mut index = 0;
        let mut close_processed = false;
        let signals =
            lth_preview_signals_for_interval("1m", "SOXL", params.clone(), &[], &timed, |step| {
                let event = &events[index];
                match step {
                    LeveragedTrendHoldPreviewStep::Open(_) => {
                        session.begin_bar(event);
                        close_processed = false;
                    }
                    LeveragedTrendHoldPreviewStep::Signal(signal, _) => {
                        let mut event = event.clone();
                        event.signal = Some(if signal.side == "buy" {
                            Signal::Buy {
                                symbol: "SOXL".into(),
                                quantity: signal.quantity,
                                reason: signal.reason.clone(),
                            }
                        } else {
                            Signal::Sell {
                                symbol: "SOXL".into(),
                                quantity: signal.quantity,
                                reason: signal.reason.clone(),
                            }
                        });
                        session.process_close(&event);
                        close_processed = true;
                    }
                    LeveragedTrendHoldPreviewStep::Close(_) => {
                        if !close_processed {
                            session.process_close(event);
                        }
                        index += 1;
                    }
                }
                session.position_snapshot()
            });
        let report = session.finish();
        let event_fingerprint = timed
            .iter()
            .map(|row| {
                format!(
                    "{}:{}:{}:{}:{}",
                    row.time, row.candle.open, row.candle.high, row.candle.low, row.candle.close
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        let hash = replay_input_hash(&[
            REPLAY_ENGINE_VERSION,
            "leveraged_trend_hold",
            LTH_PREVIEW_STRATEGY_VERSION,
            "SOXL",
            "1m",
            &serde_json::to_string(&params).unwrap(),
            &event_fingerprint,
        ]);
        (
            serde_json::to_value(signals).unwrap(),
            serde_json::to_value(report).unwrap(),
            hash,
        )
    };
    let before = run();
    assert!(
        !before.0.as_array().unwrap().is_empty(),
        "fixture must exercise actual LTH signals"
    );
    assert!(before.1["summary"]["filledOrderCount"].as_u64().unwrap() > 0);
    let assessment = lth_preview_assessment("1m", 0, timed.len(), 200, &timed, true, as_of());
    assert_eq!(
        assessment.minute_timing.unwrap().unfinished_rows,
        timed.len()
    );
    assert_eq!(run(), before);
    assert_eq!(REPLAY_ENGINE_VERSION, "strategy-replay-v17");
    assert_eq!(LTH_PREVIEW_STRATEGY_VERSION, "leveraged-trend-hold-v4");
}
