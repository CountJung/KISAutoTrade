//! Completed daily history belongs to one run, config, profile and market date.
//! Fetching is owned by the daemon; this synchronous boundary never waits for I/O.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::broker::{candles::candle_market_date, BrokerId, BrokerMarket, BrokerScope};

use super::super::{state::bounded_window, FiftyTwoWeekHighParams, OhlcCandle, StrategyConfig};
use super::StrategyManager;

/// At most 511 fetch targets plus one visible capacity diagnostic are retained.
const MAX_DAILY_HISTORY_TARGETS: usize = 511;

/// Identity carried through an asynchronous provider fetch. Never contains credentials.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailyHistoryTicket {
    pub generation: u64,
    pub strategy_id: String,
    pub symbol: String,
    pub profile_id: String,
    pub broker_id: BrokerId,
    pub broker_account_id: Option<String>,
    pub config_fingerprint: String,
    pub market_date: NaiveDate,
    pub required_bars: usize,
}

/// User-visible completion state for a strategy's daily indicator history.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyHistoryStatus {
    pub strategy_id: String,
    pub symbol: String,
    pub broker_id: BrokerId,
    pub profile_id: String,
    pub market_date: String,
    pub required_bars: usize,
    pub available_bars: usize,
    pub state: String,
    pub source: Option<String>,
    pub last_completed_date: Option<String>,
    pub last_error: Option<String>,
    pub next_retry_at: Option<String>,
}

pub(super) struct DailyHistoryEntry {
    ticket: DailyHistoryTicket,
    status: DailyHistoryStatus,
}

fn fingerprint(config: &StrategyConfig) -> String {
    let bytes = serde_json::to_vec(config).unwrap_or_default();
    format!("{:x}", Sha256::digest(bytes))
}

fn pending_status(ticket: &DailyHistoryTicket) -> DailyHistoryStatus {
    DailyHistoryStatus {
        strategy_id: ticket.strategy_id.clone(),
        symbol: ticket.symbol.clone(),
        broker_id: ticket.broker_id,
        profile_id: ticket.profile_id.clone(),
        market_date: ticket.market_date.to_string(),
        required_bars: ticket.required_bars,
        available_bars: 0,
        state: "pending".into(),
        source: None,
        last_completed_date: None,
        last_error: None,
        next_retry_at: None,
    }
}

impl StrategyManager {
    pub fn history_generation(&self) -> u64 {
        self.history_generation
    }

    /// Invalidate in-flight results without resetting held positions or unrelated strategies.
    /// Identity remains stable while config mutations invalidate indicator readiness.
    pub fn advance_history_generation(&mut self) {
        self.history_generation = self.history_generation.wrapping_add(1);
        self.daily_history.clear();
        for strategy in &mut self.strategies {
            if strategy.id().starts_with("fifty_two_week_high") {
                let symbols = strategy.config().target_symbols.clone();
                for symbol in symbols {
                    strategy.initialize_ohlc(&symbol, &[]);
                }
            }
        }
    }

    pub fn set_history_execution_identity(
        &mut self,
        scope: BrokerScope,
        profile_id: Option<String>,
    ) {
        self.history_execution_identity = Some((scope, profile_id));
    }

    pub fn clear_history_execution_identity(&mut self) {
        self.history_execution_identity = None;
    }

    pub fn history_execution_identity(&self) -> Option<(BrokerScope, Option<String>)> {
        self.history_execution_identity.clone()
    }

