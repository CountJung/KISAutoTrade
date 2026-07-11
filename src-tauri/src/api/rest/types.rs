use anyhow::Result;
use serde::{Deserialize, Serialize};

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
    /// 예수금총금액 (D+0, 매수 당일 결제 전 음수 가능)
    pub dnca_tot_amt: String,
    /// 익일정산금액 (D+1 예수금)
    pub nxdy_excc_amt: String,
    /// 가수도정산금액 (D+2 예수금, 실제 인출·매매 가능 현금)
    pub prvs_rcdl_excc_amt: String,
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
            nxdy_excc_amt: String::from("0"),
            prvs_rcdl_excc_amt: String::from("0"),
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
// 해외 잔고 조회
// ────────────────────────────────────────────────────────────────────

/// 해외 잔고 응답 (1건, TTTS3012R output1)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OverseasBalanceItem {
    /// 종목코드 (티커)
    pub ovrs_pdno: String,
    /// 종목명
    pub ovrs_item_name: String,
    /// 해외잔고수량
    pub ovrs_cblc_qty: String,
    /// 매입평균가격 (USD)
    pub pchs_avg_pric: String,
    /// 현재가 (USD)
    pub now_pric2: String,
    /// 해외주식평가금액 (USD)
    pub ovrs_stck_evlu_amt: String,
    /// 외화평가손익금액 (USD)
    pub frcr_evlu_pfls_amt: String,
    /// 평가손익율 (%)
    pub evlu_pfls_rt: String,
    /// 거래소코드 (NAS/NYS/AMS 등)
    pub ovrs_excg_cd: String,
    /// 시장명
    pub tr_mket_name: String,
}

impl Default for OverseasBalanceItem {
    fn default() -> Self {
        Self {
            ovrs_pdno: String::new(),
            ovrs_item_name: String::new(),
            ovrs_cblc_qty: String::from("0"),
            pchs_avg_pric: String::from("0"),
            now_pric2: String::from("0"),
            ovrs_stck_evlu_amt: String::from("0"),
            frcr_evlu_pfls_amt: String::from("0"),
            evlu_pfls_rt: String::from("0"),
            ovrs_excg_cd: String::new(),
            tr_mket_name: String::new(),
        }
    }
}

/// 해외 잔고 요약 (TTTS3012R output2)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OverseasBalanceSummary {
    /// 외화매입금액합계 (USD)
    pub frcr_pchs_amt1: String,
    /// 해외총손익 (USD)
    pub ovrs_tot_pfls: String,
    /// 외화평가금액합계 (USD)
    pub frcr_evlu_tota: String,
    /// 총수익률 (%)
    pub tot_pftrt: String,
}

impl Default for OverseasBalanceSummary {
    fn default() -> Self {
        Self {
            frcr_pchs_amt1: String::from("0"),
            ovrs_tot_pfls: String::from("0"),
            frcr_evlu_tota: String::from("0"),
            tot_pftrt: String::from("0"),
        }
    }
}

/// 해외 잔고 전체 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverseasBalanceResponse {
    pub items: Vec<OverseasBalanceItem>,
    pub summary: Option<OverseasBalanceSummary>,
}

// ────────────────────────────────────────────────────────────────────
// 차트 (기간별 시세)
// ────────────────────────────────────────────────────────────────────

/// 차트 캔들 1개 (일/주/월봉 또는 provider intraday timestamp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartCandle {
    /// 영업일자 YYYYMMDD 또는 provider intraday timestamp
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
    pub(super) fn code(&self) -> &'static str {
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
    /// 주문 요청에 사용한 provider TR-ID
    pub tr_id: String,
    pub rt_cd: String,
    pub msg1: String,
}

// ────────────────────────────────────────────────────────────────────
// 체결 내역 조회
// ────────────────────────────────────────────────────────────────────

/// 체결 내역 1건
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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
    /// 취소여부 / 취소확인수량 / 잔여수량 / 거부수량
    pub cncl_yn: String,
    pub cnc_cfrm_qty: String,
    pub rmn_qty: String,
    pub rjct_qty: String,
}

