//! 수동 주문 전 계좌 동기화와 실제 보유 스냅샷.

use super::*;

pub(super) async fn refresh_manual_positions(
    order_manager: &Arc<Mutex<OrderManager>>,
    submission: &OrderSubmission,
    deps: &SubmissionDeps,
) -> Result<()> {
    {
        let manager = order_manager.lock().await;
        anyhow::ensure!(
            !manager.configuration_blocks_holdings(&submission.broker_scope),
            "전략 설정 저장 중에는 전체 계좌 잔고 갱신을 보류합니다."
        );
    }
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
        let manager = order_manager.lock().await;
        anyhow::ensure!(
            !manager.configuration_blocks_holdings(&submission.broker_scope),
            "전략 설정 저장 중에는 전체 계좌 잔고 게시를 보류합니다."
        );
        let profiles = manager.profiles.read().await;
        let current_client = manager.rest_client.read().await;
        anyhow::ensure!(
            manager.manual_snapshot_matches(deps, &profiles, &current_client),
            "수동 주문 계좌 또는 프로파일이 잔고 조회 중 변경되었습니다."
        );
        deps.position_tracker.lock().await.replace(domestic);
        deps.overseas_position_tracker
            .lock()
            .await
            .replace(overseas);
        return Ok(());
    }

    let client = Arc::clone(&deps.rest_client);
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
        let manager = order_manager.lock().await;
        anyhow::ensure!(
            !manager.configuration_blocks_holdings(&submission.broker_scope),
            "전략 설정 저장 중에는 전체 계좌 잔고 게시를 보류합니다."
        );
        let profiles = manager.profiles.read().await;
        let current_client = manager.rest_client.read().await;
        anyhow::ensure!(
            manager.manual_snapshot_matches(deps, &profiles, &current_client),
            "수동 주문 계좌 또는 프로파일이 잔고 조회 중 변경되었습니다."
        );
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
        let manager = order_manager.lock().await;
        anyhow::ensure!(
            !manager.configuration_blocks_holdings(&submission.broker_scope),
            "전략 설정 저장 중에는 전체 계좌 잔고 게시를 보류합니다."
        );
        let profiles = manager.profiles.read().await;
        let current_client = manager.rest_client.read().await;
        anyhow::ensure!(
            manager.manual_snapshot_matches(deps, &profiles, &current_client),
            "수동 주문 계좌 또는 프로파일이 잔고 조회 중 변경되었습니다."
        );
        deps.position_tracker.lock().await.replace(positions);
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
