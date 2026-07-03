//! Broker-neutral domain boundary for gradually moving KIS-only code behind adapters.

pub mod adapter;
pub mod domain;
pub mod kis;
pub mod rate_limit;
pub mod toss;

pub use adapter::{BrokerAdapter, BrokerAdapterError, BrokerAdapterResult};
pub use domain::{
    BrokerAccountId, BrokerCandle, BrokerClientOrderId, BrokerCurrency, BrokerHolding, BrokerId,
    BrokerMarket, BrokerMoney, BrokerOrderId, BrokerOrderReceipt, BrokerOrderRequest,
    BrokerOrderSide, BrokerOrderStatus, BrokerOrderType, BrokerPriceQuote, BrokerQuantity,
    BrokerScope, BrokerSymbol, BrokerTimeInForce, BrokerWarning,
};
pub use kis::KisBrokerAdapter;
pub use rate_limit::RateLimitScheduler;
pub use toss::TossBrokerAdapter;
