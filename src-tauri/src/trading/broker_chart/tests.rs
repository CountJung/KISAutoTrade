//! Trait-only synthetic collector fixtures. No clients, authentication or sockets.
use super::*;
use chrono::Duration;
use std::sync::Mutex;

#[derive(Clone)]
enum Scenario {
    Complete,
    MissingSectorDate,
    InclusiveOverlap,
    ConflictingOverlap,
    RepeatedCursor,
    NoProgress,
    OneRowPages,
    DuplicatePageDate,
    AfterEnd,
    InvalidOhlcv(usize),
    KisEmpty,
    EmptyPage,
    KisFatal(&'static str),
    Oversized,
}

#[derive(Debug, Clone)]
struct Request {
    symbol: String,
    exchange: Option<String>,
    cursor: Option<String>,
}
struct MockQuotes {
    dates: Vec<String>,
    scenario: Scenario,
    calls: Mutex<Vec<Request>>,
}

impl MockQuotes {
    fn new(scenario: Scenario) -> Self {
        let mut date = NaiveDate::from_ymd_opt(2025, 9, 1).unwrap();
        let mut dates = Vec::new();
        while dates.len() < 45 {
            if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
                dates.push(date.to_string());
            }
            date += Duration::days(1);
        }
        Self {
            dates,
            scenario,
            calls: Mutex::default(),
        }
    }
    fn row(&self, symbol: &str, index: usize) -> BrokerChartDay {
        let close = 100.0 + index as f64;
        BrokerChartDay {
            date: self.dates[index].clone(),
            prices: BTreeMap::from([(
                symbol.into(),
                BrokerChartPrice {
                    open: close,
                    high: close + 1.0,
                    low: close - 1.0,
                    close,
                    volume: 1000.0,
                    currency: BrokerChartCurrency::Usd,
                },
            )]),
        }
    }
    async fn collect(&self, broker: BrokerChartBroker) -> Result<BrokerChartCollection, String> {
        collect_broker_chart(
            self,
            broker,
            20,
            self.dates.last().unwrap(),
            "2026-01-01T12:00:00Z",
            &[],
        )
        .await
    }
}

impl BrokerChartQuoteSource for MockQuotes {
    async fn fetch(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        _end: &str,
        cursor: Option<&str>,
    ) -> Result<RawBrokerChartPage, String> {
        self.calls.lock().unwrap().push(Request {
            symbol: symbol.into(),
            exchange: exchange.map(str::to_string),
            cursor: cursor.map(str::to_string),
        });
        if exchange.is_some() {
            if matches!(self.scenario, Scenario::KisEmpty) {
                return Err("BROKER_CHART_EMPTY_ROWS".into());
            }
            if let Scenario::KisFatal(code) = self.scenario {
                return Err(code.into());
            }
        }
        let page = cursor
            .and_then(|c| c.strip_prefix("cursor-"))
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(0);
        let (begin, end, next) = match self.scenario {
            Scenario::InclusiveOverlap
            | Scenario::ConflictingOverlap
            | Scenario::RepeatedCursor
            | Scenario::NoProgress => {
                if page == 0 {
                    (23, 45, Some("cursor-1".into()))
                } else if matches!(self.scenario, Scenario::NoProgress) {
                    (23, 45, Some("cursor-2".into()))
                } else {
                    (
                        2,
                        24,
                        Some(
                            if matches!(self.scenario, Scenario::RepeatedCursor) {
                                "cursor-1"
                            } else {
                                "cursor-2"
                            }
                            .into(),
                        ),
                    )
                }
            }
            Scenario::OneRowPages => (44 - page, 45 - page, Some(format!("cursor-{}", page + 1))),
            _ => (0, 45, None),
        };
        let mut rows: Vec<_> = (begin..end)
            .rev()
            .map(|index| self.row(symbol, index))
            .collect();
        match self.scenario {
            Scenario::EmptyPage => rows.clear(),
            Scenario::MissingSectorDate if symbol == "XLK" => {
                rows.retain(|day| day.date != self.dates[20])
            }
            Scenario::ConflictingOverlap if page == 1 => {
                rows[0].prices.get_mut(symbol).unwrap().volume += 1.0;
            }
            Scenario::DuplicatePageDate => rows.push(rows[0].clone()),
            Scenario::AfterEnd => rows[0].date = "2027-01-01".into(),
            Scenario::InvalidOhlcv(case) => {
                let price = rows[0].prices.get_mut(symbol).unwrap();
                match case {
                    0 => price.open = 0.0,
                    1 => price.high = 1.0,
                    2 => price.low = 1e6,
                    3 => price.close = f64::NAN,
                    4 => price.volume = -1.0,
                    _ => price.volume = 1e16,
                }
            }
            Scenario::Oversized => rows = (0..501).map(|_| self.row(symbol, 44)).collect(),
            _ => {}
        }
        Ok(RawBrokerChartPage {
            rows,
            next_cursor: next,
            response_source: None,
        })
    }
}

