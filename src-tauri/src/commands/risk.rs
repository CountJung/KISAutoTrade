use super::*;

// ────────────────────────────────────────────────────────────────────
// 리스크 관리 설정 조회 / 변경 / 비상 정지 해제
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskConfigView {
    /// 리스크 관리 활성화 여부
    pub enabled: bool,
    /// 일일 최대 순손실 한도 (원, 양수)
    pub daily_loss_limit: i64,
    /// 단일 종목 최대 비중 (0.0~1.0)
    pub max_position_ratio: f64,
    /// 전략/종목별 일일 매수 주문 제한. 매수 제한은 더 이상 차단 조건으로 사용하지 않으며 항상 0으로 노출한다.
    pub max_daily_buy_orders_per_symbol: u32,
    /// 전략/종목별 일일 매도 주문 제한. 0이면 제한 없음.
    pub max_daily_sell_orders_per_symbol: u32,
    /// 전략/종목별 연속 손실 차단 기준. 0이면 제한 없음.
    pub max_consecutive_losses_per_strategy_symbol: u32,
    /// ATR 기반 주문 수량 산정 활성화 여부
    pub volatility_sizing_enabled: bool,
    /// 거래당 허용 위험 한도(bps). 100 = 1%.
    pub risk_per_trade_bps: u32,
    /// ATR 손절폭 배수
    pub atr_stop_multiplier: f64,
    /// ATR이 준비된 종목 수
    pub atr_symbol_count: usize,
    /// 현재 연속 손실로 신규 진입이 차단된 전략/종목 조합 수
    pub blocked_strategy_symbol_count: usize,
    /// 오늘 누적 총 손실 (음수)
    pub current_loss: i64,
    /// 오늘 누적 총 수익 (양수)
    pub daily_profit: i64,
    /// 순손실 = 총손실 - 당일수익 (양수 = 순손실)
    pub net_loss: i64,
    /// 순손실 한도 소진율 (0.0 ~ 1.0+)
    pub loss_ratio: f64,
    /// 비상 정지 여부
    pub is_emergency_stop: bool,
    /// 추가 거래 가능 여부
    pub can_trade: bool,
}

pub(crate) fn build_risk_view(risk: &crate::trading::risk::RiskManager) -> RiskConfigView {
    RiskConfigView {
        enabled: risk.is_enabled(),
        daily_loss_limit: risk.daily_loss_limit,
        max_position_ratio: risk.max_position_ratio,
        max_daily_buy_orders_per_symbol: 0,
        max_daily_sell_orders_per_symbol: risk.max_daily_sell_orders_per_symbol,
        max_consecutive_losses_per_strategy_symbol: risk.max_consecutive_losses_per_strategy_symbol,
        volatility_sizing_enabled: risk.volatility_sizing_enabled,
        risk_per_trade_bps: risk.risk_per_trade_bps,
        atr_stop_multiplier: risk.atr_stop_multiplier,
        atr_symbol_count: risk.atr_symbol_count(),
        blocked_strategy_symbol_count: risk.blocked_strategy_symbol_count(),
        current_loss: risk.current_loss(),
        daily_profit: risk.daily_profit(),
        net_loss: risk.net_loss(),
        loss_ratio: risk.loss_ratio(),
        is_emergency_stop: risk.is_emergency_stop(),
        can_trade: risk.can_trade(),
    }
}

/// 리스크 runtime 스냅샷을 저장한다. 실패 시 사용자에게 알릴 수 있게 CmdError를 반환한다.
async fn persist_risk_runtime(
    state: &State<'_, AppState>,
    risk: &crate::trading::risk::RiskManager,
) -> CmdResult<()> {
    state
        .risk_store
        .save_runtime(&risk.runtime_state())
        .await
        .map_err(|error| CmdError {
            code: "RISK_PERSIST_FAILED".into(),
            message: format!(
                "리스크 상태가 적용되었지만 저장에 실패해 재시작 시 복원되지 않습니다: {error}"
            ),
        })
}

