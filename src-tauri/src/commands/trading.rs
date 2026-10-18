use super::*;

mod balance;
mod daemon;
mod daily_history;
mod history;
mod positions;
mod risk_restore;
#[cfg(test)]
mod risk_restore_tests;

pub(crate) use balance::{fetch_account_risk_balance_krw, fetch_toss_risk_balance_krw};
pub use daemon::run_trading_daemon;
pub(crate) use history::initialize_active_strategy_history;
use positions::sync_strategy_positions_from_active_broker;
pub(crate) use risk_restore::restore_risk_from_today_trades;

// ────────────────────────────────────────────────────────────────────
// 자동 매매 제어
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingStatus {
    pub is_running: bool,
    pub active_strategies: Vec<String>,
    pub position_count: usize,
    pub total_unrealized_pnl: i64,
    /// WebSocket 실시간 시세 연결 여부
    pub ws_connected: bool,
    /// 자동매매가 실행 중인 프로파일 ID (미실행 시 None)
    pub trading_profile_id: Option<String>,
    /// 자동매매가 실행 중인 broker ID (미실행 시 None)
    pub trading_broker_id: Option<BrokerId>,
    /// 자동매매가 실행 중인 broker account ID (미실행 시 None)
    pub trading_account_id: Option<String>,
    /// 잔고 부족으로 매수 정지 여부
    pub buy_suspended: bool,
    /// 매수 정지 사유 (KIS 응답 msg1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_suspended_reason: Option<String>,
    pub health: crate::trading::order::TradingHealthStatus,
    pub daily_history: Vec<crate::trading::strategy::DailyHistoryStatus>,
}

#[tauri::command]
pub async fn get_trading_status(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    let is_running = *state.is_trading.lock().await;
    let (strategies, daily_history) = {
        let manager = state.strategy_manager.lock().await;
        (manager.active_names(), manager.daily_history_status())
    };
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let history_identity = state
        .strategy_manager
        .lock()
        .await
        .history_execution_identity();
    let (trading_profile_id, trading_broker_id, trading_account_id) = if is_running {
        match history_identity {
            Some((scope, profile_id)) => (
                profile_id,
                Some(scope.broker_id),
                scope.account_id.map(|id| id.0),
            ),
            None => (None, None, None),
        }
    } else {
        (None, None, None)
    };
    let (buy_suspended, buy_suspended_reason, health) = {
        let om = state.order_manager.lock().await;
        (
            om.buy_suspended,
            om.buy_suspended_reason.clone(),
            om.health_status(),
        )
    };
    Ok(TradingStatus {
        is_running,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
        trading_broker_id,
        trading_account_id,
        buy_suspended,
        buy_suspended_reason,
        health,
        daily_history,
    })
}

