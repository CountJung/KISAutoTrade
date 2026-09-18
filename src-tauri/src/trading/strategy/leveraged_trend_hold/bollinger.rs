//! Optional confirmation filter for the existing strategy. Only completed bars
//! from one interval are used; daily indicator warmup never enters this stream.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BollingerParams {
    pub enabled: bool,
    pub period: usize,
    pub stddev_multiplier: f64,
    pub trend_period: usize,
    pub squeeze_lookback: usize,
    pub squeeze_ratio: f64,
    pub atr_stop_multiplier: f64,
}

impl Default for BollingerParams {
    fn default() -> Self {
        Self {
            enabled: false,
            period: 20,
            stddev_multiplier: 2.0,
            trend_period: 60,
            squeeze_lookback: 20,
            squeeze_ratio: 0.5,
            atr_stop_multiplier: 2.0,
        }
    }
}

struct Band {
    middle: f64,
    upper: f64,
    width: f64,
}

fn band(values: &[OhlcCandle], period: usize, multiplier: f64) -> Option<Band> {
    let window = values.get(values.len().checked_sub(period)?..)?;
    if window.iter().any(|c| c.close == 0) {
        return None;
    }
    let middle = window.iter().map(|c| c.close as f64).sum::<f64>() / period as f64;
    let variance = window
        .iter()
        .map(|c| (c.close as f64 - middle).powi(2))
        .sum::<f64>()
        / period as f64;
    let deviation = variance.sqrt() * multiplier;
    Some(Band {
        middle,
        upper: middle + deviation,
        width: 2.0 * deviation / middle,
    })
}

impl LeveragedTrendHoldStrategy {
    pub(super) fn update_bollinger_candle(
        &mut self,
        symbol: &str,
        candle: OhlcCandle,
        new_bar: bool,
    ) {
        let candles = self
            .bollinger_candles
            .entry(symbol.to_string())
            .or_default();
        if new_bar || candles.is_empty() {
            candles.push_back(candle);
        } else if let Some(last) = candles.back_mut() {
            *last = candle;
        }
        while candles.len() > 512 {
            candles.pop_front();
        }
    }

    fn completed_bollinger_candles(&self, symbol: &str) -> Option<Vec<OhlcCandle>> {
        let candles = self.bollinger_candles.get(symbol)?;
        // Both live and replay confirm the preceding bar on the next observation.
        Some(
            candles
                .iter()
                .take(candles.len().saturating_sub(1))
                .copied()
                .collect(),
        )
    }

    pub(super) fn bollinger_entry_allowed(&self, symbol: &str) -> bool {
        let p = &self.params.bollinger;
        if !p.enabled {
            return true;
        }
        self.bollinger_breakout(symbol).unwrap_or(false)
    }

    fn bollinger_breakout(&self, symbol: &str) -> Option<bool> {
        let p = &self.params.bollinger;
        if !p.stddev_multiplier.is_finite() || !p.squeeze_ratio.is_finite() {
            return None;
        }
        let period = p.period.clamp(5, 100);
        let lookback = p.squeeze_lookback.clamp(5, 100);
        let trend = p.trend_period.clamp(period, 250);
        let multiplier = p.stddev_multiplier.clamp(1.0, 4.0);
        let candles = self.completed_bollinger_candles(symbol)?;
        let n = candles.len();
        if n < (period + lookback).max(trend + 1) {
            return None;
        }
        let current = band(&candles, period, multiplier)?;
        let previous = band(&candles[..n - 1], period, multiplier)?;
        let long_ma = candles[n - trend..]
            .iter()
            .map(|c| c.close as f64)
            .sum::<f64>()
            / trend as f64;
        let widths: Vec<f64> = (1..=lookback)
            .map(|offset| band(&candles[..n - offset], period, multiplier).map(|b| b.width))
            .collect::<Option<_>>()?;
        let baseline = widths.iter().sum::<f64>() / lookback as f64;
        let recent_min = widths.iter().take(5).copied().fold(f64::INFINITY, f64::min);
        let close = candles[n - 1].close as f64;
        Some(
            recent_min <= baseline * p.squeeze_ratio.clamp(0.1, 1.0)
                && current.width > previous.width
                && current.middle > previous.middle
                && close > long_ma
                && close > current.upper
                && candles[n - 2].close as f64 <= previous.upper,
        )
    }