#[tokio::test]
async fn broker_chart_collector_freezes_latest_42_spy_sessions_and_marks_unknown_not_verified() {
    let source = MockQuotes::new(Scenario::Complete);
    let result = source.collect(BrokerChartBroker::Toss).await.unwrap();
    assert_eq!(result.input.history.len(), 22);
    assert_eq!(result.input.evaluation.len(), 20);
    assert_eq!(result.input.expected_session_dates, source.dates[3..]);
    assert!(result
        .input
        .history
        .iter()
        .chain(&result.input.evaluation)
        .all(|d| d.prices.len() == 12));
    assert!(!result.input.acknowledged_price_only);
    assert!(!result.input.source.synthetic);
    assert_eq!(result.input.source.broker_id, BrokerChartBroker::Toss);
    assert_eq!(
        result.input.source.session_basis,
        BrokerChartSessionBasis::ObservedSpySessions
    );
    assert_eq!(
        result.input.source.adjustment,
        BrokerChartAdjustment::Unadjusted
    );
    assert!(result.input.known_corporate_actions.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|s| s.contains("부재를 검증한 것이 아닙니다")));
    assert!(result.diagnostics.iter().any(|s| s.contains("공식 달력")));
    assert_eq!(result.receipts.len(), 12);
    assert!(result.exchange_mappings.is_empty());
    assert_eq!(result.collection_hash.len(), 64);
    let calls = source.calls.lock().unwrap();
    assert_eq!(calls.len(), 12);
    assert_eq!(calls[0].symbol, "SPY");
    assert!(calls
        .iter()
        .all(|c| c.exchange.is_none() && c.cursor.is_none()));
    assert_eq!(
        validate_broker_chart_input(&result.input).unwrap_err(),
        "BROKER_CHART_PRICE_ONLY_ACK_REQUIRED"
    );
}

#[tokio::test]
async fn broker_chart_collector_never_hides_sector_missing_date_by_intersection() {
    let source = MockQuotes::new(Scenario::MissingSectorDate);
    let error = source.collect(BrokerChartBroker::Toss).await.err().unwrap();
    assert_eq!(error, "BROKER_CHART_SESSION_GAP: XLK");
    assert!(source
        .calls
        .lock()
        .unwrap()
        .iter()
        .any(|c| c.symbol == "XLK"));
}

#[tokio::test]
async fn broker_chart_collector_accepts_identical_inclusive_overlap_but_rejects_conflict() {
    let source = MockQuotes::new(Scenario::InclusiveOverlap);
    let result = source.collect(BrokerChartBroker::Toss).await.unwrap();
    assert_eq!(result.input.expected_session_dates, source.dates[3..]);
    assert_eq!(result.receipts.len(), 24);
    let conflict = MockQuotes::new(Scenario::ConflictingOverlap);
    assert_eq!(
        conflict
            .collect(BrokerChartBroker::Toss)
            .await
            .err()
            .unwrap(),
        "BROKER_CHART_CONFLICTING_OVERLAP"
    );
    assert_eq!(conflict.calls.lock().unwrap().len(), 2);
    let duplicate = MockQuotes::new(Scenario::DuplicatePageDate);
    assert_eq!(
        duplicate
            .collect(BrokerChartBroker::Toss)
            .await
            .err()
            .unwrap(),
        "BROKER_CHART_DUPLICATE_PAGE_DATE"
    );
}

