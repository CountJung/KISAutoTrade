use serde::{Deserialize, Serialize};

/// Supported broker identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokerId {
    Kis,
    Toss,
}

/// Broker account identifier. KIS maps this from CANO/ACNT_PRDT_CD; Toss uses accountSeq.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BrokerAccountId(pub String);

/// Broker/account execution boundary used by risk and order guards.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerScope {
    pub broker_id: BrokerId,
    pub account_id: Option<BrokerAccountId>,
}

impl BrokerScope {
    pub fn new(broker_id: BrokerId, account_id: Option<BrokerAccountId>) -> Self {
        Self {
            broker_id,
            account_id,
        }
    }

    pub fn kis_legacy() -> Self {
        Self::new(BrokerId::Kis, None)
    }
}

/// Market scope used by broker adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokerMarket {
    Kr,
    Us,
}

/// Broker-neutral symbol wrapper. Preserve the broker's native code string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BrokerSymbol(pub String);

/// Currency values currently used by KIS and Toss stock APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BrokerCurrency {
    Krw,
    Usd,
}

/// Decimal money amount represented as a string to avoid precision loss.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerMoney {
    pub amount: String,
    pub currency: BrokerCurrency,
}

impl BrokerMoney {
    pub fn krw(amount: impl Into<String>) -> Self {
        Self {
            amount: amount.into(),
            currency: BrokerCurrency::Krw,
        }
    }

    pub fn usd(amount: impl Into<String>) -> Self {
        Self {
            amount: amount.into(),
            currency: BrokerCurrency::Usd,
        }
    }
}

/// Decimal share quantity represented as a string for broker parity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BrokerQuantity(pub String);

impl BrokerQuantity {
    pub fn from_u64(value: u64) -> Self {
        Self(value.to_string())
    }

    pub fn parse_u64(&self) -> Option<u64> {
        self.0.trim().replace(',', "").parse::<u64>().ok()
    }
}

/// Broker order identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BrokerOrderId(pub String);

/// Client-generated idempotency/order tracking identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BrokerClientOrderId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokerOrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokerOrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokerTimeInForce {
    Day,
    AtClose,
    AtOpen,
    Unknown,
}

/// Preserve unknown provider statuses while normalizing common lifecycle groups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokerOrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
    Failed,
    Replaced,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerPriceQuote {
    pub broker: BrokerId,
    pub market: BrokerMarket,
    pub symbol: BrokerSymbol,
    pub last: BrokerMoney,
    pub volume: Option<BrokerQuantity>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerCandle {
    pub symbol: BrokerSymbol,
    pub market: BrokerMarket,
    pub date: String,
    pub open: BrokerMoney,
    pub high: BrokerMoney,
    pub low: BrokerMoney,
    pub close: BrokerMoney,
    pub volume: BrokerQuantity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerHolding {
    pub broker: BrokerId,
    pub account_id: Option<BrokerAccountId>,
    pub market: BrokerMarket,
    pub symbol: BrokerSymbol,
    pub symbol_name: String,
    pub quantity: BrokerQuantity,
    pub average_price: BrokerMoney,
    pub current_price: BrokerMoney,
    pub unrealized_pnl: Option<BrokerMoney>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerOrderRequest {
    pub market: BrokerMarket,
    pub symbol: BrokerSymbol,
    pub side: BrokerOrderSide,
    pub order_type: BrokerOrderType,
    pub quantity: BrokerQuantity,
    pub price: Option<BrokerMoney>,
    pub time_in_force: BrokerTimeInForce,
    pub client_order_id: Option<BrokerClientOrderId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerOrderReceipt {
    pub broker: BrokerId,
    pub order_id: BrokerOrderId,
    pub client_order_id: Option<BrokerClientOrderId>,
    pub status: BrokerOrderStatus,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerWarning {
    pub symbol: BrokerSymbol,
    pub code: String,
    pub message: String,
    pub blocks_order: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_currency_as_uppercase() {
        let money = BrokerMoney::krw("72000");
        let json = serde_json::to_value(money).unwrap();
        assert_eq!(json["currency"], "KRW");
    }

    #[test]
    fn parses_integer_quantity_with_commas() {
        assert_eq!(BrokerQuantity("1,200".to_string()).parse_u64(), Some(1200));
        assert_eq!(BrokerQuantity("1.25".to_string()).parse_u64(), None);
    }

    #[test]
    fn broker_scope_serializes_with_camel_case_fields() {
        let scope = BrokerScope::new(BrokerId::Toss, Some(BrokerAccountId("123".to_string())));
        let json = serde_json::to_value(scope).unwrap();

        assert_eq!(json["brokerId"], "toss");
        assert_eq!(json["accountId"], "123");
    }
}
