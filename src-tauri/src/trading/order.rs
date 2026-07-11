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
///  ⑩ KIS rate-limit(EGW00133) 오류 시 1초 대기 × 최대 3회 재시도
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
        OrderStore, PendingOrderStore, StatsStore, TradeStore,
    },
    trading::{
        guard::{GuardDecision, TradeGuard},
        position::{OverseasPositionTracker, PositionTracker},
        risk::{DailyOrderSide, RiskManager},
        strategy::Signal,
    },
};

mod conflicts;
mod fills;
mod submission;

use conflicts::pending_conflict_reason_for_scope;
pub use submission::SubmissionOutcome;

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
    /// 체결 event의 durable side effect 적용을 시작했는지 여부
    #[serde(default)]
    pub application_started: bool,
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

pub struct OrderManager {
    // ── ② 미체결 주문 풀 ────────────────────────────────────────────
    /// KIS odno → PendingOrder
    pending: HashMap<String, PendingOrder>,

    // ── ③ 중복 방지 인덱스 ─────────────────────────────────────────
    /// symbol → odno (미체결 주문이 있는 종목)
    symbol_to_odno: HashMap<String, String>,
    /// provider 주문 접수 전 local in-flight 예약. 느린 broker 호출 중 중복/반대 주문을 막는다.
    submitting: HashMap<(BrokerScope, String), OrderSide>,

