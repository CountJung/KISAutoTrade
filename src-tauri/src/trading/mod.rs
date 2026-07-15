pub mod guard;
pub mod order;
pub mod position;
pub mod preflight;
pub mod risk;
pub mod simulation;
pub mod strategy;
pub mod views;

pub use guard::{GuardDecision, TradeGuard};
pub use order::{OrderManager, PendingOrder};
pub use position::PositionTracker;
pub use preflight::{evaluate_order_preflight, OrderPreflightConstraints, OrderPreflightInput};
pub use risk::RiskManager;
pub use strategy::{
    LeveragedTrendHoldParams, LeveragedTrendHoldStrategy, MovingAverageCrossStrategy,
    PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, Signal, Strategy,
    StrategyConfig, StrategyManager,
};
pub use views::{build_strategy_view, StrategyView};
