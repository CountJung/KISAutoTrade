use super::*;

#[derive(Debug, Clone)]
struct SubmissionReservation {
    broker_scope: BrokerScope,
    symbol: String,
    side: OrderSide,
}

#[derive(Debug, Clone)]
struct OrderSubmission {
    strategy_id: Option<String>,
    signal: Signal,
    symbol: String,
    symbol_name: String,
    side: OrderSide,
    quantity: u64,
    reason: String,
    total_balance: i64,
    exchange: Option<String>,
    tick_price: u64,
    broker_scope: BrokerScope,
}

struct PreparedOrderSubmission {
    submission: OrderSubmission,
    quantity: u64,
    order_type: &'static str,
    order_price: u64,
    provider_request: ProviderOrderRequest,
    order_exchange: Option<String>,
}

enum ProviderOrderRequest {
    Domestic(OrderRequest),
    Overseas(OverseasOrderRequest),
}

struct SubmissionDeps {
    broker_scope: BrokerScope,
    rest_client: Arc<RwLock<Arc<KisRestClient>>>,
    order_store: Arc<OrderStore>,
    position_tracker: Arc<Mutex<PositionTracker>>,
    overseas_position_tracker: Arc<Mutex<OverseasPositionTracker>>,
    risk_manager: Arc<Mutex<RiskManager>>,
    exchange_rate_krw: Arc<RwLock<f64>>,
}

enum PrepareDecision {
    Submit(Box<PreparedOrderSubmission>),
    Skip,
}

enum ReservationDecision {
    Reserved(SubmissionReservation),
    Skip,
}

impl PendingOrder {
    fn signal(&self) -> Signal {
        match self.record.side {
            OrderSide::Buy => Signal::Buy {
                symbol: self.record.symbol.clone(),
                quantity: self.record.quantity,
                reason: self.signal_reason.clone(),
            },
            OrderSide::Sell => Signal::Sell {
                symbol: self.record.symbol.clone(),
                quantity: self.record.quantity,
                reason: self.signal_reason.clone(),
            },
        }
    }
}

impl OrderSubmission {
    fn from_signal(
        strategy_id: Option<String>,
        signal: Signal,
        symbol_name: String,
        total_balance: i64,
        exchange: Option<String>,
        tick_price: u64,
    ) -> Option<Self> {
        match &signal {
            Signal::Buy {
                symbol,
                quantity,
                reason,
            } => Some(Self {
                strategy_id,
                signal: signal.clone(),
                symbol: symbol.clone(),
                symbol_name,
                side: OrderSide::Buy,
                quantity: *quantity,
                reason: reason.clone(),
                total_balance,
                exchange,
                tick_price,
                broker_scope: BrokerScope::kis_legacy(),
            }),
            Signal::Sell {
                symbol,
                quantity,
                reason,
            } => Some(Self {
                strategy_id,
                signal: signal.clone(),
                symbol: symbol.clone(),
                symbol_name,
                side: OrderSide::Sell,
                quantity: *quantity,
                reason: reason.clone(),
                total_balance,
                exchange,
                tick_price,
                broker_scope: BrokerScope::kis_legacy(),
            }),
            Signal::Hold => None,
        }
    }
}

