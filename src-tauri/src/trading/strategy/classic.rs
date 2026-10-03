use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::{
    state::{bounded_window, bounded_window_with_extra},
    HistoryReadiness, Signal, Strategy, StrategyConfig,
};

// ────────────────────────────────────────────────────────────────────
// RSI 전략 (Relative Strength Index)
// ────────────────────────────────────────────────────────────────────

/// RSI 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiParams {
    /// RSI 계산 기간 (기본 14)
    pub period: usize,
    /// 과매도 기준선 (기본 30) — RSI가 이 이하 → 이 이상으로 올라올 때 매수
    pub oversold: f64,
    /// 과매수 기준선 (기본 70) — RSI가 이 이상 → 이 이하로 떨어질 때 매도
    pub overbought: f64,
}

impl Default for RsiParams {
    fn default() -> Self {
        Self {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
        }
    }
}

pub struct RsiStrategy {
    config: StrategyConfig,
    params: RsiParams,
    /// 종목코드 → (가격 이력, 이전 RSI)
    states: std::collections::HashMap<String, (VecDeque<u64>, Option<f64>)>,
}

impl RsiStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: RsiParams = serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }

    /// 단순 이동평균 방식 RSI 계산
    fn calc_rsi(prices: &VecDeque<u64>, period: usize) -> Option<f64> {
        if period == 0 || prices.len() < period.saturating_add(1) {
            return None;
        }
        // 최신순으로 period+1개 추출
        let recent: Vec<u64> = prices.iter().rev().take(period + 1).cloned().collect();
        let mut gain_sum = 0.0f64;
        let mut loss_sum = 0.0f64;
        for i in 0..period {
            let diff = recent[i] as f64 - recent[i + 1] as f64;
            if diff > 0.0 {
                gain_sum += diff;
            } else {
                loss_sum += -diff;
            }
        }
        let avg_gain = gain_sum / period as f64;
        let avg_loss = loss_sum / period as f64;
        if avg_loss == 0.0 {
            return Some(100.0);
        }
        let rs = avg_gain / avg_loss;
        Some(100.0 - 100.0 / (1.0 + rs))
    }
}

impl Strategy for RsiStrategy {
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
        let cap = bounded_window_with_extra(self.params.period, 2);
        let history = prices[prices.len().saturating_sub(cap)..]
            .iter()
            .copied()
            .collect();
        let prev_rsi = Self::calc_rsi(&history, self.params.period);
        self.states.insert(symbol.to_string(), (history, prev_rsi));
    }

    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars: self.params.period.saturating_add(1),
            available_bars: state.map_or(0, |(prices, _)| prices.len()),
            ready: state.is_some_and(|(_, prev_rsi)| prev_rsi.is_some()),
        })
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let cap = bounded_window_with_extra(self.params.period, 2);
        let (prices, prev_rsi) = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| (VecDeque::with_capacity(cap), None));

        prices.push_back(price);
        if prices.len() > cap {
            prices.pop_front();
        }

        let rsi = Self::calc_rsi(prices, self.params.period);

        let signal = match (*prev_rsi, rsi) {
            (Some(prev), Some(cur)) => {
                if prev <= self.params.oversold && cur > self.params.oversold {
                    Signal::Buy {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!("RSI 과매도 반등 {:.1}", cur),
                    }
                } else if prev >= self.params.overbought && cur < self.params.overbought {
                    Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!("RSI 과매수 하락 {:.1}", cur),
                    }
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        *prev_rsi = rsi;
        signal
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 모멘텀 전략 (Price Momentum)
// ────────────────────────────────────────────────────────────────────

/// 모멘텀 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumParams {
    /// 비교 기간: N기간 전 가격 대비 현재가 변화율 측정 (기본 20)
    pub lookback_period: usize,
    /// 매매 발동 변화율 임계값 % (기본 5.0)
    pub threshold_pct: f64,
}

impl Default for MomentumParams {
    fn default() -> Self {
        Self {
            lookback_period: 20,
            threshold_pct: 5.0,
        }
    }
}

/// 종목별 모멘텀 상태
struct MomentumState {
    prices: VecDeque<u64>,
    last_buy_price: Option<u64>,
    last_sell_price: Option<u64>,
}

pub struct MomentumStrategy {
    config: StrategyConfig,
    params: MomentumParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, MomentumState>,
}

impl MomentumStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: MomentumParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for MomentumStrategy {
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
        let cap = bounded_window_with_extra(self.params.lookback_period, 1);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MomentumState {
                prices: VecDeque::with_capacity(cap),
                last_buy_price: None,
                last_sell_price: None,
            });
        state.prices = prices[prices.len().saturating_sub(cap)..]
            .iter()
            .copied()
            .collect();
    }

    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required_bars = bounded_window_with_extra(self.params.lookback_period, 1);
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars,
            available_bars: state.map_or(0, |state| state.prices.len()),
            ready: state.is_some_and(|state| {
                state.prices.len() >= required_bars
                    && state.prices.front().is_some_and(|price| *price > 0)
            }),
        })
    }

    // 현재 평가 가격을 추가한 뒤 지표를 계산하므로 마지막 준비 봉도 평가할 수 있다.
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        let required_bars = bounded_window_with_extra(self.params.lookback_period, 1);
        let available_bars = self
            .states
            .get(symbol)
            .map_or(0, |state| state.prices.len());
        Some(available_bars.saturating_add(1) >= required_bars)
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let cap = bounded_window_with_extra(self.params.lookback_period, 1);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MomentumState {
                prices: VecDeque::with_capacity(cap),
                last_buy_price: None,
                last_sell_price: None,
            });

        state.prices.push_back(price);
        if state.prices.len() > cap {
            state.prices.pop_front();
        }

        if state.prices.len() < cap {
            return Signal::Hold;
        }

        let past_price = *state.prices.front().unwrap();
        if past_price == 0 {
            return Signal::Hold;
        }

        let momentum_pct = (price as f64 - past_price as f64) / past_price as f64 * 100.0;
        let threshold = self.params.threshold_pct;

        if momentum_pct >= threshold {
            if let Some(last) = state.last_buy_price {
                let from_last = (price as f64 - last as f64) / last as f64 * 100.0;
                if from_last > -threshold {
                    return Signal::Hold;
                }
            }
            state.last_buy_price = Some(price);
            Signal::Buy {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("상승 모멘텀 +{:.1}%", momentum_pct),
            }
        } else if momentum_pct <= -threshold {
            if let Some(last) = state.last_sell_price {
                let from_last = (price as f64 - last as f64) / last as f64 * 100.0;
                if from_last < threshold {
                    return Signal::Hold;
                }
            }
            state.last_sell_price = Some(price);
            Signal::Sell {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("하락 모멘텀 {:.1}%", momentum_pct),
            }
        } else {
            Signal::Hold
        }
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let cap = bounded_window_with_extra(self.params.lookback_period, 1);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MomentumState {
                prices: VecDeque::with_capacity(cap),
                last_buy_price: None,
                last_sell_price: None,
            });
        state.last_buy_price = (quantity > 0).then_some(avg_price);
        state.last_sell_price = None;
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 이격도 전략 (Deviation Ratio from Moving Average)
// ────────────────────────────────────────────────────────────────────