    /// Called before tick evaluation. A new market day immediately blocks new entries;
    /// stop-loss evaluation continues while bounded background jobs refill the history.
    pub fn prepare_daily_history_targets(
        &mut self,
        scope: &BrokerScope,
        profile_id: &str,
        now: DateTime<Utc>,
    ) -> Vec<DailyHistoryTicket> {
        if let Some((run_scope, run_profile)) = &self.history_execution_identity {
            if run_scope != scope || run_profile.as_deref().unwrap_or_default() != profile_id {
                return Vec::new();
            }
        }
        let mut next = BTreeMap::new();
        let mut tickets = Vec::new();
        let mut overflow = None;
        let mut overflow_count = 0usize;
        for strategy in &mut self.strategies {
            let config = strategy.config().clone();
            if !strategy.is_enabled()
                || !config.id.starts_with("fifty_two_week_high")
                || !config.matches_scope(
                    scope.broker_id,
                    scope.account_id.as_ref().map(|id| id.0.as_str()),
                )
            {
                continue;
            }
            let required_bars = bounded_window(
                serde_json::from_value::<FiftyTwoWeekHighParams>(config.params.clone())
                    .unwrap_or_default()
                    .lookback_days,
            );
            let config_fingerprint = fingerprint(&config);
            for symbol in &config.target_symbols {
                let key = (config.id.clone(), symbol.clone());
                if next.contains_key(&key) {
                    continue;
                }
                let ticket = DailyHistoryTicket {
                    generation: self.history_generation,
                    strategy_id: config.id.clone(),
                    symbol: symbol.clone(),
                    profile_id: profile_id.into(),
                    broker_id: scope.broker_id,
                    broker_account_id: scope.account_id.as_ref().map(|id| id.0.clone()),
                    config_fingerprint: config_fingerprint.clone(),
                    market_date: candle_market_date(
                        now,
                        if crate::market_hours::is_domestic_symbol(symbol) {
                            BrokerMarket::Kr
                        } else {
                            BrokerMarket::Us
                        },
                    ),
                    required_bars,
                };
                if tickets.len() >= MAX_DAILY_HISTORY_TARGETS {
                    strategy.initialize_ohlc(symbol, &[]);
                    overflow_count += 1;
                    if overflow.is_none() {
                        let mut status = pending_status(&ticket);
                        status.state = "capacityExceeded".into();
                        status.last_error = Some(format!(
                            "완료 일봉 자동 갱신은 최대 {MAX_DAILY_HISTORY_TARGETS}개 대상입니다. 초과 대상은 신규 매수를 차단합니다."
                        ));
                        overflow = Some((key, DailyHistoryEntry { ticket, status }));
                    }
                    continue;
                }
                let entry = match self.daily_history.remove(&key) {
                    Some(entry) if entry.ticket == ticket => entry,
                    _ => {
                        strategy.initialize_ohlc(symbol, &[]);
                        DailyHistoryEntry {
                            status: pending_status(&ticket),
                            ticket: ticket.clone(),
                        }
                    }
                };
                tickets.push(ticket);
                next.insert(key, entry);
            }
        }
        if let Some((key, entry)) = overflow {
            let already_reported = self.daily_history.get(&key).is_some_and(|old| {
                old.ticket == entry.ticket && old.status.state == "capacityExceeded"
            });
            if !already_reported {
                tracing::warn!(
                    overflow_count,
                    "완료 일봉 대상 상한 초과: 초과 종목 신규 매수 차단"
                );
            }
            next.insert(key, entry);
        }
        self.daily_history = next;
        tickets
    }

    fn valid_daily_history_ticket(&self, ticket: &DailyHistoryTicket, now: DateTime<Utc>) -> bool {
        if ticket.generation != self.history_generation
            || ticket.market_date
                != candle_market_date(
                    now,
                    if crate::market_hours::is_domestic_symbol(&ticket.symbol) {
                        BrokerMarket::Kr
                    } else {
                        BrokerMarket::Us
                    },
                )
            || self
                .history_execution_identity
                .as_ref()
                .is_some_and(|(scope, profile)| {
                    scope.broker_id != ticket.broker_id
                        || scope.account_id.as_ref().map(|id| id.0.as_str())
                            != ticket.broker_account_id.as_deref()
                        || profile.as_deref().unwrap_or_default() != ticket.profile_id
                })
        {
            return false;
        }
        self.daily_history
            .get(&(ticket.strategy_id.clone(), ticket.symbol.clone()))
            .is_some_and(|entry| {
                entry.ticket == *ticket && entry.status.state != "capacityExceeded"
            })
            && self.strategies.iter().any(|strategy| {
                let config = strategy.config();
                strategy.is_enabled()
                    && config.id == ticket.strategy_id
                    && config.targets_symbol(&ticket.symbol)
                    && config.matches_scope(ticket.broker_id, ticket.broker_account_id.as_deref())
                    && fingerprint(config) == ticket.config_fingerprint
            })
    }

