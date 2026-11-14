use super::types::*;
use crate::trading::sector_portfolio::{validation, ALL_SYMBOLS};
use chrono::{DateTime, Utc};

fn text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 512 && !value.chars().any(char::is_control)
}

pub(super) fn input(input: &BrokerChartInput) -> Result<(), String> {
    if input.schema_version != 1 {
        return Err("INVALID_BROKER_CHART_SCHEMA".into());
    }
    if !input.acknowledged_price_only {
        return Err("BROKER_CHART_PRICE_ONLY_ACK_REQUIRED".into());
    }
    if !input.known_corporate_actions.is_empty() {
        return Err("UNSUPPORTED_BROKER_CHART_CORPORATE_ACTION".into());
    }
    if !input.initial_cash_usd.is_finite()
        || !(1.0..=1_000_000_000.0).contains(&input.initial_cash_usd)
        || !matches!(input.cost_bps, 10.0 | 20.0)
    {
        return Err("INVALID_BROKER_CHART_ASSUMPTIONS".into());
    }
    let count = input.history.len().saturating_add(input.evaluation.len());
    if input.history.len() < 22
        || input.evaluation.is_empty()
        || count > 500
        || input.expected_session_dates.len() != count
    {
        return Err("INVALID_BROKER_CHART_WINDOW".into());
    }
    if !text(&input.source.name)
        || !text(&input.source.calendar_evidence)
        || input.source.retrieved_at.len() > 64
    {
        return Err("INVALID_BROKER_CHART_SOURCE".into());
    }
    let retrieved = DateTime::parse_from_rfc3339(&input.source.retrieved_at)
        .map_err(|_| "INVALID_BROKER_CHART_RETRIEVAL_TIME".to_string())?;
    let now = Utc::now();
    if retrieved.with_timezone(&Utc) > now {
        return Err("FUTURE_BROKER_CHART_RETRIEVAL".into());
    }
    let ny = chrono_tz::America::New_York;
    let cutoff = retrieved
        .with_timezone(&ny)
        .date_naive()
        .min(now.with_timezone(&ny).date_naive());
    let mut previous = None;
    for (index, day) in input.history.iter().chain(&input.evaluation).enumerate() {
        let date =
            validation::date(&day.date).map_err(|_| "INVALID_BROKER_CHART_DATE".to_string())?;
        if previous.is_some_and(|last| date <= last) {
            return Err("DUPLICATE_OR_NON_MONOTONIC_BROKER_CHART_DATE".into());
        }
        if date >= cutoff {
            return Err("UNFINISHED_BROKER_CHART_DAY".into());
        }
        previous = Some(date);
        if day.date != input.expected_session_dates[index] {
            return Err("BROKER_CHART_SESSION_MISMATCH".into());
        }
        if day.prices.len() != ALL_SYMBOLS.len()
            || ALL_SYMBOLS
                .iter()
                .any(|symbol| !day.prices.contains_key(*symbol))
        {
            return Err("INCOMPLETE_BROKER_CHART_UNIVERSE".into());
        }
        for bar in day.prices.values() {
            if [bar.open, bar.high, bar.low, bar.close]
                .iter()
                .any(|price| !price.is_finite() || !(0.000_001..=1_000_000_000.0).contains(price))
                || !bar.volume.is_finite()
                || !(0.0..=9_007_199_254_740_991.0).contains(&bar.volume)
                || bar.high < bar.open.max(bar.close)
                || bar.low > bar.open.min(bar.close)
                || bar.low > bar.high
            {
                return Err("INVALID_BROKER_CHART_OHLCV".into());
            }
        }
    }
    Ok(())
}
