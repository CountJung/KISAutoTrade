use std::time::Duration as StdDuration;

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::{header::HeaderMap, Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::{
    adapter::{BrokerAdapter, BrokerAdapterError, BrokerAdapterResult},
    domain::{
        BrokerAccountId, BrokerCurrency, BrokerHolding, BrokerId, BrokerMarket, BrokerMoney,
        BrokerPriceQuote, BrokerQuantity, BrokerSymbol,
    },
    rate_limit::RateLimitScheduler,
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
    ) -> BrokerAdapterResult<Vec<super::domain::BrokerCandle>> {
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

#[derive(Debug)]
pub struct TossOpenApiClient {
    http: Client,
    base_url: String,
    credentials: Option<TossCredentials>,
    current_token: Mutex<Option<TossAccessToken>>,
    rate_limiter: RateLimitScheduler,
}

impl TossOpenApiClient {
    pub fn without_credentials(base_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: trim_base_url(base_url.into()),
            credentials: None,
            current_token: Mutex::new(None),
            rate_limiter: toss_rate_limiter(),
        }
    }

    pub fn new(
        base_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        account_seq: Option<impl Into<String>>,
    ) -> Self {
        Self {
            http: Client::new(),
            base_url: trim_base_url(base_url.into()),
            credentials: Some(TossCredentials {
                client_id: client_id.into(),
                client_secret: client_secret.into(),
                account_seq: account_seq.map(Into::into),
            }),
            current_token: Mutex::new(None),
            rate_limiter: toss_rate_limiter(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }

    pub async fn get_token(&self) -> anyhow::Result<String> {
        self.get_access_token()
            .await
            .map(|token| token.access_token.clone())
    }

    pub async fn get_access_token(&self) -> anyhow::Result<TossAccessToken> {
        let mut token_guard = self.current_token.lock().await;

        if let Some(token) = &*token_guard {
            if !token.is_expired() {
                return Ok(token.clone());
            }
        }

        let token = self.issue_token().await?;
        *token_guard = Some(token.clone());
        Ok(token)
    }

    pub async fn issue_token(&self) -> anyhow::Result<TossAccessToken> {
        let credentials = self.credentials()?;
        let url = format!("{}/oauth2/token", self.base_url);
        let rate_group = "toss:auth";
        let body = TossTokenRequest {
            grant_type: "client_credentials",
            client_id: &credentials.client_id,
            client_secret: &credentials.client_secret,
        };

        self.rate_limiter.wait(rate_group).await;
        let resp = self
            .http
            .post(url)
            .form(&body)
            .send()
            .await
            .context("토스증권 토큰 발급 요청 실패")?;

        let status = resp.status();
        let headers = resp.headers().clone();
        self.rate_limiter
            .apply_response_headers(rate_group, &headers)
            .await;
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format_toss_error(status, &headers, &text, "토큰 발급 실패"));
        }

        let body: TossTokenResponse = serde_json::from_str(&text)
            .with_context(|| format!("토스증권 토큰 응답 파싱 실패: body={text}"))?;
        Ok(TossAccessToken {
            access_token: body.access_token,
            token_type: body.token_type,
            expires_at: Utc::now() + Duration::seconds(body.expires_in),
        })
    }

    pub async fn list_accounts(&self) -> anyhow::Result<Vec<TossAccount>> {
        self.get_json::<TossApiResponse<Vec<TossAccount>>>("/api/v1/accounts", None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn list_prices(
        &self,
        symbols: &[BrokerSymbol],
    ) -> anyhow::Result<Vec<TossPriceResponse>> {
        if symbols.is_empty() {
            return Err(anyhow!(
                "토스증권 현재가 조회에는 symbol이 최소 1개 필요합니다"
            ));
        }
        if symbols.len() > 200 {
            return Err(anyhow!(
                "토스증권 현재가 조회는 최대 200개 symbol만 지원합니다: {}",
                symbols.len()
            ));
        }
        let joined = symbols
            .iter()
            .map(|symbol| symbol.0.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let path = format!("/api/v1/prices?symbols={}", url_encode(&joined));
        self.get_json::<TossApiResponse<Vec<TossPriceResponse>>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn list_stocks(
        &self,
        symbols: &[BrokerSymbol],
    ) -> anyhow::Result<Vec<TossStockInfo>> {
        if symbols.is_empty() {
            return Err(anyhow!(
                "토스증권 종목 기본 정보 조회에는 symbol이 최소 1개 필요합니다"
            ));
        }
        if symbols.len() > 200 {
            return Err(anyhow!(
                "토스증권 종목 기본 정보 조회는 최대 200개 symbol만 지원합니다: {}",
                symbols.len()
            ));
        }
        for symbol in symbols {
            validate_toss_symbol(&symbol.0)?;
        }
        let joined = symbols
            .iter()
            .map(|symbol| symbol.0.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let path = format!("/api/v1/stocks?symbols={}", url_encode(&joined));
        self.get_json::<TossApiResponse<Vec<TossStockInfo>>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn list_warnings(
        &self,
        symbol: &BrokerSymbol,
    ) -> anyhow::Result<Vec<TossStockWarning>> {
        validate_toss_symbol(&symbol.0)?;
        let path = format!("/api/v1/stocks/{}/warnings", url_encode(&symbol.0));
        self.get_json::<TossApiResponse<Vec<TossStockWarning>>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn get_kr_market_calendar(
        &self,
        date: Option<&str>,
    ) -> anyhow::Result<TossKrMarketCalendarResponse> {
        let path = match date {
            Some(date) => {
                validate_iso_date(date)?;
                format!("/api/v1/market-calendar/KR?date={}", url_encode(date))
            }
            None => "/api/v1/market-calendar/KR".to_string(),
        };
        self.get_json::<TossApiResponse<TossKrMarketCalendarResponse>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn get_us_market_calendar(
        &self,
        date: Option<&str>,
    ) -> anyhow::Result<TossUsMarketCalendarResponse> {
        let path = match date {
            Some(date) => {
                validate_iso_date(date)?;
                format!("/api/v1/market-calendar/US?date={}", url_encode(date))
            }
            None => "/api/v1/market-calendar/US".to_string(),
        };
        self.get_json::<TossApiResponse<TossUsMarketCalendarResponse>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn get_exchange_rate(
        &self,
        base_currency: BrokerCurrency,
        quote_currency: BrokerCurrency,
        date_time: Option<&str>,
    ) -> anyhow::Result<TossExchangeRateResponse> {
        if base_currency == quote_currency {
            return Err(anyhow!(
                "토스증권 exchange-rate는 기준 통화와 표시 통화가 달라야 합니다"
            ));
        }
        let mut path = format!(
            "/api/v1/exchange-rate?baseCurrency={}&quoteCurrency={}",
            toss_currency_code(base_currency),
            toss_currency_code(quote_currency)
        );
        if let Some(date_time) = date_time {
            path.push_str("&dateTime=");
            path.push_str(&url_encode(date_time));
        }
        self.get_json::<TossApiResponse<TossExchangeRateResponse>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn get_orderbook(
        &self,
        symbol: &BrokerSymbol,
    ) -> anyhow::Result<TossOrderbookResponse> {
        validate_toss_symbol(&symbol.0)?;
        let path = format!("/api/v1/orderbook?symbol={}", url_encode(&symbol.0));
        self.get_json::<TossApiResponse<TossOrderbookResponse>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn list_trades(
        &self,
        symbol: &BrokerSymbol,
        count: Option<u8>,
    ) -> anyhow::Result<Vec<TossTrade>> {
        let count = count.unwrap_or(50);
        if !(1..=50).contains(&count) {
            return Err(anyhow!(
                "토스증권 최근 체결 조회 count는 1~50 범위여야 합니다: {count}"
            ));
        }
        let path = format!(
            "/api/v1/trades?symbol={}&count={}",
            url_encode(&symbol.0),
            count
        );
        self.get_json::<TossApiResponse<Vec<TossTrade>>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn get_price_limits(
        &self,
        symbol: &BrokerSymbol,
    ) -> anyhow::Result<TossPriceLimitResponse> {
        validate_toss_symbol(&symbol.0)?;
        let path = format!("/api/v1/price-limits?symbol={}", url_encode(&symbol.0));
        self.get_json::<TossApiResponse<TossPriceLimitResponse>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn get_candles(
        &self,
        symbol: &BrokerSymbol,
        interval: &str,
        count: Option<u16>,
        before: Option<&str>,
        adjusted: Option<bool>,
    ) -> anyhow::Result<TossCandlePageResponse> {
        if !matches!(interval, "1m" | "1d") {
            return Err(anyhow!(
                "토스증권 candle interval은 1m 또는 1d만 지원합니다: {interval}"
            ));
        }
        let count = count.unwrap_or(100);
        if !(1..=200).contains(&count) {
            return Err(anyhow!(
                "토스증권 candle count는 1~200 범위여야 합니다: {count}"
            ));
        }
        let mut path = format!(
            "/api/v1/candles?symbol={}&interval={}&count={}",
            url_encode(&symbol.0),
            interval,
            count
        );
        if let Some(before) = before {
            path.push_str("&before=");
            path.push_str(&url_encode(before));
        }
        if let Some(adjusted) = adjusted {
            path.push_str("&adjusted=");
            path.push_str(if adjusted { "true" } else { "false" });
        }
        self.get_json::<TossApiResponse<TossCandlePageResponse>>(&path, None, None)
            .await
            .map(|response| response.result)
    }

    pub async fn fetch_openapi_overview(&self) -> anyhow::Result<TossOpenApiOverview> {
        let url = format!("{}/openapi-docs/latest/openapi.json", self.base_url);
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .context("토스증권 OpenAPI JSON 조회 실패")?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!(
                "토스증권 OpenAPI JSON 조회 실패: HTTP {status}; body={text}"
            ));
        }

        let spec: serde_json::Value = serde_json::from_str(&text)
            .with_context(|| format!("토스증권 OpenAPI JSON 파싱 실패: body={text}"))?;
        let title = spec
            .pointer("/info/title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let version = spec
            .pointer("/info/version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let server = spec
            .pointer("/servers/0/url")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let paths_count = spec
            .get("paths")
            .and_then(serde_json::Value::as_object)
            .map_or(0, serde_json::Map::len);

        Ok(TossOpenApiOverview {
            title,
            version,
            server,
            paths_count,
        })
    }

    pub async fn list_holdings(
        &self,
        account_seq: Option<&str>,
        symbol: Option<&str>,
    ) -> anyhow::Result<TossHoldingsOverview> {
        let account_seq = account_seq
            .map(str::to_string)
            .or_else(|| {
                self.credentials
                    .as_ref()
                    .and_then(|c| c.account_seq.clone())
            })
            .ok_or_else(|| anyhow!("토스증권 holdings 조회에는 accountSeq가 필요합니다"))?;

        let path = match symbol {
            Some(symbol) => format!("/api/v1/holdings?symbol={}", url_encode(symbol)),
            None => "/api/v1/holdings".to_string(),
        };

        self.get_json::<TossApiResponse<TossHoldingsOverview>>(&path, Some(&account_seq), None)
            .await
            .map(|response| response.result)
    }

    pub async fn get_buying_power(
        &self,
        account_seq: Option<&str>,
        currency: BrokerCurrency,
    ) -> anyhow::Result<TossBuyingPower> {
        let account_seq = self.require_account_seq(account_seq)?;
        let path = format!(
            "/api/v1/buying-power?currency={}",
            toss_currency_code(currency)
        );
        self.get_json::<TossApiResponse<TossBuyingPower>>(&path, Some(&account_seq), None)
            .await
            .map(|response| response.result)
    }

    pub async fn get_sellable_quantity(
        &self,
        account_seq: Option<&str>,
        symbol: &BrokerSymbol,
    ) -> anyhow::Result<TossSellableQuantity> {
        let account_seq = self.require_account_seq(account_seq)?;
        let path = format!("/api/v1/sellable-quantity?symbol={}", url_encode(&symbol.0));
        self.get_json::<TossApiResponse<TossSellableQuantity>>(&path, Some(&account_seq), None)
            .await
            .map(|response| response.result)
    }

    pub async fn list_commissions(
        &self,
        account_seq: Option<&str>,
    ) -> anyhow::Result<Vec<TossCommission>> {
        let account_seq = self.require_account_seq(account_seq)?;
        self.get_json::<TossApiResponse<Vec<TossCommission>>>(
            "/api/v1/commissions",
            Some(&account_seq),
            None,
        )
        .await
        .map(|response| response.result)
    }

    pub async fn create_order(
        &self,
        account_seq: Option<&str>,
        input: &TossOrderCreateRequest,
    ) -> anyhow::Result<TossOrderResponse> {
        input.validate()?;
        let account_seq = self.require_account_seq(account_seq)?;
        self.post_json::<TossApiResponse<TossOrderResponse>, _>(
            "/api/v1/orders",
            Some(&account_seq),
            input,
            None,
        )
        .await
        .map(|response| response.result)
    }

    pub async fn list_orders(
        &self,
        account_seq: Option<&str>,
        query: &TossOrderListQuery,
    ) -> anyhow::Result<TossPaginatedOrderResponse> {
        query.validate()?;
        let account_seq = self.require_account_seq(account_seq)?;
        let path = query.to_path();
        self.get_json::<TossApiResponse<TossPaginatedOrderResponse>>(
            &path,
            Some(&account_seq),
            None,
        )
        .await
        .map(|response| response.result)
    }

    pub async fn get_order(
        &self,
        account_seq: Option<&str>,
        order_id: &str,
    ) -> anyhow::Result<TossOrder> {
        validate_toss_order_id(order_id)?;
        let account_seq = self.require_account_seq(account_seq)?;
        let path = format!("/api/v1/orders/{}", url_encode(order_id));
        self.get_json::<TossApiResponse<TossOrder>>(&path, Some(&account_seq), None)
            .await
            .map(|response| response.result)
    }

    pub async fn modify_order(
        &self,
        account_seq: Option<&str>,
        order_id: &str,
        input: &TossOrderModifyRequest,
    ) -> anyhow::Result<TossOrderOperationResponse> {
        validate_toss_order_id(order_id)?;
        input.validate()?;
        let account_seq = self.require_account_seq(account_seq)?;
        let path = format!("/api/v1/orders/{}/modify", url_encode(order_id));
        self.post_json::<TossApiResponse<TossOrderOperationResponse>, _>(
            &path,
            Some(&account_seq),
            input,
            None,
        )
        .await
        .map(|response| response.result)
    }

    pub async fn cancel_order(
        &self,
        account_seq: Option<&str>,
        order_id: &str,
    ) -> anyhow::Result<TossOrderOperationResponse> {
        validate_toss_order_id(order_id)?;
        let account_seq = self.require_account_seq(account_seq)?;
        let path = format!("/api/v1/orders/{}/cancel", url_encode(order_id));
        self.post_json::<TossApiResponse<TossOrderOperationResponse>, _>(
            &path,
            Some(&account_seq),
            &serde_json::json!({}),
            None,
        )
        .await
        .map(|response| response.result)
    }

    async fn get_json<T>(
        &self,
        path: &str,
        account_seq: Option<&str>,
        retry_token: Option<String>,
    ) -> anyhow::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let had_retry_token = retry_token.is_some();
        let token = match retry_token {
            Some(token) => token,
            None => self.get_token().await?,
        };
        let url = format!("{}{}", self.base_url, path);
        let rate_group = toss_rate_limit_group("GET", path);
        let mut request = self.http.get(url).bearer_auth(token);
        if let Some(account_seq) = account_seq {
            request = request.header("X-Tossinvest-Account", account_seq);
        }

        self.rate_limiter.wait(rate_group).await;
        let resp = request.send().await.context("토스증권 OpenAPI 요청 실패")?;
        let status = resp.status();
        let headers = resp.headers().clone();
        self.rate_limiter
            .apply_response_headers(rate_group, &headers)
            .await;
        let text = resp.text().await.unwrap_or_default();

        if status == StatusCode::UNAUTHORIZED && !had_retry_token {
            *self.current_token.lock().await = None;
            let new_token = self.get_token().await?;
            return Box::pin(self.get_json(path, account_seq, Some(new_token))).await;
        }

        if !status.is_success() {
            return Err(format_toss_error(
                status,
                &headers,
                &text,
                "OpenAPI 요청 실패",
            ));
        }

        serde_json::from_str(&text)
            .with_context(|| format!("토스증권 OpenAPI 응답 파싱 실패: body={text}"))
    }

    async fn post_json<T, B>(
        &self,
        path: &str,
        account_seq: Option<&str>,
        body: &B,
        retry_token: Option<String>,
    ) -> anyhow::Result<T>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize + Sync,
    {
        let had_retry_token = retry_token.is_some();
        let token = match retry_token {
            Some(token) => token,
            None => self.get_token().await?,
        };
        let url = format!("{}{}", self.base_url, path);
        let rate_group = toss_rate_limit_group("POST", path);
        let mut request = self.http.post(url).bearer_auth(token).json(body);
        if let Some(account_seq) = account_seq {
            request = request.header("X-Tossinvest-Account", account_seq);
        }

        self.rate_limiter.wait(rate_group).await;
        let resp = request.send().await.context("토스증권 OpenAPI 요청 실패")?;
        let status = resp.status();
        let headers = resp.headers().clone();
        self.rate_limiter
            .apply_response_headers(rate_group, &headers)
            .await;
        let text = resp.text().await.unwrap_or_default();

        if status == StatusCode::UNAUTHORIZED && !had_retry_token {
            *self.current_token.lock().await = None;
            let new_token = self.get_token().await?;
            return Box::pin(self.post_json(path, account_seq, body, Some(new_token))).await;
        }

        if !status.is_success() {
            return Err(format_toss_error(
                status,
                &headers,
                &text,
                "OpenAPI 요청 실패",
            ));
        }

        serde_json::from_str(&text)
            .with_context(|| format!("토스증권 OpenAPI 응답 파싱 실패: body={text}"))
    }

    fn credentials(&self) -> anyhow::Result<&TossCredentials> {
        self.credentials.as_ref().ok_or_else(|| {
            anyhow!("토스증권 OpenAPI client_id/client_secret이 설정되지 않았습니다")
        })
    }

    fn require_account_seq(&self, account_seq: Option<&str>) -> anyhow::Result<String> {
        account_seq
            .map(str::to_string)
            .or_else(|| {
                self.credentials
                    .as_ref()
                    .and_then(|c| c.account_seq.clone())
            })
            .ok_or_else(|| anyhow!("토스증권 계좌 API 호출에는 accountSeq가 필요합니다"))
    }
}

#[derive(Debug, Clone)]
struct TossCredentials {
    client_id: String,
    client_secret: String,
    account_seq: Option<String>,
}

#[derive(Debug, Serialize)]
struct TossTokenRequest<'a> {
    grant_type: &'static str,
    client_id: &'a str,
    client_secret: &'a str,
}

#[derive(Debug, Deserialize)]
struct TossTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
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
struct TossApiResponse<T> {
    result: T,
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
    pub fn to_broker_candle(
        &self,
        symbol: &BrokerSymbol,
    ) -> anyhow::Result<super::domain::BrokerCandle> {
        let currency = toss_currency(&self.currency)?;
        Ok(super::domain::BrokerCandle {
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
    fn to_broker_holding(
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TossOrderListStatus {
    Open,
    Closed,
}

impl TossOrderListStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Closed => "CLOSED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TossOrderListQuery {
    pub status: TossOrderListStatus,
    pub symbol: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u8>,
}

impl TossOrderListQuery {
    pub fn open() -> Self {
        Self {
            status: TossOrderListStatus::Open,
            symbol: None,
            from: None,
            to: None,
            cursor: None,
            limit: None,
        }
    }

    fn validate(&self) -> anyhow::Result<()> {
        if let Some(symbol) = &self.symbol {
            validate_toss_symbol(symbol)?;
        }
        if let Some(from) = &self.from {
            validate_iso_date(from)?;
        }
        if let Some(to) = &self.to {
            validate_iso_date(to)?;
        }
        if let Some(limit) = self.limit {
            if !(1..=100).contains(&limit) {
                return Err(anyhow!(
                    "토스증권 주문 목록 limit은 1~100 범위여야 합니다: {limit}"
                ));
            }
        }
        Ok(())
    }

    fn to_path(&self) -> String {
        let mut params = vec![format!("status={}", self.status.as_str())];
        if let Some(symbol) = &self.symbol {
            params.push(format!("symbol={}", url_encode(symbol)));
        }
        if let Some(from) = &self.from {
            params.push(format!("from={}", url_encode(from)));
        }
        if let Some(to) = &self.to {
            params.push(format!("to={}", url_encode(to)));
        }
        if let Some(cursor) = &self.cursor {
            params.push(format!("cursor={}", url_encode(cursor)));
        }
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        format!("/api/v1/orders?{}", params.join("&"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm_high_value_order: Option<bool>,
}

impl TossOrderCreateRequest {
    pub fn with_generated_client_order_id(mut self) -> Self {
        self.client_order_id = Some(new_toss_client_order_id());
        self
    }

    fn validate(&self) -> anyhow::Result<()> {
        validate_client_order_id(self.client_order_id.as_deref())?;
        validate_toss_symbol(&self.symbol)?;
        validate_order_side(&self.side)?;
        validate_order_type(&self.order_type)?;
        if let Some(time_in_force) = &self.time_in_force {
            validate_time_in_force(time_in_force)?;
        }
        validate_optional_decimal("quantity", self.quantity.as_deref())?;
        validate_optional_decimal("price", self.price.as_deref())?;
        validate_optional_decimal("orderAmount", self.order_amount.as_deref())?;

        match (self.quantity.is_some(), self.order_amount.is_some()) {
            (true, false) | (false, true) => Ok(()),
            _ => Err(anyhow!(
                "토스증권 주문 생성은 quantity 또는 orderAmount 중 정확히 하나만 허용합니다"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderModifyRequest {
    pub order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm_high_value_order: Option<bool>,
}

impl TossOrderModifyRequest {
    fn validate(&self) -> anyhow::Result<()> {
        validate_order_type(&self.order_type)?;
        validate_optional_decimal("quantity", self.quantity.as_deref())?;
        validate_optional_decimal("price", self.price.as_deref())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderResponse {
    pub order_id: String,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderOperationResponse {
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossPaginatedOrderResponse {
    pub orders: Vec<TossOrder>,
    pub next_cursor: Option<String>,
    pub has_next: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrder {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub time_in_force: String,
    pub status: String,
    pub price: Option<String>,
    pub quantity: String,
    pub order_amount: Option<String>,
    pub currency: String,
    pub ordered_at: String,
    pub canceled_at: Option<String>,
    pub execution: TossOrderExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderExecution {
    pub filled_quantity: String,
    pub average_filled_price: Option<String>,
    pub filled_amount: Option<String>,
    pub commission: Option<String>,
    pub tax: Option<String>,
    pub filled_at: Option<String>,
    pub settlement_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TossErrorResponse {
    error: TossApiError,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TossApiError {
    request_id: Option<String>,
    code: String,
    message: String,
    data: Option<serde_json::Value>,
}

fn format_toss_error(
    status: StatusCode,
    headers: &HeaderMap,
    text: &str,
    context: &str,
) -> anyhow::Error {
    let request_id = headers
        .get("X-Request-Id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let retry_after = headers
        .get("Retry-After")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    if let Ok(parsed) = serde_json::from_str::<TossErrorResponse>(text) {
        return anyhow!(
            "{context}: HTTP {status}; code={}; message={}; request_id={:?}; header_request_id={:?}; retry_after={:?}; data={:?}",
            parsed.error.code,
            parsed.error.message,
            parsed.error.request_id,
            request_id,
            retry_after,
            parsed.error.data
        );
    }

    anyhow!(
        "{context}: HTTP {status}; request_id={:?}; retry_after={:?}; body={}",
        request_id,
        retry_after,
        text
    )
}

fn toss_rate_limiter() -> RateLimitScheduler {
    RateLimitScheduler::with_min_intervals([
        ("toss:auth", StdDuration::from_millis(500)),
        ("toss:account", StdDuration::from_millis(200)),
        ("toss:order", StdDuration::from_millis(500)),
        ("toss:order_history", StdDuration::from_millis(200)),
        ("toss:market", StdDuration::from_millis(100)),
    ])
}

fn toss_rate_limit_group(method: &str, path: &str) -> &'static str {
    if path.starts_with("/oauth2/") {
        "toss:auth"
    } else if path.starts_with("/api/v1/orders") && method.eq_ignore_ascii_case("GET") {
        "toss:order_history"
    } else if path.starts_with("/api/v1/orders") {
        "toss:order"
    } else if path.starts_with("/api/v1/accounts")
        || path.starts_with("/api/v1/holdings")
        || path.starts_with("/api/v1/buying-power")
        || path.starts_with("/api/v1/sellable-quantity")
        || path.starts_with("/api/v1/commissions")
    {
        "toss:account"
    } else {
        "toss:market"
    }
}

fn toss_currency(value: &str) -> anyhow::Result<BrokerCurrency> {
    match value {
        "KRW" => Ok(BrokerCurrency::Krw),
        "USD" => Ok(BrokerCurrency::Usd),
        other => Err(anyhow!("지원하지 않는 Toss currency: {other}")),
    }
}

fn toss_currency_code(value: BrokerCurrency) -> &'static str {
    match value {
        BrokerCurrency::Krw => "KRW",
        BrokerCurrency::Usd => "USD",
    }
}

fn toss_market(value: &str) -> anyhow::Result<BrokerMarket> {
    match value {
        "KR" => Ok(BrokerMarket::Kr),
        "US" => Ok(BrokerMarket::Us),
        other => Err(anyhow!("지원하지 않는 Toss marketCountry: {other}")),
    }
}

fn market_from_currency(currency: BrokerCurrency) -> BrokerMarket {
    match currency {
        BrokerCurrency::Krw => BrokerMarket::Kr,
        BrokerCurrency::Usd => BrokerMarket::Us,
    }
}

fn validate_toss_symbol(symbol: &str) -> anyhow::Result<()> {
    if symbol.is_empty() || symbol.len() > 32 {
        return Err(anyhow!("토스증권 symbol은 1~32자여야 합니다: {symbol}"));
    }
    if !symbol
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(anyhow!(
            "토스증권 symbol은 영문/숫자/./- 문자만 허용합니다: {symbol}"
        ));
    }
    Ok(())
}

fn validate_toss_order_id(order_id: &str) -> anyhow::Result<()> {
    if order_id.is_empty() {
        return Err(anyhow!("토스증권 orderId는 비어 있을 수 없습니다"));
    }
    if !order_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "토스증권 orderId는 영문/숫자/-/_ 문자만 허용합니다: {order_id}"
        ));
    }
    Ok(())
}

fn validate_client_order_id(value: Option<&str>) -> anyhow::Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty() || value.len() > 36 {
        return Err(anyhow!(
            "토스증권 clientOrderId는 1~36자여야 합니다: {value}"
        ));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "토스증권 clientOrderId는 영문/숫자/-/_ 문자만 허용합니다: {value}"
        ));
    }
    Ok(())
}

fn new_toss_client_order_id() -> String {
    format!("ka-{}", uuid::Uuid::new_v4().simple())
}

fn validate_order_side(value: &str) -> anyhow::Result<()> {
    if matches!(value, "BUY" | "SELL") {
        Ok(())
    } else {
        Err(anyhow!(
            "토스증권 주문 방향은 BUY 또는 SELL만 허용합니다: {value}"
        ))
    }
}

fn validate_order_type(value: &str) -> anyhow::Result<()> {
    if matches!(value, "LIMIT" | "MARKET") {
        Ok(())
    } else {
        Err(anyhow!(
            "토스증권 주문 유형은 LIMIT 또는 MARKET만 허용합니다: {value}"
        ))
    }
}

fn validate_time_in_force(value: &str) -> anyhow::Result<()> {
    if matches!(value, "DAY" | "CLS") {
        Ok(())
    } else {
        Err(anyhow!(
            "토스증권 timeInForce는 DAY 또는 CLS만 허용합니다: {value}"
        ))
    }
}

fn validate_optional_decimal(field: &str, value: Option<&str>) -> anyhow::Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty()
        || value.len() > 30
        || !value.chars().all(|c| c.is_ascii_digit() || c == '.')
        || value.matches('.').count() > 1
        || value == "."
    {
        return Err(anyhow!(
            "토스증권 {field}는 양수 decimal 문자열이어야 합니다: {value}"
        ));
    }
    Ok(())
}

fn validate_iso_date(date: &str) -> anyhow::Result<()> {
    let is_valid = date.len() == 10
        && date.chars().enumerate().all(|(index, ch)| {
            matches!(index, 4 | 7) && ch == '-' || !matches!(index, 4 | 7) && ch.is_ascii_digit()
        });
    if !is_valid {
        return Err(anyhow!(
            "토스증권 market-calendar date는 YYYY-MM-DD 형식이어야 합니다: {date}"
        ));
    }
    Ok(())
}

fn trim_base_url(value: String) -> String {
    value.trim_end_matches('/').to_string()
}

fn url_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_' | b'~' => {
                vec![byte as char]
            }
            other => format!("%{other:02X}").chars().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_official_base_url() {
        let adapter = TossBrokerAdapter::default();
        assert_eq!(adapter.base_url(), "https://openapi.tossinvest.com");
        assert_eq!(adapter.broker_id(), BrokerId::Toss);
    }

    #[test]
    fn classifies_toss_rate_limit_groups() {
        assert_eq!(toss_rate_limit_group("POST", "/oauth2/token"), "toss:auth");
        assert_eq!(
            toss_rate_limit_group("GET", "/api/v1/holdings?currency=USD"),
            "toss:account"
        );
        assert_eq!(
            toss_rate_limit_group("GET", "/api/v1/orders/order-1"),
            "toss:order_history"
        );
        assert_eq!(
            toss_rate_limit_group("POST", "/api/v1/orders/order-1/cancel"),
            "toss:order"
        );
        assert_eq!(
            toss_rate_limit_group("GET", "/api/v1/prices?symbols=AAPL"),
            "toss:market"
        );
    }

    #[test]
    fn deserializes_accounts() {
        let json = r#"{
          "result": [
            { "accountNo": "12345678901", "accountSeq": 1, "accountType": "BROKERAGE" }
          ]
        }"#;

        let response: TossApiResponse<Vec<TossAccount>> = serde_json::from_str(json).unwrap();
        assert_eq!(response.result[0].account_no, "12345678901");
        assert_eq!(response.result[0].account_seq, 1);
    }

    #[test]
    fn maps_holding_to_broker_domain() {
        let json = r#"{
          "symbol": "AAPL",
          "name": "Apple Inc.",
          "marketCountry": "US",
          "currency": "USD",
          "quantity": "10.5",
          "lastPrice": "178.5",
          "averagePurchasePrice": "155.3",
          "marketValue": {
            "purchaseAmount": "1630.65",
            "amount": "1874.25",
            "amountAfterCost": "1868.25"
          },
          "profitLoss": {
            "amount": "243.60",
            "amountAfterCost": "237.60",
            "rate": "0.1494",
            "rateAfterCost": "0.1457"
          },
          "dailyProfitLoss": {
            "amount": "25",
            "rate": "0.0141"
          },
          "cost": {
            "commission": "6.0",
            "tax": null
          }
        }"#;
        let item: TossHoldingsItem = serde_json::from_str(json).unwrap();
        let account = BrokerAccountId("1".to_string());

        let holding = item.to_broker_holding(Some(&account)).unwrap();

        assert_eq!(holding.broker, BrokerId::Toss);
        assert_eq!(holding.account_id, Some(account));
        assert_eq!(holding.market, BrokerMarket::Us);
        assert_eq!(holding.current_price.currency, BrokerCurrency::Usd);
        assert_eq!(holding.quantity.0, "10.5");
    }

    #[test]
    fn url_encodes_symbol_filter() {
        assert_eq!(url_encode("BRK.B"), "BRK.B");
        assert_eq!(url_encode("ABC DEF"), "ABC%20DEF");
    }

    #[test]
    fn deserializes_order_preflight_responses() {
        let buying_power_json = r#"{
          "result": {
            "currency": "USD",
            "cashBuyingPower": "3500.5"
          }
        }"#;
        let sellable_json = r#"{
          "result": {
            "sellableQuantity": "5.5"
          }
        }"#;
        let commissions_json = r#"{
          "result": [
            {
              "marketCountry": "KR",
              "commissionRate": "0.015",
              "startDate": "2026-01-01",
              "endDate": null
            },
            {
              "marketCountry": "US",
              "commissionRate": "0.1",
              "startDate": null,
              "endDate": "2026-06-30"
            }
          ]
        }"#;

        let buying_power: TossApiResponse<TossBuyingPower> =
            serde_json::from_str(buying_power_json).unwrap();
        let sellable: TossApiResponse<TossSellableQuantity> =
            serde_json::from_str(sellable_json).unwrap();
        let commissions: TossApiResponse<Vec<TossCommission>> =
            serde_json::from_str(commissions_json).unwrap();

        assert_eq!(
            buying_power.result.money().unwrap().currency,
            BrokerCurrency::Usd
        );
        assert_eq!(buying_power.result.cash_buying_power, "3500.5");
        assert_eq!(sellable.result.quantity().0, "5.5");
        assert_eq!(commissions.result.len(), 2);
        assert_eq!(commissions.result[0].market_country, "KR");
    }

    #[test]
    fn validates_order_create_request_shape() {
        let valid = TossOrderCreateRequest {
            client_order_id: Some("order-001_A".to_string()),
            symbol: "005930".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            time_in_force: Some("DAY".to_string()),
            quantity: Some("10".to_string()),
            price: Some("70000".to_string()),
            order_amount: None,
            confirm_high_value_order: Some(false),
        };
        assert!(valid.validate().is_ok());

        let mut both_quantity_and_amount = valid.clone();
        both_quantity_and_amount.order_amount = Some("100".to_string());
        assert!(both_quantity_and_amount.validate().is_err());

        let mut invalid_client_order_id = valid;
        invalid_client_order_id.client_order_id = Some("bad id with spaces".to_string());
        assert!(invalid_client_order_id.validate().is_err());

        let generated = new_toss_client_order_id();
        assert!(generated.len() <= 36);
        assert!(validate_client_order_id(Some(&generated)).is_ok());
    }

    #[test]
    fn deserializes_order_api_responses() {
        let create_json = r#"{
          "result": {
            "orderId": "ORD_001",
            "clientOrderId": "client-001"
          }
        }"#;
        let list_json = r#"{
          "result": {
            "orders": [
              {
                "orderId": "ORD_001",
                "symbol": "AAPL",
                "side": "BUY",
                "orderType": "LIMIT",
                "timeInForce": "DAY",
                "status": "PARTIAL_FILLED",
                "price": "185.5",
                "quantity": "5",
                "orderAmount": null,
                "currency": "USD",
                "orderedAt": "2026-03-29T10:00:00+09:00",
                "canceledAt": null,
                "execution": {
                  "filledQuantity": "2",
                  "averageFilledPrice": "185.25",
                  "filledAmount": "370.5",
                  "commission": "0.66",
                  "tax": "0",
                  "filledAt": "2026-03-29T10:00:05+09:00",
                  "settlementDate": null
                }
              }
            ],
            "nextCursor": null,
            "hasNext": false
          }
        }"#;
        let operation_json = r#"{
          "result": {
            "orderId": "ORD_002"
          }
        }"#;

        let created: TossApiResponse<TossOrderResponse> =
            serde_json::from_str(create_json).unwrap();
        let listed: TossApiResponse<TossPaginatedOrderResponse> =
            serde_json::from_str(list_json).unwrap();
        let modified: TossApiResponse<TossOrderOperationResponse> =
            serde_json::from_str(operation_json).unwrap();

        assert_eq!(created.result.order_id, "ORD_001");
        assert_eq!(
            created.result.client_order_id.as_deref(),
            Some("client-001")
        );
        assert_eq!(listed.result.orders[0].status, "PARTIAL_FILLED");
        assert_eq!(listed.result.orders[0].execution.filled_quantity, "2");
        assert!(!listed.result.has_next);
        assert_eq!(modified.result.order_id, "ORD_002");
    }

    #[test]
    fn builds_order_list_query_path() {
        let query = TossOrderListQuery {
            status: TossOrderListStatus::Closed,
            symbol: Some("BRK.B".to_string()),
            from: Some("2026-03-01".to_string()),
            to: Some("2026-03-31".to_string()),
            cursor: Some("next cursor".to_string()),
            limit: Some(100),
        };

        assert!(query.validate().is_ok());
        assert_eq!(
            query.to_path(),
            "/api/v1/orders?status=CLOSED&symbol=BRK.B&from=2026-03-01&to=2026-03-31&cursor=next%20cursor&limit=100"
        );
    }

    #[test]
    fn deserializes_market_data_responses() {
        let prices_json = r#"{
          "result": [
            {
              "symbol": "005930",
              "timestamp": "2026-03-25T09:30:00.123+09:00",
              "lastPrice": "72000",
              "currency": "KRW"
            },
            {
              "symbol": "AAPL",
              "timestamp": null,
              "lastPrice": "178.5",
              "currency": "USD"
            }
          ]
        }"#;
        let orderbook_json = r#"{
          "result": {
            "timestamp": "2026-03-25T09:30:00.123+09:00",
            "currency": "KRW",
            "asks": [{ "price": "72100", "volume": "8500" }],
            "bids": [{ "price": "72000", "volume": "12000" }]
          }
        }"#;
        let trades_json = r#"{
          "result": [
            {
              "price": "72000",
              "volume": "120",
              "timestamp": "2026-03-25T09:30:42.000+09:00",
              "currency": "KRW"
            }
          ]
        }"#;
        let limits_json = r#"{
          "result": {
            "timestamp": "2026-03-25T09:30:00.123+09:00",
            "upperLimitPrice": "93000",
            "lowerLimitPrice": "50400",
            "currency": "KRW"
          }
        }"#;

        let prices: TossApiResponse<Vec<TossPriceResponse>> =
            serde_json::from_str(prices_json).unwrap();
        let orderbook: TossApiResponse<TossOrderbookResponse> =
            serde_json::from_str(orderbook_json).unwrap();
        let trades: TossApiResponse<Vec<TossTrade>> = serde_json::from_str(trades_json).unwrap();
        let limits: TossApiResponse<TossPriceLimitResponse> =
            serde_json::from_str(limits_json).unwrap();

        let quote = prices.result[0].to_broker_price_quote().unwrap();
        assert_eq!(quote.broker, BrokerId::Toss);
        assert_eq!(quote.market, BrokerMarket::Kr);
        assert_eq!(quote.last, BrokerMoney::krw("72000"));
        assert_eq!(
            prices.result[1].to_broker_price_quote().unwrap().market,
            BrokerMarket::Us
        );
        assert_eq!(orderbook.result.asks[0].price, "72100");
        assert_eq!(orderbook.result.bids[0].volume, "12000");
        assert_eq!(trades.result[0].volume, "120");
        assert_eq!(limits.result.upper_limit_price.as_deref(), Some("93000"));
    }

    #[test]
    fn deserializes_exchange_rate_response() {
        let json = r#"{
          "result": {
            "baseCurrency": "USD",
            "quoteCurrency": "KRW",
            "rate": "1380.5",
            "midRate": "1375",
            "basisPoint": "40",
            "rateChangeType": "UP",
            "validFrom": "2026-03-25T09:30:00+09:00",
            "validUntil": "2026-03-25T09:31:00+09:00"
          }
        }"#;

        let rate: TossApiResponse<TossExchangeRateResponse> = serde_json::from_str(json).unwrap();

        assert_eq!(rate.result.base_currency, "USD");
        assert_eq!(rate.result.quote_currency, "KRW");
        assert_eq!(rate.result.rate_as_f64().unwrap(), 1380.5);
        assert_eq!(rate.result.valid_until, "2026-03-25T09:31:00+09:00");
    }

    #[test]
    fn deserializes_stock_info_and_unknown_warning_codes() {
        let stocks_json = r#"{
          "result": [
            {
              "symbol": "005930",
              "name": "삼성전자",
              "englishName": "SamsungElec",
              "isinCode": "KR7005930003",
              "market": "KOSPI",
              "securityType": "STOCK",
              "isCommonShare": true,
              "status": "ACTIVE",
              "currency": "KRW",
              "listDate": "1975-06-11",
              "delistDate": null,
              "sharesOutstanding": "5919637922",
              "leverageFactor": null,
              "koreanMarketDetail": { "sector": "전기전자" }
            }
          ]
        }"#;
        let warnings_json = r#"{
          "result": [
            {
              "warningType": "INVESTMENT_RISK",
              "exchange": "KRX",
              "startDate": "2026-03-26",
              "endDate": null
            },
            {
              "warningType": "NEW_WARNING_CODE",
              "exchange": null,
              "startDate": null,
              "endDate": null
            }
          ]
        }"#;

        let stocks: TossApiResponse<Vec<TossStockInfo>> =
            serde_json::from_str(stocks_json).unwrap();
        let warnings: TossApiResponse<Vec<TossStockWarning>> =
            serde_json::from_str(warnings_json).unwrap();

        assert_eq!(stocks.result[0].market, "KOSPI");
        assert_eq!(stocks.result[0].shares_outstanding, "5919637922");
        assert_eq!(warnings.result[0].warning_type, "INVESTMENT_RISK");
        assert!(warnings.result[0].is_blocking_for_buy());
        assert_eq!(warnings.result[1].warning_type, "NEW_WARNING_CODE");
        assert!(!warnings.result[1].is_blocking_for_buy());
    }

    #[test]
    fn deserializes_market_calendar_responses() {
        let kr_json = r#"{
          "result": {
            "today": {
              "date": "2026-03-25",
              "integrated": {
                "preMarket": null,
                "regularMarket": {
                  "startTime": "2026-03-25T09:00:00+09:00",
                  "singlePriceAuctionStartTime": "2026-03-25T15:20:00+09:00",
                  "endTime": "2026-03-25T15:30:00+09:00"
                },
                "afterMarket": null
              }
            },
            "previousBusinessDay": { "date": "2026-03-24", "integrated": null },
            "nextBusinessDay": { "date": "2026-03-26", "integrated": null }
          }
        }"#;
        let us_json = r#"{
          "result": {
            "today": {
              "date": "2026-03-25",
              "dayMarket": null,
              "preMarket": null,
              "regularMarket": {
                "startTime": "2026-03-25T22:30:00+09:00",
                "endTime": "2026-03-26T05:00:00+09:00"
              },
              "afterMarket": null
            },
            "previousBusinessDay": {
              "date": "2026-03-24",
              "dayMarket": null,
              "preMarket": null,
              "regularMarket": null,
              "afterMarket": null
            },
            "nextBusinessDay": {
              "date": "2026-03-26",
              "dayMarket": null,
              "preMarket": null,
              "regularMarket": null,
              "afterMarket": null
            }
          }
        }"#;

        let kr: TossApiResponse<TossKrMarketCalendarResponse> =
            serde_json::from_str(kr_json).unwrap();
        let us: TossApiResponse<TossUsMarketCalendarResponse> =
            serde_json::from_str(us_json).unwrap();

        let kr_regular = kr.result.today.integrated.unwrap().regular_market.unwrap();
        let us_regular = us.result.today.regular_market.unwrap();
        assert_eq!(kr_regular.start_time, "2026-03-25T09:00:00+09:00");
        assert_eq!(
            kr_regular.single_price_auction_start_time.as_deref(),
            Some("2026-03-25T15:20:00+09:00")
        );
        assert_eq!(us_regular.end_time, "2026-03-26T05:00:00+09:00");
    }

    #[test]
    fn maps_toss_candle_to_broker_candle() {
        let json = r#"{
          "result": {
            "candles": [
              {
                "timestamp": "2026-03-25T09:00:00+09:00",
                "openPrice": "71600",
                "highPrice": "72300",
                "lowPrice": "71500",
                "closePrice": "72000",
                "volume": "3521000",
                "currency": "KRW"
              }
            ],
            "nextBefore": "2026-03-24T09:00:00+09:00"
          }
        }"#;
        let response: TossApiResponse<TossCandlePageResponse> = serde_json::from_str(json).unwrap();

        let candle = response.result.candles[0]
            .to_broker_candle(&BrokerSymbol("005930".to_string()))
            .unwrap();

        assert_eq!(candle.market, BrokerMarket::Kr);
        assert_eq!(candle.date, "2026-03-25T09:00:00+09:00");
        assert_eq!(candle.open, BrokerMoney::krw("71600"));
        assert_eq!(candle.close, BrokerMoney::krw("72000"));
        assert_eq!(candle.volume, BrokerQuantity("3521000".to_string()));
        assert_eq!(
            response.result.next_before.as_deref(),
            Some("2026-03-24T09:00:00+09:00")
        );
    }

    #[tokio::test]
    async fn validates_market_data_query_limits_before_request() {
        let client = TossOpenApiClient::without_credentials("https://example.invalid");
        let too_many = (0..201)
            .map(|i| BrokerSymbol(format!("SYM{i}")))
            .collect::<Vec<_>>();

        assert!(client.list_prices(&[]).await.is_err());
        assert!(client.list_prices(&too_many).await.is_err());
        assert!(client.list_stocks(&[]).await.is_err());
        assert!(client.list_stocks(&too_many).await.is_err());
        assert!(client
            .list_warnings(&BrokerSymbol("bad symbol".to_string()))
            .await
            .is_err());
        assert!(client
            .get_kr_market_calendar(Some("20260325"))
            .await
            .is_err());
        assert!(client
            .get_us_market_calendar(Some("2026/03/25"))
            .await
            .is_err());
        assert!(client
            .list_trades(&BrokerSymbol("AAPL".to_string()), Some(51))
            .await
            .is_err());
        assert!(client
            .get_candles(
                &BrokerSymbol("AAPL".to_string()),
                "1h",
                Some(100),
                None,
                None
            )
            .await
            .is_err());
        assert!(client
            .get_candles(
                &BrokerSymbol("AAPL".to_string()),
                "1d",
                Some(201),
                None,
                None
            )
            .await
            .is_err());
    }
}
