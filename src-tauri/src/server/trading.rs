use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use super::ServerState;
use crate::broker::{
    BrokerAccountId, BrokerAdapter, BrokerCurrency, BrokerId, BrokerMarket, BrokerScope,
    TossBrokerAdapter,
};
use crate::config::AccountProfile;
use crate::trading::order::{HoldingsMarkets, HoldingsUpdate};
use crate::trading::strategy::BrokerPositionSnapshot;

/// GET /api/trading/status
pub(super) async fn trading_status_handler(
    State(s): State<ServerState>,
) -> Json<serde_json::Value> {
    let is_running = *s.is_trading.lock().await;
    let (active_strategies, daily_history, history_identity) = {
        let manager = s.strategy_manager.lock().await;
        (
            manager.active_names(),
            manager.daily_history_status(),
            manager.history_execution_identity(),
        )
    };
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
    let (position_count, total_unrealized_pnl) = {
        let tracker = s.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let (buy_suspended, buy_suspended_reason, health) = {
        let om = s.order_manager.lock().await;
        (
            om.buy_suspended,
            om.buy_suspended_reason.clone(),
            om.health_status(),
        )
    };
    Json(serde_json::json!({
        "isRunning":           is_running,
        "activeStrategies":    active_strategies,
        "positionCount":       position_count,
        "totalUnrealizedPnl":  total_unrealized_pnl,
        "wsConnected":         false,
        "tradingProfileId":    trading_profile_id,
        "tradingBrokerId":     trading_broker_id,
        "tradingAccountId":    trading_account_id,
        "dailyHistory":        daily_history,
        "buySuspended":        buy_suspended,
        "buySuspendedReason":  buy_suspended_reason,
        "health":              health,
    }))
}

async fn sync_kis_strategy_positions(s: &ServerState) -> Result<usize, String> {
    let query = s
        .order_manager
        .lock()
        .await
        .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Both)
        .await
        .map_err(|error| error.to_string())?;
    let rest = query.client();
    let mut synced = 0usize;
    let domestic = rest
        .get_balance()
        .await
        .map_err(|e| format!("KIS 국내 잔고 조회 실패: {e}"))?;
    let overseas = rest
        .get_overseas_balance()
        .await
        .map_err(|e| format!("KIS 해외 잔고 조회 실패: {e}"))?;
    for item in &domestic.items {
        item.hldg_qty
            .trim()
            .replace(',', "")
            .parse::<u64>()
            .map_err(|_| format!("{} KIS 보유수량 응답 형식 오류", item.pdno))?;
        item.pchs_avg_pric
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("{} KIS 평균매입가 응답 형식 오류", item.pdno))?;
        item.prpr
            .trim()
            .replace(',', "")
            .parse::<u64>()
            .map_err(|_| format!("{} KIS 현재가 응답 형식 오류", item.pdno))?;
    }
    for item in &overseas.items {
        item.ovrs_cblc_qty
            .trim()
            .replace(',', "")
            .parse::<u64>()
            .map_err(|_| format!("{} KIS 해외 보유수량 응답 형식 오류", item.ovrs_pdno))?;
        item.pchs_avg_pric
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("{} KIS 해외 평균매입가 응답 형식 오류", item.ovrs_pdno))?;
        item.now_pric2
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("{} KIS 해외 현재가 응답 형식 오류", item.ovrs_pdno))?;
    }
    let domestic_total = domestic
        .summary
        .as_ref()
        .and_then(|summary| {
            summary
                .tot_evlu_amt
                .trim()
                .replace(',', "")
                .parse::<i64>()
                .ok()
        })
        .unwrap_or(0);
    let exchange_rate = *s.exchange_rate_krw.read().await;
    let overseas_total = overseas
        .summary
        .as_ref()
        .and_then(|summary| {
            summary
                .frcr_evlu_tota
                .trim()
                .replace(',', "")
                .parse::<f64>()
                .ok()
        })
        .map(|value| (value * exchange_rate).round() as i64)
        .unwrap_or(0);
    let domestic_positions: Vec<_> = domestic
        .items
        .iter()
        .map(|item| {
            (
                item.pdno.clone(),
                item.prdt_name.clone(),
                normalized_u64(&item.hldg_qty),
                normalized_f64(&item.pchs_avg_pric).round() as u64,
                normalized_u64(&item.prpr),
            )
        })
        .collect();
    let overseas_positions: Vec<_> = overseas
        .items
        .iter()
        .map(|item| {
            (
                item.ovrs_pdno.clone(),
                item.ovrs_item_name.clone(),
                crate::trading::order::normalize_overseas_order_exchange(&item.ovrs_excg_cd),
                normalized_u64(&item.ovrs_cblc_qty),
                money_units(&item.pchs_avg_pric, &BrokerCurrency::Usd),
                money_units(&item.now_pric2, &BrokerCurrency::Usd),
            )
        })
        .collect();
    s.stock_store
        .upsert_many(
            domestic
                .items
                .iter()
                .map(|item| (item.pdno.clone(), item.prdt_name.clone())),
        )
        .await;
    let mut mgr = s.strategy_manager.lock().await;
    let mut orders = s.order_manager.lock().await;
    orders
        .publish_holdings(
            query,
            HoldingsUpdate::Both {
                domestic: domestic_positions.clone(),
                overseas: overseas_positions.clone(),
            },
        )
        .await
        .map_err(|error| error.to_string())?;
    for symbol in mgr.active_symbols() {
        if crate::market_hours::is_domestic_symbol(&symbol) {
            mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                broker_id: BrokerId::Kis,
                market: BrokerMarket::Kr,
                symbol,
                quantity: 0,
                avg_price: 0,
            });
        }
    }
    for (symbol, _, qty, avg, _) in &domestic_positions {
        if *qty > 0 {
            synced += 1;
        }
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Kis,
            market: BrokerMarket::Kr,
            symbol: symbol.clone(),
            quantity: *qty,
            avg_price: *avg,
        });
    }
    for symbol in mgr.active_symbols() {
        if !crate::market_hours::is_domestic_symbol(&symbol) {
            mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                broker_id: BrokerId::Kis,
                market: BrokerMarket::Us,
                symbol,
                quantity: 0,
                avg_price: 0,
            });
        }
    }
    for (symbol, _, _, qty, avg, _) in &overseas_positions {
        if *qty > 0 {
            synced += 1;
        }
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Kis,
            market: BrokerMarket::Us,
            symbol: symbol.clone(),
            quantity: *qty,
            avg_price: *avg,
        });
    }
    if domestic_total.saturating_add(overseas_total) <= 0 {
        return Err("KIS 계좌 총평가액이 0원 이하입니다.".into());
    }
    Ok(synced)
}

