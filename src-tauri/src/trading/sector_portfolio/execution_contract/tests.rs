use super::*;
use crate::trading::sector_portfolio::SectorTarget;

fn stamp(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().to_utc()
}
fn fixture() -> ExecutionPreflight {
    let date = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
    let accounts = [
        SectorStrategy::Base,
        SectorStrategy::Defensive,
        SectorStrategy::Aggressive,
    ]
    .into_iter()
    .map(|strategy| {
        let id = expected_id(strategy).unwrap();
        let weight = if strategy == SectorStrategy::Aggressive {
            0.475
        } else {
            0.3
        };
        let mut account = LockedAccountPlan {
            account_id: id.into(),
            decision_key: format!("sector-paper/comparison-20261106/{date}/{id}/preopen-decision"),
            plan_version: "sector-paper-plan-v1".into(),
            locked_plan_hash: String::new(),
            locked_at: stamp("2026-11-09T14:20:00Z"),
            cash_usd: 10_000.0,
            holdings: vec![],
            reference_closes_usd: BTreeMap::new(),
            signal_input_hash: "a".repeat(64),
            plan: SectorPlan {
                strategy,
                signal_date: "2026-11-06".into(),
                defensive_state: DefensiveState::Normal,
                selected_symbols: vec!["XLK".into()],
                target_weight: weight,
                reference_equity_usd: 10_000.0,
                rebalance: true,
                reasons: vec!["selectionChanged".into()],
                targets: SECTOR_SYMBOLS
                    .iter()
                    .map(|s| SectorTarget {
                        symbol: (*s).into(),
                        weight: if *s == "XLK" { weight } else { 0.0 },
                        budget_usd: if *s == "XLK" { 10_000.0 * weight } else { 0.0 },
                        reference_weight: 0.0,
                    })
                    .collect(),
            },
        };
        reseal(&mut account);
        account
    })
    .collect();
    ExecutionPreflight {
        trading_date: date,
        execute_at: stamp("2026-11-09T14:50:00Z"),
        accounts,
        observations: vec![DisplayObservation {
            symbol: "XLK".into(),
            price_usd: 100.0,
            provider: ChartProvider::Kis,
            source_url: "https://example.test/synthetic/XLK".into(),
            retrieved_at: stamp("2026-11-09T14:49:00Z"),
            quoted_at: None,
            provider_date: None,
            delayed: None,
        }],
        confirmed_event_keys: BTreeSet::new(),
    }
}
fn reseal(a: &mut LockedAccountPlan) {
    a.locked_plan_hash = locked_plan_hash(a).unwrap();
}

#[test]
fn same_first_valid_observation_is_shared_and_unknown_time_is_not_realtime() {
    let mut input = fixture();
    let mut invalid = input.observations[0].clone();
    invalid.price_usd = f64::NAN;
    input.observations.insert(0, invalid);
    let mut later = input.observations[1].clone();
    later.price_usd = 90.0;
    later.provider = ChartProvider::Toss;
    input.observations.push(later);
    let prepared = prepare_execution(&input).unwrap();
    assert_eq!(prepared.observations.len(), 1);
    assert_eq!(prepared.observations["XLK"].price_usd, 100.0);
    assert_eq!(prepared.accounts.len(), 3);
    assert!(prepared
        .warnings
        .iter()
        .any(|s| s.contains("시세 시각 미확인")));
    assert!(prepared.execution_new_york.contains("09:50:00-05:00"));
}

#[test]
fn missing_excluded_holding_blocks_entire_batch_without_mutation() {
    let mut input = fixture();
    input.accounts[1].holdings.push(SectorHolding {
        symbol: "XLE".into(),
        quantity: 1,
        average_cost_usd: 50.0,
        remaining_buy_cost_usd: 0.05,
    });
    input.accounts[1].cash_usd = 9_950.0;
    input.accounts[1]
        .reference_closes_usd
        .insert("XLE".into(), 50.0);
    input.accounts[1]
        .plan
        .targets
        .iter_mut()
        .find(|t| t.symbol == "XLE")
        .unwrap()
        .reference_weight = 0.005;
    reseal(&mut input.accounts[1]);
    let before = input.accounts[1].locked_plan_hash.clone();
    assert!(prepare_execution(&input).unwrap_err().contains("XLE"));
    assert_eq!(input.accounts[1].locked_plan_hash, before);
    assert_eq!(input.accounts[1].holdings[0].quantity, 1);
}

