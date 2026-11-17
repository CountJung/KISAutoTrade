//! Pure preflight only: no fills, persistence, broker calls or order registration.
//! Caller-supplied confirmed event keys are not durable idempotency guarantees.
use super::{DefensiveState, SectorHolding, SectorPlan, SectorStrategy, SECTOR_SYMBOLS};
use crate::trading::strategy::lth_research_session;
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::America::New_York;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const MAX_QUANTITY: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartProvider {
    Kis,
    Toss,
}

/// Source timestamps and delays are retained, never promoted to realtime fills.
#[derive(Debug, Clone)]
pub struct DisplayObservation {
    pub symbol: String,
    pub price_usd: f64,
    pub provider: ChartProvider,
    pub source_url: String,
    pub retrieved_at: DateTime<Utc>,
    pub quoted_at: Option<DateTime<Utc>>,
    pub provider_date: Option<NaiveDate>,
    pub delayed: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct LockedAccountPlan {
    pub account_id: String,
    pub decision_key: String,
    pub plan_version: String,
    pub locked_plan_hash: String,
    pub locked_at: DateTime<Utc>,
    pub cash_usd: f64,
    pub holdings: Vec<SectorHolding>,
    pub reference_closes_usd: BTreeMap<String, f64>,
    pub signal_input_hash: String,
    pub plan: SectorPlan,
}

#[derive(Debug, Clone)]
pub struct ExecutionPreflight {
    pub trading_date: NaiveDate,
    pub execute_at: DateTime<Utc>,
    pub accounts: Vec<LockedAccountPlan>,
    /// Acquisition order. The first valid observation for each required ticker wins.
    pub observations: Vec<DisplayObservation>,
    pub confirmed_event_keys: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct ObservationTiming {
    pub retrieved_utc: String,
    pub retrieved_new_york: String,
    pub quoted_utc: Option<String>,
    pub quoted_new_york: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedExecution {
    pub batch_key: String,
    pub accounts: Vec<LockedAccountPlan>,
    pub observations: BTreeMap<String, DisplayObservation>,
    pub observation_times: BTreeMap<String, ObservationTiming>,
    pub execution_utc: String,
    pub execution_new_york: String,
    pub observation_span_milliseconds: i64,
    pub warnings: Vec<String>,
    pub calendar_source: String,
}

/// Integrity fingerprint only, not proof the plan was externally saved before open.
pub fn locked_plan_hash(account: &LockedAccountPlan) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(
        &account.account_id,
        &account.decision_key,
        &account.plan_version,
        account.locked_at.to_rfc3339(),
        account.cash_usd,
        &account.holdings,
        &account.reference_closes_usd,
        &account.signal_input_hash,
        &account.plan,
    ))
    .map_err(|e| e.to_string())?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn expected_id(strategy: SectorStrategy) -> Option<&'static str> {
    match strategy {
        SectorStrategy::Base => Some("compare-base-v1"),
        SectorStrategy::Defensive => Some("compare-defensive-v1"),
        SectorStrategy::Aggressive => Some("compare-aggressive-v1"),
        _ => None,
    }
}
fn valid_amount(value: f64) -> bool {
    value.is_finite() && (0.0..=1_000_000_000.0).contains(&value)
}
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-8 * b.abs().max(1.0)
}
fn previous_session(date: NaiveDate) -> Option<NaiveDate> {
    (1..=7).find_map(|back| {
        let prior = date.checked_sub_signed(Duration::days(back))?;
        let noon = New_York
            .from_local_datetime(&prior.and_hms_opt(12, 0, 0)?)
            .single()?
            .to_utc();
        lth_research_session::quote_clock(noon).ok().map(|_| prior)
    })
}

