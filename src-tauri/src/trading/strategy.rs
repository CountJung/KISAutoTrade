use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 매매 신호
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Signal {
    /// 매수 신호
    Buy { symbol: String, quantity: u64, reason: String },
    /// 매도 신호
    Sell { symbol: String, quantity: u64, reason: String },
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
    /// 전략 시작 시 일봉 변동 범위(고가-저가) 배열로 초기화. 변동성 확장 전략에서 사용.
    fn initialize_range_data(&mut self, _symbol: &str, _ranges: &[u64]) {}
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
        Self { short_period: 5, long_period: 20 }
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
        let params: MaCrossParams = serde_json::from_value(config.params.clone())
            .unwrap_or_default();
        Self { config, params, states: std::collections::HashMap::new() }
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
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let cap = self.params.long_period + 1;
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| MaCrossState {
            prices: VecDeque::with_capacity(cap),
            prev_short_ma: None,
            prev_long_ma: None,
        });

        state.prices.push_back(price);
        if state.prices.len() > cap {
            state.prices.pop_front();
        }

        let short_ma = Self::moving_average(&state.prices, self.params.short_period);
        let long_ma  = Self::moving_average(&state.prices, self.params.long_period);

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
        state.prev_long_ma  = long_ma;
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
        Self { strategies: Vec::new() }
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

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 변동 범위(고가-저가) 데이터 전달 (변동성 확장 전략)
    pub fn initialize_range_data(&mut self, symbol: &str, ranges: &[u64]) {
        for s in &mut self.strategies {
            if s.config().target_symbols.contains(&symbol.to_string()) {
                s.initialize_range_data(symbol, ranges);
            }
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
        Self { period: 14, oversold: 30.0, overbought: 70.0 }
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
        Self { config, params, states: std::collections::HashMap::new() }
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
            if diff > 0.0 { gain_sum += diff; } else { loss_sum += -diff; }
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
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let cap = self.params.period + 2;
        let (prices, prev_rsi) = self.states.entry(symbol.to_string())
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
        Self { lookback_period: 20, threshold_pct: 5.0 }
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
        let params: MomentumParams = serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self { config, params, states: std::collections::HashMap::new() }
    }
}

impl Strategy for MomentumStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let cap = self.params.lookback_period + 1;
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| MomentumState {
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
        if past_price == 0 { return Signal::Hold; }

        let momentum_pct = (price as f64 - past_price as f64) / past_price as f64 * 100.0;
        let threshold = self.params.threshold_pct;

        if momentum_pct >= threshold {
            if let Some(last) = state.last_buy_price {
                let from_last = (price as f64 - last as f64) / last as f64 * 100.0;
                if from_last > -threshold { return Signal::Hold; }
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
                if from_last < threshold { return Signal::Hold; }
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
        Self { ma_period: 20, buy_threshold_pct: -5.0, sell_threshold_pct: 5.0 }
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
        let params: DeviationParams = serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self { config, params, states: std::collections::HashMap::new() }
    }
}

impl Strategy for DeviationStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let cap = self.params.ma_period;
        let prices = self.states.entry(symbol.to_string())
            .or_insert_with(|| VecDeque::with_capacity(cap));

        prices.push_back(price);
        if prices.len() > cap {
            prices.pop_front();
        }

        if prices.len() < cap {
            return Signal::Hold;
        }

        let ma: f64 = prices.iter().sum::<u64>() as f64 / prices.len() as f64;
        if ma == 0.0 { return Signal::Hold; }

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
        Self { lookback_days: 252, stop_loss_pct: 3.0 }
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
        Self { config, params, states: std::collections::HashMap::new() }
    }
}

impl Strategy for FiftyTwoWeekHighStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.target_symbols.contains(&symbol.to_string()) { return; }
        let lookback = self.params.lookback_days.min(prices.len());
        if lookback < 2 {
            tracing::warn!("52주 신고가 [{}]: 일봉 데이터 부족 ({}봉) — 전략 비활성", symbol, prices.len());
            return;
        }
        let slice = &prices[prices.len().saturating_sub(lookback)..prices.len() - 1];
        if let Some(&h) = slice.iter().max() {
            if h > 0 {
                tracing::info!("52주 신고가 초기화 [{}]: {}원 (최근 {}거래일)", symbol, h, slice.len());
                let state = self.states.entry(symbol.to_string()).or_insert(FiftyTwoWeekState {
                    prev_price: None, high_52w: None, buy_price: None,
                });
                state.high_52w = Some(h);
            }
        }
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let state = self.states.entry(symbol.to_string()).or_insert(FiftyTwoWeekState {
            prev_price: None, high_52w: None, buy_price: None,
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
                let crossed = state.prev_price.map_or(false, |prev| prev <= high && price > high);
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
        Self { buy_days: 3, sell_days: 3 }
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
        Self { config, params, states: std::collections::HashMap::new() }
    }

    fn is_consecutive_up(prices: &VecDeque<u64>, n: usize) -> bool {
        if prices.len() < n + 1 { return false; }
        let slice: Vec<u64> = prices.iter().rev().take(n + 1).cloned().collect();
        (0..n).all(|i| slice[i] > slice[i + 1])
    }

    fn is_consecutive_down(prices: &VecDeque<u64>, n: usize) -> bool {
        if prices.len() < n + 1 { return false; }
        let slice: Vec<u64> = prices.iter().rev().take(n + 1).cloned().collect();
        (0..n).all(|i| slice[i] < slice[i + 1])
    }
}