#[tokio::test]
async fn broker_chart_collector_rejects_repeated_cursor_no_progress_and_exhausts_exactly_16_pages()
{
    for (scenario, expected, count) in [
        (Scenario::RepeatedCursor, "BROKER_CHART_REPEATED_CURSOR", 2),
        (Scenario::NoProgress, "BROKER_CHART_PAGE_DID_NOT_ADVANCE", 2),
        (Scenario::OneRowPages, "BROKER_CHART_PAGE_BOUND", 16),
    ] {
        let source = MockQuotes::new(scenario);
        let error = series(
            &source,
            "SPY",
            None,
            source.dates.last().unwrap(),
            100,
            None,
            &mut Vec::new(),
        )
        .await
        .err()
        .unwrap();
        assert_eq!(error, expected);
        assert_eq!(source.calls.lock().unwrap().len(), count);
    }
}

#[tokio::test]
async fn broker_chart_collector_rejects_after_end_invalid_ohlcv_and_engine_volume_overflow() {
    let source = MockQuotes::new(Scenario::AfterEnd);
    assert_eq!(
        source.collect(BrokerChartBroker::Toss).await.err().unwrap(),
        "BROKER_CHART_PAGE_IDENTITY_OR_DATE"
    );
    for case in 0..6 {
        let source = MockQuotes::new(Scenario::InvalidOhlcv(case));
        assert_eq!(
            source.collect(BrokerChartBroker::Toss).await.err().unwrap(),
            "BROKER_CHART_INVALID_OHLCV",
            "case {case}"
        );
        assert_eq!(source.calls.lock().unwrap().len(), 1);
    }
    // Unsupported currencies cannot even enter the typed source contract.
    let source = MockQuotes::new(Scenario::Complete);
    let mut row = serde_json::to_value(source.row("SPY", 44)).unwrap();
    row["prices"]["SPY"]["currency"] = "KRW".into();
    assert!(serde_json::from_value::<BrokerChartDay>(row).is_err());
}

#[tokio::test]
async fn broker_chart_collector_rejects_unfinished_future_and_invalid_window_before_fetch() {
    for (bars, end, retrieved) in [
        (0, "2025-10-31", "2026-01-01T12:00:00Z"),
        (121, "2025-10-31", "2026-01-01T12:00:00Z"),
        (20, "2026-01-01", "2026-01-02T01:00:00Z"),
        (20, "2025-10-31", "2099-01-01T12:00:00Z"),
    ] {
        let source = MockQuotes::new(Scenario::Complete);
        assert!(
            collect_broker_chart(&source, BrokerChartBroker::Toss, bars, end, retrieved, &[])
                .await
                .is_err()
        );
        assert_eq!(source.calls.lock().unwrap().len(), 0);
    }
}

#[tokio::test]
async fn broker_chart_collector_default_kis_uses_12_single_ams_routes_and_exports_master_evidence()
{
    let source = MockQuotes::new(Scenario::Complete);
    let result = source.collect(BrokerChartBroker::Kis).await.unwrap();
    assert_eq!(result.receipts.len(), 12);
    assert!(result
        .receipts
        .iter()
        .all(|r| r.exchange.as_deref() == Some("AMS")));
    assert!(result
        .input
        .source
        .calendar_evidence
        .contains("not listing-venue certification"));
    assert_eq!(source.calls.lock().unwrap().len(), 12);
    assert!(source
        .calls
        .lock()
        .unwrap()
        .iter()
        .all(|call| call.exchange.as_deref() == Some("AMS") && call.cursor.is_none()));
    assert_eq!(result.exchange_mappings.len(), 12);
    for mapping in &result.exchange_mappings {
        assert!(ALL_SYMBOLS.contains(&mapping.symbol.as_str()));
        assert_eq!(mapping.exchange, "AMS");
        assert!(mapping.evidence.contains(KIS_AMS_MASTER_URL));
        assert!(mapping.evidence.contains(KIS_AMS_MASTER_RETRIEVED));
        assert!(mapping.evidence.contains(KIS_AMS_MASTER_SHA256));
        assert!(mapping.evidence.contains("US/USD/type3"));
        assert!(mapping.evidence.len() <= 512);
    }
    assert_eq!(
        result.collection_hash,
        hash(&(&result.input, &result.receipts, &result.exchange_mappings)).unwrap()
    );
    let exported = serde_json::to_value(&result).unwrap();
    assert_eq!(exported["exchangeMappings"].as_array().unwrap().len(), 12);
}