/// Validates all three books before returning a shared immutable observation set.
/// Errors leave the input untouched. This does not execute, reserve or save anything.
pub fn prepare_execution(input: &ExecutionPreflight) -> Result<PreparedExecution, String> {
    let clock = lth_research_session::quote_clock(input.execute_at)
        .map_err(|e| format!("SECTOR_EXECUTION_SESSION: {e:?}"))?;
    if clock.date != input.trading_date
        || input.accounts.len() != 3
        || input.observations.len() > 1024
        || input.confirmed_event_keys.len() > 10_000
    {
        return Err("SECTOR_EXECUTION_INPUT: 날짜/계정수/입력 상한 오류".into());
    }
    let date = input.trading_date;
    if !(NaiveDate::from_ymd_opt(2026, 11, 6).unwrap()
        ..=NaiveDate::from_ymd_opt(2026, 11, 24).unwrap())
        .contains(&date)
    {
        return Err("SECTOR_EXECUTION_EXPERIMENT_WINDOW".into());
    }
    let prefix = format!("sector-paper/comparison-20261106/{date}");
    let batch_key = format!("{prefix}/execution-batch");
    if input.confirmed_event_keys.contains(&batch_key) {
        return Err("SECTOR_EXECUTION_DUPLICATE: 이미 확정된 묶음".into());
    }
    let prior = previous_session(date).ok_or("SECTOR_EXECUTION_CALENDAR: 직전 거래일 확인 불가")?;
    let mut ids = BTreeSet::new();
    let mut required = BTreeSet::new();
    let mut common_reference_prices = BTreeMap::new();
    for account in &input.accounts {
        let plan = &account.plan;
        if account.plan_version != "sector-paper-plan-v1"
            || account.locked_plan_hash != locked_plan_hash(account)?
        {
            return Err("SECTOR_EXECUTION_PLAN_INTEGRITY: 버전/고정 계획 hash 불일치".into());
        }
        let id = expected_id(plan.strategy).ok_or("SECTOR_EXECUTION_ACCOUNT: 비교 전략만 지원")?;
        if account.account_id != id
            || !ids.insert(id)
            || account.decision_key != format!("{prefix}/{id}/preopen-decision")
        {
            return Err("SECTOR_EXECUTION_PLAN_ID: 계획/계정 ID 불일치".into());
        }
        if ["fill", "execution-check"].iter().any(|suffix| {
            input
                .confirmed_event_keys
                .contains(&format!("{prefix}/{id}/{suffix}"))
        }) {
            return Err(
                "SECTOR_EXECUTION_PARTIAL_OR_DUPLICATE: 확정 계정 발견, 별도 복구 필요".into(),
            );
        }
        let lock = account.locked_at.with_timezone(&New_York);
        let open = New_York
            .from_local_datetime(&date.and_hms_opt(9, 30, 0).ok_or("SECTOR_EXECUTION_DATE")?)
            .single()
            .ok_or("SECTOR_EXECUTION_DATE")?;
        if lock.date_naive() != date || lock >= open || plan.signal_date != prior.to_string() {
            return Err("SECTOR_EXECUTION_LATE_PLAN: 당일 개장 전·직전 완료일 계획 필요".into());
        }
        let weight = match plan.strategy {
            SectorStrategy::Aggressive => 0.475,
            SectorStrategy::Defensive if plan.defensive_state == DefensiveState::Defensive => 0.15,
            _ => 0.30,
        };
        let limit = if plan.strategy == SectorStrategy::Aggressive {
            2
        } else {
            3
        };
        if !valid_amount(account.cash_usd)
            || !valid_amount(plan.reference_equity_usd)
            || plan.reference_equity_usd <= 0.0
            || !near(plan.target_weight, weight)
            || plan.selected_symbols.len() > limit
        {
            return Err("SECTOR_EXECUTION_BUDGET: 현금/NAV/고정배분 오류".into());
        }
        let selected: BTreeSet<_> = plan.selected_symbols.iter().map(String::as_str).collect();
        if selected.len() != plan.selected_symbols.len()
            || selected.iter().any(|s| !SECTOR_SYMBOLS.contains(s))
            || plan.targets.len() != SECTOR_SYMBOLS.len()
        {
            return Err("SECTOR_EXECUTION_TARGET: 종목/슬롯 오류".into());
        }
        let mut targets = BTreeSet::new();
        for target in &plan.targets {
            let w = if selected.contains(target.symbol.as_str()) {
                weight
            } else {
                0.0
            };
            if (w == 0.0 && (target.weight != 0.0 || target.budget_usd != 0.0))
                || !SECTOR_SYMBOLS.contains(&target.symbol.as_str())
                || !targets.insert(target.symbol.as_str())
                || !valid_amount(target.budget_usd)
                || !near(target.weight, w)
                || !near(target.budget_usd, plan.reference_equity_usd * w)
                || !target.reference_weight.is_finite()
                || !(0.0..=1.0).contains(&target.reference_weight)
            {
                return Err("SECTOR_EXECUTION_TARGET: 잠긴 예산 불일치".into());
            }
        }
        let mut held = BTreeSet::new();
        for holding in &account.holdings {
            if !SECTOR_SYMBOLS.contains(&holding.symbol.as_str())
                || !held.insert(holding.symbol.as_str())
                || holding.quantity == 0
                || holding.quantity > MAX_QUANTITY
                || !valid_amount(holding.average_cost_usd)
                || holding.average_cost_usd <= 0.0
                || !valid_amount(holding.remaining_buy_cost_usd)
            {
                return Err("SECTOR_EXECUTION_HOLDING: 보유/원가/정수 범위 오류".into());
            }
        }
        if account.signal_input_hash.len() != 64
            || !account
                .signal_input_hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
            || account.signal_input_hash != input.accounts[0].signal_input_hash
        {
            return Err("SECTOR_EXECUTION_SIGNAL_INPUT: 공통 신호 입력 hash 필요".into());
        }
        if account.reference_closes_usd.len() != held.len()
            || account.reference_closes_usd.iter().any(|(s, p)| {
                !held.contains(s.as_str()) || !p.is_finite() || *p <= 0.0 || *p > 1_000_000_000.0
            })
        {
            return Err("SECTOR_EXECUTION_REFERENCE_PRICE".into());
        }
        for (symbol, price) in &account.reference_closes_usd {
            if common_reference_prices
                .insert(symbol.clone(), *price)
                .is_some_and(|previous| previous != *price)
            {
                return Err("SECTOR_EXECUTION_COMMON_REFERENCE_PRICE".into());
            }
        }
        let reference_nav = account.cash_usd
            + account
                .holdings
                .iter()
                .map(|h| h.quantity as f64 * account.reference_closes_usd[&h.symbol])
                .sum::<f64>();
        if !near(plan.reference_equity_usd, reference_nav) {
            return Err("SECTOR_EXECUTION_REFERENCE_NAV".into());
        }
        for target in &plan.targets {
            let value = account
                .holdings
                .iter()
                .find(|h| h.symbol == target.symbol)
                .map_or(0.0, |h| {
                    h.quantity as f64 * account.reference_closes_usd[&h.symbol]
                });
            if !near(target.reference_weight, value / reference_nav) {
                return Err("SECTOR_EXECUTION_REFERENCE_WEIGHT".into());
            }
        }
        if plan.rebalance {
            required.extend(selected.into_iter().map(str::to_string));
            required.extend(held.into_iter().map(str::to_string));
        }
    }
    let base = input
        .accounts
        .iter()
        .find(|a| a.plan.strategy == SectorStrategy::Base)
        .ok_or("SECTOR_EXECUTION_ACCOUNT")?;
    let defensive = input
        .accounts
        .iter()
        .find(|a| a.plan.strategy == SectorStrategy::Defensive)
        .ok_or("SECTOR_EXECUTION_ACCOUNT")?;
    let aggressive = input
        .accounts
        .iter()
        .find(|a| a.plan.strategy == SectorStrategy::Aggressive)
        .ok_or("SECTOR_EXECUTION_ACCOUNT")?;
    if base.plan.selected_symbols != defensive.plan.selected_symbols
        || aggressive.plan.selected_symbols
            != base
                .plan
                .selected_symbols
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
        || base.plan.defensive_state != DefensiveState::Normal
        || aggressive.plan.defensive_state != DefensiveState::Normal
    {
        return Err("SECTOR_EXECUTION_COMMON_SELECTION".into());
    }
    let mut observations = BTreeMap::new();
    for observation in &input.observations {
        if !required.contains(&observation.symbol) || observations.contains_key(&observation.symbol)
        {
            continue;
        }
        let retrieval_ok = lth_research_session::quote_clock(observation.retrieved_at)
            .is_ok_and(|c| c.date == date);
        let quote_ok = observation.quoted_at.is_none_or(|t| {
            t <= observation.retrieved_at
                && lth_research_session::quote_clock(t).is_ok_and(|c| c.date == date)
        });
        let source_ok = reqwest::Url::parse(&observation.source_url).is_ok_and(|url| {
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        });
        if !observation.price_usd.is_finite()
            || !(0.000_001..=1_000_000_000.0).contains(&observation.price_usd)
            || !source_ok
            || observation.source_url.len() > 2048
            || !retrieval_ok
            || observation.retrieved_at > input.execute_at
            || !quote_ok
            || observation.provider_date.is_some_and(|d| d != date)
        {
            continue;
        }
        observations.insert(observation.symbol.clone(), observation.clone());
    }
    let missing: Vec<_> = required
        .difference(&observations.keys().cloned().collect())
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "SECTOR_EXECUTION_MISSING_PRICES: {}",
            missing.join(",")
        ));
    }
    for account in &input.accounts {
        if !account.plan.rebalance {
            continue;
        }
        for target in &account.plan.targets {
            if account.plan.selected_symbols.contains(&target.symbol) {
                let observation = observations
                    .get(&target.symbol)
                    .ok_or("SECTOR_EXECUTION_MISSING_SELECTED_PRICE")?;
                let quantity = (target.budget_usd / observation.price_usd).floor();
                if !quantity.is_finite() || quantity > MAX_QUANTITY as f64 {
                    return Err("SECTOR_EXECUTION_QUANTITY_LIMIT".into());
                }
            }
        }
    }
    let first = observations.values().map(|o| o.retrieved_at).min();
    let last = observations.values().map(|o| o.retrieved_at).max();
    let mut warnings = vec!["준비 검증만 수행했습니다. 계획 저장 시점/출처 진위·영속 멱등성·실제 체결을 인증하지 않습니다.".into()];
    for (symbol, observation) in &observations {
        if observation.quoted_at.is_none() {
            warnings.push(format!("{symbol}: 공급자 시세 시각 미확인"));
        }
        if observation.delayed.is_none() {
            warnings.push(format!("{symbol}: 지연 여부 미확인"));
        }
        if observation.delayed == Some(true) {
            warnings.push(format!("{symbol}: 공급자 지연 표시가격"));
        }
    }
    let observation_times = observations
        .iter()
        .map(|(symbol, o)| {
            (
                symbol.clone(),
                ObservationTiming {
                    retrieved_utc: o.retrieved_at.to_rfc3339(),
                    retrieved_new_york: o.retrieved_at.with_timezone(&New_York).to_rfc3339(),
                    quoted_utc: o.quoted_at.map(|t| t.to_rfc3339()),
                    quoted_new_york: o.quoted_at.map(|t| t.with_timezone(&New_York).to_rfc3339()),
                },
            )
        })
        .collect();
    Ok(PreparedExecution {
        batch_key,
        accounts: input.accounts.clone(),
        observations,
        observation_times,
        execution_utc: input.execute_at.to_rfc3339(),
        execution_new_york: input.execute_at.with_timezone(&New_York).to_rfc3339(),
        observation_span_milliseconds: first
            .zip(last)
            .map_or(0, |(a, b)| (b - a).num_milliseconds()),
        warnings,
        calendar_source: lth_research_session::CALENDAR_SOURCE.into(),
    })
}

#[cfg(test)]
mod tests;
