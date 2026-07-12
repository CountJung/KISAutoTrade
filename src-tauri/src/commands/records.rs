use super::*;

// ────────────────────────────────────────────────────────────────────
// 당일 체결 내역 (KIS 실시간)
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_executed(state: State<'_, AppState>) -> CmdResult<Vec<ExecutedOrder>> {
    let client = state.rest_client.read().await.clone();
    client
        .get_today_executed_orders()
        .await
        .map_err(CmdError::from)
}

#[tauri::command]
pub async fn get_today_overseas_executed(
    state: State<'_, AppState>,
) -> CmdResult<Vec<OverseasExecutedOrder>> {
    let client = state.rest_client.read().await.clone();
    client
        .get_today_overseas_executed_orders()
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 로컬 체결 기록 (JSON 저장소)
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_trades(state: State<'_, AppState>) -> CmdResult<Vec<TradeRecord>> {
    let today = chrono::Local::now().date_naive();
    state
        .trade_store
        .get_by_date(today)
        .await
        .map_err(CmdError::from)
}

#[derive(Debug, Deserialize)]
pub struct GetTradesByRangeInput {
    pub from: String,
    pub to: String,
}

#[tauri::command]
pub async fn get_trades_by_range(
    input: GetTradesByRangeInput,
    state: State<'_, AppState>,
) -> CmdResult<Vec<TradeRecord>> {
    use chrono::NaiveDate;
    let from = NaiveDate::parse_from_str(&input.from, "%Y-%m-%d").map_err(|e| CmdError {
        code: "INVALID_DATE".into(),
        message: format!("from 날짜 형식 오류: {}", e),
    })?;
    let to = NaiveDate::parse_from_str(&input.to, "%Y-%m-%d").map_err(|e| CmdError {
        code: "INVALID_DATE".into(),
        message: format!("to 날짜 형식 오류: {}", e),
    })?;
    state
        .trade_store
        .get_by_range(from, to)
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 일별 통계
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_stats(state: State<'_, AppState>) -> CmdResult<DailyStats> {
    let today = chrono::Local::now().date_naive();
    state
        .stats_store
        .get_by_date(today)
        .await
        .map_err(CmdError::from)
}

#[derive(Debug, Deserialize)]
pub struct GetStatsByRangeInput {
    pub from: String,
    pub to: String,
}

#[tauri::command]
pub async fn get_stats_by_range(
    input: GetStatsByRangeInput,
    state: State<'_, AppState>,
) -> CmdResult<Vec<DailyStats>> {
    use chrono::NaiveDate;
    let from = NaiveDate::parse_from_str(&input.from, "%Y-%m-%d").map_err(|e| CmdError {
        code: "INVALID_DATE".into(),
        message: format!("from 날짜 형식 오류: {}", e),
    })?;
    let to = NaiveDate::parse_from_str(&input.to, "%Y-%m-%d").map_err(|e| CmdError {
        code: "INVALID_DATE".into(),
        message: format!("to 날짜 형식 오류: {}", e),
    })?;
    state
        .stats_store
        .get_by_range(from, to)
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// Discord 테스트 알림
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn send_test_discord(state: State<'_, AppState>) -> CmdResult<String> {
    match &state.discord {
        None => Err(CmdError {
            code: "DISCORD_NOT_CONFIGURED".into(),
            message: "Discord 봇이 설정되지 않았습니다. secure_config.json을 확인하세요.".into(),
        }),
        Some(notifier) => {
            let event = NotificationEvent::info(
                "테스트 알림".to_string(),
                "AutoConditionTrade 알림 시스템이 정상 작동 중입니다.".to_string(),
            );
            notifier.send(event).await.map_err(CmdError::from)?;
            Ok("Discord 테스트 알림 전송 완료".into())
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 체결 기록 저장
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SaveTradeInput {
    pub symbol: String,
    pub symbol_name: String,
    pub side: String,
    pub quantity: u64,
    pub price: u64,
    pub fee: u64,
    pub order_id: String,
    pub strategy_id: Option<String>,
}

#[tauri::command]
pub async fn save_trade(
    input: SaveTradeInput,
    state: State<'_, AppState>,
) -> CmdResult<TradeRecord> {
    use crate::storage::trade_store::TradeSide;

    let side = match input.side.as_str() {
        "buy" | "Buy" => TradeSide::Buy,
        "sell" | "Sell" => TradeSide::Sell,
        other => {
            return Err(CmdError {
                code: "INVALID_SIDE".into(),
                message: format!("알 수 없는 방향: {}", other),
            })
        }
    };

    let active_scope = {
        let profiles = state.profiles.read().await;
        let config = state.config.read().await.clone();
        profiles
            .get_active()
            .map(|profile| {
                BrokerScope::new(
                    profile.broker_id,
                    Some(BrokerAccountId(profile.broker_account_id())),
                )
            })
            .unwrap_or_else(|| {
                BrokerScope::new(
                    config.broker_id,
                    (!config.broker_account_id.is_empty())
                        .then(|| BrokerAccountId(config.broker_account_id.clone())),
                )
            })
    };
    let record = TradeRecord::new(
        input.symbol,
        input.symbol_name,
        side.clone(),
        input.quantity,
        input.price,
        input.fee,
        input.order_id,
        input.strategy_id,
        String::new(), // 수동 저장 시 signal_reason 없음
    )
    .with_broker_scope(&active_scope);

    state
        .trade_store
        .append(record.clone())
        .await
        .map_err(CmdError::from)?;

    if let Some(notifier) = &state.discord {
        let side_label = if side == TradeSide::Buy {
            "매수"
        } else {
            "매도"
        };
        let _ = notifier
            .send(NotificationEvent::trade(format!(
                "{} {} {}주 @{}원",
                record.symbol_name, side_label, record.quantity, record.price
            )))
            .await;
    }

    Ok(record)
}

// ────────────────────────────────────────────────────────────────────
// 통계 업데이트
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn upsert_daily_stats(stats: DailyStats, state: State<'_, AppState>) -> CmdResult<()> {
    state
        .stats_store
        .upsert(stats)
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 프론트엔드 로그 기록
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FrontendLogInput {
    /// "error" | "warn" | "info" | "debug"
    pub level: String,
    pub message: String,
    pub context: Option<String>,
}

#[tauri::command]
pub async fn write_frontend_log(input: FrontendLogInput) -> CmdResult<()> {
    let msg = if let Some(ctx) = &input.context {
        format!("[{}] {}", ctx, input.message)
    } else {
        input.message.clone()
    };
    match input.level.to_lowercase().as_str() {
        "error" => tracing::error!(target: "frontend", "{}", msg),
        "warn" => tracing::warn!(target: "frontend", "{}", msg),
        "debug" => tracing::debug!(target: "frontend", "{}", msg),
        _ => tracing::info!(target: "frontend", "{}", msg),
    }
    Ok(())
}

// ── KIS 기간별 체결 내역 ──────────────────────────────────────────
#[tauri::command]
pub async fn get_kis_executed_by_range(
    from: String, // YYYY-MM-DD
    to: String,   // YYYY-MM-DD
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::api::rest::ExecutedOrder>> {
    let from_fmt = from.replace('-', "");
    let to_fmt = to.replace('-', "");
    let client = state.rest_client.read().await.clone();
    client
        .get_executed_orders_range(&from_fmt, &to_fmt)
        .await
        .map_err(CmdError::from)
}

#[tauri::command]
pub async fn get_overseas_executed_by_range(
    from: String, // YYYY-MM-DD
    to: String,   // YYYY-MM-DD
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::api::rest::OverseasExecutedOrder>> {
    let from_fmt = from.replace('-', "");
    let to_fmt = to.replace('-', "");
    let client = state.rest_client.read().await.clone();
    client
        .get_overseas_executed_orders_range(&from_fmt, &to_fmt)
        .await
        .map_err(CmdError::from)
}

// ── 최근 로그 엔트리 (파일 기반) ──────────────────────────────────
#[tauri::command]
pub async fn get_recent_logs(
    count: u32,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::logging::LogEntry>> {
    let count = (count as usize).clamp(1, crate::logging::MAX_RECENT_LOG_COUNT);
    let log_dir = state.log_dir.clone();
    tokio::task::spawn_blocking(move || crate::logging::read_recent_entries(&log_dir, count))
        .await
        .map_err(|e| CmdError {
            code: "TASK_ERR".into(),
            message: e.to_string(),
        })
}