#[tauri::command]
pub async fn get_risk_config(state: State<'_, AppState>) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    // 날짜가 바뀌면 자동으로 당일 손익 초기화
    if risk.reset_if_new_day() {
        persist_risk_runtime(&state, &risk).await?;
    }
    Ok(build_risk_view(&risk))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRiskConfigInput {
    /// 리스크 관리 활성화 여부
    pub enabled: Option<bool>,
    pub daily_loss_limit: Option<i64>,
    /// 0.01 ~ 1.0 (1% ~ 100%)
    pub max_position_ratio: Option<f64>,
    /// 전략/종목별 일일 매수 주문 제한. 하위 호환 입력이며 저장 시 0으로 고정한다.
    pub max_daily_buy_orders_per_symbol: Option<u32>,
    /// 전략/종목별 일일 매도 주문 제한. 0이면 제한 없음.
    pub max_daily_sell_orders_per_symbol: Option<u32>,
    /// 전략/종목별 연속 손실 차단 기준. 0이면 제한 없음.
    pub max_consecutive_losses_per_strategy_symbol: Option<u32>,
    /// ATR 기반 주문 수량 산정 활성화 여부
    pub volatility_sizing_enabled: Option<bool>,
    /// 거래당 허용 위험 한도(bps). 0이면 고정 수량 유지.
    pub risk_per_trade_bps: Option<u32>,
    /// ATR 손절폭 배수
    pub atr_stop_multiplier: Option<f64>,
}

#[tauri::command]
pub async fn update_risk_config(
    input: UpdateRiskConfigInput,
    state: State<'_, AppState>,
) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    if let Some(en) = input.enabled {
        risk.set_enabled(en);
    }
    if let Some(limit) = input.daily_loss_limit {
        if limit < 0 {
            return Err(CmdError {
                code: "INVALID_PARAM".into(),
                message: "손실 한도는 0 이상이어야 합니다.".into(),
            });
        }
        risk.daily_loss_limit = limit;
    }
    if let Some(ratio) = input.max_position_ratio {
        if !(0.0..=1.0).contains(&ratio) {
            return Err(CmdError {
                code: "INVALID_PARAM".into(),
                message: "포지션 비중은 0.0~1.0 범위여야 합니다.".into(),
            });
        }
        risk.max_position_ratio = ratio;
    }
    if input.max_daily_buy_orders_per_symbol.is_some() {
        risk.max_daily_buy_orders_per_symbol = 0;
    }
    if let Some(limit) = input.max_daily_sell_orders_per_symbol {
        risk.max_daily_sell_orders_per_symbol = limit;
    }
    if let Some(limit) = input.max_consecutive_losses_per_strategy_symbol {
        risk.max_consecutive_losses_per_strategy_symbol = limit;
    }
    if let Some(enabled) = input.volatility_sizing_enabled {
        risk.volatility_sizing_enabled = enabled;
    }
    if let Some(bps) = input.risk_per_trade_bps {
        if bps > 10_000 {
            return Err(CmdError {
                code: "INVALID_PARAM".into(),
                message: "거래당 위험 한도는 0~10000bps 범위여야 합니다.".into(),
            });
        }
        risk.risk_per_trade_bps = bps;
    }
    if let Some(multiplier) = input.atr_stop_multiplier {
        if !(0.1..=20.0).contains(&multiplier) {
            return Err(CmdError {
                code: "INVALID_PARAM".into(),
                message: "ATR 손절 배수는 0.1~20.0 범위여야 합니다.".into(),
            });
        }
        risk.atr_stop_multiplier = multiplier;
    }
    tracing::info!(
        "리스크 설정 변경: 활성={}, 일일손실한도={}원, 종목비중={:.0}%, 일일주문제한(매수해제/매도)=0/{}, 연속손실차단={}회, ATR수량산정={}, 거래당위험={}bps, ATR배수={:.2}",
        risk.is_enabled(),
        risk.daily_loss_limit,
        risk.max_position_ratio * 100.0,
        risk.max_daily_sell_orders_per_symbol,
        risk.max_consecutive_losses_per_strategy_symbol,
        risk.volatility_sizing_enabled,
        risk.risk_per_trade_bps,
        risk.atr_stop_multiplier
    );
    state
        .risk_store
        .save_config(&risk.config_state())
        .await
        .map_err(|error| CmdError {
            code: "RISK_PERSIST_FAILED".into(),
            message: format!(
                "리스크 설정이 적용되었지만 저장에 실패해 재시작 시 복원되지 않습니다: {error}"
            ),
        })?;
    Ok(build_risk_view(&risk))
}