#[test]
fn source_past_date_and_future_retrieval_fail_but_irrelevant_tickers_do_not_block() {
    let mut input = fixture();
    let mut irrelevant = input.observations[0].clone();
    irrelevant.symbol = "IGNORED".into();
    irrelevant.price_usd = 0.0;
    input.observations.push(irrelevant);
    assert!(prepare_execution(&input).is_ok());
    input.observations[0].provider_date = Some(NaiveDate::from_ymd_opt(2026, 11, 6).unwrap());
    assert!(prepare_execution(&input).is_err());
    input.observations[0].provider_date = None;
    input.observations[0].retrieved_at = input.execute_at + Duration::seconds(1);
    assert!(prepare_execution(&input).is_err());
}

#[test]
fn duplicate_batch_partial_commit_and_plan_tampering_are_rejected() {
    let mut input = fixture();
    let prefix = "sector-paper/comparison-20261106/2026-11-09";
    input
        .confirmed_event_keys
        .insert(format!("{prefix}/execution-batch"));
    assert!(prepare_execution(&input).unwrap_err().contains("DUPLICATE"));
    input.confirmed_event_keys.clear();
    input
        .confirmed_event_keys
        .insert(format!("{prefix}/compare-base-v1/fill"));
    assert!(prepare_execution(&input).unwrap_err().contains("PARTIAL"));
    input.confirmed_event_keys.clear();
    input.accounts[0].cash_usd = 9.0;
    assert!(prepare_execution(&input).unwrap_err().contains("INTEGRITY"));
}

#[test]
fn plan_scope_late_lock_old_signal_and_invalid_fixed_budget_are_rejected() {
    let mut input = fixture();
    input.accounts[1] = input.accounts[0].clone();
    assert!(prepare_execution(&input).is_err());
    let mut input = fixture();
    input.accounts[0].locked_at = stamp("2026-11-09T14:30:00Z");
    reseal(&mut input.accounts[0]);
    assert!(prepare_execution(&input).is_err());
    let mut input = fixture();
    input.accounts[0].plan.signal_date = "2026-11-05".into();
    reseal(&mut input.accounts[0]);
    assert!(prepare_execution(&input).is_err());
    let mut input = fixture();
    input.accounts[0].plan.targets[0].budget_usd = 1.0;
    reseal(&mut input.accounts[0]);
    assert!(prepare_execution(&input).is_err());
}

#[test]
fn quote_clock_is_open_inclusive_close_exclusive_and_handles_dst_holidays_early_close() {
    for good in [
        "2026-11-09T14:30:00Z",
        "2026-07-06T13:30:00Z",
        "2026-11-27T17:59:59Z",
    ] {
        assert!(
            lth_research_session::quote_clock(stamp(good)).is_ok(),
            "{good}"
        );
    }
    for bad in [
        "2026-11-09T14:29:59Z",
        "2026-11-09T21:00:00Z",
        "2026-07-06T20:00:00Z",
        "2026-11-27T18:00:00Z",
        "2026-11-26T15:00:00Z",
        "2026-11-08T15:00:00Z",
        "2029-11-09T15:00:00Z",
    ] {
        assert!(
            lth_research_session::quote_clock(stamp(bad)).is_err(),
            "{bad}"
        );
    }
    assert!(lth_research_session::regular_clock(stamp("2026-11-09T14:30:00Z")).is_err());
    assert!(lth_research_session::regular_clock(stamp("2026-11-09T21:00:00Z")).is_ok());
}

#[test]
fn no_rebalance_requires_no_prices_and_never_fabricates_a_fill() {
    let mut input = fixture();
    for a in &mut input.accounts {
        a.plan.rebalance = false;
        reseal(a);
    }
    input.observations.clear();
    let output = prepare_execution(&input).unwrap();
    assert!(output.observations.is_empty());
    assert_eq!(output.accounts[0].cash_usd, 10_000.0);
    assert_eq!(output.observation_span_milliseconds, 0);
}

