use super::*;

mod history;
use history::initialize_active_strategy_history;

async fn fetch_account_risk_balance_krw(
    rest: &Arc<KisRestClient>,
    include_overseas: bool,
    exchange_rate_krw: f64,
) -> i64 {
    let domestic_total = match rest.get_balance().await {
        Ok(resp) => balance_summary_total_krw(resp.summary.as_ref()),
        Err(e) => {
            tracing::warn!("리스크 총잔고 조회 실패(국내): {}", e);
            0
        }
    };

    if !include_overseas {
        return domestic_total;
    }

    let overseas_total = match rest.get_overseas_balance().await {
        Ok(resp) => resp
            .summary
            .as_ref()
            .map(|s| (parse_amount_f64(&s.frcr_evlu_tota) * exchange_rate_krw).round() as i64)
            .unwrap_or(0),
        Err(e) => {
            tracing::warn!("리스크 총잔고 조회 실패(해외): {}", e);
            0
        }
    };

    domestic_total.saturating_add(overseas_total.max(0))
}

/// Toss 활성 프로파일 기준 리스크 총잔고(KRW 환산) 조회
///
/// KIS의 `fetch_account_risk_balance_krw`에 대응하는 Toss 경로.
/// 매수가능금액(KRW/USD 현금)과 보유 종목 평가금액(KR/US)을 모두 합산해 KRW로 환산한다.
async fn fetch_toss_risk_balance_krw(profile: &AccountProfile, exchange_rate_krw: f64) -> i64 {
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(profile.account_no.clone()),
    );
    let account_seq = profile.account_no.as_str();

    let mut total_krw = 0.0f64;

    for currency in [BrokerCurrency::Krw, BrokerCurrency::Usd] {
        match adapter.get_buying_power(Some(account_seq), currency).await {
            Ok(power) => {
                let amount = power
                    .cash_buying_power
                    .trim()
                    .replace(',', "")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                total_krw += if currency == BrokerCurrency::Usd {
                    amount * exchange_rate_krw
                } else {
                    amount
                };
            }
            Err(e) => {
                tracing::warn!("리스크 총잔고 조회 실패(Toss 매수가능금액 {:?}): {}", currency, e);
            }
        }
    }

    let account_id = BrokerAccountId(profile.broker_account_id());
    match adapter.list_holdings(Some(&account_id)).await {
        Ok(holdings) => {
            for holding in &holdings {
                let qty = holding
                    .quantity
                    .0
                    .trim()
                    .replace(',', "")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                let price = holding
                    .current_price
                    .amount
                    .trim()
                    .replace(',', "")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                let value = qty * price;
                total_krw += if holding.current_price.currency == BrokerCurrency::Usd {
                    value * exchange_rate_krw
                } else {
                    value
                };
            }
        }
        Err(e) => tracing::warn!("리스크 총잔고 조회 실패(Toss 보유종목): {}", e),
    }

    total_krw.round() as i64
}

fn calculate_atr(candles: &[OhlcCandle], period: usize) -> Option<u64> {
    if candles.len() < 2 || period == 0 {
        return None;
    }
    let start = candles.len().saturating_sub(period).max(1);
    let mut ranges = Vec::with_capacity(candles.len().saturating_sub(start));
    for idx in start..candles.len() {
        let candle = candles[idx];
        let prev_close = candles[idx - 1].close;
        let high_low = candle.high.saturating_sub(candle.low);
        let high_prev = candle.high.abs_diff(prev_close);
        let low_prev = candle.low.abs_diff(prev_close);
        ranges.push(high_low.max(high_prev).max(low_prev));
    }
    if ranges.is_empty() {
        return None;
    }
    Some((ranges.iter().sum::<u64>() as f64 / ranges.len() as f64).round() as u64)
        .filter(|atr| *atr > 0)
}

// ────────────────────────────────────────────────────────────────────
// 자동 매매 제어
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingStatus {
    pub is_running: bool,
    pub active_strategies: Vec<String>,
    pub position_count: usize,
    pub total_unrealized_pnl: i64,
    /// WebSocket 실시간 시세 연결 여부
    pub ws_connected: bool,
    /// 자동매매가 실행 중인 프로파일 ID (미실행 시 None)
    pub trading_profile_id: Option<String>,
    /// 자동매매가 실행 중인 broker ID (미실행 시 None)
    pub trading_broker_id: Option<BrokerId>,
    /// 자동매매가 실행 중인 broker account ID (미실행 시 None)
    pub trading_account_id: Option<String>,
    /// 잔고 부족으로 매수 정지 여부
    pub buy_suspended: bool,
    /// 매수 정지 사유 (KIS 응답 msg1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_suspended_reason: Option<String>,
}

