use super::*;

mod history;
use history::initialize_active_strategy_history;

pub(crate) async fn restore_risk_from_today_trades(
    trade_store: &Arc<TradeStore>,
    risk_manager: &Arc<Mutex<RiskManager>>,
    risk_store: &Arc<crate::storage::RiskStore>,
) -> CmdResult<()> {
    let trades = trade_store
        .get_by_date(chrono::Local::now().date_naive())
        .await
        .map_err(|error| CmdError {
            code: "RISK_RESTORE_FAILED".into(),
            message: format!("오늘 체결 ledger를 읽지 못했습니다: {error}"),
        })?;
    let mut risk = risk_manager.lock().await;
    risk.restore_daily_pnl(
        trades
            .into_iter()
            .filter_map(|trade| trade.realized_pnl_krw),
    );
    risk_store
        .save_runtime(&risk.runtime_state())
        .await
        .map_err(|error| CmdError {
            code: "RISK_PERSIST_FAILED".into(),
            message: format!("리스크 runtime 상태를 저장하지 못했습니다: {error}"),
        })?;
    Ok(())
}

pub(crate) async fn fetch_account_risk_balance_krw(
    rest: &Arc<KisRestClient>,
    include_overseas: bool,
    exchange_rate_krw: f64,
) -> Result<i64, String> {
    let domestic = rest
        .get_balance()
        .await
        .map_err(|e| format!("KIS 국내 잔고 조회 실패: {e}"))?;
    let domestic_total = balance_summary_total_krw(domestic.summary.as_ref());

    if !include_overseas {
        return Ok(domestic_total);
    }

    let overseas = rest
        .get_overseas_balance()
        .await
        .map_err(|e| format!("KIS 해외 잔고 조회 실패: {e}"))?;
    let overseas_total = overseas
        .summary
        .as_ref()
        .map(|s| (parse_amount_f64(&s.frcr_evlu_tota) * exchange_rate_krw).round() as i64)
        .unwrap_or(0);

    Ok(domestic_total.saturating_add(overseas_total.max(0)))
}

/// Toss 활성 프로파일 기준 리스크 총잔고(KRW 환산) 조회
///
/// KIS의 `fetch_account_risk_balance_krw`에 대응하는 Toss 경로.
/// 매수가능금액(KRW/USD 현금)과 보유 종목 평가금액(KR/US)을 모두 합산해 KRW로 환산한다.
pub(crate) async fn fetch_toss_risk_balance_krw(
    profile: &AccountProfile,
    exchange_rate_krw: f64,
) -> Result<i64, String> {
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(profile.account_no.clone()),
    );
    let account_seq = profile.account_no.as_str();

    let mut total_krw = 0.0f64;

    for currency in [BrokerCurrency::Krw, BrokerCurrency::Usd] {
        let power = adapter
            .get_buying_power(Some(account_seq), currency)
            .await
            .map_err(|e| format!("Toss {currency:?} 매수가능금액 조회 실패: {e}"))?;
        let amount = power
            .cash_buying_power
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("Toss {currency:?} 매수가능금액 형식 오류"))?;
        total_krw += if currency == BrokerCurrency::Usd {
            amount * exchange_rate_krw
        } else {
            amount
        };
    }

    let account_id = BrokerAccountId(profile.broker_account_id());
    let holdings = adapter
        .list_holdings(Some(&account_id))
        .await
        .map_err(|e| format!("Toss 보유종목 조회 실패: {e}"))?;
    for holding in &holdings {
        let qty = holding
            .quantity
            .0
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("Toss {} 보유수량 형식 오류", holding.symbol.0))?;
        let price = holding
            .current_price
            .amount
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("Toss {} 현재가 형식 오류", holding.symbol.0))?;
        let value = qty * price;
        total_krw += if holding.current_price.currency == BrokerCurrency::Usd {
            value * exchange_rate_krw
        } else {
            value
        };
    }

    Ok(total_krw.round() as i64)
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
) -> CmdResult<usize> {
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
            None => Err(CmdError {
                code: "ACCOUNT_SYNC_FAILED".into(),
                message: "활성 Toss 프로파일이 없어 잔고를 동기화할 수 없습니다.".into(),
            }),
        },
    }
}

