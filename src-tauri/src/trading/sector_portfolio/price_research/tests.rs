use super::*;
use crate::trading::sector_portfolio::{
    preview_sector_portfolio, SectorCorporateAction, SectorStrategy, ALL_SYMBOLS,
};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::collections::BTreeMap;

fn fixture() -> BrokerChartInput {
    let mut date = NaiveDate::from_ymd_opt(2025, 9, 1).unwrap();
    let mut days = Vec::new();
    while days.len() < 42 {
        if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
            let index = days.len();
            days.push(BrokerChartDay {
                date: date.to_string(),
                prices: ALL_SYMBOLS
                    .iter()
                    .enumerate()
                    .map(|(symbol_index, symbol)| {
                        let close = if *symbol == "SPY" {
                            100.0
                        } else {
                            100.0 + index as f64 * (symbol_index + 1) as f64 * 0.05
                        };
                        (
                            (*symbol).into(),
                            BrokerChartPrice {
                                open: close,
                                high: close + 1.0,
                                low: close - 1.0,
                                close,
                                volume: 1_000.0,
                                currency: BrokerChartCurrency::Usd,
                            },
                        )
                    })
                    .collect(),
            });
        }
        date += Duration::days(1);
    }
    BrokerChartInput {
        schema_version: 1,
        initial_cash_usd: 10_000.0,
        cost_bps: 10.0,
        source: BrokerChartSource {
            broker_id: BrokerChartBroker::Kis,
            name: "synthetic KIS-shaped unit fixture".into(),
            retrieved_at: "2026-01-01T00:00:00Z".into(),
            synthetic: true,
            adjustment: BrokerChartAdjustment::Unadjusted,
            currency: BrokerChartCurrency::Usd,
            interval: BrokerChartInterval::Daily,
            session_basis: BrokerChartSessionBasis::ObservedSpySessions,
            calendar_evidence: "synthetic weekdays; no actual exchange calendar assertion".into(),
        },
        expected_session_dates: days.iter().map(|d| d.date.clone()).collect(),
        evaluation: days.split_off(22),
        history: days,
        acknowledged_price_only: true,
        known_corporate_actions: Vec::new(),
    }
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-7, "{actual} != {expected}");
}

#[test]
fn broker_chart_raw_mode_never_promotes_source_and_strict_false_still_rejects() {
    let input = fixture();
    let converted = accounting_input(&input);
    assert!(!converted.source.coverage_verified);
    assert!(!converted.source.corporate_actions_verified);
    assert!(preview_sector_portfolio(&converted)
        .unwrap_err()
        .contains("UNVERIFIED_SECTOR_SOURCE"));
    let report = preview_broker_chart_price_approximation(&input).unwrap();
    assert_eq!(report.mode, "brokerChartPriceApproximation");
    assert_eq!(report.signal_basis, "unadjustedClose");
    assert!(!report.dividend_return_included && !report.economic_return_verified);
    assert!(
        !report.result.source.coverage_verified && !report.result.source.corporate_actions_verified
    );
    assert_eq!(
        report.result.source.adjustment_policy,
        "unadjustedSignalRawExecution"
    );
    assert_eq!(report.result.history_bars, 22);
    assert_eq!(report.result.evaluation_bars, 20);
    assert_eq!(report.result.warnings, report.warnings);
    assert!(!report
        .warnings
        .iter()
        .any(|v| v.contains("부재 검증이 없는 입력은 평가를 거부")));
    assert!(report
        .warnings
        .iter()
        .any(|v| v.contains("사건 부재 확인이 아닙니다")));
}

#[test]
fn broker_chart_synthetic_and_real_declarations_remain_distinct_for_kis_and_toss() {
    let mut input = fixture();
    let synthetic = preview_broker_chart_price_approximation(&input).unwrap();
    assert!(synthetic.result.source.synthetic);
    assert!(synthetic.warnings[0].contains("합성 fixture"));
    input.source.synthetic = false;
    input.source.broker_id = BrokerChartBroker::Toss;
    input.source.name = "caller-declared Toss chart input, no live fetch".into();
    let declared = preview_broker_chart_price_approximation(&input).unwrap();
    assert!(!declared.result.source.synthetic);
    assert!(declared.warnings[0].contains("입력자 선언"));
    assert!(!declared.economic_return_verified);
    assert_ne!(declared.input_hash, synthetic.input_hash);
    assert_eq!(
        serde_json::to_value(&input).unwrap()["source"]["brokerId"],
        "toss"
    );
}

#[test]
fn broker_chart_hash_has_mode_basis_schema_and_complete_ohlcv_provenance() {
    let input = fixture();
    let report = preview_broker_chart_price_approximation(&input).unwrap();
    let tagged = serde_json::to_vec(&serde_json::json!({"mode":"brokerChartPriceApproximation","signalBasis":"unadjustedClose","input":input})).unwrap();
    assert_eq!(report.input_hash, hex::encode(Sha256::digest(tagged)));
    assert_eq!(report.result.input_hash, report.input_hash);
    assert_ne!(
        report.input_hash,
        hex::encode(Sha256::digest(serde_json::to_vec(&input).unwrap()))
    );
    let mut changed = input.clone();
    changed.history[0].prices.get_mut("SPY").unwrap().volume += 1.0;
    assert_ne!(
        preview_broker_chart_price_approximation(&changed)
            .unwrap()
            .input_hash,
        report.input_hash
    );
}

