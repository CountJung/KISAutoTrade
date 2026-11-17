use super::*;
#[tokio::test]
async fn real_core_returns_summary() {
    let fake = Fake::new(vec![
        daily(),
        page(
            vec![
                row("2026-01-07T15:59:00-05:00"),
                row("2026-01-07T16:00:00-05:00"),
            ],
            None,
            false,
        ),
    ]);
    let outcome = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), context())
        .await
        .unwrap();
    assert!(
        outcome.summary.is_some(),
        "successful real core must return collection summary"
    );
}

// Synthetic fingerprints only, never evidence of an actual HTTP response.
struct Sourced {
    fake: Fake,
    present: Mutex<VecDeque<bool>>,
    ready_delay: Duration,
}
#[async_trait::async_trait]
impl BrokerAdapter for Sourced {
    fn broker_id(&self) -> BrokerId {
        self.fake.broker_id()
    }
    async fn get_price(&self, _: &BrokerSymbol) -> BrokerAdapterResult<BrokerPriceQuote> {
        panic!("offline")
    }
    async fn get_candles(
        &self,
        _: &BrokerSymbol,
        _: &str,
        _: &str,
        _: &str,
    ) -> BrokerAdapterResult<Vec<BrokerCandle>> {
        panic!("page only")
    }
    async fn get_candle_page(
        &self,
        r: &CandlePageRequest,
    ) -> BrokerAdapterResult<BrokerCandlePage> {
        self.fake.get_candle_page(r).await
    }
    async fn get_candle_page_with_source(
        &self,
        r: &CandlePageRequest,
    ) -> BrokerAdapterResult<crate::broker::candles::BrokerCandlePageWithSource> {
        let page = self.fake.get_candle_page(r).await?;
        if !self.ready_delay.is_zero() {
            // Advance the paused clock only after the actual adapter entry. Tokio
            // 1.54.1 advances before yield_now; poll once, verify the jump, then
            // drop that yield so this adapter returns Ready in the same poll.
            let before = tokio::time::Instant::now();
            let mut advance = Box::pin(tokio::time::advance(self.ready_delay));
            std::future::poll_fn(|cx| {
                let _ = std::future::Future::poll(advance.as_mut(), cx);
                assert_eq!(tokio::time::Instant::now() - before, self.ready_delay);
                std::task::Poll::Ready(())
            })
            .await;
        }
        let present = self.present.lock().unwrap().pop_front().unwrap();
        Ok(crate::broker::candles::BrokerCandlePageWithSource {
            page,
            source: present.then(|| crate::broker::candles::CandleResponseSource {
                sha256: "synthetic-test-label-not-http-evidence".into(),
                byte_length: 17,
                collected_at: context().as_of,
            }),
        })
    }
}
fn small_replies() -> Vec<Reply> {
    vec![
        daily(),
        page(
            vec![
                row("2026-01-07T15:59:00-05:00"),
                row("2026-01-07T16:00:00-05:00"),
            ],
            None,
            false,
        ),
    ]
}
async fn outcome(rows: Vec<Reply>, c: Context) -> Outcome {
    collect_with_summary(&Fake::new(rows), &BrokerSymbol("KORU".into()), c)
        .await
        .unwrap()
}
#[tokio::test]
async fn source_availability_and_old_fake_default_are_observations_only() {
    let old = outcome(small_replies(), context()).await.summary.unwrap();
    assert_eq!(old.source_availability, "unavailable");
    for (present, expected) in [
        (vec![true, true], "complete"),
        (vec![true, false], "partial"),
        (vec![false, false], "unavailable"),
    ] {
        let expected_daily_sources = usize::from(present[0]);
        let expected_minute_sources = usize::from(present[1]);
        let fake = Sourced {
            fake: Fake::new(small_replies()),
            present: Mutex::new(present.into()),
            ready_delay: Duration::ZERO,
        };
        let s = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), context())
            .await
            .unwrap()
            .summary
            .unwrap();
        assert_eq!(s.source_availability, expected);
        assert_eq!(fake.fake.calls.lock().unwrap().len(), 2);
        assert!(!s.coverage_verified);
        assert_eq!(s.publication_verification, "unknown");
        assert_eq!(s.adjustment_verification, "unverified");
        assert_eq!(s.daily_ohlc_verification, "unverified");
        assert_eq!(s.daily.sources.len(), expected_daily_sources);
        assert_eq!(s.minute.sources.len(), expected_minute_sources);
        for (src, expected_stream) in s
            .daily
            .sources
            .iter()
            .map(|src| (src, "D"))
            .chain(s.minute.sources.iter().map(|src| (src, "1m")))
        {
            assert_eq!(src.response_index, 1);
            assert_eq!(src.stream, expected_stream);
            assert_eq!(src.sha256, "synthetic-test-label-not-http-evidence");
            assert_eq!(src.byte_length, 17);
            assert_eq!(src.collected_at, context().as_of);
        }
    }
}
#[tokio::test]
async fn real_200_regular_80_afterhours_exact_statistics_and_bounds() {
    let close = DateTime::parse_from_rfc3339("2026-01-07T16:00:00-05:00").unwrap();
    let regular: Vec<_> = (0..200)
        .map(|i| row(&(close - chrono::Duration::minutes(199 - i)).to_rfc3339()))
        .collect();
    let after: Vec<_> = (1..=80)
        .map(|i| row(&(close + chrono::Duration::minutes(i)).to_rfc3339()))
        .collect();
    let mut newest = regular[80..].to_vec();
    newest.extend(after);
    let anchor = regular[80].date.clone();
    let mut c = context();
    c.count = 200;
    let fake = Fake::new(vec![
        daily(),
        page(newest, Some(cursor(&anchor)), false),
        page(regular[..=80].to_vec(), None, false),
    ]);
    let out = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), c)
        .await
        .unwrap();
    let s = out.summary.unwrap();
    let a = s.admission.unwrap();
    assert_eq!(out.daily.len(), 1);
    assert_eq!(out.minute.len(), 280);
    assert_eq!(s.requested_start, None);
    assert_eq!(s.as_of, c.as_of);
    assert_eq!(s.requested_count, 200);
    assert_eq!(
        (
            s.max_pages,
            s.max_typed_rows,
            s.page_size,
            s.page_timeout_seconds
        ),
        (16, 3200, 200, 15)
    );
    assert_eq!(
        (
            s.daily.dispatched_pages,
            s.daily.returned_pages,
            s.daily.returned_typed_rows,
            s.daily.unique_rows
        ),
        (1, 1, 1, 1)
    );
    assert_eq!(
        (
            s.minute.dispatched_pages,
            s.minute.returned_pages,
            s.minute.validated_pages
        ),
        (2, 2, 2)
    );
    assert_eq!(
        (
            s.minute.returned_typed_rows,
            s.minute.validated_typed_rows,
            s.minute.duplicate_rows,
            s.minute.unique_rows
        ),
        (281, 281, 1, 280)
    );
    assert_eq!(
        s.minute.observed_start,
        Some((close - chrono::Duration::minutes(199)).to_utc())
    );
    assert_eq!(
        s.minute.observed_end,
        Some((close + chrono::Duration::minutes(80)).to_utc())
    );
    assert_eq!(a.adopted_rows, 200);
    assert_eq!(a.used_daily_rows, 1);
    assert_eq!(a.excluded, BTreeMap::from([("OutsideRegular".into(), 80)]));
    assert_eq!(
        a.evaluation_start,
        Some((close - chrono::Duration::minutes(199)).to_utc())
    );
    assert_eq!(a.evaluation_end, Some(close.to_utc()));
    assert_eq!(a.daily_context_start.as_deref(), Some("20260106"));
    assert_eq!(a.daily_context_end.as_deref(), Some("20260106"));
    assert_eq!(s.admission_status, "lastCompleted");
    assert_eq!(
        s.normalized_input_sha256.as_deref(),
        Some("e9e8d19e3506dbbe2e5d074cb556e6b93550aced1b6018ef354c1b5ced13cc4a")
    );
    assert_eq!(&out.minute[..200], regular.as_slice());
    let calls = fake.calls.lock().unwrap();
    assert_eq!(calls.len(), 3);
    assert!(calls.iter().all(|r| r.count == 200));
    assert_eq!(calls[0].cursor, calls[1].cursor);
    assert_eq!(calls[0].cursor, Some(cursor(&c.as_of.to_rfc3339())));
}
#[tokio::test]
async fn selection_limit_is_distinct_from_collector_duplicates() {
    let fake = Fake::new(vec![
        daily(),
        page(
            vec![
                row("2026-01-07T15:58:00-05:00"),
                row("2026-01-07T15:59:00-05:00"),
                row("2026-01-07T16:00:00-05:00"),
            ],
            None,
            false,
        ),
    ]);
    let s = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), context())
        .await
        .unwrap()
        .summary
        .unwrap();
    assert_eq!(s.minute.duplicate_rows, 0);
    assert_eq!(
        s.admission.unwrap().excluded,
        BTreeMap::from([("SelectionLimit".into(), 1)])
    );
}
#[tokio::test]
async fn native_precision_currency_volume_and_aliases_survive_hashing() {
    let base = outcome(small_replies(), context()).await;
    let hash = base.summary.unwrap().normalized_input_sha256.unwrap();
    let collect = |d: Vec<BrokerCandle>, m: Vec<BrokerCandle>| {
        vec![page(d, None, true), page(m, None, false)]
    };
    let mut alias = base.minute.clone();
    alias[0].date = "2026-01-07T20:59:00Z".into();
    assert_eq!(
        outcome(collect(base.daily.clone(), alias), context())
            .await
            .summary
            .unwrap()
            .normalized_input_sha256
            .as_deref(),
        Some(hash.as_str())
    );
    for variant in 0..5 {
        let mut m = base.minute.clone();
        match variant {
            0 => m[0].open.amount = "100".into(),
            1 => m[0].open.currency = crate::broker::BrokerCurrency::Krw,
            2 => m[0].volume.0 = "9007199254740993".into(),
            3 => m[0].open.amount = "100.00000000000000001".into(),
            _ => m[0].open.amount = "100.00000000000000002".into(),
        }
        let other = outcome(collect(base.daily.clone(), m), context())
            .await
            .summary
            .unwrap()
            .normalized_input_sha256
            .unwrap();
        assert_ne!(other, hash);
        if variant == 3 {
            let mut m = base.minute.clone();
            m[0].open.amount = "100.00000000000000002".into();
            assert_ne!(
                other,
                outcome(collect(base.daily.clone(), m), context())
                    .await
                    .summary
                    .unwrap()
                    .normalized_input_sha256
                    .unwrap()
            );
        }
    }
    let mut d = base.daily.clone();
    d[0].date = "2026-01-05T23:00:00-06:00".into(); // same instant, different join date
    let changed = outcome(collect(d, base.minute), context()).await;
    assert_eq!(
        changed
            .summary
            .as_ref()
            .unwrap()
            .admission
            .as_ref()
            .unwrap()
            .daily_context_start
            .as_deref(),
        Some("20260105")
    );
    assert_ne!(
        changed.summary.unwrap().normalized_input_sha256.unwrap(),
        hash
    );
}
#[tokio::test]
async fn conflicting_overlap_keeps_only_validated_cohort_and_last_admission() {
    for variant in 0..3 {
        let mut conflict = row("2026-01-07T21:00:00Z");
        match variant {
            0 => conflict.open.amount = "100".into(),
            1 => conflict.close.currency = crate::broker::BrokerCurrency::Krw,
            _ => conflict.volume.0 = "1235".into(),
        }
        let fake = Fake::new(vec![
            daily(),
            page(
                vec![row("2026-01-07T16:00:00-05:00")],
                Some(cursor("2026-01-07T16:00:00-05:00")),
                false,
            ),
            page(
                vec![row("2026-01-07T15:59:00-05:00"), conflict],
                None,
                false,
            ),
        ]);
        let f = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), context())
            .await
            .unwrap_err();
        assert!(f.error.message.contains("conflicting_overlap"));
        assert_eq!(f.error.code, "LTH_COLLECTION_INCOMPLETE");
        assert_eq!(
            (
                f.summary.minute.returned_typed_rows,
                f.summary.minute.validated_typed_rows,
                f.summary.minute.unique_rows
            ),
            (3, 1, 1)
        );
        assert_eq!(f.summary.admission.unwrap().adopted_rows, 1);
        assert!(f.summary.normalized_input_sha256.is_none());
    }
}

