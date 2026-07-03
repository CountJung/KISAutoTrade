use async_trait::async_trait;
use thiserror::Error;

use super::domain::{
    BrokerAccountId, BrokerCandle, BrokerHolding, BrokerId, BrokerOrderId, BrokerOrderReceipt,
    BrokerOrderRequest, BrokerPriceQuote, BrokerSymbol,
};

#[derive(Debug, Error)]
pub enum BrokerAdapterError {
    #[error("{broker:?} adapter does not support {operation}")]
    Unsupported {
        broker: BrokerId,
        operation: &'static str,
    },

    #[error("invalid broker request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    Provider(#[from] anyhow::Error),
}

pub type BrokerAdapterResult<T> = Result<T, BrokerAdapterError>;

#[async_trait]
pub trait BrokerAdapter: Send + Sync {
    fn broker_id(&self) -> BrokerId;

    async fn get_price(&self, symbol: &BrokerSymbol) -> BrokerAdapterResult<BrokerPriceQuote>;

    async fn get_candles(
        &self,
        _symbol: &BrokerSymbol,
        _period_code: &str,
        _from: &str,
        _to: &str,
    ) -> BrokerAdapterResult<Vec<BrokerCandle>> {
        Err(BrokerAdapterError::Unsupported {
            broker: self.broker_id(),
            operation: "get_candles",
        })
    }

    async fn list_holdings(
        &self,
        _account_id: Option<&BrokerAccountId>,
    ) -> BrokerAdapterResult<Vec<BrokerHolding>> {
        Err(BrokerAdapterError::Unsupported {
            broker: self.broker_id(),
            operation: "list_holdings",
        })
    }

    async fn place_order(
        &self,
        _account_id: Option<&BrokerAccountId>,
        _request: BrokerOrderRequest,
    ) -> BrokerAdapterResult<BrokerOrderReceipt> {
        Err(BrokerAdapterError::Unsupported {
            broker: self.broker_id(),
            operation: "place_order",
        })
    }

    async fn get_order(
        &self,
        _account_id: Option<&BrokerAccountId>,
        _order_id: &BrokerOrderId,
    ) -> BrokerAdapterResult<BrokerOrderReceipt> {
        Err(BrokerAdapterError::Unsupported {
            broker: self.broker_id(),
            operation: "get_order",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::{
        BrokerMarket, BrokerMoney, BrokerOrderSide, BrokerOrderType, BrokerQuantity,
        BrokerTimeInForce,
    };

    struct DummyAdapter;

    #[async_trait]
    impl BrokerAdapter for DummyAdapter {
        fn broker_id(&self) -> BrokerId {
            BrokerId::Toss
        }

        async fn get_price(&self, _symbol: &BrokerSymbol) -> BrokerAdapterResult<BrokerPriceQuote> {
            Err(BrokerAdapterError::Unsupported {
                broker: self.broker_id(),
                operation: "get_price",
            })
        }
    }

    #[tokio::test]
    async fn default_place_order_returns_unsupported_with_broker_id() {
        let adapter = DummyAdapter;
        let request = BrokerOrderRequest {
            market: BrokerMarket::Kr,
            symbol: BrokerSymbol("005930".to_string()),
            side: BrokerOrderSide::Buy,
            order_type: BrokerOrderType::Limit,
            quantity: BrokerQuantity("1".to_string()),
            price: Some(BrokerMoney::krw("70000")),
            time_in_force: BrokerTimeInForce::Day,
            client_order_id: None,
        };

        let err = adapter.place_order(None, request).await.unwrap_err();

        assert!(matches!(
            err,
            BrokerAdapterError::Unsupported {
                broker: BrokerId::Toss,
                operation: "place_order"
            }
        ));
    }
}