#[tauri::command]
pub async fn get_trading_status(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    let is_running = *state.is_trading.lock().await;
    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let trading_profile_id = state.trading_profile_id.read().await.clone();
    let trading_broker_id = *state.trading_broker_id.read().await;
    let trading_account_id = state.trading_account_id.read().await.clone();
    let (buy_suspended, buy_suspended_reason) = {
        let om = state.order_manager.lock().await;
        (om.buy_suspended, om.buy_suspended_reason.clone())
    };
    Ok(TradingStatus {
        is_running,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
        trading_broker_id,
        trading_account_id,
        buy_suspended,
        buy_suspended_reason,
    })
}

async fn sync_strategy_positions_from_active_broker(
    state: &AppState,
    current_cfg: &AppConfig,
) -> usize {
    let active_profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    };
    match active_profile
        .as_ref()
        .map(|profile| profile.broker_id)
        .unwrap_or(current_cfg.broker_id)
    {
        BrokerId::Kis => sync_kis_strategy_positions(state).await,
        BrokerId::Toss => match active_profile {
            Some(profile) => sync_toss_strategy_positions(state, profile).await,
            None => {
                tracing::warn!("Toss holdings 동기화 건너뜀: 활성 Toss 프로파일이 없습니다.");
                0
            }
        },
    }
}

async fn sync_kis_strategy_positions(state: &AppState) -> usize {
    let rest = state.rest_client.read().await.clone();
    let mut synced = 0usize;
    match rest.get_balance().await {
        Ok(resp) => {
            state
                .stock_store
                .upsert_many(
                    resp.items
                        .iter()
                        .map(|i| (i.pdno.clone(), i.prdt_name.clone())),
                )
                .await;
            {
                let mut tracker = state.position_tracker.lock().await;
                tracker.load_if_empty(resp.items.iter().map(|i| {
                    (
                        i.pdno.clone(),
                        i.prdt_name.clone(),
                        i.hldg_qty.parse::<u64>().unwrap_or(0),
                        i.pchs_avg_pric.parse::<f64>().unwrap_or(0.0) as u64,
                        i.prpr.parse::<u64>().unwrap_or(0),
                    )
                }));
            }
            {
                let mut mgr = state.strategy_manager.lock().await;
                for item in &resp.items {
                    let qty = item.hldg_qty.parse::<u64>().unwrap_or(0);
                    let avg = item.pchs_avg_pric.parse::<f64>().unwrap_or(0.0) as u64;
                    if qty > 0 {
                        synced += 1;
                    }
                    mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                        broker_id: BrokerId::Kis,
                        market: BrokerMarket::Kr,
                        symbol: item.pdno.clone(),
                        quantity: qty,
                        avg_price: avg,
                    });
                }
            }
        }
        Err(e) => tracing::warn!("자동매매 시작 전 국내 잔고 동기화 실패: {}", e),
    }

    match rest.get_overseas_balance().await {
        Ok(resp) => {
            {
                let mut tracker = state.overseas_position_tracker.lock().await;
                tracker.load_if_empty(resp.items.iter().map(|i| {
                    (
                        i.ovrs_pdno.clone(),
                        i.ovrs_item_name.clone(),
                        normalize_overseas_order_exchange(&i.ovrs_excg_cd),
                        i.ovrs_cblc_qty.parse::<u64>().unwrap_or(0),
                        usd_to_cents(&i.pchs_avg_pric),
                        usd_to_cents(&i.now_pric2),
                    )
                }));
            }
            let mut mgr = state.strategy_manager.lock().await;
            for item in &resp.items {
                let qty = item.ovrs_cblc_qty.parse::<u64>().unwrap_or(0);
                let avg = usd_to_cents(&item.pchs_avg_pric);
                if qty > 0 {
                    synced += 1;
                }
                mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                    broker_id: BrokerId::Kis,
                    market: BrokerMarket::Us,
                    symbol: item.ovrs_pdno.clone(),
                    quantity: qty,
                    avg_price: avg,
                });
            }
        }
        Err(e) => tracing::warn!("자동매매 시작 전 해외 잔고 동기화 실패: {}", e),
    }

    synced
}

