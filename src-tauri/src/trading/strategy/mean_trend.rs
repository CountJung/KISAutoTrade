use std::collections::VecDeque;

use super::{
    observation::{advance_owned_age, sync_owned_age},
    state::{bounded_window, bounded_window_with_extra},
    HistoryReadiness, Signal, Strategy, StrategyConfig,
};

mod mean_reversion;
pub use mean_reversion::{
    validate_mean_reversion_params, MeanReversionExitMode, MeanReversionParams,
    MeanReversionStrategy,
};
mod trend_filter_params;
mod trend_filter_protection;
pub use trend_filter_params::{validate_trend_filter_params, TrendFilterParams};

/// 가격 지표와 실제 소유 증거는 독립이다. 신호는 수량·원가를 변경하지 않는다.
#[derive(Default)]
struct TrendFilterState {
    prices: VecDeque<u64>,
    owned_quantity: u64,
    avg_price: u64,
    held_observations: Option<u64>,
    entry_candidate: bool,
    order_pending: bool,
    observed_peak: Option<u64>,
    profit_armed: bool,
    // 새 MA 문맥에서만 의미가 있으므로 영속 checkpoint에는 저장하지 않는다.
    exit_streak: u64,
}

pub struct TrendFilterStrategy {
    config: StrategyConfig,
    params: TrendFilterParams,
    params_valid: bool,
    states: std::collections::HashMap<String, TrendFilterState>,
}

impl TrendFilterStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params = trend_filter_params::parse(&config.params);
        let params_valid = validate_trend_filter_params("trend_filter", &config.params).is_ok();
        Self {
            config,
            params,
            params_valid,
            states: std::collections::HashMap::new(),
        }
    }

    fn moving_avg(prices: &VecDeque<u64>, period: usize) -> Option<f64> {
        if period == 0 || prices.len() < period {
            return None;
        }
        // 기존 합→f64→나눗셈 순서를 유지하며 정수 합 overflow만 제거한다.
        let sum: u128 = prices.iter().rev().take(period).map(|&p| p as u128).sum();
        Some(sum as f64 / period as f64)
    }

    fn max_period(&self) -> usize {
        bounded_window(
            self.params
                .long_period
                .max(self.params.mid_period)
                .max(self.params.short_period) as usize,
        )
    }
}

