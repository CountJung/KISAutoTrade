use super::*;

// ────────────────────────────────────────────────────────────────────
// 계좌 프로파일 관리
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProfileView {
    pub id: String,
    pub broker_id: BrokerId,
    pub broker_account_id: String,
    pub name: String,
    pub is_paper_trading: bool,
    pub live_trading_consent: bool,
    pub app_key_masked: String,
    pub account_no: String,
    pub is_active: bool,
    pub is_configured: bool,
}

pub(crate) fn profile_to_view(p: &AccountProfile, active_id: &Option<String>) -> ProfileView {
    let masked = if p.app_key.len() > 6 {
        format!("{}****", &p.app_key[..6])
    } else if p.app_key.is_empty() {
        "(미설정)".into()
    } else {
        "****".into()
    };
    ProfileView {
        id: p.id.clone(),
        broker_id: p.broker_id,
        broker_account_id: p.broker_account_id(),
        name: p.name.clone(),
        is_paper_trading: p.is_paper_trading,
        live_trading_consent: p.live_trading_consent,
        app_key_masked: masked,
        account_no: p.account_no.clone(),
        is_active: active_id.as_deref() == Some(&p.id),
        is_configured: p.is_configured(),
    }
}

#[tauri::command]
pub async fn list_profiles(state: State<'_, AppState>) -> CmdResult<Vec<ProfileView>> {
    let profiles = state.profiles.read().await;
    Ok(profiles
        .profiles
        .iter()
        .map(|p| profile_to_view(p, &profiles.active_id))
        .collect())
}

#[derive(Debug, Deserialize)]
pub struct AddProfileInput {
    #[serde(default = "default_input_broker_id")]
    pub broker_id: BrokerId,
    pub name: String,
    pub is_paper_trading: bool,
    #[serde(default)]
    pub live_trading_consent: bool,
    pub app_key: String,
    pub app_secret: String,
    pub account_no: String,
}

fn default_input_broker_id() -> BrokerId {
    BrokerId::Kis
}

