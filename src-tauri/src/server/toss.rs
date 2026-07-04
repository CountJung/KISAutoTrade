use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use super::ServerState;
use crate::config::AccountProfile;

async fn active_profile(s: &ServerState) -> Result<AccountProfile, Json<serde_json::Value>> {
    let profile = {
        let profiles = s.profiles.read().await;
        profiles.get_active().cloned()
    };
    profile.ok_or_else(|| {
        Json(serde_json::json!({
            "code": "CONFIG_NOT_READY",
            "error": "활성 프로파일이 없습니다.",
        }))
    })
}

pub(super) async fn toss_market_snapshot_handler(
    State(s): State<ServerState>,
    Path(symbol): Path<String>,
) -> Json<serde_json::Value> {
    let profile = match active_profile(&s).await {
        Ok(profile) => profile,
        Err(response) => return response,
    };

    match crate::commands::get_toss_market_snapshot_for_profile(symbol, profile).await {
        Ok(snapshot) => Json(serde_json::to_value(snapshot).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({
            "code": e.code,
            "error": e.message,
        })),
    }
}

pub(super) async fn toss_stock_safety_handler(
    State(s): State<ServerState>,
    Path(symbol): Path<String>,
) -> Json<serde_json::Value> {
    let profile = match active_profile(&s).await {
        Ok(profile) => profile,
        Err(response) => return response,
    };

    match crate::commands::get_toss_stock_safety_for_profile(symbol, profile).await {
        Ok(safety) => Json(serde_json::to_value(safety).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({
            "code": e.code,
            "error": e.message,
        })),
    }
}

pub(super) async fn toss_order_preflight_handler(
    State(s): State<ServerState>,
    Json(input): Json<crate::commands::TossOrderPreflightInput>,
) -> Json<serde_json::Value> {
    let profile = match active_profile(&s).await {
        Ok(profile) => profile,
        Err(response) => return response,
    };

    match crate::commands::check_toss_order_preflight_for_profile(input, profile).await {
        Ok(preflight) => Json(serde_json::to_value(preflight).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({
            "code": e.code,
            "error": e.message,
        })),
    }
}

pub(super) async fn toss_market_calendar_handler(
    State(s): State<ServerState>,
) -> Json<serde_json::Value> {
    let profile = match active_profile(&s).await {
        Ok(profile) => profile,
        Err(response) => return response,
    };

    match crate::commands::get_toss_market_calendar_for_profile(profile).await {
        Ok(calendar) => Json(serde_json::to_value(calendar).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({
            "code": e.code,
            "error": e.message,
        })),
    }
}

#[derive(Deserialize)]
pub(super) struct TossChartQuery {
    interval: Option<String>,
    count: Option<u16>,
}

pub(super) async fn toss_chart_handler(
    State(s): State<ServerState>,
    Path(symbol): Path<String>,
    Query(params): Query<TossChartQuery>,
) -> Json<serde_json::Value> {
    let profile = match active_profile(&s).await {
        Ok(profile) => profile,
        Err(response) => return response,
    };

    let interval = params.interval.unwrap_or_else(|| "1d".to_string());
    match crate::commands::get_toss_chart_data_for_profile(symbol, interval, params.count, profile)
        .await
    {
        Ok(candles) => Json(serde_json::to_value(candles).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({
            "code": e.code,
            "error": e.message,
        })),
    }
}
