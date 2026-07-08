use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

use super::ServerState;
use crate::broker::{
    BrokerAccountId, BrokerAdapter, BrokerId, BrokerMarket, BrokerScope, TossBrokerAdapter,
};
use crate::config::AccountProfile;
use crate::trading::strategy::BrokerPositionSnapshot;

/// GET /api/trading/status
pub(super) async fn trading_status_handler(
    State(s): State<ServerState>,
) -> Json<serde_json::Value> {
    let is_running = *s.is_trading.lock().await;
    let active_strategies: Vec<String> = s.strategy_manager.lock().await.active_names();
    let (position_count, total_unrealized_pnl) = {
        let tracker = s.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let (buy_suspended, buy_suspended_reason) = {
        let om = s.order_manager.lock().await;
        (om.buy_suspended, om.buy_suspended_reason.clone())
    };
    Json(serde_json::json!({
        "isRunning":           is_running,
        "activeStrategies":    active_strategies,
        "positionCount":       position_count,
        "totalUnrealizedPnl":  total_unrealized_pnl,
        "wsConnected":         false,
        "tradingProfileId":    null,
        "buySuspended":        buy_suspended,
        "buySuspendedReason":  buy_suspended_reason,
    }))
}

async fn sync_kis_strategy_positions(s: &ServerState) -> usize {
    let rest = s.rest_client.read().await.clone();
    let mut synced = 0usize;
    match rest.get_balance().await {
        Ok(resp) => {
            s.stock_store
                .upsert_many(
                    resp.items
                        .iter()
                        .map(|i| (i.pdno.clone(), i.prdt_name.clone())),
                )
                .await;
            {
                let mut tracker = s.position_tracker.lock().await;
                tracker.load_if_empty(resp.items.iter().map(|i| {
                    (
                        i.pdno.clone(),
                        i.prdt_name.clone(),
                        i.hldg_qty.parse::<u64>().unwrap_or(0),
                        i.pchs_avg_pric.parse::<f64>().unwrap_or(0.0) as u64,
                        i.prpr.parse::<u64>().unwrap_or(0),
                    )
                }));
            }
            let mut mgr = s.strategy_manager.lock().await;
            for item in &resp.items {
                let qty = item.hldg_qty.parse::<u64>().unwrap_or(0);
                let avg = item.pchs_avg_pric.parse::<f64>().unwrap_or(0.0) as u64;
                if qty > 0 {
                    synced += 1;
                }
                mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                    broker_id: BrokerId::Kis,
                    market: BrokerMarket::Kr,
                    symbol: item.pdno.clone(),
                    quantity: qty,
                    avg_price: avg,
                });
            }
        }
        Err(e) => tracing::warn!("웹 자동매매 시작 전 국내 잔고 동기화 실패: {}", e),
    }
    synced
}

async fn sync_toss_strategy_positions(s: &ServerState, profile: AccountProfile) -> usize {
    if !profile.is_configured() {
        tracing::warn!("웹 자동매매 Toss holdings 동기화 건너뜀: 설정이 미완료입니다.");
        return 0;
    }
    let account_seq = profile.broker_account_id();
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(account_seq.clone()),
    );
    let account_id = BrokerAccountId(account_seq);
    let holdings = match adapter.list_holdings(Some(&account_id)).await {
        Ok(holdings) => holdings,
        Err(e) => {
            tracing::warn!("웹 자동매매 시작 전 Toss holdings 동기화 실패: {}", e);
            return 0;
        }
    };
    s.stock_store
        .upsert_many(
            holdings
                .iter()
                .map(|holding| (holding.symbol.0.clone(), holding.symbol_name.clone())),
        )
        .await;
    {
        let mut tracker = s.position_tracker.lock().await;
        tracker.load_if_empty(
            holdings
                .iter()
                .filter(|holding| holding.market == BrokerMarket::Kr)
                .map(|holding| {
                    (
                        holding.symbol.0.clone(),
                        holding.symbol_name.clone(),
                        decimal_quantity_units(&holding.quantity.0),
                        money_units(
                            &holding.average_price.amount,
                            &holding.average_price.currency,
                        ),
                        money_units(
                            &holding.current_price.amount,
                            &holding.current_price.currency,
                        ),
                    )
                }),
        );
    }
    let mut synced = 0usize;
    let mut mgr = s.strategy_manager.lock().await;
    for holding in &holdings {
        let quantity = decimal_quantity_units(&holding.quantity.0);
        if quantity > 0 {
            synced += 1;
        }
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Toss,
            market: holding.market,
            symbol: holding.symbol.0.clone(),
            quantity,
            avg_price: money_units(
                &holding.average_price.amount,
                &holding.average_price.currency,
            ),
        });
    }
    synced
}

