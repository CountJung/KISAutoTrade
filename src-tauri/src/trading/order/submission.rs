use super::*;

use crate::{
    broker::{
        toss::TossOrderCreateRequest, BrokerAdapter, BrokerCurrency, BrokerId, BrokerMarket,
        TossBrokerAdapter,
    },
    config::AccountProfile,
};

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
    is_manual: bool,
    requested_order_type: Option<OrderType>,
    requested_price: Option<u64>,
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
    Toss(TossOrderCreateRequest),
}

enum ProviderOrderResponse {
    Kis(OrderResponse),
    Toss {
        order_id: String,
        client_order_id: Option<String>,
    },
}

struct SubmissionDeps {
    broker_scope: BrokerScope,
    rest_client: Arc<RwLock<Arc<KisRestClient>>>,
    active_profile: Option<AccountProfile>,
    order_store: Arc<OrderStore>,
    position_tracker: Arc<Mutex<PositionTracker>>,
    overseas_position_tracker: Arc<Mutex<OverseasPositionTracker>>,
    risk_manager: Arc<Mutex<RiskManager>>,
    risk_store: Arc<RiskStore>,
    exchange_rate_krw: Arc<RwLock<f64>>,
}

enum PrepareDecision {
    Submit(Box<PreparedOrderSubmission>),
    Skip(String),
}

enum ReservationDecision {
    Reserved(SubmissionReservation),
    Skip(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionOutcome {
    Submitted { provider_order_id: String },
    Skipped { reason: String },
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
                is_manual: false,
                requested_order_type: None,
                requested_price: None,
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
                is_manual: false,
                requested_order_type: None,
                requested_price: None,
            }),
            Signal::Hold => None,
        }
    }
}

