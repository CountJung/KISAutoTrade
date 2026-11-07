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
    assert_eq!(
        before.1["phases"][0]["diagnostics"]["policy"],
        "sameLedgerObservationSplit"
    );
    assert_eq!(before.1["phases"][0]["diagnostics"]["observedBars"], 4);
    assert_eq!(before.1["phases"][1]["diagnostics"]["observedBars"], 2);
    let assessment = lth_preview_assessment("1m", 0, timed.len(), 200, &timed, true, as_of());
    assert_eq!(
        assessment.minute_timing.unwrap().unfinished_rows,
        timed.len()
    );
    assert_eq!(run(), before);
    assert_eq!(REPLAY_ENGINE_VERSION, "strategy-replay-v23");
    assert_eq!(LTH_PREVIEW_STRATEGY_VERSION, "leveraged-trend-hold-v4");
}

#[test]
fn prepared_clock_wire_and_chart_use_actual_admission_not_raw_tail() {
    let source = vec![
        candle("2026-11-27T14:31:00Z", "101"),
        candle("2026-11-27T17:59:00Z", "102"),
        candle("2026-11-27T13:00:00-05:00", "103"),
        candle("2026-11-27T18:01:00Z", "9999"),
        candle("2026-11-30T14:31:00Z", "9999"),
        candle("2029-11-27T14:31:00Z", "9999"),
        candle("2026-11-27T14:33:00Z", "bad"),
        candle("2026-11-27T14:32:00Z", "104"),
        candle("2026-11-27T09:32:00-05:00", "104"),
        candle("invalid-tail", "9999"),
    ];
    let before = serde_json::to_value(&source).unwrap();
    let cutoff = chrono::DateTime::parse_from_rfc3339("2026-11-27T18:00:00Z")
        .unwrap()
        .to_utc();
    let prepared = lth_regular::prepare(&source, &[candle("20261125", "100")], 2, cutoff).unwrap();
    let assessment = lth_regular::assessment(&source, &prepared, 2, prepared.warmup.len(), &[]);
    let json = serde_json::to_value(&assessment).unwrap();
    let timing = &json["minuteTiming"];
    let clock = &timing["executionClock"];
    assert_eq!(clock["version"], "us-regular-close-v1");
    assert_eq!(clock["tzdbVersion"], chrono_tz::IANA_TZDB_VERSION);
    assert_eq!(clock["adoptedRows"], 2);
    assert_eq!(clock["warmupRows"], 1);
    assert_eq!(clock["unknownRows"], 1);
    assert_eq!(clock["omittedExamples"], 2);
    assert_eq!(clock["days"][0]["date"], "2026-11-27");
    assert_eq!(clock["days"][0]["open"], "2026-11-27T09:30:00-05:00");
    assert_eq!(clock["days"][0]["close"], "2026-11-27T13:00:00-05:00");
    assert_eq!(clock["days"][0]["earlyClose"], true);
    assert_eq!(timing["selectedRows"], source.len());
    assert_eq!(timing["priceExcludedRows"], 1);
    for field in [
        "calendarVerified",
        "providerCoverageVerified",
        "requestedPeriodCoverageVerified",
        "actualPublicationVerified",
    ] {
        assert_eq!(timing[field], false);
    }
    let evidence = timing["executionClock"]["excluded"].as_array().unwrap();
    assert_eq!(
        evidence
            .iter()
            .map(|row| row["count"].as_u64().unwrap())
            .sum::<u64>(),
        8
    );
    assert!(evidence
        .iter()
        .any(|row| row["reason"] == "duplicateInstant" && row["count"] == 2));
    for example in clock["examples"].as_array().unwrap() {
        assert_eq!(
            example["label"],
            source[example["inputIndex"].as_u64().unwrap() as usize].date
        );
    }
    let chart = lth_regular::chart(&source, &prepared);
    assert_eq!(
        chart
            .iter()
            .map(|row| row.date.as_str())
            .collect::<Vec<_>>(),
        [source[1].date.as_str(), source[2].date.as_str()]
    );
    assert_eq!(chart[0].close, "102");
    assert_eq!(chart[1].volume, source[2].volume.0);
    assert_ne!(
        serde_json::to_value(broker_candles_to_chart(&source, 2)).unwrap(),
        serde_json::to_value(&chart).unwrap()
    );
    assert_eq!(serde_json::to_value(&source).unwrap(), before);
    assert_eq!(assessment.performance_status, "sampleOnly");
    let later = lth_regular::prepare(
        &source,
        &[candle("20261125", "100")],
        2,
        cutoff + chrono::Duration::milliseconds(10),
    )
    .unwrap();
    assert_eq!(
        lth_regular::execution_clock(&source, &later, &[]).admission_identity,
        clock["admissionIdentity"]
    );
    let earlier =
        lth_regular::prepare(&source, &[], 2, cutoff - chrono::Duration::minutes(1)).unwrap();
    assert_ne!(
        lth_regular::execution_clock(&source, &earlier, &[]).admission_identity,
        clock["admissionIdentity"]
    );
    let mut changed_calendar = later;
    changed_calendar.calendar_source = "different calendar basis";
    assert_ne!(
        lth_regular::execution_clock(&source, &changed_calendar, &[]).admission_identity,
        clock["admissionIdentity"]
    );
    assert_ne!(
        replay_input_hash(&[
            lth_regular::STRATEGY_VERSION,
            clock["admissionIdentity"].as_str().unwrap()
        ]),
        replay_input_hash(&[LTH_PREVIEW_STRATEGY_VERSION])
    );
}

