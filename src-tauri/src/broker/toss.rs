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

    async fn get_price(&self, _symbol: &BrokerSymbol) -> BrokerAdapterResult<BrokerPriceQuote> {
        Err(BrokerAdapterError::Unsupported {
            broker: BrokerId::Toss,
            operation: "get_price",
        })
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
}

impl TossOpenApiClient {
    pub fn without_credentials(base_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: trim_base_url(base_url.into()),
            credentials: None,
            current_token: Mutex::new(None),
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
        let body = TossTokenRequest {
            grant_type: "client_credentials",
            client_id: &credentials.client_id,
            client_secret: &credentials.client_secret,
        };

        let resp = self
            .http
            .post(url)
            .form(&body)
            .send()
            .await
            .context("토스증권 토큰 발급 요청 실패")?;

        let status = resp.status();
        let headers = resp.headers().clone();
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
        let mut request = self.http.get(url).bearer_auth(token);
        if let Some(account_seq) = account_seq {
            request = request.header("X-Tossinvest-Account", account_seq);
        }

        let resp = request.send().await.context("토스증권 OpenAPI 요청 실패")?;
        let status = resp.status();
        let headers = resp.headers().clone();
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
}
