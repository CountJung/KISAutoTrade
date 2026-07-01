use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// 매매 신호
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Signal {
    /// 매수 신호
    Buy {
        symbol: String,
        quantity: u64,
        reason: String,
    },
    /// 매도 신호
    Sell {
        symbol: String,
        quantity: u64,
        reason: String,
    },
    /// 관망
    Hold,
}

/// 전략 설정 (JSON 직렬화 가능)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub target_symbols: Vec<String>,
    /// 1회 주문 수량
    pub order_quantity: u64,
    // 전략별 파라미터
    pub params: serde_json::Value,
}

/// 전략 초기화용 OHLC 캔들.
#[derive(Debug, Clone, Copy)]
pub struct OhlcCandle {
    pub open: u64,
    pub high: u64,
    pub low: u64,
    pub close: u64,
}

/// 전략 trait — 모든 자동매매 전략이 구현해야 함
pub trait Strategy: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn config(&self) -> &StrategyConfig;
    fn config_mut(&mut self) -> &mut StrategyConfig;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    /// 틱 데이터를 받아 매매 신호 반환
    fn on_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Signal;
    /// 전략 시작 시 일봉 가격 배열로 초기화. 히스토리가 필요 없는 전략은 기본 no-op.
    fn initialize_historical(&mut self, _symbol: &str, _prices: &[u64]) {}
    /// 전략 시작 시 일봉 (고가, 종가) 쌍 배열로 초기화. 강한 종가 등 복합 일봉 데이터가 필요한 전략에서 재정의.
    fn initialize_candles(&mut self, _symbol: &str, _candles: &[(u64, u64)]) {}
    /// 전략 시작 시 일봉 OHLC 배열로 초기화. ADX/갭/양봉 판단이 필요한 전략에서 재정의.
    fn initialize_ohlc(&mut self, _symbol: &str, _candles: &[OhlcCandle]) {}
    /// 전략 시작 시 일봉 변동 범위(고가-저가) 배열로 초기화. 변동성 확장 전략에서 사용.
    fn initialize_range_data(&mut self, _symbol: &str, _ranges: &[u64]) {}
    /// 자동매매 시작 시 실제 잔고 기반으로 전략 내부 포지션 플래그를 동기화한다.
    fn sync_position(&mut self, _symbol: &str, _quantity: u64, _avg_price: u64) {}
    /// 전략 상태 초기화 (일 초기화 등)
    fn reset(&mut self);
}

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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return Signal::Hold;
        }

        let cap = self.params.long_period + 1;
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

pub struct StrategyManager {
    strategies: Vec<Box<dyn Strategy>>,
}

impl StrategyManager {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn add(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }

    pub fn on_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Vec<Signal> {
        self.strategies
            .iter_mut()
            .map(|s| s.on_tick(symbol, price, volume))
            .filter(|sig| *sig != Signal::Hold)
            .collect()
    }

    pub fn reset_all(&mut self) {
        self.strategies.iter_mut().for_each(|s| s.reset());
    }

    pub fn active_names(&self) -> Vec<String> {
        self.strategies
            .iter()
            .filter(|s| s.is_enabled())
            .map(|s| s.name().to_string())
            .collect()
    }

