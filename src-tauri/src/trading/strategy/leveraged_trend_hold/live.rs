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
    fn has_held_position(&self, symbol: &str) -> Option<bool> {
        Some(
            self.positions
                .get(symbol)
                .is_some_and(|position| position.in_position),
        )
    }
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn reconfigure_preserving_position(&mut self, config: StrategyConfig) -> bool {
        if !self.config.same_identity(&config) {
            return false;
        }
        let mut replacement = Self::new(config);
        replacement.positions = std::mem::take(&mut self.positions);
        replacement
            .positions
            .retain(|symbol, _| replacement.config.targets_symbol(symbol));
        replacement.states = std::mem::take(&mut self.states);
        replacement
            .states
            .retain(|symbol, _| replacement.config.targets_symbol(symbol));
        let window_cap = replacement.window_cap();
        let rebound_cap = replacement.rebound_price_cap();
        for state in replacement.states.values_mut() {
            while state.intraday_candles.len() > window_cap {
                state.intraday_candles.pop_front();
            }
            while state.rebound_prices.len() > rebound_cap {
                state.rebound_prices.pop_front();
            }
        }
        replacement.daily_context = std::mem::take(&mut self.daily_context);
        replacement
            .daily_context
            .retain(|symbol, _| replacement.config.targets_symbol(symbol));
        for candles in replacement.daily_context.values_mut() {
            while candles.len() > window_cap {
                candles.pop_front();
            }
        }
        replacement.bollinger_candles = std::mem::take(&mut self.bollinger_candles);
        replacement
            .bollinger_candles
            .retain(|symbol, _| replacement.config.targets_symbol(symbol));
        *self = replacement;
        true
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
        self.evaluate_tick(symbol, price, false)
    }

    fn on_live_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        self.evaluate_tick(symbol, price, true)
    }

    fn owned_position_metadata(&self, symbol: &str) -> Option<serde_json::Value> {
        let position = self
            .positions
            .get(symbol)
            .filter(|position| position.in_position)?;
        Some(serde_json::json!({
            "version": 1,
            "kind": "leveragedTrendHold",
            "entryPrice": position.entry_price?,
            "highWater": position.high_water?,
            "heldObservations": position.held_observations,
        }))
    }

    fn restore_owned_position_metadata(
        &mut self,
        symbol: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Protection {
            version: u64,
            kind: String,
            entry_price: u64,
            high_water: u64,
            held_observations: usize,
        }
        let protection: Protection = serde_json::from_value(metadata.clone())
            .map_err(|error| format!("Invalid LTH protection checkpoint: {error}"))?;
        if protection.version != 1
            || protection.kind != "leveragedTrendHold"
            || protection.entry_price == 0
            || protection.high_water == 0
        {
            return Err("Invalid LTH protection checkpoint identity or price".into());
        }
        let position = self
            .positions
            .get_mut(symbol)
            .filter(|position| position.in_position)
            .ok_or("Cannot restore LTH protection without confirmed owned quantity")?;
        // Actual owned average remains authoritative, including partial buys.
        position.high_water = Some(protection.high_water);
        position.held_observations = protection.held_observations;
        Ok(())
    }

    fn sync_owned_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        self.sync_params();
        if !self.is_target_symbol(symbol) {
            return;
        }
        if quantity == 0 {
            self.positions.remove(symbol);
            return;
        }
        let position =
            self.positions
                .entry(symbol.to_string())
                .or_insert(LeveragedTrendHoldPosition {
                    in_position: true,
                    entry_price: Some(avg_price),
                    high_water: Some(avg_price),
                    held_observations: 0,
                });
        position.in_position = true;
        position.entry_price = Some(avg_price);
        // Partial fills do not reset trailing activation or observation age.
        position.high_water = Some(position.high_water.unwrap_or(avg_price));
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

impl LeveragedTrendHoldStrategy {
    fn evaluate_tick(&mut self, symbol: &str, price: u64, live: bool) -> Signal {
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
                if !live {
                    self.clear_position(symbol);
                }
                return Signal::Sell {
                    symbol: symbol.to_string(),
                    quantity,
                    reason,
                };
            }

            if let Some(reason) =
                self.protection_exit_reason(symbol, price, entry_price, high, held_observations)
            {
                if !live {
                    self.clear_position(symbol);
                }
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
                    if !live {
                        self.clear_position(symbol);
                    }
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
                if !live {
                    self.positions.insert(
                        symbol.to_string(),
                        LeveragedTrendHoldPosition {
                            in_position: true,
                            entry_price: Some(price),
                            high_water: Some(price),
                            held_observations: 0,
                        },
                    );
                }
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
                if !live {
                    self.positions.insert(
                        symbol.to_string(),
                        LeveragedTrendHoldPosition {
                            in_position: true,
                            entry_price: Some(price),
                            high_water: Some(price),
                            held_observations: 0,
                        },
                    );
                }
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
                if !live {
                    self.positions.insert(
                        symbol.to_string(),
                        LeveragedTrendHoldPosition {
                            in_position: true,
                            entry_price: Some(price),
                            high_water: Some(price),
                            held_observations: 0,
                        },
                    );
                }
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
}
