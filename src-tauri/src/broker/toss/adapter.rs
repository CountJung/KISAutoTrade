use async_trait::async_trait;

use super::{
    client::TossOpenApiClient,
    orders::{
        TossOrder, TossOrderCreateRequest, TossOrderListQuery, TossOrderModifyRequest,
        TossOrderOperationResponse, TossOrderResponse, TossPaginatedOrderResponse,
    },
    types::{
        TossAccessToken, TossAccessTokenStatus, TossAccount, TossBuyingPower, TossCommission,
        TossExchangeRateResponse, TossKrMarketCalendarResponse, TossOpenApiOverview,
        TossOrderbookResponse, TossPriceLimitResponse, TossPriceResponse, TossSellableQuantity,
        TossStockInfo, TossStockWarning, TossTrade, TossUsMarketCalendarResponse,
    },
};
use crate::broker::{
    adapter::{BrokerAdapter, BrokerAdapterError, BrokerAdapterResult},
    domain::{
        BrokerAccountId, BrokerCandle, BrokerCurrency, BrokerHolding, BrokerId, BrokerPriceQuote,
        BrokerSymbol,
    },
};

/// Toss Open API adapter.
///
/// Read-only methods are implemented first. Order paths stay unsupported until idempotency,
/// rate-limit backoff, and live-account validation are wired into the trading guard.
pub struct TossBrokerAdapter {
    client: TossOpenApiClient,
}