    /// 활성 전략에 등록된 구독 종목 코드 목록 (중복 제거)
    pub fn active_symbols(&self) -> Vec<String> {
        let mut symbols: Vec<String> = self
            .strategies
            .iter()
            .filter(|s| s.is_enabled())
            .flat_map(|s| s.config().target_symbols.clone())
            .collect();
        symbols.sort_unstable();
        symbols.dedup();
        symbols
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 가격 데이터 전달 (52주 신고가 등 히스토리 기반 전략 초기화)
    pub fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        for s in &mut self.strategies {
            if s.config().target_symbols.contains(&symbol.to_string()) {
                s.initialize_historical(symbol, prices);
            }
        }
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 (고가, 종가) 쌍 데이터 전달 (강한 종가 등)
    pub fn initialize_candles(&mut self, symbol: &str, candles: &[(u64, u64)]) {
        for s in &mut self.strategies {
            if s.config().target_symbols.contains(&symbol.to_string()) {
                s.initialize_candles(symbol, candles);
            }
        }
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 OHLC 데이터 전달 (ADX/갭/양봉 판단)
    pub fn initialize_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        for s in &mut self.strategies {
            if s.config().target_symbols.contains(&symbol.to_string()) {
                s.initialize_ohlc(symbol, candles);
            }
        }
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 변동 범위(고가-저가) 데이터 전달 (변동성 확장 전략)
    pub fn initialize_range_data(&mut self, symbol: &str, ranges: &[u64]) {
        for s in &mut self.strategies {
            if s.config().target_symbols.contains(&symbol.to_string()) {
                s.initialize_range_data(symbol, ranges);
            }
        }
    }

    /// 실제 잔고를 전략별 내부 포지션 상태에 반영한다.
    pub fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        for s in &mut self.strategies {
            s.sync_position(symbol, quantity, avg_price);
        }
    }

    /// 저장된 전략 설정으로 인메모리 전략 상태 업데이트 (프로그램 재시작 또는 프로필 전환 후 복원)
    ///
    /// 모든 전략을 기본값(비활성화, 종목 없음)으로 먼저 리셋한 뒤 저장된 설정을 덮어씀.
    /// 저장된 설정이 없는 프로필로 전환할 때 이전 프로필 종목이 잔류하는 버그를 방지함.
    pub fn apply_saved_configs(&mut self, saved: &[StrategyConfig]) {
        // 1) 모든 전략 기본값으로 초기화 (프로필 전환 시 이전 상태 잔류 방지)
        for s in &mut self.strategies {
            let cfg = s.config_mut();
            cfg.enabled = false;
            cfg.target_symbols = Vec::new();
        }
        // 2) 저장된 설정 적용
        for saved_cfg in saved {
            if let Some(cfg) = self.get_config_mut(&saved_cfg.id) {
                cfg.enabled = saved_cfg.enabled;
                cfg.target_symbols = saved_cfg.target_symbols.clone();
                cfg.order_quantity = saved_cfg.order_quantity;
                cfg.params = saved_cfg.params.clone();
            }
        }
    }

    /// 전체 전략 설정 반환
    pub fn all_configs(&self) -> Vec<&StrategyConfig> {
        self.strategies.iter().map(|s| s.config()).collect()
    }

    /// 특정 ID의 전략 설정 가변 참조 반환
    pub fn get_config_mut(&mut self, id: &str) -> Option<&mut StrategyConfig> {
        self.strategies
            .iter_mut()
            .find(|s| s.id() == id)
            .map(|s| s.config_mut())
    }
}

impl Default for StrategyManager {
    fn default() -> Self {
        Self::new()
    }
}

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
        if prices.len() < period + 1 {
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

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return Signal::Hold;
        }

