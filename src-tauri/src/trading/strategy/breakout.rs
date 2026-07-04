use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::{
    state::{bounded_window, bounded_window_with_extra},
    Signal, Strategy, StrategyConfig,
};

// ────────────────────────────────────────────────────────────────────
// 52주 신고가 전략 (52-Week High Breakout)
// ────────────────────────────────────────────────────────────────────

/// 52주 신고가 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiftyTwoWeekHighParams {
    /// 조회 기간 (거래일 수, 기본 252 ≈ 1년)
    pub lookback_days: usize,
    /// 손절 기준 (매수가 대비 하락 %, 기본 3.0)
    pub stop_loss_pct: f64,
}

impl Default for FiftyTwoWeekHighParams {
    fn default() -> Self {
        Self {
            lookback_days: 252,
            stop_loss_pct: 3.0,
        }
    }
}

/// 종목별 52주 신고가 상태
struct FiftyTwoWeekState {
    prev_price: Option<u64>,
    high_52w: Option<u64>,
    buy_price: Option<u64>,
}

pub struct FiftyTwoWeekHighStrategy {
    config: StrategyConfig,
    params: FiftyTwoWeekHighParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, FiftyTwoWeekState>,
}

impl FiftyTwoWeekHighStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: FiftyTwoWeekHighParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for FiftyTwoWeekHighStrategy {
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
        let lookback = self.params.lookback_days.min(prices.len());
        if lookback < 2 {
            tracing::warn!(
                "52주 신고가 [{}]: 일봉 데이터 부족 ({}봉) — 전략 비활성",
                symbol,
                prices.len()
            );
            return;
        }
        let slice = &prices[prices.len().saturating_sub(lookback)..prices.len() - 1];
        if let Some(&h) = slice.iter().max() {
            if h > 0 {
                tracing::info!(
                    "52주 신고가 초기화 [{}]: {}원 (최근 {}거래일)",
                    symbol,
                    h,
                    slice.len()
                );
                let state = self
                    .states
                    .entry(symbol.to_string())
                    .or_insert(FiftyTwoWeekState {
                        prev_price: None,
                        high_52w: None,
                        buy_price: None,
                    });
                state.high_52w = Some(h);
            }
        }
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(FiftyTwoWeekState {
                prev_price: None,
                high_52w: None,
                buy_price: None,
            });

        let signal = match state.high_52w {
            None => Signal::Hold,
            Some(high) => {
                // ① 손절 체크
                if let Some(bp) = state.buy_price {
                    let stop_price = (bp as f64 * (1.0 - self.params.stop_loss_pct / 100.0)) as u64;
                    if price <= stop_price {
                        state.buy_price = None;
                        state.prev_price = Some(price);
                        return Signal::Sell {
                            symbol: symbol.to_string(),
                            quantity: self.config.order_quantity,
                            reason: format!(
                                "52주 신고가 손절: -{}% ({:.0}원 → {:.0}원)",
                                self.params.stop_loss_pct, bp as f64, price as f64
                            ),
                        };
                    }
                }
                // ② 52주 신고가 돌파 감지
                let crossed = state
                    .prev_price
                    .is_some_and(|prev| prev <= high && price > high);
                if crossed && state.buy_price.is_none() {
                    state.high_52w = Some(price);
                    state.buy_price = Some(price);
                    state.prev_price = Some(price);
                    return Signal::Buy {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!(
                            "52주 신고가 돌파: {:.0}원 (이전 고가 {:.0}원)",
                            price as f64, high as f64
                        ),
                    };
                }
                if price > high {
                    state.high_52w = Some(price);
                }
                Signal::Hold
            }
        };

        state.prev_price = Some(price);
        signal
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 연속 상승/하락 전략 (Consecutive Move)
// - N일 연속 종가 상승 → 매수
// - M일 연속 종가 하락 → 매도
// ────────────────────────────────────────────────────────────────────

/// 연속 상승/하락 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsecutiveMoveParams {
    /// 매수 발동 연속 상승 횟수 (기본 3)
    pub buy_days: usize,
    /// 매도 발동 연속 하락 횟수 (기본 3)
    pub sell_days: usize,
}

impl Default for ConsecutiveMoveParams {
    fn default() -> Self {
        Self {
            buy_days: 3,
            sell_days: 3,
        }
    }
}