async fn sync_kis_strategy_positions(state: &AppState) -> CmdResult<usize> {
    let rest = state.rest_client.read().await.clone();
    let mut synced = 0usize;
    let domestic = rest.get_balance().await.map_err(|e| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: format!("자동매매 시작 전 KIS 국내 잔고 동기화 실패: {e}"),
    })?;
    let overseas = rest.get_overseas_balance().await.map_err(|e| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: format!("자동매매 시작 전 KIS 해외 잔고 동기화 실패: {e}"),
    })?;
    let domestic_positions: Vec<(String, String, u64, u64, u64)> = domestic
        .items
        .iter()
        .map(|item| {
            Ok((
                item.pdno.clone(),
                item.prdt_name.clone(),
                parse_holding_u64(&item.hldg_qty, &item.pdno, "보유수량")?,
                parse_holding_price(&item.pchs_avg_pric, &item.pdno, "평균매입가")?,
                parse_holding_u64(&item.prpr, &item.pdno, "현재가")?,
            ))
        })
        .collect::<CmdResult<_>>()?;
    let overseas_positions: Vec<(String, String, String, u64, u64, u64)> = overseas
        .items
        .iter()
        .map(|item| {
            Ok((
                item.ovrs_pdno.clone(),
                item.ovrs_item_name.clone(),
                normalize_overseas_order_exchange(&item.ovrs_excg_cd),
                parse_holding_u64(&item.ovrs_cblc_qty, &item.ovrs_pdno, "해외 보유수량")?,
                parse_holding_usd_cents(&item.pchs_avg_pric, &item.ovrs_pdno, "해외 평균매입가")?,
                parse_holding_usd_cents(&item.now_pric2, &item.ovrs_pdno, "해외 현재가")?,
            ))
        })
        .collect::<CmdResult<_>>()?;

    {
        let resp = domestic;
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
            tracker.replace(domestic_positions.iter().cloned());
        }
        {
            let mut mgr = state.strategy_manager.lock().await;
            for symbol in mgr.active_symbols() {
                if is_domestic_symbol(&symbol) {
                    mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                        broker_id: BrokerId::Kis,
                        market: BrokerMarket::Kr,
                        symbol,
                        quantity: 0,
                        avg_price: 0,
                    });
                }
            }
            for (symbol, _, qty, avg, _) in &domestic_positions {
                if *qty > 0 {
                    synced += 1;
                }
                mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                    broker_id: BrokerId::Kis,
                    market: BrokerMarket::Kr,
                    symbol: symbol.clone(),
                    quantity: *qty,
                    avg_price: *avg,
                });
            }
        }
    }

    {
        {
            let mut tracker = state.overseas_position_tracker.lock().await;
            tracker.replace(overseas_positions.iter().cloned());
        }
        let mut mgr = state.strategy_manager.lock().await;
        for symbol in mgr.active_symbols() {
            if !is_domestic_symbol(&symbol) {
                mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                    broker_id: BrokerId::Kis,
                    market: BrokerMarket::Us,
                    symbol,
                    quantity: 0,
                    avg_price: 0,
                });
            }
        }
        for (symbol, _, _, qty, avg, _) in &overseas_positions {
            if *qty > 0 {
                synced += 1;
            }
            mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                broker_id: BrokerId::Kis,
                market: BrokerMarket::Us,
                symbol: symbol.clone(),
                quantity: *qty,
                avg_price: *avg,
            });
        }
    }

    Ok(synced)
}

fn parse_holding_u64(value: &str, symbol: &str, field: &str) -> CmdResult<u64> {
    value
        .trim()
        .replace(',', "")
        .parse::<u64>()
        .map_err(|_| CmdError {
            code: "ACCOUNT_SYNC_INVALID_DATA".into(),
            message: format!("{symbol} {field} 응답 형식이 올바르지 않습니다: {value:?}"),
        })
}

fn parse_holding_price(value: &str, symbol: &str, field: &str) -> CmdResult<u64> {
    value
        .trim()
        .replace(',', "")
        .parse::<f64>()
        .map(|parsed| parsed.max(0.0).round() as u64)
        .map_err(|_| CmdError {
            code: "ACCOUNT_SYNC_INVALID_DATA".into(),
            message: format!("{symbol} {field} 응답 형식이 올바르지 않습니다: {value:?}"),
        })
}

fn parse_holding_usd_cents(value: &str, symbol: &str, field: &str) -> CmdResult<u64> {
    value
        .trim()
        .replace(',', "")
        .parse::<f64>()
        .map(|parsed| (parsed.max(0.0) * 100.0).round() as u64)
        .map_err(|_| CmdError {
            code: "ACCOUNT_SYNC_INVALID_DATA".into(),
            message: format!("{symbol} {field} 응답 형식이 올바르지 않습니다: {value:?}"),
        })
}