        let cap = self.params.period + 2;
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

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return Signal::Hold;
        }

        let cap = self.params.lookback_period + 1;
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

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return Signal::Hold;
        }

        let cap = self.params.ma_period;
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
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
                    .map_or(false, |prev| prev <= high && price > high);
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return Signal::Hold;
        }

        let cap = self.params.buy_days.max(self.params.sell_days) + 1;
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return Signal::Hold;
        }

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| FailedBreakoutState {
                prices: VecDeque::new(),
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
                    if state.prices.len() > self.params.lookback_days {
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
        if !state.in_position && state.prices.len() >= self.params.lookback_days && prev_high > 0 {
            let breakout_threshold =
                (prev_high as f64 * (1.0 + self.params.buffer_pct / 100.0)) as u64;
            if price >= breakout_threshold {
                state.in_position = true;
                state.breakout_prev_high = Some(prev_high);
                state.prices.push_back(price);
                if state.prices.len() > self.params.lookback_days {
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
        if state.prices.len() > self.params.lookback_days {
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return;
        }
        let n = self.params.period as usize;
        let take = prices.len().min(n);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MeanReversionState {
                prices: VecDeque::with_capacity(n + 1),
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return Signal::Hold;
        }

        let period = self.params.period as usize;
        let std_dev = self.params.std_dev;
        let stop_loss = self.params.stop_loss_pct;
        let qty = self.config.order_quantity;

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MeanReversionState {
                prices: VecDeque::with_capacity(period + 1),
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return;
        }
        let n = self.params.long_period as usize;
        let take = prices.len().min(n);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| TrendFilterState {
                prices: VecDeque::with_capacity(n + 1),
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
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return Signal::Hold;
        }

        let max_cap = (self.params.long_period as usize) + 1;
        let long_p = self.params.long_period as usize;
        let mid_p = self.params.mid_period as usize;
        let short_p = self.params.short_period as usize;
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

    fn reset(&mut self) {
        // 가격 버퍼 유지, 포지션만 초기화
        for state in self.states.values_mut() {
            state.in_position = false;
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 11. 레버리지 추세 보유 전략 (LeveragedTrendHoldStrategy)
// ────────────────────────────────────────────────────────────────────
// 기초 ETF(SOXX/SMH 등)의 추세 조건이 좋을 때 레버리지 ETF(SOXL 등)를 매수하고,
// 레버리지 가격의 고점 대비 하락 또는 기초 ETF 추세 훼손 시 청산한다.
// target_symbols에는 기초 종목과 레버리지 종목이 모두 들어간다.
// ────────────────────────────────────────────────────────────────────

fn lth_default_qty() -> u64 {
    1
}
fn lth_default_ema_short() -> usize {
    20
}
fn lth_default_ema_long() -> usize {
    60
}
fn lth_default_rsi_period() -> usize {
    14
}
fn lth_default_adx_period() -> usize {
    14
}
fn lth_default_buy_rsi() -> f64 {
    55.0
}
fn lth_default_sell_rsi() -> f64 {
    50.0
}
fn lth_default_buy_adx() -> f64 {
    20.0
}
fn lth_default_no_trade_adx() -> f64 {
    18.0
}
fn lth_default_neutral_low() -> f64 {
    45.0
}
fn lth_default_neutral_high() -> f64 {
    55.0
}
fn lth_default_trailing_stop() -> f64 {
    1.5
}
fn lth_default_entry_start() -> i64 {
    15
}
fn lth_default_entry_end() -> i64 {
    30
}
fn lth_default_exit_before_close() -> i64 {
    20
}
fn lth_default_gap_limit() -> f64 {
    4.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeveragedTrendHoldEntry {
    /// 매수/청산 대상 레버리지 종목 (예: SOXL)
    pub leveraged_symbol: String,
    /// UI 표시용 종목명
    #[serde(default)]
    pub leveraged_symbol_name: String,
    /// 하락 추세에서 매수할 역방향 레버리지 종목 (예: SOXS). 비어 있으면 비활성.
    #[serde(default)]
    pub inverse_leveraged_symbol: String,
    /// 역방향 레버리지 종목명 (UI 표시용)
    #[serde(default)]
    pub inverse_leveraged_symbol_name: String,
    /// 추세 판단에 사용할 기초 종목들 (예: SOXX, SMH). 하나라도 통과하면 진입 가능.
    #[serde(default)]
    pub base_symbols: Vec<String>,
    /// 기초 종목명 캐시 (UI 표시용)
    #[serde(default)]
    pub base_symbol_names: HashMap<String, String>,
    /// 1회 주문 수량
    #[serde(default = "lth_default_qty")]
    pub quantity: u64,
    /// 역방향 레버리지 1회 주문 수량
    #[serde(default = "lth_default_qty")]
    pub inverse_quantity: u64,
    /// 해외 주식 여부. true이면 가격 단위 = USD, on_tick 내부 가격은 cents.
    #[serde(default)]
    pub is_overseas: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeveragedTrendHoldParams {
    #[serde(default)]
    pub entries: Vec<LeveragedTrendHoldEntry>,
    #[serde(default = "lth_default_ema_short")]
    pub ema_short_period: usize,
    #[serde(default = "lth_default_ema_long")]
    pub ema_long_period: usize,
    #[serde(default = "lth_default_rsi_period")]
    pub rsi_period: usize,
    #[serde(default = "lth_default_adx_period")]
    pub adx_period: usize,
    #[serde(default = "lth_default_buy_rsi")]
    pub entry_rsi_min: f64,
    #[serde(default = "lth_default_sell_rsi")]
    pub exit_rsi_below: f64,
    #[serde(default = "lth_default_buy_adx")]
    pub entry_adx_min: f64,
    #[serde(default = "lth_default_no_trade_adx")]
    pub no_trade_adx_below: f64,
    #[serde(default = "lth_default_neutral_low")]
    pub neutral_rsi_low: f64,
    #[serde(default = "lth_default_neutral_high")]
    pub neutral_rsi_high: f64,
    #[serde(default = "lth_default_trailing_stop")]
    pub trailing_stop_pct: f64,
    #[serde(default = "lth_default_entry_start")]
    pub entry_window_start_min: i64,
    #[serde(default = "lth_default_entry_end")]
    pub entry_window_end_min: i64,
    #[serde(default = "lth_default_exit_before_close")]
    pub exit_before_close_min: i64,
    #[serde(default = "lth_default_gap_limit")]
    pub max_gap_pct: f64,
    /// 주요 지표 발표 전후 등 수동 거래 금지 구간. 예: ["23:25-23:45", "02:55-03:10"]
    #[serde(default)]
    pub blackout_windows: Vec<String>,
}

impl Default for LeveragedTrendHoldParams {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            ema_short_period: lth_default_ema_short(),
            ema_long_period: lth_default_ema_long(),
            rsi_period: lth_default_rsi_period(),
            adx_period: lth_default_adx_period(),
            entry_rsi_min: lth_default_buy_rsi(),
            exit_rsi_below: lth_default_sell_rsi(),
            entry_adx_min: lth_default_buy_adx(),
            no_trade_adx_below: lth_default_no_trade_adx(),
            neutral_rsi_low: lth_default_neutral_low(),
            neutral_rsi_high: lth_default_neutral_high(),
            trailing_stop_pct: lth_default_trailing_stop(),
            entry_window_start_min: lth_default_entry_start(),
            entry_window_end_min: lth_default_entry_end(),
            exit_before_close_min: lth_default_exit_before_close(),
            max_gap_pct: lth_default_gap_limit(),
            blackout_windows: Vec::new(),
        }
    }
}

struct LeveragedTrendHoldMarketState {
    candles: VecDeque<OhlcCandle>,
    live_candle_started: bool,
}

struct LeveragedTrendHoldPosition {
    in_position: bool,
    entry_price: Option<u64>,
    high_water: Option<u64>,
}

struct LeveragedTrendSnapshot {
    ema_short: f64,
    ema_long: f64,
    rsi: f64,
    adx: f64,
    bullish_count_3: usize,
    bearish_count_3: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeveragedTrendDirection {
    Long,
    Inverse,
}

pub struct LeveragedTrendHoldStrategy {
    config: StrategyConfig,
    params: LeveragedTrendHoldParams,
    base_states: HashMap<String, LeveragedTrendHoldMarketState>,
    positions: HashMap<String, LeveragedTrendHoldPosition>,
    last_params: serde_json::Value,
}

impl LeveragedTrendHoldStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: LeveragedTrendHoldParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        let last_params = config.params.clone();
        Self {
            config,
            params,
            base_states: HashMap::new(),
            positions: HashMap::new(),
            last_params,
        }
    }

    fn sync_params(&mut self) {
        if self.config.params == self.last_params {
            return;
        }
        self.params = serde_json::from_value(self.config.params.clone()).unwrap_or_default();
        self.last_params = self.config.params.clone();
        let mut symbols = Vec::new();
        for entry in &self.params.entries {
            symbols.push(entry.leveraged_symbol.clone());
            if !entry.inverse_leveraged_symbol.trim().is_empty() {
                symbols.push(entry.inverse_leveraged_symbol.clone());
            }
            symbols.extend(entry.base_symbols.iter().cloned());
        }
        symbols.retain(|s| !s.trim().is_empty());
        symbols.sort_unstable();
        symbols.dedup();
        self.config.target_symbols = symbols;
    }

    fn entries_for_symbol(
        &self,
        symbol: &str,
    ) -> Vec<(LeveragedTrendHoldEntry, LeveragedTrendDirection)> {
        self.params
            .entries
            .iter()
            .filter_map(|entry| {
                if entry.base_symbols.is_empty() {
                    return None;
                }
                if entry.leveraged_symbol == symbol {
                    Some((entry.clone(), LeveragedTrendDirection::Long))
                } else if entry.inverse_leveraged_symbol == symbol {
                    Some((entry.clone(), LeveragedTrendDirection::Inverse))
                } else {
                    None
                }
            })
            .collect()
    }

    fn is_base_symbol(&self, symbol: &str) -> bool {
        self.params
            .entries
            .iter()
            .any(|e| e.base_symbols.iter().any(|b| b == symbol))
    }

    fn update_base_tick(&mut self, symbol: &str, price: u64) {
        let cap = self
            .params
            .ema_long_period
            .max(self.params.adx_period + 2)
            .max(80)
            + 5;
        let state = self
            .base_states
            .entry(symbol.to_string())
            .or_insert_with(|| LeveragedTrendHoldMarketState {
                candles: VecDeque::with_capacity(cap),
                live_candle_started: false,
            });

        if !state.live_candle_started {
            state.candles.push_back(OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            });
            state.live_candle_started = true;
        } else if let Some(last) = state.candles.back_mut() {
            last.high = last.high.max(price);
            last.low = last.low.min(price);
            last.close = price;
        }

        while state.candles.len() > cap {
            state.candles.pop_front();
        }
    }

    fn closes(candles: &VecDeque<OhlcCandle>) -> Vec<f64> {
        candles.iter().map(|c| c.close as f64).collect()
    }

    fn ema(values: &[f64], period: usize) -> Option<f64> {
        if values.len() < period || period == 0 {
            return None;
        }
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = values[0];
        for value in &values[1..] {
            ema = value * alpha + ema * (1.0 - alpha);
        }
        Some(ema)
    }

    fn rsi(values: &[f64], period: usize) -> Option<f64> {
        if values.len() < period + 1 || period == 0 {
            return None;
        }
        let start = values.len() - period - 1;
        let mut gains = 0.0;
        let mut losses = 0.0;
        for pair in values[start..].windows(2) {
            let diff = pair[1] - pair[0];
            if diff >= 0.0 {
                gains += diff;
            } else {
                losses += -diff;
            }
        }
        if losses == 0.0 {
            return Some(100.0);
        }
        let rs = (gains / period as f64) / (losses / period as f64);
        Some(100.0 - 100.0 / (1.0 + rs))
    }

    fn adx(candles: &VecDeque<OhlcCandle>, period: usize) -> Option<f64> {
        if candles.len() < period + 1 || period == 0 {
            return None;
        }
        let start = candles.len() - period - 1;
        let slice: Vec<OhlcCandle> = candles.iter().skip(start).copied().collect();
        let mut tr_sum = 0.0;
        let mut plus_dm_sum = 0.0;
        let mut minus_dm_sum = 0.0;

        for pair in slice.windows(2) {
            let prev = pair[0];
            let cur = pair[1];
            let high_diff = cur.high as f64 - prev.high as f64;
            let low_diff = prev.low as f64 - cur.low as f64;
            let plus_dm = if high_diff > low_diff && high_diff > 0.0 {
                high_diff
            } else {
                0.0
            };
            let minus_dm = if low_diff > high_diff && low_diff > 0.0 {
                low_diff
            } else {
                0.0
            };
            let high_low = cur.high.saturating_sub(cur.low) as f64;
            let high_close = (cur.high as f64 - prev.close as f64).abs();
            let low_close = (cur.low as f64 - prev.close as f64).abs();
            tr_sum += high_low.max(high_close).max(low_close);
            plus_dm_sum += plus_dm;
            minus_dm_sum += minus_dm;
        }

        if tr_sum == 0.0 {
            return Some(0.0);
        }
        let plus_di = 100.0 * plus_dm_sum / tr_sum;
        let minus_di = 100.0 * minus_dm_sum / tr_sum;
        let denom = plus_di + minus_di;
        if denom == 0.0 {
            return Some(0.0);
        }
        Some(100.0 * (plus_di - minus_di).abs() / denom)
    }

    fn bullish_count(candles: &VecDeque<OhlcCandle>, count: usize) -> usize {
        candles
            .iter()
            .rev()
            .take(count)
            .filter(|c| c.close > c.open)
            .count()
    }

    fn bearish_count(candles: &VecDeque<OhlcCandle>, count: usize) -> usize {
        candles
            .iter()
            .rev()
            .take(count)
            .filter(|c| c.close < c.open)
            .count()
    }

    fn gap_pct(candles: &VecDeque<OhlcCandle>) -> Option<f64> {
        if candles.len() < 2 {
            return None;
        }
        let cur = candles.back()?;
        let prev = candles.iter().rev().nth(1)?;
        if prev.close == 0 {
            return None;
        }
        Some((cur.open as f64 - prev.close as f64).abs() / prev.close as f64 * 100.0)
    }

    fn snapshot_for(&self, base_symbol: &str) -> Option<LeveragedTrendSnapshot> {
        let state = self.base_states.get(base_symbol)?;
        let closes = Self::closes(&state.candles);
        let ema_short = Self::ema(&closes, self.params.ema_short_period)?;
        let ema_long = Self::ema(&closes, self.params.ema_long_period)?;
        let rsi = Self::rsi(&closes, self.params.rsi_period)?;
        let adx = Self::adx(&state.candles, self.params.adx_period)?;
        Some(LeveragedTrendSnapshot {
            ema_short,
            ema_long,
            rsi,
            adx,
            bullish_count_3: Self::bullish_count(&state.candles, 3),
            bearish_count_3: Self::bearish_count(&state.candles, 3),
        })
    }

    fn base_entry_ok(
        &self,
        base_symbol: &str,
        direction: LeveragedTrendDirection,
    ) -> Option<LeveragedTrendSnapshot> {
        let state = self.base_states.get(base_symbol)?;
        let snap = self.snapshot_for(base_symbol)?;
        let close = state.candles.back()?.close as f64;
        let gap_ok = Self::gap_pct(&state.candles)
            .map(|g| g <= self.params.max_gap_pct)
            .unwrap_or(true);
        let neutral_rsi =
            snap.rsi >= self.params.neutral_rsi_low && snap.rsi <= self.params.neutral_rsi_high;

        if !gap_ok || neutral_rsi || snap.adx < self.params.no_trade_adx_below {
            return None;
        }

        let trend_ok = match direction {
            LeveragedTrendDirection::Long => {
                close > snap.ema_short
                    && snap.ema_short > snap.ema_long
                    && snap.rsi >= self.params.entry_rsi_min
                    && snap.bullish_count_3 >= 2
            }
            LeveragedTrendDirection::Inverse => {
                close < snap.ema_short
                    && snap.ema_short < snap.ema_long
                    && snap.rsi <= self.params.neutral_rsi_low
                    && snap.bearish_count_3 >= 2
            }
        };

        if trend_ok && snap.adx >= self.params.entry_adx_min {
            return Some(snap);
        }

        None
    }

    fn base_exit_reason(
        &self,
        base_symbol: &str,
        direction: LeveragedTrendDirection,
    ) -> Option<String> {
        let state = self.base_states.get(base_symbol)?;
        let snap = self.snapshot_for(base_symbol)?;
        let close = state.candles.back()?.close as f64;

        match direction {
            LeveragedTrendDirection::Long => {
                if close < snap.ema_short {
                    return Some(format!("{} EMA20 하향 이탈", base_symbol));
                }
                if snap.rsi < self.params.exit_rsi_below {
                    return Some(format!(
                        "{} RSI {:.1} < {:.1}",
                        base_symbol, snap.rsi, self.params.exit_rsi_below
                    ));
                }
            }
            LeveragedTrendDirection::Inverse => {
                if close > snap.ema_short {
                    return Some(format!("{} EMA20 상향 회복", base_symbol));
                }
                let inverse_exit_rsi = 100.0 - self.params.exit_rsi_below;
                if snap.rsi > inverse_exit_rsi {
                    return Some(format!(
                        "{} RSI {:.1} > {:.1}",
                        base_symbol, snap.rsi, inverse_exit_rsi
                    ));
                }
            }
        }

        None
    }

    fn session_minutes(is_overseas: bool) -> Option<(i64, i64)> {
        use chrono::Timelike;
        let now = chrono::Local::now();
        let mins = now.hour() as i64 * 60 + now.minute() as i64;
        if is_overseas {
            let open = 22 * 60 + 30;
            let close = 5 * 60;
            if mins >= open {
                Some((mins - open, (24 * 60 - mins) + close))
            } else if mins < close {
                Some(((24 * 60 - open) + mins, close - mins))
            } else {
                None
            }
        } else {
            let open = 9 * 60;
            let close = 15 * 60 + 30;
            if mins >= open && mins < close {
                Some((mins - open, close - mins))
            } else {
                None
            }
        }
    }

    fn in_blackout_window(windows: &[String]) -> bool {
        use chrono::Timelike;
        let now = chrono::Local::now();
        let mins = now.hour() as i64 * 60 + now.minute() as i64;
        windows.iter().any(|w| {
            let Some((start, end)) = w.split_once('-') else {
                return false;
            };
            let Some(s) = parse_hhmm(start) else {
                return false;
            };
            let Some(e) = parse_hhmm(end) else {
                return false;
            };
            if s <= e {
                mins >= s && mins <= e
            } else {
                mins >= s || mins <= e
            }
        })
    }
}

fn parse_hhmm(value: &str) -> Option<i64> {
    let (h, m) = value.trim().split_once(':')?;
    let h = h.parse::<i64>().ok()?;
    let m = m.parse::<i64>().ok()?;
    if (0..24).contains(&h) && (0..60).contains(&m) {
        Some(h * 60 + m)
    } else {
        None
    }
}

impl Strategy for LeveragedTrendHoldStrategy {
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

    fn initialize_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        self.sync_params();
        if !self.is_base_symbol(symbol) {
            return;
        }
        let cap = self
            .params
            .ema_long_period
            .max(self.params.adx_period + 2)
            .max(80)
            + 5;
        let mut state = LeveragedTrendHoldMarketState {
            candles: VecDeque::with_capacity(cap),
            live_candle_started: false,
        };
        let take = candles.len().min(cap);
        for candle in &candles[candles.len().saturating_sub(take)..] {
            state.candles.push_back(*candle);
        }
        self.base_states.insert(symbol.to_string(), state);
        tracing::info!(
            "레버리지 추세 보유 초기화 [{}]: OHLC {}봉 로드",
            symbol,
            take
        );
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        self.sync_params();

        if self.is_base_symbol(symbol) {
            self.update_base_tick(symbol, price);
            return Signal::Hold;
        }

        let entries = self.entries_for_symbol(symbol);
        if entries.is_empty() {
            return Signal::Hold;
        }

        for (entry, direction) in entries {
            let quantity = match direction {
                LeveragedTrendDirection::Long => entry.quantity,
                LeveragedTrendDirection::Inverse => entry.inverse_quantity,
            };
            let direction_label = match direction {
                LeveragedTrendDirection::Long => "정방향",
                LeveragedTrendDirection::Inverse => "역방향",
            };

            let (in_position, high_water) = self
                .positions
                .get(symbol)
                .map(|p| (p.in_position, p.high_water))
                .unwrap_or((false, None));

            if in_position {
                let high = high_water.unwrap_or(price).max(price);
                if let Some(pos) = self.positions.get_mut(symbol) {
                    pos.high_water = Some(high);
                }
                if high > 0 {
                    let drawdown = (high as f64 - price as f64) / high as f64 * 100.0;
                    if drawdown >= self.params.trailing_stop_pct {
                        if let Some(pos) = self.positions.get_mut(symbol) {
                            pos.in_position = false;
                            pos.entry_price = None;
                            pos.high_water = None;
                        }
                        return Signal::Sell {
                            symbol: symbol.to_string(),
                            quantity,
                            reason: format!(
                                "LeveragedTrendHold {} 추적손절: 고점 대비 -{:.2}% (기준 {:.2}%)",
                                direction_label, drawdown, self.params.trailing_stop_pct
                            ),
                        };
                    }
                }

                let base_exit_reason = entry
                    .base_symbols
                    .iter()
                    .filter_map(|base| self.base_exit_reason(base, direction))
                    .next();
                if let Some(reason) = base_exit_reason {
                    if let Some(pos) = self.positions.get_mut(symbol) {
                        pos.in_position = false;
                        pos.entry_price = None;
                        pos.high_water = None;
                    }
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity,
                        reason: format!(
                            "LeveragedTrendHold {} 추세 청산: {}",
                            direction_label, reason
                        ),
                    };
                }

                if let Some((_, minutes_to_close)) = Self::session_minutes(entry.is_overseas) {
                    if minutes_to_close <= self.params.exit_before_close_min {
                        if let Some(pos) = self.positions.get_mut(symbol) {
                            pos.in_position = false;
                            pos.entry_price = None;
                            pos.high_water = None;
                        }
                        return Signal::Sell {
                            symbol: symbol.to_string(),
                            quantity,
                            reason: format!(
                                "LeveragedTrendHold {} 장마감 청산: 마감 {}분 전",
                                direction_label, minutes_to_close
                            ),
                        };
                    }
                }

                continue;
            }

            let Some((elapsed, _)) = Self::session_minutes(entry.is_overseas) else {
                continue;
            };
            if elapsed < self.params.entry_window_start_min
                || elapsed > self.params.entry_window_end_min
                || Self::in_blackout_window(&self.params.blackout_windows)
            {
                continue;
            }

            let base_entry = entry
                .base_symbols
                .iter()
                .filter_map(|base| self.base_entry_ok(base, direction).map(|snap| (base, snap)))
                .next();

            if let Some((base, snap)) = base_entry {
                self.positions.insert(
                    symbol.to_string(),
                    LeveragedTrendHoldPosition {
                        in_position: true,
                        entry_price: Some(price),
                        high_water: Some(price),
                    },
                );
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity,
                    reason: match direction {
                        LeveragedTrendDirection::Long => format!(
                            "LeveragedTrendHold 정방향 진입: {} EMA{} > EMA{}, RSI {:.1}, ADX {:.1}, 최근 3봉 양봉 {}개",
                            base,
                            self.params.ema_short_period,
                            self.params.ema_long_period,
                            snap.rsi,
                            snap.adx,
                            snap.bullish_count_3
                        ),
                        LeveragedTrendDirection::Inverse => format!(
                            "LeveragedTrendHold 역방향 진입: {} EMA{} < EMA{}, RSI {:.1}, ADX {:.1}, 최근 3봉 음봉 {}개",
                            base,
                            self.params.ema_short_period,
                            self.params.ema_long_period,
                            snap.rsi,
                            snap.adx,
                            snap.bearish_count_3
                        ),
                    },
                };
            }
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        self.sync_params();
        if quantity == 0
            || !self
                .params
                .entries
                .iter()
                .any(|e| e.leveraged_symbol == symbol || e.inverse_leveraged_symbol == symbol)
        {
            return;
        }
        self.positions.insert(
            symbol.to_string(),
            LeveragedTrendHoldPosition {
                in_position: true,
                entry_price: Some(avg_price),
                high_water: Some(avg_price),
            },
        );
        tracing::info!(
            "레버리지 추세 보유 포지션 동기화: {} {}주 @ {}",
            symbol,
            quantity,
            avg_price
        );
    }

    fn reset(&mut self) {
        for state in self.base_states.values_mut() {
            state.live_candle_started = false;
        }
        for pos in self.positions.values_mut() {
            pos.in_position = false;
            pos.entry_price = None;
            pos.high_water = None;
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 11. 가격 조건 매매 전략 (PriceConditionStrategy) — 종목별 독립 설정
// ────────────────────────────────────────────────────────────────────
// 각 종목마다 매수가·익절가·익절%·손절%·수량을 독립적으로 설정한다.
// 동작 (종목별):
//  1. buy_trigger_price > 0 && 미포지션 && 현재가 ≤ buy_trigger_price → 매수
//  2. 포지션 보유 중, 다음 중 먼저 충족되는 조건에서 매도:
//     a) stop_loss_pct > 0 && 손실률 ≥ stop_loss_pct (최우선 — 손절)
//     b) sell_trigger_price > 0 && 현재가 ≥ sell_trigger_price (지정가 익절)
//     c) take_profit_pct > 0 && 수익률 ≥ take_profit_pct (비율 익절)
// ────────────────────────────────────────────────────────────────────

fn pc_default_qty() -> u64 {
    1
}
fn pc_default_tp() -> f64 {
    5.0
}
fn pc_default_sl() -> f64 {
    3.0
}

/// 가격 조건 매매 — 종목별 개별 설정 단위
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceConditionSymbolConfig {
    /// 종목코드
    pub symbol: String,
    /// 종목명 (UI 표시용)
    #[serde(default)]
    pub symbol_name: String,
    /// 1회 주문 수량 (종목별 독립)
    #[serde(default = "pc_default_qty")]
    pub quantity: u64,
    /// 매수 트리거가.
    /// - 국내(is_overseas=false): 원화 정수 (e.g. 55000)
    /// - 해외(is_overseas=true) : USD face value (e.g. 620.5)
    ///   on_tick에서 ×100(cents)으로 변환 후 비교
    /// 0이면 비활성.
    #[serde(default)]
    pub buy_trigger_price: f64,
    /// 지정 익절가. 단위는 buy_trigger_price와 동일. 0이면 비활성.
    #[serde(default)]
    pub sell_trigger_price: f64,
    /// % 익절 기준. 0이면 비활성.
    #[serde(default = "pc_default_tp")]
    pub take_profit_pct: f64,
    /// % 손절 기준. 0이면 비활성.
    #[serde(default = "pc_default_sl")]
    pub stop_loss_pct: f64,
    /// 해외 주식 여부. true이면 가격 단위 = USD (on_tick에서 ×100 변환)
    #[serde(default)]
    pub is_overseas: bool,
}

/// 가격 조건 매매 전략 파라미터 (종목 목록)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceConditionParams {
    #[serde(default)]
    pub symbols: Vec<PriceConditionSymbolConfig>,
}

pub struct PriceConditionStrategy {
    config: StrategyConfig,
    params: PriceConditionParams,
    /// symbol → (in_position, entry_price)
    positions: std::collections::HashMap<String, (bool, Option<u64>)>,
    /// params 변경 감지를 위한 마지막 파싱 기준 JSON
    last_params: serde_json::Value,
}

impl PriceConditionStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: PriceConditionParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        let last_params = config.params.clone();
        Self {
            config,
            params,
            positions: std::collections::HashMap::new(),
            last_params,
        }
    }

    /// config.params가 변경됐을 때 params 재파싱 + target_symbols 동기화
    fn sync_params(&mut self) {
        if self.config.params != self.last_params {
            self.params = serde_json::from_value(self.config.params.clone()).unwrap_or_default();
            self.last_params = self.config.params.clone();
            // target_symbols를 params.symbols 기반으로 자동 갱신 (engine 구독 목록 일치)
            self.config.target_symbols = self
                .params
                .symbols
                .iter()
                .map(|s| s.symbol.clone())
                .collect();
        }
    }
}

impl Strategy for PriceConditionStrategy {
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
        self.sync_params();

        let sym_cfg = match self.params.symbols.iter().find(|s| s.symbol == symbol) {
            Some(s) => s.clone(),
            None => return Signal::Hold,
        };

        // 해외 종목: on_tick price = USD×100(cents). 저장된 트리거가도 ×100으로 스케일 맞춤
        // 국내 종목: on_tick price = KRW 정수. 저장값 그대로 사용
        let scale: f64 = if sym_cfg.is_overseas { 100.0 } else { 1.0 };
        let unit: &str = if sym_cfg.is_overseas { "USD" } else { "원" };
        let buy_thresh = (sym_cfg.buy_trigger_price * scale).round() as u64;
        let sell_thresh = (sym_cfg.sell_trigger_price * scale).round() as u64;

        // 표시용 가격 변환 (cents → USD, 또는 KRW 그대로)
        let to_disp = |p: u64| -> f64 { p as f64 / scale };

        let pos = self
            .positions
            .entry(symbol.to_string())
            .or_insert((false, None));

        if pos.0 {
            let ep = match pos.1 {
                Some(v) => v,
                None => return Signal::Hold,
            };

            // 1) 손절 최우선
            if sym_cfg.stop_loss_pct > 0.0 && price < ep {
                let loss_pct = (ep as f64 - price as f64) / ep as f64 * 100.0;
                if loss_pct >= sym_cfg.stop_loss_pct {
                    pos.0 = false;
                    pos.1 = None;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: sym_cfg.quantity,
                        reason: format!(
                            "가격조건 손절: -{:.1}% ({:.2}{unit} → {:.2}{unit})",
                            loss_pct,
                            to_disp(ep),
                            to_disp(price)
                        ),
                    };
                }
            }

            // 2) 지정가 익절
            if sell_thresh > 0 && price >= sell_thresh {
                pos.0 = false;
                pos.1 = None;
                return Signal::Sell {
                    symbol: symbol.to_string(),
                    quantity: sym_cfg.quantity,
                    reason: format!(
                        "지정가 익절: {:.2}{unit} ≥ 목표 {:.2}{unit}",
                        to_disp(price),
                        sym_cfg.sell_trigger_price
                    ),
                };
            }

            // 3) % 익절
            if sym_cfg.take_profit_pct > 0.0 && price > ep {
                let profit_pct = (price as f64 - ep as f64) / ep as f64 * 100.0;
                if profit_pct >= sym_cfg.take_profit_pct {
                    pos.0 = false;
                    pos.1 = None;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: sym_cfg.quantity,
                        reason: format!(
                            "비율 익절: +{:.1}% ({:.2}{unit} → {:.2}{unit})",
                            profit_pct,
                            to_disp(ep),
                            to_disp(price)
                        ),
                    };
                }
            }

            return Signal::Hold;
        }

        // 미포지션: 매수 조건
        if buy_thresh > 0 && price <= buy_thresh {
            pos.0 = true;
            pos.1 = Some(price);
            return Signal::Buy {
                symbol: symbol.to_string(),
                quantity: sym_cfg.quantity,
                reason: format!(
                    "가격조건 매수: {:.2}{unit} ≤ 트리거 {:.2}{unit}",
                    to_disp(price),
                    sym_cfg.buy_trigger_price
                ),
            };
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        self.sync_params();
        if quantity == 0 || !self.params.symbols.iter().any(|s| s.symbol == symbol) {
            return;
        }
        self.positions
            .insert(symbol.to_string(), (true, Some(avg_price)));
        tracing::info!(
            "가격 조건 전략 포지션 동기화: {} {}주 @ {}",
            symbol,
            quantity,
            avg_price
        );
    }

    fn reset(&mut self) {
        self.positions.clear();
    }
}