async fn sync_toss_strategy_positions(state: &AppState, profile: AccountProfile) -> usize {
    if !profile.is_configured() {
        tracing::warn!("Toss holdings 동기화 건너뜀: 활성 Toss 프로파일 설정이 미완료입니다.");
        return 0;
    }

    let account_id = BrokerAccountId(profile.broker_account_id());
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let holdings = match adapter.list_holdings(Some(&account_id)).await {
        Ok(holdings) => holdings,
        Err(e) => {
            tracing::warn!("자동매매 시작 전 Toss holdings 동기화 실패: {}", e);
            return 0;
        }
    };

    state
        .stock_store
        .upsert_many(
            holdings
                .iter()
                .map(|holding| (holding.symbol.0.clone(), holding.symbol_name.clone())),
        )
        .await;

    {
        let mut domestic_tracker = state.position_tracker.lock().await;
        domestic_tracker.load_if_empty(
            holdings
                .iter()
                .filter(|holding| holding.market == BrokerMarket::Kr)
                .map(|holding| {
                    (
                        holding.symbol.0.clone(),
                        holding.symbol_name.clone(),
                        decimal_quantity_to_position_units(&holding.quantity.0),
                        broker_money_to_strategy_price_units(&holding.average_price),
                        broker_money_to_strategy_price_units(&holding.current_price),
                    )
                }),
        );
    }
    {
        let mut overseas_tracker = state.overseas_position_tracker.lock().await;
        overseas_tracker.load_if_empty(
            holdings
                .iter()
                .filter(|holding| holding.market == BrokerMarket::Us)
                .map(|holding| {
                    (
                        holding.symbol.0.clone(),
                        holding.symbol_name.clone(),
                        "TOSS_US".to_string(),
                        decimal_quantity_to_position_units(&holding.quantity.0),
                        broker_money_to_strategy_price_units(&holding.average_price),
                        broker_money_to_strategy_price_units(&holding.current_price),
                    )
                }),
        );
    }

    let mut synced = 0usize;
    let mut mgr = state.strategy_manager.lock().await;
    for holding in &holdings {
        let quantity = decimal_quantity_to_position_units(&holding.quantity.0);
        if quantity > 0 {
            synced += 1;
        }
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Toss,
            market: holding.market,
            symbol: holding.symbol.0.clone(),
            quantity,
            avg_price: broker_money_to_strategy_price_units(&holding.average_price),
        });
    }
    tracing::info!(
        "Toss holdings 기반 전략 포지션 동기화 완료: {}개 보유 종목",
        synced
    );
    synced
}

