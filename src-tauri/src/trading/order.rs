/// 주문 관리자 — KISAutoTrade 핵심 실행 모듈
///
/// 역할:
///  ① 전략 신호(Signal::Buy/Sell) → KIS API place_order() 실행
///  ② 미체결 주문 풀(HashMap) 유지 (odno → PendingOrder)
///  ③ 동일/반대 방향 미체결 주문 방지 (pending scan + symbol_to_odno 맵)
///  ④ 체결 이벤트(on_fill) 처리 → 미체결 풀 상태 갱신
///     (WebSocket H0STCNI0 수신 또는 폴링에서 호출)
///  ⑤ 체결 확인 시 PositionTracker.on_buy/on_sell() 연동
///  ⑥ 주문 기록을 OrderStore JSON에 저장
///  ⑦ 매도 체결 손익 → StatsStore & RiskManager 반영
///  ⑧ 주문 전 RiskManager.can_trade() + check_position_size() 검증
///  ⑨ 체결 완료 시 Discord 알림 전송
///  ⑩ KIS rate-limit(EGW00133/EGW00201) 오류 시 2초 대기 × 최대 3회 시도
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::{
    api::rest::{
        KisRestClient, OrderRequest, OrderResponse, OrderSide as RestOrderSide, OrderType,
        OverseasOrderRequest,
    },
    broker::BrokerScope,
    config::ProfilesConfig,
    notifications::{discord::DiscordNotifier, types::NotificationEvent},
    storage::{
        order_store::{OrderRecord, OrderSide, OrderStatus},
        OrderStore, PendingOrderStore, RiskStore, StatsStore, TradeStore,
    },
    trading::{
        guard::{GuardDecision, TradeGuard},
        position::{OverseasPositionTracker, PositionTracker},
        risk::{DailyOrderSide, RiskManager},
        strategy::Signal,
    },
};

mod budget;
mod configuration;
mod feedback;
mod holdings;
mod ownership;
pub(crate) use holdings::{HoldingsMarkets, HoldingsUpdate};
mod conflicts;
pub use budget::{AutoTradingBudgetView, BudgetCurrencyView};
mod execution_run;
mod fills;
pub use execution_run::ExecutionRunContext;
mod health;
mod legacy;
mod pending;
mod submission;

use conflicts::pending_conflict_reason_for_scope;
pub use submission::{OwnedStrategySignal, SubmissionOutcome};

// ────────────────────────────────────────────────────────────────────
// PendingOrder — 미체결 주문 항목
// ────────────────────────────────────────────────────────────────────

/// 미체결(Pending) 상태 주문 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingOrder {
    /// 원본 주문 기록 (OrderStore에 Pending 상태로 기록됨)
    pub record: OrderRecord,
    /// 이 주문을 촉발한 전략 신호 이유
    pub signal_reason: String,
    /// 신호를 발생시킨 전략 ID
    pub strategy_id: Option<String>,
    /// 신호 발생 시점 가격 (국내 KRW, 해외 USD cents)
    pub signal_price: u64,
    /// 주문 제출 가격 (국내 시장가 0, 해외 지정가 USD cents)
    pub order_price: u64,
    /// 해외 주문이면 KIS 주문 거래소 코드(NASD/NYSE/AMEX), 국내 주문이면 None
    pub exchange: Option<String>,
    /// 주문이 발생한 broker/account scope
    pub broker_scope: BrokerScope,
    /// Exact strategy ownership captured before submission. Legacy/manual orders
    /// never acquire a strategy owner by inference from account holdings.
    #[serde(default)]
    pub(crate) ownership_key: Option<ownership::OwnershipKey>,
    #[serde(default)]
    pub(crate) ownership_generation: Option<String>,
    #[serde(default)]
    pub(crate) strategy_position_metadata: Option<serde_json::Value>,
    /// Provider-observed average captured before a manual sell, never a later
    /// empty holdings snapshot interpreted as zero cost.
    #[serde(default)]
    pub(crate) submitted_cost_basis: Option<u64>,
    /// 주문번호 기반 조회로 이미 반영한 누적 체결 수량
    pub filled_quantity: u64,
    /// 이미 반영한 체결금액 합계(가격 내부단위 × 수량)
    #[serde(default)]
    pub filled_notional: u128,
    /// provider가 마지막으로 확인한 누적 체결수량/평균가
    #[serde(default)]
    pub confirmed_filled_quantity: u64,
    #[serde(default)]
    pub confirmed_avg_price: u64,
    /// Provider cumulative native-unit amount fixed by the durable fill intent.
    #[serde(default)]
    pub confirmed_filled_notional: Option<u128>,
    /// 체결 event의 durable side effect 적용을 시작했는지 여부
    #[serde(default)]
    pub application_started: bool,
    #[serde(default)]
    pub(crate) application_execution_date: Option<String>,
    #[serde(default)]
    pub(crate) application_exchange_rate: Option<f64>,
    /// 적용 intent에 고정한 delta 체결 손익(국내 KRW, 해외 cents).
    #[serde(default)]
    pub application_pnl: Option<i64>,
    /// broker에 전달한 client 주문 ID (지원하는 broker만 저장)
    #[serde(default)]
    pub client_order_id: Option<String>,
    /// 마지막으로 확인한 provider 주문 상태
    #[serde(default)]
    pub provider_status: Option<String>,
}

