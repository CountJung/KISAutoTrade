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
use tokio::sync::{Mutex, RwLock};

use crate::{
    api::rest::{
        KisRestClient, OrderRequest, OrderResponse, OrderSide as RestOrderSide, OrderType,
        OverseasOrderRequest,
    },
    broker::{BrokerId, BrokerScope},
    market_hours::is_domestic_symbol,
    notifications::{discord::DiscordNotifier, types::NotificationEvent},
    storage::{
        order_store::{OrderRecord, OrderSide, OrderStatus},
        trade_store::{TradeRecord, TradeSide},
        OrderStore, StatsStore, TradeStore,
    },
    trading::{
        guard::{GuardDecision, TradeGuard},
        position::{OverseasPositionTracker, PositionTracker},
        risk::{DailyOrderSide, RiskManager},
        strategy::Signal,
    },
};

// ────────────────────────────────────────────────────────────────────
// PendingOrder — 미체결 주문 항목
// ────────────────────────────────────────────────────────────────────

/// 미체결(Pending) 상태 주문 항목
#[derive(Debug, Clone)]
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

    // ── ⑪ 잔고 부족 매수 정지 ────────────────────────────────────
    /// true = KIS 잔고부족 응답 수신 후 매수 일시 정지.
    /// 매도 체결 또는 수동 해제 시 false로 전환됨.
    pub buy_suspended: bool,
    /// 매수 정지 사유 (KIS 응답 msg1)
    pub buy_suspended_reason: Option<String>,

    // ── 공유 의존성 (Arc) ───────────────────────────────────────────
    /// KIS REST 클라이언트 (프로파일 전환 시 내부 Arc만 교체됨)
    rest_client: Arc<RwLock<Arc<KisRestClient>>>,
    order_store: Arc<OrderStore>,
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
    pub fn new(
        rest_client: Arc<RwLock<Arc<KisRestClient>>>,
        order_store: Arc<OrderStore>,
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
            buy_suspended: false,
            buy_suspended_reason: None,
            rest_client,
            order_store,
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

    /// ④ 체결 이벤트 처리 (WebSocket H0STCNI0 또는 폴링에서 호출)
    ///
    /// - `odno`: KIS 주문번호
    /// - `filled_qty`: 주문번호 기준 누적 체결 수량
    /// - `avg_price`: 누적 체결 평균가(국내 원, 해외 USD × 100 cents)
    pub async fn on_fill(&mut self, odno: &str, filled_qty: u64, avg_price: u64) -> Result<()> {
        let Some(pending) = self.pending.get(odno).cloned() else {
            tracing::debug!("on_fill: odno {} 는 미체결 풀에 없음 (이미 처리됨)", odno);
            return Ok(());
        };

        let symbol = pending.record.symbol.clone();
        let symbol_name = pending.record.symbol_name.clone();
        let order_quantity = pending.record.quantity;
        let cumulative_filled = filled_qty.min(order_quantity);
        if cumulative_filled <= pending.filled_quantity {
            tracing::debug!(
                "on_fill: odno {} 누적 체결량 변화 없음 ({} / {})",
                odno,
                pending.filled_quantity,
                order_quantity
            );
            return Ok(());
        }

        let delta_qty = cumulative_filled - pending.filled_quantity;
        let is_complete = cumulative_filled >= order_quantity;

        if is_complete {
            self.pending.remove(odno);
            // ③ 중복 방지 인덱스 정리
            self.symbol_to_odno.remove(&symbol);
        } else if let Some(current) = self.pending.get_mut(odno) {
            current.filled_quantity = cumulative_filled;
            current.record.status = OrderStatus::PartiallyFilled;
            current.record.price = avg_price;
        }

        // ⑤ 포지션 연동 + ⑦ 매도 시 PnL 계산 (포지션 업데이트 전에 avg_price 읽기)
        let is_sell = matches!(pending.record.side, OrderSide::Sell);
        let is_overseas = pending.exchange.is_some() || !is_domestic_symbol(&symbol);
        let pnl = if is_overseas {
            let exchange = pending
                .exchange
                .clone()
                .unwrap_or_else(|| "UNKNOWN".to_string());
            let mut tracker = self.overseas_position_tracker.lock().await;
            match &pending.record.side {
                OrderSide::Buy => {
                    tracker.on_buy(
                        symbol.clone(),
                        symbol_name.clone(),
                        exchange.clone(),
                        delta_qty,
                        avg_price,
                    );
                    tracing::info!(
                        "해외 매수 체결: {} {} @ ${:.2} ({})",
                        symbol,
                        delta_qty,
                        avg_price as f64 / 100.0,
                        exchange
                    );
                    0i64
                }
                OrderSide::Sell => {
                    let buy_avg = tracker
                        .get(&symbol)
                        .map(|p| p.avg_price_cents)
                        .unwrap_or(0.0);
                    let realized = (avg_price as f64 - buy_avg) * delta_qty as f64;
                    tracker.on_sell(&symbol, delta_qty);
                    tracing::info!(
                        "해외 매도 체결: {} {} @ ${:.2} ({}) (PnL: ${:.2})",
                        symbol,
                        delta_qty,
                        avg_price as f64 / 100.0,
                        exchange,
                        realized / 100.0
                    );
                    realized as i64
                }
            }
        } else {
            let mut tracker = self.position_tracker.lock().await;
            match &pending.record.side {
                OrderSide::Buy => {
                    tracker.on_buy(symbol.clone(), symbol_name.clone(), delta_qty, avg_price);
                    tracing::info!("매수 체결: {} {} @ {}원", symbol, delta_qty, avg_price);
                    0i64
                }
                OrderSide::Sell => {
                    // 매도 전 매입 평균가 조회 (PnL 계산 후 포지션 감소)
                    let buy_avg = tracker.get(&symbol).map(|p| p.avg_price).unwrap_or(0.0);
                    let realized = (avg_price as f64 - buy_avg) * delta_qty as f64;
                    tracker.on_sell(&symbol, delta_qty);
                    tracing::info!(
                        "매도 체결: {} {} @ {}원 (PnL: {}원)",
                        symbol,
                        delta_qty,
                        avg_price,
                        realized as i64
                    );
                    realized as i64
                }
            }
        };

        let exchange_rate = if is_overseas {
            *self.exchange_rate_krw.read().await
        } else {
            1.0
        };

        let fee = if is_overseas {
            calculate_overseas_fee_cents(avg_price, delta_qty)
        } else {
            calculate_domestic_fee(avg_price, delta_qty, is_sell)
        };
        let fee_krw = if is_overseas {
            ((fee as f64 / 100.0) * exchange_rate).round().max(0.0) as u64
        } else {
            fee
        };
        let pnl_krw = if is_overseas {
            ((pnl as f64 / 100.0) * exchange_rate).round() as i64
        } else {
            pnl
        };

        // ⑦ 매도 체결 시 통계/리스크 반영 + ⑪ 잔고 부족 정지 자동 해제
        if is_sell {
            // 매도 체결 = 자본 확보 → 매수 정지 해제
            if self.buy_suspended {
                self.buy_suspended = false;
                self.buy_suspended_reason = None;
                tracing::info!(
                    "매도 체결로 자본 확보 — 잔고 부족 매수 정지 해제: {}",
                    symbol
                );
            }
            {
                let mut risk = self.risk_manager.lock().await;
                risk.record_pnl(pnl_krw);
                risk.record_strategy_symbol_pnl_for_scope(
                    &pending.broker_scope,
                    pending.strategy_id.as_deref().unwrap_or("unknown"),
                    &symbol,
                    pnl_krw,
                );
            }

            let today = chrono::Local::now().date_naive();
            if let Ok(mut stats) = self.stats_store.get_by_date(today).await {
                stats.total_trades += 1;
                if pnl_krw > 0 {
                    stats.winning_trades += 1;
                    stats.gross_profit += pnl_krw;
                } else if pnl_krw < 0 {
                    stats.losing_trades += 1;
                    stats.gross_loss += pnl_krw;
                }
                stats.fees_paid += fee_krw;
                stats.recalculate();
                if let Err(e) = self.stats_store.upsert(stats).await {
                    tracing::error!("통계 저장 실패: {}", e);
                }
            }
        } else {
            // 매수 체결 시 수수료만 통계 업데이트 (PnL은 매도 시 확정)
            let today = chrono::Local::now().date_naive();
            if let Ok(mut stats) = self.stats_store.get_by_date(today).await {
                stats.fees_paid += fee_krw;
                stats.recalculate();
                if let Err(e) = self.stats_store.upsert(stats).await {
                    tracing::error!("통계 저장 실패 (매수 수수료): {}", e);
                }
            }
        }

        // ⑥ 주문 기록 (체결 증가분 기준으로 재기록)
        let mut filled_record = pending.record.clone();
        filled_record.status = if is_complete {
            OrderStatus::Filled
        } else {
            OrderStatus::PartiallyFilled
        };
        filled_record.price = avg_price;
        filled_record.quantity = delta_qty;
        if let Err(e) = self.order_store.append(filled_record).await {
            tracing::error!("주문 기록 저장 실패 (체결): {}", e);
        }

        // ⑥-b TradeStore 저장 (자동매매 로컬 체결 기록)
        let trade_side = match &pending.record.side {
            OrderSide::Buy => TradeSide::Buy,
            OrderSide::Sell => TradeSide::Sell,
        };
        let order_id = pending.record.kis_order_id.clone().unwrap_or_default();
        let provider_order_id = pending
            .record
            .provider_order_id
            .clone()
            .or_else(|| Some(order_id.clone()).filter(|id| !id.is_empty()));
        let trade_record = if is_overseas {
            TradeRecord::new_overseas(
                symbol.clone(),
                symbol_name.clone(),
                trade_side,
                delta_qty,
                avg_price,
                fee,
                order_id,
                pending.strategy_id.clone(),
                pending.signal_reason.clone(),
                pending
                    .exchange
                    .clone()
                    .unwrap_or_else(|| "UNKNOWN".to_string()),
                exchange_rate,
                is_sell.then_some(pnl),
            )
            .with_execution_prices(pending.signal_price, pending.order_price)
            .with_provider_trace(
                pending.record.provider.clone(),
                provider_order_id.clone(),
                pending.record.provider_request_id.clone(),
                pending.record.provider_tr_id.clone(),
            )
        } else {
            TradeRecord::new(
                symbol.clone(),
                symbol_name.clone(),
                trade_side,
                delta_qty,
                avg_price,
                fee,
                order_id,
                pending.strategy_id.clone(),
                pending.signal_reason.clone(), // 체결 원인 (전략 신호 이유)
            )
            .with_execution_prices(pending.signal_price, pending.order_price)
            .with_provider_trace(
                pending.record.provider.clone(),
                provider_order_id.clone(),
                pending.record.provider_request_id.clone(),
                pending.record.provider_tr_id.clone(),
            )
        };
        tracing::info!(
            "체결 trace 저장: provider={} tr_id={} order_id={} request_id={}",
            trade_record.provider.as_deref().unwrap_or("unknown"),
            trade_record.provider_tr_id.as_deref().unwrap_or("-"),
            trade_record.provider_order_id.as_deref().unwrap_or("-"),
            trade_record.provider_request_id.as_deref().unwrap_or("-")
        );
        if let Err(e) = self.trade_store.append(trade_record).await {
            tracing::error!("TradeStore 저장 실패: {}", e);
        }

        // ⑨ Discord 알림
        if let Some(discord) = &self.discord {
            let side_str = if !is_sell { "매수" } else { "매도" };
            let pnl_str = if is_sell {
                if is_overseas {
                    format!(
                        " (PnL: {}${:.2}, {}{}원, 수수료 약 {}원)",
                        if pnl >= 0 { "+" } else { "" },
                        pnl as f64 / 100.0,
                        if pnl_krw >= 0 { "+" } else { "" },
                        pnl_krw,
                        fee_krw
                    )
                } else {
                    format!(
                        " (PnL: {}{}원, 수수료: {}원)",
                        if pnl >= 0 { "+" } else { "" },
                        pnl,
                        fee
                    )
                }
            } else if is_overseas {
                format!(" (수수료 약 {}원)", fee_krw)
            } else {
                format!(" (수수료: {}원)", fee)
            };
            let price_text = if is_overseas {
                format!("${:.2}", avg_price as f64 / 100.0)
            } else {
                format!("{}원", avg_price)
            };
            let content = format!(
                "{} {} {}주 @ {}{}",
                symbol_name, side_str, delta_qty, price_text, pnl_str
            );
            let _ = discord.send(NotificationEvent::trade(content)).await;
        }

        Ok(())
    }

    /// 주문 취소 이벤트 처리
    pub async fn on_cancel(&mut self, odno: &str) -> Result<()> {
        let Some(pending) = self.pending.remove(odno) else {
            return Ok(());
        };

        self.symbol_to_odno.remove(&pending.record.symbol);

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
        if self.buy_suspended {
            self.buy_suspended = false;
            self.buy_suspended_reason = None;
            tracing::info!("잔고 부족 매수 정지 해제 (수동)");
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

    pub async fn current_exchange_rate_krw(&self) -> f64 {
        *self.exchange_rate_krw.read().await
    }

    /// 일 초기화 — 자동매매 시작 시 또는 자정 리셋 시 호출
    pub fn reset_day(&mut self) {
        let n = self.pending.len();
        self.pending.clear();
        self.symbol_to_odno.clear();
        self.trade_guard.reset_day();
        // 전일 잔고부족 정지도 초기화
        self.buy_suspended = false;
        self.buy_suspended_reason = None;
        if n > 0 {
            tracing::warn!("일 초기화: 미처리 미체결 주문 {}건 폐기", n);
        }
    }

    fn pending_conflict_reason_for_scope(
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
                strategy_id,
                signal_price,
                order_price,
                broker_scope,
            },
        );

        Ok(())
    }

    /// KIS가 주문 접수 자체를 거부한 경우 pending과 분리해 실패 주문 이력으로 남긴다.
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

    /// 종목명으로 미체결 주문을 체결 처리 (시장가 주문 자동 확인용)
    ///
    /// 폴링 루프에서 주문 접수 후 다음 틱에 호출 — 시장가 주문은 즉시 체결 가정
    pub async fn confirm_fill_by_symbol(&mut self, symbol: &str, fill_price: u64) -> Result<()> {
        let ondo = match self.symbol_to_odno.get(symbol).cloned() {
            Some(o) => o,
            None => return Ok(()), // 미체결 주문 없음
        };
        let qty = self
            .pending
            .get(&ondo)
            .map(|p| p.record.quantity)
            .unwrap_or(1);
        self.on_fill(&ondo, qty, fill_price).await
    }

    /// 주문번호 기반 체결 확인.
    ///
    /// KIS 당일 체결 내역에서 pending 주문번호를 찾아 실제 체결수량/체결금액으로 반영한다.
    /// 국내는 원화 정수, 해외는 USD × 100(cents) 단위로 `on_fill()`에 전달한다.
    pub async fn confirm_pending_fills_from_broker(&mut self) -> Result<()> {
        let pending: Vec<(BrokerId, String, String)> = self
            .pending
            .iter()
            .map(|(odno, order)| {
                (
                    pending_order_provider(order),
                    odno.clone(),
                    order.record.symbol.clone(),
                )
            })
            .collect();
        if pending.is_empty() {
            return Ok(());
        }

        let kis_pending: Vec<(String, String)> = pending
            .iter()
            .filter(|(broker_id, _, _)| *broker_id == BrokerId::Kis)
            .map(|(_, odno, symbol)| (odno.clone(), symbol.clone()))
            .collect();
        if !kis_pending.is_empty() {
            self.confirm_kis_pending_fills(kis_pending).await?;
        }

        let toss_pending = pending
            .iter()
            .filter(|(broker_id, _, _)| *broker_id == BrokerId::Toss)
            .count();
        if toss_pending > 0 {
            tracing::warn!(
                "Toss pending 체결 확인 adapter 미연결 — {}건 스킵 (order detail/list adapter 연결 필요)",
                toss_pending
            );
        }

        Ok(())
    }

    async fn confirm_kis_pending_fills(&mut self, pending: Vec<(String, String)>) -> Result<()> {
        let client = self.rest_client.read().await.clone();

        if pending.iter().any(|(_, symbol)| is_domestic_symbol(symbol)) {
            let executed = client.get_today_executed_orders().await?;
            for (odno, symbol) in pending
                .iter()
                .filter(|(_, symbol)| is_domestic_symbol(symbol))
            {
                let Some(order) = executed.iter().find(|o| o.odno == *odno) else {
                    continue;
                };
                let qty = order.tot_ccld_qty.parse::<u64>().unwrap_or(0);
                if qty == 0 {
                    continue;
                }
                let amount = order.tot_ccld_amt.parse::<u64>().unwrap_or(0);
                let avg_price = if amount > 0 {
                    amount / qty
                } else {
                    order.ord_unpr.parse::<u64>().unwrap_or(0)
                };
                if avg_price == 0 {
                    continue;
                }
                tracing::info!(
                    "국내 주문번호 기반 체결 확인: odno={} symbol={} qty={} avg={}",
                    odno,
                    symbol,
                    qty,
                    avg_price
                );
                self.on_fill(odno, qty, avg_price).await?;
            }
        }

        if pending
            .iter()
            .any(|(_, symbol)| !is_domestic_symbol(symbol))
        {
            let executed = client.get_today_overseas_executed_orders().await?;
            for (odno, symbol) in pending
                .iter()
                .filter(|(_, symbol)| !is_domestic_symbol(symbol))
            {
                let Some(order) = executed.iter().find(|o| o.odno == *odno) else {
                    continue;
                };
                let qty = order.filled_qty();
                if qty == 0 {
                    continue;
                }
                let avg_price_cents = order.avg_price_cents();
                if avg_price_cents == 0 {
                    continue;
                }
                tracing::info!(
                    "해외 주문번호 기반 체결 확인: odno={} symbol={} qty={} avg_cents={}",
                    odno,
                    symbol,
                    qty,
                    avg_price_cents
                );
                self.on_fill(odno, qty, avg_price_cents).await?;
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────
// 잔고 부족 에러 감지 헬퍼
// ────────────────────────────────────────────────────────────────────

/// 국내주식 매매 수수료 추정
///
/// # 구성 (2024~2025년 기준)
/// - 위탁수수료: 0.015% (매수·매도 모두)
/// - 증권거래세: 0.20% (매도 시에만, 코스피/코스닥 모두 동일 적용)
///
/// KIS API(`TTTC8001R`)는 체결 건별(output1) 수수료를 제공하지 않으며
/// output2 합산(`prsm_tlex_smtl`) 에만 전체 기간 추정제비용이 있다.
/// 따라서 체결 시 표준 수수료율로 추정한 값을 로컬에 기록한다.
fn calculate_domestic_fee(price: u64, quantity: u64, is_sell: bool) -> u64 {
    let total = price * quantity;
    // 위탁수수료 0.015% = 3/20000
    let commission = (total as f64 * 0.00015) as u64;
    // 증권거래세 0.20% = 1/500 (매도 시에만)
    let transaction_tax = if is_sell {
        (total as f64 * 0.002) as u64
    } else {
        0
    };
    commission + transaction_tax
}

/// 해외주식 매매 수수료 추정.
///
/// KIS 해외 잔고/체결 API는 건별 수수료를 제공하지 않으므로 자동매매 guard의
/// 기본 해외 비용 추정치와 같은 10bps(0.10%)를 사용한다. 금액 단위는 USD cents.
fn calculate_overseas_fee_cents(price_cents: u64, quantity: u64) -> u64 {
    let total_cents = price_cents.saturating_mul(quantity);
    ((total_cents as f64) * 0.001).ceil() as u64
}

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

fn same_order_side(left: &OrderSide, right: &OrderSide) -> bool {
    matches!(
        (left, right),
        (OrderSide::Buy, OrderSide::Buy) | (OrderSide::Sell, OrderSide::Sell)
    )
}

fn pending_order_provider(pending: &PendingOrder) -> BrokerId {
    match pending.record.provider.as_deref() {
        Some("toss") => BrokerId::Toss,
        _ => BrokerId::Kis,
    }
}

fn order_side_label(side: &OrderSide) -> &'static str {
    match side {
        OrderSide::Buy => "매수",
        OrderSide::Sell => "매도",
    }
}

fn pending_order_conflict_reason(pending: &PendingOrder, requested_side: &OrderSide) -> String {
    let pending_side = order_side_label(&pending.record.side);
    let requested_label = order_side_label(requested_side);
    let odno = pending.record.kis_order_id.as_deref().unwrap_or("unknown");

    if same_order_side(&pending.record.side, requested_side) {
        format!(
            "{} {}주 {} 미체결 주문 이미 존재 (odno: {})",
            pending.record.symbol, pending.record.quantity, pending_side, odno
        )
    } else {
        format!(
            "{} {}주 {} 미체결 주문 존재 — 요청 {} 차단 (odno: {})",
            pending.record.symbol, pending.record.quantity, pending_side, requested_label, odno
        )
    }
}

fn pending_conflict_reason_for_scope<'a>(
    mut pending_orders: impl Iterator<Item = &'a PendingOrder>,
    broker_scope: &BrokerScope,
    symbol: &str,
    requested_side: &OrderSide,
) -> Option<String> {
    pending_orders
        .find(|pending| pending.broker_scope == *broker_scope && pending.record.symbol == symbol)
        .map(|pending| pending_order_conflict_reason(pending, requested_side))
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
mod tests {
    use super::*;
    use crate::broker::{BrokerAccountId, BrokerId};

    fn scope(broker_id: BrokerId, account_id: &str) -> BrokerScope {
        BrokerScope::new(broker_id, Some(BrokerAccountId(account_id.to_string())))
    }

    fn pending_order(side: OrderSide, broker_scope: BrokerScope) -> PendingOrder {
        let mut record = OrderRecord::new(
            "005930".to_string(),
            "삼성전자".to_string(),
            side,
            3,
            0,
            "Market".to_string(),
        );
        record.kis_order_id = Some("ODNO-1".to_string());

        PendingOrder {
            record,
            signal_reason: "test".to_string(),
            strategy_id: Some("strategy".to_string()),
            signal_price: 75_000,
            order_price: 0,
            exchange: None,
            broker_scope,
            filled_quantity: 0,
        }
    }

    #[test]
    fn pending_conflict_blocks_opposite_side_in_same_scope() {
        let broker_scope = scope(BrokerId::Kis, "kis-1");
        let pending = vec![pending_order(OrderSide::Buy, broker_scope.clone())];
        let reason = pending_conflict_reason_for_scope(
            pending.iter(),
            &broker_scope,
            "005930",
            &OrderSide::Sell,
        )
        .unwrap();

        assert!(reason.contains("매수 미체결 주문 존재"));
        assert!(reason.contains("요청 매도 차단"));
        assert!(reason.contains("ODNO-1"));
    }

    #[test]
    fn pending_conflict_blocks_same_side_in_same_scope() {
        let broker_scope = scope(BrokerId::Kis, "kis-1");
        let pending = vec![pending_order(OrderSide::Sell, broker_scope.clone())];
        let reason = pending_conflict_reason_for_scope(
            pending.iter(),
            &broker_scope,
            "005930",
            &OrderSide::Sell,
        )
        .unwrap();

        assert!(reason.contains("매도 미체결 주문 이미 존재"));
        assert!(reason.contains("ODNO-1"));
    }

    #[test]
    fn pending_conflict_scan_ignores_different_scope() {
        let mut pending = HashMap::new();
        pending.insert(
            "ODNO-1".to_string(),
            pending_order(OrderSide::Buy, scope(BrokerId::Kis, "kis-1")),
        );

        let requested_scope = scope(BrokerId::Toss, "toss-1");
        let conflict = pending_conflict_reason_for_scope(
            pending.values(),
            &requested_scope,
            "005930",
            &OrderSide::Sell,
        );

        assert!(conflict.is_none());
    }

    #[test]
    fn pending_order_provider_uses_provider_trace() {
        let mut kis_pending = pending_order(OrderSide::Buy, scope(BrokerId::Kis, "kis-1"));
        assert_eq!(pending_order_provider(&kis_pending), BrokerId::Kis);

        kis_pending.record.provider = Some("toss".to_string());
        assert_eq!(pending_order_provider(&kis_pending), BrokerId::Toss);
    }
}