async fn sync_toss_strategy_positions(
    s: &ServerState,
    profile: AccountProfile,
) -> Result<usize, String> {
    if !profile.is_configured() {
        return Err("활성 Toss 프로파일 설정이 미완료입니다.".into());
    }
    let client = s.rest_client.read().await.clone();
    let scope = BrokerScope::new(
        BrokerId::Toss,
        Some(BrokerAccountId(profile.broker_account_id())),
    );
    let query = s
        .order_manager
        .lock()
        .await
        .begin_holdings_query_for_snapshot(&scope, &client, Some(&profile), HoldingsMarkets::Both)
        .await
        .map_err(|error| error.to_string())?;
    let profile = query
        .profile()
        .cloned()
        .ok_or_else(|| "활성 Toss 프로파일이 없습니다.".to_string())?;
    let account_seq = profile.broker_account_id();
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(account_seq.clone()),
    );
    let account_id = BrokerAccountId(account_seq);
    let holdings = adapter
        .list_holdings(Some(&account_id))
        .await
        .map_err(|e| format!("Toss 보유종목 조회 실패: {e}"))?;
    for holding in &holdings {
        holding
            .quantity
            .0
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("{} Toss 보유수량 응답 형식 오류", holding.symbol.0))?;
        holding
            .average_price
            .amount
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("{} Toss 평균매입가 응답 형식 오류", holding.symbol.0))?;
        holding
            .current_price
            .amount
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("{} Toss 현재가 응답 형식 오류", holding.symbol.0))?;
    }
    let exchange_rate = *s.exchange_rate_krw.read().await;
    let mut total_krw = 0.0;
    for currency in [BrokerCurrency::Krw, BrokerCurrency::Usd] {
        let power = adapter
            .get_buying_power(Some(&account_id.0), currency)
            .await
            .map_err(|e| format!("Toss {currency:?} 매수가능금액 조회 실패: {e}"))?;
        let value = power
            .cash_buying_power
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("Toss {currency:?} 매수가능금액 형식 오류"))?;
        total_krw += if currency == BrokerCurrency::Usd {
            value * exchange_rate
        } else {
            value
        };
    }
    s.stock_store
        .upsert_many(
            holdings
                .iter()
                .map(|holding| (holding.symbol.0.clone(), holding.symbol_name.clone())),
        )
        .await;
    let domestic_positions = holdings
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
        })
        .collect();
    let overseas_positions = holdings
        .iter()
        .filter(|holding| holding.market == BrokerMarket::Us)
        .map(|holding| {
            (
                holding.symbol.0.clone(),
                holding.symbol_name.clone(),
                "TOSS_US".to_string(),
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
        })
        .collect();
    let mut synced = 0usize;
    let mut mgr = s.strategy_manager.lock().await;
    let mut orders = s.order_manager.lock().await;
    orders
        .publish_holdings(
            query,
            HoldingsUpdate::Both {
                domestic: domestic_positions,
                overseas: overseas_positions,
            },
        )
        .await
        .map_err(|error| error.to_string())?;
    for symbol in mgr.active_symbols() {
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Toss,
            market: if crate::market_hours::is_domestic_symbol(&symbol) {
                BrokerMarket::Kr
            } else {
                BrokerMarket::Us
            },
            symbol,
            quantity: 0,
            avg_price: 0,
        });
    }
    for holding in &holdings {
        let quantity = decimal_quantity_units(&holding.quantity.0);
        if quantity > 0 {
            synced += 1;
        }
        let holding_value = quantity as f64
            * holding
                .current_price
                .amount
                .trim()
                .replace(',', "")
                .parse::<f64>()
                .map_err(|_| format!("Toss {} 현재가 형식 오류", holding.symbol.0))?;
        total_krw += if holding.current_price.currency == BrokerCurrency::Usd {
            holding_value * exchange_rate
        } else {
            holding_value
        };
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
    if total_krw <= 0.0 {
        return Err("Toss 계좌 총평가액이 0원 이하입니다.".into());
    }
    Ok(synced)
}