#[tauri::command]
pub async fn start_trading(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<TradingStatus> {
    let current_cfg = state.config.read().await.clone();
    if current_cfg.broker_id == BrokerId::Kis && !current_cfg.is_kis_configured() {
        return Err(CmdError {
            code: "CONFIG_NOT_READY".into(),
            message: "KIS API 설정이 완료되지 않았습니다. Settings에서 API 키를 확인하세요.".into(),
        });
    }
    if current_cfg.broker_id == BrokerId::Toss {
        let active_profile = {
            let profiles = state.profiles.read().await;
            profiles.get_active().cloned()
        };
        let Some(profile) = active_profile.filter(|profile| profile.broker_id == BrokerId::Toss)
        else {
            return Err(CmdError {
                code: "CONFIG_NOT_READY".into(),
                message: "활성 Toss 프로파일이 없습니다.".into(),
            });
        };
        if !profile.is_configured() {
            return Err(CmdError {
                code: "CONFIG_NOT_READY".into(),
                message: "토스증권 Client ID/Secret/accountSeq 설정이 완료되지 않았습니다.".into(),
            });
        }
        if !profile.live_trading_consent {
            return Err(CmdError {
                code: "LIVE_TRADING_CONSENT_REQUIRED".into(),
                message: "Toss 실거래 동의를 저장해야 자동매매 주문 실행을 시작할 수 있습니다."
                    .into(),
            });
        }
    }

    if *state.is_trading.lock().await {
        return Err(CmdError {
            code: "ALREADY_RUNNING".into(),
            message: "자동 매매가 이미 실행 중입니다.".into(),
        });
    }

    // 자동매매 시작 전 실제 잔고를 전략 내부 포지션 상태와 동기화한다.
    // 재시작 직후 내부 in_position=false 상태로 같은 종목을 재매수하는 위험을 줄인다.
    let synced_positions = sync_strategy_positions_from_active_broker(&state, &current_cfg).await;
    tracing::info!(
        "자동매매 시작 전 broker-aware 포지션 동기화 완료: {}개",
        synced_positions
    );

    let mut is_running = state.is_trading.lock().await;
    if *is_running {
        return Err(CmdError {
            code: "ALREADY_RUNNING".into(),
            message: "자동 매매가 이미 실행 중입니다.".into(),
        });
    }
    *is_running = true;
    tracing::info!("자동 매매 시작");

    // 자동매매 시작 시점의 활성 프로파일 ID 스냅샷 저장
    {
        let (active_id, broker_id, account_id) = {
            let profiles = state.profiles.read().await;
            match profiles.get_active() {
                Some(profile) => (
                    Some(profile.id.clone()),
                    Some(profile.broker_id),
                    Some(profile.broker_account_id()),
                ),
                None => (
                    profiles.active_id.clone(),
                    Some(current_cfg.broker_id),
                    Some(current_cfg.broker_account_id.clone())
                        .filter(|account_id| !account_id.is_empty()),
                ),
            }
        };
        let execution_scope = BrokerScope::new(
            broker_id.unwrap_or(BrokerId::Kis),
            account_id.clone().map(BrokerAccountId),
        );
        state
            .order_manager
            .lock()
            .await
            .set_execution_scope(execution_scope);
        *state.trading_profile_id.write().await = active_id;
        *state.trading_broker_id.write().await = broker_id;
        *state.trading_account_id.write().await = account_id;
    }

    if let Some(notifier) = &state.discord {
        let _ = notifier
            .send(NotificationEvent::info(
                "자동 매매 시작".to_string(),
                "AutoConditionTrade 자동 매매가 시작되었습니다.".to_string(),
            ))
            .await;
    }
    drop(is_running);

    initialize_active_strategy_history(&state).await;

    // WebSocket 연결 시작 (KIS 전용 보조 — 실패해도 폴링 루프가 독립 동작)
    if current_cfg.broker_id == BrokerId::Kis {
        let rest = state.rest_client.read().await.clone();
        let ws_client = crate::api::KisWebSocketClient::new(
            rest.is_paper(),
            rest.app_key().to_string(),
            rest.app_secret().to_string(),
            rest.token_manager(),
        );

        // 활성 전략에서 구독할 종목 수집
        let symbols: Vec<String> = state.strategy_manager.lock().await.active_symbols();

        let ws_connected = Arc::clone(&state.ws_connected);
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = ws_client.subscribe(symbols, app_handle, ws_connected).await {
                tracing::error!("WebSocket 연결 실패: {}", e);
            }
        });
    } else {
        tracing::info!(
            "Toss 자동매매는 현재 polling 기반으로 실행하며 KIS WebSocket 구독을 생략합니다."
        );
    }

    // ── 폴링 기반 자동매매 루프 ──────────────────────────────────
    // run_trading_daemon() 이 앱 시작 시 영구 데몬으로 이미 실행 중이다.
    // is_trading 플래그가 true 로 바뀌면 데몬이 자동으로 폴링을 재개한다.
    // (이전 spawn 블록은 lib.rs → tauri::async_runtime::spawn(run_trading_daemon(...)) 로 이동)

    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let trading_profile_id = state.trading_profile_id.read().await.clone();
    let trading_broker_id = *state.trading_broker_id.read().await;
    let trading_account_id = state.trading_account_id.read().await.clone();
    let (buy_suspended, buy_suspended_reason) = {
        let om = state.order_manager.lock().await;
        (om.buy_suspended, om.buy_suspended_reason.clone())
    };
    Ok(TradingStatus {
        is_running: true,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
        trading_broker_id,
        trading_account_id,
        buy_suspended,
        buy_suspended_reason,
    })
}

#[tauri::command]
pub async fn stop_trading(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    let mut is_running = state.is_trading.lock().await;
    *is_running = false;
    tracing::info!("자동 매매 정지");

    // 자동매매 종료 시 트레이딩 프로파일 ID 클리어
    *state.trading_profile_id.write().await = None;
    *state.trading_broker_id.write().await = None;
    *state.trading_account_id.write().await = None;

    if let Some(notifier) = &state.discord {
        let _ = notifier
            .send(NotificationEvent::info(
                "자동 매매 정지".to_string(),
                "AutoConditionTrade 자동 매매가 정지되었습니다.".to_string(),
            ))
            .await;
    }
    drop(is_running);

    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    Ok(TradingStatus {
        is_running: false,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id: None,
        trading_broker_id: None,
        trading_account_id: None,
        buy_suspended: false,
        buy_suspended_reason: None,
    })
}

