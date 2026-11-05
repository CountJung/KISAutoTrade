use super::*;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::collections::BTreeMap;

fn fixture() -> SectorPortfolioInput {
    let mut date = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
    let mut days = Vec::new();
    while days.len() < 52 {
        if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
            let prices = ALL_SYMBOLS
                .iter()
                .enumerate()
                .map(|(index, symbol)| {
                    let price = if *symbol == "SPY" {
                        100.0
                    } else {
                        100.0 + days.len() as f64 * (index + 1) as f64 / 20.0
                    };
                    (
                        (*symbol).into(),
                        SectorPrice {
                            adjusted_close: price,
                            open: price,
                            close: price,
                        },
                    )
                })
                .collect();
            days.push(SectorDay {
                date: date.format("%Y-%m-%d").to_string(),
                prices,
            });
        }
        date += Duration::days(1);
    }
    SectorPortfolioInput {
        initial_cash_usd: 10_000.0,
        cost_bps: 10.0,
        source: SectorDataSource {
            name: "synthetic-unit-fixture".into(),
            retrieved_at: "2026-11-05T00:00:00Z".into(),
            synthetic: true,
            adjustment_policy: "adjustedSignalRawExecution".into(),
            calendar_source: "explicit synthetic weekday calendar, not official".into(),
            coverage_verified: true,
            corporate_actions_verified: true,
        },
        expected_session_dates: days.iter().map(|day| day.date.clone()).collect(),
        evaluation: days.split_off(22),
        history: days,
        corporate_actions: vec![],
    }
}

fn flat_history() -> Vec<SectorDay> {
    let mut rows = fixture().history;
    for day in &mut rows {
        for price in day.prices.values_mut() {
            price.adjusted_close = 100.0;
            price.open = 100.0;
            price.close = 100.0;
        }
    }
    rows
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-7, "{a} != {b}");
}

#[test]
fn ranks_average_ties_in_full_population_then_filter_and_ticker_breaks_score_tie() {
    let mut history = flat_history();
    for day in &mut history[2..] {
        for symbol in SECTOR_SYMBOLS {
            day.prices.get_mut(symbol).unwrap().adjusted_close = 110.0;
        }
    }
    let rows = rank_sectors(&history).unwrap();
    assert_eq!(rows.len(), 11);
    assert_eq!(rows[0].symbol, "XLB");
    for row in &rows {
        close(row.percentile20, 0.5);
        close(row.percentile5, 0.5);
        assert!(row.eligible);
    }
    let plan = plan_sector_portfolio(
        &history,
        SectorStrategy::Base,
        DefensiveState::Normal,
        &[],
        10_000.0,
        &[],
    )
    .unwrap();
    assert_eq!(plan.selected_symbols, ["XLB", "XLC", "XLE"]);
    close(plan.targets.iter().map(|row| row.budget_usd).sum(), 9_000.0);
}

#[test]
fn strict_positive_eligibility_does_not_rename_zero_as_a_trade() {
    let history = flat_history();
    let rows = rank_sectors(&history).unwrap();
    assert!(rows.iter().all(|row| !row.eligible));
    let plan = plan_sector_portfolio(
        &history,
        SectorStrategy::Aggressive,
        DefensiveState::Normal,
        &[],
        10_000.0,
        &[],
    )
    .unwrap();
    assert!(plan.selected_symbols.is_empty());
    assert!(!plan.rebalance);
    let mut input = fixture();
    input.history = history;
    for day in &mut input.evaluation {
        for p in day.prices.values_mut() {
            p.adjusted_close = 100.0;
            p.open = 100.0;
            p.close = 100.0;
        }
    }
    let result = preview_sector_portfolio(&input).unwrap();
    for row in &result.portfolios[..3] {
        assert!(row.trades.is_empty());
        close(row.summary.net_return_pct, 0.0);
    }
}

