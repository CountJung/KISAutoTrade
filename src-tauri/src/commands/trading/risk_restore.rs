use super::*;

pub(crate) async fn restore_risk_from_today_trades(
    order_store: &Arc<OrderStore>,
    trade_store: &Arc<TradeStore>,
    risk_manager: &Arc<Mutex<RiskManager>>,
    risk_store: &Arc<crate::storage::RiskStore>,
    execution_scope: &BrokerScope,
    execution_is_paper: bool,
) -> CmdResult<()> {
    let today = chrono::Local::now().date_naive();
    let mut trades = trade_store
        .get_by_date(today)
        .await
        .map_err(|error| CmdError {
            code: "RISK_RESTORE_FAILED".into(),
            message: format!("오늘 체결 ledger를 읽지 못했습니다: {error}"),
        })?;
    let orders = order_store
        .get_by_date(today)
        .await
        .map_err(|error| CmdError {
            code: "RISK_RESTORE_FAILED".into(),
            message: format!("오늘 주문 ledger를 읽지 못했습니다: {error}"),
        })?;
    trades.sort_by(|left, right| left.timestamp.cmp(&right.timestamp));
    let mode_matches = |tr_id: Option<&str>| -> CmdResult<bool> {
        if execution_scope.broker_id == BrokerId::Toss {
            return Ok(true);
        }
        OrderManager::kis_order_is_paper(tr_id)
            .map(|mode| mode == execution_is_paper)
            .map_err(|error| CmdError {
                code: "RISK_MODE_UNCONFIRMED".into(),
                message: error.to_string(),
            })
    };
    let mut scoped_trades = Vec::new();
    for trade in trades
        .into_iter()
        .filter(|trade| trade.matches_scope(execution_scope) && trade.realized_pnl_krw.is_some())
    {
        if mode_matches(trade.provider_tr_id.as_deref())? {
            scoped_trades.push(trade);
        }
    }
    let mut scoped_orders = Vec::new();
    for order in orders.into_iter().filter(|order| {
        order.matches_scope(execution_scope)
            && order.strategy_id.is_some()
            && !matches!(
                order.status,
                crate::storage::order_store::OrderStatus::Failed
            )
    }) {
        if mode_matches(order.provider_tr_id.as_deref())? {
            scoped_orders.push(order);
        }
    }
    let mut current = risk_manager.lock().await;
    let mut risk = current.clone();
    risk.reset_daily();
    // Rebuild both the totals and event receipts from the same durable source.
    // A pending intent absent from TradeStore is retried; an already recorded
    // trade cannot add its PnL again after restart.
    let mut submitted_order_ids = std::collections::HashSet::new();
    for order in scoped_orders {
        if matches!(
            order.status,
            crate::storage::order_store::OrderStatus::Failed
        ) {
            continue;
        }
        let Some(strategy_id) = order.strategy_id.as_deref() else {
            continue;
        };
        let Some(provider_order_id) = order
            .provider_order_id
            .as_deref()
            .or(order.kis_order_id.as_deref())
        else {
            continue;
        };
        if !submitted_order_ids.insert(provider_order_id.to_string()) {
            continue;
        }
        risk.record_order_submitted_for_scope(
            execution_scope,
            strategy_id,
            &order.symbol,
            match order.side {
                crate::storage::order_store::OrderSide::Buy => {
                    crate::trading::risk::DailyOrderSide::Buy
                }
                crate::storage::order_store::OrderSide::Sell => {
                    crate::trading::risk::DailyOrderSide::Sell
                }
            },
        );
    }
    for trade in &scoped_trades {
        if let Some(pnl) = trade.realized_pnl_krw {
            risk.record_fill_pnl_once(
                today,
                trade.id.strip_prefix("trade-").unwrap_or(&trade.id),
                execution_scope,
                trade.strategy_id.as_deref().unwrap_or("unknown"),
                &trade.symbol,
                pnl,
            )
            .map_err(|error| CmdError {
                code: "RISK_RESTORE_FAILED".into(),
                message: error.to_string(),
            })?;
        }
    }
    risk_store
        .save_runtime(&risk.runtime_state())
        .await
        .map_err(|error| CmdError {
            code: "RISK_PERSIST_FAILED".into(),
            message: format!("리스크 runtime 상태를 저장하지 못했습니다: {error}"),
        })?;
    *current = risk;
    Ok(())
}

#[cfg(test)]
mod tests;