impl OrderManager {
    /// 전략 신호 처리 → 주문 실행.
    ///
    /// `OrderManager` mutex를 잡은 채 provider 주문 API와 파일 저장 await를 수행하지 않는 shared 경로다.
    /// lock이 필요한 구간은 guard/pending 예약과 성공/실패 상태 반영으로 제한한다.
    pub async fn submit_signal_shared(
        order_manager: &Arc<Mutex<Self>>,
        strategy_id: Option<String>,
        signal: Signal,
        symbol_name: String,
        total_balance: i64,
        exchange: Option<String>,
        tick_price: u64,
    ) -> Result<()> {
        let Some(mut submission) = OrderSubmission::from_signal(
            strategy_id,
            signal,
            symbol_name,
            total_balance,
            exchange,
            tick_price,
        ) else {
            return Ok(());
        };

        let deps = {
            let manager = order_manager.lock().await;
            SubmissionDeps {
                broker_scope: manager.execution_scope.clone(),
                rest_client: Arc::clone(&manager.rest_client),
                order_store: Arc::clone(&manager.order_store),
                position_tracker: Arc::clone(&manager.position_tracker),
                overseas_position_tracker: Arc::clone(&manager.overseas_position_tracker),
                risk_manager: Arc::clone(&manager.risk_manager),
                exchange_rate_krw: Arc::clone(&manager.exchange_rate_krw),
            }
        };
        submission.broker_scope = deps.broker_scope.clone();

        let (held_quantity, avg_price) = current_position_snapshot(
            &submission,
            &deps.position_tracker,
            &deps.overseas_position_tracker,
        )
        .await;
        let prepared = match prepare_order_submission(&submission, held_quantity, &deps).await {
            PrepareDecision::Submit(prepared) => *prepared,
            PrepareDecision::Skip => return Ok(()),
        };

        let reservation = {
            let mut manager = order_manager.lock().await;
            match manager.reserve_submission(&prepared.submission, held_quantity, avg_price) {
                ReservationDecision::Reserved(reservation) => reservation,
                ReservationDecision::Skip => return Ok(()),
            }
        };

        let order_result = place_prepared_order(&deps.rest_client, &prepared).await;
        match order_result {
            Ok(response) => {
                let pending = build_pending_order(&prepared, response);
                let record = pending.record.clone();
                let guard_signal = pending.signal();
                {
                    let mut manager = order_manager.lock().await;
                    manager.finish_submission_success(reservation, pending, &guard_signal);
                }
                if let Err(e) = deps.order_store.append(record.clone()).await {
                    tracing::error!("주문 기록 저장 실패 (Pending): {}", e);
                }
                deps.risk_manager
                    .lock()
                    .await
                    .record_order_submitted_for_scope(
                        &prepared.submission.broker_scope,
                        prepared
                            .submission
                            .strategy_id
                            .as_deref()
                            .unwrap_or("unknown"),
                        &prepared.submission.symbol,
                        match prepared.submission.side {
                            OrderSide::Buy => DailyOrderSide::Buy,
                            OrderSide::Sell => DailyOrderSide::Sell,
                        },
                    );
                Ok(())
            }
            Err(e) => {
                let msg = e.to_string();
                let insufficient_balance = matches!(prepared.submission.side, OrderSide::Buy)
                    && is_insufficient_balance_error(&msg);
                let paper_unsupported = matches!(prepared.submission.side, OrderSide::Sell)
                    && prepared.submission.exchange.is_some()
                    && is_paper_unsupported_error(&msg);
                {
                    let mut manager = order_manager.lock().await;
                    manager.finish_submission_failure(&reservation, insufficient_balance, &msg);
                }
                append_failed_order(
                    &deps.order_store,
                    &prepared.submission,
                    prepared.quantity,
                    failed_order_price(&prepared),
                    prepared.order_type,
                    msg.clone(),
                )
                .await;
                if insufficient_balance {
                    tracing::warn!(
                        "잔고 부족 감지 — 매수 주문 정지: {} (매도 체결 또는 수동 해제 시 재개) | {}",
                        prepared.submission.symbol,
                        msg
                    );
                    Ok(())
                } else if paper_unsupported {
                    tracing::warn!(
                        "모의투자 매도 미지원 — 스킵: {} ({}) | {}",
                        prepared.submission.symbol,
                        prepared.order_exchange.as_deref().unwrap_or("-"),
                        msg
                    );
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    fn submitting_conflict_reason_for_scope(
        &self,
        broker_scope: &BrokerScope,
        symbol: &str,
        requested_side: &OrderSide,
    ) -> Option<String> {
        self.submitting
            .get(&(broker_scope.clone(), symbol.to_string()))
            .map(|side| submitting_conflict_reason(symbol, side, requested_side))
    }

    fn reserve_submission(
        &mut self,
        submission: &OrderSubmission,
        held_quantity: u64,
        avg_price: Option<u64>,
    ) -> ReservationDecision {
        if matches!(submission.side, OrderSide::Buy) && self.buy_suspended {
            tracing::debug!(
                "매수 스킵 — 잔고 부족 정지 중: {} (사유: {})",
                submission.symbol,
                self.buy_suspended_reason.as_deref().unwrap_or("알 수 없음")
            );
            return ReservationDecision::Skip;
        }

        match self.trade_guard.evaluate_for_scope(
            &submission.broker_scope,
            &submission.signal,
            held_quantity,
            avg_price,
            submission.tick_price,
            submission.exchange.is_some(),
        ) {
            GuardDecision::Allow => {}
            GuardDecision::Block { reason } => {
                tracing::info!("TradeGuard 차단 — {}", reason);
                return ReservationDecision::Skip;
            }
        }

        if let Some(reason) = self.pending_conflict_reason_for_scope(
            &submission.broker_scope,
            &submission.symbol,
            &submission.side,
        ) {
            tracing::info!("주문 스킵 — {}", reason);
            return ReservationDecision::Skip;
        }
        if let Some(reason) = self.submitting_conflict_reason_for_scope(
            &submission.broker_scope,
            &submission.symbol,
            &submission.side,
        ) {
            tracing::info!("주문 스킵 — {}", reason);
            return ReservationDecision::Skip;
        }

        let reservation = SubmissionReservation {
            broker_scope: submission.broker_scope.clone(),
            symbol: submission.symbol.clone(),
            side: submission.side.clone(),
        };
        self.submitting.insert(
            (reservation.broker_scope.clone(), reservation.symbol.clone()),
            reservation.side.clone(),
        );
        ReservationDecision::Reserved(reservation)
    }

    fn finish_submission_success(
        &mut self,
        reservation: SubmissionReservation,
        pending: PendingOrder,
        guard_signal: &Signal,
    ) {
        self.submitting
            .remove(&(reservation.broker_scope.clone(), reservation.symbol.clone()));
        let odno = pending.record.kis_order_id.clone().unwrap_or_default();
        self.symbol_to_odno
            .insert(pending.record.symbol.clone(), odno.clone());
        self.trade_guard
            .record_submitted_for_scope(&pending.broker_scope, guard_signal);
        self.pending.insert(odno, pending);
    }

    fn finish_submission_failure(
        &mut self,
        reservation: &SubmissionReservation,
        insufficient_balance: bool,
        message: &str,
    ) {
        self.submitting
            .remove(&(reservation.broker_scope.clone(), reservation.symbol.clone()));
        if insufficient_balance {
            self.buy_suspended = true;
            self.buy_suspended_reason = Some(message.to_string());
        }
    }
}

async fn current_position_snapshot(
    submission: &OrderSubmission,
    position_tracker: &Arc<Mutex<PositionTracker>>,
    overseas_position_tracker: &Arc<Mutex<OverseasPositionTracker>>,
) -> (u64, Option<u64>) {
    if submission.exchange.is_some() {
        let tracker = overseas_position_tracker.lock().await;
        tracker
            .get(&submission.symbol)
            .map(|p| (p.quantity, Some(p.avg_price_cents.round() as u64)))
            .unwrap_or((0, None))
    } else {
        let tracker = position_tracker.lock().await;
        tracker
            .get(&submission.symbol)
            .map(|p| (p.quantity, Some(p.avg_price.round() as u64)))
            .unwrap_or((0, None))
    }
}

async fn prepare_order_submission(
    submission: &OrderSubmission,
    held_quantity: u64,
    deps: &SubmissionDeps,
) -> PrepareDecision {
    if let Some((symbol, side)) = daily_order_limit_key(&submission.signal) {
        let limit_reason = deps
            .risk_manager
            .lock()
            .await
            .daily_order_limit_reason_for_scope(
                &submission.broker_scope,
                submission.strategy_id.as_deref().unwrap_or("unknown"),
                symbol,
                side,
            );
        if let Some(reason) = limit_reason {
            tracing::info!("리스크 주문 횟수 제한 — {} ({})", symbol, reason);
            return PrepareDecision::Skip;
        }
    }

    match submission.side {
        OrderSide::Buy => prepare_buy_submission(submission, deps).await,
        OrderSide::Sell => prepare_sell_submission(submission, held_quantity, deps).await,
    }
}

async fn prepare_buy_submission(
    submission: &OrderSubmission,
    deps: &SubmissionDeps,
) -> PrepareDecision {
    if let Some(reason) = deps
        .risk_manager
        .lock()
        .await
        .consecutive_loss_block_reason_for_scope(
            &submission.broker_scope,
            submission.strategy_id.as_deref().unwrap_or("unknown"),
            &submission.symbol,
        )
    {
        tracing::info!("연속 손실 진입 차단 — {} ({})", submission.symbol, reason);
        return PrepareDecision::Skip;
    }

    let is_overseas = submission.exchange.is_some();
    let exchange_rate = if is_overseas {
        *deps.exchange_rate_krw.read().await
    } else {
        1.0
    };
    let quantity = {
        let risk = deps.risk_manager.lock().await;
        if !risk.can_trade() {
            tracing::warn!(
                "리스크 한도 초과 — 매수 거부: {} (비상정지 or 손실한도)",
                submission.symbol
            );
            return PrepareDecision::Skip;
        }
        let adjusted_quantity = risk.volatility_adjusted_quantity(
            &submission.symbol,
            submission.quantity,
            submission.tick_price,
            submission.total_balance,
            is_overseas,
            exchange_rate,
        );
        if adjusted_quantity == 0 {
            return PrepareDecision::Skip;
        }
        if submission.total_balance > 0 {
            let est_amount = estimate_order_amount_krw(
                submission.tick_price,
                adjusted_quantity,
                is_overseas,
                exchange_rate,
            );
            if !risk.check_position_size(est_amount, submission.total_balance) {
                tracing::warn!(
                    "포지션 비중 초과 — 매수 거부: {} (추정 {}원 / 총잔고 {}원)",
                    submission.symbol,
                    est_amount,
                    submission.total_balance
                );
                return PrepareDecision::Skip;
            }
        }
        adjusted_quantity
    };

    let (provider_request, order_type, order_price, order_exchange) =
        build_provider_request(submission, quantity);
    PrepareDecision::Submit(Box::new(PreparedOrderSubmission {
        submission: submission.clone(),
        quantity,
        order_type,
        order_price,
        provider_request,
        order_exchange,
    }))
}

async fn prepare_sell_submission(
    submission: &OrderSubmission,
    held_quantity: u64,
    deps: &SubmissionDeps,
) -> PrepareDecision {
    if held_quantity == 0 {
        tracing::debug!("매도 스킵 — {} 보유 포지션 없음", submission.symbol);
        return PrepareDecision::Skip;
    }
    if deps.risk_manager.lock().await.is_emergency_stop() {
        tracing::warn!("비상정지 상태 — 매도 거부: {}", submission.symbol);
        return PrepareDecision::Skip;
    }
    let quantity = submission.quantity.min(held_quantity);
    let (provider_request, order_type, order_price, order_exchange) =
        build_provider_request(submission, quantity);
    PrepareDecision::Submit(Box::new(PreparedOrderSubmission {
        submission: submission.clone(),
        quantity,
        order_type,
        order_price,
        provider_request,
        order_exchange,
    }))
}

fn build_provider_request(
    submission: &OrderSubmission,
    quantity: u64,
) -> (ProviderOrderRequest, &'static str, u64, Option<String>) {
    if let Some(exchange) = &submission.exchange {
        let order_exch = order_exchange_code(exchange).to_string();
        let usd_price = submission.tick_price as f64 / 100.0;
        let req = OverseasOrderRequest {
            symbol: submission.symbol.clone(),
            exchange: order_exch.clone(),
            side: rest_order_side(&submission.side),
            quantity,
            price: usd_price,
        };
        (
            ProviderOrderRequest::Overseas(req),
            "Limit",
            submission.tick_price,
            Some(order_exch),
        )
    } else {
        let req = OrderRequest {
            symbol: submission.symbol.clone(),
            side: rest_order_side(&submission.side),
            order_type: OrderType::Market,
            quantity,
            price: 0,
        };
        (ProviderOrderRequest::Domestic(req), "Market", 0, None)
    }
}

fn rest_order_side(side: &OrderSide) -> RestOrderSide {
    match side {
        OrderSide::Buy => RestOrderSide::Buy,
        OrderSide::Sell => RestOrderSide::Sell,
    }
}

async fn place_prepared_order(
    rest_client: &Arc<RwLock<Arc<KisRestClient>>>,
    prepared: &PreparedOrderSubmission,
) -> Result<OrderResponse> {
    match &prepared.provider_request {
        ProviderOrderRequest::Domestic(req) => {
            tracing::info!(
                "{} 주문 시도: {} {}주 — {}",
                order_side_label(&prepared.submission.side),
                prepared.submission.symbol,
                prepared.quantity,
                prepared.submission.reason
            );
            place_with_retry_shared(rest_client, req).await
        }
        ProviderOrderRequest::Overseas(req) => {
            tracing::info!(
                "해외 {} 주문 시도: {} {}주 @ ${:.2} ({}) — {}",
                order_side_label(&prepared.submission.side),
                prepared.submission.symbol,
                prepared.quantity,
                prepared.submission.tick_price as f64 / 100.0,
                req.exchange,
                prepared.submission.reason
            );
            place_overseas_with_retry_shared(rest_client, req).await
        }
    }
}

fn build_pending_order(
    prepared: &PreparedOrderSubmission,
    response: OrderResponse,
) -> PendingOrder {
    tracing::info!(
        "{} 주문 접수: {} {}주 — {} (provider=kis tr_id={} odno={})",
        order_side_label(&prepared.submission.side),
        prepared.submission.symbol,
        prepared.quantity,
        prepared.submission.reason,
        response.tr_id,
        response.odno
    );

    let mut record = OrderRecord::new(
        prepared.submission.symbol.clone(),
        prepared.submission.symbol_name.clone(),
        prepared.submission.side.clone(),
        prepared.quantity,
        prepared.order_price,
        prepared.order_type.to_string(),
    )
    .with_provider_trace(
        "kis",
        Some(response.odno.clone()),
        None,
        Some(response.tr_id.clone()),
    );
    let odno = if response.odno.is_empty() {
        format!("LOCAL-{}", uuid::Uuid::new_v4())
    } else {
        response.odno
    };
    record.kis_order_id = Some(odno.clone());
    record.provider_order_id = Some(odno);

    tracing::info!(
        "주문 trace 저장: provider={} tr_id={} order_id={}",
        record.provider.as_deref().unwrap_or("unknown"),
        record.provider_tr_id.as_deref().unwrap_or("-"),
        record.provider_order_id.as_deref().unwrap_or("-")
    );

    PendingOrder {
        record,
        signal_reason: prepared.submission.reason.clone(),
        exchange: prepared.order_exchange.clone(),
        filled_quantity: 0,
        strategy_id: prepared.submission.strategy_id.clone(),
        signal_price: prepared.submission.tick_price,
        order_price: prepared.order_price,
        broker_scope: prepared.submission.broker_scope.clone(),
    }
}

async fn append_failed_order(
    order_store: &Arc<OrderStore>,
    submission: &OrderSubmission,
    quantity: u64,
    price: u64,
    order_type: &str,
    error_message: String,
) {
    let mut record = OrderRecord::new(
        submission.symbol.clone(),
        submission.symbol_name.clone(),
        submission.side.clone(),
        quantity,
        price,
        order_type.to_string(),
    );
    record.status = OrderStatus::Failed;
    record.error_message = Some(error_message);
    if let Err(e) = order_store.append(record).await {
        tracing::error!("주문 기록 저장 실패 (Failed): {}", e);
    }
}

fn failed_order_price(prepared: &PreparedOrderSubmission) -> u64 {
    if matches!(prepared.submission.side, OrderSide::Buy) || prepared.submission.exchange.is_some()
    {
        prepared.submission.tick_price
    } else {
        prepared.order_price
    }
}

async fn place_with_retry_shared(
    rest_client: &Arc<RwLock<Arc<KisRestClient>>>,
    req: &OrderRequest,
) -> Result<OrderResponse> {
    const MAX_RETRIES: u32 = 3;
    let mut last_err = anyhow::anyhow!("주문 최대 재시도 횟수(3회) 초과");

    for attempt in 0..MAX_RETRIES {
        let client = rest_client.read().await.clone();
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

async fn place_overseas_with_retry_shared(
    rest_client: &Arc<RwLock<Arc<KisRestClient>>>,
    req: &OverseasOrderRequest,
) -> Result<OrderResponse> {
    const MAX_RETRIES: u32 = 3;
    let mut last_err = anyhow::anyhow!("해외 주문 최대 재시도 횟수(3회) 초과");

    for attempt in 0..MAX_RETRIES {
        let client = rest_client.read().await.clone();
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

fn order_side_label(side: &OrderSide) -> &'static str {
    match side {
        OrderSide::Buy => "매수",
        OrderSide::Sell => "매도",
    }
}

fn submitting_conflict_reason(
    symbol: &str,
    submitting_side: &OrderSide,
    requested_side: &OrderSide,
) -> String {
    let submitting_label = order_side_label(submitting_side);
    let requested_label = order_side_label(requested_side);
    if matches!(
        (submitting_side, requested_side),
        (OrderSide::Buy, OrderSide::Buy) | (OrderSide::Sell, OrderSide::Sell)
    ) {
        format!("{symbol} {submitting_label} 주문 제출 중 — 중복 {requested_label} 차단")
    } else {
        format!("{symbol} {submitting_label} 주문 제출 중 — 요청 {requested_label} 차단")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_buy_uses_signal_price_even_for_domestic_market_order() {
        let prepared = PreparedOrderSubmission {
            submission: OrderSubmission {
                strategy_id: None,
                signal: Signal::Buy {
                    symbol: "005930".into(),
                    quantity: 1,
                    reason: "test".into(),
                },
                symbol: "005930".into(),
                symbol_name: "삼성전자".into(),
                side: OrderSide::Buy,
                quantity: 1,
                reason: "test".into(),
                total_balance: 0,
                exchange: None,
                tick_price: 72000,
                broker_scope: BrokerScope::kis_legacy(),
            },
            quantity: 1,
            order_type: "Market",
            order_price: 0,
            provider_request: ProviderOrderRequest::Domestic(OrderRequest {
                symbol: "005930".into(),
                side: RestOrderSide::Buy,
                order_type: OrderType::Market,
                quantity: 1,
                price: 0,
            }),
            order_exchange: None,
        };

        assert_eq!(failed_order_price(&prepared), 72000);
    }

    #[test]
    fn submitting_conflict_distinguishes_same_and_opposite_side() {
        assert!(
            submitting_conflict_reason("005930", &OrderSide::Buy, &OrderSide::Buy)
                .contains("중복 매수 차단")
        );
        assert!(
            submitting_conflict_reason("005930", &OrderSide::Buy, &OrderSide::Sell)
                .contains("요청 매도 차단")
        );
    }
}