// ────────────────────────────────────────────────────────────────────
// OrderManager
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingHealthStatus {
    pub last_holdings_sync_at: Option<String>,
    pub last_holdings_sync_attempt_at: Option<String>,
    pub last_holdings_sync_error: Option<String>,
    pub last_reconciliation_at: Option<String>,
    pub last_reconciliation_attempt_at: Option<String>,
    pub last_reconciliation_error: Option<String>,
    pub holdings_consecutive_failures: u32,
    pub reconciliation_consecutive_failures: u32,
    pub oldest_pending_at: Option<String>,
    pub pending_order_count: usize,
    pub persistence_blocked: bool,
    pub persistence_error: Option<String>,
    pub daemon_consecutive_failures: u32,
    pub daemon_last_error: Option<String>,
}

pub struct OrderManager {
    // ── ② 미체결 주문 풀 ────────────────────────────────────────────
    /// KIS odno → PendingOrder
    pending: HashMap<String, PendingOrder>,
    budget: Option<budget::BudgetLedger>,

    // ── ③ 중복 방지 인덱스 ─────────────────────────────────────────
    /// symbol → odno (미체결 주문이 있는 종목)
    symbol_to_odno: HashMap<String, String>,
    /// provider 주문 접수 전 local in-flight 예약. 느린 broker 호출 중 중복/반대 주문을 막는다.
    submitting: HashMap<(BrokerScope, String), OrderSide>,
    modifying: std::collections::HashSet<String>,

    // ── ⑪ 잔고 부족 매수 정지 ────────────────────────────────────
    /// true = KIS 잔고부족 응답 수신 후 매수 일시 정지.
    /// 매도 체결 또는 수동 해제 시 false로 전환됨.
    pub buy_suspended: bool,
    /// 매수 정지 사유 (KIS 응답 msg1)
    pub buy_suspended_reason: Option<String>,
    /// 주문 상태 영속화 실패 시 매수/매도 모두 차단한다.
    persistence_blocked: bool,
    health: TradingHealthStatus,

    // ── 공유 의존성 (Arc) ───────────────────────────────────────────
    /// KIS REST 클라이언트 (프로파일 전환 시 내부 Arc만 교체됨)
    rest_client: Arc<RwLock<Arc<KisRestClient>>>,
    profiles: Arc<RwLock<ProfilesConfig>>,
    order_store: Arc<OrderStore>,
    pending_order_store: Arc<PendingOrderStore>,
    trade_store: Arc<TradeStore>,
    position_tracker: Arc<Mutex<PositionTracker>>,
    overseas_position_tracker: Arc<Mutex<OverseasPositionTracker>>,
    stats_store: Arc<StatsStore>,
    exchange_rate_krw: Arc<RwLock<f64>>,
    /// 리스크 관리자 — AppState에서 Arc 공유
    pub risk_manager: Arc<Mutex<RiskManager>>,
    /// 리스크 runtime 상태 저장소 — 주문/체결 반영 직후 스냅샷을 영속화한다
    risk_store: Arc<RiskStore>,
    /// 전략 신호와 주문 실행 사이의 반복매매/휩소 방어 계층
    trade_guard: TradeGuard,
    /// 자동매매 시작 시점의 broker/account scope. 실행 중 프로파일 전환과 분리한다.
    execution_scope: BrokerScope,
    execution_epoch: u64,
    configuration_edit:
        Option<std::sync::Weak<crate::trading::configuration::ConfigurationEditReservation>>,
    configuration_identity: Option<configuration::ConfigurationIdentity>,
    ownership_document_generation: Option<String>,
    holdings_refresh_required: u8,
    active_execution_run: Option<execution_run::ExecutionRun>,
    #[cfg(test)]
    preparation_test_gate: Option<Arc<submission::PreparationTestGate>>,
    discord: Option<Arc<DiscordNotifier>>,
}