#[tokio::test]
async fn broker_chart_collector_never_substitutes_route_after_auth_http_rate_generic_rejection_or_oversize(
) {
    for code in [
        "BROKER_CHART_AUTH_FAILED",
        "BROKER_CHART_TRANSPORT_FAILED",
        "BROKER_CHART_HTTP_FAILED",
        "BROKER_CHART_RATE_LIMITED",
        "BROKER_CHART_PROVIDER_REJECTED",
    ] {
        let source = MockQuotes::new(Scenario::KisFatal(code));
        assert_eq!(
            source.collect(BrokerChartBroker::Kis).await.err().unwrap(),
            code
        );
        assert_eq!(
            source.calls.lock().unwrap().len(),
            1,
            "must not substitute another route after {code}"
        );
        assert_eq!(
            source.calls.lock().unwrap()[0].exchange.as_deref(),
            Some("AMS")
        );
    }
    let oversized = MockQuotes::new(Scenario::Oversized);
    let error = oversized
        .collect(BrokerChartBroker::Kis)
        .await
        .err()
        .unwrap();
    assert!(error.contains("OVERSIZED"));
    assert_eq!(oversized.calls.lock().unwrap().len(), 1);
    for scenario in [Scenario::KisEmpty, Scenario::EmptyPage] {
        let empty = MockQuotes::new(scenario);
        assert_eq!(
            empty.collect(BrokerChartBroker::Kis).await.err().unwrap(),
            "BROKER_CHART_EMPTY_ROWS"
        );
        assert_eq!(empty.calls.lock().unwrap().len(), 1);
        assert_eq!(
            empty.calls.lock().unwrap()[0].exchange.as_deref(),
            Some("AMS")
        );
    }
}