impl OrderManager {
    #[allow(clippy::too_many_arguments)]
    pub async fn submit_manual_order_shared(
        order_manager: &Arc<Mutex<Self>>,
        symbol: String,
        symbol_name: String,
        side: RestOrderSide,
        order_type: OrderType,
        quantity: u64,
        requested_price: u64,
        quote_price: u64,
        total_balance: i64,
        exchange: Option<String>,
        broker_scope: BrokerScope,
    ) -> Result<SubmissionOutcome> {
        let signal = match side {
            RestOrderSide::Buy => Signal::Buy {
                symbol: symbol.clone(),
                quantity,
                reason: "수동 주문".to_string(),
            },
            RestOrderSide::Sell => Signal::Sell {
                symbol: symbol.clone(),
                quantity,
                reason: "수동 주문".to_string(),
            },
        };
        let mut submission = OrderSubmission::from_signal(
            None,
            signal,
            symbol_name,
            total_balance,
            exchange,
            quote_price,
        )
        .expect("manual buy/sell always creates a submission");
        submission.is_manual = true;
        submission.requested_order_type = Some(order_type);
        submission.requested_price = (order_type == OrderType::Limit).then_some(requested_price);
        Self::submit_order_shared(order_manager, submission, Some(broker_scope)).await
    }

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
    ) -> Result<SubmissionOutcome> {
        let Some(submission) = OrderSubmission::from_signal(
            strategy_id,
            signal,
            symbol_name,
            total_balance,
            exchange,
            tick_price,
        ) else {
            return Ok(SubmissionOutcome::Skipped {
                reason: "hold signal".to_string(),
            });
        };

        Self::submit_order_shared(order_manager, submission, None).await
    }

    async fn submit_order_shared(
        order_manager: &Arc<Mutex<Self>>,
        mut submission: OrderSubmission,
        scope_override: Option<BrokerScope>,
    ) -> Result<SubmissionOutcome> {
        let deps = {
            let manager = order_manager.lock().await;
            let broker_scope = scope_override
                .clone()
                .unwrap_or_else(|| manager.execution_scope.clone());
            SubmissionDeps {
                broker_scope: broker_scope.clone(),
                rest_client: Arc::clone(&manager.rest_client),
                active_profile: {
                    let profiles = manager.profiles.read().await;
                    let account_id = broker_scope.account_id.as_ref().map(|id| id.0.as_str());
                    profiles
                        .profiles
                        .iter()
                        .find(|profile| {
                            profile.broker_id == broker_scope.broker_id
                                && account_id
                                    .map(|id| profile.broker_account_id() == id)
                                    .unwrap_or_else(|| {
                                        profiles.get_active().map(|p| p.id.as_str())
                                            == Some(profile.id.as_str())
                                    })
                        })
                        .cloned()
                },
                order_store: Arc::clone(&manager.order_store),
                position_tracker: Arc::clone(&manager.position_tracker),
                overseas_position_tracker: Arc::clone(&manager.overseas_position_tracker),
                risk_manager: Arc::clone(&manager.risk_manager),
                risk_store: Arc::clone(&manager.risk_store),
                exchange_rate_krw: Arc::clone(&manager.exchange_rate_krw),
            }
        };
        submission.broker_scope = deps.broker_scope.clone();

        if submission.is_manual {
            refresh_manual_positions(&submission, &deps).await?;
        }

        let (held_quantity, avg_price) = current_position_snapshot(
            &submission,
            &deps.position_tracker,
            &deps.overseas_position_tracker,
        )
        .await;
        let prepared = match prepare_order_submission(&submission, held_quantity, &deps).await {
            PrepareDecision::Submit(prepared) => *prepared,
            PrepareDecision::Skip(reason) => return Ok(SubmissionOutcome::Skipped { reason }),
        };

        let reservation = {
            let mut manager = order_manager.lock().await;
            match manager.reserve_submission(&prepared.submission, held_quantity, avg_price) {
                ReservationDecision::Reserved(reservation) => reservation,
                ReservationDecision::Skip(reason) => {
                    return Ok(SubmissionOutcome::Skipped { reason })
                }
            }
        };

        let order_result = place_prepared_order(&deps, &prepared).await;
        match order_result {
            Ok(response) => {
                let pending = build_pending_order(&prepared, response);
                let record = pending.record.clone();
                let provider_order_id = record
                    .provider_order_id
                    .clone()
                    .or(record.kis_order_id.clone())
                    .unwrap_or_default();
                let guard_signal = pending.signal();
                {
                    let mut manager = order_manager.lock().await;
                    manager.finish_submission_success(reservation, pending, &guard_signal);
                    if let Err(e) = manager.persist_pending_orders().await {
                        tracing::error!("미체결 주문 스냅샷 저장 실패: {}", e);
                        manager.persistence_blocked = true;
                        manager.buy_suspended = true;
                        manager.buy_suspended_reason =
                            Some(format!("미체결 주문 영속화 실패: {e}"));
                        return Err(anyhow::anyhow!(
                            "provider 주문 {provider_order_id} 접수 후 미체결 스냅샷 저장 실패: {e}. 모든 신규 주문을 차단했습니다."
                        ));
                    }
                }
                if let Err(e) = deps.order_store.append(record.clone()).await {
                    tracing::error!("주문 기록 저장 실패 (Pending): {}", e);
                    let reason = format!("주문 영속화 실패로 모든 신규 주문을 중단했습니다: {e}");
                    let mut manager = order_manager.lock().await;
                    manager.persistence_blocked = true;
                    manager.buy_suspended = true;
                    manager.buy_suspended_reason = Some(reason);
                    return Err(anyhow::anyhow!(
                        "provider 주문 {provider_order_id} 접수 후 주문 기록 저장 실패: {e}. 모든 신규 주문을 차단했습니다."
                    ));
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
                {
                    let snapshot = deps.risk_manager.lock().await.runtime_state();
                    if let Err(e) = deps.risk_store.save_runtime(&snapshot).await {
                        tracing::error!("리스크 runtime 상태 저장 실패: {}", e);
                        order_manager.lock().await.block_for_persistence_failure(
                            format!("리스크 runtime 상태 저장 실패: {e}"),
                        );
                    }
                }
                Ok(SubmissionOutcome::Submitted { provider_order_id })
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
                    Ok(SubmissionOutcome::Skipped { reason: msg })
                } else if paper_unsupported {
                    tracing::warn!(
                        "모의투자 매도 미지원 — 스킵: {} ({}) | {}",
                        prepared.submission.symbol,
                        prepared.order_exchange.as_deref().unwrap_or("-"),
                        msg
                    );
                    Ok(SubmissionOutcome::Skipped { reason: msg })
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
        if self.persistence_blocked {
            return ReservationDecision::Skip(
                self.buy_suspended_reason.clone().unwrap_or_else(|| {
                    "주문 영속화 장애로 모든 신규 주문이 차단되었습니다.".into()
                }),
            );
        }
        if matches!(submission.side, OrderSide::Buy) && self.buy_suspended {
            tracing::debug!(
                "매수 스킵 — 잔고 부족 정지 중: {} (사유: {})",
                submission.symbol,
                self.buy_suspended_reason.as_deref().unwrap_or("알 수 없음")
            );
            return ReservationDecision::Skip(
                self.buy_suspended_reason
                    .clone()
                    .unwrap_or_else(|| "신규 매수 정지 중".to_string()),
            );
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
                return ReservationDecision::Skip(reason);
            }
        }

        if let Some(reason) = self.pending_conflict_reason_for_scope(
            &submission.broker_scope,
            &submission.symbol,
            &submission.side,
        ) {
            tracing::info!("주문 스킵 — {}", reason);
            return ReservationDecision::Skip(reason);
        }
        if let Some(reason) = self.submitting_conflict_reason_for_scope(
            &submission.broker_scope,
            &submission.symbol,
            &submission.side,
        ) {
            tracing::info!("주문 스킵 — {}", reason);
            return ReservationDecision::Skip(reason);
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

async fn refresh_manual_positions(
    submission: &OrderSubmission,
    deps: &SubmissionDeps,
) -> Result<()> {
    if submission.broker_scope.broker_id == BrokerId::Toss {
        let profile = deps
            .active_profile
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("활성 Toss 프로파일이 없습니다."))?;
        let account_seq = profile.broker_account_id();
        let adapter = TossBrokerAdapter::with_credentials(
            TossBrokerAdapter::DEFAULT_BASE_URL,
            profile.app_key.clone(),
            profile.app_secret.clone(),
            Some(account_seq.clone()),
        );
        let holdings = adapter
            .list_holdings(submission.broker_scope.account_id.as_ref())
            .await
            .map_err(|error| anyhow::anyhow!("Toss 수동 주문 전 holdings 조회 실패: {error}"))?;
        let mut domestic = Vec::new();
        let mut overseas = Vec::new();
        for holding in holdings {
            let quantity = holding
                .quantity
                .0
                .trim()
                .replace(',', "")
                .parse::<f64>()
                .map_err(|_| anyhow::anyhow!("{} Toss 보유수량 형식 오류", holding.symbol.0))?;
            let quantity = if quantity <= 0.0 {
                0
            } else {
                quantity.floor().max(1.0) as u64
            };
            let money_units = |amount: &str, currency: BrokerCurrency| -> Result<u64> {
                let value = amount
                    .trim()
                    .replace(',', "")
                    .parse::<f64>()
                    .map_err(|_| anyhow::anyhow!("{} Toss 가격 형식 오류", holding.symbol.0))?;
                Ok(match currency {
                    BrokerCurrency::Krw => value.max(0.0).round() as u64,
                    BrokerCurrency::Usd => (value.max(0.0) * 100.0).round() as u64,
                })
            };
            let avg = money_units(
                &holding.average_price.amount,
                holding.average_price.currency,
            )?;
            let current = money_units(
                &holding.current_price.amount,
                holding.current_price.currency,
            )?;
            match holding.market {
                BrokerMarket::Kr => domestic.push((
                    holding.symbol.0,
                    holding.symbol_name,
                    quantity,
                    avg,
                    current,
                )),
                BrokerMarket::Us => overseas.push((
                    holding.symbol.0,
                    holding.symbol_name,
                    "TOSS_US".to_string(),
                    quantity,
                    avg,
                    current,
                )),
            }
        }
        deps.position_tracker.lock().await.replace(domestic);
        deps.overseas_position_tracker
            .lock()
            .await
            .replace(overseas);
        return Ok(());
    }

    let client = deps.rest_client.read().await.clone();
    if submission.exchange.is_some() {
        let balance = client.get_overseas_balance().await.map_err(|error| {
            anyhow::anyhow!("KIS 해외 수동 주문 전 holdings 조회 실패: {error}")
        })?;
        let mut positions = Vec::new();
        for item in balance.items {
            let quantity = item
                .ovrs_cblc_qty
                .trim()
                .replace(',', "")
                .parse::<u64>()
                .map_err(|_| anyhow::anyhow!("{} 해외 보유수량 형식 오류", item.ovrs_pdno))?;
            let parse_cents = |value: &str| -> Result<u64> {
                value
                    .trim()
                    .replace(',', "")
                    .parse::<f64>()
                    .map(|price| (price.max(0.0) * 100.0).round() as u64)
                    .map_err(Into::into)
            };
            positions.push((
                item.ovrs_pdno,
                item.ovrs_item_name,
                order_exchange_code(&item.ovrs_excg_cd).to_string(),
                quantity,
                parse_cents(&item.pchs_avg_pric)?,
                parse_cents(&item.now_pric2)?,
            ));
        }
        deps.overseas_position_tracker
            .lock()
            .await
            .replace(positions);
    } else {
        let balance = client
            .get_balance()
            .await
            .map_err(|error| anyhow::anyhow!("KIS 수동 주문 전 holdings 조회 실패: {error}"))?;
        let mut positions = Vec::new();
        for item in balance.items {
            let quantity = item.hldg_qty.trim().replace(',', "").parse::<u64>()?;
            let avg = item
                .pchs_avg_pric
                .trim()
                .replace(',', "")
                .parse::<f64>()?
                .max(0.0)
                .round() as u64;
            let current = item.prpr.trim().replace(',', "").parse::<u64>()?;
            positions.push((item.pdno, item.prdt_name, quantity, avg, current));
        }
        deps.position_tracker.lock().await.replace(positions);
    }
    Ok(())
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
            return PrepareDecision::Skip(reason);
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
        return PrepareDecision::Skip(reason);
    }

    if submission.total_balance <= 0 {
        let reason = "총 잔고 동기화 값이 0 이하라 신규 매수를 거부합니다.".to_string();
        tracing::warn!("{} — {}", reason, submission.symbol);
        return PrepareDecision::Skip(reason);
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
            return PrepareDecision::Skip("리스크 한도 또는 비상정지".to_string());
        }
        let adjusted_quantity = if submission.is_manual {
            submission.quantity
        } else {
            risk.volatility_adjusted_quantity(
                &submission.symbol,
                submission.quantity,
                submission.tick_price,
                submission.total_balance,
                is_overseas,
                exchange_rate,
            )
        };
        if adjusted_quantity == 0 {
            return PrepareDecision::Skip("변동성 수량 산정 결과가 0주".to_string());
        }
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
            return PrepareDecision::Skip("단일 종목 최대 비중 초과".to_string());
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
        return PrepareDecision::Skip("보유 포지션 없음".to_string());
    }
    if deps.risk_manager.lock().await.is_emergency_stop() {
        tracing::warn!("비상정지 상태 — 매도 거부: {}", submission.symbol);
        return PrepareDecision::Skip("비상정지 상태".to_string());
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
    if submission.broker_scope.broker_id == BrokerId::Toss {
        let is_limit = submission
            .requested_order_type
            .map(|value| value == OrderType::Limit)
            .unwrap_or_else(|| submission.exchange.is_some());
        let price = submission.requested_price.unwrap_or(submission.tick_price);
        let is_us = submission.exchange.is_some()
            || !crate::market_hours::is_domestic_symbol(&submission.symbol);
        let req = TossOrderCreateRequest {
            client_order_id: None,
            symbol: submission.symbol.clone(),
            side: toss_order_side(&submission.side).to_string(),
            order_type: if is_limit { "LIMIT" } else { "MARKET" }.to_string(),
            time_in_force: Some("DAY".to_string()),
            quantity: Some(quantity.to_string()),
            price: is_limit.then(|| toss_price_string(price, is_us)),
            order_amount: None,
            confirm_high_value_order: Some(false),
        }
        .with_generated_client_order_id();
        return (
            ProviderOrderRequest::Toss(req),
            if is_limit { "Limit" } else { "Market" },
            if is_limit { price } else { 0 },
            submission.exchange.clone().or_else(|| {
                (!crate::market_hours::is_domestic_symbol(&submission.symbol))
                    .then(|| "TOSS_US".to_string())
            }),
        );
    }

    if let Some(exchange) = &submission.exchange {
        let order_exch = order_exchange_code(exchange).to_string();
        let price = submission.requested_price.unwrap_or(submission.tick_price);
        let usd_price = price as f64 / 100.0;
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
            price,
            Some(order_exch),
        )
    } else {
        let order_type = submission.requested_order_type.unwrap_or(OrderType::Market);
        let price = if order_type == OrderType::Limit {
            submission.requested_price.unwrap_or(submission.tick_price)
        } else {
            0
        };
        let req = OrderRequest {
            symbol: submission.symbol.clone(),
            side: rest_order_side(&submission.side),
            order_type,
            quantity,
            price,
        };
        (
            ProviderOrderRequest::Domestic(req),
            if order_type == OrderType::Limit {
                "Limit"
            } else {
                "Market"
            },
            price,
            None,
        )
    }
}

fn toss_order_side(side: &OrderSide) -> &'static str {
    match side {
        OrderSide::Buy => "BUY",
        OrderSide::Sell => "SELL",
    }
}

