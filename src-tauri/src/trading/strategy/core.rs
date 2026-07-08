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
    /// 전략 시작 시 장중 가격 배열로 초기화. 실시간 틱 기반 반동/매수세 판단이 필요한 전략에서 재정의.
    fn initialize_intraday_prices(&mut self, _symbol: &str, _prices: &[u64]) {}
    /// 전략 시작 시 장중 OHLC 배열로 초기화. 미리보기와 실시간 전략의 1분봉 판단을 맞춰야 하는 전략에서 재정의.
    fn initialize_intraday_ohlc(&mut self, _symbol: &str, _candles: &[OhlcCandle]) {}
    /// 전략 시작 시 일봉 변동 범위(고가-저가) 배열로 초기화. 변동성 확장 전략에서 사용.
    fn initialize_range_data(&mut self, _symbol: &str, _ranges: &[u64]) {}
    /// 자동매매 시작 시 실제 잔고 기반으로 전략 내부 포지션 플래그를 동기화한다.
    fn sync_position(&mut self, _symbol: &str, _quantity: u64, _avg_price: u64) {}
    /// broker/account scope가 있는 실제 잔고 기반 포지션 동기화 훅.
    fn sync_position_for_broker(&mut self, snapshot: &BrokerPositionSnapshot) {
        self.sync_position(&snapshot.symbol, snapshot.quantity, snapshot.avg_price);
    }
    /// 전략 상태 초기화 (일 초기화 등)
    fn reset(&mut self);
}
