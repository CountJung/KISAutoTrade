use super::*;
use crate::commands::{
    budget_error, checked_budget_scope, AutoTradingBudgetInput, UpdateAutoTradingBudgetInput,
};

pub(super) async fn budget_handler(
    State(s): State<ServerState>,
    axum::extract::Query(input): axum::extract::Query<AutoTradingBudgetInput>,
) -> Response {
    let _profile_change = s.strategy_update_lock.lock().await;
    let result = async {
        let scope =
            checked_budget_scope(&s.profiles, input.broker_id, &input.broker_account_id).await?;
        s.order_manager
            .lock()
            .await
            .auto_trading_budget(&scope)
            .await
            .map_err(budget_error)
    }
    .await;
    budget_response(result)
}

pub(super) async fn update_budget_handler(
    State(s): State<ServerState>,
    Json(input): Json<UpdateAutoTradingBudgetInput>,
) -> Response {
    let _maintenance = s.storage_maintenance.lock().await;
    let _profile_change = s.strategy_update_lock.lock().await;
    let result = async {
        if *s.is_trading.lock().await {
            return Err(crate::commands::CmdError {
                code: "TRADING_RUNNING".into(),
                message: "자동매매를 정지한 후 예산을 변경하세요.".into(),
            });
        }
        let scope =
            checked_budget_scope(&s.profiles, input.broker_id, &input.broker_account_id).await?;
        s.order_manager
            .lock()
            .await
            .update_auto_trading_budget(&scope, input.krw_amount, input.usd_amount)
            .await
            .map_err(budget_error)
    }
    .await;
    budget_response(result)
}

fn budget_response(
    result: Result<crate::trading::order::AutoTradingBudgetView, crate::commands::CmdError>,
) -> Response {
    match result {
        Ok(view) => Json(view).into_response(),
        Err(error) => (axum::http::StatusCode::BAD_REQUEST, Json(error)).into_response(),
    }
}
