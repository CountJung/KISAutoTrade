mod breakout;
mod classic;
mod completed_daily;
mod core;
mod leveraged_trend_hold;
mod ma_cross;
mod manager;
mod mean_trend;
mod observation;
mod price_condition;
mod sequence;
mod state;

pub use breakout::{
    validate_strong_close_params, FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy,
    StrongCloseParams, StrongCloseStrategy, VolatilityExpansionParams, VolatilityExpansionStrategy,
};
pub use classic::{
    validate_deviation_entry_params, validate_momentum_entry_params, validate_rsi_params,
    DeviationEntryMode, DeviationParams, DeviationStrategy, MomentumEntryMode, MomentumParams,
    MomentumStrategy, RsiParams, RsiStrategy,
};
pub use core::{
    initialize_strategy_warmup, initialize_strategy_warmup_with_daily_bars,
    parse_completed_daily_volume, BrokerPositionSnapshot, CompletedDailyBar,
    EntryFilterPreparation, HistoryReadiness, OhlcCandle, Signal, Strategy, StrategyConfig,
    StrategySignal,
};
pub use leveraged_trend_hold::{
    LeveragedTrendHoldEntry, LeveragedTrendHoldParams, LeveragedTrendHoldPreviewSignal,
    LeveragedTrendHoldPreviewStep, LeveragedTrendHoldStrategy, LeveragedTrendHoldTimedCandle,
};
pub use ma_cross::{validate_ma_cross_entry_params, MaCrossParams, MovingAverageCrossStrategy};
pub use manager::{build_strategy, DailyHistoryStatus, DailyHistoryTicket, StrategyManager};
pub use mean_trend::{
    validate_mean_reversion_params, MeanReversionExitMode, MeanReversionParams,
    MeanReversionStrategy, TrendFilterParams, TrendFilterStrategy,
};
pub use price_condition::{
    PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig,
};
pub use sequence::{
    validate_consecutive_entry_params, validate_failed_breakout_params, ConsecutiveMoveParams,
    ConsecutiveMoveStrategy, FailedBreakoutBasis, FailedBreakoutParams, FailedBreakoutStrategy,
};

#[cfg(test)]
mod tests;
