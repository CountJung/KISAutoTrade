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

pub struct MovingAverageCrossStrategy {
    config: StrategyConfig,
    params: MaCrossParams,
    prices: VecDeque<u64>,
    prev_short_ma: Option<f64>,
    prev_long_ma: Option<f64>,
}

impl MovingAverageCrossStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: MaCrossParams = serde_json::from_value(config.params.clone())
            .unwrap_or_default();
        let cap = params.long_period + 1;
        Self {
            config,
            params,
            prices: VecDeque::with_capacity(cap),
            prev_short_ma: None,
            prev_long_ma: None,
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
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.target_symbols.contains(&symbol.to_string()) {
            return Signal::Hold;
        }

        self.prices.push_back(price);
        if self.prices.len() > self.params.long_period + 1 {
            self.prices.pop_front();
        }

        let short_ma = Self::moving_average(&self.prices, self.params.short_period);
        let long_ma = Self::moving_average(&self.prices, self.params.long_period);

        let signal = match (self.prev_short_ma, self.prev_long_ma, short_ma, long_ma) {
            (Some(ps), Some(pl), Some(cs), Some(cl)) => {
                if ps <= pl && cs > cl {
                    // 골든크로스 → 매수
                    Signal::Buy {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!("골든크로스 S{:.0} > L{:.0}", cs, cl),
                    }
                } else if ps >= pl && cs < cl {
                    // 데드크로스 → 매도
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

        self.prev_short_ma = short_ma;
        self.prev_long_ma = long_ma;

        signal
    }

    fn reset(&mut self) {
        self.prices.clear();
        self.prev_short_ma = None;
        self.prev_long_ma = None;
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
    /// 최근 price 수열 (period+1개만 유지)
    prices: VecDeque<u64>,
    prev_rsi: Option<f64>,
}

impl RsiStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: RsiParams = serde_json::from_value(config.params.clone()).unwrap_or_default();
        let cap = params.period + 2;
        Self { config, params, prices: VecDeque::with_capacity(cap), prev_rsi: None }
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

        self.prices.push_back(price);
        if self.prices.len() > self.params.period + 2 {
            self.prices.pop_front();
        }

        let rsi = Self::calc_rsi(&self.prices, self.params.period);

        let signal = match (self.prev_rsi, rsi) {
            (Some(prev), Some(cur)) => {
                if prev <= self.params.oversold && cur > self.params.oversold {
                    // 과매도 → 반등 확인 → 매수
                    Signal::Buy {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!("RSI 과매도 반등 {:.1}", cur),
                    }
                } else if prev >= self.params.overbought && cur < self.params.overbought {
                    // 과매수 → 하락 확인 → 매도
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

        self.prev_rsi = rsi;
        signal
    }

    fn reset(&mut self) {
        self.prices.clear();
        self.prev_rsi = None;
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

pub struct MomentumStrategy {
    config: StrategyConfig,
    params: MomentumParams,
    prices: VecDeque<u64>,
    /// 연속 같은 방향 신호 방지용
    last_buy_price: Option<u64>,
    last_sell_price: Option<u64>,
}

impl MomentumStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: MomentumParams = serde_json::from_value(config.params.clone()).unwrap_or_default();
        let cap = params.lookback_period + 1;
        Self {
            config,
            params,
            prices: VecDeque::with_capacity(cap),
            last_buy_price: None,
            last_sell_price: None,
        }
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

        self.prices.push_back(price);
        if self.prices.len() > self.params.lookback_period + 1 {
            self.prices.pop_front();
        }

        if self.prices.len() < self.params.lookback_period + 1 {
            return Signal::Hold;
        }

        let past_price = *self.prices.front().unwrap();
        if past_price == 0 { return Signal::Hold; }

        let momentum_pct = (price as f64 - past_price as f64) / past_price as f64 * 100.0;
        let threshold = self.params.threshold_pct;

        if momentum_pct >= threshold {
            // 이미 매수한 적 있으면 재진입 방지 (가격이 충분히 내려온 후에만 재매수)
            if let Some(last) = self.last_buy_price {
                let from_last = (price as f64 - last as f64) / last as f64 * 100.0;
                if from_last > -threshold { return Signal::Hold; }
            }
            self.last_buy_price = Some(price);
            Signal::Buy {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("상승 모멘텀 +{:.1}%", momentum_pct),
            }
        } else if momentum_pct <= -threshold {
            if let Some(last) = self.last_sell_price {
                let from_last = (price as f64 - last as f64) / last as f64 * 100.0;
                if from_last < threshold { return Signal::Hold; }
            }
            self.last_sell_price = Some(price);
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
        self.prices.clear();
        self.last_buy_price = None;
        self.last_sell_price = None;
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
    prices: VecDeque<u64>,
}

impl DeviationStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: DeviationParams = serde_json::from_value(config.params.clone()).unwrap_or_default();
        let cap = params.ma_period;
        Self { config, params, prices: VecDeque::with_capacity(cap) }
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

        self.prices.push_back(price);
        if self.prices.len() > self.params.ma_period {
            self.prices.pop_front();
        }

        if self.prices.len() < self.params.ma_period {
            return Signal::Hold;
        }

        let ma: f64 = self.prices.iter().sum::<u64>() as f64 / self.prices.len() as f64;
        if ma == 0.0 { return Signal::Hold; }

        // 이격도 = (현재가 / MA - 1) * 100
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
        self.prices.clear();
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

pub struct FiftyTwoWeekHighStrategy {
    config: StrategyConfig,
    params: FiftyTwoWeekHighParams,
    /// 직전 틱 가격 (돌파 감지용 — prev ≤ high && cur > high)
    prev_price: Option<u64>,
    /// 252 거래일 최고가 (일봉 데이터로 초기화, 이후 실시간 갱신)
    high_52w: Option<u64>,
    /// 매수 기준가 (손절 계산용)
    buy_price: Option<u64>,
}

impl FiftyTwoWeekHighStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: FiftyTwoWeekHighParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self { config, params, prev_price: None, high_52w: None, buy_price: None }
    }
}

impl Strategy for FiftyTwoWeekHighStrategy {
    fn id(&self) -> &str { &self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn config(&self) -> &StrategyConfig { &self.config }
    fn config_mut(&mut self) -> &mut StrategyConfig { &mut self.config }
    fn is_enabled(&self) -> bool { self.config.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.config.enabled = enabled; }

    /// prices: start_trading 시 KIS 차트 API의 일봉 고가(high) 배열 (오름차순)
    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        let lookback = self.params.lookback_days.min(prices.len());
        if lookback < 2 {
            tracing::warn!("52주 신고가 [{}]: 일봉 데이터 부족 ({}봉) — 전략 비활성", symbol, prices.len());
            return;
        }
        // 마지막 1봉(오늘)은 제외 — 오늘 갱신된 고가는 미확정
        let slice = &prices[prices.len().saturating_sub(lookback)..prices.len() - 1];
        if let Some(&h) = slice.iter().max() {
            if h > 0 {
                tracing::info!("52주 신고가 초기화 [{}]: {}원 (최근 {}거래일)", symbol, h, slice.len());
                self.high_52w = Some(h);
            }
        }
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled { return Signal::Hold; }
        if !self.config.target_symbols.contains(&symbol.to_string()) { return Signal::Hold; }

        let signal = match self.high_52w {
            None => {
                // 히스토리 초기화 전 — 데이터 수신만 하고 관망
                Signal::Hold
            }
            Some(high) => {
                // ① 손절 체크 (매수 포지션이 있을 때 우선 확인)
                if let Some(bp) = self.buy_price {
                    let stop_price = (bp as f64 * (1.0 - self.params.stop_loss_pct / 100.0)) as u64;
                    if price <= stop_price {
                        self.buy_price = None;
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

                // ② 52주 신고가 돌파 감지 (이전 틱이 고가 이하, 현재 틱이 고가 초과)
                let crossed = self.prev_price.map_or(false, |prev| prev <= high && price > high);
                if crossed && self.buy_price.is_none() {
                    // 신고가 갱신 → 매수 신호
                    self.high_52w = Some(price); // 새 고가로 업데이트
                    self.buy_price = Some(price);
                    Signal::Buy {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!(
                            "52주 신고가 돌파: {:.0}원 (이전 고가 {:.0}원)",
                            price as f64, high as f64
                        ),
                    }
                } else {
                    // 고가 위에서 추가 상승 중이면 고가 업데이트 (다음 돌파 기준선)
                    if price > high {
                        self.high_52w = Some(price);
                    }
                    Signal::Hold
                }
            }
        };

        self.prev_price = Some(price);
        signal
    }

    fn reset(&mut self) {
        self.prev_price = None;
        self.high_52w = None;
        self.buy_price = None;
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

pub struct ConsecutiveMoveStrategy {
    config: StrategyConfig,
    params: ConsecutiveMoveParams,
    /// 최근 가격 이력 — buy_days/sell_days 의 최대값+1 개만 유지
    prices: VecDeque<u64>,
    /// 연속 매수 후 재진입 방지 플래그
    in_position: bool,
}

impl ConsecutiveMoveStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: ConsecutiveMoveParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        let cap = params.buy_days.max(params.sell_days) + 1;
        Self { config, params, prices: VecDeque::with_capacity(cap), in_position: false }
    }

    /// 최근 n+1개 가격에서 마지막 n개 구간이 모두 연속 상승인지 확인
    fn is_consecutive_up(prices: &VecDeque<u64>, n: usize) -> bool {
        if prices.len() < n + 1 { return false; }
        let slice: Vec<u64> = prices.iter().rev().take(n + 1).cloned().collect();
        // slice[0] = 최신, slice[n] = 가장 오래된
        (0..n).all(|i| slice[i] > slice[i + 1])
    }

    /// 최근 n+1개 가격에서 마지막 n개 구간이 모두 연속 하락인지 확인
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
        self.prices.push_back(price);
        if self.prices.len() > cap {
            self.prices.pop_front();
        }

        // ① 매도 우선 확인 (포지션 있을 때 연속 하락이면 손절)
        if self.in_position && Self::is_consecutive_down(&self.prices, self.params.sell_days) {
            self.in_position = false;
            return Signal::Sell {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("{}일 연속 하락 → 매도", self.params.sell_days),
            };
        }

        // ② 매수: 포지션 없을 때 연속 상승
        if !self.in_position && Self::is_consecutive_up(&self.prices, self.params.buy_days) {
            self.in_position = true;
            return Signal::Buy {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("{}일 연속 상승 → 매수", self.params.buy_days),
            };
        }

        Signal::Hold
    }

    fn reset(&mut self) {
        self.prices.clear();
        self.in_position = false;
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

pub struct FailedBreakoutStrategy {
    config: StrategyConfig,
    params: FailedBreakoutParams,
    /// 최근 lookback_days개의 가격 히스토리 (매수 판단 시점의 전고점 계산용)
    prices: VecDeque<u64>,
    /// 포지션 진입 여부
    in_position: bool,
    /// 돌파 매수 시점의 전고점 (돌파 실패 매도 기준)
    breakout_prev_high: Option<u64>,
}

impl FailedBreakoutStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: FailedBreakoutParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            prices: VecDeque::new(),
            in_position: false,
            breakout_prev_high: None,
        }
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

        // 현재 틱 수신 전 기존 히스토리에서 전고점 계산
        let prev_high = self.prices.iter().copied().max().unwrap_or(0);

        // ① 매도 우선: 포지션 진입 후 가격이 돌파 시점 전고점 이하로 내려오면 실패 매도
        if self.in_position {
            if let Some(ref_high) = self.breakout_prev_high {
                if price < ref_high {
                    self.in_position = false;
                    self.breakout_prev_high = None;
                    self.prices.push_back(price);
                    if self.prices.len() > self.params.lookback_days {
                        self.prices.pop_front();
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

        // ② 매수: lookback_days 이상 데이터가 쌓인 상태에서 전고점 돌파 확인
        if !self.in_position && self.prices.len() >= self.params.lookback_days && prev_high > 0 {
            let breakout_threshold =
                (prev_high as f64 * (1.0 + self.params.buffer_pct / 100.0)) as u64;
            if price >= breakout_threshold {
                self.in_position = true;
                self.breakout_prev_high = Some(prev_high);
                self.prices.push_back(price);
                if self.prices.len() > self.params.lookback_days {
                    self.prices.pop_front();
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

        // 가격 히스토리 업데이트
        self.prices.push_back(price);
        if self.prices.len() > self.params.lookback_days {
            self.prices.pop_front();
        }

        Signal::Hold
    }

    fn reset(&mut self) {
        self.prices.clear();
        self.in_position = false;
        self.breakout_prev_high = None;
    }
}
