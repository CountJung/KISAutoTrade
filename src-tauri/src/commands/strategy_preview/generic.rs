use super::*;
use crate::trading::strategy::{build_strategy, initialize_strategy_warmup, Signal};

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

type PreviewRow = (ChartCandle, OhlcCandle, u64);

fn normalize_rows(
    candles: Vec<ChartCandle>,
    is_overseas: bool,
    minute: bool,
) -> CmdResult<Vec<PreviewRow>> {
    let mut rows = Vec::with_capacity(candles.len());
    for mut candle in candles {
        let mut time = candle
            .date
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>();
        if time.len() == 12 {
            time.push_str("00");
        }
        let valid_time = match (minute, time.len()) {
            (false, 8) => chrono::NaiveDate::parse_from_str(&time, "%Y%m%d").is_ok(),
            (true, 14) => chrono::NaiveDateTime::parse_from_str(&time, "%Y%m%d%H%M%S").is_ok(),
            _ => false,
        };
        let ohlc = chart_candle_to_ohlc(&candle, is_overseas).filter(|ohlc| {
            ohlc.low > 0
                && ohlc.low <= ohlc.open.min(ohlc.close)
                && ohlc.high >= ohlc.open.max(ohlc.close)
        });
        if !valid_time || ohlc.is_none() {
            return Err(CmdError {
                code: "INVALID_CANDLE".into(),
                message: "유효하지 않은 OHLC/시각이 있습니다. 평가봉을 사전자료로 이동시키지 않도록 입력을 거부합니다.".into(),
            });
        }
        let volume = chart_volume_to_u64(&candle.volume);
        candle.date = time;
        rows.push((candle, ohlc.expect("validated OHLC"), volume));
    }
    rows.sort_by(|left, right| left.0.date.cmp(&right.0.date));
    if rows.windows(2).any(|pair| pair[0].0.date == pair[1].0.date) {
        return Err(CmdError {
            code: "DUPLICATE_CANDLE".into(),
            message: "동일 시각의 봉이 중복되어 있습니다.".into(),
        });
    }
    Ok(rows)
}