#[test]
fn premarket_quote_is_rejected_but_delayed_regular_quote_is_retained() {
    let mut input = fixture();
    input.observations[0].quoted_at = Some(stamp("2026-11-09T14:29:59Z"));
    assert!(prepare_execution(&input).is_err());
    input.observations[0].quoted_at = Some(stamp("2026-11-09T14:30:00Z"));
    input.observations[0].delayed = Some(true);
    let output = prepare_execution(&input).unwrap();
    assert!(output.warnings.iter().any(|w| w.contains("지연 표시가격")));
    assert_eq!(
        output.observation_times["XLK"].quoted_new_york.as_deref(),
        Some("2026-11-09T09:30:00-05:00")
    );
}

#[test]
fn inconsistent_common_selection_and_reference_nav_fail_closed() {
    let mut input = fixture();
    let a = &mut input.accounts[1];
    a.plan.selected_symbols = vec!["XLE".into()];
    for t in &mut a.plan.targets {
        t.weight = if t.symbol == "XLE" { 0.3 } else { 0.0 };
        t.budget_usd = t.weight * 10_000.0;
    }
    reseal(a);
    assert!(prepare_execution(&input)
        .unwrap_err()
        .contains("COMMON_SELECTION"));
    let mut input = fixture();
    input.accounts[0].cash_usd = 9_000.0;
    reseal(&mut input.accounts[0]);
    assert!(prepare_execution(&input)
        .unwrap_err()
        .contains("REFERENCE_NAV"));
    let mut input = fixture();
    input.accounts[0].signal_input_hash = "b".repeat(64);
    reseal(&mut input.accounts[0]);
    assert!(prepare_execution(&input)
        .unwrap_err()
        .contains("SIGNAL_INPUT"));
}

#[test]
fn invalid_holdings_and_experiment_window_are_rejected() {
    let mut input = fixture();
    input.accounts[0].holdings.push(SectorHolding {
        symbol: "XLK".into(),
        quantity: MAX_QUANTITY + 1,
        average_cost_usd: 1.0,
        remaining_buy_cost_usd: 0.0,
    });
    reseal(&mut input.accounts[0]);
    assert!(prepare_execution(&input).unwrap_err().contains("HOLDING"));
    let mut input = fixture();
    input.trading_date = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap();
    input.execute_at = stamp("2026-11-25T14:50:00Z");
    assert!(prepare_execution(&input)
        .unwrap_err()
        .contains("EXPERIMENT_WINDOW"));
}

#[test]
fn overlapping_previous_closes_must_match_across_accounts() {
    let mut input = fixture();
    for (index, a) in input.accounts[..2].iter_mut().enumerate() {
        let price = 50.0 + index as f64;
        a.holdings.push(SectorHolding {
            symbol: "XLE".into(),
            quantity: 1,
            average_cost_usd: 50.0,
            remaining_buy_cost_usd: 0.05,
        });
        a.cash_usd = 10_000.0 - price;
        a.reference_closes_usd.insert("XLE".into(), price);
        a.plan
            .targets
            .iter_mut()
            .find(|t| t.symbol == "XLE")
            .unwrap()
            .reference_weight = price / 10_000.0;
        reseal(a);
    }
    assert!(prepare_execution(&input)
        .unwrap_err()
        .contains("COMMON_REFERENCE_PRICE"));
}

#[test]
fn tiny_nonselected_weight_and_budget_are_rejected_without_indexing_missing_price() {
    for (weight, budget) in [(1e-9, 0.0), (-1e-9, 0.0), (0.0, 1e-9)] {
        let mut input = fixture();
        let target = &mut input.accounts[0].plan.targets[0];
        target.weight = weight;
        target.budget_usd = budget;
        reseal(&mut input.accounts[0]);
        assert!(prepare_execution(&input).unwrap_err().contains("TARGET"));
    }
}

#[test]
fn malformed_or_credentialed_source_urls_are_never_retained() {
    for url in [
        "https://",
        "https:// ",
        "https://name:secret@example.test/quote",
        "http://example.test/quote",
    ] {
        let mut input = fixture();
        input.observations[0].source_url = url.into();
        assert!(prepare_execution(&input)
            .unwrap_err()
            .contains("MISSING_PRICES"));
        let valid = fixture().observations.remove(0);
        input.observations.push(valid);
        let output = prepare_execution(&input).unwrap();
        assert_eq!(
            output.observations["XLK"].source_url,
            "https://example.test/synthetic/XLK"
        );
    }
}
