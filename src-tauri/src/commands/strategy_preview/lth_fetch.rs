//! Candle fetch seam; legacy tuple projection and successful collection outcome.
use super::*;

pub(super) async fn fetch_leveraged_trend_hold_candles(
    adapter: &dyn BrokerAdapter,
    broker_symbol: &BrokerSymbol,
    interval: &str,
    collection: Option<lth_collection::Context>,
) -> CmdResult<(Vec<BrokerCandle>, Vec<BrokerCandle>)> {
    if let Some(context) = collection {
        return lth_collection::collect(adapter, broker_symbol, context).await;
    }
    let mut daily_candles = adapter
        .get_candles(broker_symbol, "D", "", "")
        .await
        .map_err(|e| CmdError {
            code: "TOSS_CANDLES_ERROR".into(),
            message: format!("Toss 일봉 조회 실패: {e}"),
        })?;
    daily_candles.sort_by(|a, b| a.date.cmp(&b.date));
    let simulation_candles = if interval == "1m" {
        adapter
            .get_candles(broker_symbol, "1m", "", "")
            .await
            .map_err(|e| CmdError {
                code: "TOSS_CANDLES_ERROR".into(),
                message: format!("Toss 1분봉 조회 실패: {e}"),
            })?
    } else {
        daily_candles.clone()
    };

    Ok((daily_candles, simulation_candles))
}

