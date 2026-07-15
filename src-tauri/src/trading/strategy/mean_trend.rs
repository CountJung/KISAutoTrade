use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::{
    state::{bounded_window, bounded_window_with_extra},
    Signal, Strategy, StrategyConfig,
};

// ────────────────────────────────────────────────────────────────────
// 09. 평균회귀 전략 (MeanReversionStrategy) — 볼린저 밴드
// ────────────────────────────────────────────────────────────────────
// 동작:
//  1. 자동매매 시작 시 `initialize_historical`로 과거 종가 배열 전달 → 가격 버퍼 사전 적재
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
//  1. 자동매매 시작 시 `initialize_historical`로 과거 종가 배열 전달 → 가격 버퍼 사전 적재
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
        let n = bounded_window(self.params.long_period as usize);
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

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let max_cap = bounded_window_with_extra(self.params.long_period as usize, 1);
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
