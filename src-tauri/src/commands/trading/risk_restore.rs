use super::*;

pub(crate) async fn restore_risk_from_today_trades(
    order_store: &Arc<OrderStore>,
    trade_store: &Arc<TradeStore>,
    risk_manager: &Arc<Mutex<RiskManager>>,
    risk_store: &Arc<crate::storage::RiskStore>,
    execution_scope: &BrokerScope,
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
    let scoped_trades: Vec<_> = trades
        .into_iter()
        .filter(|trade| trade.matches_scope(execution_scope))
        .collect();
    let mut risk = risk_manager.lock().await;
    risk.reset_daily();
    // 다른 broker/계좌의 당일 체결이 이 scope의 손실 한도에 섞이지 않게 격리한다.
    risk.restore_daily_pnl(
        scoped_trades
            .iter()
            .filter_map(|trade| trade.realized_pnl_krw),
    );
    let mut submitted_order_ids = std::collections::HashSet::new();
    for order in orders
        .into_iter()
        .filter(|order| order.matches_scope(execution_scope))
    {
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
        if let (Some(strategy_id), Some(pnl)) =
            (trade.strategy_id.as_deref(), trade.realized_pnl_krw)
        {
            risk.record_strategy_symbol_pnl_for_scope(
                execution_scope,
                strategy_id,
                &trade.symbol,
                pnl,
            );
        }
    }
    risk_store
        .save_runtime(&risk.runtime_state())
        .await
        .map_err(|error| CmdError {
            code: "RISK_PERSIST_FAILED".into(),
            message: format!("리스크 runtime 상태를 저장하지 못했습니다: {error}"),
        })?;
    Ok(())
}
