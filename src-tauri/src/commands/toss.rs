use super::*;

#[derive(Debug, Serialize)]
pub struct TossConnectionStep {
    pub id: String,
    pub label: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct TossAccountOptionView {
    pub account_seq: String,
    pub account_no_masked: String,
    pub account_type: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct TossAccountLookupInput {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize)]
pub struct TossConnectionDiagnostic {
    pub profile_id: String,
    pub profile_name: String,
    pub broker_id: BrokerId,
    pub account_seq: String,
    pub openapi_title: Option<String>,
    pub openapi_version: Option<String>,
    pub openapi_server: Option<String>,
    pub openapi_paths_count: Option<usize>,
    pub token_type: Option<String>,
    pub token_expires_at: Option<String>,
    pub accounts_count: Option<usize>,
    pub matched_account_no: Option<String>,
    pub holdings_count: Option<usize>,
    pub buying_power_krw: Option<String>,
    pub buying_power_usd: Option<String>,
    pub commissions_count: Option<usize>,
    pub sellable_quantity_symbol: Option<String>,
    pub sellable_quantity: Option<String>,
    pub is_ready: bool,
    pub steps: Vec<TossConnectionStep>,
    pub issues: Vec<String>,
}

fn toss_diag_step(
    id: impl Into<String>,
    label: impl Into<String>,
    ok: bool,
    message: impl Into<String>,
) -> TossConnectionStep {
    TossConnectionStep {
        id: id.into(),
        label: label.into(),
        ok,
        message: message.into(),
    }
}

fn mask_toss_account_no(account_no: &str) -> String {
    let suffix: String = account_no
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if suffix.is_empty() {
        "(계좌번호 없음)".to_string()
    } else {
        format!("****{suffix}")
    }
}

fn toss_account_option(account: crate::broker::toss::TossAccount) -> TossAccountOptionView {
    let account_seq = account.account_seq.to_string();
    let account_no_masked = mask_toss_account_no(&account.account_no);
    let label = format!(
        "accountSeq {} · {} · {}",
        account_seq, account_no_masked, account.account_type
    );
    TossAccountOptionView {
        account_seq,
        account_no_masked,
        account_type: account.account_type,
        label,
    }
}

pub(crate) async fn lookup_toss_accounts_with_credentials(
    client_id: String,
    client_secret: String,
) -> CmdResult<Vec<TossAccountOptionView>> {
    let client_id = client_id.trim();
    let client_secret = client_secret.trim();
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(CmdError {
            code: "MISSING_CREDENTIALS".into(),
            message: "토스증권 Client ID와 Client Secret을 모두 입력하세요.".into(),
        });
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        client_id.to_string(),
        client_secret.to_string(),
        None::<String>,
    );
    adapter
        .list_accounts()
        .await
        .map(|accounts| accounts.into_iter().map(toss_account_option).collect())
        .map_err(|e| CmdError {
            code: "TOSS_ACCOUNTS_ERROR".into(),
            message: e.to_string(),
        })
}

#[tauri::command]
pub async fn list_toss_accounts(
    input: TossAccountLookupInput,
) -> CmdResult<Vec<TossAccountOptionView>> {
    lookup_toss_accounts_with_credentials(input.client_id, input.client_secret).await
}

#[tauri::command]
pub async fn list_toss_profile_accounts(
    profile_id: String,
    state: State<'_, AppState>,
) -> CmdResult<Vec<TossAccountOptionView>> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .cloned()
            .ok_or_else(|| CmdError {
                code: "NOT_FOUND".into(),
                message: format!("프로파일을 찾을 수 없습니다: {profile_id}"),
            })?
    };

    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_MISMATCH".into(),
            message: "토스증권 프로파일만 accountSeq 목록을 조회할 수 있습니다.".into(),
        });
    }

    lookup_toss_accounts_with_credentials(profile.app_key, profile.app_secret).await
}