/// 잔고 부족 매수 정지를 수동으로 해제합니다.
/// 계좌에 자금을 입금한 경우 또는 오판 시 사용.
#[tauri::command]
pub async fn clear_buy_suspension(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    state.order_manager.lock().await.clear_buy_suspension();

    let is_running = *state.is_trading.lock().await;
    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let trading_profile_id = state.trading_profile_id.read().await.clone();
    let trading_broker_id = *state.trading_broker_id.read().await;
    let trading_account_id = state.trading_account_id.read().await.clone();
    Ok(TradingStatus {
        is_running,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
        trading_broker_id,
        trading_account_id,
        buy_suspended: false,
        buy_suspended_reason: None,
    })
}

// ────────────────────────────────────────────────────────────────────
// 자동매매 폴링 데몬 (lib.rs 에서 앱 시작 시 영구 spawn)
//
// is_trading 플래그가 false 이면 5초마다 재확인하며 대기.
// true 로 바뀌면 즉시 폴링 재개. start_trading / web API 모두 이 방식으로 제어.
//
// 설계 원칙:
//   - 레이블 루프(labeled loop)·goto 유사 패턴 사용 금지
//   - 제어 흐름은 항상 위→아래 순차 실행
//   - 내부 루프에서 외부 루프로 점프하는 break 'label / continue 'label 금지
//   - 내부 루프 조기 탈출이 필요한 경우 별도 함수로 추출 후 return 사용
// ────────────────────────────────────────────────────────────────────

/// 종목 폴링 한 사이클(틱)의 처리 결과
///
/// `poll_symbols_tick` 반환값으로 사용. 호출자가 결과에 따라
/// 시장 대기 여부를 결정한다.
#[derive(Debug, PartialEq)]
enum TickCycleResult {
    /// 모든 종목 정상 처리 완료
    Done,
    /// 장 마감 / 장외 시간 감지 → 호출자는 market_pause_until 설정 후 다음 이터레이션으로
    MarketClosed,
    /// is_trading 플래그가 false 로 바뀜 → 이번 사이클 조기 종료
    Stopped,
}

/// 자동매매 실행 scope(`OrderManager::execution_scope`)에 해당하는 계정 프로파일을 조회한다.
///
/// `account_id`가 scope에 있으면 broker_id+account_id로 매칭하고, 없으면 활성 프로파일로 폴백한다.
/// (order/submission.rs의 `SubmissionDeps.active_profile` 조회 로직과 동일한 매칭 규칙 — 시세 조회와
/// 주문 제출이 서로 다른 프로파일을 바라보지 않도록 일치시킨다.)
async fn resolve_scoped_profile(
    profiles: &Arc<RwLock<ProfilesConfig>>,
    scope: &BrokerScope,
) -> Option<AccountProfile> {
    let profiles = profiles.read().await;
    let account_id = scope.account_id.as_ref().map(|id| id.0.as_str());
    profiles
        .profiles
        .iter()
        .find(|profile| {
            profile.broker_id == scope.broker_id
                && account_id
                    .map(|id| profile.broker_account_id() == id)
                    .unwrap_or_else(|| {
                        profiles.get_active().map(|p| p.id.as_str()) == Some(profile.id.as_str())
                    })
        })
        .cloned()
}

/// 자동매매 시세 폴링에 사용할 broker 소스.
///
/// 실행 scope가 Toss면 해당 프로파일 자격증명으로 Toss 시세를 조회하고,
/// 그 외(KIS)에는 기존 `KisRestClient` 경로를 그대로 사용한다.
enum PriceSource {
    Kis,
    Toss(AccountProfile),
}

/// Toss 종목 현재가 조회 → (가격, 거래량) 튜플로 변환
///
/// 가격 단위는 KIS 경로와 동일하게 맞춘다: KRW는 원 단위 정수, USD는 센트(×100) 정수.
/// Toss `/api/v1/prices`는 거래량을 제공하지 않아 volume은 항상 0으로 반환한다
/// (leveraged_trend_hold 등 volume을 사용하지 않는 전략에는 영향 없음).
async fn fetch_toss_tick(profile: &AccountProfile, symbol: &str) -> anyhow::Result<(u64, u64)> {
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(profile.account_no.clone()),
    );
    let quote = adapter
        .get_price(&BrokerSymbol(symbol.to_string()))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let amount = quote.last.amount.trim().replace(',', "");
    let price = match quote.last.currency {
        BrokerCurrency::Krw => amount.parse::<u64>().unwrap_or(0),
        BrokerCurrency::Usd => (amount.parse::<f64>().unwrap_or(0.0) * 100.0).round() as u64,
    };
    let volume = quote
        .volume
        .and_then(|q| q.0.parse::<u64>().ok())
        .unwrap_or(0);
    Ok((price, volume))
}

