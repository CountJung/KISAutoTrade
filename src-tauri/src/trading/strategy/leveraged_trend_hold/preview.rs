use super::*;

impl LeveragedTrendHoldStrategy {
    pub(super) fn preview_snapshot(
        &self,
        symbol: &str,
    ) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
        let Some(state) = self.states.get(symbol) else {
            return (None, None, None, None);
        };
        let closes = Self::closes(&state.intraday_candles);
        (
            Self::ema(&closes, self.params.ema_short_period),
            Self::ema(&closes, self.params.ema_long_period),
            Self::rsi(&closes, self.params.rsi_period),
            Self::adx(&state.intraday_candles, self.params.adx_period),
        )
    }

    pub(super) fn preview_signal(
        &self,
        time: &str,
        side: &str,
        symbol: &str,
        price: u64,
        quantity: u64,
        reason: String,
    ) -> LeveragedTrendHoldPreviewSignal {
        let (ema_short, ema_long, rsi, adx) = self.preview_snapshot(symbol);
        LeveragedTrendHoldPreviewSignal {
            time: time.to_string(),
            side: side.to_string(),
            price_units: price,
            quantity,
            reason,
            ema_short,
            ema_long,
            rsi,
            adx,
        }
    }

    pub(super) fn rebound_indicator_label(snap: &LeveragedReboundSnapshot) -> String {
        match (snap.rsi, snap.adx) {
            (Some(rsi), Some(adx)) => format!(", RSI {:.1}, ADX {:.1}", rsi, adx),
            (Some(rsi), None) => format!(", RSI {:.1}, ADX 준비 전", rsi),
            (None, Some(adx)) => format!(", RSI 준비 전, ADX {:.1}", adx),
            (None, None) => ", RSI/ADX 준비 전".to_string(),
        }
    }

    pub(super) fn rapid_rebound_indicator_label(snap: &LeveragedRapidReboundSnapshot) -> String {
        match (snap.rsi, snap.adx) {
            (Some(rsi), Some(adx)) => format!(", RSI {:.1}, ADX {:.1}", rsi, adx),
            (Some(rsi), None) => format!(", RSI {:.1}, ADX 준비 전", rsi),
            (None, Some(adx)) => format!(", RSI 준비 전, ADX {:.1}", adx),
            (None, None) => ", RSI/ADX 준비 전".to_string(),
        }
    }

    pub fn preview_signals(
        symbol: &str,
        params: LeveragedTrendHoldParams,
        daily_candles: &[OhlcCandle],
        intraday_candles: &[LeveragedTrendHoldTimedCandle],
    ) -> Vec<LeveragedTrendHoldPreviewSignal> {
        Self::preview_signals_with_execution(
            symbol,
            params,
            daily_candles,
            intraday_candles,
            |_, _| true,
        )
    }

    /// raw signal마다 실제 replay 체결 여부를 되돌려 받아 다음 신호의 포지션 상태를 결정한다.
    pub fn preview_signals_with_execution<F>(
        symbol: &str,
        params: LeveragedTrendHoldParams,
        daily_candles: &[OhlcCandle],
        intraday_candles: &[LeveragedTrendHoldTimedCandle],
        mut execute: F,
    ) -> Vec<LeveragedTrendHoldPreviewSignal>
    where
        F: FnMut(&LeveragedTrendHoldPreviewSignal, &LeveragedTrendHoldTimedCandle) -> bool,
    {
        let normalized_symbol = symbol.trim().to_uppercase();
        if normalized_symbol.is_empty() || intraday_candles.is_empty() {
            return Vec::new();
        }

        let mut params = params;
        if !params.entries.iter().any(|entry| {
            entry
                .leveraged_symbol
                .eq_ignore_ascii_case(&normalized_symbol)
        }) {
            params.entries.push(LeveragedTrendHoldEntry {
                leveraged_symbol: normalized_symbol.clone(),
                leveraged_symbol_name: normalized_symbol.clone(),
                inverse_leveraged_symbol: String::new(),
                inverse_leveraged_symbol_name: String::new(),
                base_symbols: Vec::new(),
                base_symbol_names: HashMap::new(),
                base_symbol_roles: HashMap::new(),
                quantity: lth_default_qty(),
                inverse_quantity: lth_default_qty(),
                is_overseas: true,
            });
        }

        let config = StrategyConfig::new(
            "leveraged_trend_hold_preview",
            "레버리지 단일 티커 추세 미리보기",
            true,
            vec![normalized_symbol.clone()],
            1,
            serde_json::to_value(&params).unwrap_or_default(),
        );
        let mut strategy = Self::new(config);
        strategy.params = params;
        strategy.config.target_symbols = Self::target_symbols_for_params(&strategy.params);
        let Some(entry) = strategy.entry_for_symbol(&normalized_symbol) else {
            return Vec::new();
        };
        let quantity = entry.quantity.max(1);
        let cap = strategy.window_cap();
        let rebound_cap = strategy.rebound_price_cap();
        strategy.initialize_ohlc(&normalized_symbol, daily_candles);
        let state = LeveragedTrendHoldMarketState {
            intraday_candles: VecDeque::with_capacity(cap),
            rebound_prices: VecDeque::with_capacity(rebound_cap),
            live_candle_minute: None,
        };
        strategy.states.insert(normalized_symbol.clone(), state);

        let mut signals = Vec::new();
        let mut in_position = false;
        let mut entry_price: Option<u64> = None;
        let mut high_water: Option<u64> = None;
        let mut held_observations = 0usize;

        for timed in intraday_candles {
            let price = timed.candle.close;
            if price == 0 {
                continue;
            }
            let Some(state) = strategy.states.get_mut(&normalized_symbol) else {
                continue;
            };
            state.intraday_candles.push_back(timed.candle);
            while state.intraday_candles.len() > cap {
                state.intraday_candles.pop_front();
            }
            state.rebound_prices.push_back(price);
            while state.rebound_prices.len() > rebound_cap {
                state.rebound_prices.pop_front();
            }

            strategy.update_bollinger_candle(&normalized_symbol, timed.candle, true);

            if in_position {
                let high = high_water.unwrap_or(price).max(price);
                high_water = Some(high);
                held_observations = held_observations.saturating_add(1);
                if let Some(reason) = entry_price.and_then(|entry| {
                    strategy
                        .initial_risk_exit_reason(price, entry, high, held_observations)
                        .or_else(|| {
                            strategy.bollinger_exit_reason(&normalized_symbol, price, entry)
                        })
                }) {
                    let signal = strategy.preview_signal(
                        &timed.time,
                        "sell",
                        &normalized_symbol,
                        price,
                        quantity,
                        reason,
                    );
                    let filled = execute(&signal, timed);
                    signals.push(signal);
                    if filled {
                        in_position = false;
                        entry_price = None;
                        high_water = None;
                        held_observations = 0;
                    }
                    continue;
                }
                if let Some(reason) = entry_price.and_then(|entry| {
                    strategy.protection_exit_reason(
                        &normalized_symbol,
                        price,
                        entry,
                        high,
                        held_observations,
                    )
                }) {
                    let signal = strategy.preview_signal(
                        &timed.time,
                        "sell",
                        &normalized_symbol,
                        price,
                        quantity,
                        reason,
                    );
                    let filled = execute(&signal, timed);
                    signals.push(signal);
                    if filled {
                        in_position = false;
                        entry_price = None;
                        high_water = None;
                        held_observations = 0;
                    }
                    continue;
                }
                if let Some(mins) = Self::minute_of_day_from_time(&timed.time) {
                    if let Some((_, minutes_to_close)) = Self::preview_session_minutes(
                        mins,
                        entry.is_overseas,
                        &strategy.params.toss_us_session,
                    ) {
                        if minutes_to_close <= strategy.params.exit_before_close_min {
                            let signal = strategy.preview_signal(
                                &timed.time,
                                "sell",
                                &normalized_symbol,
                                price,
                                quantity,
                                format!(
                                    "LeveragedTrendHold 장마감 청산: 마감 {minutes_to_close}분 전"
                                ),
                            );
                            let filled = execute(&signal, timed);
                            signals.push(signal);
                            if filled {
                                in_position = false;
                                entry_price = None;
                                high_water = None;
                                held_observations = 0;
                            }
                        }
                    }
                }
                continue;
            }

            let session = Self::minute_of_day_from_time(&timed.time).and_then(|mins| {
                Self::preview_session_minutes(
                    mins,
                    entry.is_overseas,
                    &strategy.params.toss_us_session,
                )
                .filter(|_| !Self::is_blackout_minute(mins, &strategy.params.blackout_windows))
            });
            let Some((elapsed, _)) = session else {
                continue;
            };

            let in_trend_window = elapsed >= strategy.params.entry_window_start_min
                && elapsed <= strategy.params.entry_window_end_min;
            if let Some(snap) = strategy.rapid_rebound_entry_ok(&normalized_symbol) {
                let signal = strategy.preview_signal(
                    &timed.time,
                    "buy",
                    &normalized_symbol,
                    price,
                    quantity,
                    format!(
                        "LeveragedTrendHold 급반등 단독 진입: 선행 급락 {:.2}%, 저점 대비 +{:.2}% (저점 후 {}관측치{})",
                        snap.drop_pct,
                        snap.recovery_pct,
                        snap.low_age_ticks,
                        Self::rapid_rebound_indicator_label(&snap)
                    ),
                );
                let filled = execute(&signal, timed);
                signals.push(signal);
                if filled {
                    in_position = true;
                    entry_price = Some(price);
                    high_water = Some(price);
                    held_observations = 0;
                }
                continue;
            }

            if in_trend_window {
                if let Some(snap) = strategy.entry_ok(&normalized_symbol) {
                    let signal = strategy.preview_signal(
                        &timed.time,
                        "buy",
                        &normalized_symbol,
                        price,
                        quantity,
                        format!(
                            "LeveragedTrendHold 상승 추세 진입: {} EMA{} > EMA{}, RSI {:.1}, ADX {:.1}, 최근 3봉 양봉 {}개",
                            normalized_symbol,
                            strategy.params.ema_short_period,
                            strategy.params.ema_long_period,
                            snap.rsi,
                            snap.adx,
                            snap.bullish_count_3
                        ),
                    );
                    let filled = execute(&signal, timed);
                    signals.push(signal);
                    if filled {
                        in_position = true;
                        entry_price = Some(price);
                        high_water = Some(price);
                        held_observations = 0;
                    }
                    continue;
                }
            }

            if let Some(snap) = strategy.rebound_entry_ok(&normalized_symbol) {
                let signal = strategy.preview_signal(
                    &timed.time,
                    "buy",
                    &normalized_symbol,
                    price,
                    quantity,
                    format!(
                        "LeveragedTrendHold 장중 매수세 반동 진입: 확인 구간 +{:.2}% (기준 구간 하락 {:.2}%, 저점 대비 +{:.2}%{})",
                        snap.buy_pressure_pct,
                        snap.pullback_pct,
                        snap.rebound_from_low_pct,
                        Self::rebound_indicator_label(&snap)
                    ),
                );
                let filled = execute(&signal, timed);
                signals.push(signal);
                if filled {
                    in_position = true;
                    entry_price = Some(price);
                    high_water = Some(price);
                    held_observations = 0;
                }
            }
        }

        signals
    }
}