/// 종목별 연속상승/하락 상태
struct ConsecutiveMoveState {
    prices: VecDeque<u64>,
    in_position: bool,
}

pub struct ConsecutiveMoveStrategy {
    config: StrategyConfig,
    params: ConsecutiveMoveParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, ConsecutiveMoveState>,
}

impl ConsecutiveMoveStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: ConsecutiveMoveParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }

    fn is_consecutive_up(prices: &VecDeque<u64>, n: usize) -> bool {
        if prices.len() < n + 1 {
            return false;
        }
        let slice: Vec<u64> = prices.iter().rev().take(n + 1).cloned().collect();
        (0..n).all(|i| slice[i] > slice[i + 1])
    }

    fn is_consecutive_down(prices: &VecDeque<u64>, n: usize) -> bool {
        if prices.len() < n + 1 {
            return false;
        }
        let slice: Vec<u64> = prices.iter().rev().take(n + 1).cloned().collect();
        (0..n).all(|i| slice[i] < slice[i + 1])
    }
}

impl Strategy for ConsecutiveMoveStrategy {
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

        let cap = bounded_window_with_extra(self.params.buy_days.max(self.params.sell_days), 1);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| ConsecutiveMoveState {
                prices: VecDeque::with_capacity(cap),
                in_position: false,
            });

        state.prices.push_back(price);
        if state.prices.len() > cap {
            state.prices.pop_front();
        }

        if state.in_position && Self::is_consecutive_down(&state.prices, self.params.sell_days) {
            state.in_position = false;
            return Signal::Sell {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("{}일 연속 하락 → 매도", self.params.sell_days),
            };
        }

        if !state.in_position && Self::is_consecutive_up(&state.prices, self.params.buy_days) {
            state.in_position = true;
            return Signal::Buy {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("{}일 연속 상승 → 매수", self.params.buy_days),
            };
        }

        Signal::Hold
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 06. 돌파 실패 전략 (FailedBreakoutStrategy)
// ────────────────────────────────────────────────────────────────────
// 동작:
//  1. 최근 lookback_days개 가격에서 전고점(prev_high) 계산
//  2. 현재가 ≥ prev_high × (1 + buffer_pct/100) → 전고점 돌파 → 매수
//  3. 매수 후 현재가 < 돌파 시점의 prev_high → 돌파 실패 → 매도
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedBreakoutParams {
    /// 전고점을 계산하기 위한 과거 기간 (기본 20)
    pub lookback_days: usize,
    /// 전고점 대비 돌파로 인정하는 버퍼 % (기본 0.5)
    pub buffer_pct: f64,
}

impl Default for FailedBreakoutParams {
    fn default() -> Self {
        Self {
            lookback_days: 20,
            buffer_pct: 0.5,
        }
    }
}

/// 종목별 돌파실패 상태
struct FailedBreakoutState {
    prices: VecDeque<u64>,
    in_position: bool,
    breakout_prev_high: Option<u64>,
}

pub struct FailedBreakoutStrategy {
    config: StrategyConfig,
    params: FailedBreakoutParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, FailedBreakoutState>,
}

impl FailedBreakoutStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: FailedBreakoutParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for FailedBreakoutStrategy {
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