/// 종목 목록을 순차적으로 순회하여 현재가 조회 + 전략 신호 처리
///
/// `break 'label` / `continue 'label` 없이 `return`으로 조기 탈출한다.
/// 장 마감 감지 시 `TickCycleResult::MarketClosed` 를 반환하고,
/// 호출자(run_trading_daemon)가 market_pause_until 을 설정한다.
#[allow(clippy::too_many_arguments)]
async fn poll_symbols_tick(
    symbols: &[String],
    is_trading: &Arc<Mutex<bool>>,
    strategy_mgr: &Arc<Mutex<crate::trading::strategy::StrategyManager>>,
    order_mgr: &Arc<Mutex<crate::trading::order::OrderManager>>,
    stock_store: &Arc<crate::storage::stock_store::StockStore>,
    rest: &Arc<KisRestClient>,
    price_source: &PriceSource,
    delay_ms: u64,
    total_balance_krw: i64,
    fills_pending: &mut Vec<(String, u64)>,
    market_calendar: Option<&MarketCalendarOverride>,
) -> TickCycleResult {
    for symbol in symbols {
        // 종료 플래그 선행 확인
        if !*is_trading.lock().await {
            return TickCycleResult::Stopped;
        }

        // 해당 종목 시장 개장 여부 확인 (폐장이면 건너뜀)
        if !is_market_open_for_with_calendar(symbol, market_calendar) {
            tracing::debug!(
                "시장 폐장 — 건너뜀: {} ({})",
                symbol,
                if is_domestic_symbol(symbol) {
                    "KRX"
                } else {
                    "US"
                }
            );
            continue;
        }

        // 현재가 조회 + 해외 주문용 거래소 코드 캡처 (활성 broker 프로파일에 맞는 소스로 조회)
        let (tick, exchange_opt): (Result<(u64, u64), String>, Option<String>) = match price_source
        {
            PriceSource::Toss(profile) => match fetch_toss_tick(profile, symbol).await {
                Ok((price, volume)) => {
                    let exch = (!is_domestic_symbol(symbol)).then(|| "TOSS_US".to_string());
                    (Ok((price, volume)), exch)
                }
                Err(e) => (Err(e.to_string()), None),
            },
            PriceSource::Kis if is_domestic_symbol(symbol) => {
                let t = rest
                    .get_price(symbol)
                    .await
                    .map(|p| {
                        let price = p.stck_prpr.parse::<u64>().unwrap_or(0);
                        let volume = p.acml_vol.parse::<u64>().unwrap_or(0);
                        (price, volume)
                    })
                    .map_err(|e| e.to_string());
                (t, None)
            }
            PriceSource::Kis => match fetch_overseas_tick(rest, symbol).await {
                Ok((price, volume, exch)) => (Ok((price, volume)), Some(exch)),
                Err(e) => (Err(e.to_string()), None),
            },
        };

        // 틱 처리: 현재가 기반 전략 신호 생성 → 주문
        match tick {
            Ok((price, volume)) if price > 0 => {
                let signals = strategy_mgr.lock().await.on_tick(symbol, price, volume);
                for strategy_signal in signals {
                    let symbol_name = stock_store
                        .get_name(symbol)
                        .await
                        .unwrap_or_else(|| symbol.clone());
                    let submit_result = crate::trading::order::OrderManager::submit_signal_shared(
                        order_mgr,
                        Some(strategy_signal.strategy_id),
                        strategy_signal.signal,
                        symbol_name,
                        total_balance_krw,
                        exchange_opt.clone(),
                        price,
                    )
                    .await;
                    match submit_result {
                        Ok(()) => {
                            fills_pending.push((symbol.clone(), price));
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            if is_market_closed_error(&msg) {
                                tracing::info!(
                                    "장 마감/장외 시간 감지 (주문, {}) — 5분 대기: {}",
                                    symbol,
                                    msg
                                );
                                return TickCycleResult::MarketClosed;
                            }
                            tracing::warn!("신호 처리 실패 ({}): {}", symbol, msg);
                        }
                    }
                }
            }
            Ok(_) => {
                tracing::debug!("현재가 0 — 건너뜀: {}", symbol);
            }
            Err(e) => {
                if is_market_closed_error(&e) {
                    tracing::info!(
                        "장 마감/장외 시간 감지 (현재가, {}) — 5분 대기: {}",
                        symbol,
                        e
                    );
                    return TickCycleResult::MarketClosed;
                }
                tracing::warn!("현재가 조회 실패 ({}): {}", symbol, e);
            }
        }

        // 종목 간 API 호출 딜레이
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        // 딜레이 후 종료 플래그 재확인
        if !*is_trading.lock().await {
            return TickCycleResult::Stopped;
        }
    }
    TickCycleResult::Done
}