async fn sync_toss_strategy_positions(
    state: &AppState,
    profile: AccountProfile,
) -> CmdResult<usize> {
    if !profile.is_configured() {
        return Err(CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: "활성 Toss 프로파일 설정이 미완료되어 잔고를 동기화할 수 없습니다.".into(),
        });
    }

    let account_id = BrokerAccountId(profile.broker_account_id());
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let holdings = adapter
        .list_holdings(Some(&account_id))
        .await
        .map_err(|e| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: format!("자동매매 시작 전 Toss 보유종목 동기화 실패: {e}"),
        })?;
    let parsed_holdings: Vec<(String, String, BrokerMarket, u64, u64, u64)> = holdings
        .iter()
        .map(|holding| {
            let quantity = holding
                .quantity
                .0
                .trim()
                .replace(',', "")
                .parse::<f64>()
                .map_err(|_| CmdError {
                    code: "ACCOUNT_SYNC_INVALID_DATA".into(),
                    message: format!(
                        "{} Toss 보유수량 응답 형식이 올바르지 않습니다.",
                        holding.symbol.0
                    ),
                })?;
            let parse_money = |money: &BrokerMoney, field: &str| -> CmdResult<u64> {
                let value = money
                    .amount
                    .trim()
                    .replace(',', "")
                    .parse::<f64>()
                    .map_err(|_| CmdError {
                        code: "ACCOUNT_SYNC_INVALID_DATA".into(),
                        message: format!(
                            "{} Toss {field} 응답 형식이 올바르지 않습니다.",
                            holding.symbol.0
                        ),
                    })?;
                Ok(match money.currency {
                    BrokerCurrency::Krw => value.max(0.0).round() as u64,
                    BrokerCurrency::Usd => (value.max(0.0) * 100.0).round() as u64,
                })
            };
            Ok((
                holding.symbol.0.clone(),
                holding.symbol_name.clone(),
                holding.market,
                if quantity <= 0.0 {
                    0
                } else {
                    quantity.floor().max(1.0) as u64
                },
                parse_money(&holding.average_price, "평균매입가")?,
                parse_money(&holding.current_price, "현재가")?,
            ))
        })
        .collect::<CmdResult<_>>()?;

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
        domestic_tracker.replace(
            parsed_holdings
                .iter()
                .filter(|holding| holding.2 == BrokerMarket::Kr)
                .map(|holding| {
                    (
                        holding.0.clone(),
                        holding.1.clone(),
                        holding.3,
                        holding.4,
                        holding.5,
                    )
                }),
        );
    }
    {
        let mut overseas_tracker = state.overseas_position_tracker.lock().await;
        overseas_tracker.replace(
            parsed_holdings
                .iter()
                .filter(|holding| holding.2 == BrokerMarket::Us)
                .map(|holding| {
                    (
                        holding.0.clone(),
                        holding.1.clone(),
                        "TOSS_US".to_string(),
                        holding.3,
                        holding.4,
                        holding.5,
                    )
                }),
        );
    }

    let mut synced = 0usize;
    let mut mgr = state.strategy_manager.lock().await;
    for symbol in mgr.active_symbols() {
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Toss,
            market: if is_domestic_symbol(&symbol) {
                BrokerMarket::Kr
            } else {
                BrokerMarket::Us
            },
            symbol,
            quantity: 0,
            avg_price: 0,
        });
    }
    for holding in &parsed_holdings {
        let quantity = holding.3;
        if quantity > 0 {
            synced += 1;
        }
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Toss,
            market: holding.2,
            symbol: holding.0.clone(),
            quantity,
            avg_price: holding.4,
        });
    }
    tracing::info!(
        "Toss holdings 기반 전략 포지션 동기화 완료: {}개 보유 종목",
        synced
    );
    Ok(synced)
}

