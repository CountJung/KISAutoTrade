use anyhow::Result;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::RwLock;

use crate::broker::RateLimitScheduler;

use super::token::TokenManager;

const KIS_RATE_GROUP_ACCOUNT: &str = "kis:account";
const KIS_RATE_GROUP_EXECUTION: &str = "kis:execution";
const KIS_RATE_GROUP_ORDER: &str = "kis:order";
const KIS_RATE_GROUP_QUOTE: &str = "kis:quote";

const KIS_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const KIS_HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const KIS_MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

fn kis_read_min_interval(is_paper: bool) -> Duration {
    if is_paper {
        Duration::from_millis(500)
    } else {
        Duration::from_millis(50)
    }
}

fn kis_http_client() -> Client {
    Client::builder()
        .connect_timeout(KIS_CONNECT_TIMEOUT)
        .timeout(KIS_HTTP_TIMEOUT)
        .build()
        .expect("KIS reqwest client with timeout should build")
}

/// KIS 응답 본문을 크기 상한과 함께 읽는다.
async fn read_kis_response_text(mut resp: Response) -> Result<String> {
    use anyhow::{anyhow, Context};
    if resp.content_length().unwrap_or(0) > KIS_MAX_RESPONSE_BYTES as u64 {
        return Err(anyhow!(
            "KIS 응답 본문이 {KIS_MAX_RESPONSE_BYTES} bytes 상한을 초과했습니다"
        ));
    }
    let mut body = Vec::new();
    while let Some(chunk) = resp.chunk().await.context("KIS 응답 본문 읽기 실패")? {
        if body.len().saturating_add(chunk.len()) > KIS_MAX_RESPONSE_BYTES {
            return Err(anyhow!(
                "KIS 응답 본문이 {KIS_MAX_RESPONSE_BYTES} bytes 상한을 초과했습니다"
            ));
        }
        body.extend_from_slice(&chunk);
    }
    String::from_utf8(body).context("KIS 응답 본문 UTF-8 파싱 실패")
}

/// 한국투자증권 REST API 클라이언트
pub struct KisRestClient {
    client: Client,
    base_url: String,
    app_key: String,
    app_secret: String,
    account_no: String,
    is_paper: bool,
    token_manager: Arc<RwLock<TokenManager>>,
    /// KIS API 진단 로그 활성화 플래그 (런타임 토글 가능)
    api_debug: Arc<AtomicBool>,
    rate_limiter: RateLimitScheduler,
}

mod exchange;
mod types;
pub use exchange::fetch_usd_krw_rate;
pub use types::*;

// ────────────────────────────────────────────────────────────────────
// 클라이언트 구현
// ────────────────────────────────────────────────────────────────────

impl KisRestClient {
    pub fn new(
        base_url: String,
        app_key: String,
        app_secret: String,
        account_no: String,
        is_paper: bool,
        token_manager: Arc<RwLock<TokenManager>>,
    ) -> Self {
        // 프로파일 전환 등으로 client가 재생성돼도 같은 credential scope의
        // pacing/pause/운영 상태를 process-wide로 공유한다.
        let scope = format!("kis|{base_url}|{app_key}|paper={is_paper}");
        let rate_limiter = crate::broker::rate_limit::shared_scheduler(&scope, || {
            RateLimitScheduler::with_min_intervals([
                (KIS_RATE_GROUP_ACCOUNT, kis_read_min_interval(is_paper)),
                (KIS_RATE_GROUP_EXECUTION, kis_read_min_interval(is_paper)),
                (KIS_RATE_GROUP_QUOTE, kis_read_min_interval(is_paper)),
                (KIS_RATE_GROUP_ORDER, Duration::from_secs(1)),
            ])
        });
        Self {
            client: kis_http_client(),
            base_url,
            app_key,
            app_secret,
            account_no,
            is_paper,
            token_manager,
            api_debug: Arc::new(AtomicBool::new(false)),
            rate_limiter,
        }
    }

    /// TokenManager 참조 반환 (WebSocket 클라이언트와 토큰 공유용)
    pub fn token_manager(&self) -> Arc<RwLock<TokenManager>> {
        Arc::clone(&self.token_manager)
    }

