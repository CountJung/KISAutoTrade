use super::*;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderbookView {
    pub timestamp: Option<String>,
    pub currency: String,
    pub asks: Vec<TossOrderbookEntry>,
    pub bids: Vec<TossOrderbookEntry>,
}

impl From<TossOrderbookResponse> for TossOrderbookView {
    fn from(value: TossOrderbookResponse) -> Self {
        Self {
            timestamp: value.timestamp,
            currency: value.currency,
            asks: value.asks,
            bids: value.bids,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossTradeView {
    pub price: String,
    pub volume: String,
    pub timestamp: String,
    pub currency: String,
}

impl From<TossTrade> for TossTradeView {
    fn from(value: TossTrade) -> Self {
        Self {
            price: value.price,
            volume: value.volume,
            timestamp: value.timestamp,
            currency: value.currency,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossPriceLimitView {
    pub timestamp: String,
    pub upper_limit_price: Option<String>,
    pub lower_limit_price: Option<String>,
    pub currency: String,
}

impl From<TossPriceLimitResponse> for TossPriceLimitView {
    fn from(value: TossPriceLimitResponse) -> Self {
        Self {
            timestamp: value.timestamp,
            upper_limit_price: value.upper_limit_price,
            lower_limit_price: value.lower_limit_price,
            currency: value.currency,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketSnapshotView {
    pub broker_id: BrokerId,
    pub market: BrokerMarket,
    pub symbol: String,
    pub timestamp: Option<String>,
    pub price: BrokerMoneyView,
    pub orderbook: TossOrderbookView,
    pub trades: Vec<TossTradeView>,
    pub price_limits: TossPriceLimitView,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossStockInfoView {
    pub symbol: String,
    pub name: String,
    pub english_name: String,
    pub isin_code: String,
    pub market: String,
    pub security_type: String,
    pub is_common_share: bool,
    pub status: String,
    pub currency: String,
    pub list_date: Option<String>,
    pub delist_date: Option<String>,
    pub shares_outstanding: String,
    pub leverage_factor: Option<String>,
}

impl From<TossStockInfo> for TossStockInfoView {
    fn from(value: TossStockInfo) -> Self {
        Self {
            symbol: value.symbol,
            name: value.name,
            english_name: value.english_name,
            isin_code: value.isin_code,
            market: value.market,
            security_type: value.security_type,
            is_common_share: value.is_common_share,
            status: value.status,
            currency: value.currency,
            list_date: value.list_date,
            delist_date: value.delist_date,
            shares_outstanding: value.shares_outstanding,
            leverage_factor: value.leverage_factor,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossStockWarningView {
    pub warning_type: String,
    pub label: String,
    pub exchange: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub blocking_for_buy: bool,
}

impl From<TossStockWarning> for TossStockWarningView {
    fn from(value: TossStockWarning) -> Self {
        let blocking_for_buy = value.is_blocking_for_buy();
        let label = toss_warning_label(&value.warning_type).to_string();
        Self {
            warning_type: value.warning_type,
            label,
            exchange: value.exchange,
            start_date: value.start_date,
            end_date: value.end_date,
            blocking_for_buy,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossStockSafetyView {
    pub broker_id: BrokerId,
    pub symbol: String,
    pub stock_info: Option<TossStockInfoView>,
    pub warnings: Vec<TossStockWarningView>,
    pub has_blocking_warning: bool,
    pub buy_blocked: bool,
    pub buy_block_reason: Option<String>,
}

fn toss_warning_label(warning_type: &str) -> &str {
    match warning_type {
        "LIQUIDATION_TRADING" => "정리매매",
        "OVERHEATED" => "단기과열",
        "INVESTMENT_WARNING" => "투자경고",
        "INVESTMENT_RISK" => "투자위험",
        "VI_STATIC_AND_DYNAMIC" => "VI 정적+동적",
        "VI_STATIC" => "VI 정적",
        "VI_DYNAMIC" => "VI 동적",
        "STOCK_WARRANTS" => "신주인수권",
        _ => warning_type,
    }
}

pub(super) fn normalize_toss_symbol(symbol: String) -> CmdResult<String> {
    let symbol = symbol.trim().to_uppercase();
    if symbol.is_empty() || symbol.len() > 32 {
        return Err(CmdError {
            code: "INVALID_SYMBOL".into(),
            message: format!("Toss symbol은 1~32자여야 합니다: {symbol}"),
        });
    }
    if !symbol
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(CmdError {
            code: "INVALID_SYMBOL".into(),
            message: format!("Toss symbol은 영문/숫자/./- 문자만 허용합니다: {symbol}"),
        });
    }
    Ok(symbol)
}

pub async fn get_toss_stock_safety_for_profile(
    symbol: String,
    profile: AccountProfile,
) -> CmdResult<TossStockSafetyView> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 종목 유의사항은 Toss 활성 프로파일에서만 조회할 수 있습니다.".into(),
        });
    }

    let symbol = normalize_toss_symbol(symbol)?;
    let broker_symbol = BrokerSymbol(symbol.clone());
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );

    let (stocks, warnings) = tokio::try_join!(
        adapter.list_stocks(std::slice::from_ref(&broker_symbol)),
        adapter.list_warnings(&broker_symbol),
    )
    .map_err(|e| CmdError {
        code: "TOSS_STOCK_SAFETY_ERROR".into(),
        message: e.to_string(),
    })?;

    let stock_info = stocks
        .into_iter()
        .find(|item| item.symbol.eq_ignore_ascii_case(&symbol))
        .map(TossStockInfoView::from);
    let status_block_reason = stock_info.as_ref().and_then(|info| {
        (info.status != "ACTIVE").then(|| format!("상장 상태가 ACTIVE가 아닙니다: {}", info.status))
    });
    let warnings: Vec<TossStockWarningView> = warnings
        .into_iter()
        .map(TossStockWarningView::from)
        .collect();
    let blocking_labels = warnings
        .iter()
        .filter(|warning| warning.blocking_for_buy)
        .map(|warning| warning.label.as_str())
        .collect::<Vec<_>>();
    let has_blocking_warning = !blocking_labels.is_empty();
    let warning_block_reason =
        has_blocking_warning.then(|| format!("매수 유의사항: {}", blocking_labels.join(", ")));
    let buy_block_reason = match (status_block_reason, warning_block_reason) {
        (Some(status), Some(warnings)) => Some(format!("{status}; {warnings}")),
        (Some(status), None) => Some(status),
        (None, Some(warnings)) => Some(warnings),
        (None, None) => None,
    };
    let buy_blocked = buy_block_reason.is_some();

    Ok(TossStockSafetyView {
        broker_id: BrokerId::Toss,
        symbol,
        stock_info,
        warnings,
        has_blocking_warning,
        buy_blocked,
        buy_block_reason,
    })
}

#[tauri::command]
pub async fn get_toss_stock_safety(
    symbol: String,
    state: State<'_, AppState>,
) -> CmdResult<TossStockSafetyView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    get_toss_stock_safety_for_profile(symbol, profile).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketSessionView {
    pub start_time: String,
    pub end_time: String,
}

impl From<&TossMarketSession> for TossMarketSessionView {
    fn from(value: &TossMarketSession) -> Self {
        Self {
            start_time: value.start_time.clone(),
            end_time: value.end_time.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketDayView {
    pub date: String,
    pub day_session: Option<TossMarketSessionView>,
    pub pre_session: Option<TossMarketSessionView>,
    pub regular_session: Option<TossMarketSessionView>,
    pub after_session: Option<TossMarketSessionView>,
    pub is_day_open: bool,
    pub is_pre_open: bool,
    pub is_regular_open: bool,
    pub is_after_open: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketCalendarView {
    pub broker_id: BrokerId,
    pub kr: TossMarketDayView,
    pub us: TossMarketDayView,
    pub summary: String,
}

fn toss_market_day_calendar(session: Option<&TossMarketSession>) -> CmdResult<MarketDayCalendar> {
    match session {
        Some(session) => {
            let regular_session =
                MarketSessionWindow::parse(&session.start_time, &session.end_time).ok_or_else(
                    || CmdError {
                        code: "TOSS_MARKET_CALENDAR_MAPPING_ERROR".into(),
                        message: format!(
                            "Toss market-calendar 세션 시간을 해석할 수 없습니다: {} ~ {}",
                            session.start_time, session.end_time
                        ),
                    },
                )?;
            Ok(MarketDayCalendar {
                regular_session: Some(regular_session),
            })
        }
        None => Ok(MarketDayCalendar::closed()),
    }
}

fn toss_market_session_window(
    session: Option<&TossMarketSession>,
) -> CmdResult<Option<MarketSessionWindow>> {
    let Some(session) = session else {
        return Ok(None);
    };
    let window =
        MarketSessionWindow::parse(&session.start_time, &session.end_time).ok_or_else(|| {
            CmdError {
                code: "TOSS_MARKET_CALENDAR_MAPPING_ERROR".into(),
                message: format!(
                    "Toss market-calendar 세션 시간을 해석할 수 없습니다: {} ~ {}",
                    session.start_time, session.end_time
                ),
            }
        })?;
    Ok(Some(window))
}

fn toss_calendar_override(
    kr: &TossKrMarketCalendarResponse,
    us: &TossUsMarketCalendarResponse,
) -> CmdResult<MarketCalendarOverride> {
    let kr_regular = kr
        .today
        .integrated
        .as_ref()
        .and_then(|integrated| integrated.regular_market.as_ref());
    let us_regular = us.today.regular_market.as_ref();
    Ok(MarketCalendarOverride {
        kr: Some(toss_market_day_calendar(kr_regular)?),
        us: Some(toss_market_day_calendar(us_regular)?),
        us_sessions: Some(UsMarketSessionCalendar {
            day_session: toss_market_session_window(us.today.day_market.as_ref())?,
            pre_session: toss_market_session_window(us.today.pre_market.as_ref())?,
            regular_session: toss_market_session_window(us.today.regular_market.as_ref())?,
            after_session: toss_market_session_window(us.today.after_market.as_ref())?,
        }),
    })
}

fn toss_calendar_view(
    kr: TossKrMarketCalendarResponse,
    us: TossUsMarketCalendarResponse,
) -> CmdResult<TossMarketCalendarView> {
    let override_calendar = toss_calendar_override(&kr, &us)?;
    let kst = chrono::FixedOffset::east_opt(9 * 3600).expect("KST FixedOffset 생성 실패");
    let now = chrono::Utc::now().with_timezone(&kst);
    let kr_regular = kr
        .today
        .integrated
        .as_ref()
        .and_then(|integrated| integrated.regular_market.as_ref());
    let kr_pre = kr
        .today
        .integrated
        .as_ref()
        .and_then(|integrated| integrated.pre_market.as_ref());
    let kr_after = kr
        .today
        .integrated
        .as_ref()
        .and_then(|integrated| integrated.after_market.as_ref());
    let us_day = us.today.day_market.as_ref();
    let us_pre = us.today.pre_market.as_ref();
    let us_regular = us.today.regular_market.as_ref();
    let us_after = us.today.after_market.as_ref();
    let is_open = |session: Option<&TossMarketSession>| {
        session
            .and_then(|session| MarketSessionWindow::parse(&session.start_time, &session.end_time))
            .is_some_and(|session| session.contains(now))
    };

    Ok(TossMarketCalendarView {
        broker_id: BrokerId::Toss,
        kr: TossMarketDayView {
            date: kr.today.date,
            day_session: None,
            pre_session: kr_pre.map(TossMarketSessionView::from),
            regular_session: kr_regular.map(TossMarketSessionView::from),
            after_session: kr_after.map(TossMarketSessionView::from),
            is_day_open: false,
            is_pre_open: is_open(kr_pre),
            is_regular_open: override_calendar
                .kr
                .as_ref()
                .is_some_and(|day| day.is_open_at(now)),
            is_after_open: is_open(kr_after),
        },
        us: TossMarketDayView {
            date: us.today.date,
            day_session: us_day.map(TossMarketSessionView::from),
            pre_session: us_pre.map(TossMarketSessionView::from),
            regular_session: us_regular.map(TossMarketSessionView::from),
            after_session: us_after.map(TossMarketSessionView::from),
            is_day_open: is_open(us_day),
            is_pre_open: is_open(us_pre),
            is_regular_open: override_calendar
                .us
                .as_ref()
                .is_some_and(|day| day.is_open_at(now)),
            is_after_open: is_open(us_after),
        },
        summary: open_markets_summary_with_calendar(Some(&override_calendar)),
    })
}

pub async fn get_toss_market_calendar_for_profile(
    profile: AccountProfile,
) -> CmdResult<TossMarketCalendarView> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss market-calendar는 Toss 활성 프로파일에서만 조회할 수 있습니다.".into(),
        });
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let (kr, us) = tokio::try_join!(
        adapter.get_kr_market_calendar(None),
        adapter.get_us_market_calendar(None),
    )
    .map_err(|e| CmdError {
        code: "TOSS_MARKET_CALENDAR_ERROR".into(),
        message: e.to_string(),
    })?;
    toss_calendar_view(kr, us)
}

pub(crate) async fn get_active_toss_calendar_override(
    profiles: &Arc<RwLock<ProfilesConfig>>,
) -> Option<MarketCalendarOverride> {
    let profile = {
        let profiles = profiles.read().await;
        profiles.get_active().cloned()
    }?;
    if profile.broker_id != BrokerId::Toss {
        return None;
    }
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let (kr, us) = tokio::try_join!(
        adapter.get_kr_market_calendar(None),
        adapter.get_us_market_calendar(None),
    )
    .ok()?;
    match toss_calendar_override(&kr, &us) {
        Ok(calendar) => Some(calendar),
        Err(e) => {
            tracing::warn!(
                "Toss market-calendar 매핑 실패 — 기존 장 시간 fallback 사용: {}",
                e.message
            );
            None
        }
    }
}

#[tauri::command]
pub async fn get_toss_market_calendar(
    state: State<'_, AppState>,
) -> CmdResult<TossMarketCalendarView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    get_toss_market_calendar_for_profile(profile).await
}

pub async fn get_toss_market_snapshot_for_profile(
    symbol: String,
    profile: AccountProfile,
) -> Result<TossMarketSnapshotView, CmdError> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 시세 snapshot은 Toss 활성 프로파일에서만 조회할 수 있습니다.".into(),
        });
    }

    let symbol = normalize_toss_symbol(symbol)?;

    let broker_symbol = BrokerSymbol(symbol.clone());
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );

    let (prices, orderbook, trades, limits) = tokio::try_join!(
        adapter.list_prices(std::slice::from_ref(&broker_symbol)),
        adapter.get_orderbook(&broker_symbol),
        adapter.list_trades(&broker_symbol, Some(10)),
        adapter.get_price_limits(&broker_symbol),
    )
    .map_err(|e| CmdError {
        code: "TOSS_MARKET_DATA_ERROR".into(),
        message: e.to_string(),
    })?;

    let price = prices
        .into_iter()
        .find(|item| item.symbol.eq_ignore_ascii_case(&symbol))
        .ok_or_else(|| CmdError {
            code: "TOSS_PRICE_NOT_FOUND".into(),
            message: format!("Toss 현재가 응답에 요청 symbol이 없습니다: {symbol}"),
        })?;
    let quote = price.to_broker_price_quote().map_err(|e| CmdError {
        code: "TOSS_PRICE_MAPPING_ERROR".into(),
        message: e.to_string(),
    })?;

    Ok(TossMarketSnapshotView {
        broker_id: BrokerId::Toss,
        market: quote.market,
        symbol,
        timestamp: price.timestamp,
        price: quote.last.into(),
        orderbook: orderbook.into(),
        trades: trades.into_iter().map(TossTradeView::from).collect(),
        price_limits: limits.into(),
    })
}

#[tauri::command]
pub async fn get_toss_market_snapshot(
    symbol: String,
    state: State<'_, AppState>,
) -> CmdResult<TossMarketSnapshotView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    get_toss_market_snapshot_for_profile(symbol, profile).await
}

pub async fn get_toss_chart_data_for_profile(
    symbol: String,
    interval: String,
    count: Option<u16>,
    profile: AccountProfile,
) -> CmdResult<Vec<ChartCandle>> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 캔들 조회는 Toss 활성 프로파일에서만 사용할 수 있습니다.".into(),
        });
    }

