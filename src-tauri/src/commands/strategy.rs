use super::*;

// ────────────────────────────────────────────────────────────────────
// 포지션 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionView {
    pub symbol: String,
    pub symbol_name: String,
    pub quantity: u64,
    pub avg_price: f64,
    pub current_price: u64,
    pub unrealized_pnl: i64,
    pub unrealized_pnl_rate: f64,
}

impl From<&Position> for PositionView {
    fn from(p: &Position) -> Self {
        Self {
            symbol: p.symbol.clone(),
            symbol_name: p.symbol_name.clone(),
            quantity: p.quantity,
            avg_price: p.avg_price,
            current_price: p.current_price,
            unrealized_pnl: p.unrealized_pnl(),
            unrealized_pnl_rate: p.unrealized_pnl_rate(),
        }
    }
}

#[tauri::command]
pub async fn get_positions(state: State<'_, AppState>) -> CmdResult<Vec<PositionView>> {
    let tracker = state.position_tracker.lock().await;
    let mut positions: Vec<PositionView> = tracker
        .all()
        .iter()
        .map(|p| PositionView::from(*p))
        .collect();
    positions.sort_by_key(|position| std::cmp::Reverse(position.quantity));
    Ok(positions)
}

// ────────────────────────────────────────────────────────────────────
// 전략 관리
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_strategies(state: State<'_, AppState>) -> CmdResult<Vec<StrategyView>> {
    let configs: Vec<StrategyConfig> = {
        let mgr = state.strategy_manager.lock().await;
        mgr.all_configs().into_iter().cloned().collect()
    };
    let mut views = Vec::with_capacity(configs.len());
    for cfg in &configs {
        views.push(build_strategy_view(cfg, &state.stock_store).await);
    }
    Ok(views)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStrategyInput {
    pub id: String,
    pub enabled: Option<bool>,
    pub target_symbols: Option<Vec<String>>,
    pub order_quantity: Option<u64>,
    pub params: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn update_strategy(
    input: UpdateStrategyInput,
    state: State<'_, AppState>,
) -> CmdResult<StrategyView> {
    let _strategy_update = state.strategy_update_lock.lock().await;
    let active_scope = {
        let cfg = state.config.read().await.clone();
        let account_id = if cfg.broker_account_id.is_empty() {
            None
        } else {
            Some(cfg.broker_account_id.clone())
        };
        (cfg.broker_id, account_id)
    };
    let previous_configs: Vec<crate::trading::strategy::StrategyConfig> = {
        let mgr = state.strategy_manager.lock().await;
        mgr.all_configs().into_iter().cloned().collect()
    };
    let updated_config = {
        let mut mgr = state.strategy_manager.lock().await;
        mgr.update_config(&input.id, |cfg| {
            if let Some(enabled) = input.enabled {
                cfg.enabled = enabled;
            }
            if let Some(symbols) = input.target_symbols {
                cfg.target_symbols = symbols;
            }
            if let Some(qty) = input.order_quantity {
                cfg.order_quantity = qty;
            }
            if let Some(params) = input.params {
                cfg.params = params;
            }
            cfg.set_scope(active_scope.0, active_scope.1.clone());
        })
        .ok_or_else(|| CmdError {
            code: "STRATEGY_NOT_FOUND".into(),
            message: format!("전략을 찾을 수 없습니다: {}", input.id),
        })?
    };
    let view = build_strategy_view(&updated_config, &state.stock_store).await;

    // 변경된 전략 설정을 디스크에 영구 저장 (프로파일별)
    let profile_id = state.profiles.read().await.active_id.clone();
    if let Some(pid) = &profile_id {
        let all_configs: Vec<crate::trading::strategy::StrategyConfig> = {
            let mgr = state.strategy_manager.lock().await;
            mgr.all_configs().into_iter().cloned().collect()
        };
        if let Err(error) = state.strategy_store.save(pid, &all_configs).await {
            state
                .strategy_manager
                .lock()
                .await
                .apply_saved_configs_for_scope(
                    &previous_configs,
                    active_scope.0,
                    active_scope.1.clone(),
                );
            return Err(CmdError {
                code: "PERSISTENCE_ERROR".into(),
                message: format!("전략 설정을 저장하지 못했습니다: {error}"),
            });
        }
    }

    Ok(view)
}
