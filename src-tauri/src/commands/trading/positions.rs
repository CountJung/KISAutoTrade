use super::*;
use crate::trading::order::{HoldingsMarkets, HoldingsUpdate};

pub(super) async fn sync_strategy_positions_from_active_broker(
    state: &AppState,
    current_cfg: &AppConfig,
) -> CmdResult<usize> {
    let active_profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    };
    match active_profile
        .as_ref()
        .map(|profile| profile.broker_id)
        .unwrap_or(current_cfg.broker_id)
    {
        BrokerId::Kis => sync_kis_strategy_positions(state).await,
        BrokerId::Toss => match active_profile {
            Some(profile) => sync_toss_strategy_positions(state, profile).await,
            None => Err(CmdError {
                code: "ACCOUNT_SYNC_FAILED".into(),
                message: "활성 Toss 프로파일이 없어 잔고를 동기화할 수 없습니다.".into(),
            }),
        },
    }
}

async fn sync_kis_strategy_positions(state: &AppState) -> CmdResult<usize> {
    let query = state
        .order_manager
        .lock()
        .await
        .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Both)
        .await
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;
    let rest = query.client();
    let mut synced = 0usize;
    let domestic = rest.get_balance().await.map_err(|e| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: format!("자동매매 시작 전 KIS 국내 잔고 동기화 실패: {e}"),
    })?;
    let overseas = rest.get_overseas_balance().await.map_err(|e| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: format!("자동매매 시작 전 KIS 해외 잔고 동기화 실패: {e}"),
    })?;
    let HoldingsUpdate::Domestic(domestic_positions) = HoldingsUpdate::from_kis_domestic(&domestic)
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_INVALID_DATA".into(),
            message: error.to_string(),
        })?
    else {
        unreachable!()
    };
    let HoldingsUpdate::Overseas(overseas_positions) = HoldingsUpdate::from_kis_overseas(&overseas)
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_INVALID_DATA".into(),
            message: error.to_string(),
        })?
    else {
        unreachable!()
    };

    state
        .stock_store
        .upsert_many(
            domestic
                .items
                .iter()
                .map(|item| (item.pdno.clone(), item.prdt_name.clone())),
        )
        .await;
    let mut mgr = state.strategy_manager.lock().await;
    let mut orders = state.order_manager.lock().await;
    orders
        .publish_holdings(
            query,
            HoldingsUpdate::Both {
                domestic: domestic_positions.clone(),
                overseas: overseas_positions.clone(),
            },
        )
        .await
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;
    synced += domestic_positions.iter().filter(|item| item.2 > 0).count();
    synced += overseas_positions.iter().filter(|item| item.3 > 0).count();
    orders
        .sync_strategy_ownership(&mut mgr)
        .await
        .map_err(|error| CmdError {
            code: "OWNERSHIP_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;

    Ok(synced)
}

async fn sync_toss_strategy_positions(
    state: &AppState,
    profile: AccountProfile,
) -> CmdResult<usize> {
    if !profile.is_configured() {
        return Err(CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: "활성 Toss 프로파일 설정이 미완료되어 잔고를 동기화할 수 없습니다.".into(),
        });
    }

    let client = state.rest_client.read().await.clone();
    let scope = BrokerScope::new(
        BrokerId::Toss,
        Some(BrokerAccountId(profile.broker_account_id())),
    );
    let query = state
        .order_manager
        .lock()
        .await
        .begin_holdings_query_for_snapshot(&scope, &client, Some(&profile), HoldingsMarkets::Both)
        .await
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;
    let profile = query.profile().cloned().ok_or_else(|| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: "활성 Toss 프로파일이 없습니다.".into(),
    })?;
    let account_id = BrokerAccountId(profile.broker_account_id());
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let holdings = adapter
        .list_holdings(Some(&account_id))
        .await
        .map_err(|e| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: format!("자동매매 시작 전 Toss 보유종목 동기화 실패: {e}"),
        })?;
    let update = HoldingsUpdate::from_toss(&scope, &holdings).map_err(|error| CmdError {
        code: "ACCOUNT_SYNC_INVALID_DATA".into(),
        message: error.to_string(),
    })?;
    let HoldingsUpdate::Both { domestic, overseas } = &update else {
        unreachable!()
    };
    let synced = domestic.iter().filter(|item| item.2 > 0).count()
        + overseas.iter().filter(|item| item.3 > 0).count();

    state
        .stock_store
        .upsert_many(
            holdings
                .iter()
                .map(|holding| (holding.symbol.0.clone(), holding.symbol_name.clone())),
        )
        .await;

    let mut mgr = state.strategy_manager.lock().await;
    let mut orders = state.order_manager.lock().await;
    orders
        .publish_holdings(query, update)
        .await
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;
    orders
        .sync_strategy_ownership(&mut mgr)
        .await
        .map_err(|error| CmdError {
            code: "OWNERSHIP_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;
    tracing::info!(
        "Toss holdings 기반 전략 포지션 동기화 완료: {}개 보유 종목",
        synced
    );
    Ok(synced)
}
