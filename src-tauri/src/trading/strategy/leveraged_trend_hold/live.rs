use super::*;

impl Strategy for LeveragedTrendHoldStrategy {
    fn id(&self) -> &str {
        &self.config.id
    }
    fn name(&self) -> &str {
        &self.config.name
    }
    fn config(&self) -> &StrategyConfig {
        &self.config
    }
    fn config_mut(&mut self) -> &mut StrategyConfig {
        &mut self.config
    }
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn initialize_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        self.sync_params();
        if !self.is_target_symbol(symbol) {
            return;
        }
        // Daily bars are retained solely as bounded context. Reinitializing them
        // must not reset intraday observations, Bollinger, or execution state.
        let take = candles.len().min(self.window_cap());
        self.daily_context.insert(
            symbol.to_string(),
            candles[candles.len().saturating_sub(take)..]
                .iter()
                .copied()
                .collect(),
        );
        tracing::info!(
            "LTH daily context [{}]: {} bars (intraday unchanged)",
            symbol,
            take
        );
    }

    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.sync_params();
        if !self.is_target_symbol(symbol) {
            return;
        }
        let cap = self.rebound_price_cap();
        let window_cap = self.window_cap();
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| {
            LeveragedTrendHoldMarketState {
                intraday_candles: VecDeque::with_capacity(window_cap),
                rebound_prices: VecDeque::with_capacity(cap),
                live_candle_minute: None,
            }
        });
        state.rebound_prices.clear();
        let take = prices.len().min(cap);
        for price in prices[prices.len().saturating_sub(take)..]
            .iter()
            .copied()
            .filter(|price| *price > 0)
        {
            state.rebound_prices.push_back(price);
        }
        tracing::info!(
            "레버리지 단일 티커 장중 반동 초기화 [{}]: 가격 {}개 로드",
            symbol,
            state.rebound_prices.len()
        );
    }

    fn initialize_intraday_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        self.sync_params();
        if !self.is_target_symbol(symbol) {
            return;
        }
        let cap = self.window_cap();
        let rebound_cap = self.rebound_price_cap();
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| {
            LeveragedTrendHoldMarketState {
                intraday_candles: VecDeque::with_capacity(cap),
                rebound_prices: VecDeque::with_capacity(rebound_cap),
                live_candle_minute: None,
            }
        });

        // This is a replacement snapshot, never an append to daily/history bars.
        state.intraday_candles.clear();
        let take = candles.len().min(cap);
        for candle in &candles[candles.len().saturating_sub(take)..] {
            state.intraday_candles.push_back(*candle);
            while state.intraday_candles.len() > cap {
                state.intraday_candles.pop_front();
            }
        }

        state.rebound_prices.clear();
        let rebound_take = candles.len().min(rebound_cap);
        for candle in &candles[candles.len().saturating_sub(rebound_take)..] {
            if candle.close > 0 {
                state.rebound_prices.push_back(candle.close);
            }
        }
        state.live_candle_minute = None;
        self.bollinger_candles.insert(
            symbol.to_string(),
            candles
                .iter()
                .rev()
                .take(512)
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
        );
        tracing::info!(
            "레버리지 단일 티커 장중 OHLC 초기화 [{}]: 1분봉 {}개 로드, 반동 가격 {}개 로드",
            symbol,
            take,
            state.rebound_prices.len()
        );
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        self.sync_params();
        let Some(entry) = self.entry_for_symbol(symbol) else {
            return Signal::Hold;
        };
        let new_observation = self.update_target_tick(symbol, price);

        let quantity = entry.quantity.max(1);
        let (in_position, entry_price, high_water, held_observations) = self
            .positions
            .get(symbol)
            .map(|p| {
                (
                    p.in_position,
                    p.entry_price,
                    p.high_water,
                    p.held_observations,
                )
            })
            .unwrap_or((false, None, None, 0));

        if in_position {
            let entry_price = entry_price.unwrap_or(price);
            let high = high_water.unwrap_or(price).max(price);
            let held_observations = if new_observation {
                held_observations.saturating_add(1)
            } else {
                held_observations
            };
            if let Some(pos) = self.positions.get_mut(symbol) {
                pos.entry_price = Some(entry_price);
                pos.high_water = Some(high);
                pos.held_observations = held_observations;
            }

            if let Some(reason) = self
                .initial_risk_exit_reason(price, entry_price, high, held_observations)
                .or_else(|| self.bollinger_exit_reason(symbol, price, entry_price))
            {
                self.clear_position(symbol);
                return Signal::Sell {
                    symbol: symbol.to_string(),
                    quantity,
                    reason,
                };
            }

            if let Some(reason) =
                self.protection_exit_reason(symbol, price, entry_price, high, held_observations)
            {
                self.clear_position(symbol);
                return Signal::Sell {
                    symbol: symbol.to_string(),
                    quantity,
                    reason,
                };
            }

            if let Some((_, minutes_to_close)) =
                Self::session_minutes(entry.is_overseas, &self.params.toss_us_session)
            {
                if minutes_to_close <= self.params.exit_before_close_min {
                    self.clear_position(symbol);
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity,
                        reason: format!(
                            "LeveragedTrendHold 장마감 청산: 마감 {}분 전",
                            minutes_to_close
                        ),
                    };
                }
            }

            return Signal::Hold;
        }

        let Some((elapsed, _)) =
            Self::session_minutes(entry.is_overseas, &self.params.toss_us_session)
        else {
            return Signal::Hold;
        };
        let in_blackout = Self::in_blackout_window(&self.params.blackout_windows);
        let in_trend_window = elapsed >= self.params.entry_window_start_min
            && elapsed <= self.params.entry_window_end_min
            && !in_blackout;
        let can_check_rebound = self.params.intraday_rebound_enabled && !in_blackout;
        let can_check_rapid_rebound = self.params.rapid_rebound_enabled && !in_blackout;

        if can_check_rapid_rebound {
            if let Some(snap) = self.rapid_rebound_entry_ok(symbol) {
                self.positions.insert(
                    symbol.to_string(),
                    LeveragedTrendHoldPosition {
                        in_position: true,
                        entry_price: Some(price),
                        high_water: Some(price),
                        held_observations: 0,
                    },
                );
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity,
                    reason: format!(
                        "LeveragedTrendHold 급반등 단독 진입: 선행 급락 {:.2}%, 저점 대비 +{:.2}% (저점 후 {}관측치{})",
                        snap.drop_pct,
                        snap.recovery_pct,
                        snap.low_age_ticks,
                        Self::rapid_rebound_indicator_label(&snap)
                    ),
                };
            }
        }

        if in_trend_window {
            if let Some(snap) = self.entry_ok(symbol) {
                self.positions.insert(
                    symbol.to_string(),
                    LeveragedTrendHoldPosition {
                        in_position: true,
                        entry_price: Some(price),
                        high_water: Some(price),
                        held_observations: 0,
                    },
                );
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity,
                    reason: format!(
                        "LeveragedTrendHold 상승 추세 진입: {} EMA{} > EMA{}, RSI {:.1}, ADX {:.1}, 최근 3봉 양봉 {}개",
                        symbol,
                        self.params.ema_short_period,
                        self.params.ema_long_period,
                        snap.rsi,
                        snap.adx,
                        snap.bullish_count_3
                    ),
                };
            }
        }

        if can_check_rebound {
            if let Some(snap) = self.rebound_entry_ok(symbol) {
                self.positions.insert(
                    symbol.to_string(),
                    LeveragedTrendHoldPosition {
                        in_position: true,
                        entry_price: Some(price),
                        high_water: Some(price),
                        held_observations: 0,
                    },
                );
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity,
                    reason: format!(
                        "LeveragedTrendHold 장중 매수세 반동 진입: 확인 구간 +{:.2}% (기준 구간 하락 {:.2}%, 저점 대비 +{:.2}%{})",
                        snap.buy_pressure_pct,
                        snap.pullback_pct,
                        snap.rebound_from_low_pct,
                        Self::rebound_indicator_label(&snap)
                    ),
                };
            }
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        self.sync_params();
        if !self.is_target_symbol(symbol) {
            return;
        }
        if quantity == 0 {
            self.positions.remove(symbol);
            return;
        }
        self.positions.insert(
            symbol.to_string(),
            LeveragedTrendHoldPosition {
                in_position: true,
                entry_price: Some(avg_price),
                high_water: Some(avg_price),
                held_observations: self.min_hold_observations(),
            },
        );
        tracing::info!(
            "레버리지 단일 티커 추세 포지션 동기화: {} {}주 @ {}",
            symbol,
            quantity,
            avg_price
        );
    }

    fn reset(&mut self) {
        for state in self.states.values_mut() {
            state.live_candle_minute = None;
            state.rebound_prices.clear();
        }
        for pos in self.positions.values_mut() {
            pos.in_position = false;
            pos.entry_price = None;
            pos.high_water = None;
            pos.held_observations = 0;
        }
    }
}
