use anyhow::Result;
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::{build_daily_path, read_json_or_default, write_json};

/// 체결 방향
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

/// 체결 상태
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TradeStatus {
    Filled,
    PartiallyFilled,
    Cancelled,
}

/// 체결 시장 구분
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TradeMarket {
    Domestic,
    Overseas,
}

fn default_trade_market() -> TradeMarket {
    TradeMarket::Domestic
}

fn default_currency() -> String {
    "KRW".to_string()
}

/// 체결 기록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: String,
    pub timestamp: String,
    pub symbol: String,
    pub symbol_name: String,
    pub side: TradeSide,
    pub quantity: u64,
    pub price: u64,
    pub total_amount: u64,
    pub fee: u64,
    pub strategy_id: Option<String>,
    pub order_id: String,
    pub status: TradeStatus,
    /// 체결 원인 — 어떤 전략 신호에 의해 매매됐는지 기록
    /// (기존 JSON 파일과의 하위 호환을 위해 default 적용)
    #[serde(default)]
    pub signal_reason: String,
    /// 국내/해외 시장 구분 (기존 JSON은 domestic으로 간주)
    #[serde(default = "default_trade_market")]
    pub market: TradeMarket,
    /// 원본 체결 통화 (국내 KRW, 해외 USD)
    #[serde(default = "default_currency")]
    pub currency: String,
    /// 해외 거래소 코드 (NASD/NYSE/AMEX)
    #[serde(default)]
    pub exchange: Option<String>,
    /// 해외 체결 단가(USD)
    #[serde(default)]
    pub price_usd: Option<f64>,
    /// 해외 체결 금액(USD)
    #[serde(default)]
    pub total_amount_usd: Option<f64>,
    /// 해외 수수료 추정액(USD)
    #[serde(default)]
    pub fee_usd: Option<f64>,
    /// 체결 시점 USD/KRW 환율
    #[serde(default)]
    pub exchange_rate_krw: Option<f64>,
    /// 체결 금액 KRW 환산
    #[serde(default)]
    pub total_amount_krw: Option<i64>,
    /// 수수료 KRW 환산
    #[serde(default)]
    pub fee_krw: Option<i64>,
    /// 매도 체결 손익(USD, 수수료 차감 전)
    #[serde(default)]
    pub realized_pnl_usd: Option<f64>,
    /// 매도 체결 손익(KRW 환산, 수수료 차감 전)
    #[serde(default)]
    pub realized_pnl_krw: Option<i64>,
    /// 신호 발생 시점 가격 (국내 KRW, 해외 USD cents)
    #[serde(default)]
    pub signal_price: Option<u64>,
    /// 주문 제출 가격 (국내 시장가 0, 해외 지정가 USD cents)
    #[serde(default)]
    pub order_price: Option<u64>,
    /// 슬리피지 비용 (국내 KRW, 해외 USD cents). 양수면 불리, 음수면 유리.
    #[serde(default)]
    pub slippage: Option<i64>,
    /// 신호가 대비 슬리피지 비용(bps). 양수면 불리, 음수면 유리.
    #[serde(default)]
    pub slippage_bps: Option<i32>,
    /// 원천 provider 식별자 (예: kis, toss)
    #[serde(default)]
    pub provider: Option<String>,
    /// provider가 반환한 주문 ID (KIS odno, Toss order id 등)
    #[serde(default)]
    pub provider_order_id: Option<String>,
    /// provider 원본 요청 추적 ID (Toss requestId, X-Request-Id 등)
    #[serde(default)]
    pub provider_request_id: Option<String>,
    /// provider 요청 TR-ID (KIS TR-ID 등)
    #[serde(default)]
    pub provider_tr_id: Option<String>,
}