fn decimal_quantity_units(value: &str) -> u64 {
    let parsed = value.trim().replace(',', "").parse::<f64>().unwrap_or(0.0);
    if parsed <= 0.0 {
        0
    } else {
        parsed.floor().max(1.0) as u64
    }
}

fn money_units(value: &str, currency: &crate::broker::BrokerCurrency) -> u64 {
    let parsed = value.trim().replace(',', "").parse::<f64>().unwrap_or(0.0);
    match currency {
        crate::broker::BrokerCurrency::Krw => parsed.round().max(0.0) as u64,
        crate::broker::BrokerCurrency::Usd => (parsed.max(0.0) * 100.0).round() as u64,
    }
}

/// POST /api/trading/start — is_trading = true (폴링 데몬이 자동으로 재개)
pub(super) async fn trading_start_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let current_cfg = s.config.read().await.clone();
    if current_cfg.broker_id == BrokerId::Kis && !current_cfg.is_kis_configured() {
        return Json(serde_json::json!({
            "ok": false,
            "code": "CONFIG_NOT_READY",
            "message": "KIS API 설정이 완료되지 않았습니다. Settings에서 API 키를 확인하세요.",
        }));
    }
    let active_profile = {
        let profiles = s.profiles.read().await;
        profiles.get_active().cloned()
    };
    if current_cfg.broker_id == BrokerId::Toss {
        let Some(profile) = active_profile
            .as_ref()
            .filter(|profile| profile.broker_id == BrokerId::Toss)
        else {
            return Json(serde_json::json!({
                "ok": false,
                "code": "CONFIG_NOT_READY",
                "message": "활성 Toss 프로파일이 없습니다.",
            }));
        };
        if !profile.is_configured() {
            return Json(serde_json::json!({
                "ok": false,
                "code": "CONFIG_NOT_READY",
                "message": "토스증권 Client ID/Secret/accountSeq 설정이 완료되지 않았습니다.",
            }));
        }
        if !profile.live_trading_consent {
            return Json(serde_json::json!({
                "ok": false,
                "code": "LIVE_TRADING_CONSENT_REQUIRED",
                "message": "Toss 실거래 동의를 저장해야 자동매매 주문 실행을 시작할 수 있습니다.",
            }));
        }
    }
    if *s.is_trading.lock().await {
        return Json(serde_json::json!({ "ok": false, "message": "이미 실행 중입니다." }));
    }

    let synced_positions = match current_cfg.broker_id {
        BrokerId::Kis => sync_kis_strategy_positions(&s).await,
        BrokerId::Toss => {
            let profile = active_profile
                .clone()
                .expect("Toss profile was validated before sync");
            sync_toss_strategy_positions(&s, profile).await
        }
    };
    let (active_id, broker_id, account_id) = {
        let profiles = s.profiles.read().await;
        match profiles.get_active() {
            Some(profile) => (
                Some(profile.id.clone()),
                Some(profile.broker_id),
                Some(profile.broker_account_id()),
            ),
            None => (
                profiles.active_id.clone(),
                Some(current_cfg.broker_id),
                Some(current_cfg.broker_account_id.clone()).filter(|id| !id.is_empty()),
            ),
        }
    };
    let execution_scope = BrokerScope::new(
        broker_id.unwrap_or(BrokerId::Kis),
        account_id.clone().map(BrokerAccountId),
    );
    s.order_manager
        .lock()
        .await
        .set_execution_scope(execution_scope);

    *s.is_trading.lock().await = true;
    tracing::info!(
        "자동매매 시작 (웹 API 요청): profile={:?} broker={:?} account={:?} synced_positions={}",
        active_id,
        broker_id,
        account_id,
        synced_positions
    );
    Json(serde_json::json!({ "ok": true, "message": "자동매매 시작됨" }))
}

