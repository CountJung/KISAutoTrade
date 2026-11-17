use crate::trading::sector_data::ValidateSectorDataInput;
use axum::{http::StatusCode, Json};

/// Same pure command as desktop; existing POST token/Origin/body limits apply.
pub(super) async fn validate_handler(
    Json(input): Json<ValidateSectorDataInput>,
) -> (StatusCode, Json<serde_json::Value>) {
    match crate::commands::validate_sector_data(input).await {
        Ok(report) => (StatusCode::OK, Json(serde_json::json!(report))),
        Err(error) => (
            if error.code == "SECTOR_DATA_VALIDATION_FAILED" {
                StatusCode::INTERNAL_SERVER_ERROR
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            },
            Json(serde_json::json!({"code":error.code,"error":error.message})),
        ),
    }
}

#[cfg(test)]
mod tests;