#[allow(clippy::too_many_arguments)]
pub async fn run_trading_daemon(
    is_trading: Arc<Mutex<bool>>,
    strategy_mgr: Arc<Mutex<crate::trading::strategy::StrategyManager>>,
    order_mgr: Arc<Mutex<crate::trading::order::OrderManager>>,
    risk_mgr: Arc<Mutex<crate::trading::risk::RiskManager>>,
    rest_arc: Arc<RwLock<Arc<KisRestClient>>>,
    stock_store: Arc<crate::storage::stock_store::StockStore>,
    profiles: Arc<RwLock<ProfilesConfig>>,
    exchange_rate_krw: Arc<RwLock<f64>>,
) {
    tracing::info!("자동매매 폴링 데몬 시작 (is_trading=false 대기 중)");
    let mut last_reset_date = chrono::Local::now().date_naive();
    let mut market_pause_until: Option<tokio::time::Instant> = None;
    let mut fills_pending: Vec<(String, u64)> = Vec::new();
    let mut was_running = false;

    // 레이블 없는 단순 루프 — 제어 흐름은 위→아래 순차 실행
    loop {
        let is_running = *is_trading.lock().await;

        // ── Phase 1: 자동매매 비활성 → 5초 대기 후 재확인 ──────────
        if !is_running {
            if was_running {
                fills_pending.clear();
                market_pause_until = None;
                tracing::info!("자동매매 폴링 데몬 일시 정지 (is_trading=false)");
            }
            was_running = false;
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        // ── Phase 2: 방금 활성화됨 → 로컬 상태 초기화 ──────────────
        if !was_running {
            was_running = true;
            fills_pending.clear();
            market_pause_until = None;
            last_reset_date = chrono::Local::now().date_naive();
            tracing::info!("자동매매 폴링 데몬 활성화");
        }

        // ── Phase 3: 장 마감으로 대기 중 → 30초 슬립 후 재확인 ─────
        if let Some(pause_until) = market_pause_until {
            if tokio::time::Instant::now() < pause_until {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                continue;
            }
            tracing::info!("장 마감 대기 완료 — 폴링 재개");
            market_pause_until = None;
        }

        // ── Phase 4: 이전 틱 시장가 주문 자동 체결 확인 ────────────
        if !fills_pending.is_empty() {
            if let Err(e) = OrderManager::confirm_pending_fills_from_broker_shared(&order_mgr).await
            {
                tracing::debug!(
                    "주문번호 기반 체결 확인 실패 — 다음 틱 가격 확인으로 보완: {}",
                    e
                );
            }
            let fills = std::mem::take(&mut fills_pending);
            for (sym, fill_price) in fills {
                if let Err(e) = order_mgr
                    .lock()
                    .await
                    .confirm_fill_by_symbol(&sym, fill_price)
                    .await
                {
                    tracing::warn!("자동 체결 확인 실패 ({}): {}", sym, e);
                }
            }
        }

        // ── Phase 5: 날짜 변경 시 일별 초기화 ──────────────────────
        let today = chrono::Local::now().date_naive();
        if today != last_reset_date {
            last_reset_date = today;
            risk_mgr.lock().await.reset_if_new_day();
            order_mgr.lock().await.reset_day();
            tracing::info!("자동매매 일별 초기화 완료 ({})", today);
        }

        // ── Phase 6: 활성 전략의 종목 수집 ─────────────────────────
        let symbols: Vec<String> = strategy_mgr.lock().await.active_symbols();
        if symbols.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            continue;
        }

        // 자동매매 실행 scope에 맞는 시세/잔고 소스 결정 (Toss 활성 시 KIS REST 폴백 금지)
        let execution_scope = order_mgr.lock().await.execution_scope().clone();
        let price_source = if execution_scope.broker_id == BrokerId::Toss {
            match resolve_scoped_profile(&profiles, &execution_scope).await {
                Some(profile) => PriceSource::Toss(profile),
                None => {
                    tracing::warn!(
                        "자동매매 실행 scope가 Toss이지만 일치하는 프로파일을 찾지 못해 이번 틱을 건너뜁니다."
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    continue;
                }
            }
        } else {
            PriceSource::Kis
        };

        let rest = rest_arc.read().await.clone();
        let delay_ms: u64 = if rest.is_paper() { 700 } else { 150 };

        // ── Phase 7: 전체 시장 폐장 여부 사전 체크 ─────────────────
        let market_calendar = get_active_toss_calendar_override(&profiles).await;

        if symbols
            .iter()
            .all(|s| !is_market_open_for_with_calendar(s, market_calendar.as_ref()))
        {
            tracing::info!(
                "모든 시장 폐장 ({}) — 5분 대기 후 재확인",
                open_markets_summary_with_calendar(market_calendar.as_ref())
            );
            market_pause_until =
                Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(300));
            continue;
        }
        tracing::debug!(
            "시장 상태: {}",
            open_markets_summary_with_calendar(market_calendar.as_ref())
        );

        let exchange_rate = *exchange_rate_krw.read().await;
        let total_balance_krw = match &price_source {
            PriceSource::Toss(profile) => {
                fetch_toss_risk_balance_krw(profile, exchange_rate).await
            }
            PriceSource::Kis => {
                fetch_account_risk_balance_krw(
                    &rest,
                    symbols.iter().any(|s| !is_domestic_symbol(s)),
                    exchange_rate,
                )
                .await
            }
        };
        if total_balance_krw <= 0 {
            tracing::warn!(
                "리스크 총잔고가 0원으로 조회되어 ATR 수량 산정과 포지션 비중 검사를 건너뜁니다."
            );
        }

        // ── Phase 8: 종목별 현재가 조회 + 전략 신호 처리 ───────────
        //   내부 루프는 poll_symbols_tick() 으로 분리 — goto 유사 패턴 없음
        let tick_result = poll_symbols_tick(
            &symbols,
            &is_trading,
            &strategy_mgr,
            &order_mgr,
            &stock_store,
            &rest,
            &price_source,
            delay_ms,
            total_balance_krw,
            &mut fills_pending,
            market_calendar.as_ref(),
        )
        .await;

        if tick_result == TickCycleResult::MarketClosed {
            market_pause_until =
                Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(300));
            continue;
        }

        // ── Phase 9: 다음 틱까지 10초 대기 (100ms 단위 — 종료 신호 즉시 반응) ──
        for _ in 0u32..100 {
            if !*is_trading.lock().await {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 자동매매 폴링 루프 헬퍼
// ────────────────────────────────────────────────────────────────────

/// 장 마감 / 장외 시간 오류 여부 감지
///
/// KIS API가 시장 비운영 시간에 반환하는 공통 메시지 패턴을 검사한다.
///
/// ## 실제 KIS 응답 예시 (에러 로그에서 수집)
/// - `"모의투자 장종료 입니다."`
/// - `"모의투자 장시작전 입니다."`
/// - `"장운영시간이 아닙니다."`
/// - `"시간외거래"`
fn is_market_closed_error(msg: &str) -> bool {
    msg.contains("장종료")
        || msg.contains("장마감")
        || msg.contains("장시작전")
        || msg.contains("장운영시간")
        || msg.contains("시간외거래")
        || msg.contains("OPCODE-100")
}

/// 해외 주식 현재가 조회 (NAS → NYS → AMS 순으로 시도)
/// 반환값: (price_cents: u64, volume: u64, exchange: String)
/// - price_cents = USD 현재가 × 100 (정수화하여 on_tick에 전달)
/// - exchange = 성공한 거래소 코드 ("NAS" / "NYS" / "AMS")
async fn fetch_overseas_tick(
    rest: &std::sync::Arc<crate::api::rest::KisRestClient>,
    symbol: &str,
) -> anyhow::Result<(u64, u64, String)> {
    for exchange in &["NAS", "NYS", "AMS"] {
        match rest.get_overseas_price(symbol, exchange).await {
            Ok(p) => {
                let price_f: f64 = p.last.parse().unwrap_or(0.0);
                if price_f > 0.0 {
                    // USD → 센트(×100) 변환으로 u64 정수화
                    let price_cents = (price_f * 100.0).round() as u64;
                    let volume = p.tvol.parse::<u64>().unwrap_or(0);
                    return Ok((price_cents, volume, exchange.to_string()));
                }
            }
            Err(_) => continue,
        }
    }
    anyhow::bail!("해외 현재가 조회 실패: {} (NAS/NYS/AMS 모두 실패)", symbol)
}
