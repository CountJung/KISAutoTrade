use super::*;

mod continuous;
mod driver;
mod generic;
use crate::trading::simulation::{
    replay_input_hash, BacktestReportView, ReplayAssessmentView, ReplayMetadataView, ReplaySession,
    SimulationAssumptions, SimulationEvent, REPLAY_ENGINE_VERSION,
};
use crate::trading::strategy::{
    LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldPreviewStep, LeveragedTrendHoldTimedCandle,
    Signal,
};
pub use continuous::{
    preview_strategy_continuous_from_candles, ChunkContinuityView, ContinuousStrategyPreviewChunk,
    ContinuousStrategyPreviewInput, ContinuousStrategyPreviewView, ReplayContinuityView,
};
pub use generic::{
    preview_strategy_from_candles, StrategyPreparationView, StrategyPreviewInput,
    StrategyPreviewSignalView, StrategyPreviewView,
};

const LTH_PREVIEW_STRATEGY_VERSION: &str = "leveraged-trend-hold-v4";

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

/// Align user scenario labels with the actual normalized evaluation bars. Never
/// shift an unmatched dividend/FX event to the nearest observed bar.
fn align_accounting_scenarios(
    assumptions: &mut SimulationAssumptions,
    labels: &[String],
    interval: &str,
    is_overseas: bool,
) -> CmdResult<()> {
    if assumptions.fx_scenario.len() > 64
        || assumptions.dividend_scenario.len() > 64
        || assumptions
            .fx_scenario
            .iter()
            .any(|point| point.bar_time.len() > 64)
        || assumptions
            .dividend_scenario
            .iter()
            .any(|point| point.bar_time.len() > 64)
    {
        return Err(CmdError {
            code: "INVALID_ACCOUNTING_SCENARIO".into(),
            message: "환율·배당 시나리오는 각각 최대 64개, 시각은 64바이트 이하여야 합니다.".into(),
        });
    }
    let canonical = |label: &str| {
        if matches!(interval, "1m" | "M1") {
            generic::normalize_minute_time(label)
        } else {
            let compact = label.replace('-', "");
            (compact.len() == 8 && chrono::NaiveDate::parse_from_str(&compact, "%Y%m%d").is_ok())
                .then_some(compact)
        }
    };
    let mut aliases = std::collections::HashMap::<String, Vec<String>>::new();
    for label in labels {
        if let Some(key) = canonical(label) {
            aliases.entry(key).or_default().push(label.clone());
        }
    }
    for label in assumptions
        .fx_scenario
        .iter_mut()
        .map(|point| &mut point.bar_time)
        .chain(
            assumptions
                .dividend_scenario
                .iter_mut()
                .map(|point| &mut point.bar_time),
        )
    {
        let matches = canonical(label).and_then(|key| aliases.get(&key));
        match matches {
            Some(matches) if matches.len() == 1 => *label = matches[0].clone(),
            _ => return Err(CmdError { code: "INVALID_ACCOUNTING_SCENARIO".into(),
                message: "시나리오 시각은 평가봉 하나와 정확히 일치해야 합니다. 사전자료·누락봉·중복 시각으로 옮기지 않습니다.".into() }),
        }
    }
    assumptions
        .validate_scenarios(is_overseas, labels)
        .map_err(|message| CmdError {
            code: "INVALID_ACCOUNTING_SCENARIO".into(),
            message,
        })
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
    _daily_open_minute: i64,
    _daily_close_minute: i64,
    _daily_close_day_offset: i64,
) -> Vec<LeveragedTrendHoldTimedCandle> {
    let mut candles = candles.to_vec();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    let start = candles.len().saturating_sub(count as usize);
    let mut timed = Vec::with_capacity(candles.len() - start);
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
            // Daily prices are a chart diagnostic, never simulated minute observations.
            timed.push(LeveragedTrendHoldTimedCandle {
                time: normalize_preview_chart_time(&source.date, "1d"),
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

#[tauri::command]
pub async fn preview_strategy_continuous(
    input: ContinuousStrategyPreviewInput,
) -> CmdResult<ContinuousStrategyPreviewView> {
    preview_strategy_continuous_from_candles(input)
}

#[tauri::command]
pub async fn preview_strategy(input: StrategyPreviewInput) -> CmdResult<StrategyPreviewView> {
    preview_strategy_from_candles(input)
}

fn lth_preview_signals_for_interval<F>(
    interval: &str,
    symbol: &str,
    params: crate::trading::strategy::LeveragedTrendHoldParams,
    daily_context: &[OhlcCandle],
    observations: &[LeveragedTrendHoldTimedCandle],
    execute: F,
) -> Vec<LeveragedTrendHoldPreviewSignal>
where
    F: FnMut(LeveragedTrendHoldPreviewStep<'_>) -> (u64, u64),
{
    if interval != "1m" {
        return Vec::new();
    }
    crate::trading::strategy::LeveragedTrendHoldStrategy::preview_signals_with_driver(
        symbol,
        params,
        daily_context,
        observations,
        execute,
    )
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
    if interval != "1m"
        && input
            .params
            .pointer("/bollinger/enabled")
            .and_then(|v| v.as_bool())
            == Some(true)
    {
        return Err(CmdError { code: "BOLLINGER_INTERVAL".into(), message: "볼린저 보강 옵션은 1분봉 미리보기로 점검하세요. 일봉 warmup과 분봉을 혼합하지 않습니다.".into() });
    }
    let count = input.count.unwrap_or(200).clamp(20, 200);
    let params_fingerprint = serde_json::to_string(&input.params).unwrap_or_default();
    let mut assumptions = input.assumptions.clone();
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
            open_units: candle.candle.open,
            close_units: candle.candle.close,
            high_units: candle.candle.high,
            low_units: candle.candle.low,
            signal: None,
        })
        .collect::<Vec<_>>();
    if interval == "1d"
        && (!assumptions.fx_scenario.is_empty() || !assumptions.dividend_scenario.is_empty())
    {
        return Err(CmdError { code: "SCENARIO_NOT_EVALUABLE".into(),
            message: "LTH 일봉은 자료 진단입니다. 현금흐름 시나리오 성과는 분봉 표본에서만 계산할 수 있습니다.".into() });
    }
    align_accounting_scenarios(
        &mut assumptions,
        &base_execution_events
            .iter()
            .map(|event| event.time.clone())
            .collect::<Vec<_>>(),
        interval,
        entry_is_overseas,
    )?;
    let assumptions_fingerprint = serde_json::to_string(&assumptions).unwrap_or_default();
    let benchmark_quantity = params
        .entries
        .iter()
        .find(|entry| entry.leveraged_symbol.eq_ignore_ascii_case(&symbol))
        .map(|entry| entry.quantity.max(1))
        .unwrap_or(1);
    let mut session = ReplaySession::new(
        "leveraged_trend_hold",
        &symbol,
        entry_is_overseas,
        assumptions,
        &[],
        if interval == "1m" {
            base_execution_events.len()
        } else {
            0
        },
    );
    let mut observation_index = 0;
    let mut close_processed = false;
    let preview_signals =
        lth_preview_signals_for_interval(interval, &symbol, params, &ohlc, &timed, |step| {
            let Some(event) = base_execution_events.get(observation_index) else {
                return session.position_snapshot();
            };
            match step {
                LeveragedTrendHoldPreviewStep::Open(_) => {
                    session.begin_bar(event);
                    close_processed = false;
                }
                LeveragedTrendHoldPreviewStep::Signal(preview_signal, _) => {
                    let mut event = event.clone();
                    event.signal = Some(if preview_signal.side == "buy" {
                        Signal::Buy {
                            symbol: symbol.clone(),
                            quantity: preview_signal.quantity,
                            reason: preview_signal.reason.clone(),
                        }
                    } else {
                        Signal::Sell {
                            symbol: symbol.clone(),
                            quantity: preview_signal.quantity,
                            reason: preview_signal.reason.clone(),
                        }
                    });
                    session.process_close(&event);
                    close_processed = true;
                }
                LeveragedTrendHoldPreviewStep::Close(_) => {
                    if !close_processed {
                        session.process_close(event);
                    }
                    observation_index += 1;
                }
            }
            session.position_snapshot()
        });
    let simulation_events = base_execution_events;
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
    let message = if interval == "1d" {
        format!("{symbol} 일봉 {}개는 가격 진단만 제공합니다. 실제 장중 관측이 없어 진입·청산과 투자 성과를 평가할 수 없습니다.", chart_candles.len())
    } else if signal_views.is_empty() {
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
        LTH_PREVIEW_STRATEGY_VERSION,
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
        strategy_version: LTH_PREVIEW_STRATEGY_VERSION.into(),
        source_interval: interval.into(),
        replay_cadence: if interval == "1m" {
            "minuteClose"
        } else {
            "dailyDiagnostic"
        }
        .into(),
        live_cadence_seconds: 10,
        warmup_count: 0,
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
        // Provider time/DST/session coverage has not been independently validated.
        look_ahead_safe: false,
        input_hash,
        assessment: Some(ReplayAssessmentView::lth(
            interval,
            ohlc.len(),
            if interval == "1m" { timed.len() } else { 0 },
        )),
    };
    let mut backtest = session.finish();
    if interval == "1m" {
        crate::trading::simulation::attach_comparison(
            &mut backtest,
            "leveraged_trend_hold",
            &symbol,
            entry_is_overseas,
            benchmark_quantity,
            &[],
            &simulation_events,
        );
    }

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

#[cfg(test)]
#[path = "strategy_preview/preparation_tests.rs"]
mod preparation_tests;

#[cfg(test)]
#[path = "strategy_preview/daily_event_tests.rs"]
mod daily_event_tests;

#[cfg(test)]
#[path = "strategy_preview/entry_filter_tests.rs"]
mod entry_filter_tests;

#[cfg(test)]
#[path = "strategy_preview/momentum_entry_tests.rs"]
mod momentum_entry_tests;

#[cfg(test)]
#[path = "strategy_preview/deviation_entry_tests.rs"]
mod deviation_entry_tests;

#[cfg(test)]
#[path = "strategy_preview/mean_reversion_entry_tests.rs"]
mod mean_reversion_entry_tests;