impl OrderManager {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rest_client: Arc<RwLock<Arc<KisRestClient>>>,
        profiles: Arc<RwLock<ProfilesConfig>>,
        order_store: Arc<OrderStore>,
        pending_order_store: Arc<PendingOrderStore>,
        trade_store: Arc<TradeStore>,
        position_tracker: Arc<Mutex<PositionTracker>>,
        overseas_position_tracker: Arc<Mutex<OverseasPositionTracker>>,
        stats_store: Arc<StatsStore>,
        exchange_rate_krw: Arc<RwLock<f64>>,
        risk_manager: Arc<Mutex<RiskManager>>,
        risk_store: Arc<RiskStore>,
        discord: Option<Arc<DiscordNotifier>>,
    ) -> Self {
        Self {
            pending: HashMap::new(),
            budget: None,
            symbol_to_odno: HashMap::new(),
            submitting: HashMap::new(),
            modifying: std::collections::HashSet::new(),
            buy_suspended: false,
            buy_suspended_reason: None,
            persistence_blocked: false,
            health: TradingHealthStatus::default(),
            rest_client,
            profiles,
            order_store,
            pending_order_store,
            trade_store,
            position_tracker,
            overseas_position_tracker,
            stats_store,
            exchange_rate_krw,
            risk_manager,
            risk_store,
            trade_guard: TradeGuard::default(),
            execution_scope: BrokerScope::kis_legacy(),
            execution_epoch: 0,
            configuration_edit: None,
            configuration_identity: None,
            ownership_document_generation: None,
            holdings_refresh_required: 0,
            active_execution_run: None,
            #[cfg(test)]
            preparation_test_gate: None,
            discord,
        }
    }

    // ── 공개 API ────────────────────────────────────────────────────

    /// 자동매매 실행 scope를 시작 시점 broker/account로 고정한다.
    pub fn set_execution_scope(&mut self, scope: BrokerScope) {
        self.end_execution_run();
        tracing::info!("주문 실행 scope 설정: {:?}", scope);
        self.execution_scope = scope;
    }

    pub fn execution_scope(&self) -> &BrokerScope {
        &self.execution_scope
    }

    pub fn order_store_handle(&self) -> Arc<OrderStore> {
        Arc::clone(&self.order_store)
    }

    /// ① 전략 신호 처리 → 주문 실행
    ///
    /// - `symbol_name`: 한국어 종목명 (PositionTracker, 알림에 사용)
    /// - `total_balance`: 총 잔고(원) — 0이면 포지션 비중 검사 skip
    /// - `exchange`: None = 국내, Some("NAS"/"NYS"/"AMS") = 해외
    /// - `tick_price`: 현재가 (국내 = 원, 해외 = USD × 100)
    pub async fn submit_signal(
        &mut self,
        strategy_id: Option<String>,
        signal: Signal,
        symbol_name: &str,
        total_balance: i64,
        exchange: Option<String>,
        tick_price: u64,
    ) -> Result<()> {
        if !matches!(signal, Signal::Hold) {
            anyhow::bail!(
                "자동매매 주문은 전용 예산을 검증하는 submit_signal_shared 경로를 사용해야 합니다."
            );
        }
        let broker_scope = self.execution_scope.clone();
        let (held_quantity, avg_price) = match &signal {
            Signal::Buy { symbol, .. } | Signal::Sell { symbol, .. } => {
                if exchange.is_some() {
                    let tracker = self.overseas_position_tracker.lock().await;
                    tracker
                        .get(symbol)
                        .map(|p| (p.quantity, Some(p.avg_price_cents.round() as u64)))
                        .unwrap_or((0, None))
                } else {
                    let tracker = self.position_tracker.lock().await;
                    tracker
                        .get(symbol)
                        .map(|p| (p.quantity, Some(p.avg_price.round() as u64)))
                        .unwrap_or((0, None))
                }
            }
            Signal::Hold => (0, None),
        };

        match self.trade_guard.evaluate_for_scope(
            &broker_scope,
            &signal,
            held_quantity,
            avg_price,
            tick_price,
            exchange.is_some(),
        ) {
            GuardDecision::Allow => {}
            GuardDecision::Block { reason } => {
                tracing::info!("TradeGuard 차단 — {}", reason);
                return Ok(());
            }
        }

        if let Some((symbol, side)) = daily_order_limit_key(&signal) {
            let limit_reason = self
                .risk_manager
                .lock()
                .await
                .daily_order_limit_reason_for_scope(
                    &broker_scope,
                    strategy_id.as_deref().unwrap_or("unknown"),
                    symbol,
                    side,
                );
            if let Some(reason) = limit_reason {
                tracing::info!("리스크 주문 횟수 제한 — {} ({})", symbol, reason);
                return Ok(());
            }
        }

        if let Signal::Buy { symbol, .. } = &signal {
            if let Some(reason) = self
                .risk_manager
                .lock()
                .await
                .consecutive_loss_block_reason_for_scope(
                    &broker_scope,
                    strategy_id.as_deref().unwrap_or("unknown"),
                    symbol,
                )
            {
                tracing::info!("연속 손실 진입 차단 — {} ({})", symbol, reason);
                return Ok(());
            }
        }

        match signal {
            Signal::Buy {
                symbol,
                quantity,
                reason,
            } => {
                self.process_buy(
                    symbol,
                    symbol_name.to_string(),
                    quantity,
                    reason,
                    total_balance,
                    exchange,
                    tick_price,
                    strategy_id,
                    broker_scope,
                )
                .await
            }
            Signal::Sell {
                symbol,
                quantity,
                reason,
            } => {
                self.process_sell(
                    symbol,
                    symbol_name.to_string(),
                    quantity,
                    reason,
                    exchange,
                    tick_price,
                    strategy_id,
                    broker_scope,
                )
                .await
            }
            Signal::Hold => Ok(()),
        }
    }

    pub async fn current_exchange_rate_krw(&self) -> f64 {
        *self.exchange_rate_krw.read().await
    }
}