pub fn preview_strategy_from_candles(
    input: StrategyPreviewInput,
) -> CmdResult<StrategyPreviewView> {
    let symbol = normalize_preview_symbol(input.symbol)?;
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

    let minute = matches!(interval.as_str(), "1m" | "M1");
    let mut replay_rows = normalize_rows(input.candles, input.is_overseas, minute)?;
    let legacy_warmup = input.warmup_count.unwrap_or(0);
    if legacy_warmup >= replay_rows.len() {
        return Err(CmdError {
            code: "NO_EVALUATION_CANDLES".into(),
            message: "사전자료를 제외한 평가봉이 한 개 이상 필요합니다.".into(),
        });
    }
    let mut history_rows = normalize_rows(input.history_candles, input.is_overseas, minute)?;
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
    let candles = replay_rows
        .iter()
        .map(|(candle, _, _)| candle.clone())
        .collect();

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
    let warmup_count = history_rows.len();

    if warmup_count > 0 {
        let ohlc = history_rows
            .iter()
            .map(|(_, ohlc, _)| *ohlc)
            .collect::<Vec<_>>();
        if matches!(interval.as_str(), "1m" | "M1") {
            initialize_strategy_warmup(strategy.as_mut(), &symbol, &[], &ohlc);
        } else {
            initialize_strategy_warmup(strategy.as_mut(), &symbol, &ohlc, &[]);
        }
    }

    let initial_readiness = strategy.history_readiness(&symbol);
    let mut first_ready_time = initial_readiness
        .filter(|state| state.ready)
        .map(|_| replay_rows[0].0.date.clone());
    let mut unready_evaluation_bars = 0;
    let mut evaluated_bars = 0;

    let mut signals = Vec::new();
    let mut simulation_events = Vec::with_capacity(replay_rows.len());
    for (candle, ohlc, volume) in &replay_rows {
        if daily {
            strategy.on_trading_day_start(&symbol, ohlc.open);
        }
        if strategy.can_evaluate_next_tick(&symbol) == Some(true) {
            evaluated_bars += 1;
        }
        let signal = if daily {
            strategy.on_daily_close_tick(&symbol, ohlc, *volume)
        } else {
            strategy.on_tick(&symbol, ohlc.close, *volume)
        };
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
        if daily {
            strategy.on_completed_candle(&symbol, ohlc);
        }
        if strategy
            .history_readiness(&symbol)
            .is_some_and(|state| state.ready)
        {
            if first_ready_time.is_none() {
                first_ready_time = Some(candle.date.clone());
            }
        } else if strategy.history_readiness(&symbol).is_some() {
            unready_evaluation_bars += 1;
        }
        if let Some(view) =
            signal_to_preview_view(signal, candle.date.clone(), ohlc.close, input.is_overseas)
        {
            signals.push(view);
        }
    }

    let first_time = simulation_events
        .first()
        .map(|event| event.chart_time.clone())
        .unwrap_or_default();
    let last_time = simulation_events
        .last()
        .map(|event| event.chart_time.clone())
        .unwrap_or_default();
    let event_fingerprint = history_rows
        .iter()
        .chain(replay_rows.iter())
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
            "D" | "1d" => "dailyCloseWithDayBoundary",
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
        assessment: None,
    };
    let backtest = run_backtest(
        &input.strategy_id,
        &symbol,
        input.is_overseas,
        assumptions,
        &simulation_events,
    );
    let final_readiness = strategy.history_readiness(&symbol);
    let history_status = match initial_readiness {
        None => "unsupported",
        Some(state) if state.required_bars == 0 => "notRequired",
        Some(state) if state.ready => "sufficient",
        Some(_) => "insufficient",
    };
    let indicator_status = match final_readiness {
        None => "unsupported",
        Some(state) if state.ready => "ready",
        Some(_) => "warmingUp",
    };
    let outcome = if initial_readiness.is_none() || evaluated_bars == 0 {
        "notEvaluable"
    } else if backtest.summary.filled_order_count > 0 {
        "traded"
    } else if !signals.is_empty() {
        "noTrades"
    } else {
        "conditionsNotMet"
    };
    let message = match outcome {
        "notEvaluable" => format!(
            "{symbol} 조건 평가가 가능한 관측 구간이 없거나 준비 상태가 미지원이므로 무거래 성과로 판정할 수 없습니다."
        ),
        "conditionsNotMet" => {
            format!("{symbol} 지표 준비 후 매수/청산 조건이 충족되지 않았습니다.")
        }
        "noTrades" => format!(
            "{symbol} 신호 {}개가 발생했으나 실행 조건으로 체결되지 않았습니다.",
            signals.len()
        ),
        _ => format!(
            "{symbol} 신호 {}개, 체결 가정 {}개를 찾았습니다.",
            signals.len(),
            backtest.summary.filled_order_count
        ),
    };
    let preparation = StrategyPreparationView {
        required_history_bars: initial_readiness.map(|state| state.required_bars),
        provided_history_bars: warmup_count,
        available_history_bars: initial_readiness.map(|state| state.available_bars),
        ready_at_start: initial_readiness.map(|state| state.ready),
        ready_at_end: final_readiness.map(|state| state.ready),
        first_ready_time,
        unready_evaluation_bars,
        evaluated_bars: initial_readiness.map(|_| evaluated_bars),
        history_status: history_status.into(),
        indicator_status: indicator_status.into(),
        outcome: outcome.into(),
    };

    Ok(StrategyPreviewView {
        strategy_id: input.strategy_id,
        symbol,
        candles,
        signals,
        generated_at: chrono::Local::now().to_rfc3339(),
        message,
        replay,
        backtest,
        preparation,
    })
}