#[tauri::command]
pub async fn start_trading(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<TradingStatus> {
    let _storage_maintenance = state.storage_maintenance.lock().await;
    if state.database_manager.config_view().await.active_backend
        == crate::storage::database::StorageBackend::Database
    {
        state
            .database_manager
            .status()
            .await
            .map_err(|error| CmdError {
                code: "STORAGE_UNAVAILABLE".into(),
                message: format!(
                    "DB 저장소를 확인할 수 없어 자동매매를 시작하지 않습니다: {error}"
                ),
            })?;
    }
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
    let synced_positions = sync_strategy_positions_from_active_broker(&state, &current_cfg).await?;
    let active_profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    };
    let broker_id = active_profile
        .as_ref()
        .map(|profile| profile.broker_id)
        .unwrap_or(current_cfg.broker_id);
    let exchange_rate = *state.exchange_rate_krw.read().await;
    let initial_total_balance = match broker_id {
        BrokerId::Kis => {
            let include_overseas = state
                .strategy_manager
                .lock()
                .await
                .active_symbols()
                .iter()
                .any(|symbol| !is_domestic_symbol(symbol));
            let rest = state.rest_client.read().await.clone();
            fetch_account_risk_balance_krw(&rest, include_overseas, exchange_rate).await
        }
        BrokerId::Toss => match active_profile.as_ref() {
            Some(profile) => fetch_toss_risk_balance_krw(profile, exchange_rate).await,
            None => Err("활성 Toss 프로파일이 없습니다.".into()),
        },
    }
    .map_err(|message| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: format!("자동매매 시작 전 계좌 평가액 동기화 실패: {message}"),
    })?;
    if initial_total_balance <= 0 {
        return Err(CmdError {
            code: "ACCOUNT_BALANCE_UNAVAILABLE".into(),
            message: "계좌 총평가액이 0원 이하로 확인되어 자동매매를 시작하지 않습니다.".into(),
        });
    }
    tracing::info!(
        "자동매매 시작 전 broker-aware 계좌 동기화 완료: {}개, 총평가액={}원",
        synced_positions,
        initial_total_balance
    );

    let reconciliation_account_id = active_profile
        .as_ref()
        .map(|profile| profile.broker_account_id())
        .or_else(|| {
            (!current_cfg.broker_account_id.is_empty())
                .then(|| current_cfg.broker_account_id.clone())
        });
    state
        .order_manager
        .lock()
        .await
        .set_execution_scope(BrokerScope::new(
            broker_id,
            reconciliation_account_id.map(BrokerAccountId),
        ));

    OrderManager::confirm_pending_fills_from_broker_shared(&state.order_manager)
        .await
        .map_err(|error| CmdError {
            code: "PENDING_RECONCILIATION_FAILED".into(),
            message: format!(
                "복원된 미체결 주문을 broker와 대조하지 못해 자동매매를 시작하지 않습니다: {error}"
            ),
        })?;
    restore_risk_from_today_trades(&state.trade_store, &state.risk_manager, &state.risk_store)
        .await?;

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

fn toss_us_session_policy_from_params(params: &serde_json::Value) -> UsTradingSessionPolicy {
    let value = params
        .get("toss_us_session")
        .or_else(|| params.get("tossUsSession"))
        .and_then(|value| value.as_str());
    UsTradingSessionPolicy::parse(value)
}

fn strategy_market_open_for_symbol(
    config: &crate::trading::strategy::StrategyConfig,
    symbol: &str,
    price_source: &PriceSource,
    market_calendar: Option<&MarketCalendarOverride>,
) -> bool {
    if matches!(price_source, PriceSource::Toss(_)) && !is_domestic_symbol(symbol) {
        is_market_open_for_with_calendar_policy(
            symbol,
            market_calendar,
            toss_us_session_policy_from_params(&config.params),
        )
    } else {
        is_market_open_for_with_calendar(symbol, market_calendar)
    }
}