    /// is_paper 모드 반환
    pub fn is_paper(&self) -> bool {
        self.is_paper
    }

    /// KIS API 진단 로그 ON/OFF (런타임 토글)
    pub fn set_api_debug(&self, enabled: bool) {
        self.api_debug.store(enabled, Ordering::Relaxed);
    }

    /// app_key 반환 (WebSocket 클라이언트 생성용)
    pub fn app_key(&self) -> &str {
        &self.app_key
    }

    /// app_secret 반환 (WebSocket 클라이언트 생성용)
    pub fn app_secret(&self) -> &str {
        &self.app_secret
    }

    /// 공통 인증 헤더 빌더
    async fn auth_headers(&self, tr_id: &str) -> Result<reqwest::header::HeaderMap> {
        let token = self.token_manager.read().await.get_token().await?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("content-type", "application/json; charset=utf-8".parse()?);
        headers.insert("authorization", format!("Bearer {}", token).parse()?);
        headers.insert("appkey", self.app_key.parse()?);
        headers.insert("appsecret", self.app_secret.parse()?);
        headers.insert("tr_id", tr_id.parse()?);
        headers.insert("custtype", "P".parse()?);
        Ok(headers)
    }

    async fn send_with_rate_limit(
        &self,
        group: &'static str,
        request: RequestBuilder,
    ) -> Result<Response> {
        self.rate_limiter.wait(group).await;
        let resp = match request.send().await {
            Ok(resp) => resp,
            Err(error) => {
                self.rate_limiter.record_outcome(group, false).await;
                return Err(error.into());
            }
        };
        let headers = resp.headers().clone();
        self.rate_limiter
            .apply_response_headers(group, &headers)
            .await;
        self.rate_limiter
            .record_outcome(group, resp.status().is_success())
            .await;
        Ok(resp)
    }

    /// 계좌번호 분리 (CANO 8자리 + ACNT_PRDT_CD 2자리)
    fn split_account(&self) -> (&str, &str) {
        if self.account_no.len() >= 10 {
            (&self.account_no[..8], &self.account_no[8..10])
        } else {
            (&self.account_no, "01")
        }
    }

    // ──────────────────────────────────────────────────────────
    // 잔고 조회
    // GET /uapi/domestic-stock/v1/trading/inquire-balance
    // ──────────────────────────────────────────────────────────
    pub async fn get_balance(&self) -> Result<BalanceResponse> {
        let tr_id = if self.is_paper {
            "VTTC8434R"
        } else {
            "TTTC8434R"
        };
        let (cano, acnt_prdt_cd) = self.split_account();

        let url = format!(
            "{}/uapi/domestic-stock/v1/trading/inquire-balance",
            self.base_url
        );
        let headers = self.auth_headers(tr_id).await?;

        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_ACCOUNT,
                self.client.get(&url).headers(headers).query(&[
                    ("CANO", cano),
                    ("ACNT_PRDT_CD", acnt_prdt_cd),
                    ("AFHR_FLPR_YN", "N"),
                    ("OFL_YN", ""),
                    ("INQR_DVSN", "02"),
                    ("UNPR_DVSN", "01"),
                    ("FUND_STTL_ICLD_YN", "N"),
                    ("FNCG_AMT_AUTO_RDPT_YN", "N"),
                    ("PRCS_DVSN", "01"),
                    ("CTX_AREA_FK100", ""),
                    ("CTX_AREA_NK100", ""),
                ]),
            )
            .await?;

        let status = resp.status();
        let body = read_kis_response_text(resp).await.unwrap_or_default();

        if !status.is_success() {
            anyhow::bail!("잔고 조회 실패 HTTP {}: {}", status, body);
        }

