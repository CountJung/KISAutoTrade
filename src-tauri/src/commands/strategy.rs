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
    pub expected_profile_id: String,
    pub expected_broker_id: crate::broker::BrokerId,
    pub expected_broker_account_id: Option<String>,
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
    use crate::trading::configuration::{
        update_strategy_configuration, ConfigurationUpdateRequest, ConfigurationUpdateState,
    };
    let updated_config = update_strategy_configuration(
        ConfigurationUpdateState {
            update_lock: state.strategy_update_lock.clone(),
            strategies: state.strategy_manager.clone(),
            orders: state.order_manager.clone(),
            store: state.strategy_store.clone(),
        },
        ConfigurationUpdateRequest {
            id: input.id,
            expected_profile_id: input.expected_profile_id,
            expected_scope: BrokerScope::new(
                input.expected_broker_id,
                input.expected_broker_account_id.map(BrokerAccountId),
            ),
            enabled: input.enabled,
            target_symbols: input.target_symbols,
            order_quantity: input.order_quantity,
            params: input.params,
        },
    )
    .await
    .map_err(|error| CmdError {
        code: error.code.into(),
        message: error.message,
    })?;
    Ok(build_strategy_view(&updated_config, &state.stock_store).await)
}
