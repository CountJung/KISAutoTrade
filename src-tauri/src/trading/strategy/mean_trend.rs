use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::{
    state::{bounded_window, bounded_window_with_extra},
    HistoryReadiness, Signal, Strategy, StrategyConfig,
};

// ────────────────────────────────────────────────────────────────────
// 09. 평균회귀 전략 (MeanReversionStrategy) — 볼린저 밴드
// ────────────────────────────────────────────────────────────────────
// 동작:
//  1. 공통 warmup의 `initialize_ohlc` 기본 훅이 종가를 `initialize_historical`에 전달
//  2. 실시간 틱마다 볼린저 밴드 계산:
//       mean      = 최근 period 개의 평균
//       std_dev   = population std deviation
//       upper     = mean + std_dev * 배율
//       lower     = mean - std_dev * 배율
//  3. 미포지션 && 현재가 < lower band → 매수 (과매도, 평균 회귀 기대)
//  4. 포지션 보유 && (현재가 > upper band → 익절 매도 OR 손절 기준 초과 → 손절)
// ────────────────────────────────────────────────────────────────────

/// 평균회귀 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanReversionParams {
    /// 볼린저 밴드 기간 (기본 20)
    pub period: u32,
    /// 표준편차 배율 (기본 2.0)
    pub std_dev: f64,
    /// 손절 기준 % (기본 5.0)
    pub stop_loss_pct: f64,
}

impl Default for MeanReversionParams {
    fn default() -> Self {
        Self {
            period: 20,
            std_dev: 2.0,
            stop_loss_pct: 5.0,
        }
    }
}

/// 종목별 평균회귀 상태
struct MeanReversionState {
    prices: VecDeque<u64>,
    in_position: bool,
    entry_price: Option<u64>,
}

pub struct MeanReversionStrategy {
    config: StrategyConfig,
    params: MeanReversionParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, MeanReversionState>,
}

impl MeanReversionStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: MeanReversionParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }

    fn bollinger_bands(
        prices: &VecDeque<u64>,
        period: usize,
        std_dev_mult: f64,
    ) -> Option<(f64, f64, f64)> {
        if prices.len() < period {
            return None;
        }
        let slice: Vec<f64> = prices
            .iter()
            .rev()
            .take(period)
            .map(|&p| p as f64)
            .collect();
        let mean = slice.iter().sum::<f64>() / period as f64;
        let variance = slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let std = variance.sqrt();
        let upper = mean + std_dev_mult * std;
        let lower = mean - std_dev_mult * std;
        Some((mean, upper, lower))
    }
}