impl TradeRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        symbol_name: String,
        side: TradeSide,
        quantity: u64,
        price: u64,
        fee: u64,
        order_id: String,
        strategy_id: Option<String>,
        signal_reason: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now().to_rfc3339(),
            symbol,
            symbol_name,
            side,
            quantity,
            price,
            total_amount: price * quantity,
            fee,
            strategy_id,
            order_id,
            status: TradeStatus::Filled,
            signal_reason,
            market: TradeMarket::Domestic,
            currency: "KRW".to_string(),
            exchange: None,
            price_usd: None,
            total_amount_usd: None,
            fee_usd: None,
            exchange_rate_krw: None,
            total_amount_krw: Some((price * quantity) as i64),
            fee_krw: Some(fee as i64),
            realized_pnl_usd: None,
            realized_pnl_krw: None,
            signal_price: None,
            order_price: None,
            slippage: None,
            slippage_bps: None,
            provider: None,
            provider_order_id: None,
            provider_request_id: None,
            provider_tr_id: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_overseas(
        symbol: String,
        symbol_name: String,
        side: TradeSide,
        quantity: u64,
        price_cents: u64,
        fee_cents: u64,
        order_id: String,
        strategy_id: Option<String>,
        signal_reason: String,
        exchange: String,
        exchange_rate_krw: f64,
        realized_pnl_cents: Option<i64>,
    ) -> Self {
        let price_usd = price_cents as f64 / 100.0;
        let total_amount_usd = price_usd * quantity as f64;
        let fee_usd = fee_cents as f64 / 100.0;
        let realized_pnl_usd = realized_pnl_cents.map(|v| v as f64 / 100.0);
        let to_krw = |usd: f64| (usd * exchange_rate_krw).round() as i64;

        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now().to_rfc3339(),
            symbol,
            symbol_name,
            side,
            quantity,
            price: price_cents,
            total_amount: price_cents * quantity,
            fee: fee_cents,
            strategy_id,
            order_id,
            status: TradeStatus::Filled,
            signal_reason,
            market: TradeMarket::Overseas,
            currency: "USD".to_string(),
            exchange: Some(exchange),
            price_usd: Some(price_usd),
            total_amount_usd: Some(total_amount_usd),
            fee_usd: Some(fee_usd),
            exchange_rate_krw: Some(exchange_rate_krw),
            total_amount_krw: Some(to_krw(total_amount_usd)),
            fee_krw: Some(to_krw(fee_usd).max(0)),
            realized_pnl_usd,
            realized_pnl_krw: realized_pnl_usd.map(to_krw),
            signal_price: None,
            order_price: None,
            slippage: None,
            slippage_bps: None,
            provider: None,
            provider_order_id: None,
            provider_request_id: None,
            provider_tr_id: None,
        }
    }

    /// 신호가/주문가/체결가 기반 슬리피지 분석 필드를 채운다.
    pub fn with_execution_prices(mut self, signal_price: u64, order_price: u64) -> Self {
        let slippage = match self.side {
            TradeSide::Buy => self.price as i64 - signal_price as i64,
            TradeSide::Sell => signal_price as i64 - self.price as i64,
        };
        self.signal_price = Some(signal_price);
        self.order_price = Some(order_price);
        self.slippage = Some(slippage);
        self.slippage_bps = if signal_price > 0 {
            Some(((slippage as f64 / signal_price as f64) * 10_000.0).round() as i32)
        } else {
            None
        };
        self
    }

    pub fn with_provider_trace(
        mut self,
        provider: Option<String>,
        order_id: Option<String>,
        request_id: Option<String>,
        tr_id: Option<String>,
    ) -> Self {
        self.provider = provider;
        self.provider_order_id = order_id;
        self.provider_request_id = request_id;
        self.provider_tr_id = tr_id;
        self
    }
}

/// 체결 기록 저장소
pub struct TradeStore {
    data_dir: PathBuf,
    write_lock: Mutex<()>,
}

impl TradeStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            write_lock: Mutex::new(()),
        }
    }

    /// 오늘의 체결 기록 파일 경로
    fn today_path(&self) -> PathBuf {
        build_daily_path(
            &self.data_dir,
            "trades",
            Local::now().date_naive(),
            "trades.json",
        )
    }

    /// 체결 기록 저장
    pub async fn append(&self, record: TradeRecord) -> Result<()> {
        let _write = self.write_lock.lock().await;
        let path = self.today_path();
        let mut records: Vec<TradeRecord> = read_json_or_default(&path).await?;
        records.push(record);
        write_json(&path, &records).await?;
        Ok(())
    }

    /// 특정 날짜의 체결 기록 조회
    pub async fn get_by_date(&self, date: NaiveDate) -> Result<Vec<TradeRecord>> {
        let path = build_daily_path(&self.data_dir, "trades", date, "trades.json");
        read_json_or_default(&path).await
    }

    /// 날짜 범위 체결 기록 조회
    pub async fn get_by_range(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<TradeRecord>> {
        let mut all = Vec::new();
        let mut current = from;
        while current <= to {
            let mut records = self.get_by_date(current).await?;
            all.append(&mut records);
            current = current.succ_opt().unwrap_or(current);
            if current == from {
                break; // 무한루프 방지
            }
        }
        Ok(all)
    }
}