#[test]
fn execution_clock_empty_unknown_and_collection_bounds_remain_explicit() {
    let source = vec![
        candle("invalid", "100"),
        candle("2029-01-02T14:31:00Z", "100"),
        candle("2026-03-10T13:31:00Z", "100"),
    ];
    let prepared = lth_regular::prepare(&source, &[], 20, as_of()).unwrap();
    let assessment = lth_regular::assessment(&source, &prepared, 20, 0, &[]);
    assert_eq!(assessment.performance_status, "notEvaluable");
    let clock = assessment.minute_timing.unwrap().execution_clock.unwrap();
    assert_eq!(clock.adopted_rows, 0);
    assert_eq!(clock.unknown_rows, 1);
    assert!(clock.days.is_empty());
    assert_eq!(clock.excluded.iter().map(|row| row.count).sum::<usize>(), 3);
    let source = (1..=20)
        .map(|day| candle(&format!("2026-03-{day:02}T14:31:00Z"), "100"))
        .collect::<Vec<_>>();
    let prepared = lth_regular::prepare(
        &source,
        &[],
        200,
        chrono::DateTime::parse_from_rfc3339("2026-03-21T20:00:00Z")
            .unwrap()
            .to_utc(),
    )
    .unwrap();
    let clock = lth_regular::execution_clock(&source, &prepared, &[]);
    assert_eq!(clock.days.len(), 8);
    assert_eq!(clock.omitted_days, clock.day_count - 8);
    let kr = lth_preview_assessment("1m", 0, 0, 20, &[], false, as_of());
    assert!(kr.minute_timing.unwrap().execution_clock.is_none());
    assert!(lth_preview_assessment("1d", 0, 0, 20, &[], true, as_of())
        .minute_timing
        .is_none());
}

#[test]
fn execution_clock_non_aligned_close_labels_keep_exact_planned_window() {
    let mut mismatches = Vec::new();
    for (label, expected_close) in [
        ("2026-01-07T09:31:30-05:00", "2026-01-07T16:00:00-05:00"),
        ("2026-11-27T09:31:30-05:00", "2026-11-27T13:00:00-05:00"),
    ] {
        let source = vec![candle(label, "100")];
        let cutoff = chrono::DateTime::parse_from_rfc3339(label)
            .unwrap()
            .to_utc();
        let prepared = lth_regular::prepare(&source, &[], 20, cutoff).unwrap();
        assert_eq!(
            prepared.timed.len(),
            1,
            "full minute is inside regular window"
        );
        let assessment = lth_regular::assessment(&source, &prepared, 20, 0, &[]);
        let timing = assessment.minute_timing.unwrap();
        assert_eq!(timing.non_minute_aligned_rows, 1);
        let clock = timing.execution_clock.unwrap();
        assert_eq!(clock.days.len(), 1);
        if clock.days[0].close != expected_close {
            mismatches.push((label, expected_close, clock.days[0].close.clone()));
        }
    }
    assert!(
        mismatches.is_empty(),
        "planned close must not sum independently truncated elapsed/to_close: {mismatches:?}"
    );
}