/// 비상 정지 수동 해제
#[tauri::command]
pub async fn clear_emergency_stop(state: State<'_, AppState>) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    risk.clear_emergency_stop();
    persist_risk_runtime(&state, &risk).await?;
    Ok(build_risk_view(&risk))
}

/// 비상 정지 수동 발동 (사용자가 직접 자동매매를 중단시킬 때)
#[tauri::command]
pub async fn activate_emergency_stop(state: State<'_, AppState>) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    risk.trigger_emergency_stop();
    persist_risk_runtime(&state, &risk).await?;
    Ok(build_risk_view(&risk))
}

// ────────────────────────────────────────────────────────────────────
// 미체결 주문 목록 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingOrderView {
    pub odno: String,
    pub symbol: String,
    pub symbol_name: String,
    /// "buy" | "sell"
    pub side: String,
    /// "pending" | "partially_filled" | "filled" | "cancelled" | "failed"
    pub status: String,
    pub quantity: u64,
    pub filled_quantity: u64,
    pub remaining_quantity: u64,
    pub timestamp: String,
    pub signal_reason: String,
    pub provider: Option<String>,
    pub provider_order_id: Option<String>,
    pub provider_request_id: Option<String>,
    pub provider_tr_id: Option<String>,
    pub broker_id: crate::broker::BrokerId,
    pub broker_account_id: Option<String>,
}

pub(crate) fn pending_order_to_view(p: &crate::trading::order::PendingOrder) -> PendingOrderView {
    PendingOrderView {
        odno: p.record.kis_order_id.clone().unwrap_or_default(),
        symbol: p.record.symbol.clone(),
        symbol_name: p.record.symbol_name.clone(),
        side: match &p.record.side {
            crate::storage::order_store::OrderSide::Buy => "buy".into(),
            crate::storage::order_store::OrderSide::Sell => "sell".into(),
        },
        status: match &p.record.status {
            crate::storage::order_store::OrderStatus::Pending => "pending".into(),
            crate::storage::order_store::OrderStatus::Filled => "filled".into(),
            crate::storage::order_store::OrderStatus::PartiallyFilled => "partially_filled".into(),
            crate::storage::order_store::OrderStatus::Cancelled => "cancelled".into(),
            crate::storage::order_store::OrderStatus::Rejected => "rejected".into(),
            crate::storage::order_store::OrderStatus::Expired => "expired".into(),
            crate::storage::order_store::OrderStatus::Failed => "failed".into(),
        },
        quantity: p.record.quantity,
        filled_quantity: p.filled_quantity,
        remaining_quantity: p.record.quantity.saturating_sub(p.filled_quantity),
        timestamp: p.record.timestamp.clone(),
        signal_reason: p.signal_reason.clone(),
        provider: p.record.provider.clone(),
        provider_order_id: p.record.provider_order_id.clone(),
        provider_request_id: p.record.provider_request_id.clone(),
        provider_tr_id: p.record.provider_tr_id.clone(),
        broker_id: p.broker_scope.broker_id,
        broker_account_id: p.broker_scope.account_id.as_ref().map(|id| id.0.clone()),
    }
}

#[tauri::command]
pub async fn get_pending_orders(state: State<'_, AppState>) -> CmdResult<Vec<PendingOrderView>> {
    let mgr = state.order_manager.lock().await;
    let views = mgr
        .pending_orders()
        .iter()
        .map(|p| pending_order_to_view(p))
        .collect();
    Ok(views)
}