pub(super) async fn fetch_outcome(
    adapter: &dyn BrokerAdapter,
    broker_symbol: &BrokerSymbol,
    interval: &str,
    collection: Option<lth_collection::Context>,
) -> CmdResult<lth_collection::Outcome> {
    if let Some(context) = collection {
        return lth_collection::collect_with_summary(adapter, broker_symbol, context)
            .await
            .map_err(|lth_collection::Failure { error, summary }| {
                drop(summary);
                error
            });
    }
    // The compatibility projection remains a real production path for KR/day.
    let (daily, minute) =
        super::fetch_leveraged_trend_hold_candles(adapter, broker_symbol, interval, None).await?;
    Ok(lth_collection::Outcome {
        daily,
        minute,
        summary: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::candles::{BrokerCandlePageWithSource, CandleResponseSource};
    use crate::broker::{
        BrokerAdapterError, BrokerAdapterResult, BrokerCandlePage, BrokerMarket, BrokerPriceQuote,
        BrokerQuantity, CandlePageRequest,
    };
    use std::sync::Mutex;

    struct Fake {
        calls: Mutex<Vec<String>>,
        fail: bool,
        sources: bool,
        minute_source: bool,
        fail_minute: bool,
    }
    fn row(date: String) -> BrokerCandle {
        BrokerCandle {
            symbol: BrokerSymbol("KORU".into()),
            market: BrokerMarket::Us,
            date,
            open: BrokerMoney::usd("100"),
            high: BrokerMoney::usd("102"),
            low: BrokerMoney::usd("99"),
            close: BrokerMoney::usd("101"),
            volume: BrokerQuantity("1234".into()),
        }
    }
    fn rows(interval: &str) -> Vec<BrokerCandle> {
        if interval == "D" {
            vec![row("2026-01-06T00:00:00-05:00".into())]
        } else {
            (41..=59)
                .map(|m| row(format!("2026-01-07T15:{m}:00-05:00")))
                .chain(std::iter::once(row("2026-01-07T16:00:00-05:00".into())))
                .collect()
        }
    }
    fn instant() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-01-07T23:00:00-05:00")
            .unwrap()
            .to_utc()
    }
    fn fake(fail: bool, sources: bool) -> Fake {
        Fake {
            calls: Mutex::new(vec![]),
            fail,
            sources,
            minute_source: true,
            fail_minute: false,
        }
    }
    #[async_trait::async_trait]
    impl BrokerAdapter for Fake {
        fn broker_id(&self) -> BrokerId {
            BrokerId::Toss
        }
        async fn get_price(&self, _: &BrokerSymbol) -> BrokerAdapterResult<BrokerPriceQuote> {
            panic!("no price IO")
        }
        async fn get_candles(
            &self,
            _: &BrokerSymbol,
            interval: &str,
            _: &str,
            _: &str,
        ) -> BrokerAdapterResult<Vec<BrokerCandle>> {
            self.calls.lock().unwrap().push(interval.into());
            if self.fail || (self.fail_minute && interval == "1m") {
                return Err(BrokerAdapterError::InvalidRequest(
                    "offline original".into(),
                ));
            }
            let mut result = rows(interval);
            for row in &mut result {
                row.market = BrokerMarket::Kr;
                row.open = BrokerMoney::krw("100");
                row.high = BrokerMoney::krw("102");
                row.low = BrokerMoney::krw("99");
                row.close = BrokerMoney::krw("101");
            }
            result.reverse();
            Ok(result)
        }
        async fn get_candle_page_with_source(
            &self,
            request: &CandlePageRequest,
        ) -> BrokerAdapterResult<BrokerCandlePageWithSource> {
            assert_eq!(
                request.cursor.as_ref().unwrap(),
                &crate::broker::CandleCursor::Toss {
                    before: instant().to_rfc3339()
                }
            );
            self.calls.lock().unwrap().push(request.interval.clone());
            if self.fail {
                return Err(BrokerAdapterError::InvalidRequest(
                    "offline original".into(),
                ));
            }
            Ok(BrokerCandlePageWithSource {
                page: BrokerCandlePage {
                    candles: rows(&request.interval),
                    next_cursor: None,
                    end_reason: None,
                    timestamp_semantics: if request.interval == "D" {
                        "tradingDate"
                    } else {
                        "barClose"
                    },
                },
                source: (self.sources && (request.interval == "D" || self.minute_source)).then(
                    || CandleResponseSource {
                        sha256: "ab".repeat(32),
                        byte_length: 123,
                        collected_at: instant() + chrono::Duration::seconds(1),
                    },
                ),
            })
        }
    }
    #[tokio::test(start_paused = true)]
    async fn actual_pipeline_serializes_success_summary_camel_case_without_certification() {
        for (sources, minute_source, availability) in [
            (true, true, "complete"),
            (true, false, "partial"),
            (false, false, "unavailable"),
        ] {
            let mut fake = fake(false, sources);
            fake.minute_source = minute_source;
            let view = execute_lth_preview_with_adapter(
                lth_runtime::fixture(),
                instant(),
                Some(lth_collection::Context::new(instant(), 20)),
                &fake,
            )
            .await
            .unwrap();
            let wire = serde_json::to_value(&view).unwrap();
            let summary = wire
                .get("collectionSummary")
                .expect("successful US collection summary must reach public View");
            assert_eq!(summary["asOf"], "2026-01-08T04:00:00Z");
            assert_eq!(summary["requestedStart"], serde_json::Value::Null);
            assert_eq!(summary["requestedCount"], 20);
            assert_eq!(summary["maxPages"], 16);
            assert_eq!(summary["maxTypedRows"], 3200);
            assert_eq!(summary["pageSize"], 200);
            assert_eq!(summary["pageTimeoutSeconds"], 15);
            assert_eq!(summary["remainingBudgetMsAtStart"], 30000);
            assert_eq!(summary["sourceAvailability"], availability);
            assert_eq!(summary["coverageVerified"], false);
            assert_eq!(summary["publicationVerification"], "unknown");
            assert_eq!(summary["dailyOhlcVerification"], "unverified");
            assert_eq!(summary["adjustmentVerification"], "unverified");
            assert_eq!(summary["admission"]["adoptedRows"], 20);
            assert_eq!(summary["admission"]["usedDailyRows"], 1);
            assert_eq!(
                summary["admission"]["evaluationStart"],
                "2026-01-07T20:41:00Z"
            );
            assert_eq!(
                summary["admission"]["evaluationEnd"],
                "2026-01-07T21:00:00Z"
            );
            assert_eq!(summary["admission"]["dailyContextStart"], "20260106");
            for (stream, count) in [("daily", 1), ("minute", 20)] {
                for key in ["dispatchedPages", "returnedPages", "validatedPages"] {
                    assert_eq!(summary[stream][key], 1);
                }
                for key in ["returnedTypedRows", "validatedTypedRows", "uniqueRows"] {
                    assert_eq!(summary[stream][key], count);
                }
                assert_eq!(summary[stream]["duplicateRows"], 0);
                assert_eq!(
                    summary[stream]["sources"].as_array().unwrap().len(),
                    usize::from(sources && (stream == "daily" || minute_source))
                );
                if sources && (stream == "daily" || minute_source) {
                    assert_eq!(
                        summary[stream]["sources"][0]["collectedAt"],
                        "2026-01-08T04:00:01Z"
                    );
                    assert_eq!(summary[stream]["sources"][0]["byteLength"], 123);
                }
            }
            let hash = summary["normalizedInputSha256"].as_str().unwrap();
            assert_eq!(hash.len(), 64);
            assert_ne!(hash, "ab".repeat(32));
            assert_ne!(hash, view.replay.input_hash);
            assert!(!view.replay.look_ahead_safe);
            assert_eq!(*fake.calls.lock().unwrap(), ["D", "1m"]);
        }
    }
    fn legacy_prepared(interval: &str) -> PreparedLthPreview {
        prepare_lth_preview(LeveragedTrendHoldPreviewInput {
            symbol: "KORU".into(), interval: Some(interval.into()), count: Some(20),
            params: serde_json::json!({"entries":[{"leveraged_symbol":"KORU","quantity":1,"is_overseas":false}]}),
            assumptions: Default::default(), expected_profile_id: None, expected_broker_account_id: None,
        }, lth_runtime::fixture().profile).unwrap()
    }
    #[tokio::test]
    async fn legacy_pipeline_tuple_projection_order_errors_and_replay_hash_are_preserved() {
        for interval in ["1m", "1d"] {
            let fake = fake(false, false);
            let prepared = legacy_prepared(interval);
            let view = execute_lth_preview_with_adapter(prepared, instant(), None, &fake)
                .await
                .unwrap();
            assert!(serde_json::to_value(&view)
                .unwrap()
                .get("collectionSummary")
                .is_none());
            let repeated = legacy_prepared(interval);
            let again = execute_lth_preview_with_adapter(repeated, instant(), None, &fake)
                .await
                .unwrap();
            assert_eq!(view.replay.input_hash, again.replay.input_hash);
            assert_eq!(
                serde_json::to_value(view.backtest).unwrap(),
                serde_json::to_value(again.backtest).unwrap()
            );
            let expected = if interval == "1m" {
                vec!["D", "1m", "D", "1m"]
            } else {
                vec!["D", "D"]
            };
            assert_eq!(*fake.calls.lock().unwrap(), expected);
        }
        let mut minute_failure = fake(false, false);
        minute_failure.fail_minute = true;
        let error = execute_lth_preview_with_adapter(
            legacy_prepared("1m"),
            instant(),
            None,
            &minute_failure,
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "TOSS_CANDLES_ERROR");
        assert!(error.message.contains("Toss 1분봉 조회 실패:"));
        assert!(error.message.contains("offline original"));
        assert_eq!(*minute_failure.calls.lock().unwrap(), ["D", "1m"]);
        let fake = fake(true, false);
        let error =
            execute_lth_preview_with_adapter(lth_runtime::fixture(), instant(), None, &fake)
                .await
                .unwrap_err();
        assert_eq!(error.code, "TOSS_CANDLES_ERROR");
        assert!(error.message.contains("Toss 일봉 조회 실패:"));
        assert!(error.message.contains("offline original"));
        assert_eq!(*fake.calls.lock().unwrap(), ["D"]);
    }
    #[tokio::test(start_paused = true)]
    async fn collection_failure_preserves_error_and_stops_io_without_success_view() {
        let fake = fake(true, true);
        let error = execute_lth_preview_with_adapter(
            lth_runtime::fixture(),
            instant(),
            Some(lth_collection::Context::new(instant(), 20)),
            &fake,
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "LTH_COLLECTION_INCOMPLETE");
        assert!(error.message.contains("provider_error"));
        assert_eq!(*fake.calls.lock().unwrap(), ["D"]);
    }
    // Fixed oracles were generated from the preserved pre-A5.3 production
    // body with only an injected offline adapter, not from this implementation.
    #[tokio::test]
    async fn legacy_pipeline_matches_before_fixed_hash_and_backtest() {
        use sha2::Digest;
        for (interval, input_hash, report_sha256) in [
            (
                "1m",
                "68cf84d26d25de5b",
                "0b8f69bd80ea0f629c223594bd4cad899baf66b038ec3b9d6fa0542e7789fb16",
            ),
            (
                "1d",
                "bc755c56fd89f95b",
                "23f60ce2f9f3bd8a0fbff59cea0176908d79a782e07b1dc8d5c7a25126ef3116",
            ),
        ] {
            let adapter = fake(false, false);
            let view = execute_lth_preview_with_adapter(
                legacy_prepared(interval),
                instant(),
                None,
                &adapter,
            )
            .await
            .unwrap();
            assert_eq!(
                view.replay.input_hash, input_hash,
                "legacy engine hash changed from before snapshot: {interval}"
            );
            let report =
                serde_json::to_vec(&serde_json::to_value(&view.backtest).unwrap()).unwrap();
            assert_eq!(
                sha2::Sha256::digest(report)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                report_sha256,
                "legacy backtest changed from before snapshot: {interval}"
            );
        }
    }
}