    // ── ⑪ 잔고 부족 매수 정지 ────────────────────────────────────
    /// true = KIS 잔고부족 응답 수신 후 매수 일시 정지.
    /// 매도 체결 또는 수동 해제 시 false로 전환됨.
    pub buy_suspended: bool,
    /// 매수 정지 사유 (KIS 응답 msg1)
    pub buy_suspended_reason: Option<String>,
    /// 주문 상태 영속화 실패 시 매수/매도 모두 차단한다.
    persistence_blocked: bool,

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
    /// 전략 신호와 주문 실행 사이의 반복매매/휩소 방어 계층
    trade_guard: TradeGuard,
    /// 자동매매 시작 시점의 broker/account scope. 실행 중 프로파일 전환과 분리한다.
    execution_scope: BrokerScope,
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
        discord: Option<Arc<DiscordNotifier>>,
    ) -> Self {
        Self {
            pending: HashMap::new(),
            symbol_to_odno: HashMap::new(),
            submitting: HashMap::new(),
            buy_suspended: false,
            buy_suspended_reason: None,
            persistence_blocked: false,
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
            trade_guard: TradeGuard::default(),
            execution_scope: BrokerScope::kis_legacy(),
            discord,
        }
    }

    // ── 공개 API ────────────────────────────────────────────────────

    /// 자동매매 실행 scope를 시작 시점 broker/account로 고정한다.
    pub fn set_execution_scope(&mut self, scope: BrokerScope) {
        tracing::info!("주문 실행 scope 설정: {:?}", scope);
        self.execution_scope = scope;
    }

    pub fn execution_scope(&self) -> &BrokerScope {
        &self.execution_scope
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

    /// 주문 취소 이벤트 처리
    pub async fn on_cancel(&mut self, odno: &str) -> Result<()> {
        let Some(pending) = self.pending.remove(odno) else {
            return Ok(());
        };

        self.symbol_to_odno.remove(&pending.record.symbol);

        if let Err(error) = self.persist_pending_orders().await {
            self.track_pending_order(odno.to_string(), pending.clone());
            return Err(error);
        }

        let mut record = pending.record;
        record.status = OrderStatus::Cancelled;
        if let Err(e) = self.order_store.append(record.clone()).await {
            tracing::error!("주문 기록 저장 실패 (Cancelled): {}", e);
        }

        tracing::info!("주문 취소 처리: {} (odno: {})", record.symbol, odno);
        Ok(())
    }

    /// 잔고 부족으로 매수가 정지됐는지 여부
    pub fn is_buy_suspended(&self) -> bool {
        self.buy_suspended
    }

    /// 잔고 부족 매수 정지 수동 해제 (예: 입금 후 사용자 요청)
    pub fn clear_buy_suspension(&mut self) {
        if self.persistence_blocked {
            tracing::warn!("영속화 장애 차단은 수동 매수 정지 해제로 해제할 수 없습니다.");
            return;
        }
        if self.buy_suspended {
            self.buy_suspended = false;
            self.buy_suspended_reason = None;
            tracing::info!("잔고 부족 매수 정지 해제 (수동)");
        }
    }

    pub fn block_for_persistence_failure(&mut self, reason: String) {
        self.persistence_blocked = true;
        self.buy_suspended = true;
        self.buy_suspended_reason = Some(reason);
    }

    pub async fn suspend_buying_for_account_sync(&mut self, detail: String) {
        let reason = format!("계좌 동기화 실패: {detail}");
        let should_notify = self.buy_suspended_reason.as_deref() != Some(reason.as_str());
        self.buy_suspended = true;
        self.buy_suspended_reason = Some(reason.clone());
        if should_notify {
            if let Some(discord) = &self.discord {
                let _ = discord
                    .send(NotificationEvent::error(
                        "자동매매 신규 매수 중단",
                        format!(
                            "{reason} 계좌 조회가 정상화될 때까지 신규 매수를 제출하지 않습니다."
                        ),
                    ))
                    .await;
            }
        }
    }

    pub fn clear_account_sync_suspension(&mut self) {
        if self
            .buy_suspended_reason
            .as_deref()
            .is_some_and(|reason| reason.starts_with("계좌 동기화 실패:"))
        {
            self.buy_suspended = false;
            self.buy_suspended_reason = None;
            tracing::info!("계좌 동기화 정상화 — 신규 매수 차단 해제");
        }
    }

    /// 미체결 주문 건수
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// 특정 종목의 미체결 odno 조회
    pub fn get_pending_odno(&self, symbol: &str) -> Option<&str> {
        self.symbol_to_odno.get(symbol).map(String::as_str)
    }

    /// 미체결 주문 목록 (UI 표시용)
    pub fn pending_orders(&self) -> Vec<&PendingOrder> {
        self.pending.values().collect()
    }

    pub fn restore_pending_orders(&mut self, orders: Vec<PendingOrder>) {
        self.pending.clear();
        self.symbol_to_odno.clear();
        for pending in orders {
            let Some(key) = pending
                .record
                .provider_order_id
                .clone()
                .or_else(|| pending.record.kis_order_id.clone())
                .filter(|value| !value.trim().is_empty())
            else {
                tracing::error!(
                    "미체결 주문 복원 제외: provider 주문번호 없음 ({})",
                    pending.record.id
                );
                continue;
            };
            self.track_pending_order(key, pending);
        }
    }

    pub async fn persist_pending_orders(&self) -> Result<()> {
        let snapshot: Vec<PendingOrder> = self.pending.values().cloned().collect();
        self.pending_order_store.replace(&snapshot).await
    }

    /// 외부 주문 경로(수동 주문 등)에서 받은 provider 주문을 미체결 풀에 편입한다.
    pub fn track_pending_order(&mut self, key: String, pending: PendingOrder) {
        self.symbol_to_odno
            .insert(pending.record.symbol.clone(), key.clone());
        self.pending.insert(key, pending);
    }

    /// provider 정정 성공 후 로컬 pending snapshot과 주문번호 key를 갱신한다.
    pub fn update_pending_order_snapshot(
        &mut self,
        order_id: &str,
        new_order_id: Option<&str>,
        quantity: Option<u64>,
        price: Option<u64>,
        order_type: Option<String>,
    ) -> bool {
        let Some(mut pending) = self.pending.remove(order_id) else {
            return false;
        };
        if let Some(quantity) = quantity {
            pending.record.quantity = quantity;
        }
        if let Some(price) = price {
            pending.record.price = price;
            pending.order_price = price;
        }
        if let Some(order_type) = order_type {
            pending.record.order_type = order_type;
        }
        let key = new_order_id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(order_id)
            .to_string();
        pending.record.kis_order_id = Some(key.clone());
        pending.record.provider_order_id = Some(key.clone());
        self.symbol_to_odno
            .insert(pending.record.symbol.clone(), key.clone());
        self.pending.insert(key, pending);
        true
    }

    pub async fn current_exchange_rate_krw(&self) -> f64 {
        *self.exchange_rate_krw.read().await
    }

    /// 일 초기화 — 자동매매 시작 시 또는 자정 리셋 시 호출
    pub fn reset_day(&mut self) {
        self.submitting.clear();
        self.trade_guard.reset_day();
        // 매수 중단 사유는 자정만으로 해소되지 않는다. 계좌 재동기화, 매도 체결,
        // 영속화 복구 또는 사용자의 명시적 해제에서만 상태를 바꾼다.
        if !self.pending.is_empty() {
            tracing::info!(
                "일 초기화: 미체결 주문 {}건은 provider reconciliation을 위해 유지",
                self.pending.len()
            );
        }
    }

    pub(crate) fn pending_conflict_reason_for_scope(
        &self,
        broker_scope: &BrokerScope,
        symbol: &str,
        requested_side: &OrderSide,
    ) -> Option<String> {
        pending_conflict_reason_for_scope(
            self.pending.values(),
            broker_scope,
            symbol,
            requested_side,
        )
    }

    // ── 내부 구현 ────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    async fn process_buy(
        &mut self,
        symbol: String,
        symbol_name: String,
        quantity: u64,
        reason: String,
        total_balance: i64,
        exchange: Option<String>,
        tick_price: u64,
        strategy_id: Option<String>,
        broker_scope: BrokerScope,
    ) -> Result<()> {
        // ⑪ 잔고 부족 매수 정지 체크
        if self.buy_suspended {
            tracing::debug!(
                "매수 스킵 — 잔고 부족 정지 중: {} (사유: {})",
                symbol,
                self.buy_suspended_reason.as_deref().unwrap_or("알 수 없음")
            );
            return Ok(());
        }

        // ③ 동일/반대 방향 미체결 주문 방지
        if let Some(reason) =
            self.pending_conflict_reason_for_scope(&broker_scope, &symbol, &OrderSide::Buy)
        {
            tracing::info!("매수 스킵 — {}", reason);
            return Ok(());
        }

        let is_overseas = exchange.is_some();
        let exchange_rate = if is_overseas {
            *self.exchange_rate_krw.read().await
        } else {
            1.0
        };

        // ⑧ 리스크 검증 + ATR 기반 수량 산정
        let quantity = {
            let risk = self.risk_manager.lock().await;
            if !risk.can_trade() {
                tracing::warn!(
                    "리스크 한도 초과 — 매수 거부: {} (비상정지 or 손실한도)",
                    symbol
                );
                return Ok(());
            }
            let adjusted_quantity = risk.volatility_adjusted_quantity(
                &symbol,
                quantity,
                tick_price,
                total_balance,
                is_overseas,
                exchange_rate,
            );
            if adjusted_quantity == 0 {
                return Ok(());
            }
            if total_balance > 0 {
                let est_amount = estimate_order_amount_krw(
                    tick_price,
                    adjusted_quantity,
                    is_overseas,
                    exchange_rate,
                );
                if !risk.check_position_size(est_amount, total_balance) {
                    tracing::warn!(
                        "포지션 비중 초과 — 매수 거부: {} (추정 {}원 / 총잔고 {}원)",
                        symbol,
                        est_amount,
                        total_balance
                    );
                    return Ok(());
                }
            }
            adjusted_quantity
        };

        // ① 주문 실행 — 해외(지정가 USD) / 국내(시장가 KRW) 분기
        let order_result = if let Some(ref exch) = exchange {
            // 해외 지정가 주문 (KIS 해외는 시장가 미지원)
            // fetch_overseas_tick 반환 코드(NAS/NYS/AMS) → 주문 코드(NASD/NYSE/AMEX) 변환
            let order_exch = match exch.as_str() {
                "NAS" => "NASD",
                "NYS" => "NYSE",
                "AMS" => "AMEX",
                other => other,
            };
            let usd_price = tick_price as f64 / 100.0;
            let req = OverseasOrderRequest {
                symbol: symbol.clone(),
                exchange: order_exch.to_string(),
                side: RestOrderSide::Buy,
                quantity,
                price: usd_price,
            };
            tracing::info!(
                "해외 매수 주문 시도: {} {}주 @ ${:.2} ({}) — {}",
                symbol,
                quantity,
                usd_price,
                order_exch,
                reason
            );
            self.place_overseas_with_retry(&req).await
        } else {
            // 국내 시장가 매수
            let req = OrderRequest {
                symbol: symbol.clone(),
                side: RestOrderSide::Buy,
                order_type: OrderType::Market,
                quantity,
                price: 0,
            };
            self.place_with_retry(&req).await
        };

        // ⑪ 잔고 부족 에러 감지 — 이후 매수 주문 정지
        let response = match order_result {
            Ok(resp) => resp,
            Err(e) => {
                let msg = e.to_string();
                if is_insufficient_balance_error(&msg) {
                    self.buy_suspended = true;
                    self.buy_suspended_reason = Some(msg.clone());
                    self.record_failed_order(
                        symbol.clone(),
                        symbol_name.clone(),
                        OrderSide::Buy,
                        quantity,
                        tick_price,
                        if exchange.is_some() {
                            "Limit"
                        } else {
                            "Market"
                        },
                        msg.clone(),
                    )
                    .await;
                    tracing::warn!(
                        "잔고 부족 감지 — 매수 주문 정지: {} (매도 체결 또는 수동 해제 시 재개) | {}",
                        symbol,
                        msg
                    );
                    return Ok(()); // 에러 전파 없이 정상 종료 (상위 루프가 계속 실행되도록)
                }
                self.record_failed_order(
                    symbol.clone(),
                    symbol_name.clone(),
                    OrderSide::Buy,
                    quantity,
                    tick_price,
                    if exchange.is_some() {
                        "Limit"
                    } else {
                        "Market"
                    },
                    msg,
                )
                .await;
                return Err(e);
            }
        };
        tracing::info!(
            "매수 주문 접수: {} {}주 — {} (provider=kis tr_id={} odno={})",
            symbol,
            quantity,
            reason,
            response.tr_id,
            response.odno
        );

        let order_price = if exchange.is_some() { tick_price } else { 0 };
        self.register_pending(
            symbol,
            symbol_name,
            OrderSide::Buy,
            quantity,
            reason,
            response,
            exchange.map(|exch| order_exchange_code(&exch).to_string()),
            strategy_id,
            tick_price,
            order_price,
            broker_scope,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn process_sell(
        &mut self,
        symbol: String,
        symbol_name: String,
        quantity: u64,
        reason: String,
        exchange: Option<String>,
        tick_price: u64,
        strategy_id: Option<String>,
        broker_scope: BrokerScope,
    ) -> Result<()> {
        // ③ 동일/반대 방향 미체결 주문 방지
        if let Some(reason) =
            self.pending_conflict_reason_for_scope(&broker_scope, &symbol, &OrderSide::Sell)
        {
            tracing::info!("매도 스킵 — {}", reason);
            return Ok(());
        }

        // 보유 포지션 확인
        // - 국내: PositionTracker
        // - 해외: OverseasPositionTracker
        let sell_qty = if exchange.is_none() {
            let position_qty = {
                let tracker = self.position_tracker.lock().await;
                tracker.get(&symbol).map(|p| p.quantity).unwrap_or(0)
            };
            if position_qty == 0 {
                tracing::debug!("매도 스킵 — {} 보유 포지션 없음", symbol);
                return Ok(());
            }
            quantity.min(position_qty)
        } else {
            let position_qty = {
                let tracker = self.overseas_position_tracker.lock().await;
                tracker.get(&symbol).map(|p| p.quantity).unwrap_or(0)
            };
            if position_qty == 0 {
                tracing::debug!("해외 매도 스킵 — {} 보유 포지션 없음", symbol);
                return Ok(());
            }
            quantity.min(position_qty)
        };

        // ⑧ 비상정지 확인 (매도는 손실한도와 무관하게 실행 허용)
        if self.risk_manager.lock().await.is_emergency_stop() {
            tracing::warn!("비상정지 상태 — 매도 거부: {}", symbol);
            return Ok(());
        }

        // ① 주문 실행 — 해외(지정가 USD) / 국내(시장가 KRW) 분기
        let response = if let Some(ref exch) = exchange {
            let order_exch = match exch.as_str() {
                "NAS" => "NASD",
                "NYS" => "NYSE",
                "AMS" => "AMEX",
                other => other,
            };
            let usd_price = tick_price as f64 / 100.0;
            let req = OverseasOrderRequest {
                symbol: symbol.clone(),
                exchange: order_exch.to_string(),
                side: RestOrderSide::Sell,
                quantity: sell_qty,
                price: usd_price,
            };
            tracing::info!(
                "해외 매도 주문 시도: {} {}주 @ ${:.2} ({}) — {}",
                symbol,
                sell_qty,
                usd_price,
                order_exch,
                reason
            );
            match self.place_overseas_with_retry(&req).await {
                Ok(resp) => resp,
                Err(e) => {
                    let msg = e.to_string();
                    // KIS 모의투자에서 특정 종목/거래소(AMEX 등) 매도 미지원
                    // → 에러 전파 없이 스킵하여 자동매매 루프 스팸 방지
                    if is_paper_unsupported_error(&msg) {
                        self.record_failed_order(
                            symbol.clone(),
                            symbol_name.clone(),
                            OrderSide::Sell,
                            sell_qty,
                            tick_price,
                            "Limit",
                            msg.clone(),
                        )
                        .await;
                        tracing::warn!(
                            "모의투자 매도 미지원 — 스킵: {} ({}) | {}",
                            symbol,
                            order_exch,
                            msg
                        );
                        return Ok(());
                    }
                    self.record_failed_order(
                        symbol.clone(),
                        symbol_name.clone(),
                        OrderSide::Sell,
                        sell_qty,
                        tick_price,
                        "Limit",
                        msg,
                    )
                    .await;
                    return Err(e);
                }
            }
        } else {
            let req = OrderRequest {
                symbol: symbol.clone(),
                side: RestOrderSide::Sell,
                order_type: OrderType::Market,
                quantity: sell_qty,
                price: 0,
            };
            match self.place_with_retry(&req).await {
                Ok(resp) => resp,
                Err(e) => {
                    let msg = e.to_string();
                    self.record_failed_order(
                        symbol.clone(),
                        symbol_name.clone(),
                        OrderSide::Sell,
                        sell_qty,
                        0,
                        "Market",
                        msg,
                    )
                    .await;
                    return Err(e);
                }
            }
        };
        tracing::info!(
            "매도 주문 접수: {} {}주 — {} (provider=kis tr_id={} odno={})",
            symbol,
            sell_qty,
            reason,
            response.tr_id,
            response.odno
        );

        let order_price = if exchange.is_some() { tick_price } else { 0 };
        self.register_pending(
            symbol,
            symbol_name,
            OrderSide::Sell,
            sell_qty,
            reason,
            response,
            exchange.map(|exch| order_exchange_code(&exch).to_string()),
            strategy_id,
            tick_price,
            order_price,
            broker_scope,
        )
        .await
    }

    /// 미체결 풀 등록 + ⑥ 주문 기록 저장 (Pending)
    #[allow(clippy::too_many_arguments)]
    async fn register_pending(
        &mut self,
        symbol: String,
        symbol_name: String,
        side: OrderSide,
        quantity: u64,
        reason: String,
        response: OrderResponse,
        exchange: Option<String>,
        strategy_id: Option<String>,
        signal_price: u64,
        order_price: u64,
        broker_scope: BrokerScope,
    ) -> Result<()> {
        let mut record = OrderRecord::new(
            symbol.clone(),
            symbol_name,
            side.clone(),
            quantity,
            order_price,
            if exchange.is_some() {
                "Limit"
            } else {
                "Market"
            }
            .to_string(),
        )
        .with_provider_trace(
            "kis",
            Some(response.odno.clone()),
            None,
            Some(response.tr_id.clone()),
        );
        // KIS 모의투자 환경에서 ondo가 빈 문자열로 반환될 수 있음 → 로컬 UUID로 대체
        let odno = if response.odno.is_empty() {
            format!("LOCAL-{}", uuid::Uuid::new_v4())
        } else {
            response.odno
        };
        record.kis_order_id = Some(odno.clone());
        record.provider_order_id = Some(odno.clone());

        if let Err(e) = self.order_store.append(record.clone()).await {
            tracing::error!("주문 기록 저장 실패 (Pending): {}", e);
        }

        tracing::info!(
            "주문 trace 저장: provider={} tr_id={} order_id={}",
            record.provider.as_deref().unwrap_or("unknown"),
            record.provider_tr_id.as_deref().unwrap_or("-"),
            record.provider_order_id.as_deref().unwrap_or("-")
        );

        self.symbol_to_odno.insert(symbol.clone(), odno.clone());
        let guard_signal = match side {
            OrderSide::Buy => Signal::Buy {
                symbol: symbol.clone(),
                quantity,
                reason: reason.clone(),
            },
            OrderSide::Sell => Signal::Sell {
                symbol: symbol.clone(),
                quantity,
                reason: reason.clone(),
            },
        };
        self.trade_guard
            .record_submitted_for_scope(&broker_scope, &guard_signal);
        self.risk_manager
            .lock()
            .await
            .record_order_submitted_for_scope(
                &broker_scope,
                strategy_id.as_deref().unwrap_or("unknown"),
                &symbol,
                match side {
                    OrderSide::Buy => DailyOrderSide::Buy,
                    OrderSide::Sell => DailyOrderSide::Sell,
                },
            );
        self.pending.insert(
            odno,
            PendingOrder {
                record,
                signal_reason: reason,
                exchange,
                filled_quantity: 0,
                filled_notional: 0,
                confirmed_filled_quantity: 0,
                confirmed_avg_price: 0,
                application_started: false,
                application_pnl: None,
                strategy_id,
                signal_price,
                order_price,
                broker_scope,
                client_order_id: None,
                provider_status: Some("pending".to_string()),
            },
        );

        Ok(())
    }

    /// KIS가 주문 접수 자체를 거부한 경우 pending과 분리해 실패 주문 이력으로 남긴다.
    #[allow(clippy::too_many_arguments)]
    async fn record_failed_order(
        &self,
        symbol: String,
        symbol_name: String,
        side: OrderSide,
        quantity: u64,
        price: u64,
        order_type: &str,
        error_message: String,
    ) {
        let mut record = OrderRecord::new(
            symbol,
            symbol_name,
            side,
            quantity,
            price,
            order_type.to_string(),
        );
        record.status = OrderStatus::Failed;
        record.error_message = Some(error_message);
        if let Err(e) = self.order_store.append(record).await {
            tracing::error!("주문 기록 저장 실패 (Failed): {}", e);
        }
    }

    /// ⑩ 재시도 로직 — KIS rate-limit 오류 시 2초 대기 × 최대 3회
    /// - EGW00133: 초당 주문건수 초과
    /// - EGW00201: 초당 거래건수 초과 (price/order 공통)
    async fn place_with_retry(&self, req: &OrderRequest) -> Result<OrderResponse> {
        const MAX_RETRIES: u32 = 3;
        let mut last_err = anyhow::anyhow!("주문 최대 재시도 횟수(3회) 초과");

        for attempt in 0..MAX_RETRIES {
            let client = self.rest_client.read().await.clone();
            match client.place_order(req).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    let msg = e.to_string();
                    let is_rate_limit = msg.contains("EGW00133") || msg.contains("EGW00201");
                    if is_rate_limit && attempt < MAX_RETRIES - 1 {
                        tracing::warn!(
                            "KIS rate-limit ({}) — 2초 후 재시도 ({}/{})",
                            if msg.contains("EGW00201") {
                                "EGW00201"
                            } else {
                                "EGW00133"
                            },
                            attempt + 1,
                            MAX_RETRIES
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        last_err = e;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Err(last_err)
    }

    /// ⑩ 해외 주문 재시도 — 국내와 동일한 rate-limit 재시도 로직
    async fn place_overseas_with_retry(&self, req: &OverseasOrderRequest) -> Result<OrderResponse> {
        const MAX_RETRIES: u32 = 3;
        let mut last_err = anyhow::anyhow!("해외 주문 최대 재시도 횟수(3회) 초과");

        for attempt in 0..MAX_RETRIES {
            let client = self.rest_client.read().await.clone();
            match client.place_overseas_order(req).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    let msg = e.to_string();
                    let is_rate_limit = msg.contains("EGW00133") || msg.contains("EGW00201");
                    if is_rate_limit && attempt < MAX_RETRIES - 1 {
                        tracing::warn!(
                            "KIS rate-limit (해외, {}) — 2초 후 재시도 ({}/{})",
                            if msg.contains("EGW00201") {
                                "EGW00201"
                            } else {
                                "EGW00133"
                            },
                            attempt + 1,
                            MAX_RETRIES
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        last_err = e;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Err(last_err)
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
