//! Shared production replay driver. Chunk boundaries never recreate strategy/risk state.
use super::continuous::ChunkContinuityView;
use super::generic::{signal_to_preview_view, PreviewRow};
use super::*;
use crate::trading::simulation::ExecutionModel;
use crate::trading::strategy::{
    build_strategy, initialize_strategy_warmup_with_daily_bars, parse_completed_daily_volume,
    CompletedDailyBar,
};

pub(super) fn run_prepared_preview(
    input: StrategyPreviewInput,
    symbol: String,
    history_rows: Vec<PreviewRow>,
    replay_chunks: Vec<Vec<PreviewRow>>,
    calculation_mode: &str,
) -> CmdResult<(StrategyPreviewView, Vec<ChunkContinuityView>)> {
    let interval = input.interval.as_deref().unwrap_or("1d").to_string();
    let daily = matches!(interval.as_str(), "D" | "1d");
    crate::trading::strategy::validate_failed_breakout_params(&input.strategy_id, &input.params)
        .map_err(|message| CmdError {
            code: "INVALID_STRATEGY_PARAMS".into(),
            message,
        })?;
    crate::trading::strategy::validate_volatility_expansion_params(
        &input.strategy_id,
        &input.params,
    )
    .map_err(|message| CmdError {
        code: "INVALID_STRATEGY_PARAMS".into(),
        message,
    })?;
    crate::trading::strategy::validate_strong_close_params(&input.strategy_id, &input.params)
        .map_err(|message| CmdError {
            code: "INVALID_STRATEGY_PARAMS".into(),
            message,
        })?;
    if input.strategy_id.starts_with("failed_breakout")
        && !daily
        && (input
            .params
            .get("breakout_basis")
            .and_then(serde_json::Value::as_str)
            == Some("completedHigh")
            || ["atr_buffer_multiplier", "min_volume_ratio"]
                .iter()
                .any(|key| {
                    input
                        .params
                        .get(*key)
                        .and_then(serde_json::Value::as_f64)
                        .is_some_and(|v| v > 0.0)
                }))
    {
        return Err(CmdError {
            code: "UNSUPPORTED_ENTRY_FILTER_INTERVAL".into(),
            message:
                "돌파 진입 확인은 이전 완료 일봉 자료를 요구합니다. 일봉(D/1d) replay를 선택하세요."
                    .into(),
        });
    }
    crate::trading::strategy::validate_rsi_params(&input.strategy_id, &input.params).map_err(
        |message| CmdError {
            code: "INVALID_STRATEGY_PARAMS".into(),
            message,
        },
    )?;
    crate::trading::strategy::validate_ma_cross_entry_params(&input.strategy_id, &input.params)
        .map_err(|message| CmdError {
            code: "INVALID_STRATEGY_PARAMS".into(),
            message,
        })?;
    if input.strategy_id.starts_with("ma_cross")
        && !daily
        && [
            "long_trend_period",
            "min_long_ma_slope_pct",
            "min_atr_pct",
            "min_bandwidth_pct",
        ]
        .iter()
        .any(|key| {
            input
                .params
                .get(*key)
                .and_then(serde_json::Value::as_f64)
                .is_some_and(|v| v > 0.0)
        })
    {
        return Err(CmdError {
            code: "UNSUPPORTED_ENTRY_FILTER_INTERVAL".into(),
            message:
                "MA 진입 확인은 직전 완료 일봉 맥락을 요구합니다. 일봉(D/1d) replay를 선택하세요."
                    .into(),
        });
    }
    crate::trading::strategy::validate_consecutive_entry_params(&input.strategy_id, &input.params)
        .map_err(|message| CmdError {
            code: "INVALID_STRATEGY_PARAMS".into(),
            message,
        })?;
    crate::trading::strategy::validate_momentum_entry_params(&input.strategy_id, &input.params)
        .map_err(|message| CmdError {
            code: "INVALID_STRATEGY_PARAMS".into(),
            message,
        })?;
    crate::trading::strategy::validate_deviation_entry_params(&input.strategy_id, &input.params)
        .map_err(|message| CmdError {
            code: "INVALID_STRATEGY_PARAMS".into(),
            message,
        })?;
    crate::trading::strategy::validate_mean_reversion_params(&input.strategy_id, &input.params)
        .map_err(|message| CmdError {
            code: "INVALID_STRATEGY_PARAMS".into(),
            message,
        })?;
    if input.strategy_id.starts_with("consecutive_move")
        && !daily
        && ["min_move_atr_multiplier", "long_trend_period"]
            .iter()
            .any(|key| {
                input
                    .params
                    .get(*key)
                    .and_then(serde_json::Value::as_f64)
                    .is_some_and(|value| value > 0.0)
            })
    {
        return Err(CmdError {code:"UNSUPPORTED_ENTRY_FILTER_INTERVAL".into(), message:"ATR/장기 MA 진입 확인은 직전 완료 일봉 맥락을 요구합니다. 일봉(D/1d) replay를 선택하세요.".into()});
    }
    let hash_daily_volume = input.strategy_id.starts_with("failed_breakout")
        && input
            .params
            .get("min_volume_ratio")
            .and_then(serde_json::Value::as_f64)
            .is_some_and(|v| v > 0.0);
    let data_source = input
        .data_source
        .clone()
        .unwrap_or_else(|| "providedChartCandles".into());
    let strategy_version = input
        .strategy_version
        .clone()
        .unwrap_or_else(|| REPLAY_ENGINE_VERSION.into());
    let mut assumptions = input.assumptions.clone();
    let next_open = assumptions.execution_model == ExecutionModel::NextOpen;
    let params_fingerprint = serde_json::to_string(&input.params).unwrap_or_default();
    let scope_fingerprint = format!(
        "{:?}|{}",
        input.broker_id,
        input.broker_account_id.as_deref().unwrap_or_default()
    );
    let replay_len = replay_chunks.iter().map(Vec::len).sum();
    align_accounting_scenarios(
        &mut assumptions,
        &replay_chunks
            .iter()
            .flatten()
            .map(|row| row.0.date.clone())
            .collect::<Vec<_>>(),
        &interval,
        input.is_overseas,
    )?;
    let assumptions_fingerprint = serde_json::to_string(&assumptions).unwrap_or_default();
    let candles = replay_chunks
        .iter()
        .flatten()
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
            initialize_strategy_warmup_with_daily_bars(strategy.as_mut(), &symbol, &[], &ohlc);
        } else {
            let bars = history_rows
                .iter()
                .map(|(candle, ohlc, _)| CompletedDailyBar {
                    ohlc: *ohlc,
                    volume: parse_completed_daily_volume(&candle.volume),
                })
                .collect::<Vec<_>>();
            initialize_strategy_warmup_with_daily_bars(strategy.as_mut(), &symbol, &bars, &[]);
        }
    }

    let initial_readiness = strategy.history_readiness(&symbol);
    let mut first_ready_time = initial_readiness
        .filter(|state| state.ready)
        .map(|_| replay_chunks[0][0].0.date.clone());
    let mut unready_evaluation_bars = 0;
    let mut evaluated_bars = 0;

    let mut signals = Vec::new();
    let history_events = history_rows
        .iter()
        .map(|(candle, ohlc, _)| SimulationEvent {
            time: candle.date.clone(),
            chart_time: candle.date.clone(),
            open_units: ohlc.open,
            close_units: ohlc.close,
            high_units: ohlc.high,
            low_units: ohlc.low,
            signal: None,
        })
        .collect::<Vec<_>>();
    let mut session = ReplaySession::new(
        &input.strategy_id,
        &symbol,
        input.is_overseas,
        assumptions,
        &history_events,
        replay_len,
    );
    let mut simulation_events = Vec::with_capacity(replay_len);
    let mut synchronized_position = (0, 0);
    let mut chunk_ledger = Vec::with_capacity(replay_chunks.len());
    for (chunk_index, chunk) in replay_chunks.iter().enumerate() {
        let start_quantity = session.position_snapshot().0;
        for (candle, ohlc, volume) in chunk {
            let mut event = SimulationEvent {
                time: candle.date.clone(),
                chart_time: candle.date.clone(),
                open_units: ohlc.open,
                close_units: ohlc.close,
                high_units: ohlc.high,
                low_units: ohlc.low,
                signal: None,
            };
            session.begin_bar(&event);
            let snapshot = session.position_snapshot();
            if snapshot != synchronized_position {
                strategy.sync_position(&symbol, snapshot.0, snapshot.1);
                synchronized_position = snapshot;
            }
            if input.strategy_id.starts_with("failed_breakout")
                || input.strategy_id.starts_with("strong_close")
                || input.strategy_id.starts_with("volatility_expansion")
            {
                strategy.set_order_pending(&symbol, session.has_pending_signal());
            }
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
            event.signal = (signal != Signal::Hold).then(|| signal.clone());
            session.process_close(&event);
            let snapshot = session.position_snapshot();
            if signal != Signal::Hold || snapshot != synchronized_position {
                strategy.sync_position(&symbol, snapshot.0, snapshot.1);
                synchronized_position = snapshot;
            }
            if input.strategy_id.starts_with("failed_breakout")
                || input.strategy_id.starts_with("strong_close")
                || input.strategy_id.starts_with("volatility_expansion")
            {
                strategy.set_order_pending(&symbol, session.has_pending_signal());
            }
            simulation_events.push(event);
            if daily {
                strategy.on_completed_daily_bar(
                    &symbol,
                    &CompletedDailyBar {
                        ohlc: *ohlc,
                        volume: parse_completed_daily_volume(&candle.volume),
                    },
                );
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

        chunk_ledger.push(ChunkContinuityView {
            index: chunk_index,
            start: chunk
                .first()
                .map(|row| row.0.date.clone())
                .unwrap_or_default(),
            end: chunk
                .last()
                .map(|row| row.0.date.clone())
                .unwrap_or_default(),
            bars: chunk.len(),
            start_quantity,
            end_quantity: session.position_snapshot().0,
            pending_at_end: session.has_pending_signal(),
        });
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
        .chain(replay_chunks.iter().flatten())
        .map(|(candle, ohlc, volume)| {
            let base = format!(
                "{}:{}:{}:{}:{}:{}",
                candle.date, ohlc.open, ohlc.high, ohlc.low, ohlc.close, volume
            );
            if hash_daily_volume {
                format!(
                    "{base}:dailyVolume={}",
                    parse_completed_daily_volume(&candle.volume)
                        .map_or_else(|| "unknown".into(), |v| v.to_string())
                )
            } else {
                base
            }
        })
        .collect::<Vec<_>>()
        .join("|");
    let order_quantity_fingerprint = input.order_quantity.to_string();
    let warmup_count_fingerprint = warmup_count.to_string();
    let input_hash = replay_input_hash(&[
        REPLAY_ENGINE_VERSION,
        calculation_mode,
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
        look_ahead_safe: next_open,
        input_hash,
        assessment: None,
    };
    let mut backtest = session.finish();
    crate::trading::simulation::attach_comparison(
        &mut backtest,
        &input.strategy_id,
        &symbol,
        input.is_overseas,
        input.order_quantity.max(1),
        &history_events,
        &simulation_events,
    );
    let final_readiness = strategy.history_readiness(&symbol);
    let volatility_expansion_preparation = strategy.volatility_expansion_preparation(&symbol);
    let entry_filter_preparation = strategy.entry_filter_preparation(&symbol);
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

    Ok((
        StrategyPreviewView {
            strategy_id: input.strategy_id,
            symbol,
            candles,
            signals,
            generated_at: chrono::Local::now().to_rfc3339(),
            message,
            replay,
            backtest,
            preparation,
            entry_filter_preparation,
            volatility_expansion_preparation,
        },
        chunk_ledger,
    ))
}
