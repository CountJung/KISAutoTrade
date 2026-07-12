use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex, OnceLock},
};

use anyhow::{anyhow, Context};
use chrono::{Duration, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;

use super::{
    error::format_toss_error,
    http::{body_snippet, read_toss_response_text, toss_http_client, trim_base_url, url_encode},
    orders::{
        TossOrder, TossOrderCreateRequest, TossOrderListQuery, TossOrderModifyRequest,
        TossOrderOperationResponse, TossOrderResponse, TossPaginatedOrderResponse,
    },
    support::{
        toss_currency_code, toss_rate_limit_group, toss_rate_limiter, validate_iso_date,
        validate_toss_order_id, validate_toss_symbol,
    },
    types::{
        TossAccessToken, TossAccount, TossApiResponse, TossBuyingPower, TossCandlePageResponse,
        TossCommission, TossCredentials, TossExchangeRateResponse, TossHoldingsOverview,
        TossKrMarketCalendarResponse, TossOpenApiOverview, TossOrderbookResponse,
        TossPriceLimitResponse, TossPriceResponse, TossSellableQuantity, TossStockInfo,
        TossStockWarning, TossTokenRequest, TossTokenResponse, TossTrade,
        TossUsMarketCalendarResponse,
    },
};
use crate::broker::{
    domain::{BrokerCurrency, BrokerSymbol},
    rate_limit::RateLimitScheduler,
};

pub struct TossOpenApiClient {
    http: Client,
    base_url: String,
    credentials: Option<TossCredentials>,
    token_state: Arc<TossTokenState>,
    rate_limiter: RateLimitScheduler,
}

struct TossTokenState {
    current_token: AsyncMutex<Option<TossAccessToken>>,
    token_refresh: AsyncMutex<()>,
}

impl TossTokenState {
    fn new() -> Self {
        Self {
            current_token: AsyncMutex::new(None),
            token_refresh: AsyncMutex::new(()),
        }
    }
}

static TOSS_TOKEN_STATES: OnceLock<StdMutex<HashMap<String, Arc<TossTokenState>>>> =
    OnceLock::new();

fn toss_token_state_key(base_url: &str, client_id: &str) -> String {
    format!("{base_url}|{client_id}")
}

fn shared_toss_token_state(base_url: &str, client_id: &str) -> Arc<TossTokenState> {
    let key = toss_token_state_key(base_url, client_id);
    let states = TOSS_TOKEN_STATES.get_or_init(|| StdMutex::new(HashMap::new()));
    let mut states = states.lock().expect("Toss token cache mutex poisoned");
    states
        .entry(key)
        .or_insert_with(|| Arc::new(TossTokenState::new()))
        .clone()
}

impl TossOpenApiClient {
    pub fn without_credentials(base_url: impl Into<String>) -> Self {
        let base_url = trim_base_url(base_url.into());
        let rate_limiter = toss_rate_limiter(&base_url, None);
        Self {
            http: toss_http_client(),
            base_url,
            credentials: None,
            token_state: Arc::new(TossTokenState::new()),
            rate_limiter,
        }
    }

    pub fn new(
        base_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        account_seq: Option<impl Into<String>>,
    ) -> Self {
        let base_url = trim_base_url(base_url.into());
        let client_id = client_id.into();
        let client_secret = client_secret.into();
        let token_state = shared_toss_token_state(&base_url, &client_id);
        let rate_limiter = toss_rate_limiter(&base_url, Some(&client_id));
        Self {
            http: toss_http_client(),
            base_url,
            credentials: Some(TossCredentials {
                client_id,
                client_secret,
                account_seq: account_seq.map(Into::into),
            }),
            token_state,
            rate_limiter,
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
        if let Some(token) = self
            .token_state
            .current_token
            .lock()
            .await
            .as_ref()
            .cloned()
        {
            if !token.is_expired() {
                return Ok(token);
            }
        }

        let _refresh_guard = self.token_state.token_refresh.lock().await;
        if let Some(token) = self
            .token_state
            .current_token
            .lock()
            .await
            .as_ref()
            .cloned()
        {
            if !token.is_expired() {
                return Ok(token);
            }
        }

        let token = self.request_new_token().await?;
        *self.token_state.current_token.lock().await = Some(token.clone());
        Ok(token)
    }

    pub async fn issue_token(&self) -> anyhow::Result<TossAccessToken> {
        let _refresh_guard = self.token_state.token_refresh.lock().await;
        let token = self.request_new_token().await?;
        *self.token_state.current_token.lock().await = Some(token.clone());
        Ok(token)
    }

    async fn request_new_token(&self) -> anyhow::Result<TossAccessToken> {
        let credentials = self.credentials()?;
        let url = format!("{}/oauth2/token", self.base_url);
        let rate_group = "toss:auth";
        let body = TossTokenRequest {
            grant_type: "client_credentials",
            client_id: &credentials.client_id,
            client_secret: &credentials.client_secret,
        };

        self.rate_limiter.wait(rate_group).await;
        let resp = match self.http.post(url).form(&body).send().await {
            Ok(resp) => resp,
            Err(error) => {
                self.rate_limiter.record_outcome(rate_group, false).await;
                return Err(error).context("토스증권 토큰 발급 요청 실패");
            }
        };

        let status = resp.status();
        let headers = resp.headers().clone();
        self.rate_limiter
            .apply_response_headers(rate_group, &headers)
            .await;
        self.rate_limiter
            .record_outcome(rate_group, status.is_success())
            .await;
        let text = read_toss_response_text(resp).await?;
        if !status.is_success() {
            return Err(format_toss_error(status, &headers, &text, "토큰 발급 실패"));
        }

        let body: TossTokenResponse = serde_json::from_str(&text).with_context(|| {
            format!("토스증권 토큰 응답 파싱 실패: body={}", body_snippet(&text))
        })?;
        Ok(TossAccessToken {
            access_token: body.access_token,
            token_type: body.token_type,
            expires_at: Utc::now() + Duration::seconds(body.expires_in),
        })
    }

    async fn refresh_after_unauthorized(&self, rejected_token: &str) -> anyhow::Result<String> {
        {
            let mut current = self.token_state.current_token.lock().await;
            if current
                .as_ref()
                .is_some_and(|token| token.access_token == rejected_token)
            {
                *current = None;
            }
        }
        self.get_token().await
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
        let text = read_toss_response_text(resp).await?;
        if !status.is_success() {
            return Err(anyhow!(
                "토스증권 OpenAPI JSON 조회 실패: HTTP {status}; body={}",
                body_snippet(&text)
            ));
        }

        let spec: serde_json::Value = serde_json::from_str(&text).with_context(|| {
            format!(
                "토스증권 OpenAPI JSON 파싱 실패: body={}",
                body_snippet(&text)
            )
        })?;
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
        let mut request = self.http.get(url).bearer_auth(&token);
        if let Some(account_seq) = account_seq {
            request = request.header("X-Tossinvest-Account", account_seq);
        }

        self.rate_limiter.wait(rate_group).await;
        let retry_request = request.try_clone();
        let resp = match request.send().await {
            Ok(resp) => resp,
            Err(first_error) => {
                if let Some(retry_request) = retry_request {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    match retry_request.send().await {
                        Ok(resp) => resp,
                        Err(retry_error) => {
                            self.rate_limiter.record_outcome(rate_group, false).await;
                            return Err(anyhow!(
                                "토스증권 OpenAPI 요청 실패: path={path}; first={first_error}; retry={retry_error}"
                            ));
                        }
                    }
                } else {
                    self.rate_limiter.record_outcome(rate_group, false).await;
                    return Err(anyhow!(
                        "토스증권 OpenAPI 요청 실패: path={path}; {first_error}"
                    ));
                }
            }
        };
        let status = resp.status();
        let headers = resp.headers().clone();
        self.rate_limiter
            .apply_response_headers(rate_group, &headers)
            .await;
        self.rate_limiter
            .record_outcome(rate_group, status.is_success())
            .await;
        let text = read_toss_response_text(resp).await?;

        if status == StatusCode::UNAUTHORIZED && !had_retry_token {
            let new_token = self.refresh_after_unauthorized(&token).await?;
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

        serde_json::from_str(&text).with_context(|| {
            format!(
                "토스증권 OpenAPI 응답 파싱 실패: body={}",
                body_snippet(&text)
            )
        })
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
        let mut request = self.http.post(url).bearer_auth(&token).json(body);
        if let Some(account_seq) = account_seq {
            request = request.header("X-Tossinvest-Account", account_seq);
        }

        self.rate_limiter.wait(rate_group).await;
        let resp = match request.send().await {
            Ok(resp) => resp,
            Err(error) => {
                self.rate_limiter.record_outcome(rate_group, false).await;
                return Err(error)
                    .with_context(|| format!("토스증권 OpenAPI 요청 실패: path={path}"));
            }
        };
        let status = resp.status();
        let headers = resp.headers().clone();
        self.rate_limiter
            .apply_response_headers(rate_group, &headers)
            .await;
        self.rate_limiter
            .record_outcome(rate_group, status.is_success())
            .await;
        let text = read_toss_response_text(resp).await?;

        if status == StatusCode::UNAUTHORIZED && !had_retry_token {
            let new_token = self.refresh_after_unauthorized(&token).await?;
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

        serde_json::from_str(&text).with_context(|| {
            format!(
                "토스증권 OpenAPI 응답 파싱 실패: body={}",
                body_snippet(&text)
            )
        })
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