        tracing::debug!("잔고 조회 raw 응답: {}", body);

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output1: Option<Vec<BalanceItem>>,
            output2: Option<Vec<BalanceSummary>>,
        }

        let raw: Raw = serde_json::from_str(&body).map_err(|e| {
            tracing::error!(
                "잔고 JSON 파싱 실패: {}\nraw body: {}",
                e,
                &body[..body.len().min(500)]
            );
            anyhow::anyhow!("잔고 JSON 파싱 실패: {}", e)
        })?;
        if raw.rt_cd != "0" {
            anyhow::bail!("잔고 조회 오류: {}", raw.msg1);
        }

        Ok(BalanceResponse {
            items: raw.output1.unwrap_or_default(),
            summary: raw.output2.and_then(|v| v.into_iter().next()),
        })
    }

    // ──────────────────────────────────────────────────────────
    // 해외 잔고 조회
    // GET /uapi/overseas-stock/v1/trading/inquire-balance
    // TR-ID: TTTS3012R (실전) / VTTS3012R (모의)
    // ──────────────────────────────────────────────────────────
    pub async fn get_overseas_balance(&self) -> Result<OverseasBalanceResponse> {
        let tr_id = if self.is_paper {
            "VTTS3012R"
        } else {
            "TTTS3012R"
        };
        let (cano, acnt_prdt_cd) = self.split_account();

        let url = format!(
            "{}/uapi/overseas-stock/v1/trading/inquire-balance",
            self.base_url
        );
        let headers = self.auth_headers(tr_id).await?;

        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_ACCOUNT,
                self.client.get(&url).headers(headers).query(&[
                    ("CANO", cano),
                    ("ACNT_PRDT_CD", acnt_prdt_cd),
                    ("OVRS_EXCG_CD", ""),  // 빈 문자열 = 전체 거래소
                    ("TR_CRCY_CD", "USD"), // 달러 기준
                    ("CTX_AREA_FK200", ""),
                    ("CTX_AREA_NK200", ""),
                ]),
            )
            .await?;

        let status = resp.status();
        let body = read_kis_response_text(resp).await.unwrap_or_default();

        if !status.is_success() {
            anyhow::bail!("해외 잔고 조회 실패 HTTP {}: {}", status, body);
        }

        tracing::debug!("해외 잔고 조회 raw 응답: {}", body);

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output1: Option<Vec<OverseasBalanceItem>>,
            // output2는 배열이 아닌 단일 오브젝트(map)로 반환됨
            output2: Option<OverseasBalanceSummary>,
        }

        let raw: Raw = serde_json::from_str(&body).map_err(|e| {
            tracing::error!(
                "해외 잔고 JSON 파싱 실패: {}\nraw body: {}",
                e,
                &body[..body.len().min(500)]
            );
            anyhow::anyhow!("해외 잔고 JSON 파싱 실패: {}", e)
        })?;

        if raw.rt_cd != "0" {
            anyhow::bail!("해외 잔고 조회 오류: {}", raw.msg1);
        }

        Ok(OverseasBalanceResponse {
            items: raw.output1.unwrap_or_default(),
            summary: raw.output2,
        })
    }

    // ──────────────────────────────────────────────────────────
    // 주문 (매수/매도)
    // POST /uapi/domestic-stock/v1/trading/order-cash
    // ──────────────────────────────────────────────────────────
    pub async fn place_order(&self, req: &OrderRequest) -> Result<OrderResponse> {
        let tr_id = match (self.is_paper, &req.side) {
            (false, OrderSide::Buy) => "TTTC0802U",
            (false, OrderSide::Sell) => "TTTC0801U",
            (true, OrderSide::Buy) => "VTTC0802U",
            (true, OrderSide::Sell) => "VTTC0801U",
        };

        let (cano, acnt_prdt_cd) = self.split_account();
        let url = format!(
            "{}/uapi/domestic-stock/v1/trading/order-cash",
            self.base_url
        );
        let headers = self.auth_headers(tr_id).await?;

        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "CANO")]
            cano: &'a str,
            #[serde(rename = "ACNT_PRDT_CD")]
            acnt_prdt_cd: &'a str,
            #[serde(rename = "PDNO")]
            pdno: &'a str,
            #[serde(rename = "ORD_DVSN")]
            ord_dvsn: &'a str,
            #[serde(rename = "ORD_QTY")]
            ord_qty: String,
            #[serde(rename = "ORD_UNPR")]
            ord_unpr: String,
        }

        let body = Body {
            cano,
            acnt_prdt_cd,
            pdno: &req.symbol,
            ord_dvsn: req.order_type.code(),
            ord_qty: req.quantity.to_string(),
            ord_unpr: req.price.to_string(),
        };

        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_ORDER,
                self.client.post(&url).headers(headers).json(&body),
            )
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = read_kis_response_text(resp).await.unwrap_or_default();
            anyhow::bail!("주문 실패 HTTP {}: {}", status, text);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output: Option<OrderOutput>,
        }
        #[derive(Deserialize)]
        struct OrderOutput {
            odno: Option<String>,
            ord_tmd: Option<String>,
        }

        let raw: Raw = serde_json::from_str(&read_kis_response_text(resp).await?)?;
        if raw.rt_cd != "0" {
            anyhow::bail!("주문 오류: {}", raw.msg1);
        }

        let out = raw.output.unwrap_or(OrderOutput {
            odno: None,
            ord_tmd: None,
        });
        Ok(OrderResponse {
            odno: out.odno.unwrap_or_default(),
            ord_tmd: out.ord_tmd.unwrap_or_default(),
            tr_id: tr_id.to_string(),
            rt_cd: raw.rt_cd,
            msg1: raw.msg1,
        })
    }

    // ──────────────────────────────────────────────────────────
    // 당일 체결 내역 조회
    // GET /uapi/domestic-stock/v1/trading/inquire-daily-ccld
    // ──────────────────────────────────────────────────────────
    pub async fn get_today_executed_orders(&self) -> Result<Vec<ExecutedOrder>> {
        let today = chrono::Local::now().format("%Y%m%d").to_string();
        self.get_executed_orders_range(&today, &today).await
    }

    /// 몂 범위 체결 내역 (from/to: YYYYMMDD)
    pub async fn get_executed_orders_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<ExecutedOrder>> {
        let tr_id = if self.is_paper {
            "VTTC8001R"
        } else {
            "TTTC8001R"
        };
        let (cano, acnt_prdt_cd) = self.split_account();

        let url = format!(
            "{}/uapi/domestic-stock/v1/trading/inquire-daily-ccld",
            self.base_url
        );
        let headers = self.auth_headers(tr_id).await?;

        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_EXECUTION,
                self.client.get(&url).headers(headers).query(&[
                    ("CANO", cano),
                    ("ACNT_PRDT_CD", acnt_prdt_cd),
                    ("INQR_STRT_DT", from),
                    ("INQR_END_DT", to),
                    ("SLL_BUY_DVSN_CD", "00"),
                    ("INQR_DVSN", "00"),
                    ("PDNO", ""),
                    ("CCLD_DVSN", "00"), // 전체: 체결/미체결/취소 상태 reconciliation
                    ("ORD_GNO_BRNO", ""),
                    ("ODNO", ""),
                    ("INQR_DVSN_3", "00"),
                    ("INQR_DVSN_1", ""),
                    ("CTX_AREA_FK100", ""),
                    ("CTX_AREA_NK100", ""),
                ]),
            )
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = read_kis_response_text(resp).await.unwrap_or_default();
            anyhow::bail!("체결 내역 조회 실패 HTTP {}: {}", status, text);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output1: Option<Vec<ExecutedOrder>>,
        }

        let body = read_kis_response_text(resp).await?;
        if self.api_debug.load(Ordering::Relaxed) {
            tracing::info!(
                "[KIS-DEBUG][{}] 체결내역 조회 params CANO={} FROM={} TO={} 모의={}",
                tr_id,
                cano,
                from,
                to,
                self.is_paper
            );
            tracing::info!("[KIS-DEBUG][{}] 체결내역 조회 response: {}", tr_id, body);
        }
        let raw: Raw = serde_json::from_str(&body).map_err(|e| {
            anyhow::anyhow!(
                "체결 내역 JSON 파싱 오류: {} (body={})",
                e,
                &body[..body.len().min(200)]
            )
        })?;
        if raw.rt_cd != "0" {
            anyhow::bail!("체결 내역 조회 오류: {}", raw.msg1);
        }

        let orders = raw.output1.unwrap_or_default();
        tracing::debug!("체결 내역 조회: {}~{} → {}건", from, to, orders.len());
        Ok(orders)
    }

    // ──────────────────────────────────────────────────────────
    // 해외 주문체결 내역 조회
    // GET /uapi/overseas-stock/v1/trading/inquire-ccnl
    // TR-ID: TTTS3035R(실전) / VTTS3035R(모의)
    // ──────────────────────────────────────────────────────────
    pub async fn get_today_overseas_executed_orders(&self) -> Result<Vec<OverseasExecutedOrder>> {
        let today = chrono::Local::now().format("%Y%m%d").to_string();
        self.get_overseas_executed_orders_range(&today, &today)
            .await
    }

    /// 해외 주문체결 내역 날짜 범위 조회 (from/to: YYYYMMDD)
    pub async fn get_overseas_executed_orders_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<OverseasExecutedOrder>> {
        let tr_id = if self.is_paper {
            "VTTS3035R"
        } else {
            "TTTS3035R"
        };
        let (cano, acnt_prdt_cd) = self.split_account();
        let url = format!(
            "{}/uapi/overseas-stock/v1/trading/inquire-ccnl",
            self.base_url
        );

        let mut all_orders = Vec::new();
        let mut ctx_fk200 = String::new();
        let mut ctx_nk200 = String::new();
        let mut tr_cont = String::new();

        for depth in 0..10 {
            let mut headers = self.auth_headers(tr_id).await?;
            if !tr_cont.is_empty() {
                headers.insert("tr_cont", tr_cont.parse()?);
            }

            // KIS 모의투자는 해외 체결 조회 필터 조건을 거의 지원하지 않으므로 전체 조회로 고정한다.
            let pdno = "";
            let sll_buy_dvsn = "00";
            let ccld_nccs_dvsn = "00";
            let ovrs_excg_cd = "";
            let sort_sqn = if self.is_paper { "" } else { "DS" };

            let resp = self
                .send_with_rate_limit(
                    KIS_RATE_GROUP_EXECUTION,
                    self.client.get(&url).headers(headers).query(&[
                        ("CANO", cano),
                        ("ACNT_PRDT_CD", acnt_prdt_cd),
                        ("PDNO", pdno),
                        ("ORD_STRT_DT", from),
                        ("ORD_END_DT", to),
                        ("SLL_BUY_DVSN", sll_buy_dvsn),
                        ("CCLD_NCCS_DVSN", ccld_nccs_dvsn),
                        ("OVRS_EXCG_CD", ovrs_excg_cd),
                        ("SORT_SQN", sort_sqn),
                        ("ORD_DT", ""),
                        ("ORD_GNO_BRNO", ""),
                        ("ODNO", ""),
                        ("CTX_AREA_FK200", &ctx_fk200),
                        ("CTX_AREA_NK200", &ctx_nk200),
                    ]),
                )
                .await?;

            let status = resp.status();
            let response_tr_cont = resp
                .headers()
                .get("tr_cont")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            let body = read_kis_response_text(resp).await.unwrap_or_default();
            if !status.is_success() {
                anyhow::bail!("해외 체결 내역 조회 실패 HTTP {}: {}", status, body);
            }

            #[derive(Deserialize)]
            struct Raw {
                rt_cd: String,
                msg1: String,
                output: Option<Vec<OverseasExecutedOrder>>,
                ctx_area_fk200: Option<String>,
                ctx_area_nk200: Option<String>,
            }

            if self.api_debug.load(Ordering::Relaxed) {
                tracing::info!(
                    "[KIS-DEBUG][{}] 해외 체결내역 조회 params CANO={} FROM={} TO={} 모의={}",
                    tr_id,
                    cano,
                    from,
                    to,
                    self.is_paper
                );
                tracing::info!(
                    "[KIS-DEBUG][{}] 해외 체결내역 조회 response: {}",
                    tr_id,
                    body
                );
            }

            let raw: Raw = serde_json::from_str(&body).map_err(|e| {
                anyhow::anyhow!(
                    "해외 체결 내역 JSON 파싱 오류: {} (body={})",
                    e,
                    &body[..body.len().min(200)]
                )
            })?;
            if raw.rt_cd != "0" {
                anyhow::bail!("해외 체결 내역 조회 오류: {}", raw.msg1);
            }

            all_orders.extend(raw.output.unwrap_or_default());
            ctx_fk200 = raw.ctx_area_fk200.unwrap_or_default();
            ctx_nk200 = raw.ctx_area_nk200.unwrap_or_default();

            if !matches!(response_tr_cont.as_str(), "M" | "F") {
                break;
            }
            tr_cont = "N".to_string();

            if depth < 9 {
                tokio::time::sleep(tokio::time::Duration::from_millis(if self.is_paper {
                    550
                } else {
                    55
                }))
                .await;
            }
        }

        tracing::debug!(
            "해외 체결 내역 조회: {}~{} → {}건",
            from,
            to,
            all_orders.len()
        );
        Ok(all_orders)
    }
    // GET /uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice
    // TR-ID: FHKST03010100 (실전/모의 공통)
    // ──────────────────────────────────────────────────────────
    // 기간별 차트 데이터 조회
    // GET /uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice
    // TR-ID: FHKST03010100 (실전/모의 공통)
    // ──────────────────────────────────────────────────────────
    pub async fn get_chart_data(
        &self,
        symbol: &str,
        period_code: &str, // "D"=일, "W"=주, "M"=월
        start_date: &str,  // YYYYMMDD
        end_date: &str,    // YYYYMMDD
    ) -> Result<Vec<ChartCandle>> {
        let url = format!(
            "{}/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice",
            self.base_url
        );
        let headers = self.auth_headers("FHKST03010100").await?;

        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_QUOTE,
                self.client.get(&url).headers(headers).query(&[
                    ("FID_COND_MRKT_DIV_CODE", "J"),
                    ("FID_INPUT_ISCD", symbol),
                    ("FID_INPUT_DATE_1", start_date),
                    ("FID_INPUT_DATE_2", end_date),
                    ("FID_PERIOD_DIV_CODE", period_code),
                    ("FID_ORG_ADJ_PRC", "0"),
                ]),
            )
            .await?;

        let status = resp.status();
        let body = read_kis_response_text(resp).await.unwrap_or_default();

        if !status.is_success() {
            anyhow::bail!("차트 조회 HTTP {}: {}", status, body);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output2: Option<Vec<RawCandle>>,
        }

        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct RawCandle {
            stck_bsop_date: String, // 영업일자
            stck_oprc: String,      // 시가
            stck_hgpr: String,      // 고가
            stck_lwpr: String,      // 저가
            stck_clpr: String,      // 종가
            acml_vol: String,       // 누적거래량
        }

        let raw: Raw = serde_json::from_str(&body).map_err(|e| {
            tracing::error!(
                "차트 JSON 파싱 실패: {}\nbody preview: {}",
                e,
                &body[..body.len().min(300)]
            );
            anyhow::anyhow!("차트 JSON 파싱 실패: {}", e)
        })?;

        if raw.rt_cd != "0" {
            anyhow::bail!("차트 조회 오류: {}", raw.msg1);
        }

        // KIS는 최신순(내림차순) 반환 → 오름차순으로 뒤집기
        let mut candles: Vec<ChartCandle> = raw
            .output2
            .unwrap_or_default()
            .into_iter()
            .filter(|c| !c.stck_bsop_date.is_empty())
            .map(|c| ChartCandle {
                date: c.stck_bsop_date,
                open: c.stck_oprc,
                high: c.stck_hgpr,
                low: c.stck_lwpr,
                close: c.stck_clpr,
                volume: c.acml_vol,
            })
            .collect();

        candles.reverse();
        tracing::debug!(
            "차트 데이터 조회 완료: {} ({}) {} 봉",
            symbol,
            period_code,
            candles.len()
        );
        Ok(candles)
    }

    // ──────────────────────────────────────────────────────────
    // 해외 현재가 조회
    // GET /uapi/overseas-price/v1/quotations/price
    // TR-ID: HHDFS76200200 (실전/모의 공통)
    // EXCD: NAS(NASDAQ), NYS(NYSE), AMS(AMEX) 등
    // ──────────────────────────────────────────────────────────
    pub async fn get_overseas_price(
        &self,
        symbol: &str,
        exchange: &str,
    ) -> Result<OverseasPriceResponse> {
        let url = format!("{}/uapi/overseas-price/v1/quotations/price", self.base_url);
        let headers = self.auth_headers("HHDFS76200200").await?;

        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_QUOTE,
                self.client.get(&url).headers(headers).query(&[
                    ("AUTH", ""),
                    ("EXCD", exchange),
                    ("SYMB", symbol),
                ]),
            )
            .await?;

        let status = resp.status();
        let body = read_kis_response_text(resp).await.unwrap_or_default();

        if !status.is_success() {
            anyhow::bail!("해외 현재가 조회 실패 HTTP {}: {}", status, body);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output: Option<OverseasPriceResponse>,
        }

        let raw: Raw = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("해외 현재가 JSON 파싱 실패: {}", e))?;

        if raw.rt_cd != "0" {
            anyhow::bail!("해외 현재가 조회 오류: {}", raw.msg1);
        }

        raw.output
            .ok_or_else(|| anyhow::anyhow!("해외 현재가 응답 없음"))
    }

    // ──────────────────────────────────────────────────────────
    // 해외 주문 (지정가만 지원)
    // POST /uapi/overseas-stock/v1/trading/order
    // TR-ID: TTTT1002U(실전매수) TTTT1006U(실전매도)
    //         VTTT1002U(모의매수) VTTT1006U(모의매도)
    // ──────────────────────────────────────────────────────────
    pub async fn place_overseas_order(&self, req: &OverseasOrderRequest) -> Result<OrderResponse> {
        validate_overseas_order(req, self.is_paper)?;

        let tr_id = match (self.is_paper, &req.side) {
            (false, OrderSide::Buy) => "TTTT1002U",
            (false, OrderSide::Sell) => "TTTT1006U",
            (true, OrderSide::Buy) => "VTTT1002U",
            (true, OrderSide::Sell) => "VTTT1006U",
        };

        let (cano, acnt_prdt_cd) = self.split_account();
        let url = format!("{}/uapi/overseas-stock/v1/trading/order", self.base_url);
        let headers = self.auth_headers(tr_id).await?;

        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "CANO")]
            cano: &'a str,
            #[serde(rename = "ACNT_PRDT_CD")]
            acnt_prdt_cd: &'a str,
            #[serde(rename = "OVRS_EXCG_CD")]
            ovrs_excg_cd: &'a str,
            #[serde(rename = "PDNO")]
            pdno: &'a str,
            #[serde(rename = "ORD_DVSN")]
            ord_dvsn: &'static str,
            #[serde(rename = "ORD_QTY")]
            ord_qty: String,
            #[serde(rename = "OVRS_ORD_UNPR")]
            ovrs_ord_unpr: String,
            #[serde(rename = "ORD_SVR_DVSN_CD")]
            ord_svr_dvsn_cd: &'static str,
        }

        let body = Body {
            cano,
            acnt_prdt_cd,
            ovrs_excg_cd: &req.exchange,
            pdno: &req.symbol,
            ord_dvsn: "00", // 지정가
            ord_qty: req.quantity.to_string(),
            ovrs_ord_unpr: format!("{:.2}", req.price),
            ord_svr_dvsn_cd: "0",
        };

        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_ORDER,
                self.client.post(&url).headers(headers).json(&body),
            )
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = read_kis_response_text(resp).await.unwrap_or_default();
            anyhow::bail!("해외 주문 실패 HTTP {}: {}", status, text);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output: Option<OverseasOrderOutput>,
        }
        #[derive(Deserialize)]
        struct OverseasOrderOutput {
            odno: Option<String>,
            ord_tmd: Option<String>,
        }

        let raw: Raw = serde_json::from_str(&read_kis_response_text(resp).await?)?;
        if raw.rt_cd != "0" {
            anyhow::bail!("해외 주문 오류: {}", raw.msg1);
        }

        let out = raw.output.unwrap_or(OverseasOrderOutput {
            odno: None,
            ord_tmd: None,
        });
        Ok(OrderResponse {
            odno: out.odno.unwrap_or_default(),
            ord_tmd: out.ord_tmd.unwrap_or_default(),
            tr_id: tr_id.to_string(),
            rt_cd: raw.rt_cd,
            msg1: raw.msg1,
        })
    }

    // ──────────────────────────────────────────────────────────
    // 해외주식 기간별시세 (차트 데이터)
    // GET /uapi/overseas-price/v1/quotations/dailyprice
    // TR-ID: HHDFS76200200 (실전/모의 공통)
    // GUBN: 0=일별, 1=주별, 2=월별
    // BYMD: 조회 기준일 YYYYMMDD (빈 문자열이면 당일 기준)
    // MODP: 1=수정주가 반영
    // ──────────────────────────────────────────────────────────
    pub async fn get_overseas_chart_data(
        &self,
        symbol: &str,
        exchange: &str,
        period_code: &str, // "D"=일, "W"=주, "M"=월
        base_date: &str,   // YYYYMMDD — 비워두면 오늘 기준
    ) -> Result<Vec<ChartCandle>> {
        let url = format!(
            "{}/uapi/overseas-price/v1/quotations/dailyprice",
            self.base_url
        );
        let headers = self.auth_headers("HHDFS76200200").await?;

        let gubn = match period_code {
            "W" => "1",
            "M" => "2",
            _ => "0", // D
        };

        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_QUOTE,
                self.client.get(&url).headers(headers).query(&[
                    ("AUTH", ""),
                    ("EXCD", exchange),
                    ("SYMB", symbol),
                    ("GUBN", gubn),
                    ("BYMD", base_date),
                    ("MODP", "1"),
                ]),
            )
            .await?;

        let status = resp.status();
        let body = read_kis_response_text(resp).await.unwrap_or_default();

        if !status.is_success() {
            anyhow::bail!("해외 차트 조회 HTTP {}: {}", status, body);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output2: Option<Vec<RawOverseasCandle>>,
        }

        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct RawOverseasCandle {
            bass_dt: String, // 기준일자
            open: String,    // 시가
            high: String,    // 고가
            low: String,     // 저가
            clos: String,    // 종가
            tvol: String,    // 거래량
        }

        let raw: Raw = serde_json::from_str(&body).map_err(|e| {
            tracing::error!(
                "해외 차트 JSON 파싱 실패: {}\nbody: {}",
                e,
                &body[..body.len().min(300)]
            );
            anyhow::anyhow!("해외 차트 JSON 파싱 실패: {}", e)
        })?;

        if raw.rt_cd != "0" {
            anyhow::bail!("해외 차트 조회 오류: {}", raw.msg1);
        }

        // KIS는 최신순(내림차순) 반환 → 오름차순으로 뒤집기
        let mut candles: Vec<ChartCandle> = raw
            .output2
            .unwrap_or_default()
            .into_iter()
            .filter(|c| !c.bass_dt.is_empty())
            .map(|c| ChartCandle {
                date: c.bass_dt,
                open: c.open,
                high: c.high,
                low: c.low,
                close: c.clos,
                volume: c.tvol,
            })
            .collect();

        candles.reverse();
        tracing::debug!(
            "해외 차트 조회 완료: {} {} ({}) {} 봉",
            exchange,
            symbol,
            period_code,
            candles.len()
        );
        Ok(candles)
    }

    // ──────────────────────────────────────────────────────────
    // 현재가 조회
    // GET /uapi/domestic-stock/v1/quotations/inquire-price
    // ──────────────────────────────────────────────────────────
    pub async fn get_price(&self, symbol: &str) -> Result<PriceResponse> {
        let url = format!(
            "{}/uapi/domestic-stock/v1/quotations/inquire-price",
            self.base_url
        );
        let headers = self.auth_headers("FHKST01010100").await?;

        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_QUOTE,
                self.client
                    .get(&url)
                    .headers(headers)
                    .query(&[("fid_cond_mrkt_div_code", "J"), ("fid_input_iscd", symbol)]),
            )
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = read_kis_response_text(resp).await.unwrap_or_default();
            anyhow::bail!("현재가 조회 실패 HTTP {}: {}", status, text);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output: Option<PriceResponse>,
        }

        let raw: Raw = serde_json::from_str(&read_kis_response_text(resp).await?)?;
        if raw.rt_cd != "0" {
            anyhow::bail!("현재가 조회 오류: {}", raw.msg1);
        }

        raw.output
            .ok_or_else(|| anyhow::anyhow!("현재가 응답 없음"))
    }
}