#[test]
fn broker_chart_unknown_requires_ack_and_known_split_or_dividend_stops_every_book() {
    let mut input = fixture();
    input.acknowledged_price_only = false;
    assert_eq!(
        preview_broker_chart_price_approximation(&input).unwrap_err(),
        "BROKER_CHART_PRICE_ONLY_ACK_REQUIRED"
    );
    input.acknowledged_price_only = true;
    for kind in ["split", "dividend"] {
        input.known_corporate_actions = vec![SectorCorporateAction {
            symbol: "SPY".into(),
            date: input.evaluation[0].date.clone(),
            kind: kind.into(),
            split_ratio: Some(2.0),
            dividend_per_share: None,
            pay_date: None,
        }];
        assert_eq!(
            preview_broker_chart_price_approximation(&input).unwrap_err(),
            "UNSUPPORTED_BROKER_CHART_CORPORATE_ACTION"
        );
    }
}

#[test]
fn broker_chart_contract_rejects_other_vendors_currency_adjustment_and_unknown_fields() {
    let original = serde_json::to_value(fixture()).unwrap();
    for (pointer, value) in [
        ("/source/brokerId", "yahoo"),
        ("/source/adjustment", "adjusted"),
        ("/source/currency", "KRW"),
        ("/source/interval", "1m"),
        ("/history/0/prices/SPY/currency", "KRW"),
    ] {
        let mut json = original.clone();
        *json.pointer_mut(pointer).unwrap() = value.into();
        assert!(
            serde_json::from_value::<BrokerChartInput>(json).is_err(),
            "{pointer}"
        );
    }
    let mut json = original;
    json["source"]["coverageVerified"] = true.into();
    assert!(serde_json::from_value::<BrokerChartInput>(json).is_err());
    let price = serde_json::to_string(&fixture().history[0].prices["SPY"]).unwrap();
    let duplicate =
        format!("{{\"date\":\"2025-09-01\",\"prices\":{{\"SPY\":{price},\"SPY\":{price}}}}}");
    assert!(serde_json::from_str::<BrokerChartDay>(&duplicate)
        .unwrap_err()
        .to_string()
        .contains("DUPLICATE_BROKER_CHART_TICKER"));
    let conflicting = duplicate.replacen("\"open\":100.0", "\"open\":99.0", 1);
    assert!(serde_json::from_str::<BrokerChartDay>(&conflicting).is_err());
}

#[test]
fn broker_chart_invalid_universe_dates_calendar_ohlcv_and_completion_fail_closed() {
    for case in 0..15 {
        let mut input = fixture();
        match case {
            0 => {
                input.evaluation[0].prices.remove("SPY");
            }
            1 => {
                input.evaluation[0]
                    .prices
                    .insert("QQQ".into(), input.history[0].prices["SPY"].clone());
            }
            2 => input.evaluation[1].date = input.evaluation[0].date.clone(),
            3 => input.expected_session_dates.swap(22, 23),
            4 => input.history[0].date = "2025-09-06".into(),
            5 => input.source.retrieved_at = "2099-01-01T00:00:00Z".into(),
            6 => {
                input.source.retrieved_at =
                    format!("{}T23:59:00Z", input.evaluation.last().unwrap().date)
            }
            7 => input.history[0].prices.get_mut("SPY").unwrap().open = 0.0,
            8 => input.history[0].prices.get_mut("SPY").unwrap().high = 99.0,
            9 => input.history[0].prices.get_mut("SPY").unwrap().low = 101.0,
            10 => input.history[0].prices.get_mut("SPY").unwrap().volume = -1.0,
            11 => input.history[0].prices.get_mut("SPY").unwrap().close = f64::NAN,
            12 => input.source.calendar_evidence.clear(),
            13 => input.history.truncate(21),
            _ => input.cost_bps = 30.0,
        }
        assert!(
            preview_broker_chart_price_approximation(&input).is_err(),
            "case {case}"
        );
    }
}

