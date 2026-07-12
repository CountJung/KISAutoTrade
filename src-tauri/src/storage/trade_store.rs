use anyhow::Result;
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::{build_daily_path, read_json_or_default, write_json};
use crate::broker::{BrokerId, BrokerScope};

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
    /// provider 체결일(YYYY-MM-DD). 재시작 복구가 자정을 넘겨도 원 거래일에 귀속한다.
    #[serde(default)]
    pub execution_date: Option<String>,
    /// 체결이 발생한 broker. 기존 레코드는 None(scope 도입 전 KIS)으로 간주한다.
    #[serde(default)]
    pub broker_id: Option<BrokerId>,
    /// 체결이 발생한 broker 계좌 ID (KIS CANO-상품코드, Toss accountSeq)
    #[serde(default)]
    pub broker_account_id: Option<String>,
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
            execution_date: None,
            broker_id: None,
            broker_account_id: None,
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
            execution_date: None,
            broker_id: None,
            broker_account_id: None,
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

    /// 체결이 발생한 broker/account scope를 기록한다.
    pub fn with_broker_scope(mut self, scope: &BrokerScope) -> Self {
        self.broker_id = Some(scope.broker_id);
        self.broker_account_id = scope.account_id.as_ref().map(|account| account.0.clone());
        self
    }

    /// 이 체결이 주어진 실행 scope에 속하는지 판정한다.
    ///
    /// - scope 미기록 레거시 레코드는 scope 도입 전 KIS 체결로 간주한다.
    /// - scope에 계좌가 없으면(legacy KIS scope) broker 일치만 요구한다.
    pub fn matches_scope(&self, scope: &BrokerScope) -> bool {
        match self.broker_id {
            Some(broker_id) => {
                broker_id == scope.broker_id
                    && match &scope.account_id {
                        None => true,
                        Some(account) => self.broker_account_id.as_deref() == Some(&account.0),
                    }
            }
            None => scope.broker_id == BrokerId::Kis,
        }
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
    fn path_for(&self, date: NaiveDate) -> PathBuf {
        build_daily_path(&self.data_dir, "trades", date, "trades.json")
    }

    /// 체결 기록 저장
    pub async fn append(&self, record: TradeRecord) -> Result<()> {
        self.append_on(Local::now().date_naive(), record).await
    }

    pub async fn append_on(&self, date: NaiveDate, record: TradeRecord) -> Result<()> {
        let _write = self.write_lock.lock().await;
        let path = self.path_for(date);
        let mut records: Vec<TradeRecord> = read_json_or_default(&path).await?;
        if records.iter().any(|existing| existing.id == record.id) {
            return Ok(());
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::BrokerAccountId;

    fn record() -> TradeRecord {
        TradeRecord::new(
            "005930".to_string(),
            "삼성전자".to_string(),
            TradeSide::Sell,
            1,
            70_000,
            0,
            "ODNO-1".to_string(),
            None,
            String::new(),
        )
    }

    fn scope(broker_id: BrokerId, account: &str) -> BrokerScope {
        BrokerScope::new(broker_id, Some(BrokerAccountId(account.to_string())))
    }

    #[test]
    fn scoped_record_matches_only_same_broker_account() {
        let kis_a = scope(BrokerId::Kis, "11111111-01");
        let kis_b = scope(BrokerId::Kis, "22222222-01");
        let toss_a = scope(BrokerId::Toss, "11111111-01");
        let record = record().with_broker_scope(&kis_a);

        assert!(record.matches_scope(&kis_a));
        assert!(!record.matches_scope(&kis_b));
        assert!(!record.matches_scope(&toss_a));
    }

    #[test]
    fn account_less_scope_matches_any_account_of_same_broker() {
        let record = record().with_broker_scope(&scope(BrokerId::Kis, "11111111-01"));

        assert!(record.matches_scope(&BrokerScope::kis_legacy()));
        assert!(!record.matches_scope(&BrokerScope::new(BrokerId::Toss, None)));
    }

    #[test]
    fn legacy_record_without_scope_counts_as_kis_only() {
        let record = record();

        assert!(record.matches_scope(&BrokerScope::kis_legacy()));
        assert!(record.matches_scope(&scope(BrokerId::Kis, "11111111-01")));
        assert!(!record.matches_scope(&scope(BrokerId::Toss, "11111111-01")));
    }

    #[test]
    fn json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none() {
        let scoped = record().with_broker_scope(&scope(BrokerId::Toss, "seq-1"));
        let json = serde_json::to_string(&scoped).expect("serialize");
        let parsed: TradeRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.broker_id, Some(BrokerId::Toss));
        assert_eq!(parsed.broker_account_id.as_deref(), Some("seq-1"));

        // scope 필드가 없는 기존 JSON은 None으로 역직렬화된다.
        let mut value: serde_json::Value = serde_json::from_str(&json).expect("value");
        value.as_object_mut().map(|object| {
            object.remove("broker_id");
            object.remove("broker_account_id")
        });
        let legacy: TradeRecord =
            serde_json::from_value(value).expect("legacy deserialize");
        assert_eq!(legacy.broker_id, None);
        assert_eq!(legacy.broker_account_id, None);
    }
}