fn toss_price_string(price_units: u64, is_us: bool) -> String {
    if is_us {
        format!("{:.4}", price_units as f64 / 100.0)
    } else {
        price_units.to_string()
    }
}

fn rest_order_side(side: &OrderSide) -> RestOrderSide {
    match side {
        OrderSide::Buy => RestOrderSide::Buy,
        OrderSide::Sell => RestOrderSide::Sell,
    }
}

async fn place_prepared_order(
    deps: &SubmissionDeps,
    prepared: &PreparedOrderSubmission,
) -> Result<ProviderOrderResponse> {
    match &prepared.provider_request {
        ProviderOrderRequest::Domestic(req) => {
            tracing::info!(
                "{} 주문 시도: {} {}주 — {}",
                order_side_label(&prepared.submission.side),
                prepared.submission.symbol,
                prepared.quantity,
                prepared.submission.reason
            );
            place_with_retry_shared(&deps.rest_client, req)
                .await
                .map(ProviderOrderResponse::Kis)
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
            place_overseas_with_retry_shared(&deps.rest_client, req)
                .await
                .map(ProviderOrderResponse::Kis)
        }
        ProviderOrderRequest::Toss(req) => {
            let profile = deps.active_profile.as_ref().ok_or_else(|| {
                anyhow::anyhow!("활성 Toss 프로파일이 없어 주문을 제출할 수 없습니다.")
            })?;
            if profile.broker_id != BrokerId::Toss {
                return Err(anyhow::anyhow!(
                    "자동매매 실행 scope는 Toss지만 활성 프로파일은 Toss가 아닙니다."
                ));
            }
            if !profile.live_trading_consent {
                return Err(anyhow::anyhow!(
                    "Toss 실거래 동의가 저장되지 않아 자동매매 주문을 차단했습니다."
                ));
            }
            let account_seq = profile.broker_account_id();
            if account_seq.trim().is_empty() {
                return Err(anyhow::anyhow!("Toss accountSeq가 설정되지 않았습니다."));
            }
            tracing::info!(
                "Toss {} 주문 시도: {} {}주 {} — {}",
                order_side_label(&prepared.submission.side),
                prepared.submission.symbol,
                prepared.quantity,
                req.order_type,
                prepared.submission.reason
            );
            let adapter = TossBrokerAdapter::with_credentials(
                TossBrokerAdapter::DEFAULT_BASE_URL,
                profile.app_key.clone(),
                profile.app_secret.clone(),
                Some(account_seq.clone()),
            );
            let response = adapter
                .create_order(Some(&account_seq), req)
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(ProviderOrderResponse::Toss {
                order_id: response.order_id,
                client_order_id: response
                    .client_order_id
                    .or_else(|| req.client_order_id.clone()),
            })
        }
    }
}