#[test]
fn defensive_zero_boundary_mixed_holds_previous_and_positive_pair_recovers() {
    let mut history = flat_history();
    let plan = |rows: &[SectorDay], state| {
        plan_sector_portfolio(rows, SectorStrategy::Defensive, state, &[], 10_000.0, &[]).unwrap()
    };
    assert_eq!(
        plan(&history, DefensiveState::Normal).defensive_state,
        DefensiveState::Defensive
    );
    history[21].prices.get_mut("SPY").unwrap().adjusted_close = 101.0;
    assert_eq!(
        plan(&history, DefensiveState::Defensive).defensive_state,
        DefensiveState::Defensive
    );
    assert_eq!(
        plan(&history, DefensiveState::Normal).defensive_state,
        DefensiveState::Normal
    );
    history[20].prices.get_mut("SPY").unwrap().adjusted_close = 101.0;
    assert_eq!(
        plan(&history, DefensiveState::Defensive).defensive_state,
        DefensiveState::Normal
    );
}

#[test]
fn drift_at_five_percentage_points_rebalances_below_does_not_and_order_is_not_set_change() {
    let history = fixture().history;
    let empty = plan_sector_portfolio(
        &history,
        SectorStrategy::Base,
        DefensiveState::Normal,
        &[],
        10_000.0,
        &[],
    )
    .unwrap();
    // Price100 makes the chosen quantities match35%,30%,30% at NAV10000.
    let mut history = history;
    for bar in history.last_mut().unwrap().prices.values_mut() {
        bar.close = 100.0;
    }
    let holdings = empty
        .selected_symbols
        .iter()
        .enumerate()
        .map(|(i, symbol)| SectorHolding {
            symbol: symbol.clone(),
            quantity: if i == 0 { 35 } else { 30 },
            average_cost_usd: 100.0,
            remaining_buy_cost_usd: 0.0,
        })
        .collect::<Vec<_>>();
    let mut previous = empty.selected_symbols.clone();
    previous.reverse();
    let at = plan_sector_portfolio(
        &history,
        SectorStrategy::Base,
        DefensiveState::Normal,
        &holdings,
        500.0,
        &previous,
    )
    .unwrap();
    assert!(at.reasons.iter().any(|v| v == "weightDriftAtLeast5pp"));
    let mut below = holdings;
    below[0].quantity = 34;
    let below = plan_sector_portfolio(
        &history,
        SectorStrategy::Base,
        DefensiveState::Normal,
        &below,
        600.0,
        &previous,
    )
    .unwrap();
    assert!(!below.rebalance);
}

