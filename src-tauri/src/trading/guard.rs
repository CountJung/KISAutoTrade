use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Duration, Local, NaiveDate};

use crate::broker::BrokerScope;
use crate::trading::strategy::Signal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuardSide {
    Buy,
    Sell,
}

#[derive(Debug)]
pub enum GuardDecision {
    Allow,
    Block { reason: String },
}

#[derive(Debug, Clone)]
pub struct TradeGuardConfig {
    pub same_side_cooldown_min: i64,
    pub after_sell_buy_cooldown_min: i64,
    pub whipsaw_window_min: i64,
    pub whipsaw_cooldown_min: i64,
    pub max_opposite_signals: usize,
    pub min_expected_profit_bps: i32,
    pub domestic_slippage_bps: i32,
    pub overseas_slippage_bps: i32,
    pub overseas_fee_bps: i32,
}

impl Default for TradeGuardConfig {
    fn default() -> Self {
        Self {
            same_side_cooldown_min: 30,
            after_sell_buy_cooldown_min: 60,
            whipsaw_window_min: 15,
            whipsaw_cooldown_min: 30,
            max_opposite_signals: 2,
            min_expected_profit_bps: 0,
            domestic_slippage_bps: 5,
            overseas_slippage_bps: 10,
            overseas_fee_bps: 10,
        }
    }
}

pub struct TradeGuard {
    config: TradeGuardConfig,
    last_order_at: HashMap<(BrokerScope, String, GuardSide), DateTime<Local>>,
    recent_sides: HashMap<(BrokerScope, String), VecDeque<(DateTime<Local>, GuardSide)>>,
    cooldown_until: HashMap<(BrokerScope, String), DateTime<Local>>,
    stop_loss_block_date: HashMap<(BrokerScope, String), NaiveDate>,
}

impl TradeGuard {
    pub fn new(config: TradeGuardConfig) -> Self {
        Self {
            config,
            last_order_at: HashMap::new(),
            recent_sides: HashMap::new(),
            cooldown_until: HashMap::new(),
            stop_loss_block_date: HashMap::new(),
        }
    }

    pub fn evaluate(
        &mut self,
        signal: &Signal,
        held_quantity: u64,
        avg_price: Option<u64>,
        tick_price: u64,
        is_overseas: bool,
    ) -> GuardDecision {
        self.evaluate_for_scope(
            &BrokerScope::kis_legacy(),
            signal,
            held_quantity,
            avg_price,
            tick_price,
            is_overseas,
        )
    }