#[test]
fn broker_chart_raw_signal_uses_only_prior_close_and_future_cannot_rewrite_prefix() {
    let input = fixture();
    let full = preview_broker_chart_price_approximation(&input).unwrap();
    let first = &full.result.rankings[0];
    assert_eq!(first.signal_date, input.history[21].date);
    for rank in &first.rankings {
        close(
            rank.r20,
            input.history[21].prices[&rank.symbol].close
                / input.history[1].prices[&rank.symbol].close
                - 1.0,
        );
        close(
            rank.r5,
            input.history[21].prices[&rank.symbol].close
                / input.history[16].prices[&rank.symbol].close
                - 1.0,
        );
    }
    let mut prefix = input.clone();
    prefix.evaluation.truncate(5);
    prefix.expected_session_dates.truncate(27);
    let short = preview_broker_chart_price_approximation(&prefix).unwrap();
    let mut changed = input;
    for day in &mut changed.evaluation[5..] {
        for price in day.prices.values_mut() {
            price.open = 1_000.0;
            price.close = 1_000.0;
            price.high = 1_001.0;
            price.low = 999.0;
        }
    }
    let changed = preview_broker_chart_price_approximation(&changed).unwrap();
    for ((a, b), c) in full
        .result
        .portfolios
        .iter()
        .zip(&short.result.portfolios)
        .zip(&changed.result.portfolios)
    {
        assert_eq!(
            serde_json::to_value(&a.daily[..5]).unwrap(),
            serde_json::to_value(&b.daily).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&a.daily[..5]).unwrap(),
            serde_json::to_value(&c.daily[..5]).unwrap()
        );
    }
}

#[test]
fn broker_chart_three_strategy_raw_gaps_reconcile_cash_integer_fees_and_pnl() {
    let mut input = fixture();
    input.evaluation.truncate(2);
    input.expected_session_dates.truncate(24);
    for (index, day) in input.evaluation.iter_mut().enumerate() {
        for (symbol, price) in &mut day.prices {
            if symbol != "SPY" {
                price.open = if index == 0 { 100.0 } else { 80.0 };
                price.close = if index == 0 {
                    if symbol == "XLC" {
                        500.0
                    } else {
                        120.0
                    }
                } else {
                    80.0
                };
                price.high = price.open.max(price.close);
                price.low = price.open.min(price.close);
            }
        }
    }
    let report = preview_broker_chart_price_approximation(&input).unwrap();
    let strategies = &report.result.portfolios[..3];
    assert_eq!(
        strategies.iter().map(|p| p.strategy).collect::<Vec<_>>(),
        [
            SectorStrategy::Base,
            SectorStrategy::Defensive,
            SectorStrategy::Aggressive
        ]
    );
    assert_eq!(strategies[0].daily[0].plan.target_weight, 0.30);
    assert_eq!(strategies[1].daily[0].plan.target_weight, 0.15);
    assert_eq!(strategies[2].daily[0].plan.target_weight, 0.475);
    assert!(strategies.iter().any(|p| p.summary.cash_limited_count > 0));
    for portfolio in strategies {
        let mut cash = input.initial_cash_usd;
        let mut positions: BTreeMap<String, (u64, f64, f64)> = BTreeMap::new();
        let mut costs = 0.0;
        let mut realized = 0.0;
        let mut saw_buy = false;
        let mut date = String::new();
        for trade in &portfolio.trades {
            if trade.date != date {
                date.clone_from(&trade.date);
                saw_buy = false;
            }
            let amount = trade.quantity as f64 * trade.price_usd;
            close(trade.cost_usd, amount * 0.001);
            costs += trade.cost_usd;
            if trade.side == "sell" {
                assert!(!saw_buy, "sell must precede buys each day");
                let held = positions.get_mut(&trade.symbol).unwrap();
                assert!(trade.quantity <= held.0);
                let fraction = trade.quantity as f64 / held.0 as f64;
                realized += amount - trade.cost_usd - held.1 * fraction - held.2 * fraction;
                held.0 -= trade.quantity;
                held.1 *= 1.0 - fraction;
                held.2 *= 1.0 - fraction;
                cash += amount - trade.cost_usd;
            } else {
                saw_buy = true;
                cash -= amount + trade.cost_usd;
                let held = positions.entry(trade.symbol.clone()).or_default();
                held.0 += trade.quantity;
                held.1 += amount;
                held.2 += trade.cost_usd;
            }
            assert!(cash >= -1e-8);
            close(cash, trade.cash_after_usd);
        }
        let last = input.evaluation.last().unwrap();
        let holdings_value: f64 = positions
            .iter()
            .map(|(symbol, held)| held.0 as f64 * last.prices[symbol].close)
            .sum();
        let unrealized: f64 = positions
            .iter()
            .map(|(symbol, held)| held.0 as f64 * last.prices[symbol].close - held.1 - held.2)
            .sum();
        close(portfolio.summary.ending_cash_usd, cash);
        close(portfolio.summary.ending_equity_usd, cash + holdings_value);
        close(portfolio.summary.total_cost_usd, costs);
        close(portfolio.summary.realized_pnl_usd, realized);
        close(portfolio.summary.unrealized_pnl_usd, unrealized);
        close(
            cash + holdings_value - input.initial_cash_usd,
            realized + unrealized,
        );
    }
    input.cost_bps = 20.0;
    let stressed = preview_broker_chart_price_approximation(&input).unwrap();
    for trade in stressed.result.portfolios.iter().flat_map(|p| &p.trades) {
        close(
            trade.cost_usd,
            trade.quantity as f64 * trade.price_usd * 0.002,
        );
    }
}