/// Toss 종목 현재가 조회 → (가격, 거래량) 튜플로 변환
///
/// 가격 단위는 KIS 경로와 동일하게 맞춘다: KRW는 원 단위 정수, USD는 센트(×100) 정수.
/// Toss `/api/v1/prices`는 거래량을 제공하지 않아 volume은 항상 0으로 반환한다
/// (leveraged_trend_hold 등 volume을 사용하지 않는 전략에는 영향 없음).
async fn fetch_toss_tick(profile: &AccountProfile, symbol: &str) -> anyhow::Result<(u64, u64)> {
    let normalized_symbol = symbol.trim().to_uppercase();
    if normalized_symbol.is_empty() {
        anyhow::bail!("Toss 현재가 조회 symbol이 비어 있습니다.");
    }
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(profile.account_no.clone()),
    );
    let quote = adapter
        .get_price(&BrokerSymbol(normalized_symbol.clone()))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let amount = quote.last.amount.trim().replace(',', "");
    let price_amount = amount.parse::<f64>().map_err(|e| {
        anyhow::anyhow!("Toss 현재가 파싱 실패({normalized_symbol}): {amount} ({e})")
    })?;
    if price_amount <= 0.0 {
        anyhow::bail!("Toss 현재가가 0 이하입니다: {normalized_symbol}={amount}");
    }
    let price = match quote.last.currency {
        BrokerCurrency::Krw => price_amount.round() as u64,
        BrokerCurrency::Usd => (price_amount * 100.0).round() as u64,
    };
    let volume = quote
        .volume
        .and_then(|q| q.0.trim().replace(',', "").parse::<u64>().ok())
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
    market_calendar: Option<&MarketCalendarOverride>,
) -> TickCycleResult {
    for symbol in symbols {
        // 종료 플래그 선행 확인
        if !*is_trading.lock().await {
            return TickCycleResult::Stopped;
        }

        // 해당 종목을 사용하는 활성 전략 중 현재 허용된 세션이 있는지 확인한다.
        // Toss 미국 전략은 params.toss_us_session에 따라 day/pre/regular/after/auto gate를 적용한다.
        let symbol_has_open_strategy = strategy_mgr
            .lock()
            .await
            .any_active_config_for_symbol(symbol, |config| {
                strategy_market_open_for_symbol(config, symbol, price_source, market_calendar)
            });
        if !symbol_has_open_strategy {
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
                let signals =
                    strategy_mgr
                        .lock()
                        .await
                        .on_tick_filtered(symbol, price, volume, |config| {
                            strategy_market_open_for_symbol(
                                config,
                                symbol,
                                price_source,
                                market_calendar,
                            )
                        });
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
                        Ok(crate::trading::order::SubmissionOutcome::Submitted {
                            provider_order_id,
                        }) => {
                            tracing::info!(
                                "전략 주문 제출 완료: symbol={} providerOrderId={}",
                                symbol,
                                provider_order_id
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                        }
                        Ok(crate::trading::order::SubmissionOutcome::Skipped { reason }) => {
                            tracing::debug!("전략 주문 스킵: symbol={} reason={}", symbol, reason);
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
    let mut was_running = false;

    // 레이블 없는 단순 루프 — 제어 흐름은 위→아래 순차 실행
    loop {
        let is_running = *is_trading.lock().await;

        // pending reconciliation은 자동매매 on/off와 무관하게 계속 수행한다.
        if let Err(error) = OrderManager::confirm_pending_fills_from_broker_shared(&order_mgr).await
        {
            tracing::warn!("provider 미체결 주문 대조 실패: {}", error);
        }

        // ── Phase 1: 자동매매 비활성 → 5초 대기 후 재확인 ──────────
        if !is_running {
            if was_running {
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

        // ── Phase 4: 날짜 변경 시 일별 초기화 ──────────────────────
        let today = chrono::Local::now().date_naive();
        if today != last_reset_date {
            last_reset_date = today;
            let reset = risk_mgr.lock().await.reset_if_new_day();
            let mut order_mgr_guard = order_mgr.lock().await;
            order_mgr_guard.reset_day();
            if reset {
                order_mgr_guard.persist_risk_runtime().await;
            }
            drop(order_mgr_guard);
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

        let all_markets_closed = {
            let mgr = strategy_mgr.lock().await;
            symbols.iter().all(|symbol| {
                !mgr.any_active_config_for_symbol(symbol, |config| {
                    strategy_market_open_for_symbol(
                        config,
                        symbol,
                        &price_source,
                        market_calendar.as_ref(),
                    )
                })
            })
        };

        if all_markets_closed {
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
        let total_balance_result = match &price_source {
            PriceSource::Toss(profile) => fetch_toss_risk_balance_krw(profile, exchange_rate).await,
            PriceSource::Kis => {
                fetch_account_risk_balance_krw(
                    &rest,
                    symbols.iter().any(|s| !is_domestic_symbol(s)),
                    exchange_rate,
                )
                .await
            }
        };
        let total_balance_krw = match total_balance_result {
            Ok(total) if total > 0 => {
                order_mgr.lock().await.clear_account_sync_suspension();
                total
            }
            Ok(_) => {
                tracing::error!("계좌 총평가액이 0원 이하이므로 이번 자동주문 주기를 차단합니다.");
                order_mgr
                    .lock()
                    .await
                    .suspend_buying_for_account_sync("총평가액이 0원 이하입니다.".into())
                    .await;
                continue;
            }
            Err(error) => {
                tracing::error!(
                    "계좌 스냅샷 갱신 실패로 이번 자동주문 주기를 차단합니다: {}",
                    error
                );
                order_mgr
                    .lock()
                    .await
                    .suspend_buying_for_account_sync(error)
                    .await;
                continue;
            }
        };

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