        let lookback = bounded_window(self.params.lookback_days);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| FailedBreakoutState {
                prices: VecDeque::with_capacity(lookback),
                in_position: false,
                breakout_prev_high: None,
            });

        let prev_high = state.prices.iter().copied().max().unwrap_or(0);

        // ① 매도 우선: 돌파 실패
        if state.in_position {
            if let Some(ref_high) = state.breakout_prev_high {
                if price < ref_high {
                    state.in_position = false;
                    state.breakout_prev_high = None;
                    state.prices.push_back(price);
                    if state.prices.len() > lookback {
                        state.prices.pop_front();
                    }
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!("돌파 실패: 현재가 {} < 전고점 {} → 매도", price, ref_high),
                    };
                }
            }
        }

        // ② 매수: 전고점 돌파
        if !state.in_position && state.prices.len() >= lookback && prev_high > 0 {
            let breakout_threshold =
                (prev_high as f64 * (1.0 + self.params.buffer_pct / 100.0)) as u64;
            if price >= breakout_threshold {
                state.in_position = true;
                state.breakout_prev_high = Some(prev_high);
                state.prices.push_back(price);
                if state.prices.len() > lookback {
                    state.prices.pop_front();
                }
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity: self.config.order_quantity,
                    reason: format!(
                        "전고점 돌파 매수: {} ≥ {} (전고점 {} + {:.1}% 버퍼)",
                        price, breakout_threshold, prev_high, self.params.buffer_pct
                    ),
                };
            }
        }

        state.prices.push_back(price);
        if state.prices.len() > lookback {
            state.prices.pop_front();
        }

        Signal::Hold
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 07. 강한 종가 전략 (StrongCloseStrategy)
// ────────────────────────────────────────────────────────────────────
// 동작:
//  1. 자동매매 시작 시 `initialize_candles`로 일봉 (고가, 종가) 배열 전달
//  2. 전일 종가가 전일 고가 대비 threshold_pct% 이내이면 "강한 종가" → 다음날(당일) 매수 신호 대기
//  3. 당일 첫 틱 수신 시 매수 신호 발생 (1회 발생 후 pending 해제)
//  4. 매도 조건: 매수 후 현재가가 매수가 대비 stop_loss_pct% 이상 하락 시 손절
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrongCloseParams {
    /// 고가 대비 종가가 이 % 이내이면 강한 종가로 판단 (기본 3.0)
    pub threshold_pct: f64,
    /// 매수 후 손절 기준 % (기본 3.0)
    pub stop_loss_pct: f64,
}

impl Default for StrongCloseParams {
    fn default() -> Self {
        Self {
            threshold_pct: 3.0,
            stop_loss_pct: 3.0,
        }
    }
}

/// 종목별 강한종가 상태
struct StrongCloseState {
    pending_buy: bool,
    in_position: bool,
    entry_price: Option<u64>,
}

pub struct StrongCloseStrategy {
    config: StrategyConfig,
    params: StrongCloseParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, StrongCloseState>,
}

impl StrongCloseStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: StrongCloseParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for StrongCloseStrategy {
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

    fn initialize_candles(&mut self, symbol: &str, candles: &[(u64, u64)]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        if let Some(&(high, close)) = candles.last() {
            if high == 0 {
                return;
            }
            let gap_pct = (high as f64 - close as f64) / high as f64 * 100.0;
            if gap_pct <= self.params.threshold_pct {
                let state = self
                    .states
                    .entry(symbol.to_string())
                    .or_insert(StrongCloseState {
                        pending_buy: false,
                        in_position: false,
                        entry_price: None,
                    });
                state.pending_buy = true;
                tracing::info!(
                    "강한 종가 감지 ({}): 고가={}, 종가={}, 이격={:.2}% → 다음 틱 매수 대기",
                    symbol,
                    high,
                    close,
                    gap_pct
                );
            }
        }
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(StrongCloseState {
                pending_buy: false,
                in_position: false,
                entry_price: None,
            });

        // ① 손절 우선
        if state.in_position {
            if let Some(ep) = state.entry_price {
                let loss_pct = (ep as f64 - price as f64) / ep as f64 * 100.0;
                if loss_pct >= self.params.stop_loss_pct {
                    state.in_position = false;
                    state.entry_price = None;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!(
                            "강한 종가 손절: 현재가 {} (매수가 {} 대비 -{:.2}%)",
                            price, ep, loss_pct
                        ),
                    };
                }
            }
            return Signal::Hold;
        }

        // ② 강한 종가 후 첫 틱 매수
        if state.pending_buy {
            state.pending_buy = false;
            state.in_position = true;
            state.entry_price = Some(price);
            return Signal::Buy {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!(
                    "강한 종가 후 매수: 현재가 {} (전일 종가가 고가 대비 {:.1}% 이내)",
                    price, self.params.threshold_pct
                ),
            };
        }

