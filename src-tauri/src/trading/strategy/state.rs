pub(crate) const MAX_STRATEGY_BUFFER: usize = 512;

pub(crate) fn bounded_window(period: usize) -> usize {
    period.clamp(1, MAX_STRATEGY_BUFFER)
}

pub(crate) fn bounded_window_with_extra(period: usize, extra: usize) -> usize {
    bounded_window(period)
        .saturating_add(extra)
        .min(MAX_STRATEGY_BUFFER.saturating_add(extra))
}
