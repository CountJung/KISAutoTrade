//! Pure calculations over already validated, completed DAILY OHLC.
use super::OhlcCandle;
use std::collections::VecDeque;

pub(super) fn valid_ohlc(c: &OhlcCandle) -> bool {
    c.low > 0 && c.low <= c.open.min(c.close) && c.high >= c.open.max(c.close)
}

pub(super) fn mean_close(bars: &VecDeque<OhlcCandle>, period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    Some(
        bars.iter()
            .rev()
            .take(period)
            .map(|c| c.close as u128)
            .sum::<u128>() as f64
            / period as f64,
    )
}

/// Arithmetic mean of 14 true ranges; zero is a valid observed value.
pub(super) fn mean_true_range14(bars: &VecDeque<OhlcCandle>) -> Option<f64> {
    if bars.len() < 15 {
        return None;
    }
    let bars = bars.iter().skip(bars.len() - 15).collect::<Vec<_>>();
    let total = bars
        .windows(2)
        .map(|w| {
            let (p, c) = (w[0], w[1]);
            (c.high - c.low)
                .max(c.high.abs_diff(p.close))
                .max(c.low.abs_diff(p.close)) as f64
        })
        .sum::<f64>();
    let atr = total / 14.0;
    Some(atr)
}

/// Preserve the existing positive-only contract for MA and Consecutive.
pub(super) fn atr14(bars: &VecDeque<OhlcCandle>) -> Option<f64> {
    mean_true_range14(bars).filter(|atr| *atr > 0.0)
}
