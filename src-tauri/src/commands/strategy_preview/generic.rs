use super::*;
use crate::trading::strategy::Signal;

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
    /// 평가 시작보다 이전인 사전자료. 평가 기간과 별도로 정규화한다.
    #[serde(default)]
    pub history_candles: Vec<ChartCandle>,
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
    pub preparation: StrategyPreparationView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_filter_preparation: Option<crate::trading::strategy::EntryFilterPreparation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatility_expansion_preparation:
        Option<crate::trading::strategy::VolatilityExpansionPreparation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyPreparationView {
    pub required_history_bars: Option<usize>,
    pub provided_history_bars: usize,
    pub available_history_bars: Option<usize>,
    pub ready_at_start: Option<bool>,
    pub ready_at_end: Option<bool>,
    pub first_ready_time: Option<String>,
    pub unready_evaluation_bars: usize,
    pub evaluated_bars: Option<usize>,
    pub history_status: String,
    pub indicator_status: String,
    pub outcome: String,
}

pub(super) fn signal_to_preview_view(
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

pub(super) type PreviewRow = (ChartCandle, OhlcCandle, u64);

pub(super) fn normalize_minute_time(value: &str) -> Option<String> {
    use chrono::SecondsFormat;

    // Preserve the legacy canonical output byte for byte.
    crate::trading::simulation::minute_time::parse_minute_instant(value)
        .map(|instant| instant.to_rfc3339_opts(SecondsFormat::Nanos, true))
}

pub(super) fn normalize_rows(
    candles: Vec<ChartCandle>,
    is_overseas: bool,
    minute: bool,
) -> CmdResult<Vec<PreviewRow>> {
    normalize_rows_with_order(candles, is_overseas, minute, false)
}

pub(super) fn normalize_ordered_rows(
    candles: Vec<ChartCandle>,
    is_overseas: bool,
    minute: bool,
) -> CmdResult<Vec<PreviewRow>> {
    normalize_rows_with_order(candles, is_overseas, minute, true)
}

fn normalize_rows_with_order(
    candles: Vec<ChartCandle>,
    is_overseas: bool,
    minute: bool,
    strict_order: bool,
) -> CmdResult<Vec<PreviewRow>> {
    let mut rows = Vec::with_capacity(candles.len());
    for mut candle in candles {
        let time = candle
            .date
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>();
        let normalized_time = if minute {
            normalize_minute_time(&candle.date)
        } else if time.len() == 8 && chrono::NaiveDate::parse_from_str(&time, "%Y%m%d").is_ok() {
            Some(time)
        } else {
            None
        };
        let ohlc = chart_candle_to_ohlc(&candle, is_overseas).filter(|ohlc| {
            ohlc.low > 0
                && ohlc.low <= ohlc.open.min(ohlc.close)
                && ohlc.high >= ohlc.open.max(ohlc.close)
        });
        if normalized_time.is_none() || ohlc.is_none() {
            return Err(CmdError {
                code: "INVALID_CANDLE".into(),
                message: "유효하지 않은 OHLC/시각이 있습니다. 평가봉을 사전자료로 이동시키지 않도록 입력을 거부합니다.".into(),
            });
        }
        let volume = chart_volume_to_u64(&candle.volume);
        candle.date = normalized_time.expect("validated timestamp");
        rows.push((candle, ohlc.expect("validated OHLC"), volume));
    }
    if strict_order && rows.windows(2).any(|pair| pair[0].0.date >= pair[1].0.date) {
        return Err(CmdError { code: "NON_MONOTONIC_CANDLES".into(), message: "연속 replay의 사전자료와 각 청크는 중복 없이 엄격한 시간순이어야 합니다. 역전을 정렬로 숨기지 않습니다.".into() });
    }
    if !strict_order {
        rows.sort_by(|left, right| left.0.date.cmp(&right.0.date));
    }
    if rows.windows(2).any(|pair| pair[0].0.date == pair[1].0.date) {
        return Err(CmdError {
            code: "DUPLICATE_CANDLE".into(),
            message: "동일 시각의 봉이 중복되어 있습니다.".into(),
        });
    }
    Ok(rows)
}

#[cfg(test)]
#[path = "generic_tests.rs"]
mod tests;

pub fn preview_strategy_from_candles(
    mut input: StrategyPreviewInput,
) -> CmdResult<StrategyPreviewView> {
    let symbol = normalize_preview_symbol(input.symbol.clone())?;
    // LTH on_tick uses the live minute/session clock. Never run it for history,
    // even as an unsupported diagnostic; the timed preview has its own contract.
    if input.strategy_id.starts_with("leveraged_trend_hold") {
        return Err(CmdError {
            code: "UNSUPPORTED_GENERIC_REPLAY".into(),
            message: "레버리지 추세 보유는 일반 replay를 지원하지 않습니다. 입력 시각을 사용하는 전용 preview_leveraged_trend_hold 경로를 사용하세요.".into(),
        });
    }
    if input.candles.is_empty() {
        return Err(CmdError {
            code: "NO_CANDLES".into(),
            message: format!("{symbol} 미리보기 차트 데이터가 비어 있습니다."),
        });
    }
    if input
        .candles
        .len()
        .saturating_add(input.history_candles.len())
        > 500
    {
        return Err(CmdError {
            code: "TOO_MANY_CANDLES".into(),
            message: format!(
                "미리보기 입력은 최대 500봉까지 지원합니다: {}봉",
                input
                    .candles
                    .len()
                    .saturating_add(input.history_candles.len())
            ),
        });
    }

    let interval = input.interval.as_deref().unwrap_or("1d").to_string();
    let daily = matches!(interval.as_str(), "D" | "1d");
    if !daily
        && (input.strategy_id.starts_with("strong_close")
            || input.strategy_id.starts_with("volatility_expansion"))
    {
        return Err(CmdError {
            code: "UNSUPPORTED_REPLAY_INTERVAL".into(),
            message: "강한 종가·변동성 확장 replay는 완료 일봉(D/1d)만 지원합니다.".into(),
        });
    }
    let minute = matches!(interval.as_str(), "1m" | "M1");
    let mut replay_rows = normalize_rows(
        std::mem::take(&mut input.candles),
        input.is_overseas,
        minute,
    )?;
    let legacy_warmup = input.warmup_count.unwrap_or(0);
    if legacy_warmup >= replay_rows.len() {
        return Err(CmdError {
            code: "NO_EVALUATION_CANDLES".into(),
            message: "사전자료를 제외한 평가봉이 한 개 이상 필요합니다.".into(),
        });
    }
    let mut history_rows = normalize_rows(
        std::mem::take(&mut input.history_candles),
        input.is_overseas,
        minute,
    )?;
    if legacy_warmup > 0 && !history_rows.is_empty() {
        return Err(CmdError {
            code: "AMBIGUOUS_HISTORY".into(),
            message: "historyCandles와 warmupCount 준비 구간을 동시에 지정할 수 없습니다.".into(),
        });
    }
    if legacy_warmup > 0 {
        history_rows.extend(replay_rows.drain(..legacy_warmup));
    }
    if history_rows
        .last()
        .is_some_and(|last| last.0.date >= replay_rows[0].0.date)
    {
        return Err(CmdError {
            code: "HISTORY_OVERLAP".into(),
            message: "사전자료는 평가 시작보다 엄격히 이전이어야 합니다.".into(),
        });
    }
    super::driver::run_prepared_preview(
        input,
        symbol,
        history_rows,
        vec![replay_rows],
        "boundedWindow",
    )
    .map(|(preview, _)| preview)
}
