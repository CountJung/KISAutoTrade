use super::*;
use crate::trading::strategy::{build_strategy, LeveragedTrendHoldTimedCandle, Signal};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeveragedTrendHoldPreviewInput {
    pub symbol: String,
    pub params: serde_json::Value,
    pub count: Option<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeveragedTrendHoldPreviewSignalView {
    pub time: String,
    pub side: String,
    pub price: f64,
    pub quantity: u64,
    pub reason: String,
    pub ema_short: Option<f64>,
    pub ema_long: Option<f64>,
    pub rsi: Option<f64>,
    pub adx: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeveragedTrendHoldPreviewView {
    pub symbol: String,
    pub candles: Vec<ChartCandle>,
    pub signals: Vec<LeveragedTrendHoldPreviewSignalView>,
    pub generated_at: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyPreviewInput {
    pub strategy_id: String,
    pub strategy_name: String,
    pub symbol: String,
    pub is_overseas: bool,
    pub order_quantity: u64,
    pub params: serde_json::Value,
    pub candles: Vec<ChartCandle>,
    pub warmup_count: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyPreviewSignalView {
    pub time: String,
    pub side: String,
    pub price: f64,
    pub quantity: u64,
    pub reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyPreviewView {
    pub strategy_id: String,
    pub symbol: String,
    pub candles: Vec<ChartCandle>,
    pub signals: Vec<StrategyPreviewSignalView>,
    pub generated_at: String,
    pub message: String,
}

fn normalize_preview_symbol(symbol: String) -> CmdResult<String> {
    let symbol = symbol.trim().to_uppercase();
    if symbol.is_empty() {
        return Err(CmdError {
            code: "INVALID_SYMBOL".into(),
            message: "미리보기 종목 티커를 입력하세요.".into(),
        });
    }
    if !symbol
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(CmdError {
            code: "INVALID_SYMBOL".into(),
            message: format!("지원하지 않는 Toss 티커 형식입니다: {symbol}"),
        });
    }
    Ok(symbol)
}

fn broker_money_amount(money: &BrokerMoney) -> Option<f64> {
    let amount = money.amount.trim().replace(',', "").parse::<f64>().ok()?;
    (amount > 0.0).then_some(amount)
}

fn broker_money_to_strategy_units(money: &BrokerMoney) -> Option<u64> {
    let amount = broker_money_amount(money)?;
    Some(match money.currency {
        BrokerCurrency::Krw => amount.round() as u64,
        BrokerCurrency::Usd => (amount * 100.0).round() as u64,
    })
}

fn normalize_preview_chart_time(value: &str, interval: &str) -> String {
    if interval == "1d" {
        return value
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(8)
            .collect();
    }
    value.to_string()
}

fn broker_candles_to_ohlc(candles: &[BrokerCandle]) -> Vec<OhlcCandle> {
    let mut candles = candles.to_vec();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    candles
        .iter()
        .filter_map(|c| {
            Some(OhlcCandle {
                open: broker_money_to_strategy_units(&c.open)?,
                high: broker_money_to_strategy_units(&c.high)?,
                low: broker_money_to_strategy_units(&c.low)?,
                close: broker_money_to_strategy_units(&c.close)?,
            })
        })
        .filter(|c| c.open > 0 && c.high > 0 && c.low > 0 && c.close > 0)
        .collect()
}

fn broker_candles_to_timed_ohlc(candles: &[BrokerCandle]) -> Vec<LeveragedTrendHoldTimedCandle> {
    let mut candles = candles.to_vec();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    candles
        .iter()
        .filter_map(|c| {
            Some(LeveragedTrendHoldTimedCandle {
                time: normalize_preview_chart_time(&c.date, "1m"),
                candle: OhlcCandle {
                    open: broker_money_to_strategy_units(&c.open)?,
                    high: broker_money_to_strategy_units(&c.high)?,
                    low: broker_money_to_strategy_units(&c.low)?,
                    close: broker_money_to_strategy_units(&c.close)?,
                },
            })
        })
        .filter(|c| {
            c.candle.open > 0 && c.candle.high > 0 && c.candle.low > 0 && c.candle.close > 0
        })
        .collect()
}

fn broker_candles_to_chart(candles: &[BrokerCandle], count: u16) -> Vec<ChartCandle> {
    let mut candles = candles.to_vec();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    let start = candles.len().saturating_sub(count as usize);
    candles
        .iter()
        .skip(start)
        .filter_map(|candle| {
            Some(ChartCandle {
                date: normalize_preview_chart_time(&candle.date, "1m"),
                open: broker_money_amount(&candle.open)?.to_string(),
                high: broker_money_amount(&candle.high)?.to_string(),
                low: broker_money_amount(&candle.low)?.to_string(),
                close: broker_money_amount(&candle.close)?.to_string(),
                volume: candle.volume.0.clone(),
            })
        })
        .collect()
}

fn strategy_units_to_price(units: u64, currency: BrokerCurrency) -> f64 {
    match currency {
        BrokerCurrency::Krw => units as f64,
        BrokerCurrency::Usd => units as f64 / 100.0,
    }
}

fn chart_amount_to_units(value: &str, is_overseas: bool) -> Option<u64> {
    let amount = value.trim().replace(',', "").parse::<f64>().ok()?;
    if !amount.is_finite() || amount <= 0.0 {
        return None;
    }
    Some(if is_overseas {
        (amount * 100.0).round() as u64
    } else {
        amount.round() as u64
    })
}

fn chart_volume_to_u64(value: &str) -> u64 {
    value
        .trim()
        .replace(',', "")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| v.round() as u64)
        .unwrap_or(0)
}

fn chart_candle_to_ohlc(candle: &ChartCandle, is_overseas: bool) -> Option<OhlcCandle> {
    Some(OhlcCandle {
        open: chart_amount_to_units(&candle.open, is_overseas)?,
        high: chart_amount_to_units(&candle.high, is_overseas)?,
        low: chart_amount_to_units(&candle.low, is_overseas)?,
        close: chart_amount_to_units(&candle.close, is_overseas)?,
    })
}

fn signal_to_preview_view(
    signal: Signal,
    time: String,
    price_units: u64,
    is_overseas: bool,
) -> Option<StrategyPreviewSignalView> {
    let price = if is_overseas {
        price_units as f64 / 100.0
    } else {
        price_units as f64
    };

    match signal {
        Signal::Buy {
            quantity, reason, ..
        } => Some(StrategyPreviewSignalView {
            time,
            side: "buy".to_string(),
            price,
            quantity,
            reason,
        }),
        Signal::Sell {
            quantity, reason, ..
        } => Some(StrategyPreviewSignalView {
            time,
            side: "sell".to_string(),
            price,
            quantity,
            reason,
        }),
        Signal::Hold => None,
    }
}

fn uses_startup_history(strategy_id: &str) -> bool {
    strategy_id.starts_with("fifty_two_week_high")
        || strategy_id.starts_with("strong_close")
        || strategy_id.starts_with("volatility_expansion")
}

fn default_warmup_count(strategy_id: &str, candle_len: usize) -> usize {
    if !uses_startup_history(strategy_id) || candle_len < 3 {
        return 0;
    }
    (candle_len / 2).clamp(1, candle_len.saturating_sub(1)).min(120)
}

pub fn preview_strategy_from_candles(
    input: StrategyPreviewInput,
) -> CmdResult<StrategyPreviewView> {
    let symbol = normalize_preview_symbol(input.symbol)?;
    if input.candles.is_empty() {
        return Err(CmdError {
            code: "NO_CANDLES".into(),
            message: format!("{symbol} 미리보기 차트 데이터가 비어 있습니다."),
        });
    }

    let mut candles = input.candles;
    candles.sort_by(|a, b| a.date.cmp(&b.date));

    let replay_rows = candles
        .iter()
        .filter_map(|candle| {
            let ohlc = chart_candle_to_ohlc(candle, input.is_overseas)?;
            Some((candle.clone(), ohlc, chart_volume_to_u64(&candle.volume)))
        })
        .filter(|(_, ohlc, _)| ohlc.open > 0 && ohlc.high > 0 && ohlc.low > 0 && ohlc.close > 0)
        .collect::<Vec<_>>();

    if replay_rows.is_empty() {
        return Err(CmdError {
            code: "NO_VALID_CANDLES".into(),
            message: format!("{symbol} 미리보기용 유효 캔들이 없습니다."),
        });
    }

    let mut config = StrategyConfig::new(
        input.strategy_id.clone(),
        input.strategy_name.clone(),
        true,
        vec![symbol.clone()],
        input.order_quantity.max(1),
        input.params,
    );
    if config.id.starts_with("price_condition") {
        if let Some(symbols) = config.params.get("symbols").and_then(|v| v.as_array()) {
            config.target_symbols = symbols
                .iter()
                .filter_map(|item| item.get("symbol").and_then(|v| v.as_str()).map(str::to_string))
                .collect();
        }
    }

    let mut strategy = build_strategy(config);
    let warmup_count = input
        .warmup_count
        .unwrap_or_else(|| default_warmup_count(&input.strategy_id, replay_rows.len()))
        .min(replay_rows.len().saturating_sub(1));

    if warmup_count > 0 {
        let warmup = &replay_rows[..warmup_count];
        let closes = warmup.iter().map(|(_, ohlc, _)| ohlc.close).collect::<Vec<_>>();
        let high_close = warmup
            .iter()
            .map(|(_, ohlc, _)| (ohlc.high, ohlc.close))
            .collect::<Vec<_>>();
        let ohlc = warmup.iter().map(|(_, ohlc, _)| *ohlc).collect::<Vec<_>>();
        let ranges = ohlc
            .iter()
            .map(|candle| candle.high.saturating_sub(candle.low))
            .collect::<Vec<_>>();

        strategy.initialize_historical(&symbol, &closes);
        strategy.initialize_candles(&symbol, &high_close);
        strategy.initialize_ohlc(&symbol, &ohlc);
        strategy.initialize_intraday_prices(&symbol, &closes);
        strategy.initialize_intraday_ohlc(&symbol, &ohlc);
        strategy.initialize_range_data(&symbol, &ranges);
    }

    let mut signals = Vec::new();
    for (candle, ohlc, volume) in replay_rows.iter().skip(warmup_count) {
        let signal = strategy.on_tick(&symbol, ohlc.close, *volume);
        if let Some(view) =
            signal_to_preview_view(signal, candle.date.clone(), ohlc.close, input.is_overseas)
        {
            signals.push(view);
        }
    }

    let message = if signals.is_empty() {
        format!("현재 파라미터와 차트 데이터 기준으로 {symbol} 매수/청산 신호가 없습니다.")
    } else {
        format!(
            "현재 파라미터와 차트 데이터 기준으로 {symbol} 신호 {}개를 찾았습니다.",
            signals.len()
        )
    };

    Ok(StrategyPreviewView {
        strategy_id: input.strategy_id,
        symbol,
        candles,
        signals,
        generated_at: chrono::Local::now().to_rfc3339(),
        message,
    })
}

#[tauri::command]
pub async fn preview_strategy(input: StrategyPreviewInput) -> CmdResult<StrategyPreviewView> {
    preview_strategy_from_candles(input)
}

pub async fn preview_leveraged_trend_hold_for_profile(
    input: LeveragedTrendHoldPreviewInput,
    profile: AccountProfile,
) -> CmdResult<LeveragedTrendHoldPreviewView> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "레버리지 전략 미리보기는 Toss 1분봉을 사용하는 Toss 활성 프로파일에서 사용할 수 있습니다.".into(),
        });
    }

    let symbol = normalize_preview_symbol(input.symbol)?;
    let count = input.count.unwrap_or(200).clamp(20, 200);
    let params: LeveragedTrendHoldParams =
        serde_json::from_value(input.params).map_err(|e| CmdError {
            code: "INVALID_PARAMS".into(),
            message: format!("레버리지 전략 파라미터를 해석할 수 없습니다: {e}"),
        })?;

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let broker_symbol = BrokerSymbol(symbol.clone());

    let daily_candles = adapter
        .get_candles(&broker_symbol, "D", "", "")
        .await
        .map_err(|e| CmdError {
            code: "TOSS_CANDLES_ERROR".into(),
            message: format!("Toss 일봉 조회 실패: {e}"),
        })?;
    let intraday_candles = adapter
        .get_candles(&broker_symbol, "1m", "", "")
        .await
        .map_err(|e| CmdError {
            code: "TOSS_CANDLES_ERROR".into(),
            message: format!("Toss 1분봉 조회 실패: {e}"),
        })?;

    if intraday_candles.is_empty() {
        return Err(CmdError {
            code: "NO_CANDLES".into(),
            message: format!("{symbol} Toss 1분봉 데이터가 비어 있습니다."),
        });
    }

    let currency = intraday_candles
        .iter()
        .find_map(|candle| broker_money_amount(&candle.close).map(|_| candle.close.currency))
        .unwrap_or(BrokerCurrency::Usd);
    let chart_candles = broker_candles_to_chart(&intraday_candles, count);
    let ohlc = broker_candles_to_ohlc(&daily_candles);
    let timed = broker_candles_to_timed_ohlc(&intraday_candles);
    let signal_views = LeveragedTrendHoldStrategy::preview_signals(&symbol, params, &ohlc, &timed)
        .into_iter()
        .map(|signal| LeveragedTrendHoldPreviewSignalView {
            time: signal.time,
            side: signal.side,
            price: strategy_units_to_price(signal.price_units, currency),
            quantity: signal.quantity,
            reason: signal.reason,
            ema_short: signal
                .ema_short
                .map(|v| strategy_units_to_price(v.round() as u64, currency)),
            ema_long: signal
                .ema_long
                .map(|v| strategy_units_to_price(v.round() as u64, currency)),
            rsi: signal.rsi,
            adx: signal.adx,
        })
        .collect::<Vec<_>>();

    let message = if signal_views.is_empty() {
        "현재 파라미터와 Toss 1분봉 기준으로 매수/청산 신호가 없습니다.".to_string()
    } else {
        format!(
            "현재 파라미터와 Toss 1분봉 기준으로 {}개 신호를 찾았습니다.",
            signal_views.len()
        )
    };

    Ok(LeveragedTrendHoldPreviewView {
        symbol,
        candles: chart_candles,
        signals: signal_views,
        generated_at: chrono::Local::now().to_rfc3339(),
        message,
    })
}

#[tauri::command]
pub async fn preview_leveraged_trend_hold(
    input: LeveragedTrendHoldPreviewInput,
    state: State<'_, AppState>,
) -> CmdResult<LeveragedTrendHoldPreviewView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    preview_leveraged_trend_hold_for_profile(input, profile).await
}