    /// Only completed, chronologically ordered OHLC belongs here. The collector validates
    /// dates; unlike startup warmup, this method never drops the last completed bar.
    pub fn apply_daily_history(
        &mut self,
        ticket: &DailyHistoryTicket,
        candles: &[OhlcCandle],
        source: String,
        last_completed_date: Option<String>,
        now: DateTime<Utc>,
    ) -> bool {
        // A missing-profile diagnostic ticket can report failure but never accept data.
        if ticket.profile_id.is_empty() || !self.valid_daily_history_ticket(ticket, now) {
            return false;
        }
        // The current response timestamp does not make old candles current. This
        // conservative bound also leaves exceptional long holidays blocked for review.
        let completed_date = last_completed_date.as_deref().and_then(|date| {
            NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(date, "%Y%m%d"))
                .ok()
        });
        let fresh = completed_date.is_some_and(|date| {
            let days = ticket.market_date.signed_duration_since(date).num_days();
            (1..=10).contains(&days)
        });
        let valid = candles[candles.len().saturating_sub(ticket.required_bars)..]
            .iter()
            .copied()
            .filter(|c| {
                fresh && c.low > 0 && c.high >= c.open.max(c.close) && c.low <= c.open.min(c.close)
            })
            .collect::<Vec<_>>();
        let strategy = self
            .strategies
            .iter_mut()
            .find(|strategy| strategy.id() == ticket.strategy_id);
        let Some(strategy) = strategy else {
            return false;
        };
        strategy.initialize_ohlc(&ticket.symbol, &valid);
        let readiness = strategy.history_readiness(&ticket.symbol);
        let entry = self
            .daily_history
            .get_mut(&(ticket.strategy_id.clone(), ticket.symbol.clone()));
        let Some(entry) = entry else { return false };
        entry.status.available_bars = readiness.map_or(0, |state| state.available_bars);
        entry.status.state = if readiness.is_some_and(|state| state.ready) {
            "ready"
        } else {
            "insufficient"
        }
        .into();
        entry.status.source = Some(source.chars().take(128).collect());
        entry.status.last_completed_date = completed_date.map(|date| date.to_string());
        entry.status.last_error = if !fresh {
            Some("최신 완료 일봉 날짜를 확인하지 못했거나 시장 날짜보다 10일을 초과해 오래되었습니다. 신규 매수는 차단하며 보유 손절은 유지합니다.".into())
        } else {
            (entry.status.state != "ready").then(|| {
                format!(
                    "완료 일봉 부족: {}/{}봉. 신규 매수는 차단하며 보유 손절은 유지합니다.",
                    entry.status.available_bars, ticket.required_bars
                )
            })
        };
        entry.status.next_retry_at = None;
        true
    }

    pub fn mark_daily_history_failure(
        &mut self,
        ticket: &DailyHistoryTicket,
        error: &str,
        next_retry_at: Option<String>,
        now: DateTime<Utc>,
    ) -> bool {
        if !self.valid_daily_history_ticket(ticket, now) {
            return false;
        }
        if let Some(strategy) = self
            .strategies
            .iter_mut()
            .find(|strategy| strategy.id() == ticket.strategy_id)
        {
            strategy.initialize_ohlc(&ticket.symbol, &[]);
        }
        let entry = self
            .daily_history
            .get_mut(&(ticket.strategy_id.clone(), ticket.symbol.clone()));
        let Some(entry) = entry else { return false };
        entry.status.available_bars = 0;
        entry.status.state = "failed".into();
        entry.status.last_error = Some(error.chars().take(1024).collect());
        entry.status.next_retry_at = next_retry_at;
        true
    }

    pub fn mark_daily_history_pending(
        &mut self,
        ticket: &DailyHistoryTicket,
        now: DateTime<Utc>,
    ) -> bool {
        if !self.valid_daily_history_ticket(ticket, now) {
            return false;
        }
        if let Some(strategy) = self
            .strategies
            .iter_mut()
            .find(|strategy| strategy.id() == ticket.strategy_id)
        {
            strategy.initialize_ohlc(&ticket.symbol, &[]);
        }
        let entry = self
            .daily_history
            .get_mut(&(ticket.strategy_id.clone(), ticket.symbol.clone()));
        let Some(entry) = entry else { return false };
        entry.status.available_bars = 0;
        entry.status.state = "pending".into();
        entry.status.next_retry_at = None;
        true
    }

    pub fn set_daily_history_retry(
        &mut self,
        ticket: &DailyHistoryTicket,
        next_retry_at: Option<String>,
    ) -> bool {
        let Some(entry) = self
            .daily_history
            .get_mut(&(ticket.strategy_id.clone(), ticket.symbol.clone()))
        else {
            return false;
        };
        if entry.ticket != *ticket || ticket.generation != self.history_generation {
            return false;
        }
        entry.status.next_retry_at = next_retry_at;
        true
    }

    pub fn set_daily_history_collection_reason(
        &mut self,
        ticket: &DailyHistoryTicket,
        reason: &str,
        now: DateTime<Utc>,
    ) -> bool {
        if !self.valid_daily_history_ticket(ticket, now) {
            return false;
        }
        let Some(entry) = self
            .daily_history
            .get_mut(&(ticket.strategy_id.clone(), ticket.symbol.clone()))
        else {
            return false;
        };
        if entry.status.state == "insufficient" {
            let message = format!(
                "{} ({reason})",
                entry
                    .status
                    .last_error
                    .as_deref()
                    .unwrap_or("완료 일봉 부족")
            );
            entry.status.last_error = Some(message.chars().take(1024).collect());
        }
        true
    }

    /// A quote request may cross midnight after preparation. Recheck under the same
    /// manager lock as tick evaluation so yesterday's indicator cannot create a buy.
    pub fn invalidate_stale_daily_history_for_symbol(&mut self, symbol: &str, now: DateTime<Utc>) {
        let today = candle_market_date(
            now,
            if crate::market_hours::is_domestic_symbol(symbol) {
                BrokerMarket::Kr
            } else {
                BrokerMarket::Us
            },
        );
        let reason = format!("시장 날짜가 {today}로 변경되어 완료 일봉을 다시 준비합니다.");
        for strategy in &mut self.strategies {
            let key = (strategy.id().to_string(), symbol.to_string());
            let Some(entry) = self.daily_history.get_mut(&key) else {
                continue;
            };
            if entry.status.state != "capacityExceeded"
                && entry.ticket.market_date != today
                && entry.status.last_error.as_deref() != Some(reason.as_str())
            {
                strategy.initialize_ohlc(symbol, &[]);
                entry.status.state = "pending".into();
                entry.status.available_bars = 0;
                entry.status.last_error = Some(reason.clone());
                entry.status.next_retry_at = None;
            }
        }
    }

    pub fn daily_history_status(&self) -> Vec<DailyHistoryStatus> {
        self.daily_history
            .values()
            .map(|entry| entry.status.clone())
            .collect()
    }

    pub fn is_daily_history_ready(&self, ticket: &DailyHistoryTicket) -> bool {
        self.daily_history
            .get(&(ticket.strategy_id.clone(), ticket.symbol.clone()))
            .is_some_and(|entry| entry.ticket == *ticket && entry.status.state == "ready")
    }
}

#[cfg(test)]
#[path = "live_history_tests.rs"]
mod tests;