#[tokio::test]
async fn independent_python_literal_oracle_matches_real_core() {
    let out = outcome(small_replies(), context()).await;
    let payload = normalized_payload(&out.daily, &out.minute, context()).unwrap();
    assert_eq!(
        std::str::from_utf8(&payload).unwrap(),
        r#"{"version":"lth-typed-collected-v1","asOf":"2026-01-08T04:00:00Z","requestedCount":2,"daily":[{"symbol":"KORU","market":"us","instant":"2026-01-06T05:00:00Z","tradingDate":"20260106","sourceLabel":"2026-01-06T00:00:00-05:00","open":{"amount":"100.00","currency":"USD"},"high":{"amount":"102.00","currency":"USD"},"low":{"amount":"99.00","currency":"USD"},"close":{"amount":"101.00","currency":"USD"},"volume":"1234"}],"minute":[{"symbol":"KORU","market":"us","instant":"2026-01-07T20:59:00Z","tradingDate":null,"sourceLabel":null,"open":{"amount":"100.00","currency":"USD"},"high":{"amount":"102.00","currency":"USD"},"low":{"amount":"99.00","currency":"USD"},"close":{"amount":"101.00","currency":"USD"},"volume":"1234"},{"symbol":"KORU","market":"us","instant":"2026-01-07T21:00:00Z","tradingDate":null,"sourceLabel":null,"open":{"amount":"100.00","currency":"USD"},"high":{"amount":"102.00","currency":"USD"},"low":{"amount":"99.00","currency":"USD"},"close":{"amount":"101.00","currency":"USD"},"volume":"1234"}]}"#
    );
    assert_eq!(
        out.summary.unwrap().normalized_input_sha256.as_deref(),
        Some("ee1a4aa1a00ab9ce225b197b57ffaac9d5a7dbbf9f3cc24e80aab98d94c9462e")
    );
}

