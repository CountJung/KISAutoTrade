//! 리스크 검사와 broker별 주문 요청 준비.

use super::*;

pub(super) async fn prepare_order_submission(
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
            .unwrap_or_else(|| {
                (!submission.is_manual && matches!(submission.side, OrderSide::Buy))
                    || submission.exchange.is_some()
            });
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
        let order_type = submission.requested_order_type.unwrap_or(
            if !submission.is_manual && matches!(submission.side, OrderSide::Buy) {
                OrderType::Limit
            } else {
                OrderType::Market
            },
        );
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