pub(crate) async fn run_toss_connection_diagnostic(
    profile: AccountProfile,
) -> TossConnectionDiagnostic {
    let account_seq = profile.broker_account_id();
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(account_seq.clone()),
    );

    let mut steps = Vec::new();
    let mut issues = Vec::new();
    let mut openapi_title = None;
    let mut openapi_version = None;
    let mut openapi_server = None;
    let mut openapi_paths_count = None;
    let mut token_type = None;
    let mut token_expires_at = None;
    let mut accounts_count = None;
    let mut matched_account_no = None;
    let mut holdings_count = None;
    let mut first_holding_symbol = None;
    let mut buying_power_krw = None;
    let mut buying_power_usd = None;
    let mut commissions_count = None;
    let mut sellable_quantity_symbol = None;
    let mut sellable_quantity = None;

    match adapter.openapi_overview().await {
        Ok(overview) => {
            let ok = overview.server == TossBrokerAdapter::DEFAULT_BASE_URL
                && !overview.version.is_empty()
                && overview.paths_count > 0;
            if !ok {
                issues.push("토스증권 OpenAPI 스펙 메타데이터가 예상과 다릅니다.".into());
            }
            steps.push(toss_diag_step(
                "openapi",
                "OpenAPI 스펙",
                ok,
                format!(
                    "{} v{} · paths {}",
                    overview.title, overview.version, overview.paths_count
                ),
            ));
            openapi_title = Some(overview.title);
            openapi_version = Some(overview.version);
            openapi_server = Some(overview.server);
            openapi_paths_count = Some(overview.paths_count);
        }
        Err(e) => {
            let message = e.to_string();
            issues.push(message.clone());
            steps.push(toss_diag_step("openapi", "OpenAPI 스펙", false, message));
        }
    }

    let credentials_present =
        !profile.app_key.trim().is_empty() && !profile.app_secret.trim().is_empty();
    let account_seq_valid = account_seq.trim().parse::<i64>().is_ok();

    if !credentials_present {
        let message = "토스증권 client_id/client_secret이 설정되지 않았습니다.".to_string();
        issues.push(message.clone());
        steps.push(toss_diag_step("token", "토큰 발급", false, message));
    } else {
        match adapter.check_token().await {
            Ok(token) => {
                token_type = Some(token.token_type.clone());
                token_expires_at = Some(token.expires_at.to_rfc3339());
                steps.push(toss_diag_step(
                    "token",
                    "토큰 발급",
                    true,
                    format!("{} token · 만료 {}", token.token_type, token.expires_at),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("token", "토큰 발급", false, message));
            }
        }
    }

    if credentials_present {
        match adapter.list_accounts().await {
            Ok(accounts) => {
                accounts_count = Some(accounts.len());
                matched_account_no = accounts
                    .iter()
                    .find(|account| account.account_seq.to_string() == account_seq)
                    .map(|account| account.account_no.clone());
                let ok = account_seq.trim().is_empty()
                    || matched_account_no.is_some()
                    || !account_seq_valid;
                if !ok {
                    issues.push(format!(
                        "저장된 accountSeq({account_seq})와 일치하는 토스 계좌를 찾지 못했습니다."
                    ));
                }
                let message = match &matched_account_no {
                    Some(account_no) => {
                        format!("{}개 계좌 조회 · 저장 계좌 {}", accounts.len(), account_no)
                    }
                    None if account_seq.trim().is_empty() => {
                        format!("{}개 계좌 조회 · accountSeq를 저장하세요", accounts.len())
                    }
                    None if !account_seq_valid => {
                        format!(
                            "{}개 계좌 조회 · accountSeq는 숫자여야 합니다",
                            accounts.len()
                        )
                    }
                    None => format!("{}개 계좌 조회 · 저장 계좌 불일치", accounts.len()),
                };
                steps.push(toss_diag_step("accounts", "계좌 조회", ok, message));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("accounts", "계좌 조회", false, message));
            }
        }
    } else {
        steps.push(toss_diag_step(
            "accounts",
            "계좌 조회",
            false,
            "토큰 발급 전이라 계좌 조회를 건너뛰었습니다.",
        ));
    }

    if account_seq.trim().is_empty() {
        let message = "토스증권 accountSeq가 설정되지 않았습니다.".to_string();
        issues.push(message.clone());
        steps.push(toss_diag_step("holdings", "잔고 조회", false, message));
    } else if !account_seq_valid {
        let message = "토스증권 accountSeq는 숫자여야 합니다.".to_string();
        issues.push(message.clone());
        steps.push(toss_diag_step("holdings", "잔고 조회", false, message));
    } else if credentials_present {
        let account_id = BrokerAccountId(account_seq.clone());
        match adapter.list_holdings(Some(&account_id)).await {
            Ok(holdings) => {
                holdings_count = Some(holdings.len());
                first_holding_symbol = holdings.first().map(|holding| holding.symbol.0.clone());
                steps.push(toss_diag_step(
                    "holdings",
                    "잔고 조회",
                    true,
                    format!("{}개 보유 종목 조회", holdings.len()),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("holdings", "잔고 조회", false, message));
            }
        }
    } else {
        steps.push(toss_diag_step(
            "holdings",
            "잔고 조회",
            false,
            "토큰 발급 전이라 잔고 조회를 건너뛰었습니다.",
        ));
    }

    if credentials_present && account_seq_valid && !account_seq.trim().is_empty() {
        match adapter
            .get_buying_power(Some(&account_seq), BrokerCurrency::Krw)
            .await
        {
            Ok(power) => {
                buying_power_krw = Some(power.cash_buying_power.clone());
                steps.push(toss_diag_step(
                    "buyingPowerKrw",
                    "매수가능금액(KRW)",
                    true,
                    format!("{} {}", power.cash_buying_power, power.currency),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step(
                    "buyingPowerKrw",
                    "매수가능금액(KRW)",
                    false,
                    message,
                ));
            }
        }

        match adapter
            .get_buying_power(Some(&account_seq), BrokerCurrency::Usd)
            .await
        {
            Ok(power) => {
                buying_power_usd = Some(power.cash_buying_power.clone());
                steps.push(toss_diag_step(
                    "buyingPowerUsd",
                    "매수가능금액(USD)",
                    true,
                    format!("{} {}", power.cash_buying_power, power.currency),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step(
                    "buyingPowerUsd",
                    "매수가능금액(USD)",
                    false,
                    message,
                ));
            }
        }

        match adapter.list_commissions(Some(&account_seq)).await {
            Ok(commissions) => {
                commissions_count = Some(commissions.len());
                steps.push(toss_diag_step(
                    "commissions",
                    "수수료 조회",
                    true,
                    format!("{}개 수수료 정책 조회", commissions.len()),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("commissions", "수수료 조회", false, message));
            }
        }

        if let Some(symbol) = &first_holding_symbol {
            let broker_symbol = BrokerSymbol(symbol.clone());
            match adapter
                .get_sellable_quantity(Some(&account_seq), &broker_symbol)
                .await
            {
                Ok(quantity) => {
                    sellable_quantity_symbol = Some(symbol.clone());
                    sellable_quantity = Some(quantity.sellable_quantity.clone());
                    steps.push(toss_diag_step(
                        "sellableQuantity",
                        "매도가능수량",
                        true,
                        format!("{}: {}", symbol, quantity.sellable_quantity),
                    ));
                }
                Err(e) => {
                    let message = e.to_string();
                    issues.push(message.clone());
                    steps.push(toss_diag_step(
                        "sellableQuantity",
                        "매도가능수량",
                        false,
                        message,
                    ));
                }
            }
        } else {
            steps.push(toss_diag_step(
                "sellableQuantity",
                "매도가능수량",
                true,
                "보유 종목이 없어 매도가능수량 조회를 건너뛰었습니다.",
            ));
        }
    }

    let is_ready = issues.is_empty() && steps.iter().all(|step| step.ok);

    TossConnectionDiagnostic {
        profile_id: profile.id,
        profile_name: profile.name,
        broker_id: BrokerId::Toss,
        account_seq,
        openapi_title,
        openapi_version,
        openapi_server,
        openapi_paths_count,
        token_type,
        token_expires_at,
        accounts_count,
        matched_account_no,
        holdings_count,
        buying_power_krw,
        buying_power_usd,
        commissions_count,
        sellable_quantity_symbol,
        sellable_quantity,
        is_ready,
        steps,
        issues,
    }
}

#[tauri::command]
pub async fn check_toss_profile_connection(
    profile_id: String,
    state: State<'_, AppState>,
) -> CmdResult<TossConnectionDiagnostic> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .cloned()
            .ok_or_else(|| CmdError {
                code: "NOT_FOUND".into(),
                message: format!("프로파일을 찾을 수 없습니다: {profile_id}"),
            })?
    };

    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_MISMATCH".into(),
            message: "토스증권 프로파일만 Toss 연결 진단을 실행할 수 있습니다.".into(),
        });
    }

    Ok(run_toss_connection_diagnostic(profile).await)
}

