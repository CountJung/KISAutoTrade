use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::token::TokenManager;

/// 한국투자증권 REST API 클라이언트
pub struct KisRestClient {
    client: Client,
    base_url: String,
    app_key: String,
    app_secret: String,
    account_no: String,
    is_paper: bool,
    token_manager: Arc<RwLock<TokenManager>>,
}

// ────────────────────────────────────────────────────────────────────
// 공통 응답
// ────────────────────────────────────────────────────────────────────

/// KIS 공통 응답 코드 "0" = 성공
#[derive(Debug, Deserialize)]
pub struct KisOutput1<T> {
    pub rt_cd: String,
    pub msg_cd: String,
    pub msg1: String,
    pub output1: Option<T>,
    pub output2: Option<serde_json::Value>,
}

impl<T> KisOutput1<T> {
    pub fn ok(&self) -> bool {
        self.rt_cd == "0"
    }
}

// ────────────────────────────────────────────────────────────────────
// 잔고 조회
// ────────────────────────────────────────────────────────────────────

/// 잔고 응답 (1건)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BalanceItem {
    /// 종목코드
    pub pdno: String,
    /// 종목명
    pub prdt_name: String,
    /// 보유수량
    pub hldg_qty: String,
    /// 매입평균가격
    pub pchs_avg_pric: String,
    /// 현재가
    pub prpr: String,
    /// 평가손익금액
    pub evlu_pfls_amt: String,
    /// 평가손익율
    pub evlu_pfls_rt: String,
}

impl Default for BalanceItem {
    fn default() -> Self {
        Self {
            pdno: String::new(),
            prdt_name: String::new(),
            hldg_qty: String::from("0"),
            pchs_avg_pric: String::from("0"),
            prpr: String::from("0"),
            evlu_pfls_amt: String::from("0"),
            evlu_pfls_rt: String::from("0"),
        }
    }
}

/// 잔고 요약 (output2)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BalanceSummary {
    /// 예수금총금액
    pub dnca_tot_amt: String,
    /// 총평가금액
    pub tot_evlu_amt: String,
    /// 순자산금액
    pub nass_amt: String,
    /// 총수익율
    pub tot_evlu_pfls_rt: String,
}

impl Default for BalanceSummary {
    fn default() -> Self {
        Self {
            dnca_tot_amt: String::from("0"),
            tot_evlu_amt: String::from("0"),
            nass_amt: String::from("0"),
            tot_evlu_pfls_rt: String::from("0"),
        }
    }
}

/// 잔고 전체 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub items: Vec<BalanceItem>,
    pub summary: Option<BalanceSummary>,
}

// ────────────────────────────────────────────────────────────────────
// 차트 (기간별 시세)
// ────────────────────────────────────────────────────────────────────

/// 차트 캔들 1개 (일/주/월봉)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartCandle {
    /// 영업일자 YYYYMMDD
    pub date: String,
    /// 시가
    pub open: String,
    /// 고가
    pub high: String,
    /// 저가
    pub low: String,
    /// 종가
    pub close: String,
    /// 누적거래량
    pub volume: String,
}

// ────────────────────────────────────────────────────────────────────
// 주문
// ────────────────────────────────────────────────────────────────────

/// 주문 방향
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// 주문 유형
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    /// 지정가
    Limit,
    /// 시장가
    Market,
}

impl OrderType {
    fn code(&self) -> &'static str {
        match self {
            OrderType::Limit => "00",
            OrderType::Market => "01",
        }
    }
}

/// 주문 요청
#[derive(Debug, Serialize)]
pub struct OrderRequest {
    /// 종목코드 (6자리)
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: u64,
    /// 지정가 (시장가일 때 0)
    pub price: u64,
}

/// 주문 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    /// 주문번호
    pub odno: String,
    /// 주문시각
    pub ord_tmd: String,
    pub rt_cd: String,
    pub msg1: String,
}

