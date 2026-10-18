use super::*;
use crate::trading::strategy::StrategySignal;

/// Optional protection state captured from the exact live strategy at evaluation.
pub struct OwnedStrategySignal {
    pub signal: StrategySignal,
    pub position_metadata: Option<serde_json::Value>,
}

impl From<StrategySignal> for OwnedStrategySignal {
    fn from(signal: StrategySignal) -> Self {
        Self {
            signal,
            position_metadata: None,
        }
    }
}

mod owned;
mod positions;
mod prepare;
mod provider;
mod records;

use positions::{current_position_snapshot, refresh_manual_positions};
use prepare::prepare_order_submission;
use provider::place_prepared_order;
use records::{append_failed_order, build_pending_order, failed_order_price};

use crate::{
    broker::{
        toss::TossOrderCreateRequest, BrokerAdapter, BrokerId, BrokerMarket, TossBrokerAdapter,
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
    expected_run: Option<ExecutionRunContext>,
    budget_id: Option<String>,
    requested_order_type: Option<OrderType>,
    requested_price: Option<u64>,
    ownership_key: Option<super::ownership::OwnershipKey>,
    ownership_generation: Option<String>,
    strategy_position_metadata: Option<serde_json::Value>,
    submitted_cost_basis: Option<u64>,
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
    rest_client: Arc<KisRestClient>,
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
    StaleRun(String),
    Reserved(SubmissionReservation),
    Skip(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionOutcome {
    StaleRun {
        reason: String,
    },
    Submitted {
        provider_order_id: String,
    },
    Skipped {
        reason: String,
        held_quantity: u64,
        avg_price: Option<u64>,
    },
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
                expected_run: None,
                budget_id: None,
                requested_order_type: None,
                requested_price: None,
                ownership_key: None,
                ownership_generation: None,
                strategy_position_metadata: None,
                submitted_cost_basis: None,
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
                expected_run: None,
                budget_id: None,
                requested_order_type: None,
                requested_price: None,
                ownership_key: None,
                ownership_generation: None,
                strategy_position_metadata: None,
                submitted_cost_basis: None,
            }),
            Signal::Hold => None,
        }
    }
}

impl OrderManager {
    /// 전략 raw signal 직후 provider 호출 전의 실제 position tracker snapshot.
    /// provider/network 오류처럼 `SubmissionOutcome`을 만들지 못한 경로에서도 전략 상태를 복원한다.
    pub async fn current_position_snapshot_for_signal_shared(
        order_manager: &Arc<Mutex<Self>>,
        signal: &Signal,
        exchange: Option<String>,
        tick_price: u64,
    ) -> (u64, Option<u64>) {
        order_manager
            .lock()
            .await
            .current_position_snapshot_for_signal(signal, exchange, tick_price)
            .await
    }

