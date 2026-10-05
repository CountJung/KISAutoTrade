use super::*;

impl LeveragedTrendHoldStrategy {
    pub(super) fn exit_reason(&self, symbol: &str) -> Option<String> {
        let state = self.states.get(symbol)?;
        let snap = self.snapshot_for(symbol)?;
        let close = state.intraday_candles.back()?.close as f64;

        if close < snap.ema_short {
            return Some(format!(
                "{} EMA{} 하향 이탈",
                symbol, self.params.ema_short_period
            ));
        }
        if snap.ema_short < snap.ema_long {
            return Some(format!(
                "{} EMA{} < EMA{}",
                symbol, self.params.ema_short_period, self.params.ema_long_period
            ));
        }
        if snap.rsi < self.params.exit_rsi_below {
            return Some(format!(
                "{} RSI {:.1} < {:.1}",
                symbol, snap.rsi, self.params.exit_rsi_below
            ));
        }
        None
    }

    pub(super) fn clear_position(&mut self, symbol: &str) {
        if let Some(pos) = self.positions.get_mut(symbol) {
            pos.in_position = false;
            pos.entry_price = None;
            pos.high_water = None;
            pos.held_observations = 0;
        }
    }

    pub(super) fn min_hold_observations(&self) -> usize {
        self.params.min_hold_observations.min(120)
    }

    pub(super) fn entry_failure_observations(&self) -> usize {
        self.params.entry_failure_observations.clamp(1, 120)
    }

    pub(super) fn failed_rebound_observations(&self) -> usize {
        self.entry_failure_observations()
            .max(self.min_hold_observations())
    }

    pub(super) fn profit_pct(entry_price: u64, price: u64) -> Option<f64> {
        if entry_price == 0 {
            return None;
        }
        Some((price as f64 - entry_price as f64) / entry_price as f64 * 100.0)
    }

    pub(super) fn initial_risk_exit_reason(
        &self,
        price: u64,
        entry_price: u64,
        high_water: u64,
        held_observations: usize,
    ) -> Option<String> {
        let current_profit = Self::profit_pct(entry_price, price)?;
        let initial_stop = self.params.initial_stop_loss_pct.clamp(0.1, 20.0);
        if current_profit <= -initial_stop {
            return Some(format!(
                "LeveragedTrendHold 초기 손절: 현재 수익률 {:.2}% <= -{:.2}%",
                current_profit, initial_stop
            ));
        }

        let activation_profit = self.params.trailing_activation_profit_pct.clamp(0.1, 100.0);
        let high_profit = Self::profit_pct(entry_price, high_water)?;
        if held_observations >= self.failed_rebound_observations()
            && high_profit < activation_profit
            && current_profit < 0.0
        {
            return Some(format!(
                "LeveragedTrendHold 반등 실패 손절: {}관측치 동안 활성 수익 {:.2}% 미도달, 현재 수익률 {:.2}%",
                held_observations, activation_profit, current_profit
            ));
        }

        None
    }

    pub(super) fn protection_exit_reason(
        &self,
        symbol: &str,
        price: u64,
        entry_price: u64,
        high_water: u64,
        held_observations: usize,
    ) -> Option<String> {
        if held_observations < self.min_hold_observations() || high_water == 0 {
            return None;
        }

        let activation_profit = self.params.trailing_activation_profit_pct.clamp(0.1, 100.0);
        let high_profit = Self::profit_pct(entry_price, high_water)?;
        if high_profit < activation_profit {
            return None;
        }

        let buffer = self.params.breakeven_buffer_pct.clamp(0.0, 20.0);
        let current_profit = Self::profit_pct(entry_price, price)?;
        if current_profit <= buffer {
            return Some(format!(
                "LeveragedTrendHold 본전 보호 청산: 고점 수익률 {:.2}% 활성 후 현재 수익률 {:.2}% <= 보호 버퍼 {:.2}%",
                high_profit, current_profit, buffer
            ));
        }

        let trailing_stop = self.params.trailing_stop_pct.clamp(0.1, 100.0);
        let drawdown = (high_water as f64 - price as f64) / high_water as f64 * 100.0;
        if drawdown >= trailing_stop {
            return Some(format!(
                "LeveragedTrendHold 수익 보호 추적손절: 고점 대비 -{:.2}% (기준 {:.2}%, 고점 수익률 {:.2}%)",
                drawdown, trailing_stop, high_profit
            ));
        }

        if let Some(reason) = self.exit_reason(symbol) {
            return Some(format!(
                "LeveragedTrendHold 수익 보호 추세 청산: {} (현재 수익률 {:.2}%, 보호 버퍼 {:.2}%)",
                reason, current_profit, buffer
            ));
        }

        None
    }
}