        Signal::Hold
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 08. 변동성 확장 전략 (VolatilityExpansionStrategy)
// ────────────────────────────────────────────────────────────────────
// 동작:
//  1. 자동매매 시작 시 `initialize_range_data`로 일봉 변동폭(고-저) 배열 전달 → 평균 변동폭 계산
//  2. 장중 첫 틱 = 시가(day_open), 이후 틱마다 당일 고/저 추적
//  3. 당일 변동폭 > 평균 변동폭 × expansion_factor AND 현재가 > day_open → 매수 (변동성 방향 확인)
//  4. 매수 후 stop_loss_pct% 하락 시 손절 매도
// ────────────────────────────────────────────────────────────────────

/// 변동성 확장 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityExpansionParams {
    /// 평균 변동폭 계산에 사용할 과거 기간 (기본 10거래일)
    pub lookback_days: usize,
    /// 평균 변동폭 대비 확장 배율 (기본 2.0배 이상이면 발동)
    pub expansion_factor: f64,
    /// 매수 후 손절 기준 % (기본 3.0)
    pub stop_loss_pct: f64,
}

impl Default for VolatilityExpansionParams {
    fn default() -> Self {
        Self {
            lookback_days: 10,
            expansion_factor: 2.0,
            stop_loss_pct: 3.0,
        }
    }
}

/// 종목별 변동성 확장 상태
struct VolatilityExpansionState {
    avg_range: Option<f64>,
    day_open: Option<u64>,
    day_high: u64,
    day_low: u64,
    in_position: bool,
    entry_price: Option<u64>,
}

pub struct VolatilityExpansionStrategy {
    config: StrategyConfig,
    params: VolatilityExpansionParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, VolatilityExpansionState>,
}

impl VolatilityExpansionStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: VolatilityExpansionParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for VolatilityExpansionStrategy {
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

    fn initialize_range_data(&mut self, symbol: &str, ranges: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let lookback = self.params.lookback_days.min(ranges.len());
        if lookback == 0 {
            tracing::warn!(
                "변동성 확장 [{}]: 일봉 데이터 없음 — avg_range 미초기화",
                symbol
            );
            return;
        }
        let slice = &ranges[ranges.len().saturating_sub(lookback)..];
        let avg = slice.iter().sum::<u64>() as f64 / slice.len() as f64;
        tracing::info!(
            "변동성 확장 초기화 [{}]: 평균 변동폭 {:.0}원 (최근 {}거래일)",
            symbol,
            avg,
            slice.len()
        );
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(VolatilityExpansionState {
                avg_range: None,
                day_open: None,
                day_high: 0,
                day_low: u64::MAX,
                in_position: false,
                entry_price: None,
            });
        state.avg_range = Some(avg);
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(VolatilityExpansionState {
                avg_range: None,
                day_open: None,
                day_high: 0,
                day_low: u64::MAX,
                in_position: false,
                entry_price: None,
            });

        if price > state.day_high {
            state.day_high = price;
        }
        if price < state.day_low {
            state.day_low = price;
        }
        if state.day_open.is_none() {
            state.day_open = Some(price);
        }

        // ① 손절 우선
        if state.in_position {
            if let Some(ep) = state.entry_price {
                let loss_pct = (ep as f64 - price as f64) / ep as f64 * 100.0;
                if loss_pct >= self.params.stop_loss_pct {
                    state.in_position = false;
                    state.entry_price = None;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!(
                            "변동성 확장 손절: 현재가 {} (매수가 {} 대비 -{:.2}%)",
                            price, ep, loss_pct
                        ),
                    };
                }
            }
            return Signal::Hold;
        }

        // ② 매수 조건
        if let (Some(ar), Some(day_open)) = (state.avg_range, state.day_open) {
            if state.day_low == u64::MAX {
                return Signal::Hold;
            }
            let intraday_range = state.day_high.saturating_sub(state.day_low);
            let threshold = ar * self.params.expansion_factor;
            if intraday_range as f64 > threshold && price > day_open {
                state.in_position = true;
                state.entry_price = Some(price);
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity: self.config.order_quantity,
                    reason: format!(
                        "변동성 확장 매수: 당일 변동폭 {}원 > 평균 {:.0}원 × {:.1} (상승 방향)",
                        intraday_range, ar, self.params.expansion_factor
                    ),
                };
            }
        }

        Signal::Hold
    }

    fn reset(&mut self) {
        // 일 초기화: 당일 고/저/시가 리셋, avg_range는 유지
        for state in self.states.values_mut() {
            state.day_open = None;
            state.day_high = 0;
            state.day_low = u64::MAX;
            state.in_position = false;
            state.entry_price = None;
        }
    }
}
