//! Bounded, stateless request: one warmup and one live replay state across chunks.
use super::generic::{normalize_ordered_rows, PreviewRow};
use super::*;

const MAX_CHUNKS: usize = 16;
const MAX_CHUNK_BARS: usize = 500;
const MAX_EVALUATION_BARS: usize = 4096;
const MAX_HISTORY_BARS: usize = 512;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContinuousStrategyPreviewChunk {
    pub candles: Vec<ChartCandle>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContinuousStrategyPreviewInput {
    pub strategy_id: String,
    pub strategy_name: String,
    pub symbol: String,
    pub is_overseas: bool,
    pub order_quantity: u64,
    pub params: serde_json::Value,
    pub chunks: Vec<ContinuousStrategyPreviewChunk>,
    #[serde(default)]
    pub history_candles: Vec<ChartCandle>,
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
pub struct ChunkContinuityView {
    pub index: usize,
    pub start: String,
    pub end: String,
    pub bars: usize,
    pub start_quantity: u64,
    pub end_quantity: u64,
    pub pending_at_end: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayContinuityView {
    pub mode: &'static str,
    pub chunk_count: usize,
    pub evaluation_bars: usize,
    pub history_bars: usize,
    pub chunks: Vec<ChunkContinuityView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousStrategyPreviewView {
    #[serde(flatten)]
    pub preview: StrategyPreviewView,
    pub continuity: ReplayContinuityView,
}

fn invalid(code: &str, message: &str) -> CmdError {
    CmdError {
        code: code.into(),
        message: message.into(),
    }
}

pub fn preview_strategy_continuous_from_candles(
    input: ContinuousStrategyPreviewInput,
) -> CmdResult<ContinuousStrategyPreviewView> {
    let symbol = normalize_preview_symbol(input.symbol.clone())?;
    if input.strategy_id.starts_with("leveraged_trend_hold") {
        return Err(invalid("UNSUPPORTED_GENERIC_REPLAY", "레버리지 추세 보유는 입력 시각을 사용하는 전용 replay가 필요하며 일반 연속 replay로 실행하지 않습니다."));
    }
    if input.chunks.is_empty()
        || input.chunks.len() > MAX_CHUNKS
        || input.history_candles.len() > MAX_HISTORY_BARS
        || input
            .chunks
            .iter()
            .any(|chunk| chunk.candles.is_empty() || chunk.candles.len() > MAX_CHUNK_BARS)
    {
        return Err(invalid(
            "INVALID_CONTINUOUS_BOUNDS",
            "연속 replay는 1~16개 청크·청크당 1~500봉·사전자료 최대 512봉을 지원합니다.",
        ));
    }
    let evaluation_bars = input
        .chunks
        .iter()
        .map(|chunk| chunk.candles.len())
        .sum::<usize>();
    if evaluation_bars > MAX_EVALUATION_BARS {
        return Err(invalid(
            "TOO_MANY_CONTINUOUS_CANDLES",
            "연속 replay 평가봉은 최대 4096개입니다.",
        ));
    }
    let interval = input.interval.as_deref().unwrap_or("1d");
    if !matches!(interval, "D" | "1d" | "1m" | "M1" | "W" | "M") {
        return Err(invalid(
            "UNSUPPORTED_REPLAY_INTERVAL",
            "연속 replay는 일/주/월/분봉의 기존 명시 간격만 지원합니다.",
        ));
    }
    if !matches!(interval, "D" | "1d")
        && (input.strategy_id.starts_with("strong_close")
            || input.strategy_id.starts_with("volatility_expansion"))
    {
        return Err(invalid(
            "UNSUPPORTED_REPLAY_INTERVAL",
            "강한 종가·변동성 확장 replay는 완료 일봉(D/1d)만 지원합니다.",
        ));
    }
    let minute = matches!(interval, "1m" | "M1");
    let history_rows = normalize_ordered_rows(input.history_candles, input.is_overseas, minute)?;
    let history_bars = history_rows.len();
    let mut chunks: Vec<Vec<PreviewRow>> = Vec::with_capacity(input.chunks.len());
    let mut last_time = history_rows.last().map(|row| row.0.date.clone());
    for chunk in input.chunks {
        let rows = normalize_ordered_rows(chunk.candles, input.is_overseas, minute)?;
        let first = &rows[0].0.date;
        if last_time.as_ref().is_some_and(|last| last >= first) {
            return Err(invalid("CONTINUOUS_TIME_OVERLAP", "사전자료는 평가 시작보다 엄격히 이전이고, 청크 사이도 중복·역전 없이 엄격한 시간순이어야 합니다."));
        }
        last_time = rows.last().map(|row| row.0.date.clone());
        chunks.push(rows);
    }
    let meta = StrategyPreviewInput {
        strategy_id: input.strategy_id,
        strategy_name: input.strategy_name,
        symbol: input.symbol,
        is_overseas: input.is_overseas,
        order_quantity: input.order_quantity,
        params: input.params,
        candles: Vec::new(),
        history_candles: Vec::new(),
        warmup_count: None,
        interval: input.interval,
        data_source: input.data_source,
        strategy_version: input.strategy_version,
        broker_id: input.broker_id,
        broker_account_id: input.broker_account_id,
        assumptions: input.assumptions,
    };
    let (mut preview, ledger) =
        super::driver::run_prepared_preview(meta, symbol, history_rows, chunks, "continuous")?;
    if preview.backtest.overfit_warning.is_some() {
        preview.backtest.overfit_warning = Some("연속 장부의 앞/뒤 구간 수익이 약화되었습니다. 포지션·원가·리스크가 이어지므로 독립적인 학습/검증 표본으로 해석하지 마세요.".into());
    }
    Ok(ContinuousStrategyPreviewView {
        preview,
        continuity: ReplayContinuityView {
            mode: "continuous",
            chunk_count: ledger.len(),
            evaluation_bars,
            history_bars,
            chunks: ledger,
        },
    })
}

#[cfg(test)]
#[path = "continuous_tests.rs"]
mod tests;