#[tokio::test]
async fn broker_chart_collector_manual_route_mapping_never_falls_back_or_accepts_partial_mapping() {
    let source = MockQuotes::new(Scenario::KisEmpty);
    let mappings: Vec<_> = ALL_SYMBOLS
        .iter()
        .map(|symbol| super::super::sector_data::SectorExchangeMapping {
            symbol: (*symbol).into(),
            exchange: "NAS".into(),
            evidence: "synthetic manual route; no listing-market assertion".into(),
        })
        .collect();
    assert_eq!(
        collect_broker_chart(
            &source,
            BrokerChartBroker::Kis,
            20,
            source.dates.last().unwrap(),
            "2026-01-01T12:00:00Z",
            &mappings
        )
        .await
        .err()
        .unwrap(),
        "BROKER_CHART_EMPTY_ROWS"
    );
    assert_eq!(source.calls.lock().unwrap().len(), 1);
    let source = MockQuotes::new(Scenario::Complete);
    assert_eq!(
        collect_broker_chart(
            &source,
            BrokerChartBroker::Kis,
            20,
            source.dates.last().unwrap(),
            "2026-01-01T12:00:00Z",
            &mappings[..11]
        )
        .await
        .err()
        .unwrap(),
        "BROKER_CHART_INVALID_MAPPING"
    );
    assert!(source.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn broker_chart_collector_manual_mapping_overrides_seed_and_evidence_changes_hash() {
    let mappings: Vec<_> = ALL_SYMBOLS
        .iter()
        .enumerate()
        .map(
            |(index, symbol)| super::super::sector_data::SectorExchangeMapping {
                symbol: (*symbol).into(),
                exchange: if index % 2 == 0 { "NAS" } else { "NYS" }.into(),
                evidence: "synthetic explicit override, not listing-market verification".into(),
            },
        )
        .collect();
    let source = MockQuotes::new(Scenario::Complete);
    let result = collect_broker_chart(
        &source,
        BrokerChartBroker::Kis,
        20,
        source.dates.last().unwrap(),
        "2026-01-01T12:00:00Z",
        &mappings,
    )
    .await
    .unwrap();
    assert_eq!(source.calls.lock().unwrap().len(), 12);
    for call in source.calls.lock().unwrap().iter() {
        let mapping = mappings.iter().find(|m| m.symbol == call.symbol).unwrap();
        assert_eq!(call.exchange.as_deref(), Some(mapping.exchange.as_str()));
    }
    assert_eq!(
        serde_json::to_value(&result.exchange_mappings).unwrap(),
        serde_json::to_value(&mappings).unwrap()
    );
    let mut changed = mappings.clone();
    changed[0].evidence.push_str("; additional source note");
    let second = collect_broker_chart(
        &source,
        BrokerChartBroker::Kis,
        20,
        source.dates.last().unwrap(),
        "2026-01-01T12:00:00Z",
        &changed,
    )
    .await
    .unwrap();
    assert_eq!(
        serde_json::to_value(&result.input).unwrap(),
        serde_json::to_value(&second.input).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&result.receipts).unwrap(),
        serde_json::to_value(&second.receipts).unwrap()
    );
    assert_ne!(result.collection_hash, second.collection_hash);
}

#[tokio::test]
async fn broker_chart_collector_invalid_kis_mapping_or_any_toss_mapping_stops_before_fetch() {
    let good = kis_default_mappings();
    for case in 0..7 {
        let mut mappings = good.clone();
        match case {
            0 => mappings[0].symbol = mappings[1].symbol.clone(),
            1 => mappings[0].symbol = "QQQ".into(),
            2 => mappings[0].exchange = "ARCA".into(),
            3 => mappings[0].evidence.clear(),
            4 => mappings[0].evidence = "a".repeat(513),
            5 => mappings[0].evidence = "control\ncharacter".into(),
            _ => {
                mappings.push(mappings[0].clone());
            }
        }
        let source = MockQuotes::new(Scenario::Complete);
        assert_eq!(
            collect_broker_chart(
                &source,
                BrokerChartBroker::Kis,
                20,
                source.dates.last().unwrap(),
                "2026-01-01T12:00:00Z",
                &mappings
            )
            .await
            .err()
            .unwrap(),
            "BROKER_CHART_INVALID_MAPPING",
            "case {case}"
        );
        assert!(source.calls.lock().unwrap().is_empty());
    }
    let source = MockQuotes::new(Scenario::Complete);
    assert_eq!(
        collect_broker_chart(
            &source,
            BrokerChartBroker::Toss,
            20,
            source.dates.last().unwrap(),
            "2026-01-01T12:00:00Z",
            &good
        )
        .await
        .err()
        .unwrap(),
        "BROKER_CHART_TOSS_MAPPING_NOT_SUPPORTED"
    );
    assert!(source.calls.lock().unwrap().is_empty());
}

#[test]
fn broker_chart_provider_daily_labels_are_ny_midnight_across_dst_not_host_date() {
    assert_eq!(
        provider::toss_daily_date("2025-10-31T13:00:00+09:00").unwrap(),
        "2025-10-31"
    );
    assert_eq!(
        provider::toss_daily_date("2025-11-03T14:00:00+09:00").unwrap(),
        "2025-11-03"
    );
    assert!(provider::toss_daily_date("2025-11-03T00:00:00Z").is_err());
    assert!(provider::toss_daily_date("2025-11-03T14:00:01+09:00").is_err());
}

// Fixed before-change oracle: Complete collector output captured before adding the DTO.
// Input/receipt canonical JSON hashes were independently calculated with Python SHA256.
#[tokio::test]
async fn broker_chart_none_preserves_fixed_before_json_and_collection_hash() {
    for (broker, input_hash, receipts_hash, collection_hash) in [
        (
            BrokerChartBroker::Toss,
            "606cbce7b57d02fb4aca8360c2f89ee132df0aa9bbedfded1ad7a2492c54fa6b",
            "6467a7cba5a842948473cda6ebd7801fd1a4277e1b11222667f828ca8112443f",
            "c0e496af693303290c668cbecdf2851ff41ee7b0d155dc9810f6fa440e282cce",
        ),
        (
            BrokerChartBroker::Kis,
            "b993a9bf221576afbad3fe1c647c1d60d3267e112a742f1cb5241c06fba3c77d",
            "76d6fd7738e2a438d13e80ba8e5a2c9e4b82596b9a107613777e9d5c0f957515",
            "ce3831f21b6aa524cc919ac8af0326eff2fe3b4fed496c62c22cd4083258694e",
        ),
    ] {
        let result = MockQuotes::new(Scenario::Complete)
            .collect(broker)
            .await
            .unwrap();
        let input = serde_json::to_value(&result.input).unwrap();
        let receipts = serde_json::to_value(&result.receipts).unwrap();
        assert_eq!(hash(&input).unwrap(), input_hash);
        assert_eq!(hash(&receipts).unwrap(), receipts_hash);
        assert_eq!(result.collection_hash, collection_hash);
        assert!(receipts
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r.get("responseSource").is_none()));
    }
}