#[tauri::command]
pub async fn start_trading(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<TradingStatus> {
    let _storage_maintenance = state.storage_maintenance.lock().await;
    // Serialize profile/config publication across the entire startup snapshot.
    let _strategy_update = state.strategy_update_lock.lock().await;
    if state.database_manager.config_view().await.active_backend
        == crate::storage::database::StorageBackend::Database
    {
        state
            .database_manager
            .status()
            .await
            .map_err(|error| CmdError {
                code: "STORAGE_UNAVAILABLE".into(),
                message: format!(
                    "DB 저장소를 확인할 수 없어 자동매매를 시작하지 않습니다: {error}"
                ),
            })?;
    }
    let current_cfg = state.config.read().await.clone();
    if current_cfg.broker_id == BrokerId::Kis && !current_cfg.is_kis_configured() {
        return Err(CmdError {
            code: "CONFIG_NOT_READY".into(),
            message: "KIS API 설정이 완료되지 않았습니다. Settings에서 API 키를 확인하세요.".into(),
        });
    }
    if current_cfg.broker_id == BrokerId::Toss {
        let active_profile = {
            let profiles = state.profiles.read().await;
            profiles.get_active().cloned()
        };
        let Some(profile) = active_profile.filter(|profile| profile.broker_id == BrokerId::Toss)
        else {
            return Err(CmdError {
                code: "CONFIG_NOT_READY".into(),
                message: "활성 Toss 프로파일이 없습니다.".into(),
            });
        };
        if !profile.is_configured() {
            return Err(CmdError {
                code: "CONFIG_NOT_READY".into(),
                message: "토스증권 Client ID/Secret/accountSeq 설정이 완료되지 않았습니다.".into(),
            });
        }
        if !profile.live_trading_consent {
            return Err(CmdError {
                code: "LIVE_TRADING_CONSENT_REQUIRED".into(),
                message: "Toss 실거래 동의를 저장해야 자동매매 주문 실행을 시작할 수 있습니다."
                    .into(),
            });
        }
    }

    if *state.is_trading.lock().await {
        return Err(CmdError {
            code: "ALREADY_RUNNING".into(),
            message: "자동 매매가 이미 실행 중입니다.".into(),
        });
    }

    // 자동매매 시작 전 실제 잔고를 전략 내부 포지션 상태와 동기화한다.
    // 재시작 직후 내부 in_position=false 상태로 같은 종목을 재매수하는 위험을 줄인다.
    let synced_positions =
        match sync_strategy_positions_from_active_broker(&state, &current_cfg).await {
            Ok(count) => {
                state.order_manager.lock().await.mark_holdings_sync(None);
                count
            }
            Err(error) => {
                state
                    .order_manager
                    .lock()
                    .await
                    .mark_holdings_sync(Some(error.message.clone()));
                return Err(error);
            }
        };
    let active_profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    };
    let broker_id = active_profile
        .as_ref()
        .map(|profile| profile.broker_id)
        .unwrap_or(current_cfg.broker_id);
    let exchange_rate = *state.exchange_rate_krw.read().await;
    let initial_total_balance = match broker_id {
        BrokerId::Kis => {
            let include_overseas = state
                .strategy_manager
                .lock()
                .await
                .active_symbols()
                .iter()
                .any(|symbol| !is_domestic_symbol(symbol));
            let rest = state.rest_client.read().await.clone();
            fetch_account_risk_balance_krw(&rest, include_overseas, exchange_rate).await
        }
        BrokerId::Toss => match active_profile.as_ref() {
            Some(profile) => fetch_toss_risk_balance_krw(profile, exchange_rate).await,
            None => Err("활성 Toss 프로파일이 없습니다.".into()),
        },
    }
    .map_err(|message| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: format!("자동매매 시작 전 계좌 평가액 동기화 실패: {message}"),
    })?;
    if initial_total_balance <= 0 {
        return Err(CmdError {
            code: "ACCOUNT_BALANCE_UNAVAILABLE".into(),
            message: "계좌 총평가액이 0원 이하로 확인되어 자동매매를 시작하지 않습니다.".into(),
        });
    }
    tracing::info!(
        "자동매매 시작 전 broker-aware 계좌 동기화 완료: {}개, 총평가액={}원",
        synced_positions,
        initial_total_balance
    );

    let reconciliation_account_id = active_profile
        .as_ref()
        .map(|profile| profile.broker_account_id())
        .or_else(|| {
            (!current_cfg.broker_account_id.is_empty())
                .then(|| current_cfg.broker_account_id.clone())
        });
    let execution_scope =
        BrokerScope::new(broker_id, reconciliation_account_id.map(BrokerAccountId));
    state
        .order_manager
        .lock()
        .await
        .set_execution_scope(execution_scope.clone());

    OrderManager::confirm_pending_fills_from_broker_shared(&state.order_manager)
        .await
        .map_err(|error| CmdError {
            code: "PENDING_RECONCILIATION_FAILED".into(),
            message: format!(
                "복원된 미체결 주문을 broker와 대조하지 못해 자동매매를 시작하지 않습니다: {error}"
            ),
        })?;
    restore_risk_from_today_trades(
        &state.order_store,
        &state.trade_store,
        &state.risk_manager,
        &state.risk_store,
        &execution_scope,
    )
    .await?;

    let mut is_running = state.is_trading.lock().await;
    if *is_running {
        return Err(CmdError {
            code: "ALREADY_RUNNING".into(),
            message: "자동 매매가 이미 실행 중입니다.".into(),
        });
    }
    tracing::info!("자동 매매 시작 준비");

    // 자동매매 시작 시점의 활성 프로파일 ID 스냅샷 저장
    {
        let (active_id, broker_id, account_id) = {
            let profiles = state.profiles.read().await;
            match profiles.get_active() {
                Some(profile) => (
                    Some(profile.id.clone()),
                    Some(profile.broker_id),
                    Some(profile.broker_account_id()),
                ),
                None => (
                    profiles.active_id.clone(),
                    Some(current_cfg.broker_id),
                    Some(current_cfg.broker_account_id.clone())
                        .filter(|account_id| !account_id.is_empty()),
                ),
            }
        };
        let execution_scope = BrokerScope::new(
            broker_id.unwrap_or(BrokerId::Kis),
            account_id.clone().map(BrokerAccountId),
        );
        state
            .order_manager
            .lock()
            .await
            .set_execution_scope(execution_scope);
        *state.trading_profile_id.write().await = active_id;
        *state.trading_broker_id.write().await = broker_id;
        *state.trading_account_id.write().await = account_id;
    }

    // daemon이 `is_trading`을 읽기 전에 live와 preview가 공유하는 히스토리/warmup
    // 초기화를 끝낸다. 이 lock을 먼저 풀면 초기화되지 않은 상태로 첫 tick이 처리될 수 있다.
    initialize_active_strategy_history(
        &state.strategy_manager,
        &state.order_manager,
        &state.profiles,
        &state.rest_client,
        &state.risk_manager,
    )
    .await;
    let history_scope = state.order_manager.lock().await.execution_scope().clone();
    let history_profile_id = state.trading_profile_id.read().await.clone();
    let rest_snapshot = state.rest_client.read().await.clone();
    {
        // The bool → strategy → order lock order also fences quote and reservation work.
        let mut manager = state.strategy_manager.lock().await;
        manager.advance_history_generation();
        state
            .order_manager
            .lock()
            .await
            .begin_execution_run(
                history_scope.clone(),
                history_profile_id.clone(),
                manager.history_generation(),
                rest_snapshot,
                active_profile,
            )
            .map_err(|error| CmdError {
                code: "EXECUTION_CONTEXT_CHANGED".into(),
                message: error.to_string(),
            })?;
        manager.set_history_execution_identity(history_scope, history_profile_id);
    }
    *is_running = true;
    drop(is_running);

    if let Some(notifier) = &state.discord {
        let _ = notifier
            .send(NotificationEvent::info(
                "자동 매매 시작".to_string(),
                "AutoConditionTrade 자동 매매가 시작되었습니다.".to_string(),
            ))
            .await;
    }

    // WebSocket 연결 시작 (KIS 전용 보조 — 실패해도 폴링 루프가 독립 동작)
    if current_cfg.broker_id == BrokerId::Kis {
        let rest = state.rest_client.read().await.clone();
        let ws_client = crate::api::KisWebSocketClient::new(
            rest.is_paper(),
            rest.app_key().to_string(),
            rest.app_secret().to_string(),
            rest.token_manager(),
        );

        // 활성 전략에서 구독할 종목 수집
        let symbols: Vec<String> = state.strategy_manager.lock().await.active_symbols();

        let ws_connected = Arc::clone(&state.ws_connected);
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = ws_client.subscribe(symbols, app_handle, ws_connected).await {
                tracing::error!("WebSocket 연결 실패: {}", e);
            }
        });
    } else {
        tracing::info!(
            "Toss 자동매매는 현재 polling 기반으로 실행하며 KIS WebSocket 구독을 생략합니다."
        );
    }

    // ── 폴링 기반 자동매매 루프 ──────────────────────────────────
    // run_trading_daemon() 이 앱 시작 시 영구 데몬으로 이미 실행 중이다.
    // is_trading 플래그가 true 로 바뀌면 데몬이 자동으로 폴링을 재개한다.
    // (이전 spawn 블록은 lib.rs → tauri::async_runtime::spawn(run_trading_daemon(...)) 로 이동)

    let (strategies, daily_history) = {
        let manager = state.strategy_manager.lock().await;
        (manager.active_names(), manager.daily_history_status())
    };
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let trading_profile_id = state.trading_profile_id.read().await.clone();
    let trading_broker_id = *state.trading_broker_id.read().await;
    let trading_account_id = state.trading_account_id.read().await.clone();
    let (buy_suspended, buy_suspended_reason, health) = {
        let om = state.order_manager.lock().await;
        (
            om.buy_suspended,
            om.buy_suspended_reason.clone(),
            om.health_status(),
        )
    };
    Ok(TradingStatus {
        is_running: true,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
        trading_broker_id,
        trading_account_id,
        buy_suspended,
        buy_suspended_reason,
        health,
        daily_history,
    })
}

