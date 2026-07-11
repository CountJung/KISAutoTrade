//! Tauri-only database administration commands.
//!
//! These commands intentionally have no Axum route while the LAN web server has no admin
//! authentication. Passwords and destructive database actions therefore stay in desktop IPC.

use super::*;
use crate::storage::database::{
    DatabaseConfigView, DatabaseStatusView, DatabaseTransferResult, JsonStorageInventoryView,
    SaveDatabaseConfigInput, StorageBackend,
};

fn database_error(error: anyhow::Error) -> CmdError {
    CmdError {
        code: "DATABASE_ERROR".to_string(),
        message: error.to_string(),
    }
}

#[tauri::command]
pub async fn get_database_config(state: State<'_, AppState>) -> CmdResult<DatabaseConfigView> {
    Ok(state.database_manager.config_view().await)
}

#[tauri::command]
pub async fn save_database_config(
    input: SaveDatabaseConfigInput,
    state: State<'_, AppState>,
) -> CmdResult<DatabaseConfigView> {
    let _maintenance = state.storage_maintenance.lock().await;
    if *state.is_trading.lock().await {
        return Err(CmdError {
            code: "TRADING_ACTIVE".to_string(),
            message: "자동매매를 정지한 뒤 DB 연결 설정을 변경하세요.".to_string(),
        });
    }
    state
        .database_manager
        .save_config(input)
        .await
        .map_err(database_error)
}

#[tauri::command]
pub async fn test_database_connection(state: State<'_, AppState>) -> CmdResult<DatabaseStatusView> {
    state
        .database_manager
        .status()
        .await
        .map_err(database_error)
}

#[tauri::command]
pub async fn create_database_tables(state: State<'_, AppState>) -> CmdResult<DatabaseStatusView> {
    let _maintenance = state.storage_maintenance.lock().await;
    if *state.is_trading.lock().await {
        return Err(CmdError {
            code: "TRADING_ACTIVE".to_string(),
            message: "자동매매를 정지한 뒤 DB 테이블을 생성하세요.".to_string(),
        });
    }
    state
        .database_manager
        .create_tables()
        .await
        .map_err(database_error)
}

#[tauri::command]
pub async fn clear_database_tables(
    confirmation: String,
    state: State<'_, AppState>,
) -> CmdResult<DatabaseStatusView> {
    let _maintenance = state.storage_maintenance.lock().await;
    ensure_trading_stopped(&state, "DB 데이터를 비우려면 자동매매를 먼저 정지하세요.").await?;
    state
        .database_manager
        .clear_tables(&confirmation)
        .await
        .map_err(database_error)
}

#[tauri::command]
pub async fn drop_database_tables(
    confirmation: String,
    state: State<'_, AppState>,
) -> CmdResult<DatabaseStatusView> {
    let _maintenance = state.storage_maintenance.lock().await;
    ensure_trading_stopped(&state, "DB 테이블을 삭제하려면 자동매매를 먼저 정지하세요.").await?;
    state
        .database_manager
        .drop_tables(&confirmation)
        .await
        .map_err(database_error)
}

#[tauri::command]
pub async fn inspect_json_storage(
    state: State<'_, AppState>,
) -> CmdResult<JsonStorageInventoryView> {
    state
        .database_manager
        .json_inventory()
        .await
        .map_err(database_error)
}

#[tauri::command]
pub async fn import_json_to_database(
    state: State<'_, AppState>,
) -> CmdResult<DatabaseTransferResult> {
    let _maintenance = state.storage_maintenance.lock().await;
    ensure_trading_stopped(
        &state,
        "JSON 데이터를 DB로 옮기려면 자동매매를 먼저 정지하세요.",
    )
    .await?;
    state
        .database_manager
        .import_json()
        .await
        .map_err(database_error)
}

#[tauri::command]
pub async fn export_database_to_json(
    state: State<'_, AppState>,
) -> CmdResult<DatabaseTransferResult> {
    let _maintenance = state.storage_maintenance.lock().await;
    ensure_trading_stopped(&state, "DB 데이터를 반출하려면 자동매매를 먼저 정지하세요.").await?;
    state
        .database_manager
        .export_json()
        .await
        .map_err(database_error)
}

#[tauri::command]
pub async fn set_storage_backend(
    backend: StorageBackend,
    state: State<'_, AppState>,
) -> CmdResult<DatabaseStatusView> {
    let _maintenance = state.storage_maintenance.lock().await;
    ensure_trading_stopped(
        &state,
        "저장 backend를 변경하려면 자동매매를 먼저 정지하세요.",
    )
    .await?;
    state
        .database_manager
        .set_backend(backend)
        .await
        .map_err(database_error)
}

async fn ensure_trading_stopped(state: &AppState, message: &str) -> CmdResult<()> {
    if *state.is_trading.lock().await {
        Err(CmdError {
            code: "TRADING_ACTIVE".to_string(),
            message: message.to_string(),
        })
    } else {
        Ok(())
    }
}