fn decimal_quantity_units(value: &str) -> u64 {
    let parsed = normalized_f64(value);
    if parsed <= 0.0 {
        0
    } else {
        parsed.floor().max(1.0) as u64
    }
}

fn money_units(value: &str, currency: &crate::broker::BrokerCurrency) -> u64 {
    let parsed = normalized_f64(value);
    match currency {
        crate::broker::BrokerCurrency::Krw => parsed.round().max(0.0) as u64,
        crate::broker::BrokerCurrency::Usd => (parsed.max(0.0) * 100.0).round() as u64,
    }
}

fn normalized_u64(value: &str) -> u64 {
    value.trim().replace(',', "").parse::<u64>().unwrap_or(0)
}

fn normalized_f64(value: &str) -> f64 {
    value.trim().replace(',', "").parse::<f64>().unwrap_or(0.0)
}

/// POST /api/trading/start — is_trading = true (폴링 데몬이 자동으로 재개)
pub(super) async fn trading_start_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let _storage_maintenance = s.storage_maintenance.lock().await;
    let _strategy_update = s.strategy_update_lock.lock().await;
    if s.database_manager.config_view().await.active_backend
        == crate::storage::database::StorageBackend::Database
    {
        if let Err(error) = s.database_manager.status().await {
            return Json(serde_json::json!({
                "ok": false,
                "code": "STORAGE_UNAVAILABLE",
                "message": format!("DB 저장소를 확인할 수 없어 자동매매를 시작하지 않습니다: {error}")
            }));
        }
    }
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
    let synced_positions = match synced_positions {
        Ok(count) => count,
        Err(message) => {
            return Json(serde_json::json!({
                "ok": false,
                "code": "ACCOUNT_SYNC_FAILED",
                "message": format!("자동매매 시작 전 계좌 동기화 실패: {message}")
            }));
        }
    };
    let reconciliation_account_id = active_profile
        .as_ref()
        .map(|profile| profile.broker_account_id())
        .or_else(|| {
            (!current_cfg.broker_account_id.is_empty())
                .then(|| current_cfg.broker_account_id.clone())
        });
    let execution_scope = BrokerScope::new(
        active_profile
            .as_ref()
            .map(|profile| profile.broker_id)
            .unwrap_or(current_cfg.broker_id),
        reconciliation_account_id.map(BrokerAccountId),
    );
    s.order_manager
        .lock()
        .await
        .set_execution_scope(execution_scope.clone());
    if let Err(error) =
        crate::trading::order::OrderManager::confirm_pending_fills_from_broker_shared(
            &s.order_manager,
        )
        .await
    {
        return Json(serde_json::json!({
            "ok": false,
            "code": "PENDING_RECONCILIATION_FAILED",
            "message": format!("복원된 미체결 주문을 broker와 대조하지 못했습니다: {error}")
        }));
    }
    let order_store = s.order_manager.lock().await.order_store_handle();
    if let Err(error) = crate::commands::restore_risk_from_today_trades(
        &order_store,
        &s.trade_store,
        &s.risk_manager,
        &s.risk_store,
        &execution_scope,
    )
    .await
    {
        return Json(serde_json::json!({
            "ok": false,
            "code": error.code,
            "message": error.message
        }));
    }
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

    let mut is_running = s.is_trading.lock().await;
    crate::commands::initialize_active_strategy_history(
        &s.strategy_manager,
        &s.order_manager,
        &s.profiles,
        &s.rest_client,
        &s.risk_manager,
    )
    .await;
    let rest_snapshot = s.rest_client.read().await.clone();
    {
        let history_scope = s.order_manager.lock().await.execution_scope().clone();
        let mut manager = s.strategy_manager.lock().await;
        manager.advance_history_generation();
        if let Err(error) = s.order_manager.lock().await.begin_execution_run(
            history_scope.clone(),
            active_id.clone(),
            manager.history_generation(),
            rest_snapshot,
            active_profile,
        ) {
            return Json(serde_json::json!({
                "ok": false,
                "code": "EXECUTION_CONTEXT_CHANGED",
                "message": error.to_string()
            }));
        }
        manager.set_history_execution_identity(history_scope, active_id.clone());
    }
    *is_running = true;
    drop(is_running);
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
    {
        let mut is_running = s.is_trading.lock().await;
        *is_running = false;
        let mut manager = s.strategy_manager.lock().await;
        manager.advance_history_generation();
        manager.clear_history_execution_identity();
        s.order_manager.lock().await.end_execution_run();
    }
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
    expected_profile_id: String,
    expected_broker_id: crate::broker::BrokerId,
    expected_broker_account_id: Option<String>,
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
) -> (StatusCode, Json<serde_json::Value>) {
    use crate::trading::configuration::{
        update_strategy_configuration, ConfigurationUpdateRequest, ConfigurationUpdateState,
    };
    let updated_config = match update_strategy_configuration(
        ConfigurationUpdateState {
            update_lock: s.strategy_update_lock.clone(),
            strategies: s.strategy_manager.clone(),
            orders: s.order_manager.clone(),
            store: s.strategy_store.clone(),
        },
        ConfigurationUpdateRequest {
            id,
            expected_profile_id: body.expected_profile_id,
            expected_scope: BrokerScope::new(
                body.expected_broker_id,
                body.expected_broker_account_id.map(BrokerAccountId),
            ),
            enabled: body.enabled,
            target_symbols: body.target_symbols,
            order_quantity: body.order_quantity,
            params: body.params,
        },
    )
    .await
    {
        Ok(config) => config,
        Err(error) => {
            let status = match error.code {
                "SCOPE_MISMATCH"
                | "CONFIGURATION_BUSY"
                | "HELD_TARGET_REMOVAL"
                | "HELD_MARKET_CHANGE" => StatusCode::CONFLICT,
                "STRATEGY_NOT_FOUND" => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            return (
                status,
                Json(serde_json::json!({ "code": error.code, "error": error.message })),
            );
        }
    };
    let mut symbol_names = std::collections::HashMap::new();
    for code in &updated_config.target_symbols {
        symbol_names.insert(
            code.clone(),
            s.stock_store
                .get_name(code)
                .await
                .unwrap_or_else(|| code.clone()),
        );
    }

    (
        StatusCode::OK,
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
        })),
    )
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