    pub fn evaluate_for_scope(
        &mut self,
        scope: &BrokerScope,
        signal: &Signal,
        held_quantity: u64,
        avg_price: Option<u64>,
        tick_price: u64,
        is_overseas: bool,
    ) -> GuardDecision {
        let (symbol, side) = match signal {
            Signal::Buy { symbol, .. } => (symbol, GuardSide::Buy),
            Signal::Sell { symbol, .. } => (symbol, GuardSide::Sell),
            Signal::Hold => return GuardDecision::Allow,
        };

        let now = Local::now();
        self.prune_recent(scope, symbol, now);

        let symbol_key = (scope.clone(), symbol.clone());
        if let Some(until) = self.cooldown_until.get(&symbol_key) {
            if *until > now {
                return GuardDecision::Block {
                    reason: format!("휩소 쿨다운 중: {}까지", until.format("%H:%M:%S")),
                };
            }
        }

        if side == GuardSide::Buy {
            let today = now.date_naive();
            if self.stop_loss_block_date.get(&symbol_key) == Some(&today) {
                return GuardDecision::Block {
                    reason: "손절 후 당일 재진입 금지".to_string(),
                };
            }
        }

        if let Some(last_same) = self
            .last_order_at
            .get(&(scope.clone(), symbol.clone(), side))
        {
            let elapsed = now.signed_duration_since(*last_same);
            if elapsed < Duration::minutes(self.config.same_side_cooldown_min) {
                return GuardDecision::Block {
                    reason: format!(
                        "동일 방향 쿨다운: {}분 미경과",
                        self.config.same_side_cooldown_min
                    ),
                };
            }
        }

        if side == GuardSide::Buy {
            if let Some(last_sell) =
                self.last_order_at
                    .get(&(scope.clone(), symbol.clone(), GuardSide::Sell))
            {
                let elapsed = now.signed_duration_since(*last_sell);
                if elapsed < Duration::minutes(self.config.after_sell_buy_cooldown_min) {
                    return GuardDecision::Block {
                        reason: format!(
                            "매도 후 재매수 쿨다운: {}분 미경과",
                            self.config.after_sell_buy_cooldown_min
                        ),
                    };
                }
            }
        }

        if self.is_whipsaw(scope, symbol, side) {
            let until = now + Duration::minutes(self.config.whipsaw_cooldown_min);
            self.cooldown_until.insert(symbol_key, until);
            return GuardDecision::Block {
                reason: format!(
                    "최근 {}분 내 반대 신호 반복 — {}분 쿨다운",
                    self.config.whipsaw_window_min, self.config.whipsaw_cooldown_min
                ),
            };
        }

        if side == GuardSide::Sell && held_quantity > 0 {
            if let Some(avg) = avg_price {
                if avg > 0
                    && tick_price > avg
                    && !self.has_positive_expected_profit(
                        avg,
                        tick_price,
                        held_quantity,
                        is_overseas,
                    )
                {
                    return GuardDecision::Block {
                        reason: "수수료·세금·슬리피지 차감 후 기대 순익이 0 이하".to_string(),
                    };
                }
            }
        }

        GuardDecision::Allow
    }

    pub fn record_submitted(&mut self, signal: &Signal) {
        self.record_submitted_for_scope(&BrokerScope::kis_legacy(), signal);
    }

    pub fn record_submitted_for_scope(&mut self, scope: &BrokerScope, signal: &Signal) {
        let (symbol, side, reason) = match signal {
            Signal::Buy { symbol, reason, .. } => (symbol.clone(), GuardSide::Buy, reason),
            Signal::Sell { symbol, reason, .. } => (symbol.clone(), GuardSide::Sell, reason),
            Signal::Hold => return,
        };
        let now = Local::now();
        let symbol_key = (scope.clone(), symbol.clone());
        self.last_order_at
            .insert((scope.clone(), symbol.clone(), side), now);
        if side == GuardSide::Sell && is_stop_loss_reason(reason) {
            self.stop_loss_block_date
                .insert(symbol_key.clone(), now.date_naive());
        }
        self.recent_sides
            .entry(symbol_key)
            .or_default()
            .push_back((now, side));
    }

    pub fn reset_day(&mut self) {
        self.stop_loss_block_date.clear();
        self.cooldown_until.clear();
        self.recent_sides.clear();
    }

    fn prune_recent(&mut self, scope: &BrokerScope, symbol: &str, now: DateTime<Local>) {
        let cutoff = now - Duration::minutes(self.config.whipsaw_window_min);
        let key = (scope.clone(), symbol.to_string());
        if let Some(recent) = self.recent_sides.get_mut(&key) {
            while recent.front().map(|(ts, _)| *ts < cutoff).unwrap_or(false) {
                recent.pop_front();
            }
        }
    }

    fn is_whipsaw(&self, scope: &BrokerScope, symbol: &str, side: GuardSide) -> bool {
        let key = (scope.clone(), symbol.to_string());
        let Some(recent) = self.recent_sides.get(&key) else {
            return false;
        };
        let opposite_count = recent.iter().filter(|(_, s)| *s != side).count();
        opposite_count >= self.config.max_opposite_signals
    }

