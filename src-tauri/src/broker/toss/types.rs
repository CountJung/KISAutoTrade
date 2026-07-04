use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::support::{market_from_currency, toss_currency, toss_market};
use crate::broker::{
    adapter::{BrokerAdapterError, BrokerAdapterResult},
    domain::{
        BrokerAccountId, BrokerCandle, BrokerHolding, BrokerId, BrokerMoney, BrokerPriceQuote,
        BrokerQuantity, BrokerSymbol,
    },
};

pub(super) struct TossCredentials {
    pub(super) client_id: String,
    pub(super) client_secret: String,
    pub(super) account_seq: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct TossTokenRequest<'a> {
    pub(super) grant_type: &'static str,
    pub(super) client_id: &'a str,
    pub(super) client_secret: &'a str,
}

#[derive(Debug, Deserialize)]
pub(super) struct TossTokenResponse {
    pub(super) access_token: String,
    pub(super) token_type: String,
    pub(super) expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossAccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
}

impl TossAccessToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at - Duration::minutes(5)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossAccessTokenStatus {
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
}

impl From<TossAccessToken> for TossAccessTokenStatus {
    fn from(token: TossAccessToken) -> Self {
        Self {
            token_type: token.token_type,
            expires_at: token.expires_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOpenApiOverview {
    pub title: String,
    pub version: String,
    pub server: String,
    pub paths_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TossApiResponse<T> {
    pub(super) result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossAccount {
    pub account_no: String,
    pub account_seq: i64,
    pub account_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossPriceResponse {
    pub symbol: String,
    pub timestamp: Option<String>,
    pub last_price: String,
    pub currency: String,
}

impl TossPriceResponse {
    pub fn to_broker_price_quote(&self) -> BrokerAdapterResult<BrokerPriceQuote> {
        let currency = toss_currency(&self.currency).map_err(BrokerAdapterError::Provider)?;
        Ok(BrokerPriceQuote {
            broker: BrokerId::Toss,
            market: market_from_currency(currency),
            symbol: BrokerSymbol(self.symbol.clone()),
            last: BrokerMoney {
                amount: self.last_price.clone(),
                currency,
            },
            volume: None,
            raw: serde_json::to_value(self).unwrap_or(serde_json::Value::Null),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossExchangeRateResponse {
    pub base_currency: String,
    pub quote_currency: String,
    pub rate: String,
    pub mid_rate: String,
    pub basis_point: String,
    pub rate_change_type: String,
    pub valid_from: String,
    pub valid_until: String,
}

impl TossExchangeRateResponse {
    pub fn rate_as_f64(&self) -> anyhow::Result<f64> {
        self.rate
            .parse::<f64>()
            .with_context(|| format!("토스증권 exchange-rate rate 파싱 실패: {}", self.rate))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossStockInfo {
    pub symbol: String,
    pub name: String,
    pub english_name: String,
    pub isin_code: String,
    pub market: String,
    pub security_type: String,
    pub is_common_share: bool,
    pub status: String,
    pub currency: String,
    pub list_date: Option<String>,
    pub delist_date: Option<String>,
    pub shares_outstanding: String,
    pub leverage_factor: Option<String>,
    pub korean_market_detail: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossStockWarning {
    pub warning_type: String,
    pub exchange: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl TossStockWarning {
    pub fn is_blocking_for_buy(&self) -> bool {
        matches!(
            self.warning_type.as_str(),
            "LIQUIDATION_TRADING" | "INVESTMENT_RISK" | "STOCK_WARRANTS"
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketSession {
    pub start_time: String,
    pub end_time: String,
    pub single_price_auction_start_time: Option<String>,
    pub single_price_auction_end_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossKrIntegratedHour {
    pub pre_market: Option<TossMarketSession>,
    pub regular_market: Option<TossMarketSession>,
    pub after_market: Option<TossMarketSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossKrMarketDay {
    pub date: String,
    pub integrated: Option<TossKrIntegratedHour>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossKrMarketCalendarResponse {
    pub today: TossKrMarketDay,
    pub previous_business_day: TossKrMarketDay,
    pub next_business_day: TossKrMarketDay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossUsMarketDay {
    pub date: String,
    pub day_market: Option<TossMarketSession>,
    pub pre_market: Option<TossMarketSession>,
    pub regular_market: Option<TossMarketSession>,
    pub after_market: Option<TossMarketSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossUsMarketCalendarResponse {
    pub today: TossUsMarketDay,
    pub previous_business_day: TossUsMarketDay,
    pub next_business_day: TossUsMarketDay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderbookEntry {
    pub price: String,
    pub volume: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderbookResponse {
    pub timestamp: Option<String>,
    pub currency: String,
    pub asks: Vec<TossOrderbookEntry>,
    pub bids: Vec<TossOrderbookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossTrade {
    pub price: String,
    pub volume: String,
    pub timestamp: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossPriceLimitResponse {
    pub timestamp: String,
    pub upper_limit_price: Option<String>,
    pub lower_limit_price: Option<String>,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossCandlePageResponse {
    pub candles: Vec<TossCandle>,
    pub next_before: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossCandle {
    pub timestamp: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub close_price: String,
    pub volume: String,
    pub currency: String,
}

impl TossCandle {
    pub fn to_broker_candle(&self, symbol: &BrokerSymbol) -> anyhow::Result<BrokerCandle> {
        let currency = toss_currency(&self.currency)?;
        Ok(BrokerCandle {
            symbol: symbol.clone(),
            market: market_from_currency(currency),
            date: self.timestamp.clone(),
            open: BrokerMoney {
                amount: self.open_price.clone(),
                currency,
            },
            high: BrokerMoney {
                amount: self.high_price.clone(),
                currency,
            },
            low: BrokerMoney {
                amount: self.low_price.clone(),
                currency,
            },
            close: BrokerMoney {
                amount: self.close_price.clone(),
                currency,
            },
            volume: BrokerQuantity(self.volume.clone()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossHoldingsOverview {
    pub total_purchase_amount: TossCurrencyAmount,
    pub market_value: TossOverviewMarketValue,
    pub profit_loss: TossOverviewProfitLoss,
    pub daily_profit_loss: TossOverviewDailyProfitLoss,
    pub items: Vec<TossHoldingsItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossHoldingsItem {
    pub symbol: String,
    pub name: String,
    pub market_country: String,
    pub currency: String,
    pub quantity: String,
    pub last_price: String,
    pub average_purchase_price: String,
    pub market_value: TossMarketValue,
    pub profit_loss: TossProfitLoss,
    pub daily_profit_loss: TossDailyProfitLoss,
    pub cost: TossCost,
}

impl TossHoldingsItem {
    pub(super) fn to_broker_holding(
        &self,
        account_id: Option<&BrokerAccountId>,
    ) -> anyhow::Result<BrokerHolding> {
        let currency = toss_currency(&self.currency)?;
        Ok(BrokerHolding {
            broker: BrokerId::Toss,
            account_id: account_id.cloned(),
            market: toss_market(&self.market_country)?,
            symbol: BrokerSymbol(self.symbol.clone()),
            symbol_name: self.name.clone(),
            quantity: BrokerQuantity(self.quantity.clone()),
            average_price: BrokerMoney {
                amount: self.average_purchase_price.clone(),
                currency,
            },
            current_price: BrokerMoney {
                amount: self.last_price.clone(),
                currency,
            },
            unrealized_pnl: Some(BrokerMoney {
                amount: self.profit_loss.amount.clone(),
                currency,
            }),
            raw: serde_json::to_value(self).unwrap_or(serde_json::Value::Null),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossCurrencyAmount {
    pub krw: String,
    pub usd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOverviewMarketValue {
    pub amount: TossCurrencyAmount,
    pub amount_after_cost: TossCurrencyAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOverviewProfitLoss {
    pub amount: TossCurrencyAmount,
    pub amount_after_cost: TossCurrencyAmount,
    pub rate: String,
    pub rate_after_cost: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOverviewDailyProfitLoss {
    pub amount: TossCurrencyAmount,
    pub rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketValue {
    pub purchase_amount: String,
    pub amount: String,
    pub amount_after_cost: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossProfitLoss {
    pub amount: String,
    pub amount_after_cost: String,
    pub rate: String,
    pub rate_after_cost: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossDailyProfitLoss {
    pub amount: String,
    pub rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossCost {
    pub commission: String,
    pub tax: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossBuyingPower {
    pub currency: String,
    pub cash_buying_power: String,
}

impl TossBuyingPower {
    pub fn money(&self) -> anyhow::Result<BrokerMoney> {
        Ok(BrokerMoney {
            amount: self.cash_buying_power.clone(),
            currency: toss_currency(&self.currency)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossSellableQuantity {
    pub sellable_quantity: String,
}

impl TossSellableQuantity {
    pub fn quantity(&self) -> BrokerQuantity {
        BrokerQuantity(self.sellable_quantity.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossCommission {
    pub market_country: String,
    pub commission_rate: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