impl Strategy for MeanReversionStrategy {
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

    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let n = bounded_window(self.params.period as usize);
        let take = prices.len().min(n);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MeanReversionState {
                prices: VecDeque::with_capacity(bounded_window_with_extra(n, 1)),
                in_position: false,
                entry_price: None,
            });
        state.prices.clear();
        for &p in prices[prices.len().saturating_sub(take)..].iter() {
            state.prices.push_back(p);
        }
        tracing::info!(
            "평균회귀 초기화 [{}]: 과거 {}개 가격 로드 (period={})",
            symbol,
            state.prices.len(),
            n
        );
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required_bars = bounded_window(self.params.period as usize);
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars,
            available_bars: state.map_or(0, |state| state.prices.len()),
            ready: state.is_some_and(|state| {
                Self::bollinger_bands(&state.prices, required_bars, self.params.std_dev).is_some()
            }),
        })
    }

    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        self.history_readiness(symbol)
            .map(|state| state.available_bars.saturating_add(1) >= state.required_bars)
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let period = bounded_window(self.params.period as usize);
        let std_dev = self.params.std_dev;
        let stop_loss = self.params.stop_loss_pct;
        let qty = self.config.order_quantity;

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MeanReversionState {
                prices: VecDeque::with_capacity(bounded_window_with_extra(period, 1)),
                in_position: false,
                entry_price: None,
            });

        state.prices.push_back(price);
        while state.prices.len() > period + 1 {
            state.prices.pop_front();
        }

        let (mean, upper, lower) = match Self::bollinger_bands(&state.prices, period, std_dev) {
            Some(b) => b,
            None => return Signal::Hold,
        };

        if state.in_position {
            if let Some(ep) = state.entry_price {
                let loss_pct = (ep as f64 - price as f64) / ep as f64 * 100.0;
                if loss_pct >= stop_loss {
                    state.in_position = false;
                    state.entry_price = None;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: qty,
                        reason: format!(
                            "평균회귀 손절: 현재가 {} (매수가 {} 대비 -{:.2}%)",
                            price, ep, loss_pct
                        ),
                    };
                }
            }
            if price as f64 > upper {
                state.in_position = false;
                state.entry_price = None;
                return Signal::Sell {
                    symbol: symbol.to_string(),
                    quantity: qty,
                    reason: format!(
                        "평균회귀 익절: 현재가 {} > 상단밴드 {:.0} (mean={:.0})",
                        price, upper, mean
                    ),
                };
            }
            return Signal::Hold;
        }

        if (price as f64) < lower {
            state.in_position = true;
            state.entry_price = Some(price);
            return Signal::Buy {
                symbol: symbol.to_string(),
                quantity: qty,
                reason: format!(
                    "평균회귀 매수: 현재가 {} < 하단밴드 {:.0} (mean={:.0})",
                    price, lower, mean
                ),
            };
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let period = bounded_window(self.params.period as usize);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MeanReversionState {
                prices: VecDeque::with_capacity(bounded_window_with_extra(period, 1)),
                in_position: false,
                entry_price: None,
            });
        state.in_position = quantity > 0;
        state.entry_price = (quantity > 0 && avg_price > 0).then_some(avg_price);
    }

    fn reset(&mut self) {
        // 가격 버퍼 유지, 포지션만 초기화
        for state in self.states.values_mut() {
            state.in_position = false;
            state.entry_price = None;
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 10. 추세 필터 전략 (TrendFilterStrategy)
// ────────────────────────────────────────────────────────────────────
// 동작:
//  1. 공통 warmup의 `initialize_ohlc` 기본 훅이 종가를 `initialize_historical`에 전달
//  2. 실시간 틱마다 3개의 이동평균 계산:
//       short_MA  = 최근 short_period 개의 평균
//       mid_MA    = 최근 mid_period 개의 평균
//       long_MA   = 최근 long_period 개의 평균
//  3. 미포지션 AND 현재가 > long_MA AND short_MA > mid_MA → 매수 (상승 추세 확인)
//  4. 포지션 보유 AND 현재가 < long_MA → 장기 추세 전환 → 청산 매도
// ────────────────────────────────────────────────────────────────────

/// 추세 필터 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendFilterParams {
    /// 장기 추세 기준 이동평균 기간 (기본 200일)
    pub long_period: u32,
    /// 단기 이동평균 기간 (기본 5일)
    pub short_period: u32,
    /// 중기 이동평균 기간 (기본 20일)
    pub mid_period: u32,
}

impl Default for TrendFilterParams {
    fn default() -> Self {
        Self {
            long_period: 200,
            short_period: 5,
            mid_period: 20,
        }
    }
}

/// 종목별 추세필터 상태
struct TrendFilterState {
    prices: VecDeque<u64>,
    in_position: bool,
}

pub struct TrendFilterStrategy {
    config: StrategyConfig,
    params: TrendFilterParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, TrendFilterState>,
}

impl TrendFilterStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: TrendFilterParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }

    fn moving_avg(prices: &VecDeque<u64>, period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        let sum: u64 = prices.iter().rev().take(period).sum();
        Some(sum as f64 / period as f64)
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
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let n = bounded_window(
            self.params
                .long_period
                .max(self.params.mid_period)
                .max(self.params.short_period) as usize,
        );
        let take = prices.len().min(n);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| TrendFilterState {
                prices: VecDeque::with_capacity(bounded_window_with_extra(n, 1)),
                in_position: false,
            });
        state.prices.clear();
        for &p in prices[prices.len().saturating_sub(take)..].iter() {
            state.prices.push_back(p);
        }
        tracing::info!(
            "추세 필터 초기화 [{}]: 과거 {}개 가격 로드 (long_period={})",
            symbol,
            state.prices.len(),
            n
        );
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required_bars = [
            self.params.long_period,
            self.params.mid_period,
            self.params.short_period,
        ]
        .into_iter()
        .map(|period| bounded_window(period as usize))
        .max()
        .unwrap_or(1);
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars,
            available_bars: state.map_or(0, |state| state.prices.len()),
            ready: state
                .is_some_and(|state| Self::moving_avg(&state.prices, required_bars).is_some()),
        })
    }

    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        self.history_readiness(symbol)
            .map(|state| state.available_bars.saturating_add(1) >= state.required_bars)
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let max_cap = bounded_window_with_extra(
            self.params
                .long_period
                .max(self.params.mid_period)
                .max(self.params.short_period) as usize,
            1,
        );
        let long_p = bounded_window(self.params.long_period as usize);
        let mid_p = bounded_window(self.params.mid_period as usize);
        let short_p = bounded_window(self.params.short_period as usize);
        let qty = self.config.order_quantity;

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| TrendFilterState {
                prices: VecDeque::with_capacity(max_cap),
                in_position: false,
            });

        state.prices.push_back(price);
        while state.prices.len() > max_cap {
            state.prices.pop_front();
        }

        let long_ma = Self::moving_avg(&state.prices, long_p);
        let mid_ma = Self::moving_avg(&state.prices, mid_p);
        let short_ma = Self::moving_avg(&state.prices, short_p);

        match (long_ma, mid_ma, short_ma) {
            (Some(lma), Some(mma), Some(sma)) => {
                if state.in_position && (price as f64) < lma {
                    state.in_position = false;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: qty,
                        reason: format!("추세 필터 청산: 현재가 {} < 장기MA {:.0}", price, lma),
                    };
                }
                if !state.in_position && (price as f64) > lma && sma > mma {
                    state.in_position = true;
                    return Signal::Buy {
                        symbol: symbol.to_string(),
                        quantity: qty,
                        reason: format!(
                            "추세 필터 매수: 현재가 {} > 장기MA {:.0}, 단기MA {:.0} > 중기MA {:.0}",
                            price, lma, sma, mma
                        ),
                    };
                }
                Signal::Hold
            }
            _ => Signal::Hold,
        }
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, _avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let cap = bounded_window_with_extra(self.params.long_period as usize, 1);
        self.states
            .entry(symbol.to_string())
            .or_insert_with(|| TrendFilterState {
                prices: VecDeque::with_capacity(cap),
                in_position: false,
            })
            .in_position = quantity > 0;
    }

    fn reset(&mut self) {
        // 가격 버퍼 유지, 포지션만 초기화
        for state in self.states.values_mut() {
            state.in_position = false;
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
        let candles = daily(std::iter::repeat(10_000).take(252));
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
        let candles = daily(std::iter::repeat(10_000).take(252));
        let intraday = daily(std::iter::repeat(15_000).take(10));
        for strategy in [&mut mean as &mut dyn Strategy, &mut trend] {
            initialize_strategy_warmup(strategy, "SOXQ", &candles, &[]);
            initialize_strategy_warmup(strategy, "OTHER", &candles, &[]);
            initialize_strategy_warmup(strategy, "SOXQ", &[], &[]);
            initialize_strategy_warmup(strategy, "SOXQ", &[], &intraday);
        }
        assert_eq!(mean.states.len(), 1);
        assert_eq!(trend.states.len(), 1);
        assert!(mean.states["SOXQ"].in_position);
        assert_eq!(mean.states["SOXQ"].entry_price, Some(10_000));
        assert!(trend.states["SOXQ"].in_position);
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