#[test]
fn repeated_input_is_deterministic_and_appending_future_cannot_change_prior_decisions() {
    let input = fixture();
    let full = preview_sector_portfolio(&input).unwrap();
    assert_eq!(
        serde_json::to_value(&full).unwrap(),
        serde_json::to_value(preview_sector_portfolio(&input).unwrap()).unwrap()
    );
    let mut short = input.clone();
    short.evaluation.truncate(3);
    short.expected_session_dates.truncate(25);
    let prefix = preview_sector_portfolio(&short).unwrap();
    for (a, b) in full.portfolios.iter().zip(&prefix.portfolios) {
        assert_eq!(
            serde_json::to_value(&a.daily[..3]).unwrap(),
            serde_json::to_value(&b.daily).unwrap()
        );
        assert_eq!(
            serde_json::to_value(
                a.trades
                    .iter()
                    .filter(|t| t.date <= short.evaluation[2].date)
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            serde_json::to_value(&b.trades).unwrap()
        );
    }
}

#[test]
fn independent_ledger_reconciles_fees_holdings_cash_turnover_and_pnl_for_all_books() {
    let input = fixture();
    let result = preview_sector_portfolio(&input).unwrap();
    assert_eq!(result.portfolios.len(), 5);
    for portfolio in &result.portfolios {
        let mut cash = input.initial_cash_usd;
        let mut shares: BTreeMap<String, u64> = BTreeMap::new();
        let mut fees = 0.0;
        let mut turnover = 0.0;
        for trade in &portfolio.trades {
            let amount = trade.price_usd * trade.quantity as f64;
            close(trade.cost_usd, amount * 0.001);
            fees += trade.cost_usd;
            turnover += amount;
            if trade.side == "buy" {
                cash -= amount + trade.cost_usd;
                *shares.entry(trade.symbol.clone()).or_default() += trade.quantity;
            } else {
                cash += amount - trade.cost_usd;
                *shares.get_mut(&trade.symbol).unwrap() -= trade.quantity;
            }
            close(cash, trade.cash_after_usd);
            assert!(cash >= 0.0);
        }
        let last = input.evaluation.last().unwrap();
        let equity = cash
            + shares
                .iter()
                .map(|(s, q)| *q as f64 * last.prices[s].close)
                .sum::<f64>();
        close(equity, portfolio.summary.ending_equity_usd);
        close(cash, portfolio.summary.ending_cash_usd);
        close(fees, portfolio.summary.total_cost_usd);
        close(
            turnover / input.initial_cash_usd * 100.0,
            portfolio.summary.turnover_pct,
        );
        close(
            equity - input.initial_cash_usd,
            portfolio.summary.realized_pnl_usd + portfolio.summary.unrealized_pnl_usd,
        );
    }
    let spy = &result.portfolios[3];
    assert_eq!(spy.trades.len(), 1);
    assert_eq!(spy.trades[0].quantity, 99);
    close(spy.summary.ending_cash_usd, 90.1); // 10000 - 99*100*1.001
    let cash = &result.portfolios[4];
    assert!(cash.trades.is_empty());
    close(cash.summary.net_return_pct, 0.0);
}

#[test]
fn same_symbol_prices_are_shared_and_gap_costs_never_borrow_cash() {
    let mut input = fixture();
    for day in &mut input.evaluation {
        for price in day.prices.values_mut() {
            price.open *= 2.0;
        }
    }
    let result = preview_sector_portfolio(&input).unwrap();
    for p in &result.portfolios {
        for trade in &p.trades {
            close(
                trade.price_usd,
                input
                    .evaluation
                    .iter()
                    .find(|d| d.date == trade.date)
                    .unwrap()
                    .prices[&trade.symbol]
                    .open,
            );
            assert!(trade.cash_after_usd >= 0.0);
        }
        assert!(p
            .daily
            .iter()
            .all(|d| d.cash_usd >= 0.0 && d.exposure_pct <= 100.0));
    }
    let defensive = &result.portfolios[1];
    close(defensive.daily[0].plan.target_weight, 0.15);
    let aggressive = &result.portfolios[2];
    close(aggressive.daily[0].plan.target_weight, 0.475);
}

#[test]
fn data_failures_reject_whole_comparison_and_corporate_actions_never_become_zero_return() {
    let input = fixture();
    let reject = |candidate: SectorPortfolioInput, code: &str| {
        assert!(preview_sector_portfolio(&candidate)
            .unwrap_err()
            .contains(code))
    };
    let mut bad = input.clone();
    bad.history.pop();
    reject(bad, "INSUFFICIENT_SECTOR_HISTORY");
    let mut bad = input.clone();
    bad.evaluation[0].prices.remove("XLB");
    reject(bad, "INCOMPLETE_SECTOR_UNIVERSE");
    let mut bad = input.clone();
    bad.evaluation[0].prices.get_mut("XLB").unwrap().open = f64::NAN;
    reject(bad, "INVALID_SECTOR_PRICE");
    let mut bad = input.clone();
    bad.evaluation[1].date = bad.evaluation[0].date.clone();
    reject(bad, "NON_MONOTONIC");
    let mut bad = input.clone();
    bad.expected_session_dates.remove(4);
    reject(bad, "CALENDAR_MISMATCH");
    let mut bad = input.clone();
    bad.source.coverage_verified = false;
    reject(bad, "UNVERIFIED_SECTOR_SOURCE");
    let mut bad = input.clone();
    bad.source.retrieved_at = format!("{}T20:00:00Z", bad.evaluation.last().unwrap().date);
    reject(bad, "UNFINISHED_SECTOR_DAY");
    for kind in ["split", "dividend"] {
        let mut bad = input.clone();
        bad.corporate_actions.push(SectorCorporateAction {
            symbol: "XLK".into(),
            date: bad.evaluation[0].date.clone(),
            kind: kind.into(),
            split_ratio: Some(2.0),
            dividend_per_share: None,
            pay_date: None,
        });
        reject(bad, "UNSUPPORTED_SECTOR_CORPORATE_ACTION");
    }
}

#[test]
fn planner_rejects_wrong_strategy_holdings_and_previous_selection() {
    let history = fixture().history;
    let spy = SectorHolding {
        symbol: "SPY".into(),
        quantity: 1,
        average_cost_usd: 100.0,
        remaining_buy_cost_usd: 0.0,
    };
    assert!(plan_sector_portfolio(
        &history,
        SectorStrategy::Base,
        DefensiveState::Normal,
        &[spy],
        1000.0,
        &[]
    )
    .is_err());
    assert!(plan_sector_portfolio(
        &history,
        SectorStrategy::Cash,
        DefensiveState::Normal,
        &[],
        1000.0,
        &["XLK".into()]
    )
    .is_err());
}

#[test]
fn twenty_bp_is_total_stress_cost_and_not_added_to_ten_bp() {
    let mut input = fixture();
    input.cost_bps = 20.0;
    let report = preview_sector_portfolio(&input).unwrap();
    for portfolio in &report.portfolios {
        for trade in &portfolio.trades {
            close(
                trade.cost_usd,
                trade.price_usd * trade.quantity as f64 * 0.002,
            );
        }
    }
    assert!(report.warnings.iter().any(|w| w.contains("총20bp")));
}

#[test]
fn supported_extreme_quantity_is_safe_and_overflow_is_rejected_before_mutation() {
    let mut input = fixture();
    input.initial_cash_usd = 1_000_000_000.0;
    for day in input.history.iter_mut().chain(&mut input.evaluation) {
        for price in day.prices.values_mut() {
            price.adjusted_close *= 0.000_001;
            price.open = 0.000_001;
            price.close = 0.000_001;
        }
    }
    let report = preview_sector_portfolio(&input).unwrap();
    assert!(report
        .portfolios
        .iter()
        .flat_map(|p| &p.trades)
        .all(|t| t.quantity <= 9_007_199_254_740_991));
    // A huge observed mark followed by a tiny next open would overflow safe
    // JSON integer sizing; reject the whole comparison rather than saturating.
    input.evaluation[0]
        .prices
        .values_mut()
        .for_each(|p| p.close = 1_000_000_000.0);
    assert!(preview_sector_portfolio(&input)
        .unwrap_err()
        .contains("SECTOR_QUANTITY_LIMIT"));
}

#[test]
fn defensive_reduction_partially_sells_before_buys_and_reconciles_remaining_cost() {
    let mut input = fixture();
    for (index, day) in input.history.iter_mut().enumerate() {
        let value = 100.0 + index as f64 * 0.01;
        let spy = day.prices.get_mut("SPY").unwrap();
        spy.adjusted_close = value;
        spy.open = value;
        spy.close = value;
    }
    for day in &mut input.evaluation {
        let spy = day.prices.get_mut("SPY").unwrap();
        spy.adjusted_close = 80.0;
        spy.open = 80.0;
        spy.close = 80.0;
    }
    let result = preview_sector_portfolio(&input).unwrap();
    let defensive = &result.portfolios[1];
    assert_eq!(
        defensive.daily[0].plan.defensive_state,
        DefensiveState::Normal
    );
    assert_eq!(
        defensive.daily[1].plan.defensive_state,
        DefensiveState::Normal
    );
    assert_eq!(
        defensive.daily[2].plan.defensive_state,
        DefensiveState::Defensive
    );
    assert!(defensive
        .trades
        .iter()
        .any(|trade| trade.side == "sell" && trade.quantity > 0));
    for date in input.evaluation.iter().map(|row| &row.date) {
        let trades = defensive
            .trades
            .iter()
            .filter(|trade| &trade.date == date)
            .collect::<Vec<_>>();
        if let Some(first_buy) = trades.iter().position(|trade| trade.side == "buy") {
            assert!(trades[first_buy..].iter().all(|trade| trade.side == "buy"));
        }
    }
    close(
        defensive.summary.ending_equity_usd - input.initial_cash_usd,
        defensive.summary.realized_pnl_usd + defensive.summary.unrealized_pnl_usd,
    );
    assert!(defensive
        .ending_holdings
        .iter()
        .all(|row| row.remaining_buy_cost_usd >= 0.0));
}

#[test]
fn selection_change_liquidates_dropped_symbol_then_allocates_new_rank() {
    let mut input = fixture();
    let original = plan_sector_portfolio(
        &input.history,
        SectorStrategy::Base,
        DefensiveState::Normal,
        &[],
        10_000.0,
        &[],
    )
    .unwrap();
    let dropped = original.selected_symbols[0].clone();
    for day in &mut input.evaluation {
        let bar = day.prices.get_mut(&dropped).unwrap();
        bar.adjusted_close = 90.0;
        bar.close = 90.0;
    }
    let result = preview_sector_portfolio(&input).unwrap();
    let base = &result.portfolios[0];
    assert!(base.daily[0].plan.selected_symbols.contains(&dropped));
    assert!(!base.daily[1].plan.selected_symbols.contains(&dropped));
    assert!(base.daily[1]
        .plan
        .reasons
        .iter()
        .any(|r| r == "selectionChanged"));
    assert!(base
        .trades
        .iter()
        .any(|t| t.date == input.evaluation[1].date
            && t.symbol == dropped
            && t.side == "sell"
            && t.quantity > 0));
    assert!(base.ending_holdings.iter().all(|row| row.symbol != dropped));
    close(
        base.summary.ending_equity_usd - input.initial_cash_usd,
        base.summary.realized_pnl_usd + base.summary.unrealized_pnl_usd,
    );
}

#[test]
fn first_mixed_spy_pair_starts_normal_without_replaying_history_state() {
    let mut input = fixture();
    for day in &mut input.history {
        day.prices.get_mut("SPY").unwrap().adjusted_close = 100.0;
    }
    input.history[21]
        .prices
        .get_mut("SPY")
        .unwrap()
        .adjusted_close = 101.0;
    let report = preview_sector_portfolio(&input).unwrap();
    assert_eq!(
        report.portfolios[1].daily[0].plan.defensive_state,
        DefensiveState::Normal
    );
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.contains("새 독립 가상실험")));
}