// ────────────────────────────────────────────────────────────────────
// 체결 내역 조회
// ────────────────────────────────────────────────────────────────────

/// 체결 내역 1건
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutedOrder {
    /// 종목코드
    pub pdno: String,
    /// 종목명
    pub prdt_name: String,
    /// 매도매수구분코드 "01"=매도, "02"=매수
    pub sll_buy_dvsn_cd: String,
    /// 주문수량
    pub ord_qty: String,
    /// 주문단가
    pub ord_unpr: String,
    /// 체결수량
    pub tot_ccld_qty: String,
    /// 체결금액
    pub tot_ccld_amt: String,
    /// 주문번호
    pub odno: String,
    /// 주문일자
    pub ord_dt: String,
    /// 주문시각
    pub ord_tmd: String,
}

// ────────────────────────────────────────────────────────────────────
// 현재가 조회
// ────────────────────────────────────────────────────────────────────

/// 현재가 응답
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PriceResponse {
    /// 현재가
    pub stck_prpr: String,
    /// 전일대비
    pub prdy_vrss: String,
    /// 전일대비율
    pub prdy_ctrt: String,
    /// 거래량
    pub acml_vol: String,
    /// 종목명
    pub hts_kor_isnm: String,
    /// 시가
    pub stck_oprc: String,
    /// 고가
    pub stck_hgpr: String,
    /// 저가
    pub stck_lwpr: String,
    /// 상한가
    pub stck_mxpr: String,
    /// 하한가
    pub stck_llam: String,
    /// 52주 최고가
    pub w52_hgpr: String,
    /// 52주 최저가
    pub w52_lwpr: String,
}

// ────────────────────────────────────────────────────────────────────
// 종목 검색
// ────────────────────────────────────────────────────────────────────

/// 종목 검색 결과 1건
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StockSearchItem {
    /// 종목코드
    pub pdno: String,
    /// 종목명 (prdt_abrv_name 필드도 수용)
    #[serde(alias = "prdt_abrv_name")]
    pub prdt_name: String,
}

// ────────────────────────────────────────────────────────────────────
// 해외 현재가
// ────────────────────────────────────────────────────────────────────

/// 해외 현재가 응답 (KIS HHDFS76200200)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OverseasPriceResponse {
    /// 현지 현재가 (USD 등)
    pub last: String,
    /// 전일대비
    pub diff: String,
    /// 등락률 (%)
    pub rate: String,
    /// 거래량
    pub tvol: String,
    /// 종목명
    pub name: String,
    /// 시가
    pub open: String,
    /// 고가
    pub high: String,
    /// 저가
    pub low: String,
    /// 52주 최고
    pub h52p: String,
    /// 52주 최저
    pub l52p: String,
    /// 거래소 심볼 (rsym)
    pub rsym: String,
}

// ────────────────────────────────────────────────────────────────────
// 해외 주문
// ────────────────────────────────────────────────────────────────────