fn work_counts() -> (usize, usize) {
    (
        PREPARE_CALLS.with(|n| n.get()),
        HASH_CALLS.with(|n| n.get()),
    )
}
#[tokio::test(start_paused = true)]
async fn expired_zero_and_pending_deadline_keep_summary_without_post_failure_work() {
    for zero in [false, true] {
        let mut c = context();
        if zero {
            c.count = 0;
        } else {
            c.deadline = tokio::time::Instant::now();
        }
        let before = work_counts();
        let fake = Fake::new(vec![]);
        let f = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), c)
            .await
            .unwrap_err();
        assert!(f.error.message.contains(if zero {
            "invalid_policy"
        } else {
            "total_deadline"
        }));
        assert_eq!(fake.calls.lock().unwrap().len(), 0);
        assert_eq!(work_counts(), before);
        assert_eq!(f.summary.admission_status, "unavailable");
        assert!(f.summary.admission.is_none());
        assert_eq!(f.summary.daily.dispatched_pages, 0);
        assert!(f.summary.normalized_input_sha256.is_none());
    }
    let mut c = context();
    c.deadline = tokio::time::Instant::now() + Duration::from_millis(5);
    let dropped = Arc::new(AtomicBool::new(false));
    let fake = Fake::new(vec![
        daily(),
        page(
            vec![row("2026-01-07T16:00:00-05:00")],
            Some(cursor("2026-01-07T16:00:00-05:00")),
            false,
        ),
        Reply::Pending(dropped.clone()),
    ]);
    let before = work_counts();
    let f = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), c)
        .await
        .unwrap_err();
    assert!(f.error.message.contains("total_deadline"));
    assert!(dropped.load(Ordering::SeqCst));
    assert_eq!(f.summary.minute.dispatched_pages, 2);
    assert_eq!(f.summary.minute.returned_pages, 1);
    assert_eq!(f.summary.admission_status, "lastCompleted");
    assert_eq!(f.summary.admission.unwrap().adopted_rows, 1);
    assert_eq!(work_counts(), (before.0 + 1, before.1));
}
#[tokio::test(start_paused = true)]
async fn ready_noncooperative_response_cannot_trigger_prepare_or_hash_after_deadline() {
    let mut c = context();
    c.deadline = tokio::time::Instant::now() + Duration::from_millis(20);
    let fake = Sourced {
        fake: Fake::new(vec![daily()]),
        present: Mutex::new(vec![true].into()),
        ready_delay: Duration::from_millis(40),
    };
    let before = work_counts();
    let f = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), c)
        .await
        .unwrap_err();
    assert!(f.error.message.contains("total_deadline"));
    assert_eq!(fake.fake.calls.lock().unwrap().len(), 1);
    assert_eq!(f.summary.daily.returned_pages, 1);
    assert_eq!(f.summary.daily.validated_pages, 0);
    assert_eq!(work_counts(), before);
    assert!(f.summary.admission.is_none());
    assert!(f.summary.normalized_input_sha256.is_none());
}
#[tokio::test]
async fn failures_preserve_errors_and_known_partial_snapshot_without_final_hash() {
    let t = "2026-01-07T16:00:00-05:00";
    let cases = vec![
        (vec![daily(), page(vec![], None, false)], "empty_page", 0, 1),
        (
            vec![
                daily(),
                page(vec![row(t)], Some(cursor(t)), false),
                Reply::Error,
            ],
            "provider_error",
            1,
            1,
        ),
        (
            vec![
                daily(),
                page(vec![row(t)], Some(cursor(t)), false),
                page(
                    vec![row(t)],
                    Some(cursor("2026-01-07T15:59:00-05:00")),
                    false,
                ),
            ],
            "no_progress",
            1,
            1,
        ),
        (
            vec![
                daily(),
                page(vec![row(t)], Some(cursor(t)), false),
                page(
                    vec![row("2026-01-07T15:59:00-05:00")],
                    Some(cursor("2026-01-07T21:00:00Z")),
                    false,
                ),
            ],
            "cursor_no_progress",
            1,
            1,
        ),
        (
            vec![
                page(vec![row("2026-01-07T00:00:00-05:00")], None, true),
                page(vec![row("2026-01-07T15:59:00-05:00"), row(t)], None, false),
            ],
            "daily_context_missing",
            2,
            2,
        ),
    ];
    for (replies, reason, adopted, prepare_count) in cases {
        let fake = Fake::new(replies);
        let before = work_counts();
        let f = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), context())
            .await
            .unwrap_err();
        assert_eq!(f.error.code, "LTH_COLLECTION_INCOMPLETE");
        assert!(f.error.message.contains(reason), "{}", f.error.message);
        assert!(!f.error.message.contains("secret"));
        assert_eq!(
            f.summary
                .admission
                .as_ref()
                .map(|a| a.adopted_rows)
                .unwrap_or(0),
            adopted
        );
        assert_eq!(
            work_counts(),
            (
                before.0 + if adopted == 0 { 0 } else { prepare_count },
                before.1
            )
        );
        assert!(f.summary.normalized_input_sha256.is_none());
    }
    for cap in ["page_limit", "row_limit"] {
        let mut c = context();
        if cap == "page_limit" {
            c.max_pages = 1;
        } else {
            c.max_rows = 1;
        }
        let before = work_counts();
        let fake = Fake::new(vec![daily()]);
        let f = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), c)
            .await
            .unwrap_err();
        assert!(f.error.message.contains(cap));
        assert_eq!(f.summary.daily.unique_rows, 1);
        assert_eq!(f.summary.minute.dispatched_pages, 0);
        assert_eq!(work_counts(), before);
    }
}
#[tokio::test]
async fn normalized_adapter_stall_and_invalid_page_are_not_valid_cohort() {
    for bad_size in [false, true] {
        let mut p = BrokerCandlePage {
            candles: vec![row("2026-01-07T16:00:00-05:00"); if bad_size { 201 } else { 1 }],
            next_cursor: None,
            end_reason: Some("cursorDidNotAdvance".into()),
            timestamp_semantics: "barClose",
        };
        if bad_size {
            p.end_reason = None;
        }
        let fake = Fake::new(vec![daily(), Reply::Page(p)]);
        let before = work_counts();
        let f = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), context())
            .await
            .unwrap_err();
        assert!(f.error.message.contains(if bad_size {
            "invalid_page_size"
        } else {
            "cursor_no_progress"
        }));
        assert_eq!(f.summary.minute.returned_pages, 1);
        assert_eq!(f.summary.minute.validated_pages, 0);
        assert_eq!(f.summary.minute.unique_rows, 0);
        assert!(f.summary.admission.is_none());
        assert_eq!(work_counts(), before);
    }
}