/// 이격도 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationParams {
    /// 기준 이동평균 기간 (기본 20)
    pub ma_period: usize,
    /// 매수 발동 이격 기준 % — 현재가가 MA 대비 이 % 이하이면 매수 (기본 -5.0, 음수)
    pub buy_threshold_pct: f64,
    /// 매도 발동 이격 기준 % — 현재가가 MA 대비 이 % 이상이면 매도 (기본 5.0, 양수)
    pub sell_threshold_pct: f64,
}

impl Default for DeviationParams {
    fn default() -> Self {
        Self {
            ma_period: 20,
            buy_threshold_pct: -5.0,
            sell_threshold_pct: 5.0,
        }
    }
}

pub struct DeviationStrategy {
    config: StrategyConfig,
    params: DeviationParams,
    /// 종목코드 → 가격 이력
    states: std::collections::HashMap<String, VecDeque<u64>>,
}

impl DeviationStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: DeviationParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for DeviationStrategy {
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
        let cap = bounded_window(self.params.ma_period);
        self.states.insert(
            symbol.to_string(),
            prices[prices.len().saturating_sub(cap)..]
                .iter()
                .copied()
                .collect(),
        );
    }

    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required_bars = bounded_window(self.params.ma_period);
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars,
            available_bars: state.map_or(0, VecDeque::len),
            ready: state.is_some_and(|prices| {
                prices.len() >= required_bars && prices.iter().any(|price| *price > 0)
            }),
        })
    }

    // 현재 평가 가격을 추가한 뒤 지표를 계산하므로 마지막 준비 봉도 평가할 수 있다.
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        let required_bars = bounded_window(self.params.ma_period);
        let available_bars = self.states.get(symbol).map_or(0, VecDeque::len);
        Some(available_bars.saturating_add(1) >= required_bars)
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let cap = bounded_window(self.params.ma_period);
        let prices = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| VecDeque::with_capacity(cap));

        prices.push_back(price);
        if prices.len() > cap {
            prices.pop_front();
        }

        if prices.len() < cap {
            return Signal::Hold;
        }

        let ma: f64 = prices.iter().sum::<u64>() as f64 / prices.len() as f64;
        if ma == 0.0 {
            return Signal::Hold;
        }

        let deviation_pct = (price as f64 / ma - 1.0) * 100.0;

        if deviation_pct <= self.params.buy_threshold_pct {
            Signal::Buy {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("이격도 저점 {:.1}% (MA {:.0}원)", deviation_pct, ma),
            }
        } else if deviation_pct >= self.params.sell_threshold_pct {
            Signal::Sell {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("이격도 고점 +{:.1}% (MA {:.0}원)", deviation_pct, ma),
            }
        } else {
            Signal::Hold
        }
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;

    fn config(params: serde_json::Value) -> StrategyConfig {
        StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
    }

    #[test]
    fn rsi_initializer_restores_crossing_baseline_without_historical_signals() {
        let mut strategy = RsiStrategy::new(config(serde_json::json!({
            "period": 3, "oversold": 30.0, "overbought": 70.0,
        })));
        strategy.initialize_historical("SOXQ", &[100, 90, 80, 70]);
        assert_eq!(strategy.states["SOXQ"].1, Some(0.0));
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(
            strategy.on_tick("SOXQ", 120, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_historical("SOXQ", &[100, 90, 80]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
        assert_eq!(strategy.states["SOXQ"].1, None);
    }

    #[test]
    fn momentum_history_does_not_create_or_erase_order_state() {
        let mut strategy = MomentumStrategy::new(config(serde_json::json!({
            "lookback_period": 3, "threshold_pct": 5.0,
        })));
        strategy.initialize_historical("SOXQ", &[100, 100, 100, 100]);
        assert_eq!(strategy.states["SOXQ"].last_buy_price, None);
        assert!(matches!(
            strategy.on_tick("SOXQ", 110, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_historical("SOXQ", &[100, 100, 100, 100]);
        assert_eq!(strategy.states["SOXQ"].last_buy_price, Some(110));
        assert!(matches!(strategy.on_tick("SOXQ", 115, 0), Signal::Hold));
        strategy.initialize_historical("SOXQ", &[100, 100]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    }

    #[test]
    fn deviation_history_is_ready_before_first_evaluation_and_bounded() {
        let mut strategy = DeviationStrategy::new(config(serde_json::json!({
            "ma_period": 3, "buy_threshold_pct": -5.0, "sell_threshold_pct": 5.0,
        })));
        strategy.initialize_historical("SOXQ", &vec![100; 1000]);
        assert_eq!(strategy.states["SOXQ"].len(), 3);
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(
            strategy.on_tick("SOXQ", 50, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_intraday_prices("SOXQ", &[20, 20]);
        assert_eq!(strategy.states["SOXQ"].front(), Some(&20));
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    }
    #[test]
    fn momentum_counts_first_comparison_but_excludes_seed_prices() {
        let mut strategy = MomentumStrategy::new(config(serde_json::json!({
            "lookback_period": 3, "threshold_pct": 5.0,
        })));
        let mut evaluated = 0;
        for (index, price) in [100, 100, 100, 110].into_iter().enumerate() {
            let can_evaluate = strategy.can_evaluate_next_tick("SOXQ").unwrap();
            assert_eq!(can_evaluate, index == 3);
            evaluated += usize::from(can_evaluate);
            let signal = strategy.on_tick("SOXQ", price, 0);
            assert_eq!(matches!(signal, Signal::Buy { .. }), index == 3);
        }
        assert_eq!(evaluated, 1);
    }

    #[test]
    fn deviation_counts_current_price_as_last_required_observation() {
        let mut strategy = DeviationStrategy::new(config(serde_json::json!({
            "ma_period": 3, "buy_threshold_pct": -5.0, "sell_threshold_pct": 5.0,
        })));
        let mut evaluated = 0;
        for (index, price) in [100, 100, 50].into_iter().enumerate() {
            let can_evaluate = strategy.can_evaluate_next_tick("SOXQ").unwrap();
            assert_eq!(can_evaluate, index == 2);
            evaluated += usize::from(can_evaluate);
            let signal = strategy.on_tick("SOXQ", price, 0);
            assert_eq!(matches!(signal, Signal::Buy { .. }), index == 2);
        }
        assert_eq!(evaluated, 1);
        let mut flat = DeviationStrategy::new(config(serde_json::json!({
            "ma_period": 3, "buy_threshold_pct": -5.0, "sell_threshold_pct": 5.0,
        })));
        flat.initialize_historical("SOXQ", &[100, 100]);
        assert!(!flat.history_readiness("SOXQ").unwrap().ready);
        assert_eq!(flat.can_evaluate_next_tick("SOXQ"), Some(true));
        assert!(matches!(flat.on_tick("SOXQ", 100, 0), Signal::Hold));
    }
}
