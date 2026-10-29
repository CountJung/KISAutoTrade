//! Toss order preflight inputs, views and read-only evaluation.
use super::*;

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

pub(crate) fn toss_currency_from_view(money: &BrokerMoneyView) -> BrokerCurrency {
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
    let safety = get_toss_stock_safety_for_profile(symbol.clone(), profile.clone()).await?;
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
            commission_rate_ratio: commission_rate.clone(),
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
    if !profile.live_trading_consent {
        warnings.push("Toss 실거래 동의가 저장되지 않아 주문 제출은 차단됩니다.".into());
    }

    let safety_ok = !(side == BrokerOrderSide::Buy && safety.buy_blocked);
    let order_adapter_supported = true;
    let liquidity_ok = decision.liquidity_ok;
    let can_submit =
        liquidity_ok && safety_ok && order_adapter_supported && profile.live_trading_consent;

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