impl Strategy for ConsecutiveMoveStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let cap = self.params.buy_days.max(self.params.sell_days) + 1;
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| ConsecutiveMoveState {
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
        Self { lookback_days: 20, buffer_pct: 0.5 }
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
        Self { config, params, states: std::collections::HashMap::new() }
    }
}

impl Strategy for FailedBreakoutStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let state = self.states.entry(symbol.to_string()).or_insert_with(|| FailedBreakoutState {
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
                        reason: format!(
                            "돌파 실패: 현재가 {} < 전고점 {} → 매도",
                            price, ref_high
                        ),
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
        Self { threshold_pct: 3.0, stop_loss_pct: 3.0 }
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
        Self { config, params, states: std::collections::HashMap::new() }
    }
}

impl Strategy for StrongCloseStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn initialize_candles(&mut self, symbol: &str, candles: &[(u64, u64)]) {
        if !self.config.target_symbols.contains(&symbol.to_string()) { return; }
        if let Some(&(high, close)) = candles.last() {
            if high == 0 { return; }
            let gap_pct = (high as f64 - close as f64) / high as f64 * 100.0;
            if gap_pct <= self.params.threshold_pct {
                let state = self.states.entry(symbol.to_string()).or_insert(StrongCloseState {
                    pending_buy: false, in_position: false, entry_price: None,
                });
                state.pending_buy = true;
                tracing::info!(
                    "강한 종가 감지 ({}): 고가={}, 종가={}, 이격={:.2}% → 다음 틱 매수 대기",
                    symbol, high, close, gap_pct
                );
            }
        }
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let state = self.states.entry(symbol.to_string()).or_insert(StrongCloseState {
            pending_buy: false, in_position: false, entry_price: None,
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
        Self { lookback_days: 10, expansion_factor: 2.0, stop_loss_pct: 3.0 }
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
        Self { config, params, states: std::collections::HashMap::new() }
    }
}

impl Strategy for VolatilityExpansionStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn initialize_range_data(&mut self, symbol: &str, ranges: &[u64]) {
        if !self.config.target_symbols.contains(&symbol.to_string()) { return; }
        let lookback = self.params.lookback_days.min(ranges.len());
        if lookback == 0 {
            tracing::warn!("변동성 확장 [{}]: 일봉 데이터 없음 — avg_range 미초기화", symbol);
            return;
        }
        let slice = &ranges[ranges.len().saturating_sub(lookback)..];
        let avg = slice.iter().sum::<u64>() as f64 / slice.len() as f64;
        tracing::info!(
            "변동성 확장 초기화 [{}]: 평균 변동폭 {:.0}원 (최근 {}거래일)",
            symbol, avg, slice.len()
        );
        let state = self.states.entry(symbol.to_string()).or_insert(VolatilityExpansionState {
            avg_range: None, day_open: None, day_high: 0, day_low: u64::MAX,
            in_position: false, entry_price: None,
        });
        state.avg_range = Some(avg);
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let state = self.states.entry(symbol.to_string()).or_insert(VolatilityExpansionState {
            avg_range: None, day_open: None, day_high: 0, day_low: u64::MAX,
            in_position: false, entry_price: None,
        });

        if price > state.day_high { state.day_high = price; }
        if price < state.day_low  { state.day_low  = price; }
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
            if state.day_low == u64::MAX { return Signal::Hold; }
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
        Self { period: 20, std_dev: 2.0, stop_loss_pct: 5.0 }
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
        Self { config, params, states: std::collections::HashMap::new() }
    }

    fn bollinger_bands(prices: &VecDeque<u64>, period: usize, std_dev_mult: f64) -> Option<(f64, f64, f64)> {
        if prices.len() < period { return None; }
        let slice: Vec<f64> = prices.iter().rev().take(period).map(|&p| p as f64).collect();
        let mean = slice.iter().sum::<f64>() / period as f64;
        let variance = slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let std = variance.sqrt();
        let upper = mean + std_dev_mult * std;
        let lower = mean - std_dev_mult * std;
        Some((mean, upper, lower))
    }
}