/// POST /api/trading/stop — is_trading = false (폴링 데몬 자동 일시 정지)
pub(super) async fn trading_stop_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    *s.is_trading.lock().await = false;
    tracing::info!("자동매매 정지 (웹 API 요청)");
    Json(serde_json::json!({ "ok": true, "message": "자동매매 정지됨" }))
}

/// GET /api/strategies — 전략 목록 (이름, 활성 여부, 대상 종목)
pub(super) async fn strategies_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let configs: Vec<crate::trading::strategy::StrategyConfig> = {
        let mgr = s.strategy_manager.lock().await;
        mgr.all_configs().into_iter().cloned().collect()
    };

    let mut views = Vec::with_capacity(configs.len());
    for cfg in &configs {
        views.push(crate::trading::build_strategy_view(cfg, &s.stock_store).await);
    }
    Json(serde_json::json!(views))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UpdateStrategyBody {
    enabled: Option<bool>,
    target_symbols: Option<Vec<String>>,
    order_quantity: Option<u64>,
    params: Option<serde_json::Value>,
}

/// POST /api/strategies/:id — 전략 파라미터 업데이트
pub(super) async fn update_strategy_handler(
    State(s): State<ServerState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateStrategyBody>,
) -> Json<serde_json::Value> {
    let active_scope = {
        let cfg = s.config.read().await.clone();
        let account_id = if cfg.broker_account_id.is_empty() {
            None
        } else {
            Some(cfg.broker_account_id.clone())
        };
        (cfg.broker_id, account_id)
    };
    let updated_config = {
        let mut mgr = s.strategy_manager.lock().await;
        match mgr.update_config(&id, |cfg| {
            if let Some(en) = body.enabled {
                cfg.enabled = en;
            }
            if let Some(sym) = body.target_symbols {
                cfg.target_symbols = sym;
            }
            if let Some(qty) = body.order_quantity {
                cfg.order_quantity = qty;
            }
            if let Some(p) = body.params {
                cfg.params = p;
            }
            cfg.set_scope(active_scope.0, active_scope.1.clone());
        }) {
            Some(config) => config,
            None => {
                return Json(
                    serde_json::json!({ "error": format!("전략을 찾을 수 없습니다: {}", id) }),
                )
            }
        }
    };

    let mut symbol_names = std::collections::HashMap::new();
    for code in &updated_config.target_symbols {
        let name = s
            .stock_store
            .get_name(code)
            .await
            .unwrap_or_else(|| code.to_string());
        symbol_names.insert(code.clone(), name);
    }

    let profile_id = s.profiles.read().await.active_id.clone();
    if let Some(pid) = &profile_id {
        let all_configs: Vec<crate::trading::strategy::StrategyConfig> = {
            let mgr = s.strategy_manager.lock().await;
            mgr.all_configs().into_iter().cloned().collect()
        };
        if let Err(e) = s.strategy_store.save(pid, &all_configs).await {
            tracing::warn!("전략 설정 저장 실패: {}", e);
        }
    }

    Json(serde_json::json!({
        "id":              updated_config.id,
        "name":            updated_config.name,
        "enabled":         updated_config.enabled,
        "brokerId":        updated_config.broker_id,
        "brokerAccountId": updated_config.broker_account_id,
        "targetSymbols":   updated_config.target_symbols,
        "targetSymbolNames": symbol_names,
        "orderQuantity":   updated_config.order_quantity,
        "params":          updated_config.params,
    }))
}

/// POST /api/strategy/preview — 프론트가 제공한 차트 캔들로 전략 신호 미리보기
pub(super) async fn strategy_preview_handler(
    Json(input): Json<crate::commands::StrategyPreviewInput>,
) -> Json<serde_json::Value> {
    match crate::commands::preview_strategy_from_candles(input) {
        Ok(preview) => Json(serde_json::to_value(preview).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({
            "code": e.code,
            "error": e.message,
        })),
    }
}