#[derive(Debug, Clone, Deserialize)]
pub struct TossOrderPreflightInput {
    pub symbol: String,
    pub side: String,
    pub quantity: String,
    pub price: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderPreflightView {
    pub broker_id: BrokerId,
    pub account_seq: String,
    pub symbol: String,
    pub market: BrokerMarket,
    pub side: BrokerOrderSide,
    pub quantity: String,
    pub price: BrokerMoneyView,
    pub price_source: String,
    pub buying_power: Option<BrokerMoneyView>,
    pub sellable_quantity: Option<String>,
    pub commission_rate: Option<String>,
    pub gross_amount: BrokerMoneyView,
    pub estimated_commission: Option<BrokerMoneyView>,
    pub required_cash: Option<BrokerMoneyView>,
    pub liquidity_ok: bool,
    pub safety_ok: bool,
    pub order_adapter_supported: bool,
    pub can_submit: bool,
    pub blocked_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

fn parse_toss_order_side(side: &str) -> CmdResult<BrokerOrderSide> {
    match side.trim().to_ascii_lowercase().as_str() {
        "buy" => Ok(BrokerOrderSide::Buy),
        "sell" => Ok(BrokerOrderSide::Sell),
        other => Err(CmdError {
            code: "INVALID_SIDE".into(),
            message: format!("알 수 없는 Toss 주문 방향: {other}"),
        }),
    }
}

fn toss_currency_from_view(money: &BrokerMoneyView) -> BrokerCurrency {
    money.currency
}

fn toss_market_country(market: BrokerMarket) -> &'static str {
    match market {
        BrokerMarket::Kr => "KR",
        BrokerMarket::Us => "US",
    }
}

fn select_toss_commission(
    commissions: &[TossCommission],
    market: BrokerMarket,
) -> Option<&TossCommission> {
    let country = toss_market_country(market);
    commissions
        .iter()
        .find(|commission| commission.market_country.eq_ignore_ascii_case(country))
        .or_else(|| commissions.first())
}

pub async fn check_toss_order_preflight_for_profile(
    input: TossOrderPreflightInput,
    profile: AccountProfile,
) -> CmdResult<TossOrderPreflightView> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 주문 전 검증은 Toss 활성 프로파일에서만 사용할 수 있습니다.".into(),
        });
    }

    let account_seq = profile.broker_account_id();
    if account_seq.trim().is_empty() {
        return Err(CmdError {
            code: "CONFIG_NOT_READY".into(),
            message: "토스증권 accountSeq가 설정되지 않았습니다.".into(),
        });
    }

    let symbol = normalize_toss_symbol(input.symbol)?;
    let side = parse_toss_order_side(&input.side)?;
    let quantity = input.quantity.trim().replace(',', "");
    if parse_decimal_amount(&quantity).unwrap_or(0.0) <= 0.0 {
        return Err(CmdError {
            code: "INVALID_QUANTITY".into(),
            message: "Toss 주문 전 검증 수량은 0보다 커야 합니다.".into(),
        });
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(account_seq.clone()),
    );

    let snapshot = get_toss_market_snapshot_for_profile(symbol.clone(), profile.clone()).await?;
    let safety = get_toss_stock_safety_for_profile(symbol.clone(), profile).await?;
    let currency = toss_currency_from_view(&snapshot.price);
    let input_price = input.price.as_deref().and_then(parse_decimal_amount);
    let snapshot_price = parse_decimal_amount(&snapshot.price.amount).unwrap_or(0.0);
    let (price_amount, price_source) = match input_price.filter(|value| *value > 0.0) {
        Some(value) => (format_money_amount(value, currency), "input".to_string()),
        None => (
            format_money_amount(snapshot_price, currency),
            "snapshot".to_string(),
        ),
    };
    let price = BrokerMoney {
        amount: price_amount,
        currency,
    };

    let commissions = adapter
        .list_commissions(Some(&account_seq))
        .await
        .map_err(|e| CmdError {
            code: "TOSS_PREFLIGHT_COMMISSIONS_ERROR".into(),
            message: e.to_string(),
        })?;
    let commission_rate = select_toss_commission(&commissions, snapshot.market)
        .map(|commission| commission.commission_rate.clone());

    let (buying_power, sellable_quantity) = match side {
        BrokerOrderSide::Buy => {
            let power = adapter
                .get_buying_power(Some(&account_seq), currency)
                .await
                .map_err(|e| CmdError {
                    code: "TOSS_PREFLIGHT_BUYING_POWER_ERROR".into(),
                    message: e.to_string(),
                })?
                .money()
                .map_err(|e| CmdError {
                    code: "TOSS_PREFLIGHT_BUYING_POWER_MAPPING_ERROR".into(),
                    message: e.to_string(),
                })?;
            (Some(power), None)
        }
        BrokerOrderSide::Sell => {
            let qty = adapter
                .get_sellable_quantity(Some(&account_seq), &BrokerSymbol(symbol.clone()))
                .await
                .map_err(|e| CmdError {
                    code: "TOSS_PREFLIGHT_SELLABLE_ERROR".into(),
                    message: e.to_string(),
                })?
                .quantity();
            (None, Some(qty))
        }
    };

    let decision = evaluate_order_preflight(
        &OrderPreflightInput {
            side,
            quantity: BrokerQuantity(quantity.clone()),
            price: price.clone(),
        },
        &OrderPreflightConstraints {
            buying_power: buying_power.clone(),
            sellable_quantity: sellable_quantity.clone(),
            commission_rate_percent: commission_rate.clone(),
        },
    );

    let mut blocked_reasons = decision.blocked_reasons;
    if let Some(reason) = safety.buy_block_reason.as_ref() {
        if side == BrokerOrderSide::Buy {
            blocked_reasons.push(reason.clone());
        }
    }

    let mut warnings = Vec::new();
    if commission_rate.is_none() {
        warnings
            .push("시장과 일치하는 Toss 수수료 정책을 찾지 못해 수수료 0으로 추정했습니다.".into());
    }
    warnings.push("Toss 주문 생성 adapter는 아직 소액 검증 gate 전이라 제출이 차단됩니다.".into());

    let safety_ok = !(side == BrokerOrderSide::Buy && safety.buy_blocked);
    let order_adapter_supported = false;
    let liquidity_ok = decision.liquidity_ok;
    let can_submit = liquidity_ok && safety_ok && order_adapter_supported;

    Ok(TossOrderPreflightView {
        broker_id: BrokerId::Toss,
        account_seq,
        symbol,
        market: snapshot.market,
        side,
        quantity,
        price: price.into(),
        price_source,
        buying_power: buying_power.map(Into::into),
        sellable_quantity: sellable_quantity.map(|quantity| quantity.0),
        commission_rate,
        gross_amount: decision.gross_amount.into(),
        estimated_commission: decision.estimated_commission.map(Into::into),
        required_cash: decision.required_cash.map(Into::into),
        liquidity_ok,
        safety_ok,
        order_adapter_supported,
        can_submit,
        blocked_reasons,
        warnings,
    })
}

#[tauri::command]
pub async fn check_toss_order_preflight(
    input: TossOrderPreflightInput,
    state: State<'_, AppState>,
) -> CmdResult<TossOrderPreflightView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    check_toss_order_preflight_for_profile(input, profile).await
}