// Separate from production wiring: metadata alone must not alter economic input/results.
#[tokio::test]
async fn broker_chart_source_metadata_changes_collection_only() {
    struct WithSource(MockQuotes, &'static str);
    impl BrokerChartQuoteSource for WithSource {
        async fn fetch(
            &self,
            symbol: &str,
            exchange: Option<&str>,
            end: &str,
            cursor: Option<&str>,
        ) -> Result<RawBrokerChartPage, String> {
            let mut page = self.0.fetch(symbol, exchange, end, cursor).await?;
            page.response_source = Some(
                crate::broker::candles::CandleResponseSource {
                    sha256: self.1.repeat(64),
                    byte_length: 183,
                    collected_at: chrono::DateTime::parse_from_rfc3339("2026-01-01T12:00:00Z")
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                }
                .into(),
            );
            Ok(page)
        }
    }
    let mut collections = Vec::new();
    let mut reports = Vec::new();
    for sha in ["a", "b"] {
        let source = WithSource(MockQuotes::new(Scenario::Complete), sha);
        let mut collection = collect_broker_chart(
            &source,
            BrokerChartBroker::Toss,
            20,
            source.0.dates.last().unwrap(),
            "2026-01-01T12:00:00Z",
            &[],
        )
        .await
        .unwrap();
        let exported = serde_json::to_value(&collection).unwrap();
        assert_eq!(
            exported["receipts"][0]["responseSource"],
            serde_json::json!({
                "sha256": sha.repeat(64), "byteLength": 183, "receivedAt": "2026-01-01T12:00:00Z"
            })
        );
        collection.input.acknowledged_price_only = true;
        let report = preview_broker_chart_price_approximation(&collection.input).unwrap();
        assert!(!report.economic_return_verified && !report.dividend_return_included);
        reports.push(serde_json::to_value(report).unwrap());
        collections.push(collection);
    }
    assert_eq!(
        hash(&collections[0].input).unwrap(),
        hash(&collections[1].input).unwrap()
    );
    assert_eq!(reports[0], reports[1]);
    for (a, b) in collections[0].receipts.iter().zip(&collections[1].receipts) {
        assert_eq!(a.rows_hash, b.rows_hash);
    }
    assert_ne!(
        collections[0].collection_hash,
        collections[1].collection_hash
    );
}