#[tauri::command]
pub async fn add_profile(
    input: AddProfileInput,
    state: State<'_, AppState>,
) -> CmdResult<ProfileView> {
    let _strategy_update = state.strategy_update_lock.lock().await;
    let profile = AccountProfile::new(
        input.name,
        input.is_paper_trading,
        input.app_key,
        input.app_secret,
        input.account_no,
    );
    let profile = AccountProfile {
        broker_id: input.broker_id,
        live_trading_consent: input.live_trading_consent,
        ..profile
    };

    let (view, is_first) = {
        let mut profiles = state.profiles.write().await;
        let was_empty = profiles.profiles.is_empty();
        let added = profiles.add(profile);
        let view = profile_to_view(&added, &profiles.active_id);
        (view, was_empty)
    };

    // 첫 번째 프로파일이면 자동 활성화
    if is_first {
        apply_active_profile(&state).await?;
    }

    save_profiles(&state).await?;
    Ok(view)
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileInput {
    pub id: String,
    pub broker_id: Option<BrokerId>,
    pub name: Option<String>,
    pub is_paper_trading: Option<bool>,
    pub live_trading_consent: Option<bool>,
    /// 빈 문자열 = 변경 안 함
    pub app_key: Option<String>,
    /// 빈 문자열 = 변경 안 함
    pub app_secret: Option<String>,
    pub account_no: Option<String>,
}

#[tauri::command]
pub async fn update_profile(
    input: UpdateProfileInput,
    state: State<'_, AppState>,
) -> CmdResult<ProfileView> {
    let _strategy_update = state.strategy_update_lock.lock().await;
    let is_active = state.profiles.read().await.active_id.as_deref() == Some(input.id.as_str());
    if is_active && *state.is_trading.lock().await {
        return Err(CmdError {
            code: "TRADING_RUNNING".into(),
            message: "자동매매 실행 중에는 활성 프로파일을 수정할 수 없습니다.".into(),
        });
    }
    let view = {
        let mut profiles = state.profiles.write().await;
        let updated = profiles
            .update(
                &input.id,
                input.broker_id,
                input.name,
                input.is_paper_trading,
                input.live_trading_consent,
                input.app_key,
                input.app_secret,
                input.account_no,
            )
            .ok_or_else(|| CmdError {
                code: "PROFILE_NOT_FOUND".into(),
                message: format!("프로파일을 찾을 수 없습니다: {}", input.id),
            })?;
        profile_to_view(&updated, &profiles.active_id)
    };

    // 수정된 프로파일이 현재 활성이면 즉시 반영
    let is_active = {
        let profiles = state.profiles.read().await;
        profiles.active_id.as_deref() == Some(&input.id)
    };
    if is_active {
        apply_active_profile(&state).await?;
    }

    save_profiles(&state).await?;
    Ok(view)
}

#[tauri::command]
pub async fn delete_profile(id: String, state: State<'_, AppState>) -> CmdResult<()> {
    let _strategy_update = state.strategy_update_lock.lock().await;
    let is_active = state.profiles.read().await.active_id.as_deref() == Some(id.as_str());
    if is_active && *state.is_trading.lock().await {
        return Err(CmdError {
            code: "TRADING_RUNNING".into(),
            message: "자동매매 실행 중에는 활성 프로파일을 삭제할 수 없습니다.".into(),
        });
    }
    let deleted = {
        let mut profiles = state.profiles.write().await;
        profiles.delete(&id)
    };

    if !deleted {
        return Err(CmdError {
            code: "PROFILE_NOT_FOUND".into(),
            message: format!("프로파일을 찾을 수 없습니다: {}", id),
        });
    }

    apply_active_profile(&state).await?;
    save_profiles(&state).await?;
    Ok(())
}

#[tauri::command]
pub async fn set_active_profile(
    id: String,
    state: State<'_, AppState>,
) -> CmdResult<AppConfigView> {
    let _strategy_update = state.strategy_update_lock.lock().await;
    if *state.is_trading.lock().await {
        return Err(CmdError {
            code: "TRADING_RUNNING".into(),
            message: "자동매매 실행 중에는 활성 프로파일을 전환할 수 없습니다.".into(),
        });
    }
    let ok = {
        let mut profiles = state.profiles.write().await;
        profiles.set_active(&id)
    };

    if !ok {
        return Err(CmdError {
            code: "PROFILE_NOT_FOUND".into(),
            message: format!("프로파일을 찾을 수 없습니다: {}", id),
        });
    }

    apply_active_profile(&state).await?;
    save_profiles(&state).await?;
    drop(_strategy_update);
    get_app_config(state).await
}

/// 현재 active_id 기반으로 config + rest_client + 전략 설정 교체
pub(super) async fn apply_active_profile(state: &AppState) -> CmdResult<()> {
    let (new_config, active_id) = {
        let profiles = state.profiles.read().await;
        let cfg = match profiles.get_active() {
            Some(p) => AppConfig::from_profile(p, &state.discord_config),
            None => AppConfig::empty(&state.discord_config),
        };
        (cfg, profiles.active_id.clone())
    };

    let new_client = make_rest_client(&new_config);

    *state.config.write().await = new_config;
    *state.rest_client.write().await = new_client;

    // 이전 프로파일의 보유 포지션이 새 broker/account scope로 새어들지 않게 초기화한다.
    // 다음 잔고 동기화(replace)와 수동 주문 직전 refresh에서 새 계좌 스냅샷으로 채워진다.
    state.position_tracker.lock().await.clear();
    state.overseas_position_tracker.lock().await.clear();

    // 프로파일 전환 시 해당 프로파일의 전략 설정 로드 (재시작 없이도 반영)
    let active_scope = {
        let cfg = state.config.read().await.clone();
        let account_id = if cfg.broker_account_id.is_empty() {
            None
        } else {
            Some(cfg.broker_account_id.clone())
        };
        (cfg.broker_id, account_id)
    };
    state
        .order_manager
        .lock()
        .await
        .set_execution_scope(BrokerScope::new(
            active_scope.0,
            active_scope.1.clone().map(BrokerAccountId),
        ));
    if let Some(pid) = &active_id {
        let saved = state
            .strategy_store
            .load(pid)
            .await
            .map_err(CmdError::from)?;
        let mut mgr = state.strategy_manager.lock().await;
        mgr.apply_saved_configs_for_scope(&saved, active_scope.0, active_scope.1.clone());
        tracing::info!(
            "프로파일 전환 — 전략 설정 복원: 프로파일 '{}', {}개 전략",
            pid,
            saved.len()
        );
    } else {
        let mut mgr = state.strategy_manager.lock().await;
        mgr.apply_saved_configs_for_scope(&[], active_scope.0, active_scope.1.clone());
    }

    tracing::info!("활성 프로파일 적용 완료");
    Ok(())
}

/// profiles.json 비동기 저장
pub(super) async fn save_profiles(state: &AppState) -> CmdResult<()> {
    let profiles = state.profiles.read().await.clone();
    profiles
        .save(&state.profiles_path)
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 잔고 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct BalanceResult {
    pub items: Vec<BalanceItem>,
    pub summary: Option<BalanceSummary>,
}

#[tauri::command]
pub async fn get_balance(state: State<'_, AppState>) -> CmdResult<BalanceResult> {
    let client = state.rest_client.read().await.clone();
    match client.get_balance().await {
        Ok(resp) => {
            tracing::info!(
                "잔고 조회 성공: 보유종목 {}개, 총평가금액 {}원",
                resp.items.len(),
                resp.summary
                    .as_ref()
                    .map(|s| s.tot_evlu_amt.as_str())
                    .unwrap_or("미제공")
            );
            // 잔고 응답의 종목코드+이름 데이터 자동 수집
            state
                .stock_store
                .upsert_many(
                    resp.items
                        .iter()
                        .map(|i| (i.pdno.clone(), i.prdt_name.clone())),
                )
                .await;
            // 앱 재시작 후 position_tracker가 비어있으면 잔고 응답으로 복원
            {
                let mut tracker = state.position_tracker.lock().await;
                tracker.replace(resp.items.iter().map(|i| {
                    (
                        i.pdno.clone(),
                        i.prdt_name.clone(),
                        i.hldg_qty.parse::<u64>().unwrap_or(0),
                        i.pchs_avg_pric.parse::<f64>().unwrap_or(0.0) as u64,
                        i.prpr.parse::<u64>().unwrap_or(0),
                    )
                }));
            }
            Ok(BalanceResult {
                items: resp.items,
                summary: resp.summary,
            })
        }
        Err(e) => {
            tracing::error!("잔고 조회 실패: {}", e);
            Err(CmdError::from(e))
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 해외 잔고 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OverseasBalanceResult {
    pub items: Vec<OverseasBalanceItem>,
    pub summary: Option<OverseasBalanceSummary>,
}

#[tauri::command]
pub async fn get_overseas_balance(state: State<'_, AppState>) -> CmdResult<OverseasBalanceResult> {
    let client = state.rest_client.read().await.clone();
    match client.get_overseas_balance().await {
        Ok(resp) => {
            tracing::info!("해외 잔고 조회 성공: 보유종목 {}개", resp.items.len());
            // 해외 잔고는 국내 position_tracker에 혼입하지 않고 별도 tracker에만 복원한다.
            {
                let mut tracker = state.overseas_position_tracker.lock().await;
                tracker.replace(resp.items.iter().map(|i| {
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
            Ok(OverseasBalanceResult {
                items: resp.items,
                summary: resp.summary,
            })
        }
        Err(e) => {
            tracing::error!("해외 잔고 조회 실패: {}", e);
            Err(CmdError::from(e))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerMoneyView {
    pub amount: String,
    pub currency: BrokerCurrency,
}

impl From<BrokerMoney> for BrokerMoneyView {
    fn from(money: BrokerMoney) -> Self {
        Self {
            amount: money.amount,
            currency: money.currency,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerHoldingView {
    pub broker_id: BrokerId,
    pub account_id: Option<String>,
    pub market: BrokerMarket,
    pub symbol: String,
    pub symbol_name: String,
    pub quantity: String,
    pub average_price: BrokerMoneyView,
    pub current_price: BrokerMoneyView,
    pub unrealized_pnl: Option<BrokerMoneyView>,
}

impl From<BrokerHolding> for BrokerHoldingView {
    fn from(holding: BrokerHolding) -> Self {
        Self {
            broker_id: holding.broker,
            account_id: holding.account_id.map(|id| id.0),
            market: holding.market,
            symbol: holding.symbol.0,
            symbol_name: holding.symbol_name,
            quantity: holding.quantity.0,
            average_price: holding.average_price.into(),
            current_price: holding.current_price.into(),
            unrealized_pnl: holding.unrealized_pnl.map(Into::into),
        }
    }
}

pub async fn list_broker_holdings_for_profile(
    profile: AccountProfile,
    rest_client: Arc<KisRestClient>,
) -> Result<Vec<BrokerHoldingView>, CmdError> {
    let account_id = BrokerAccountId(profile.broker_account_id());
    let holdings = match profile.broker_id {
        BrokerId::Kis => {
            let adapter = KisBrokerAdapter::new(rest_client);
            adapter.list_holdings(Some(&account_id)).await
        }
        BrokerId::Toss => {
            let adapter = TossBrokerAdapter::with_credentials(
                TossBrokerAdapter::DEFAULT_BASE_URL,
                profile.app_key,
                profile.app_secret,
                Some(profile.account_no),
            );
            adapter.list_holdings(Some(&account_id)).await
        }
    }
    .map_err(|e| CmdError {
        code: "BROKER_HOLDINGS_ERROR".into(),
        message: e.to_string(),
    })?;

    let mut views: Vec<BrokerHoldingView> =
        holdings.into_iter().map(BrokerHoldingView::from).collect();
    views.sort_by(|a, b| {
        broker_market_sort_key(a.market)
            .cmp(&broker_market_sort_key(b.market))
            .then_with(|| a.symbol.cmp(&b.symbol))
    });
    Ok(views)
}

fn broker_market_sort_key(market: BrokerMarket) -> u8 {
    match market {
        BrokerMarket::Kr => 0,
        BrokerMarket::Us => 1,
    }
}

#[tauri::command]
pub async fn get_broker_holdings(state: State<'_, AppState>) -> CmdResult<Vec<BrokerHoldingView>> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    };
    let Some(profile) = profile else {
        return Ok(Vec::new());
    };
    let rest_client = state.rest_client.read().await.clone();
    list_broker_holdings_for_profile(profile, rest_client).await
}