#[tauri::command]
pub async fn stop_trading(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    let mut is_running = state.is_trading.lock().await;
    *is_running = false;
    {
        let mut manager = state.strategy_manager.lock().await;
        manager.advance_history_generation();
        manager.clear_history_execution_identity();
        state.order_manager.lock().await.end_execution_run();
    }
    tracing::info!("자동 매매 정지");

    // 자동매매 종료 시 트레이딩 프로파일 ID 클리어
    *state.trading_profile_id.write().await = None;
    *state.trading_broker_id.write().await = None;
    *state.trading_account_id.write().await = None;

    if let Some(notifier) = &state.discord {
        let _ = notifier
            .send(NotificationEvent::info(
                "자동 매매 정지".to_string(),
                "AutoConditionTrade 자동 매매가 정지되었습니다.".to_string(),
            ))
            .await;
    }
    drop(is_running);

    let (strategies, daily_history) = {
        let manager = state.strategy_manager.lock().await;
        (manager.active_names(), manager.daily_history_status())
    };
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let (buy_suspended, buy_suspended_reason, health) = {
        let manager = state.order_manager.lock().await;
        (
            manager.buy_suspended,
            manager.buy_suspended_reason.clone(),
            manager.health_status(),
        )
    };
    Ok(TradingStatus {
        is_running: false,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id: None,
        trading_broker_id: None,
        trading_account_id: None,
        buy_suspended,
        buy_suspended_reason,
        health,
        daily_history,
    })
}