impl Strategy for TrendFilterStrategy {
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
    fn has_held_position(&self, symbol: &str) -> Option<bool> {
        Some(
            self.states
                .get(symbol)
                .is_some_and(|s| s.owned_quantity > 0),
        )
    }
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn reconfigure_preserving_position(&mut self, config: StrategyConfig) -> bool {
        if !self.config.same_identity(&config)
            || self
                .states
                .iter()
                .any(|(symbol, s)| s.owned_quantity > 0 && !config.targets_symbol(symbol))
        {
            return false;
        }
        let mut replacement = Self::new(config);
        let core_changed = self.params.long_period != replacement.params.long_period
            || self.params.mid_period != replacement.params.mid_period
            || self.params.short_period != replacement.params.short_period;
        replacement.states = std::mem::take(&mut self.states);
        replacement
            .states
            .retain(|symbol, _| replacement.config.targets_symbol(symbol));
        let cap = bounded_window_with_extra(replacement.max_period(), 1);
        for s in replacement.states.values_mut() {
            while s.prices.len() > cap {
                s.prices.pop_front();
            }
            if core_changed {
                s.exit_streak = 0;
            }
            if !s.order_pending && s.owned_quantity == 0 {
                s.entry_candidate = false;
            }
        }
        *self = replacement;
        true
    }

    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let n = self.max_period();
        let s = self.states.entry(symbol.into()).or_default();
        s.prices = prices[prices.len().saturating_sub(n)..]
            .iter()
            .copied()
            .collect();
        s.exit_streak = 0;
        // 지표 준비는 보유 나이/최고가/무장·소유를 만들거나 덮지 않는다.
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required_bars = self.max_period();
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars,
            available_bars: state.map_or(0, |s| s.prices.len()),
            ready: state.is_some_and(|s| Self::moving_avg(&s.prices, required_bars).is_some()),
        })
    }
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        if self
            .states
            .get(symbol)
            .is_some_and(|s| s.owned_quantity > 0)
        {
            Some(true)
        } else {
            self.history_readiness(symbol)
                .map(|s| s.available_bars.saturating_add(1) >= s.required_bars)
        }
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        let cap = bounded_window_with_extra(self.max_period(), 1);
        let s = self.states.entry(symbol.into()).or_default();
        advance_owned_age(s.owned_quantity, &mut s.held_observations);
        if s.owned_quantity > 0 {
            if price > 0 {
                s.observed_peak = Some(s.observed_peak.unwrap_or(price).max(price));
            }
            if self.params.profit_activation_pct > 0.0
                && self.params.profit_trailing_pct > 0.0
                && s.avg_price > 0
                && price as f64
                    >= s.avg_price as f64 * (1.0 + self.params.profit_activation_pct / 100.0)
            {
                s.profit_armed = true;
            }
        }
        if !self.config.enabled {
            return Signal::Hold;
        }
        s.prices.push_back(price);
        while s.prices.len() > cap {
            s.prices.pop_front();
        }
        let mas = (
            Self::moving_avg(&s.prices, bounded_window(self.params.long_period as usize)),
            Self::moving_avg(&s.prices, bounded_window(self.params.mid_period as usize)),
            Self::moving_avg(&s.prices, bounded_window(self.params.short_period as usize)),
        );
        if s.owned_quantity > 0 {
            // 원래 청산은 비정렬 기간에서도 세 MA 모두 준비돼야 한다.
            let below = matches!(mas, (Some(lma), Some(_), Some(_)) if (price as f64) < lma);
            s.exit_streak = if below {
                s.exit_streak.saturating_add(1)
            } else {
                0
            };
            let reason = if s.exit_streak >= self.params.exit_confirm_observations {
                Some(if self.params.exit_confirm_observations == 1 {
                    format!(
                        "추세 필터 청산: 현재가 {} < 장기MA {:.0}",
                        price,
                        mas.0.unwrap_or_default()
                    )
                } else {
                    format!("추세 필터 장기 MA 이탈 청산: {}연속 관측", s.exit_streak)
                })
            } else if s.profit_armed
                && self.params.profit_trailing_pct > 0.0
                && s.observed_peak.is_some_and(|peak| {
                    price as f64 <= peak as f64 * (1.0 - self.params.profit_trailing_pct / 100.0)
                })
            {
                Some("추세 필터 수익 보호 되밀림".into())
            } else if self.params.max_hold_observations > 0
                && s.held_observations
                    .is_none_or(|age| age >= self.params.max_hold_observations)
            {
                Some("추세 필터 최대 보유 관측 제한".into())
            } else if !self.params_valid {
                // 저장/preview는 거부한다. 이미 손상된 runtime은 진입을 막고 보호를 조용히 끄지 않는다.
                Some("추세 필터 설정 불명: 보수적 잔량 청산 후보".into())
            } else {
                None
            };
            return reason.map_or(Signal::Hold, |reason| Signal::Sell {
                symbol: symbol.into(),
                quantity: s.owned_quantity,
                reason,
            });
        }
        s.exit_streak = 0;
        if self.params_valid
            && matches!(mas, (Some(lma), Some(mma), Some(sma)) if price as f64 > lma && sma > mma)
        {
            if !s.order_pending {
                s.entry_candidate = true;
            }
            return Signal::Buy {
                symbol: symbol.into(),
                quantity: self.config.order_quantity,
                reason: format!(
                    "추세 필터 매수: 현재가 {} > 장기MA {:.0}, 단기MA {:.0} > 중기MA {:.0}",
                    price,
                    mas.0.unwrap_or_default(),
                    mas.2.unwrap_or_default(),
                    mas.1.unwrap_or_default()
                ),
            };
        }
        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        let previous = s.owned_quantity;
        sync_owned_age(
            previous,
            quantity,
            &mut s.held_observations,
            &mut s.entry_candidate,
        );
        if (quantity > 0 && previous == 0) || quantity == 0 {
            s.exit_streak = 0;
            s.observed_peak = None;
            s.profit_armed = false;
        }
        s.owned_quantity = quantity;
        s.avg_price = if quantity > 0 { avg_price } else { 0 };
    }
    fn set_order_pending(&mut self, symbol: &str, pending: bool) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        s.order_pending = pending;
        if !pending && s.owned_quantity == 0 {
            s.entry_candidate = false;
        }
    }
    fn discard_unsubmitted_signal(&mut self, symbol: &str) {
        if let Some(s) = self.states.get_mut(symbol) {
            if !s.order_pending && s.owned_quantity == 0 {
                s.entry_candidate = false;
            }
        }
    }
    fn owned_position_metadata(&self, symbol: &str) -> Option<serde_json::Value> {
        self.states
            .get(symbol)
            .and_then(trend_filter_protection::encode)
    }
    fn restore_owned_position_metadata(
        &mut self,
        symbol: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String> {
        let s = self
            .states
            .get_mut(symbol)
            .ok_or("추세 필터 소유 상태 없이 복원할 수 없습니다")?;
        trend_filter_protection::restore(s, metadata)
    }
    fn reset(&mut self) {
        // 기존 가격 버퍼 유지 계약. 실제 소유·보호·확인값은 초기화한다.
        for s in self.states.values_mut() {
            let prices = std::mem::take(&mut s.prices);
            *s = TrendFilterState {
                prices,
                ..Default::default()
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trading::strategy::{initialize_strategy_warmup, OhlcCandle};

    fn config(id: &str, params: serde_json::Value) -> StrategyConfig {
        StrategyConfig::new(id, id, true, vec!["SOXQ".into()], 10, params)
    }

    fn daily(closes: impl Iterator<Item = u64>) -> Vec<OhlcCandle> {
        closes
            .map(|close| OhlcCandle {
                open: close,
                high: 20_000,
                low: close - 100,
                close,
            })
            .collect()
    }

    #[test]
    fn warmup_mean_reversion_uses_close_bands_and_first_signal() {
        let params = MeanReversionParams::default();
        let mut strategy = MeanReversionStrategy::new(config(
            "mean_reversion",
            serde_json::to_value(&params).unwrap(),
        ));
        let candles = daily(std::iter::repeat_n(10_000, 252));
        initialize_strategy_warmup(&mut strategy, "SOXQ", &candles, &[]);

        let state = &strategy.states["SOXQ"];
        assert_eq!(state.prices.len(), 20);
        assert_eq!(
            MeanReversionStrategy::bollinger_bands(&state.prices, 20, params.std_dev),
            Some((10_000.0, 10_000.0, 10_000.0))
        );
        assert!(matches!(strategy.on_tick("SOXQ", 10_000, 0), Signal::Hold));
        assert!(matches!(
            strategy.on_tick("SOXQ", 9_000, 0),
            Signal::Buy { .. }
        ));
    }

    #[test]
    fn warmup_trend_filter_uses_200_closes_and_first_signal() {
        let mut strategy = TrendFilterStrategy::new(config(
            "trend_filter",
            serde_json::to_value(TrendFilterParams::default()).unwrap(),
        ));
        let candles = daily(10_000..10_252);
        initialize_strategy_warmup(&mut strategy, "SOXQ", &candles, &[]);

        let state = &strategy.states["SOXQ"];
        assert_eq!(state.prices.len(), 200);
        assert_eq!(
            TrendFilterStrategy::moving_avg(&state.prices, 200),
            Some(10_151.5)
        );
        assert_eq!(
            TrendFilterStrategy::moving_avg(&state.prices, 20),
            Some(10_241.5)
        );
        assert_eq!(
            TrendFilterStrategy::moving_avg(&state.prices, 5),
            Some(10_249.0)
        );
        assert!(matches!(
            strategy.on_tick("SOXQ", 10_252, 0),
            Signal::Buy { .. }
        ));
    }

    #[test]
    fn warmup_preserves_positions_and_ignores_unrelated_symbol() {
        let mut mean = MeanReversionStrategy::new(config(
            "mean_reversion",
            serde_json::to_value(MeanReversionParams::default()).unwrap(),
        ));
        let mut trend = TrendFilterStrategy::new(config(
            "trend_filter",
            serde_json::to_value(TrendFilterParams::default()).unwrap(),
        ));
        mean.sync_position("SOXQ", 10, 10_000);
        trend.sync_position("SOXQ", 10, 10_000);
        let candles = daily(std::iter::repeat_n(10_000, 252));
        let intraday = daily(std::iter::repeat_n(15_000, 10));
        for strategy in [&mut mean as &mut dyn Strategy, &mut trend] {
            initialize_strategy_warmup(strategy, "SOXQ", &candles, &[]);
            initialize_strategy_warmup(strategy, "OTHER", &candles, &[]);
            initialize_strategy_warmup(strategy, "SOXQ", &[], &[]);
            initialize_strategy_warmup(strategy, "SOXQ", &[], &intraday);
        }
        assert_eq!(mean.states.len(), 1);
        assert_eq!(trend.states.len(), 1);
        assert!(mean.states["SOXQ"].owned_quantity > 0);
        assert_eq!(mean.states["SOXQ"].entry_price, Some(10_000));
        assert!((trend.states["SOXQ"].owned_quantity > 0));
        assert_eq!(mean.states["SOXQ"].prices.len(), 20);
        assert_eq!(trend.states["SOXQ"].prices.len(), 200);
        assert!(mean.states["SOXQ"]
            .prices
            .iter()
            .all(|price| *price == 10_000));
        assert!(trend.states["SOXQ"]
            .prices
            .iter()
            .all(|price| *price == 10_000));
    }
}

#[cfg(test)]
mod reconfigure_tests;

#[cfg(test)]
#[path = "mean_trend/live_feedback_tests.rs"]
mod live_feedback_tests;

#[cfg(test)]
mod trend_filter_tests;
