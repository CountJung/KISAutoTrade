use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::{state::bounded_window_with_extra, Signal, Strategy, StrategyConfig};

// ────────────────────────────────────────────────────────────────────
// 이동평균 교차 전략 (Golden Cross / Death Cross)
// ────────────────────────────────────────────────────────────────────

/// MA 교차 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaCrossParams {
    /// 단기 이동평균 기간
    pub short_period: usize,
    /// 장기 이동평균 기간
    pub long_period: usize,
}

impl Default for MaCrossParams {
    fn default() -> Self {
        Self {
            short_period: 5,
            long_period: 20,
        }
    }
}

/// 종목별 MA교차 상태
struct MaCrossState {
    prices: VecDeque<u64>,
    prev_short_ma: Option<f64>,
    prev_long_ma: Option<f64>,
}

pub struct MovingAverageCrossStrategy {
    config: StrategyConfig,
    params: MaCrossParams,
    /// 종목코드 → 개별 상태 (다중 종목 지원)
    states: std::collections::HashMap<String, MaCrossState>,
}

impl MovingAverageCrossStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: MaCrossParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }

    fn moving_average(prices: &VecDeque<u64>, period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        let sum: u64 = prices.iter().rev().take(period).sum();
        Some(sum as f64 / period as f64)
    }
}

impl Strategy for MovingAverageCrossStrategy {
    fn id(&self) -> &str {
        &self.config.id
    }
    fn name(&self) -> &str {
        &self.config.name
    }
    fn config(&self) -> &StrategyConfig {
        &self.config
    }
    fn config_mut(&mut self) -> &mut StrategyConfig {
        &mut self.config
    }
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let cap = bounded_window_with_extra(self.params.long_period, 1);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MaCrossState {
                prices: VecDeque::with_capacity(cap),
                prev_short_ma: None,
                prev_long_ma: None,
            });

        state.prices.push_back(price);
        if state.prices.len() > cap {
            state.prices.pop_front();
        }

        let short_ma = Self::moving_average(&state.prices, self.params.short_period);
        let long_ma = Self::moving_average(&state.prices, self.params.long_period);

        let signal = match (state.prev_short_ma, state.prev_long_ma, short_ma, long_ma) {
            (Some(ps), Some(pl), Some(cs), Some(cl)) => {
                if ps <= pl && cs > cl {
                    Signal::Buy {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!("골든크로스 S{:.0} > L{:.0}", cs, cl),
                    }
                } else if ps >= pl && cs < cl {
                    Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!("데드크로스 S{:.0} < L{:.0}", cs, cl),
                    }
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        state.prev_short_ma = short_ma;
        state.prev_long_ma = long_ma;
        signal
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 전략 저장소 (런타임에 여러 전략을 관리)
// ────────────────────────────────────────────────────────────────────