    /// Read under the caller's order/run gate so feedback cannot cross a
    /// reservation, fill publication, or profile switch.
    pub(crate) async fn current_position_snapshot_for_signal(
        &self,
        signal: &Signal,
        exchange: Option<String>,
        tick_price: u64,
    ) -> (u64, Option<u64>) {
        let Some(submission) = OrderSubmission::from_signal(
            None,
            signal.clone(),
            String::new(),
            0,
            exchange,
            tick_price,
        ) else {
            return (0, None);
        };
        current_position_snapshot(
            &submission,
            &self.position_tracker,
            &self.overseas_position_tracker,
        )
        .await
    }

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
        expected_run: ExecutionRunContext,
        strategy_signal: impl Into<OwnedStrategySignal> + Send,
        symbol_name: String,
        total_balance: i64,
        exchange: Option<String>,
        tick_price: u64,
    ) -> Result<SubmissionOutcome> {
        if !order_manager
            .lock()
            .await
            .execution_run_matches(&expected_run)
        {
            return Ok(stale_run_outcome());
        }
        let OwnedStrategySignal {
            signal: strategy_signal,
            position_metadata,
        } = strategy_signal.into();
        let Some(mut submission) = OrderSubmission::from_signal(
            Some(strategy_signal.strategy_id),
            strategy_signal.signal,
            symbol_name,
            total_balance,
            exchange,
            tick_price,
        ) else {
            return Ok(SubmissionOutcome::Skipped {
                reason: "hold signal".to_string(),
                held_quantity: 0,
                avg_price: None,
            });
        };

        submission.broker_scope = expected_run.broker_scope.clone();
        submission.strategy_position_metadata = position_metadata;
        submission.expected_run = Some(expected_run);
        Self::submit_order_shared(order_manager, submission, None).await
    }

    async fn submit_order_shared(
        order_manager: &Arc<Mutex<Self>>,
        mut submission: OrderSubmission,
        scope_override: Option<BrokerScope>,
    ) -> Result<SubmissionOutcome> {
        let deps = {
            let manager = order_manager.lock().await;
            if !manager.submission_run_is_current(&submission) {
                return Ok(stale_run_outcome());
            }
            let requested_scope = scope_override.as_ref().unwrap_or(&submission.broker_scope);
            if manager.configuration_blocks_order(requested_scope, &submission.symbol)
                || (submission.is_manual && manager.configuration_blocks_holdings(requested_scope))
            {
                return Ok(SubmissionOutcome::Skipped {
                    reason: "전략 설정을 저장하는 동안 해당 계좌·종목의 주문을 잠시 보류합니다."
                        .into(),
                    held_quantity: 0,
                    avg_price: None,
                });
            }
            let (broker_scope, rest_client, active_profile) =
                if let Some(expected) = &submission.expected_run {
                    if !manager.execution_run_matches(expected) {
                        return Ok(stale_run_outcome());
                    }
                    let run = manager
                        .active_execution_run
                        .as_ref()
                        .expect("matched execution run exists");
                    (
                        expected.broker_scope.clone(),
                        Arc::clone(&run.rest_client),
                        run.profile.clone(),
                    )
                } else {
                    let broker_scope = scope_override
                        .clone()
                        .unwrap_or_else(|| manager.execution_scope.clone());
                    let client = manager.rest_client.read().await.clone();
                    let profiles = manager.profiles.read().await;
                    anyhow::ensure!(
                        manager.execution_scope == broker_scope,
                        "수동 주문 계좌가 현재 실행 scope와 다릅니다."
                    );
                    let profile = profiles.get_active().cloned();
                    anyhow::ensure!(
                        profile.is_some() || broker_scope.broker_id == BrokerId::Kis,
                        "활성 수동 주문 프로파일이 없습니다."
                    );
                    execution_run::validate_snapshot(
                        &broker_scope,
                        profile.as_ref().map(|p| p.id.as_str()),
                        &client,
                        profile.as_ref(),
                    )?;
                    (broker_scope, client, profile)
                };
            SubmissionDeps {
                broker_scope,
                rest_client,
                active_profile,
                order_store: Arc::clone(&manager.order_store),
                position_tracker: Arc::clone(&manager.position_tracker),
                overseas_position_tracker: Arc::clone(&manager.overseas_position_tracker),
                risk_manager: Arc::clone(&manager.risk_manager),
                risk_store: Arc::clone(&manager.risk_store),
                exchange_rate_krw: Arc::clone(&manager.exchange_rate_krw),
            }
        };
        if submission.is_manual {
            submission.broker_scope = deps.broker_scope.clone();
        }

        if submission.is_manual {
            refresh_manual_positions(order_manager, &submission, &deps).await?;
        }

        let (observed_quantity, observed_avg) = current_position_snapshot(
            &submission,
            &deps.position_tracker,
            &deps.overseas_position_tracker,
        )
        .await;
        let usd = submission.exchange.is_some()
            || !crate::market_hours::is_domestic_symbol(&submission.symbol);
        let (held_quantity, avg_price) = {
            let mut manager = order_manager.lock().await;
            if !manager.submission_run_is_current(&submission) {
                return Ok(stale_run_outcome());
            }
            let budget_owned = manager
                .budget_owned(&submission.broker_scope, &submission.symbol, usd)
                .await?;
            owned::prepare_ownership(
                &mut manager,
                &mut submission,
                &deps,
                observed_quantity,
                observed_avg,
                budget_owned,
            )
            .await?
        };
        #[cfg(test)]
        {
            let gate = order_manager.lock().await.preparation_test_gate.clone();
            if let Some(gate) = gate {
                gate.arrived.wait().await;
                gate.resume.wait().await;
            }
        }
        let mut prepared = match prepare_order_submission(&submission, held_quantity, &deps).await {
            PrepareDecision::Submit(prepared) => *prepared,
            PrepareDecision::Skip(reason) => {
                if submission_is_stale(order_manager, &submission).await {
                    return Ok(stale_run_outcome());
                }
                return Ok(SubmissionOutcome::Skipped {
                    reason,
                    held_quantity,
                    avg_price,
                });
            }
        };

        let reservation = {
            let mut manager = order_manager.lock().await;
            if submission.is_manual {
                let profiles = manager.profiles.read().await;
                let current_client = manager.rest_client.read().await;
                if !manager.manual_snapshot_matches(&deps, &profiles, &current_client) {
                    return Ok(stale_run_outcome());
                }
            }
            let (latest_observed, latest_avg) = current_position_snapshot(
                &prepared.submission,
                &deps.position_tracker,
                &deps.overseas_position_tracker,
            )
            .await;
            let budget_owned = manager
                .budget_owned(&submission.broker_scope, &submission.symbol, usd)
                .await?;
            let (latest_held, latest_cost) = owned::prepare_ownership(
                &mut manager,
                &mut prepared.submission,
                &deps,
                latest_observed,
                latest_avg,
                budget_owned,
            )
            .await?;
            if matches!(prepared.submission.side, OrderSide::Sell)
                && prepared.quantity > latest_held
            {
                return Ok(SubmissionOutcome::Skipped {
                    reason: "확인된 전략 소유·매도 가능 수량이 변경되었습니다.".into(),
                    held_quantity: latest_held,
                    avg_price: latest_cost,
                });
            }
            match manager.reserve_submission(&prepared.submission, latest_held, latest_cost) {
                ReservationDecision::Reserved(reservation) => {
                    if !submission.is_manual {
                        match manager
                            .reserve_budget(
                                &submission.broker_scope,
                                &submission.symbol,
                                usd,
                                matches!(submission.side, OrderSide::Buy),
                                prepared.quantity,
                                submission.tick_price,
                            )
                            .await
                        {
                            Ok(id) => prepared.submission.budget_id = Some(id),
                            Err(error) => {
                                manager.finish_submission_failure(
                                    &reservation,
                                    false,
                                    &error.to_string(),
                                );
                                return Err(error);
                            }
                        }
                    }
                    reservation
                }
                ReservationDecision::StaleRun(reason) => {
                    return Ok(SubmissionOutcome::StaleRun { reason });
                }
                ReservationDecision::Skip(reason) => {
                    return Ok(SubmissionOutcome::Skipped {
                        reason,
                        held_quantity,
                        avg_price,
                    })
                }
            }
        };

        let order_result = place_prepared_order(&deps, &prepared).await;
        match order_result {
            Ok(response) => {
                let mut pending = build_pending_order(&prepared, response);
                if let Some(id) = &prepared.submission.budget_id {
                    pending.record.id = id.clone();
                }
                let record = pending.record.clone();
                let provider_order_id = record
                    .provider_order_id
                    .clone()
                    .or(record.kis_order_id.clone())
                    .unwrap_or_default();
                let guard_signal = pending.signal();
                {
                    let mut manager = order_manager.lock().await;
                    manager.finish_submission_success(reservation, pending.clone(), &guard_signal);
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
                order_manager.lock().await.accept_budget(&pending).await?;
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
                        order_manager
                            .lock()
                            .await
                            .block_for_persistence_failure(format!(
                                "리스크 runtime 상태 저장 실패: {e}"
                            ));
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
                    let same_provider_mode = deps.broker_scope.broker_id != BrokerId::Kis
                        || manager.rest_client.read().await.is_paper()
                            == deps.rest_client.is_paper();
                    manager.finish_submission_failure(
                        &reservation,
                        insufficient_balance && same_provider_mode,
                        &msg,
                    );
                    if e.downcast_ref::<crate::api::rest::KisOrderRejected>()
                        .is_some()
                        || crate::broker::toss::error::definitively_rejected(&e)
                    {
                        if let Some(id) = &prepared.submission.budget_id {
                            manager.reject_budget(id).await?;
                        }
                    }
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
                        "잔고 부족 응답 — 주문 제출 실패: {} | {}",
                        prepared.submission.symbol,
                        msg
                    );
                    Ok(SubmissionOutcome::Skipped {
                        reason: msg,
                        held_quantity,
                        avg_price,
                    })
                } else if paper_unsupported {
                    tracing::warn!(
                        "모의투자 매도 미지원 — 스킵: {} ({}) | {}",
                        prepared.submission.symbol,
                        prepared.order_exchange.as_deref().unwrap_or("-"),
                        msg
                    );
                    Ok(SubmissionOutcome::Skipped {
                        reason: msg,
                        held_quantity,
                        avg_price,
                    })
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

    fn manual_snapshot_matches(
        &self,
        deps: &SubmissionDeps,
        profiles: &ProfilesConfig,
        current_client: &Arc<KisRestClient>,
    ) -> bool {
        if self.execution_scope != deps.broker_scope
            || !Arc::ptr_eq(current_client, &deps.rest_client)
        {
            return false;
        }
        match (&deps.active_profile, profiles.get_active()) {
            (Some(expected), Some(current)) => {
                expected.id == current.id
                    && expected.broker_id == current.broker_id
                    && expected.account_no == current.account_no
                    && expected.app_key == current.app_key
                    && expected.app_secret == current.app_secret
                    && expected.is_paper_trading == current.is_paper_trading
                    && expected.live_trading_consent == current.live_trading_consent
            }
            (None, None) => {
                execution_run::validate_snapshot(&deps.broker_scope, None, &deps.rest_client, None)
                    .is_ok()
            }
            _ => false,
        }
    }

    fn submission_run_is_current(&self, submission: &OrderSubmission) -> bool {
        match (submission.is_manual, &submission.expected_run) {
            (true, None) => true,
            (false, Some(expected)) => {
                expected.broker_scope == submission.broker_scope
                    && self.execution_run_matches(expected)
            }
            _ => false,
        }
    }

    fn reserve_submission(
        &mut self,
        submission: &OrderSubmission,
        held_quantity: u64,
        avg_price: Option<u64>,
    ) -> ReservationDecision {
        if !self.submission_run_is_current(submission) {
            return ReservationDecision::StaleRun(stale_run_reason());
        }
        if self.configuration_blocks_order(&submission.broker_scope, &submission.symbol)
            || (submission.is_manual
                && self.configuration_blocks_holdings(&submission.broker_scope))
        {
            return ReservationDecision::Skip(
                "전략 설정 저장 중에는 해당 계좌·종목의 주문을 잠시 보류합니다.".into(),
            );
        }
        if self.persistence_blocked {
            return ReservationDecision::Skip(
                self.buy_suspended_reason.clone().unwrap_or_else(|| {
                    "주문 영속화 장애로 모든 신규 주문이 차단되었습니다.".into()
                }),
            );
        }
        if self.holdings_refresh_required != 0 {
            return ReservationDecision::Skip(
                "체결 후 증권사 잔고를 다시 확인하고 있습니다.".into(),
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
            side: submission.side,
        };
        self.submitting.insert(
            (reservation.broker_scope.clone(), reservation.symbol.clone()),
            reservation.side,
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
        // 늦은 옛 계좌 응답은 새 계좌의 매수 상태를 바꾸지 않는다.
        // 같은 계좌의 새 run에는 잔고 부족 사실이 여전히 유효하다.
        if insufficient_balance && self.execution_scope == reservation.broker_scope {
            self.buy_suspended = true;
            self.buy_suspended_reason = Some(message.to_string());
        }
    }
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
                ownership_key: None,
                ownership_generation: None,
                strategy_position_metadata: None,
                submitted_cost_basis: None,
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
                expected_run: None,
                budget_id: None,
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

fn stale_run_reason() -> String {
    "자동매매 실행 또는 전략 설정이 변경되어 이전 신호를 폐기했습니다.".into()
}

fn stale_run_outcome() -> SubmissionOutcome {
    SubmissionOutcome::StaleRun {
        reason: stale_run_reason(),
    }
}

async fn submission_is_stale(
    manager: &Arc<Mutex<OrderManager>>,
    submission: &OrderSubmission,
) -> bool {
    !manager.lock().await.submission_run_is_current(submission)
}

#[cfg(test)]
pub(super) struct PreparationTestGate {
    arrived: tokio::sync::Barrier,
    resume: tokio::sync::Barrier,
}

#[cfg(test)]
mod run_tests;