/// 해외 주문체결 내역 1건 (TTTS3035R / VTTS3035R output)
///
/// KIS 해외 체결 응답은 필드가 상품/환경별로 일부 비어 있을 수 있어 문자열 기본값으로
/// 넓게 수용한다. 자동 체결 확인에는 `odno`, 체결수량, 체결단가/체결금액만 사용한다.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OverseasExecutedOrder {
    /// 상품번호 / 티커
    pub pdno: String,
    /// 상품명
    pub prdt_name: String,
    /// 해외거래소코드
    pub ovrs_excg_cd: String,
    /// 매도매수구분
    pub sll_buy_dvsn: String,
    /// 체결/미체결 구분
    pub ccld_nccs_dvsn: String,
    /// 주문수량
    pub ft_ord_qty: String,
    /// 해외주문단가
    pub ft_ord_unpr3: String,
    /// 체결수량
    pub ft_ccld_qty: String,
    /// 체결단가
    pub ft_ccld_unpr3: String,
    /// 체결금액
    pub ft_ccld_amt3: String,
    /// 주문번호
    pub odno: String,
    /// 주문일자
    pub ord_dt: String,
    /// 주문시각
    pub ord_tmd: String,
    /// 주문채번지점번호
    pub ord_gno_brno: String,
    /// 미체결수량
    pub nccs_qty: String,
    /// 처리상태명
    pub prcs_stat_name: String,
    /// 거부사유
    pub rjct_rson: String,
}

impl OverseasExecutedOrder {
    pub fn filled_qty(&self) -> u64 {
        first_positive_u64(&[&self.ft_ccld_qty])
    }

    pub fn avg_price_cents(&self) -> u64 {
        let qty = self.filled_qty();
        let amount_cents = parse_decimal_cents(&self.ft_ccld_amt3);
        if qty > 0 && amount_cents > 0 {
            return amount_cents / qty;
        }
        parse_decimal_cents(&self.ft_ccld_unpr3).max(parse_decimal_cents(&self.ft_ord_unpr3))
    }

    pub fn is_terminal(&self) -> bool {
        let ordered = first_positive_u64(&[&self.ft_ord_qty]);
        let filled = self.filled_qty();
        let remaining = first_positive_u64(&[&self.nccs_qty]);
        let status = self.prcs_stat_name.trim().to_ascii_lowercase();
        let explicit_terminal = matches!(
            status.as_str(),
            "체결완료"
                | "취소완료"
                | "주문취소"
                | "주문거부"
                | "거부"
                | "filled"
                | "canceled"
                | "cancelled"
                | "rejected"
                | "expired"
        );
        (ordered > 0 && filled >= ordered)
            || (remaining == 0
                && (explicit_terminal || filled > 0 || !self.rjct_rson.trim().is_empty()))
    }
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
    /// 시장구분: "KOSPI" | "KOSDAQ" | "KONEX" | "US" (없으면 None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
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

const PAPER_UNSUPPORTED_SELL_SYMBOLS: &[&str] = &["VOO", "SPY", "QQQM"];

pub(super) fn validate_overseas_order(req: &OverseasOrderRequest, is_paper: bool) -> Result<()> {
    let symbol = req.symbol.trim().to_uppercase();
    let exchange = req.exchange.trim().to_uppercase();

    if symbol.is_empty() {
        anyhow::bail!("해외 주문 사전 검증 실패: 종목코드가 비어 있습니다.");
    }
    if !matches!(exchange.as_str(), "NASD" | "NYSE" | "AMEX") {
        anyhow::bail!(
            "해외 주문 사전 검증 실패: 지원하지 않는 거래소 코드입니다: {}",
            req.exchange
        );
    }
    if req.quantity == 0 {
        anyhow::bail!("해외 주문 사전 검증 실패: 주문 수량은 1 이상이어야 합니다.");
    }
    if !req.price.is_finite() || req.price <= 0.0 {
        anyhow::bail!("해외 주문 사전 검증 실패: 해외 주문은 0보다 큰 USD 지정가가 필요합니다.");
    }

    let paper_sell_limited = is_paper
        && matches!(req.side, OrderSide::Sell)
        && (exchange == "AMEX" || PAPER_UNSUPPORTED_SELL_SYMBOLS.contains(&symbol.as_str()));
    if paper_sell_limited {
        anyhow::bail!(
            "PAPER_OVERSEAS_UNSUPPORTED: 모의투자 미지원 사전 검증 - {} ({}) 매도 주문은 KIS 모의투자 해외 주문 universe에서 제한됩니다. 해당업무가 제공되지 않습니다. 실전투자로 전환하거나 NASD/NYSE 지원 종목으로 검증하세요.",
            symbol,
            exchange
        );
    }

    Ok(())
}

fn parse_decimal_cents(value: &str) -> u64 {
    let clean = value.trim().replace(',', "");
    if clean.is_empty() {
        return 0;
    }
    clean
        .parse::<f64>()
        .ok()
        .map(|v| (v * 100.0).round() as u64)
        .unwrap_or(0)
}

fn first_positive_u64(values: &[&str]) -> u64 {
    values
        .iter()
        .filter_map(|v| v.trim().replace(',', "").parse::<u64>().ok())
        .find(|v| *v > 0)
        .unwrap_or(0)
}
