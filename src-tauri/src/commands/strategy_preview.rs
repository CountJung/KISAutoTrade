use super::*;
use crate::trading::simulation::{
    replay_input_hash, report_position_snapshot, run_backtest, BacktestReportView,
    ReplayMetadataView, SimulationAssumptions, SimulationEvent, REPLAY_ENGINE_VERSION,
};
use crate::trading::strategy::{
    build_strategy, initialize_strategy_warmup, LeveragedTrendHoldTimedCandle, Signal,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeveragedTrendHoldPreviewInput {
    pub symbol: String,
    pub params: serde_json::Value,
    pub interval: Option<String>,
    pub count: Option<u16>,
    #[serde(default)]
    pub assumptions: SimulationAssumptions,
    pub expected_profile_id: Option<String>,
    pub expected_broker_account_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeveragedTrendHoldPreviewSignalView {
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_time: Option<String>,
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
    pub interval: String,
    pub candle_count: usize,
    pub candles: Vec<ChartCandle>,
    pub signals: Vec<LeveragedTrendHoldPreviewSignalView>,
    pub generated_at: String,
    pub message: String,
    pub replay: ReplayMetadataView,
    pub backtest: BacktestReportView,
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
    pub interval: Option<String>,
    pub data_source: Option<String>,
    pub strategy_version: Option<String>,
    pub broker_id: Option<BrokerId>,
    pub broker_account_id: Option<String>,
    #[serde(default)]
    pub assumptions: SimulationAssumptions,
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
    pub replay: ReplayMetadataView,
    pub backtest: BacktestReportView,
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

fn daily_warmup_end_before(candles: &[BrokerCandle], replay_date: &str) -> usize {
    candles
        .iter()
        .position(|candle| {
            candle
                .date
                .chars()
                .filter(|ch| ch.is_ascii_digit())
                .take(8)
                .collect::<String>()
                .as_str()
                >= replay_date
        })
        .unwrap_or(candles.len())
}

fn broker_candles_to_timed_ohlc(
    candles: &[BrokerCandle],
    count: u16,
    interval: &str,
    daily_open_minute: i64,
    daily_close_minute: i64,
    daily_close_day_offset: i64,
) -> Vec<LeveragedTrendHoldTimedCandle> {
    let mut candles = candles.to_vec();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    let start = candles.len().saturating_sub(count as usize);
    let mut timed =
        Vec::with_capacity((candles.len() - start) * if interval == "1d" { 2 } else { 1 });
    for source in candles.iter().skip(start) {
        let Some(candle) = (|| {
            Some(OhlcCandle {
                open: broker_money_to_strategy_units(&source.open)?,
                high: broker_money_to_strategy_units(&source.high)?,
                low: broker_money_to_strategy_units(&source.low)?,
                close: broker_money_to_strategy_units(&source.close)?,
            })
        })() else {
            continue;
        };
        if candle.open == 0 || candle.high == 0 || candle.low == 0 || candle.close == 0 {
            continue;
        }
        if interval == "1d" {
            let Some(open_time) = daily_preview_time(&source.date, daily_open_minute, 0) else {
                continue;
            };
            let Some(close_time) =
                daily_preview_time(&source.date, daily_close_minute, daily_close_day_offset)
            else {
                continue;
            };
            timed.push(LeveragedTrendHoldTimedCandle {
                time: open_time,
                candle: OhlcCandle {
                    open: candle.open,
                    high: candle.open,
                    low: candle.open,
                    close: candle.open,
                },
            });
            timed.push(LeveragedTrendHoldTimedCandle {
                time: close_time,
                candle,
            });
        } else {
            timed.push(LeveragedTrendHoldTimedCandle {
                time: normalize_preview_chart_time(&source.date, "1m"),
                candle,
            });
        }
    }
    timed
}

fn daily_preview_time(value: &str, minute_of_day: i64, day_offset: i64) -> Option<String> {
    let date: String = value
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(8)
        .collect();
    if date.len() != 8 {
        return None;
    }
    let date = chrono::NaiveDate::parse_from_str(&date, "%Y%m%d").ok()?
        + chrono::Duration::days(day_offset);
    let minute = minute_of_day.rem_euclid(24 * 60);
    Some(format!(
        "{}{:02}{:02}00",
        date.format("%Y%m%d"),
        minute / 60,
        minute % 60
    ))
}

fn daily_chart_time(value: &str, close_day_offset: i64) -> String {
    let normalized = normalize_preview_chart_time(value, "1d");
    if close_day_offset == 0 {
        return normalized;
    }
    let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
    let minute = digits
        .get(8..12)
        .and_then(|hhmm| hhmm.parse::<i64>().ok())
        .map(|hhmm| (hhmm / 100) * 60 + hhmm % 100);
    if minute.is_some_and(|minute| minute < 12 * 60) {
        return chrono::NaiveDate::parse_from_str(&normalized, "%Y%m%d")
            .ok()
            .map(|date| {
                (date - chrono::Duration::days(1))
                    .format("%Y%m%d")
                    .to_string()
            })
            .unwrap_or(normalized);
    }
    normalized
}

fn minute_replay_trading_date(value: &str, is_overseas: bool, toss_us_session: &str) -> String {
    let minute = value
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .get(8..12)
        .and_then(|hhmm| hhmm.parse::<i64>().ok())
        .map(|hhmm| (hhmm / 100) * 60 + hhmm % 100);
    let crosses_midnight = is_overseas
        && minute.is_some_and(|minute| minute < 5 * 60)
        && matches!(
            crate::market_hours::UsTradingSessionPolicy::parse(Some(toss_us_session)),
            crate::market_hours::UsTradingSessionPolicy::Auto
                | crate::market_hours::UsTradingSessionPolicy::Regular
        );
    daily_chart_time(value, i64::from(crosses_midnight))
}

fn preview_session_bounds(
    is_overseas: bool,
    toss_us_session: &str,
    entry_window_start_min: i64,
) -> (i64, i64, i64) {
    let (open, close, close_day_offset) = if !is_overseas {
        (9 * 60, 15 * 60 + 30, 0)
    } else {
        match crate::market_hours::UsTradingSessionPolicy::parse(Some(toss_us_session)) {
            crate::market_hours::UsTradingSessionPolicy::Auto
            | crate::market_hours::UsTradingSessionPolicy::Day => (9 * 60, 16 * 60 + 50, 0),
            crate::market_hours::UsTradingSessionPolicy::Pre => (17 * 60, 22 * 60 + 30, 0),
            crate::market_hours::UsTradingSessionPolicy::Regular => (22 * 60 + 30, 5 * 60, 1),
            crate::market_hours::UsTradingSessionPolicy::After => (5 * 60, 7 * 60, 0),
        }
    };
    (
        open + entry_window_start_min.max(0),
        close,
        close_day_offset,
    )
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
    (candle_len / 2)
        .clamp(1, candle_len.saturating_sub(1))
        .min(120)
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
    if input.candles.len() > 500 {
        return Err(CmdError {
            code: "TOO_MANY_CANDLES".into(),
            message: format!(
                "미리보기 입력은 최대 500봉까지 지원합니다: {}봉",
                input.candles.len()
            ),
        });
    }

    let interval = input.interval.as_deref().unwrap_or("1d").to_string();
    let data_source = input
        .data_source
        .clone()
        .unwrap_or_else(|| "providedChartCandles".into());
    let strategy_version = input
        .strategy_version
        .clone()
        .unwrap_or_else(|| REPLAY_ENGINE_VERSION.into());
    let assumptions = input.assumptions.clone();
    let params_fingerprint = serde_json::to_string(&input.params).unwrap_or_default();
    let assumptions_fingerprint = serde_json::to_string(&assumptions).unwrap_or_default();
    let scope_fingerprint = format!(
        "{:?}|{}",
        input.broker_id,
        input.broker_account_id.as_deref().unwrap_or_default()
    );

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
                .filter_map(|item| {
                    item.get("symbol")
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                })
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
        let ohlc = warmup.iter().map(|(_, ohlc, _)| *ohlc).collect::<Vec<_>>();
        if matches!(interval.as_str(), "1m" | "M1") {
            initialize_strategy_warmup(strategy.as_mut(), &symbol, &[], &ohlc);
        } else {
            initialize_strategy_warmup(strategy.as_mut(), &symbol, &ohlc, &[]);
        }
    }

    let mut signals = Vec::new();
    let mut simulation_events = Vec::with_capacity(replay_rows.len().saturating_sub(warmup_count));
    for (candle, ohlc, volume) in replay_rows.iter().skip(warmup_count) {
        let signal = strategy.on_tick(&symbol, ohlc.close, *volume);
        simulation_events.push(SimulationEvent {
            time: candle.date.clone(),
            chart_time: candle.date.clone(),
            close_units: ohlc.close,
            high_units: ohlc.high,
            low_units: ohlc.low,
            signal: (signal != Signal::Hold).then(|| signal.clone()),
        });
        if signal != Signal::Hold {
            let partial_report = run_backtest(
                &input.strategy_id,
                &symbol,
                input.is_overseas,
                assumptions.clone(),
                &simulation_events,
            );
            let (held_quantity, average_price) =
                report_position_snapshot(&partial_report, input.is_overseas);
            strategy.sync_position(&symbol, held_quantity, average_price);
        }
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

    let first_time = simulation_events
        .first()
        .map(|event| event.chart_time.clone())
        .unwrap_or_default();
    let last_time = simulation_events
        .last()
        .map(|event| event.chart_time.clone())
        .unwrap_or_default();
    let event_fingerprint = replay_rows
        .iter()
        .map(|(candle, ohlc, volume)| {
            format!(
                "{}:{}:{}:{}:{}:{}",
                candle.date, ohlc.open, ohlc.high, ohlc.low, ohlc.close, volume
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let order_quantity_fingerprint = input.order_quantity.to_string();
    let warmup_count_fingerprint = warmup_count.to_string();
    let input_hash = replay_input_hash(&[
        REPLAY_ENGINE_VERSION,
        &input.strategy_id,
        &strategy_version,
        &symbol,
        &interval,
        &data_source,
        &params_fingerprint,
        &order_quantity_fingerprint,
        &warmup_count_fingerprint,
        &assumptions_fingerprint,
        &scope_fingerprint,
        &event_fingerprint,
    ]);
    let replay = ReplayMetadataView {
        engine_version: REPLAY_ENGINE_VERSION.into(),
        strategy_version,
        source_interval: interval.clone(),
        replay_cadence: match interval.as_str() {
            "1m" | "M1" => "minuteClose",
            "D" | "1d" => "dailyClose",
            "W" => "weeklyClose",
            "M" => "monthlyClose",
            _ => "candleClose",
        }
        .into(),
        live_cadence_seconds: 10,
        warmup_count,
        data_start: first_time,
        data_end: last_time,
        data_source,
        deterministic: true,
        look_ahead_safe: true,
        input_hash,
    };
    let backtest = run_backtest(
        &input.strategy_id,
        &symbol,
        input.is_overseas,
        assumptions,
        &simulation_events,
    );

    Ok(StrategyPreviewView {
        strategy_id: input.strategy_id,
        symbol,
        candles,
        signals,
        generated_at: chrono::Local::now().to_rfc3339(),
        message,
        replay,
        backtest,
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
    if input
        .expected_profile_id
        .as_deref()
        .is_some_and(|expected| expected != profile.id)
    {
        return Err(CmdError {
            code: "SCOPE_MISMATCH".into(),
            message:
                "미리보기 요청 후 활성 프로파일이 변경되었습니다. 현재 scope에서 다시 실행하세요."
                    .into(),
        });
    }
    if input
        .expected_broker_account_id
        .as_deref()
        .is_some_and(|expected| expected != profile.account_no)
    {
        return Err(CmdError {
            code: "SCOPE_MISMATCH".into(),
            message: "미리보기 요청의 Toss accountSeq가 현재 활성 계좌와 다릅니다.".into(),
        });
    }

    let symbol = normalize_preview_symbol(input.symbol)?;
    let interval = match input.interval.as_deref().unwrap_or("1m") {
        "1m" | "M1" | "m" => "1m",
        "1d" | "D" | "d" => "1d",
        other => {
            return Err(CmdError {
                code: "INVALID_INTERVAL".into(),
                message: format!("레버리지 미리보기 봉 단위는 1m 또는 1d만 지원합니다: {other}"),
            });
        }
    };
    let count = input.count.unwrap_or(200).clamp(20, 200);
    let params_fingerprint = serde_json::to_string(&input.params).unwrap_or_default();
    let assumptions = input.assumptions.clone();
    let assumptions_fingerprint = serde_json::to_string(&assumptions).unwrap_or_default();
    let params: LeveragedTrendHoldParams =
        serde_json::from_value(input.params).map_err(|e| CmdError {
            code: "INVALID_PARAMS".into(),
            message: format!("레버리지 전략 파라미터를 해석할 수 없습니다: {e}"),
        })?;

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(profile.account_no.clone()),
    );
    let broker_symbol = BrokerSymbol(symbol.clone());

    let mut daily_candles = adapter
        .get_candles(&broker_symbol, "D", "", "")
        .await
        .map_err(|e| CmdError {
            code: "TOSS_CANDLES_ERROR".into(),
            message: format!("Toss 일봉 조회 실패: {e}"),
        })?;
    daily_candles.sort_by(|a, b| a.date.cmp(&b.date));
    let simulation_candles = if interval == "1m" {
        adapter
            .get_candles(&broker_symbol, "1m", "", "")
            .await
            .map_err(|e| CmdError {
                code: "TOSS_CANDLES_ERROR".into(),
                message: format!("Toss 1분봉 조회 실패: {e}"),
            })?
    } else {
        daily_candles.clone()
    };

    if simulation_candles.is_empty() {
        return Err(CmdError {
            code: "NO_CANDLES".into(),
            message: format!("{symbol} Toss {interval} 데이터가 비어 있습니다."),
        });
    }

    let currency = simulation_candles
        .iter()
        .find_map(|candle| broker_money_amount(&candle.close).map(|_| candle.close.currency))
        .unwrap_or(BrokerCurrency::Usd);
    let chart_candles = broker_candles_to_chart(&simulation_candles, count);
    let replay_start = daily_candles.len().saturating_sub(count as usize);
    let entry_is_overseas = params
        .entries
        .iter()
        .find(|entry| entry.leveraged_symbol.eq_ignore_ascii_case(&symbol))
        .map(|entry| entry.is_overseas)
        .unwrap_or(true);
    let (daily_open_minute, daily_close_minute, daily_close_day_offset) = preview_session_bounds(
        entry_is_overseas,
        &params.toss_us_session,
        params.entry_window_start_min,
    );
    let timed = broker_candles_to_timed_ohlc(
        &simulation_candles,
        count,
        interval,
        daily_open_minute,
        daily_close_minute,
        daily_close_day_offset,
    );
    let first_replay_date = timed.first().map(|candle| {
        if interval == "1m" {
            minute_replay_trading_date(&candle.time, entry_is_overseas, &params.toss_us_session)
        } else {
            candle
                .time
                .chars()
                .filter(|ch| ch.is_ascii_digit())
                .take(8)
                .collect::<String>()
        }
    });
    let warmup_daily = if interval == "1d" {
        &daily_candles[..replay_start]
    } else {
        let cutoff = first_replay_date.as_deref().unwrap_or_default();
        let warmup_end = daily_warmup_end_before(&daily_candles, cutoff);
        &daily_candles[..warmup_end]
    };
    let ohlc = broker_candles_to_ohlc(warmup_daily);
    let base_execution_events = timed
        .iter()
        .map(|candle| SimulationEvent {
            time: candle.time.clone(),
            chart_time: if interval == "1d" {
                daily_chart_time(&candle.time, daily_close_day_offset)
            } else {
                candle.time.clone()
            },
            close_units: candle.candle.close,
            high_units: candle.candle.high,
            low_units: candle.candle.low,
            signal: None,
        })
        .collect::<Vec<_>>();
    let event_index_by_time = base_execution_events
        .iter()
        .enumerate()
        .map(|(index, event)| (event.time.clone(), index))
        .collect::<std::collections::HashMap<_, _>>();
    let mut execution_events = Vec::with_capacity(base_execution_events.len());
    let mut next_execution_index = 0usize;
    let preview_signals = LeveragedTrendHoldStrategy::preview_signals_with_execution(
        &symbol,
        params,
        &ohlc,
        &timed,
        |preview_signal, _| {
            let Some(&signal_index) = event_index_by_time.get(&preview_signal.time) else {
                return false;
            };
            while next_execution_index <= signal_index {
                execution_events.push(base_execution_events[next_execution_index].clone());
                next_execution_index += 1;
            }
            let signal = match preview_signal.side.as_str() {
                "buy" => Signal::Buy {
                    symbol: symbol.clone(),
                    quantity: preview_signal.quantity,
                    reason: preview_signal.reason.clone(),
                },
                "sell" => Signal::Sell {
                    symbol: symbol.clone(),
                    quantity: preview_signal.quantity,
                    reason: preview_signal.reason.clone(),
                },
                _ => Signal::Hold,
            };
            if let Some(event) = execution_events.last_mut() {
                event.signal = (signal != Signal::Hold).then_some(signal);
            }
            let report = run_backtest(
                "leveraged_trend_hold",
                &symbol,
                entry_is_overseas,
                assumptions.clone(),
                &execution_events,
            );
            report
                .trades
                .last()
                .is_some_and(|trade| trade.status == "filled")
        },
    );
    let mut signals_by_time = std::collections::HashMap::<String, Vec<Signal>>::new();
    for signal in &preview_signals {
        let replay_signal = match signal.side.as_str() {
            "buy" => Signal::Buy {
                symbol: symbol.clone(),
                quantity: signal.quantity,
                reason: signal.reason.clone(),
            },
            "sell" => Signal::Sell {
                symbol: symbol.clone(),
                quantity: signal.quantity,
                reason: signal.reason.clone(),
            },
            _ => Signal::Hold,
        };
        if replay_signal != Signal::Hold {
            signals_by_time
                .entry(signal.time.clone())
                .or_default()
                .push(replay_signal);
        }
    }
    let mut simulation_events = base_execution_events;
    for event in &mut simulation_events {
        event.signal = signals_by_time
            .remove(&event.time)
            .and_then(|mut signals| signals.pop());
    }
    let signal_views = preview_signals
        .into_iter()
        .map(|signal| LeveragedTrendHoldPreviewSignalView {
            chart_time: (interval == "1d")
                .then(|| daily_chart_time(&signal.time, daily_close_day_offset)),
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

    let interval_label = if interval == "1m" {
        "1분봉"
    } else {
        "일봉"
    };
    let message = if signal_views.is_empty() {
        format!(
            "현재 파라미터와 Toss {interval_label} 실제 {}봉 기준으로 매수/청산 신호가 없습니다.",
            chart_candles.len()
        )
    } else {
        format!(
            "현재 파라미터와 Toss {interval_label} 실제 {}봉 기준으로 {}개 신호를 찾았습니다.",
            chart_candles.len(),
            signal_views.len()
        )
    };

    let warmup_fingerprint = serde_json::to_string(warmup_daily).unwrap_or_default();
    let event_fingerprint = timed
        .iter()
        .map(|candle| {
            format!(
                "{}:{}:{}:{}:{}",
                candle.time,
                candle.candle.open,
                candle.candle.high,
                candle.candle.low,
                candle.candle.close
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let count_fingerprint = count.to_string();
    let scope_fingerprint = format!("{}|{}", profile.id, profile.account_no);
    let input_hash = replay_input_hash(&[
        REPLAY_ENGINE_VERSION,
        "leveraged_trend_hold",
        "leveraged-trend-hold-v2",
        &symbol,
        interval,
        &params_fingerprint,
        &assumptions_fingerprint,
        &scope_fingerprint,
        &count_fingerprint,
        &warmup_fingerprint,
        &event_fingerprint,
    ]);
    let replay = ReplayMetadataView {
        engine_version: REPLAY_ENGINE_VERSION.into(),
        strategy_version: "leveraged-trend-hold-v2".into(),
        source_interval: interval.into(),
        replay_cadence: if interval == "1m" {
            "minuteClose"
        } else {
            "dailyOpenAndClose"
        }
        .into(),
        live_cadence_seconds: 10,
        warmup_count: ohlc.len(),
        data_start: simulation_events
            .first()
            .map(|event| event.chart_time.clone())
            .unwrap_or_default(),
        data_end: simulation_events
            .last()
            .map(|event| event.chart_time.clone())
            .unwrap_or_default(),
        data_source: format!("toss:{interval}"),
        deterministic: true,
        look_ahead_safe: true,
        input_hash,
    };
    let backtest = run_backtest(
        "leveraged_trend_hold",
        &symbol,
        entry_is_overseas,
        assumptions,
        &simulation_events,
    );

    Ok(LeveragedTrendHoldPreviewView {
        symbol,
        interval: interval.to_string(),
        candle_count: chart_candles.len(),
        candles: chart_candles,
        signals: signal_views,
        generated_at: chrono::Local::now().to_rfc3339(),
        message,
        replay,
        backtest,
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

#[cfg(test)]
#[path = "strategy_preview/tests.rs"]
mod tests;