    let symbol = normalize_toss_symbol(symbol)?;

    let interval = match interval.as_str() {
        "1m" | "M1" | "m" => "1m",
        "1d" | "D" | "d" => "1d",
        other => {
            return Err(CmdError {
                code: "INVALID_INTERVAL".into(),
                message: format!("Toss candles interval은 1m 또는 1d만 지원합니다: {other}"),
            });
        }
    };
    let count = count.unwrap_or(200);
    if !(1..=200).contains(&count) {
        return Err(CmdError {
            code: "INVALID_COUNT".into(),
            message: format!("Toss candles count는 1~200 범위여야 합니다: {count}"),
        });
    }

    let symbol = BrokerSymbol(symbol);
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let candles = adapter
        .get_candles(&symbol, interval, "", "")
        .await
        .map_err(|e| CmdError {
            code: "TOSS_CANDLES_ERROR".into(),
            message: e.to_string(),
        })?;

    let mut chart: Vec<ChartCandle> = candles
        .into_iter()
        .take(count as usize)
        .map(|candle| ChartCandle {
            date: normalize_toss_chart_time(&candle.date, interval),
            open: candle.open.amount,
            high: candle.high.amount,
            low: candle.low.amount,
            close: candle.close.amount,
            volume: candle.volume.0,
        })
        .collect();
    chart.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(chart)
}

fn normalize_toss_chart_time(value: &str, interval: &str) -> String {
    if interval == "1d" {
        return value
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(8)
            .collect();
    }
    value.to_string()
}

#[tauri::command]
pub async fn get_toss_chart_data(
    symbol: String,
    interval: String,
    count: Option<u16>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<ChartCandle>> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    get_toss_chart_data_for_profile(symbol, interval, count, profile).await
}