impl Strategy for MeanReversionStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.target_symbols.contains(&symbol.to_string()) { return; }
        let n = self.params.period as usize;
        let take = prices.len().min(n);
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| MeanReversionState {
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
            symbol, state.prices.len(), n
        );
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let period = self.params.period as usize;
        let std_dev = self.params.std_dev;
        let stop_loss = self.params.stop_loss_pct;
        let qty = self.config.order_quantity;

        let state = self.states.entry(symbol.to_string()).or_insert_with(|| MeanReversionState {
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
        Self { long_period: 200, short_period: 5, mid_period: 20 }
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
        Self { config, params, states: std::collections::HashMap::new() }
    }

    fn moving_avg(prices: &VecDeque<u64>, period: usize) -> Option<f64> {
        if prices.len() < period { return None; }
        let sum: u64 = prices.iter().rev().take(period).sum();
        Some(sum as f64 / period as f64)
    }
}

impl Strategy for TrendFilterStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.target_symbols.contains(&symbol.to_string()) { return; }
        let n = self.params.long_period as usize;
        let take = prices.len().min(n);
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| TrendFilterState {
            prices: VecDeque::with_capacity(n + 1),
            in_position: false,
        });
        state.prices.clear();
        for &p in prices[prices.len().saturating_sub(take)..].iter() {
            state.prices.push_back(p);
        }
        tracing::info!(
            "추세 필터 초기화 [{}]: 과거 {}개 가격 로드 (long_period={})",
            symbol, state.prices.len(), n
        );
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let max_cap = (self.params.long_period as usize) + 1;
        let long_p  = self.params.long_period as usize;
        let mid_p   = self.params.mid_period as usize;
        let short_p = self.params.short_period as usize;
        let qty     = self.config.order_quantity;

        let state = self.states.entry(symbol.to_string()).or_insert_with(|| TrendFilterState {
            prices: VecDeque::with_capacity(max_cap),
            in_position: false,
        });

        state.prices.push_back(price);
        while state.prices.len() > max_cap {
            state.prices.pop_front();
        }

        let long_ma  = Self::moving_avg(&state.prices, long_p);
        let mid_ma   = Self::moving_avg(&state.prices, mid_p);
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

fn pc_default_qty() -> u64 { 1 }
fn pc_default_tp()  -> f64 { 5.0 }
fn pc_default_sl()  -> f64 { 3.0 }

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
        Self { config, params, positions: std::collections::HashMap::new(), last_params }
    }

    /// config.params가 변경됐을 때 params 재파싱 + target_symbols 동기화
    fn sync_params(&mut self) {
        if self.config.params != self.last_params {
            self.params = serde_json::from_value(self.config.params.clone())
                .unwrap_or_default();
            self.last_params = self.config.params.clone();
            // target_symbols를 params.symbols 기반으로 자동 갱신 (engine 구독 목록 일치)
            self.config.target_symbols =
                self.params.symbols.iter().map(|s| s.symbol.clone()).collect();
        }
    }
}

impl Strategy for PriceConditionStrategy {
    fn id(&self)             -> &str            { &self.config.id }
    fn name(&self)           -> &str            { &self.config.name }
    fn config(&self)         -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self)     -> bool            { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool)    { self.config.enabled = enabled; }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        self.sync_params();

        let sym_cfg = match self.params.symbols.iter().find(|s| s.symbol == symbol) {
            Some(s) => s.clone(),
            None => return Signal::Hold,
        };

        // 해외 종목: on_tick price = USD×100(cents). 저장된 트리거가도 ×100으로 스케일 맞춤
        // 국내 종목: on_tick price = KRW 정수. 저장값 그대로 사용
        let scale: f64   = if sym_cfg.is_overseas { 100.0 } else { 1.0 };
        let unit: &str   = if sym_cfg.is_overseas { "USD" } else { "원" };
        let buy_thresh   = (sym_cfg.buy_trigger_price  * scale).round() as u64;
        let sell_thresh  = (sym_cfg.sell_trigger_price * scale).round() as u64;

        // 표시용 가격 변환 (cents → USD, 또는 KRW 그대로)
        let to_disp = |p: u64| -> f64 { p as f64 / scale };

        let pos = self.positions.entry(symbol.to_string()).or_insert((false, None));

        if pos.0 {
            let ep = match pos.1 { Some(v) => v, None => return Signal::Hold };

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
                            loss_pct, to_disp(ep), to_disp(price)
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
                        to_disp(price), sym_cfg.sell_trigger_price
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
                            profit_pct, to_disp(ep), to_disp(price)
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
                    to_disp(price), sym_cfg.buy_trigger_price
                ),
            };
        }

        Signal::Hold
    }

    fn reset(&mut self) {
        self.positions.clear();
    }
}

