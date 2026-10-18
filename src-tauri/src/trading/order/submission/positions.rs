//! 수동 주문 전 계좌 동기화와 실제 보유 스냅샷.

use super::*;

pub(super) async fn refresh_manual_positions(
    order_manager: &Arc<Mutex<OrderManager>>,
    submission: &OrderSubmission,
    deps: &SubmissionDeps,
) -> Result<()> {
    let markets = if submission.broker_scope.broker_id == BrokerId::Toss {
        HoldingsMarkets::Both
    } else if submission.exchange.is_some() {
        HoldingsMarkets::Overseas
    } else {
        HoldingsMarkets::Domestic
    };
    let query = order_manager
        .lock()
        .await
        .begin_holdings_query_for_snapshot(
            &submission.broker_scope,
            &deps.rest_client,
            deps.active_profile.as_ref(),
            markets,
        )
        .await?;
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
        let update = HoldingsUpdate::from_toss(&submission.broker_scope, &holdings)?;
        order_manager
            .lock()
            .await
            .publish_holdings(query, update)
            .await?;
        return Ok(());
    }

    let client = Arc::clone(&deps.rest_client);
    if submission.exchange.is_some() {
        let balance = client.get_overseas_balance().await.map_err(|error| {
            anyhow::anyhow!("KIS 해외 수동 주문 전 holdings 조회 실패: {error}")
        })?;
        let update = HoldingsUpdate::from_kis_overseas(&balance)?;
        order_manager
            .lock()
            .await
            .publish_holdings(query, update)
            .await?;
    } else {
        let balance = client
            .get_balance()
            .await
            .map_err(|error| anyhow::anyhow!("KIS 수동 주문 전 holdings 조회 실패: {error}"))?;
        let update = HoldingsUpdate::from_kis_domestic(&balance)?;
        order_manager
            .lock()
            .await
            .publish_holdings(query, update)
            .await?;
    }
    Ok(())
}

pub(super) async fn current_position_snapshot(
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
