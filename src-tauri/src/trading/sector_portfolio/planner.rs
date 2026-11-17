use super::*;
use std::collections::BTreeSet;

fn return_n(history: &[SectorDay], symbol: &str, days: usize, offset: usize) -> f64 {
    let end = history.len() - 1 - offset;
    history[end].prices[symbol].adjusted_close / history[end - days].prices[symbol].adjusted_close
        - 1.0
}

pub(super) fn spy_returns(history: &[SectorDay]) -> (f64, f64) {
    (
        return_n(history, "SPY", 20, 0),
        return_n(history, "SPY", 20, 1),
    )
}

/// Rank the fixed eleven-sector universe before filtering eligibility. Tied RS
/// values receive their average rank; SPY never enters the ranking population.
pub fn rank_sectors(history: &[SectorDay]) -> Result<Vec<SectorRank>, String> {
    validation::history(history)?;
    let spy5 = return_n(history, "SPY", 5, 0);
    let spy20 = return_n(history, "SPY", 20, 0);
    let mut rows: Vec<_> = SECTOR_SYMBOLS
        .iter()
        .map(|symbol| {
            let r5 = return_n(history, symbol, 5, 0);
            let r20 = return_n(history, symbol, 20, 0);
            SectorRank {
                symbol: (*symbol).into(),
                r5,
                r20,
                rs5: r5 - spy5,
                rs20: r20 - spy20,
                percentile5: 0.0,
                percentile20: 0.0,
                score: 0.0,
                eligible: r20 > 0.0 && r20 - spy20 > 0.0,
            }
        })
        .collect();
    for period in [5, 20] {
        let mut indices: Vec<_> = (0..rows.len()).collect();
        let value = |row: &SectorRank| if period == 5 { row.rs5 } else { row.rs20 };
        indices.sort_by(|&a, &b| value(&rows[a]).total_cmp(&value(&rows[b])));
        let mut begin = 0;
        while begin < indices.len() {
            let mut end = begin + 1;
            while end < indices.len() && value(&rows[indices[end]]) == value(&rows[indices[begin]])
            {
                end += 1;
            }
            // One-based average rank minus one, divided by N-1=10.
            let percentile = (begin + end - 1) as f64 / 20.0;
            for index in &indices[begin..end] {
                if period == 5 {
                    rows[*index].percentile5 = percentile;
                } else {
                    rows[*index].percentile20 = percentile;
                }
            }
            begin = end;
        }
    }
    for row in &mut rows {
        row.score = (row.percentile5 + row.percentile20) / 2.0;
    }
    rows.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.rs20.total_cmp(&a.rs20))
            .then_with(|| a.symbol.cmp(&b.symbol))
    });
    Ok(rows)
}

/// Reusable preopen allocation planner. Only completed history and confirmed
/// holdings/cash enter this function; execution prices are deliberately absent.
pub fn plan_sector_portfolio(
    history: &[SectorDay],
    strategy: SectorStrategy,
    previous_state: DefensiveState,
    holdings: &[SectorHolding],
    cash_usd: f64,
    previous_selection: &[String],
) -> Result<SectorPlan, String> {
    let ranking = rank_sectors(history)?;
    validation::holdings(holdings, cash_usd)?;
    if previous_selection
        .iter()
        .any(|symbol| !ALL_SYMBOLS.contains(&symbol.as_str()))
        || previous_selection.iter().collect::<BTreeSet<_>>().len() != previous_selection.len()
    {
        return Err("INVALID_SECTOR_PREVIOUS_SELECTION".into());
    }
    let allowed_symbol = |symbol: &str| match strategy {
        SectorStrategy::Spy => symbol == "SPY",
        SectorStrategy::Cash => false,
        _ => SECTOR_SYMBOLS.contains(&symbol),
    };
    let maximum_previous = match strategy {
        SectorStrategy::Spy => 1,
        SectorStrategy::Cash => 0,
        SectorStrategy::Aggressive => 2,
        _ => 3,
    };
    if holdings
        .iter()
        .any(|row| row.quantity > 0 && !allowed_symbol(&row.symbol))
        || previous_selection
            .iter()
            .any(|symbol| !allowed_symbol(symbol))
        || previous_selection.len() > maximum_previous
    {
        return Err("SECTOR_STRATEGY_HOLDING_MISMATCH".into());
    }
    let (current, previous) = spy_returns(history);
    let state = if strategy == SectorStrategy::Defensive {
        if current <= 0.0 && previous <= 0.0 {
            DefensiveState::Defensive
        } else if current > 0.0 && previous > 0.0 {
            DefensiveState::Normal
        } else {
            previous_state
        }
    } else {
        DefensiveState::Normal
    };
    let (maximum, weight) = match strategy {
        SectorStrategy::Base => (3, 0.3),
        SectorStrategy::Defensive => (
            3,
            if state == DefensiveState::Defensive {
                0.15
            } else {
                0.3
            },
        ),
        SectorStrategy::Aggressive => (2, 0.475),
        SectorStrategy::Spy => (1, 1.0),
        SectorStrategy::Cash => (0, 0.0),
    };
    let selected: Vec<String> = if strategy == SectorStrategy::Spy {
        vec!["SPY".into()]
    } else {
        ranking
            .iter()
            .filter(|row| row.eligible)
            .take(maximum)
            .map(|row| row.symbol.clone())
            .collect()
    };
    let last = history.last().expect("validated history");
    let equity = cash_usd
        + holdings
            .iter()
            .map(|row| row.quantity as f64 * last.prices[&row.symbol].close)
            .sum::<f64>();
    if !equity.is_finite() || equity <= 0.0 {
        return Err("INVALID_SECTOR_EQUITY".into());
    }
    let symbols: &[&str] = if strategy == SectorStrategy::Spy {
        &["SPY"]
    } else {
        &SECTOR_SYMBOLS
    };
    let targets: Vec<_> = symbols
        .iter()
        .map(|symbol| {
            let quantity = holdings
                .iter()
                .find(|row| row.symbol == *symbol)
                .map_or(0, |row| row.quantity);
            let target_weight = if selected.iter().any(|item| item == symbol) {
                weight
            } else {
                0.0
            };
            SectorTarget {
                symbol: (*symbol).into(),
                weight: target_weight,
                budget_usd: equity * target_weight,
                reference_weight: quantity as f64 * last.prices[*symbol].close / equity,
            }
        })
        .collect();
    let set = |symbols: &[String]| symbols.iter().cloned().collect::<BTreeSet<_>>();
    let mut reasons = Vec::new();
    if set(&selected) != set(previous_selection) {
        reasons.push("selectionChanged".into());
    }
    if strategy == SectorStrategy::Defensive && state != previous_state {
        reasons.push("defensiveStateChanged".into());
    }
    // A benchmark is buy-and-hold after its first allocation, not a daily rebalance.
    if !matches!(strategy, SectorStrategy::Spy | SectorStrategy::Cash)
        && targets
            .iter()
            .any(|row| (row.reference_weight - row.weight).abs() >= 0.05 - 1e-12)
    {
        reasons.push("weightDriftAtLeast5pp".into());
    }
    Ok(SectorPlan {
        strategy,
        signal_date: last.date.clone(),
        defensive_state: state,
        selected_symbols: selected,
        target_weight: weight,
        reference_equity_usd: equity,
        rebalance: !reasons.is_empty(),
        reasons,
        targets,
    })
}
