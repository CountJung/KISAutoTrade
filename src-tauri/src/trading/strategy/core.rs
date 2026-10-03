use serde::{Deserialize, Serialize};

use crate::broker::{BrokerId, BrokerMarket};

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

/// 어떤 전략이 발생시킨 신호인지 함께 보존하는 자동매매 신호
#[derive(Debug, Clone)]
pub struct StrategySignal {
    pub strategy_id: String,
    pub signal: Signal,
}

/// Broker별 잔고에서 복원한 전략 포지션 스냅샷.
///
/// 가격 단위는 주문 경로와 동일하게 국내/KRW는 원, 해외/USD는 cents를 사용한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerPositionSnapshot {
    pub broker_id: BrokerId,
    pub market: BrokerMarket,
    pub symbol: String,
    pub quantity: u64,
    pub avg_price: u64,
}

/// 전략 설정 (JSON 직렬화 가능)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    #[serde(default = "default_strategy_broker_id")]
    pub broker_id: BrokerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broker_account_id: Option<String>,
    pub target_symbols: Vec<String>,
    /// 1회 주문 수량
    pub order_quantity: u64,
    // 전략별 파라미터
    pub params: serde_json::Value,
}

fn default_strategy_broker_id() -> BrokerId {
    BrokerId::Kis
}

impl StrategyConfig {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        enabled: bool,
        target_symbols: Vec<String>,
        order_quantity: u64,
        params: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            enabled,
            broker_id: BrokerId::Kis,
            broker_account_id: None,
            target_symbols,
            order_quantity,
            params,
        }
    }

    pub fn with_scope(mut self, broker_id: BrokerId, broker_account_id: Option<String>) -> Self {
        self.set_scope(broker_id, broker_account_id);
        self
    }

    pub fn set_scope(&mut self, broker_id: BrokerId, broker_account_id: Option<String>) {
        self.broker_id = broker_id;
        self.broker_account_id = broker_account_id.filter(|v| !v.trim().is_empty());
    }

    pub fn matches_scope(&self, broker_id: BrokerId, broker_account_id: Option<&str>) -> bool {
        self.broker_id == broker_id
            && self.broker_account_id.as_deref() == broker_account_id.filter(|v| !v.is_empty())
    }

    pub fn targets_symbol(&self, symbol: &str) -> bool {
        self.target_symbols.iter().any(|target| target == symbol)
    }
}

/// 전략 초기화용 OHLC 캔들.
#[derive(Debug, Clone, Copy)]
pub struct OhlcCandle {
    pub open: u64,
    pub high: u64,
    pub low: u64,
    pub close: u64,
}

/// 전략 상태에서 읽은 실제 지표 준비 상태. 가격 단위가 아닌 관측 봉 수다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryReadiness {
    pub required_bars: usize,
    pub available_bars: usize,
    pub ready: bool,
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
    /// 일봉 replay의 날짜 시작. 당일 시가만 공개하며 보유 포지션을 지우지 않는다.
    fn on_trading_day_start(&mut self, _symbol: &str, _open: u64) {}
    /// 완료 시점에 공개된 당일 OHLC로 종가 관측을 평가한다.
    fn on_daily_close_tick(&mut self, symbol: &str, candle: &OhlcCandle, volume: u64) -> Signal {
        self.on_tick(symbol, candle.close, volume)
    }
    /// 체결/차단 피드백 뒤 완료된 일봉을 다음 평가일의 과거 상태에 반영한다.
    fn on_completed_candle(&mut self, _symbol: &str, _candle: &OhlcCandle) {}
    /// 전략별 단일 가격 배열 초기화. 기본 OHLC 훅은 종가를 전달한다.
    /// 고가 등 다른 값이 필요한 전략은 initialize_ohlc를 재정의한다.
    fn initialize_historical(&mut self, _symbol: &str, _prices: &[u64]) {}
    /// 전략 시작 시 일봉 (고가, 종가) 쌍 배열로 초기화. 강한 종가 등 복합 일봉 데이터가 필요한 전략에서 재정의.
    fn initialize_candles(&mut self, _symbol: &str, _candles: &[(u64, u64)]) {}
    /// 전략 시작 시 일봉 OHLC 배열로 초기화. 기본은 종가 기반 지표를 초기화한다.
    fn initialize_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        let closes = candles
            .iter()
            .map(|candle| candle.close)
            .collect::<Vec<_>>();
        self.initialize_historical(symbol, &closes);
    }
    /// 전략 시작 시 장중 가격 배열로 초기화. 실시간 틱 기반 반동/매수세 판단이 필요한 전략에서 재정의.
    fn initialize_intraday_prices(&mut self, _symbol: &str, _prices: &[u64]) {}
    /// 전략 시작 시 장중 OHLC 배열로 초기화. 미리보기와 실시간 전략의 1분봉 판단을 맞춰야 하는 전략에서 재정의.
    fn initialize_intraday_ohlc(&mut self, _symbol: &str, _candles: &[OhlcCandle]) {}
    /// 전략 시작 시 일봉 변동 범위(고가-저가) 배열로 초기화. 변동성 확장 전략에서 사용.
    fn initialize_range_data(&mut self, _symbol: &str, _ranges: &[u64]) {}
    /// 실제 지표와 버퍼의 준비 상태. 미지원 전략은 준비 완료로 추측하지 않는다.
    fn history_readiness(&self, _symbol: &str) -> Option<HistoryReadiness> {
        None
    }
    /// 다음 tick에서 매매 조건을 평가할 수 있는지. 교차 전략은 이전 지표가 필요하다.
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        self.history_readiness(symbol).map(|state| state.ready)
    }
    /// 자동매매 시작 시 실제 잔고 기반으로 전략 내부 포지션 플래그를 동기화한다.
    fn sync_position(&mut self, _symbol: &str, _quantity: u64, _avg_price: u64) {}
    /// broker/account scope가 있는 실제 잔고 기반 포지션 동기화 훅.
    fn sync_position_for_broker(&mut self, snapshot: &BrokerPositionSnapshot) {
        self.sync_position(&snapshot.symbol, snapshot.quantity, snapshot.avg_price);
    }
    /// 전략 상태 초기화 (일 초기화 등)
    fn reset(&mut self);
}

/// live 시작과 deterministic preview가 공유하는 전략 warmup 정규화 경계.
///
/// 일봉과 장중 봉을 구분해 전달하므로 1분봉을 52주 고가/일봉 ATR처럼 잘못 해석하거나,
/// replay 이후 미래 봉을 initializer에 넣는 일을 방지한다.
pub fn initialize_strategy_warmup(
    strategy: &mut dyn Strategy,
    symbol: &str,
    daily_ohlc: &[OhlcCandle],
    intraday_ohlc: &[OhlcCandle],
) {
    if !daily_ohlc.is_empty() {
        let high_close = daily_ohlc
            .iter()
            .map(|candle| (candle.high, candle.close))
            .collect::<Vec<_>>();
        let ranges = daily_ohlc
            .iter()
            .map(|candle| candle.high.saturating_sub(candle.low))
            .collect::<Vec<_>>();
        strategy.initialize_candles(symbol, &high_close);
        strategy.initialize_ohlc(symbol, daily_ohlc);
        strategy.initialize_range_data(symbol, &ranges);
    }
    if !intraday_ohlc.is_empty() {
        let closes = intraday_ohlc
            .iter()
            .map(|candle| candle.close)
            .collect::<Vec<_>>();
        strategy.initialize_intraday_prices(symbol, &closes);
        strategy.initialize_intraday_ohlc(symbol, intraday_ohlc);
    }
}