    pub(super) fn bollinger_exit_reason(
        &self,
        symbol: &str,
        price: u64,
        entry: u64,
    ) -> Option<String> {
        let p = &self.params.bollinger;
        if !p.enabled {
            return None;
        }
        let candles = self.completed_bollinger_candles(symbol)?;
        let n = candles.len();
        if n >= 15 && p.atr_stop_multiplier.is_finite() {
            let tr = candles[n - 15..]
                .windows(2)
                .map(|pair| {
                    let (prev, cur) = (pair[0], pair[1]);
                    (cur.high.saturating_sub(cur.low) as f64)
                        .max((cur.high as f64 - prev.close as f64).abs())
                        .max((cur.low as f64 - prev.close as f64).abs())
                })
                .sum::<f64>()
                / 14.0;
            if tr > 0.0
                && price as f64 <= entry as f64 - tr * p.atr_stop_multiplier.clamp(0.5, 10.0)
            {
                return Some("LeveragedTrendHold 볼린저 보강 ATR 손절".into());
            }
        }
        let b = band(&candles, p.period.clamp(5, 100), 2.0)?;
        if (candles.last()?.close as f64) < b.middle {
            return Some("LeveragedTrendHold 볼린저 중심선 종가 이탈".into());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strategy() -> LeveragedTrendHoldStrategy {
        let mut config = StrategyConfig::new(
            "bb-test",
            "LeveragedTrendHold",
            true,
            vec!["SOXL".into()],
            1,
            serde_json::json!({}),
        );
        config.params = serde_json::json!({"bollinger": {"enabled": true}, "entries": [{"leveraged_symbol": "SOXL", "quantity": 1, "is_overseas": true}]});
        LeveragedTrendHoldStrategy::new(config)
    }
    fn candle(close: u64) -> OhlcCandle {
        OhlcCandle {
            open: close,
            high: close + 1,
            low: close - 1,
            close,
        }
    }

    #[test]
    fn flat_squeeze_needs_completed_upward_breakout() {
        let mut s = strategy();
        for _ in 0..65 {
            s.update_bollinger_candle("SOXL", candle(100), true);
        }
        assert!(!s.bollinger_entry_allowed("SOXL"));
        s.update_bollinger_candle("SOXL", candle(120), true);
        assert!(
            !s.bollinger_entry_allowed("SOXL"),
            "unclosed bar is excluded"
        );
        s.update_bollinger_candle("SOXL", candle(121), true);
        assert!(s.bollinger_entry_allowed("SOXL"));
        assert!(
            s.bollinger_exit_reason("SOXL", 121, 120).is_none(),
            "upper band walking is not an exit"
        );
    }

    #[test]
    fn downward_breakout_and_insufficient_history_block_entries() {
        let mut s = strategy();
        assert!(!s.bollinger_entry_allowed("SOXL"));
        for _ in 0..65 {
            s.update_bollinger_candle("SOXL", candle(100), true);
        }
        s.update_bollinger_candle("SOXL", candle(80), true);
        s.update_bollinger_candle("SOXL", candle(81), true);
        assert!(!s.bollinger_entry_allowed("SOXL"));
        assert!(s
            .bollinger_exit_reason("SOXL", 99, 100)
            .unwrap()
            .contains("중심선"));
        assert!(s
            .bollinger_exit_reason("SOXL", 70, 100)
            .unwrap()
            .contains("ATR"));
    }

    #[test]
    fn disabled_option_preserves_entries_and_buffer_is_bounded() {
        let mut s = strategy();
        s.params.bollinger.enabled = false;
        assert!(s.bollinger_entry_allowed("SOXL"));
        assert!(s.bollinger_exit_reason("SOXL", 1, 100).is_none());
        for _ in 0..1000 {
            s.update_bollinger_candle("SOXL", candle(100), true);
        }
        assert_eq!(s.bollinger_candles["SOXL"].len(), 512);
    }

    #[test]
    fn live_and_replay_share_confirmed_breakout_and_exit_without_future_prices() {
        let mut s = strategy();
        s.params.rapid_rebound_enabled = true;
        s.config.params = serde_json::to_value(&s.params).unwrap();
        s.last_params = s.config.params.clone();
        let params = s.params.clone();
        let prices = std::iter::repeat(100).take(65).chain([90, 120, 121, 80]);
        let timed: Vec<_> = prices
            .enumerate()
            .map(|(i, close)| LeveragedTrendHoldTimedCandle {
                time: format!("20260707{:02}{:02}00", 18 + i / 60, i % 60),
                candle: OhlcCandle {
                    open: close,
                    high: close,
                    low: close,
                    close,
                },
            })
            .collect();
        let replay =
            LeveragedTrendHoldStrategy::preview_signals("SOXL", params.clone(), &[], &timed);
        let live: Vec<_> = timed
            .iter()
            .filter_map(|t| match s.on_tick("SOXL", t.candle.close, 0) {
                Signal::Buy { reason, .. } => Some(("buy".to_string(), reason)),
                Signal::Sell { reason, .. } => Some(("sell".to_string(), reason)),
                Signal::Hold => None,
            })
            .collect();
        assert!(live.iter().any(|(side, _)| side == "buy"));
        assert!(live.iter().any(|(side, _)| side == "sell"));
        assert_eq!(
            live,
            replay
                .iter()
                .map(|s| (s.side.clone(), s.reason.clone()))
                .collect::<Vec<_>>()
        );
        let prefix = LeveragedTrendHoldStrategy::preview_signals("SOXL", params, &[], &timed[..68]);
        assert_eq!(
            prefix.iter().map(|s| &s.reason).collect::<Vec<_>>(),
            replay
                .iter()
                .take(prefix.len())
                .map(|s| &s.reason)
                .collect::<Vec<_>>()
        );
    }
}