    fn has_positive_expected_profit(
        &self,
        avg_price: u64,
        sell_price: u64,
        quantity: u64,
        is_overseas: bool,
    ) -> bool {
        let gross = sell_price
            .saturating_sub(avg_price)
            .saturating_mul(quantity);
        let bps = if is_overseas {
            self.config.overseas_fee_bps + self.config.overseas_slippage_bps
        } else {
            // 국내 매도 비용: 위탁수수료 0.015% + 거래세 0.20% + 슬리피지.
            22 + self.config.domestic_slippage_bps
        };
        let sell_amount = sell_price.saturating_mul(quantity);
        let estimated_cost = ((sell_amount as f64) * (bps as f64 / 10_000.0)).ceil() as u64;
        let min_profit = ((sell_amount as f64)
            * (self.config.min_expected_profit_bps.max(0) as f64 / 10_000.0))
            .ceil() as u64;
        gross > estimated_cost.saturating_add(min_profit)
    }
}

impl Default for TradeGuard {
    fn default() -> Self {
        Self::new(TradeGuardConfig::default())
    }
}

fn is_stop_loss_reason(reason: &str) -> bool {
    reason.contains("손절") || reason.contains("stop") || reason.contains("Stop")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::{BrokerAccountId, BrokerId};

    fn scope(account: &str) -> BrokerScope {
        BrokerScope::new(BrokerId::Kis, Some(BrokerAccountId(account.to_string())))
    }

    fn buy_signal() -> Signal {
        Signal::Buy {
            symbol: "005930".to_string(),
            quantity: 1,
            reason: "test buy".to_string(),
        }
    }

    fn sell_signal(reason: &str) -> Signal {
        Signal::Sell {
            symbol: "005930".to_string(),
            quantity: 1,
            reason: reason.to_string(),
        }
    }

    #[test]
    fn blocks_repeated_same_side_signal_inside_cooldown() {
        let mut guard = TradeGuard::default();
        let signal = buy_signal();

        assert!(matches!(
            guard.evaluate(&signal, 0, None, 70_000, false),
            GuardDecision::Allow
        ));
        guard.record_submitted(&signal);

        assert!(matches!(
            guard.evaluate(&signal, 0, None, 70_000, false),
            GuardDecision::Block { .. }
        ));
    }

    #[test]
    fn cooldown_is_isolated_by_broker_account_scope() {
        let mut guard = TradeGuard::default();
        let signal = buy_signal();
        let account_a = scope("11111111-01");
        let account_b = scope("22222222-01");

        guard.record_submitted_for_scope(&account_a, &signal);

        assert!(matches!(
            guard.evaluate_for_scope(&account_a, &signal, 0, None, 70_000, false),
            GuardDecision::Block { .. }
        ));
        assert!(matches!(
            guard.evaluate_for_scope(&account_b, &signal, 0, None, 70_000, false),
            GuardDecision::Allow
        ));
    }

    #[test]
    fn blocks_reentry_after_stop_loss_sell_on_same_day() {
        let mut guard = TradeGuard::default();
        let sell = sell_signal("손절 매도");
        guard.record_submitted(&sell);

        assert!(matches!(
            guard.evaluate(&buy_signal(), 0, None, 70_000, false),
            GuardDecision::Block { reason } if reason.contains("손절 후")
        ));
    }

    #[test]
    fn blocks_whipsaw_after_repeated_opposite_signals() {
        let config = TradeGuardConfig {
            same_side_cooldown_min: 0,
            after_sell_buy_cooldown_min: 0,
            max_opposite_signals: 2,
            ..TradeGuardConfig::default()
        };
        let mut guard = TradeGuard::new(config);
        let sell = sell_signal("test sell");
        guard.record_submitted(&sell);
        guard.record_submitted(&sell);

        assert!(matches!(
            guard.evaluate(&buy_signal(), 0, None, 70_000, false),
            GuardDecision::Block { reason } if reason.contains("반대 신호 반복")
        ));
    }

    #[test]
    fn blocks_tiny_profit_after_costs() {
        let mut guard = TradeGuard::default();
        let sell = sell_signal("익절 매도");

        assert!(matches!(
            guard.evaluate(&sell, 1, Some(70_000), 70_010, false),
            GuardDecision::Block { reason } if reason.contains("기대 순익")
        ));
    }
}
