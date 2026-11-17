use crate::trading::sector_portfolio::SectorPortfolioInput;
use axum::{http::StatusCode, Json};

/// Pure raw-chart approximation only; no authenticated quote collection on REST.
pub(super) async fn broker_chart_preview_handler(
    Json(input): Json<crate::trading::sector_portfolio::BrokerChartInput>,
) -> (StatusCode, Json<serde_json::Value>) {
    match crate::commands::preview_broker_chart_portfolio(input).await {
        Ok(report) => (StatusCode::OK, Json(serde_json::json!(report))),
        Err(error) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"code":error.code,"error":error.message})),
        ),
    }
}

/// Uses the exact desktop calculation and the router's existing POST security.
pub(super) async fn preview_handler(
    Json(input): Json<SectorPortfolioInput>,
) -> (StatusCode, Json<serde_json::Value>) {
    match crate::commands::calculate_sector_portfolio(input).await {
        Ok(report) => (StatusCode::OK, Json(serde_json::json!(report))),
        Err(error) => (
            if error.code == "INVALID_SECTOR_PORTFOLIO_INPUT" {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            },
            Json(serde_json::json!({"code":error.code,"error":error.message})),
        ),
    }
}

#[cfg(test)]
mod tests;
