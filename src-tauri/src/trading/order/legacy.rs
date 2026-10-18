//! 기존 단일 관리자 주문 처리와 KIS 재시도 경로.

use super::*;

impl OrderManager {
    // ── 내부 구현 ────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn process_buy(
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
    pub(super) async fn process_sell(
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
            side,
            quantity,
            order_price,
            if exchange.is_some() {
                "Limit"
            } else {
                "Market"
            }
            .to_string(),
        )
        .with_strategy_id(strategy_id.clone())
        .with_provider_trace(
            "kis",
            Some(response.odno.clone()),
            None,
            Some(response.tr_id.clone()),
        )
        .with_broker_scope(&broker_scope);
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
        self.persist_risk_runtime().await;
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
        )
        .with_broker_scope(&self.execution_scope);
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
