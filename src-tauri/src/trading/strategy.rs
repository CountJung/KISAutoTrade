mod breakout;
mod classic;
mod core;
mod leveraged_trend_hold;
mod ma_cross;
mod manager;
mod mean_trend;
mod price_condition;
mod state;

pub use breakout::{
    ConsecutiveMoveParams, ConsecutiveMoveStrategy, FailedBreakoutParams, FailedBreakoutStrategy,
    FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy, StrongCloseParams, StrongCloseStrategy,
    VolatilityExpansionParams, VolatilityExpansionStrategy,
};
pub use classic::{
    DeviationParams, DeviationStrategy, MomentumParams, MomentumStrategy, RsiParams, RsiStrategy,
};
pub use core::{
    BrokerPositionSnapshot, OhlcCandle, Signal, Strategy, StrategyConfig, StrategySignal,
};
pub use leveraged_trend_hold::{
    LeveragedTrendHoldEntry, LeveragedTrendHoldParams, LeveragedTrendHoldPreviewSignal,
    LeveragedTrendHoldStrategy, LeveragedTrendHoldTimedCandle,
};
pub use ma_cross::{MaCrossParams, MovingAverageCrossStrategy};
pub use manager::{build_strategy, StrategyManager};
pub use mean_trend::{
    MeanReversionParams, MeanReversionStrategy, TrendFilterParams, TrendFilterStrategy,
};
pub use price_condition::{
    PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig,
};

#[cfg(test)]
mod tests;