/// 잔고 부족 매수 정지를 수동으로 해제합니다.
/// 계좌에 자금을 입금한 경우 또는 오판 시 사용.
#[tauri::command]
pub async fn clear_buy_suspension(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    state.order_manager.lock().await.clear_buy_suspension();

    let is_running = *state.is_trading.lock().await;
    let (strategies, daily_history) = {
        let manager = state.strategy_manager.lock().await;
        (manager.active_names(), manager.daily_history_status())
    };
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let history_identity = state
        .strategy_manager
        .lock()
        .await
        .history_execution_identity();
    let (trading_profile_id, trading_broker_id, trading_account_id) = if is_running {
        match history_identity {
            Some((scope, profile_id)) => (
                profile_id,
                Some(scope.broker_id),
                scope.account_id.map(|id| id.0),
            ),
            None => (None, None, None),
        }
    } else {
        (None, None, None)
    };
    let (buy_suspended, buy_suspended_reason, health) = {
        let manager = state.order_manager.lock().await;
        (
            manager.buy_suspended,
            manager.buy_suspended_reason.clone(),
            manager.health_status(),
        )
    };
    Ok(TradingStatus {
        is_running,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
        trading_broker_id,
        trading_account_id,
        buy_suspended,
        buy_suspended_reason,
        health,
        daily_history,
    })
}

/// 자동매매 실행 scope(`OrderManager::execution_scope`)에 해당하는 계정 프로파일을 조회한다.
///
/// `account_id`가 scope에 있으면 broker_id+account_id로 매칭하고, 없으면 활성 프로파일로 폴백한다.
/// (order/submission.rs의 `SubmissionDeps.active_profile` 조회 로직과 동일한 매칭 규칙 — 시세 조회와
/// 주문 제출이 서로 다른 프로파일을 바라보지 않도록 일치시킨다.)
async fn resolve_scoped_profile(
    profiles: &Arc<RwLock<ProfilesConfig>>,
    scope: &BrokerScope,
) -> Option<AccountProfile> {
    let profiles = profiles.read().await;
    let account_id = scope.account_id.as_ref().map(|id| id.0.as_str());
    profiles
        .profiles
        .iter()
        .find(|profile| {
            profile.broker_id == scope.broker_id
                && account_id
                    .map(|id| profile.broker_account_id() == id)
                    .unwrap_or_else(|| {
                        profiles.get_active().map(|p| p.id.as_str()) == Some(profile.id.as_str())
                    })
        })
        .cloned()
}