#[test]
fn every_daily_nav_drawdown_and_sample_volatility_are_independently_audited() {
    let mut input = fixture();
    for (index, day) in input.evaluation.iter_mut().enumerate() {
        let multiplier = if index % 3 == 0 {
            0.9
        } else if index % 3 == 1 {
            1.05
        } else {
            1.0
        };
        for price in day.prices.values_mut() {
            price.close *= multiplier;
        }
    }
    let report = preview_sector_portfolio(&input).unwrap();
    for portfolio in &report.portfolios {
        let mut cash = input.initial_cash_usd;
        let mut positions: BTreeMap<String, u64> = BTreeMap::new();
        let mut prior = input.initial_cash_usd;
        let mut peak = prior;
        let mut max_drawdown: f64 = 0.0;
        let mut max_loss: f64 = 0.0;
        let mut returns = Vec::new();
        for (bar, output) in input.evaluation.iter().zip(&portfolio.daily) {
            for trade in portfolio
                .trades
                .iter()
                .filter(|trade| trade.date == bar.date)
            {
                let notional = trade.quantity as f64 * trade.price_usd;
                let balance = positions.entry(trade.symbol.clone()).or_default();
                if trade.side == "buy" {
                    cash -= notional + trade.cost_usd;
                    *balance += trade.quantity;
                } else {
                    cash += notional - trade.cost_usd;
                    *balance -= trade.quantity;
                }
            }
            let invested = positions
                .iter()
                .map(|(symbol, quantity)| *quantity as f64 * bar.prices[symbol].close)
                .sum::<f64>();
            let nav = cash + invested;
            let change = (nav - prior) / prior * 100.0;
            peak = peak.max(nav);
            let drawdown = (peak - nav) / peak * 100.0;
            max_drawdown = max_drawdown.max(drawdown);
            max_loss = max_loss.max(-change);
            close(output.cash_usd, cash);
            close(output.equity_usd, nav);
            close(output.daily_return_pct, change);
            close(output.drawdown_pct, drawdown);
            close(output.exposure_pct, invested / nav * 100.0);
            prior = nav;
            returns.push(change);
        }
        close(portfolio.summary.max_drawdown_pct, max_drawdown);
        close(portfolio.summary.max_daily_loss_pct, max_loss);
        let n = returns.len() as f64;
        let sum = returns.iter().sum::<f64>();
        let squares = returns.iter().map(|r| r * r).sum::<f64>();
        close(
            portfolio.summary.daily_volatility_pct,
            ((squares - sum * sum / n) / (n - 1.0)).max(0.0).sqrt(),
        );
    }
}