#[tokio::test(start_paused = true)]
async fn reservation_remaining_budget_is_captured_once_at_collection_entry() {
    let c = context();
    tokio::time::advance(Duration::from_secs(7)).await;
    let fake = Sourced {
        fake: Fake::new(small_replies()),
        present: Mutex::new(vec![true, true].into()),
        ready_delay: Duration::from_millis(40),
    };
    let out = collect_with_summary(&fake, &BrokerSymbol("KORU".into()), c)
        .await
        .unwrap();
    let summary = out.summary.unwrap();
    assert_eq!(summary.remaining_budget_ms_at_start, 23_000);
    assert_eq!(
        c.deadline
            .duration_since(tokio::time::Instant::now())
            .as_millis(),
        22_920,
        "both pages consumed virtual time without replenishing the reservation"
    );
    assert_eq!(fake.fake.calls.lock().unwrap().len(), 2);
    assert_eq!(summary.as_of, c.as_of);

    let mut invalid = context();
    invalid.count = 0;
    tokio::time::advance(Duration::from_secs(3)).await;
    let never_called = Fake::new(Vec::new());
    let failed = collect_with_summary(&never_called, &BrokerSymbol("KORU".into()), invalid)
        .await
        .unwrap_err();
    assert!(failed.error.message.contains("invalid_policy"));
    assert_eq!(failed.summary.remaining_budget_ms_at_start, 27_000);
    assert_eq!(never_called.calls.lock().unwrap().len(), 0);
}
