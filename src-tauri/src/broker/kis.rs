use std::sync::Arc;

use async_trait::async_trait;

use crate::api::rest::{
    BalanceItem, KisRestClient, OrderRequest, OrderSide as KisOrderSide, OrderType as KisOrderType,
};

use super::{
    adapter::{BrokerAdapter, BrokerAdapterError, BrokerAdapterResult},
    domain::{
        BrokerAccountId, BrokerHolding, BrokerId, BrokerMarket, BrokerMoney, BrokerOrderId,
        BrokerOrderReceipt, BrokerOrderRequest, BrokerOrderSide, BrokerOrderStatus,
        BrokerOrderType, BrokerPriceQuote, BrokerQuantity, BrokerSymbol,
    },
};

/// Adapter boundary for existing KIS REST calls.
pub struct KisBrokerAdapter {
    client: Arc<KisRestClient>,
}

impl KisBrokerAdapter {
    pub fn new(client: Arc<KisRestClient>) -> Self {
        Self { client }
    }

    fn map_balance_item(item: BalanceItem) -> BrokerHolding {
        BrokerHolding {
            broker: BrokerId::Kis,
            account_id: None,
            market: BrokerMarket::Kr,
            symbol: BrokerSymbol(item.pdno.clone()),
            symbol_name: item.prdt_name.clone(),
            quantity: BrokerQuantity(item.hldg_qty.clone()),
            average_price: BrokerMoney::krw(item.pchs_avg_pric.clone()),
            current_price: BrokerMoney::krw(item.prpr.clone()),
            unrealized_pnl: Some(BrokerMoney::krw(item.evlu_pfls_amt.clone())),
            raw: serde_json::to_value(item).unwrap_or_default(),
        }
    }
}

#[async_trait]
impl BrokerAdapter for KisBrokerAdapter {
    fn broker_id(&self) -> BrokerId {
        BrokerId::Kis
    }

    async fn get_price(&self, symbol: &BrokerSymbol) -> BrokerAdapterResult<BrokerPriceQuote> {
        let price = self.client.get_price(&symbol.0).await?;
        Ok(BrokerPriceQuote {
            broker: BrokerId::Kis,
            market: BrokerMarket::Kr,
            symbol: symbol.clone(),
            last: BrokerMoney::krw(price.stck_prpr.clone()),
            volume: Some(BrokerQuantity(price.acml_vol.clone())),
            raw: serde_json::to_value(price).unwrap_or_default(),
        })
    }

    async fn list_holdings(
        &self,
        _account_id: Option<&BrokerAccountId>,
    ) -> BrokerAdapterResult<Vec<BrokerHolding>> {
        let balance = self.client.get_balance().await?;
        Ok(balance
            .items
            .into_iter()
            .map(Self::map_balance_item)
            .collect())
    }

    async fn place_order(
        &self,
        _account_id: Option<&BrokerAccountId>,
        request: BrokerOrderRequest,
    ) -> BrokerAdapterResult<BrokerOrderReceipt> {
        if request.market != BrokerMarket::Kr {
            return Err(BrokerAdapterError::Unsupported {
                broker: BrokerId::Kis,
                operation: "place_order for non-KR market through KisBrokerAdapter",
            });
        }

        let quantity = request.quantity.parse_u64().ok_or_else(|| {
            BrokerAdapterError::InvalidRequest(format!(
                "KIS domestic quantity must be an integer: {}",
                request.quantity.0
            ))
        })?;
        let price = match (&request.order_type, &request.price) {
            (BrokerOrderType::Market, _) => 0,
            (BrokerOrderType::Limit, Some(money)) => money
                .amount
                .trim()
                .replace(',', "")
                .parse::<u64>()
                .map_err(|_| {
                    BrokerAdapterError::InvalidRequest(format!(
                        "KIS domestic limit price must be integer KRW: {}",
                        money.amount
                    ))
                })?,
            (BrokerOrderType::Limit, None) => {
                return Err(BrokerAdapterError::InvalidRequest(
                    "KIS limit order requires a price".to_string(),
                ));
            }
        };

        let response = self
            .client
            .place_order(&OrderRequest {
                symbol: request.symbol.0.clone(),
                side: match request.side {
                    BrokerOrderSide::Buy => KisOrderSide::Buy,
                    BrokerOrderSide::Sell => KisOrderSide::Sell,
                },
                order_type: match request.order_type {
                    BrokerOrderType::Limit => KisOrderType::Limit,
                    BrokerOrderType::Market => KisOrderType::Market,
                },
                quantity,
                price,
            })
            .await?;

        Ok(BrokerOrderReceipt {
            broker: BrokerId::Kis,
            order_id: BrokerOrderId(response.odno.clone()),
            client_order_id: request.client_order_id,
            status: BrokerOrderStatus::Pending,
            raw: serde_json::to_value(response).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_kis_balance_item_to_broker_holding() {
        let holding = KisBrokerAdapter::map_balance_item(BalanceItem {
            pdno: "005930".to_string(),
            prdt_name: "삼성전자".to_string(),
            hldg_qty: "10".to_string(),
            pchs_avg_pric: "70000".to_string(),
            prpr: "72000".to_string(),
            evlu_pfls_amt: "20000".to_string(),
            evlu_pfls_rt: "2.85".to_string(),
        });

        assert_eq!(holding.broker, BrokerId::Kis);
        assert_eq!(holding.account_id, None);
        assert_eq!(holding.market, BrokerMarket::Kr);
        assert_eq!(holding.symbol, BrokerSymbol("005930".to_string()));
        assert_eq!(holding.symbol_name, "삼성전자");
        assert_eq!(holding.quantity, BrokerQuantity("10".to_string()));
        assert_eq!(holding.average_price, BrokerMoney::krw("70000"));
        assert_eq!(holding.current_price, BrokerMoney::krw("72000"));
        assert_eq!(holding.unrealized_pnl, Some(BrokerMoney::krw("20000")));
        assert_eq!(holding.raw["pdno"], "005930");
    }
}
