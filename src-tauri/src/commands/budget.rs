use super::*;
use crate::trading::order::AutoTradingBudgetView;

/// Explicit scope prevents a stale settings screen from funding another account.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTradingBudgetInput {
    pub broker_id: BrokerId,
    pub broker_account_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAutoTradingBudgetInput {
    pub broker_id: BrokerId,
    pub broker_account_id: String,
    pub krw_amount: u64,
    /// USD cents, never a floating point dollar amount.
    pub usd_amount: u64,
}

pub(crate) async fn checked_budget_scope(
    profiles: &Arc<RwLock<ProfilesConfig>>,
    broker_id: BrokerId,
    broker_account_id: &str,
) -> CmdResult<BrokerScope> {
    let profiles = profiles.read().await;
    let profile = profiles.get_active().ok_or_else(|| CmdError {
        code: "BUDGET_SCOPE_REQUIRED".into(),
        message: "자동매매 예산을 설정할 계좌를 먼저 선택하세요.".into(),
    })?;
    if broker_account_id.trim().is_empty()
        || profile.broker_id != broker_id
        || profile.broker_account_id() != broker_account_id
    {
        return Err(CmdError {
            code: "BUDGET_SCOPE_CHANGED".into(),
            message: "선택 계좌가 변경되었습니다. 예산 화면을 새로 불러오세요.".into(),
        });
    }
    Ok(BrokerScope::new(
        broker_id,
        Some(BrokerAccountId(broker_account_id.to_string())),
    ))
}

pub(crate) fn budget_error(error: anyhow::Error) -> CmdError {
    CmdError {
        code: "AUTO_TRADING_BUDGET_ERROR".into(),
        message: error.to_string(),
    }
}

#[tauri::command]
pub async fn get_auto_trading_budget(
    input: AutoTradingBudgetInput,
    state: State<'_, AppState>,
) -> CmdResult<AutoTradingBudgetView> {
    let _profile_change = state.strategy_update_lock.lock().await;
    let scope =
        checked_budget_scope(&state.profiles, input.broker_id, &input.broker_account_id).await?;
    state
        .order_manager
        .lock()
        .await
        .auto_trading_budget(&scope)
        .await
        .map_err(budget_error)
}

#[tauri::command]
pub async fn update_auto_trading_budget(
    input: UpdateAutoTradingBudgetInput,
    state: State<'_, AppState>,
) -> CmdResult<AutoTradingBudgetView> {
    let _maintenance = state.storage_maintenance.lock().await;
    let _profile_change = state.strategy_update_lock.lock().await;
    if *state.is_trading.lock().await {
        return Err(CmdError {
            code: "TRADING_RUNNING".into(),
            message: "자동매매를 정지한 후 예산을 변경하세요.".into(),
        });
    }
    let scope =
        checked_budget_scope(&state.profiles, input.broker_id, &input.broker_account_id).await?;
    state
        .order_manager
        .lock()
        .await
        .update_auto_trading_budget(&scope, input.krw_amount, input.usd_amount)
        .await
        .map_err(budget_error)
}