impl TossBrokerAdapter {
    pub const DEFAULT_BASE_URL: &'static str = "https://openapi.tossinvest.com";

    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: TossOpenApiClient::without_credentials(base_url),
        }
    }

    pub fn with_credentials(
        base_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        account_seq: Option<impl Into<String>>,
    ) -> Self {
        Self {
            client: TossOpenApiClient::new(base_url, client_id, client_secret, account_seq),
        }
    }

    pub fn base_url(&self) -> &str {
        self.client.base_url()
    }

    pub async fn issue_token(&self) -> BrokerAdapterResult<TossAccessToken> {
        self.client
            .issue_token()
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn check_token(&self) -> BrokerAdapterResult<TossAccessTokenStatus> {
        self.client
            .get_access_token()
            .await
            .map(TossAccessTokenStatus::from)
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn openapi_overview(&self) -> BrokerAdapterResult<TossOpenApiOverview> {
        self.client
            .fetch_openapi_overview()
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn list_accounts(&self) -> BrokerAdapterResult<Vec<TossAccount>> {
        self.client
            .list_accounts()
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn list_prices(
        &self,
        symbols: &[BrokerSymbol],
    ) -> BrokerAdapterResult<Vec<TossPriceResponse>> {
        self.client
            .list_prices(symbols)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn list_stocks(
        &self,
        symbols: &[BrokerSymbol],
    ) -> BrokerAdapterResult<Vec<TossStockInfo>> {
        self.client
            .list_stocks(symbols)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn list_warnings(
        &self,
        symbol: &BrokerSymbol,
    ) -> BrokerAdapterResult<Vec<TossStockWarning>> {
        self.client
            .list_warnings(symbol)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn get_kr_market_calendar(
        &self,
        date: Option<&str>,
    ) -> BrokerAdapterResult<TossKrMarketCalendarResponse> {
        self.client
            .get_kr_market_calendar(date)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn get_us_market_calendar(
        &self,
        date: Option<&str>,
    ) -> BrokerAdapterResult<TossUsMarketCalendarResponse> {
        self.client
            .get_us_market_calendar(date)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn get_exchange_rate(
        &self,
        base_currency: BrokerCurrency,
        quote_currency: BrokerCurrency,
    ) -> BrokerAdapterResult<TossExchangeRateResponse> {
        self.client
            .get_exchange_rate(base_currency, quote_currency, None)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn get_orderbook(
        &self,
        symbol: &BrokerSymbol,
    ) -> BrokerAdapterResult<TossOrderbookResponse> {
        self.client
            .get_orderbook(symbol)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn list_trades(
        &self,
        symbol: &BrokerSymbol,
        count: Option<u8>,
    ) -> BrokerAdapterResult<Vec<TossTrade>> {
        self.client
            .list_trades(symbol, count)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn get_price_limits(
        &self,
        symbol: &BrokerSymbol,
    ) -> BrokerAdapterResult<TossPriceLimitResponse> {
        self.client
            .get_price_limits(symbol)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn get_buying_power(
        &self,
        account_seq: Option<&str>,
        currency: BrokerCurrency,
    ) -> BrokerAdapterResult<TossBuyingPower> {
        self.client
            .get_buying_power(account_seq, currency)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn get_sellable_quantity(
        &self,
        account_seq: Option<&str>,
        symbol: &BrokerSymbol,
    ) -> BrokerAdapterResult<TossSellableQuantity> {
        self.client
            .get_sellable_quantity(account_seq, symbol)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn list_commissions(
        &self,
        account_seq: Option<&str>,
    ) -> BrokerAdapterResult<Vec<TossCommission>> {
        self.client
            .list_commissions(account_seq)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn create_order(
        &self,
        account_seq: Option<&str>,
        input: &TossOrderCreateRequest,
    ) -> BrokerAdapterResult<TossOrderResponse> {
        self.client
            .create_order(account_seq, input)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn list_orders(
        &self,
        account_seq: Option<&str>,
        query: &TossOrderListQuery,
    ) -> BrokerAdapterResult<TossPaginatedOrderResponse> {
        self.client
            .list_orders(account_seq, query)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn get_order(
        &self,
        account_seq: Option<&str>,
        order_id: &str,
    ) -> BrokerAdapterResult<TossOrder> {
        self.client
            .get_order(account_seq, order_id)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn modify_order(
        &self,
        account_seq: Option<&str>,
        order_id: &str,
        input: &TossOrderModifyRequest,
    ) -> BrokerAdapterResult<TossOrderOperationResponse> {
        self.client
            .modify_order(account_seq, order_id, input)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub async fn cancel_order(
        &self,
        account_seq: Option<&str>,
        order_id: &str,
    ) -> BrokerAdapterResult<TossOrderOperationResponse> {
        self.client
            .cancel_order(account_seq, order_id)
            .await
            .map_err(BrokerAdapterError::Provider)
    }

    pub fn broker_id(&self) -> BrokerId {
        BrokerId::Toss
    }
}

impl Default for TossBrokerAdapter {
    fn default() -> Self {
        Self::new(Self::DEFAULT_BASE_URL)
    }
}

impl std::fmt::Debug for TossBrokerAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TossBrokerAdapter")
            .field("base_url", &self.base_url())
            .field("has_credentials", &self.client.has_credentials())
            .finish()
    }
}

impl std::fmt::Display for TossBrokerAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TossBrokerAdapter({})", self.base_url())
    }
}

#[async_trait]
impl BrokerAdapter for TossBrokerAdapter {
    fn broker_id(&self) -> BrokerId {
        BrokerId::Toss
    }

    async fn get_price(&self, symbol: &BrokerSymbol) -> BrokerAdapterResult<BrokerPriceQuote> {
        let prices = self
            .client
            .list_prices(std::slice::from_ref(symbol))
            .await
            .map_err(BrokerAdapterError::Provider)?;
        let price = prices
            .into_iter()
            .find(|item| item.symbol == symbol.0)
            .ok_or_else(|| {
                BrokerAdapterError::InvalidRequest(format!(
                    "Toss price response did not include requested symbol: {}",
                    symbol.0
                ))
            })?;

        Ok(price.to_broker_price_quote()?)
    }

    async fn get_candles(
        &self,
        symbol: &BrokerSymbol,
        period_code: &str,
        _from: &str,
        _to: &str,
    ) -> BrokerAdapterResult<Vec<BrokerCandle>> {
        let interval = match period_code {
            "1m" | "M1" | "m" => "1m",
            "1d" | "D" | "d" => "1d",
            other => {
                return Err(BrokerAdapterError::InvalidRequest(format!(
                    "Toss candles support only 1m or 1d interval: {other}"
                )));
            }
        };
        let page = self
            .client
            .get_candles(symbol, interval, Some(200), None, Some(true))
            .await
            .map_err(BrokerAdapterError::Provider)?;

        page.candles
            .iter()
            .map(|candle| candle.to_broker_candle(symbol))
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(BrokerAdapterError::Provider)
    }

    async fn list_holdings(
        &self,
        account_id: Option<&BrokerAccountId>,
    ) -> BrokerAdapterResult<Vec<BrokerHolding>> {
        let overview = self
            .client
            .list_holdings(account_id.map(|id| id.0.as_str()), None)
            .await
            .map_err(BrokerAdapterError::Provider)?;

        overview
            .items
            .iter()
            .map(|item| {
                item.to_broker_holding(account_id)
                    .map_err(BrokerAdapterError::Provider)
            })
            .collect()
    }
}