fn build_pending_order(
    prepared: &PreparedOrderSubmission,
    response: ProviderOrderResponse,
) -> PendingOrder {
    match response {
        ProviderOrderResponse::Kis(response) => build_kis_pending_order(prepared, response),
        ProviderOrderResponse::Toss {
            order_id,
            client_order_id,
        } => build_toss_pending_order(prepared, order_id, client_order_id),
    }
}

fn build_kis_pending_order(
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
        filled_notional: 0,
        confirmed_filled_quantity: 0,
        confirmed_avg_price: 0,
        application_started: false,
        application_pnl: None,
        strategy_id: prepared.submission.strategy_id.clone(),
        signal_price: prepared.submission.tick_price,
        order_price: prepared.order_price,
        broker_scope: prepared.submission.broker_scope.clone(),
        client_order_id: None,
        provider_status: Some("pending".to_string()),
    }
}

fn build_toss_pending_order(
    prepared: &PreparedOrderSubmission,
    order_id: String,
    client_order_id: Option<String>,
) -> PendingOrder {
    tracing::info!(
        "Toss {} 주문 접수: {} {}주 — {} (order_id={})",
        order_side_label(&prepared.submission.side),
        prepared.submission.symbol,
        prepared.quantity,
        prepared.submission.reason,
        order_id
    );

    let client_order_id_for_snapshot = client_order_id.clone();
    let mut record = OrderRecord::new(
        prepared.submission.symbol.clone(),
        prepared.submission.symbol_name.clone(),
        prepared.submission.side.clone(),
        prepared.quantity,
        prepared.order_price,
        format!("TOSS_{}", prepared.order_type.to_uppercase()),
    )
    .with_provider_trace("toss", Some(order_id.clone()), client_order_id, None);
    record.kis_order_id = Some(order_id.clone());
    record.provider_order_id = Some(order_id.clone());

    PendingOrder {
        record,
        signal_reason: prepared.submission.reason.clone(),
        exchange: prepared.order_exchange.clone(),
        filled_quantity: 0,
        filled_notional: 0,
        confirmed_filled_quantity: 0,
        confirmed_avg_price: 0,
        application_started: false,
        application_pnl: None,
        strategy_id: prepared.submission.strategy_id.clone(),
        signal_price: prepared.submission.tick_price,
        order_price: prepared.order_price,
        broker_scope: prepared.submission.broker_scope.clone(),
        client_order_id: client_order_id_for_snapshot,
        provider_status: Some("pending".to_string()),
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
                is_manual: false,
                requested_order_type: None,
                requested_price: None,
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