// ────────────────────────────────────────────────────────────────────
// 잔고 부족 에러 감지 헬퍼
// ────────────────────────────────────────────────────────────────────

fn estimate_order_amount_krw(
    tick_price: u64,
    quantity: u64,
    is_overseas: bool,
    exchange_rate_krw: f64,
) -> i64 {
    if is_overseas {
        ((tick_price as f64 / 100.0) * quantity as f64 * exchange_rate_krw)
            .round()
            .max(0.0) as i64
    } else {
        tick_price.saturating_mul(quantity) as i64
    }
}

fn order_exchange_code(exchange: &str) -> &str {
    match exchange {
        "NAS" => "NASD",
        "NYS" => "NYSE",
        "AMS" => "AMEX",
        other => other,
    }
}

fn daily_order_limit_key(signal: &Signal) -> Option<(&str, DailyOrderSide)> {
    match signal {
        Signal::Buy { symbol, .. } => Some((symbol.as_str(), DailyOrderSide::Buy)),
        Signal::Sell { symbol, .. } => Some((symbol.as_str(), DailyOrderSide::Sell)),
        Signal::Hold => None,
    }
}

/// KIS API 응답에서 잔고 부족 오류인지 판별.
///
/// 알려진 KIS 에러코드/메시지:
/// - `APBK0013`: 주문가능금액 부족
/// - `APBK0915`: 잔고 부족
/// - `APBK0017`: 주문가능금액이 없습니다
/// - msg1 키워드: "잔고부족", "잔고 부족", "주문가능금액부족", "주문가능금액 부족"
fn is_insufficient_balance_error(msg: &str) -> bool {
    msg.contains("잔고부족")
        || msg.contains("잔고 부족")
        || msg.contains("주문가능금액부족")
        || msg.contains("주문가능금액 부족")
        || msg.contains("APBK0013")
        || msg.contains("APBK0915")
        || msg.contains("APBK0017")
}

/// KIS 모의투자에서 해당 종목/거래소가 지원되지 않는 에러 감지.
///
/// 발생 상황:
/// - AMEX(NYSE Arca) 거래소: 모의투자에서 AMEX 주문 미지원
/// - 일부 ETF(QQQM 등): KIS 모의투자 지원 종목 목록에 미포함
///
/// 이 에러는 자동매매 매도 루프에서 스킵(Ok 반환)하여 스팸 방지.
/// 수동 UI 주문에서는 그대로 사용자에게 표시됨.
fn is_paper_unsupported_error(msg: &str) -> bool {
    msg.contains("해당업무가 제공되지 않습니다")
        || msg.contains("모의투자 미지원")
        || msg.contains("PAPER_OVERSEAS_UNSUPPORTED")
}

#[cfg(test)]
pub(crate) mod test_support;