/// 해외 주문 요청
#[derive(Debug, Serialize)]
pub struct OverseasOrderRequest {
    /// 티커 (AAPL 등)
    pub symbol: String,
    /// KIS 거래소 코드 (NASD / NYSE / AMEX)
    pub exchange: String,
    pub side: OrderSide,
    pub quantity: u64,
    /// USD 가격 (해외는 지정가만 지원)
    pub price: f64,
}

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
        Self {
            client: Client::new(),
            base_url,
            app_key,
            app_secret,
            account_no,
            is_paper,
            token_manager,
        }
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
        let tr_id = if self.is_paper { "VTTC8434R" } else { "TTTC8434R" };
        let (cano, acnt_prdt_cd) = self.split_account();

        let url = format!("{}/uapi/domestic-stock/v1/trading/inquire-balance", self.base_url);
        let headers = self.auth_headers(tr_id).await?;

        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .query(&[
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
            ])
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

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
            tracing::error!("잔고 JSON 파싱 실패: {}\nraw body: {}", e, &body[..body.len().min(500)]);
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
        let url = format!("{}/uapi/domestic-stock/v1/trading/order-cash", self.base_url);
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
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
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

        let raw: Raw = resp.json().await?;
        if raw.rt_cd != "0" {
            anyhow::bail!("주문 오류: {}", raw.msg1);
        }

        let out = raw.output.unwrap_or(OrderOutput { odno: None, ord_tmd: None });
        Ok(OrderResponse {
            odno: out.odno.unwrap_or_default(),
            ord_tmd: out.ord_tmd.unwrap_or_default(),
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
    pub async fn get_executed_orders_range(&self, from: &str, to: &str) -> Result<Vec<ExecutedOrder>> {
        let tr_id = if self.is_paper { "VTTC8001R" } else { "TTTC8001R" };
        let (cano, acnt_prdt_cd) = self.split_account();

        let url = format!(
            "{}/uapi/domestic-stock/v1/trading/inquire-daily-ccld",
            self.base_url
        );
        let headers = self.auth_headers(tr_id).await?;

        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .query(&[
                ("CANO", cano),
                ("ACNT_PRDT_CD", acnt_prdt_cd),
                ("INQR_STRT_DT", from),
                ("INQR_END_DT", to),
                ("SLL_BUY_DVSN_CD", "00"),
                ("INQR_DVSN", "00"),
                ("PDNO", ""),
                ("CCLD_DVSN", "01"),
                ("ORD_GNO_BRNO", ""),
                ("ODNO", ""),
                ("INQR_DVSN_3", "00"),
                ("INQR_DVSN_1", ""),
                ("CTX_AREA_FK100", ""),
                ("CTX_AREA_NK100", ""),
            ])
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("체결 내역 조회 실패 HTTP {}: {}", status, text);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output1: Option<Vec<ExecutedOrder>>,
        }

        let raw: Raw = resp.json().await?;
        if raw.rt_cd != "0" {
            anyhow::bail!("체결 내역 조회 오류: {}", raw.msg1);
        }

        Ok(raw.output1.unwrap_or_default())
    }

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
            .client
            .get(&url)
            .headers(headers)
            .query(&[
                ("FID_COND_MRKT_DIV_CODE", "J"),
                ("FID_INPUT_ISCD", symbol),
                ("FID_INPUT_DATE_1", start_date),
                ("FID_INPUT_DATE_2", end_date),
                ("FID_PERIOD_DIV_CODE", period_code),
                ("FID_ORG_ADJ_PRC", "0"),
            ])
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

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
            tracing::error!("차트 JSON 파싱 실패: {}\nbody preview: {}", e, &body[..body.len().min(300)]);
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
        tracing::debug!("차트 데이터 조회 완료: {} ({}) {} 봉", symbol, period_code, candles.len());
        Ok(candles)
    }

    // ──────────────────────────────────────────────────────────
    // 해외 현재가 조회
    // GET /uapi/overseas-price/v1/quotations/price
    // TR-ID: HHDFS76200200 (실전/모의 공통)
    // EXCD: NAS(NASDAQ), NYS(NYSE), AMS(AMEX) 등
    // ──────────────────────────────────────────────────────────
    pub async fn get_overseas_price(&self, symbol: &str, exchange: &str) -> Result<OverseasPriceResponse> {
        let url = format!("{}/uapi/overseas-price/v1/quotations/price", self.base_url);
        let headers = self.auth_headers("HHDFS76200200").await?;

        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .query(&[("AUTH", ""), ("EXCD", exchange), ("SYMB", symbol)])
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            anyhow::bail!("해외 현재가 조회 실패 HTTP {}: {}", status, body);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output: Option<OverseasPriceResponse>,
        }

        let raw: Raw = serde_json::from_str(&body).map_err(|e| {
            anyhow::anyhow!("해외 현재가 JSON 파싱 실패: {}", e)
        })?;

        if raw.rt_cd != "0" {
            anyhow::bail!("해외 현재가 조회 오류: {}", raw.msg1);
        }

        raw.output.ok_or_else(|| anyhow::anyhow!("해외 현재가 응답 없음"))
    }

    // ──────────────────────────────────────────────────────────
    // 해외 주문 (지정가만 지원)
    // POST /uapi/overseas-stock/v1/trading/order
    // TR-ID: TTTT1002U(실전매수) TTTT1006U(실전매도)
    //         VTTT1002U(모의매수) VTTT1006U(모의매도)
    // ──────────────────────────────────────────────────────────
    pub async fn place_overseas_order(&self, req: &OverseasOrderRequest) -> Result<OrderResponse> {
        let tr_id = match (self.is_paper, &req.side) {
            (false, OrderSide::Buy)  => "TTTT1002U",
            (false, OrderSide::Sell) => "TTTT1006U",
            (true,  OrderSide::Buy)  => "VTTT1002U",
            (true,  OrderSide::Sell) => "VTTT1006U",
        };

        let (cano, acnt_prdt_cd) = self.split_account();
        let url = format!("{}/uapi/overseas-stock/v1/trading/order", self.base_url);
        let headers = self.auth_headers(tr_id).await?;

        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "CANO")]           cano: &'a str,
            #[serde(rename = "ACNT_PRDT_CD")]   acnt_prdt_cd: &'a str,
            #[serde(rename = "OVRS_EXCG_CD")]   ovrs_excg_cd: &'a str,
            #[serde(rename = "PDNO")]            pdno: &'a str,
            #[serde(rename = "ORD_DVSN")]        ord_dvsn: &'static str,
            #[serde(rename = "ORD_QTY")]         ord_qty: String,
            #[serde(rename = "OVRS_ORD_UNPR")]   ovrs_ord_unpr: String,
            #[serde(rename = "ORD_SVR_DVSN_CD")] ord_svr_dvsn_cd: &'static str,
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
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
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

        let raw: Raw = resp.json().await?;
        if raw.rt_cd != "0" {
            anyhow::bail!("해외 주문 오류: {}", raw.msg1);
        }

        let out = raw.output.unwrap_or(OverseasOrderOutput { odno: None, ord_tmd: None });
        Ok(OrderResponse {
            odno: out.odno.unwrap_or_default(),
            ord_tmd: out.ord_tmd.unwrap_or_default(),
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
            _   => "0", // D
        };

        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .query(&[
                ("AUTH", ""),
                ("EXCD", exchange),
                ("SYMB", symbol),
                ("GUBN", gubn),
                ("BYMD", base_date),
                ("MODP", "1"),
            ])
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

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
            open:    String, // 시가
            high:    String, // 고가
            low:     String, // 저가
            clos:    String, // 종가
            tvol:    String, // 거래량
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
                date:   c.bass_dt,
                open:   c.open,
                high:   c.high,
                low:    c.low,
                close:  c.clos,
                volume: c.tvol,
            })
            .collect();

        candles.reverse();
        tracing::debug!(
            "해외 차트 조회 완료: {} {} ({}) {} 봉",
            exchange, symbol, period_code, candles.len()
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
            .client
            .get(&url)
            .headers(headers)
            .query(&[("fid_cond_mrkt_div_code", "J"), ("fid_input_iscd", symbol)])
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("현재가 조회 실패 HTTP {}: {}", status, text);
        }

        #[derive(Deserialize)]
        struct Raw {
            rt_cd: String,
            msg1: String,
            output: Option<PriceResponse>,
        }

        let raw: Raw = resp.json().await?;
        if raw.rt_cd != "0" {
            anyhow::bail!("현재가 조회 오류: {}", raw.msg1);
        }

        raw.output.ok_or_else(|| anyhow::anyhow!("현재가 응답 없음"))
    }
}