#[test]
fn partial_sale_allocates_actual_buy_fee_and_recovery_buys_after_two_positive_days() {
    let mut input = fixture();
    for (index, day) in input.history.iter_mut().enumerate() {
        let spy = day.prices.get_mut("SPY").unwrap();
        spy.adjusted_close = 100.0 + index as f64 * 0.01;
    }
    for (index, day) in input.evaluation.iter_mut().enumerate() {
        let spy = day.prices.get_mut("SPY").unwrap();
        spy.adjusted_close = if index < 2 { 80.0 } else { 100.5 };
    }
    input.evaluation.truncate(3);
    input.expected_session_dates.truncate(25);
    let reduced = preview_sector_portfolio(&input).unwrap();
    let book = &reduced.portfolios[1];
    assert_eq!(
        book.daily[2].plan.defensive_state,
        DefensiveState::Defensive
    );
    let mut realized = 0.0;
    for holding in &book.ending_holdings {
        let buy = book
            .trades
            .iter()
            .find(|t| t.side == "buy" && t.symbol == holding.symbol)
            .unwrap();
        let sell = book
            .trades
            .iter()
            .find(|t| t.side == "sell" && t.symbol == holding.symbol)
            .unwrap();
        assert!(sell.quantity > 0 && sell.quantity < buy.quantity);
        let allocated = buy.cost_usd * sell.quantity as f64 / buy.quantity as f64;
        close(holding.remaining_buy_cost_usd, buy.cost_usd - allocated);
        close(holding.average_cost_usd, buy.price_usd);
        realized +=
            sell.quantity as f64 * (sell.price_usd - buy.price_usd) - sell.cost_usd - allocated;
    }
    close(book.summary.realized_pnl_usd, realized);
    // The first positive SPY close is evaluation[2]. At evaluation[3] open,
    // only that one positive close is available; the pair is still mixed.
    // Recovery can first execute at evaluation[4] open, after evaluation[3]
    // has completed and supplied the second positive close.
    let full_fixture = fixture();
    for index in [3, 4] {
        let mut extra = full_fixture.evaluation[index].clone();
        extra.prices.get_mut("SPY").unwrap().adjusted_close = 100.5;
        input.expected_session_dates.push(extra.date.clone());
        input.evaluation.push(extra);
    }
    let recovered = preview_sector_portfolio(&input).unwrap();
    let book = &recovered.portfolios[1];
    assert!(recovered.rankings[3].spy_r20 > 0.0);
    assert!(recovered.rankings[3].previous_spy_r20 < 0.0);
    assert_eq!(
        book.daily[3].plan.defensive_state,
        DefensiveState::Defensive
    );
    close(book.daily[3].plan.target_weight, 0.15);
    assert!(recovered.rankings[4].spy_r20 > 0.0);
    assert!(recovered.rankings[4].previous_spy_r20 > 0.0);
    assert_eq!(book.daily[4].plan.defensive_state, DefensiveState::Normal);
    assert_eq!(book.daily[4].plan.signal_date, input.evaluation[3].date);
    close(book.daily[4].plan.target_weight, 0.3);
    assert!(book
        .trades
        .iter()
        .any(|t| t.date == input.evaluation[4].date && t.side == "buy" && t.quantity > 0));
}

#[test]
fn unaffordable_signal_is_a_zero_quantity_diagnostic_not_a_filled_trade() {
    let mut input = fixture();
    input.initial_cash_usd = 1.0;
    for day in &mut input.evaluation {
        day.prices.get_mut("SPY").unwrap().open = 1.0;
    }
    let report = preview_sector_portfolio(&input).unwrap();
    let spy = &report.portfolios[3];
    assert_eq!(spy.trades.len(), 1);
    assert_eq!(spy.trades[0].requested_quantity, 1);
    assert_eq!(spy.trades[0].quantity, 0);
    assert_eq!(spy.trades[0].status, "notAffordable");
    assert_eq!(spy.summary.filled_trade_count, 0);
    assert_eq!(spy.summary.cash_limited_count, 1);
    for book in &report.portfolios {
        close(book.summary.net_return_pct, 0.0);
        assert!(book.daily.iter().all(|row| row.cash_usd == 1.0));
        close(book.summary.total_cost_usd, 0.0);
    }
}
