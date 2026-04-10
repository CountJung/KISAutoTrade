/// 주문 관리자 — KISAutoTrade 핵심 실행 모듈
///
/// 역할:
///  ① 전략 신호(Signal::Buy/Sell) → KIS API place_order() 실행
///  ② 미체결 주문 풀(HashMap) 유지 (odno → PendingOrder)
///  ③ 동일 종목 중복 주문 방지 (symbol_to_odno 맵)
///  ④ 체결 이벤트(on_fill) 처리 → 미체결 풀 상태 갱신
///     (WebSocket H0STCNI0 수신 또는 폴링에서 호출)
///  ⑤ 체결 확인 시 PositionTracker.on_buy/on_sell() 연동
///  ⑥ 주문 기록을 OrderStore JSON에 저장
///  ⑦ 매도 체결 손익 → StatsStore & RiskManager 반영
///  ⑧ 주문 전 RiskManager.can_trade() + check_position_size() 검증
///  ⑨ 체결 완료 시 Discord 알림 전송
///  ⑩ KIS rate-limit(EGW00133) 오류 시 1초 대기 × 최대 3회 재시도
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{Mutex, RwLock};

use crate::{
    api::rest::{
        KisRestClient, OrderRequest, OrderResponse, OrderSide as RestOrderSide, OrderType,
        OverseasOrderRequest,
    },
    notifications::{discord::DiscordNotifier, types::NotificationEvent},
    storage::{
        order_store::{OrderRecord, OrderSide, OrderStatus},
        trade_store::{TradeRecord, TradeSide},
        OrderStore, StatsStore, TradeStore,
    },
    trading::{position::PositionTracker, risk::RiskManager, strategy::Signal},
};

// ────────────────────────────────────────────────────────────────────
// PendingOrder — 미체결 주문 항목
// ────────────────────────────────────────────────────────────────────

/// 미체결(Pending) 상태 주문 항목
#[derive(Debug, Clone)]
pub struct PendingOrder {
    /// 원본 주문 기록 (OrderStore에 Pending 상태로 기록됨)
    pub record: OrderRecord,
    /// 이 주문을 촉발한 전략 신호 이유
    pub signal_reason: String,
}

// ────────────────────────────────────────────────────────────────────
// OrderManager
// ────────────────────────────────────────────────────────────────────

pub struct OrderManager {
    // ── ② 미체결 주문 풀 ────────────────────────────────────────────
    /// KIS odno → PendingOrder
    pending: HashMap<String, PendingOrder>,

    // ── ③ 중복 방지 인덱스 ─────────────────────────────────────────
    /// symbol → odno (미체결 주문이 있는 종목)
    symbol_to_odno: HashMap<String, String>,

    // ── ⑪ 잔고 부족 매수 정지 ────────────────────────────────────
    /// true = KIS 잔고부족 응답 수신 후 매수 일시 정지.
    /// 매도 체결 또는 수동 해제 시 false로 전환됨.
    pub buy_suspended: bool,
    /// 매수 정지 사유 (KIS 응답 msg1)
    pub buy_suspended_reason: Option<String>,

    // ── 공유 의존성 (Arc) ───────────────────────────────────────────
    /// KIS REST 클라이언트 (프로파일 전환 시 내부 Arc만 교체됨)
    rest_client: Arc<RwLock<Arc<KisRestClient>>>,
    order_store: Arc<OrderStore>,
    trade_store: Arc<TradeStore>,
    position_tracker: Arc<Mutex<PositionTracker>>,
    stats_store: Arc<StatsStore>,
    /// 리스크 관리자 — AppState에서 Arc 공유
    pub risk_manager: Arc<Mutex<RiskManager>>,
    discord: Option<Arc<DiscordNotifier>>,
}

impl OrderManager {
    pub fn new(
        rest_client: Arc<RwLock<Arc<KisRestClient>>>,
        order_store: Arc<OrderStore>,
        trade_store: Arc<TradeStore>,
        position_tracker: Arc<Mutex<PositionTracker>>,
        stats_store: Arc<StatsStore>,
        risk_manager: Arc<Mutex<RiskManager>>,
        discord: Option<Arc<DiscordNotifier>>,
    ) -> Self {
        Self {
            pending: HashMap::new(),
            symbol_to_odno: HashMap::new(),
            buy_suspended: false,
            buy_suspended_reason: None,
            rest_client,
            order_store,
            trade_store,
            position_tracker,
            stats_store,
            risk_manager,
            discord,
        }
    }

    // ── 공개 API ────────────────────────────────────────────────────

    /// ① 전략 신호 처리 → 주문 실행
    ///
    /// - `symbol_name`: 한국어 종목명 (PositionTracker, 알림에 사용)
    /// - `total_balance`: 총 잔고(원) — 0이면 포지션 비중 검사 skip
    /// - `exchange`: None = 국내, Some("NAS"/"NYS"/"AMS") = 해외
    /// - `tick_price`: 현재가 (국내 = 원, 해외 = USD × 100)
    pub async fn submit_signal(
        &mut self,
        signal: Signal,
        symbol_name: &str,
        total_balance: i64,
        exchange: Option<String>,
        tick_price: u64,
    ) -> Result<()> {
        match signal {
            Signal::Buy { symbol, quantity, reason } => {
                self.process_buy(symbol, symbol_name.to_string(), quantity, reason, total_balance, exchange, tick_price)
                    .await
            }
            Signal::Sell { symbol, quantity, reason } => {
                self.process_sell(symbol, symbol_name.to_string(), quantity, reason, exchange, tick_price)
                    .await
            }
            Signal::Hold => Ok(()),
        }
    }

    /// ④ 체결 이벤트 처리 (WebSocket H0STCNI0 또는 폴링에서 호출)
    ///
    /// - `odno`: KIS 주문번호
    /// - `filled_qty`: 체결 수량
    /// - `avg_price`: 체결 평균가(원)
    pub async fn on_fill(&mut self, odno: &str, filled_qty: u64, avg_price: u64) -> Result<()> {
        let Some(pending) = self.pending.remove(odno) else {
            tracing::debug!("on_fill: odno {} 는 미체결 풀에 없음 (이미 처리됨)", odno);
            return Ok(());
        };

        let symbol = pending.record.symbol.clone();
        let symbol_name = pending.record.symbol_name.clone();

        // ③ 중복 방지 인덱스 정리
        self.symbol_to_odno.remove(&symbol);

        // ⑤ 포지션 연동 + ⑦ 매도 시 PnL 계산 (포지션 업데이트 전에 avg_price 읽기)
        let pnl = {
            let mut tracker = self.position_tracker.lock().await;
            match &pending.record.side {
                OrderSide::Buy => {
                    tracker.on_buy(symbol.clone(), symbol_name.clone(), filled_qty, avg_price);
                    tracing::info!("매수 체결: {} {} @ {}원", symbol, filled_qty, avg_price);
                    0i64
                }
                OrderSide::Sell => {
                    // 매도 전 매입 평균가 조회 (PnL 계산 후 포지션 감소)
                    let buy_avg = tracker.get(&symbol).map(|p| p.avg_price).unwrap_or(0.0);
                    let realized = (avg_price as f64 - buy_avg) * filled_qty as f64;
                    tracker.on_sell(&symbol, filled_qty);
                    tracing::info!(
                        "매도 체결: {} {} @ {}원 (PnL: {}원)",
                        symbol,
                        filled_qty,
                        avg_price,
                        realized as i64
                    );
                    realized as i64
                }
            }
        };

        // ⑦ 매도 체결 시 통계/리스크 반영 + ⑪ 잔고 부족 정지 자동 해제
        if matches!(pending.record.side, OrderSide::Sell) {
            // 매도 체결 = 자본 확보 → 매수 정지 해제
            if self.buy_suspended {
                self.buy_suspended = false;
                self.buy_suspended_reason = None;
                tracing::info!("매도 체결로 자본 확보 — 잔고 부족 매수 정지 해제: {}", symbol);
            }
            self.risk_manager.lock().await.record_pnl(pnl);

            let today = chrono::Local::now().date_naive();
            if let Ok(mut stats) = self.stats_store.get_by_date(today).await {
                stats.total_trades += 1;
                if pnl > 0 {
                    stats.winning_trades += 1;
                    stats.gross_profit += pnl;
                } else if pnl < 0 {
                    stats.losing_trades += 1;
                    stats.gross_loss += pnl;
                }
                stats.recalculate();
                if let Err(e) = self.stats_store.upsert(stats).await {
                    tracing::error!("통계 저장 실패: {}", e);
                }
            }
        }

        // ⑥ 주문 기록 (Filled 상태로 재기록)
        let mut filled_record = pending.record.clone();
        filled_record.status = OrderStatus::Filled;
        filled_record.price = avg_price;
        filled_record.quantity = filled_qty;
        if let Err(e) = self.order_store.append(filled_record).await {
            tracing::error!("주문 기록 저장 실패 (Filled): {}", e);
        }

        // ⑥-b TradeStore 저장 (자동매매 로컬 체결 기록)
        let trade_side = match &pending.record.side {
            OrderSide::Buy  => TradeSide::Buy,
            OrderSide::Sell => TradeSide::Sell,
        };
        let order_id = pending.record.kis_order_id.clone().unwrap_or_default();
        let trade_record = TradeRecord::new(
            symbol.clone(),
            symbol_name.clone(),
            trade_side,
            filled_qty,
            avg_price,
            0,        // fee: KIS 수수료 미포함 (TODO)
            order_id,
            None,     // strategy_id: OrderRecord에 없음
            pending.signal_reason.clone(), // 체결 원인 (전략 신호 이유)
        );
        if let Err(e) = self.trade_store.append(trade_record).await {
            tracing::error!("TradeStore 저장 실패: {}", e);
        }

        // ⑨ Discord 알림
        if let Some(discord) = &self.discord {
            let side_str = if matches!(pending.record.side, OrderSide::Buy) { "매수" } else { "매도" };
            let pnl_str = if matches!(pending.record.side, OrderSide::Sell) {
                format!(" (PnL: {}{}원)", if pnl >= 0 { "+" } else { "" }, pnl)
            } else {
                String::new()
            };
            let content = format!(
                "{} {} {}주 @ {}원{}",
                symbol_name, side_str, filled_qty, avg_price, pnl_str
            );
            let _ = discord.send(NotificationEvent::trade(content)).await;
        }

        Ok(())
    }

    /// 주문 취소 이벤트 처리
    pub async fn on_cancel(&mut self, odno: &str) -> Result<()> {
        let Some(pending) = self.pending.remove(odno) else {
            return Ok(());
        };

        self.symbol_to_odno.remove(&pending.record.symbol);

        let mut record = pending.record;
        record.status = OrderStatus::Cancelled;
        if let Err(e) = self.order_store.append(record.clone()).await {
            tracing::error!("주문 기록 저장 실패 (Cancelled): {}", e);
        }

        tracing::info!("주문 취소 처리: {} (odno: {})", record.symbol, odno);
        Ok(())
    }

    /// 잔고 부족으로 매수가 정지됐는지 여부
    pub fn is_buy_suspended(&self) -> bool {
        self.buy_suspended
    }

    /// 잔고 부족 매수 정지 수동 해제 (예: 입금 후 사용자 요청)
    pub fn clear_buy_suspension(&mut self) {
        if self.buy_suspended {
            self.buy_suspended = false;
            self.buy_suspended_reason = None;
            tracing::info!("잔고 부족 매수 정지 해제 (수동)");
        }
    }

    /// 미체결 주문 건수
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// 특정 종목의 미체결 odno 조회
    pub fn get_pending_odno(&self, symbol: &str) -> Option<&str> {
        self.symbol_to_odno.get(symbol).map(String::as_str)
    }

    /// 미체결 주문 목록 (UI 표시용)
    pub fn pending_orders(&self) -> Vec<&PendingOrder> {
        self.pending.values().collect()
    }

    /// 일 초기화 — 자동매매 시작 시 또는 자정 리셋 시 호출
    pub fn reset_day(&mut self) {
        let n = self.pending.len();
        self.pending.clear();
        self.symbol_to_odno.clear();
        // 전일 잔고부족 정지도 초기화
        self.buy_suspended = false;
        self.buy_suspended_reason = None;
        if n > 0 {
            tracing::warn!("일 초기화: 미처리 미체결 주문 {}건 폐기", n);
        }
    }

    // ── 내부 구현 ────────────────────────────────────────────────────

    async fn process_buy(
        &mut self,
        symbol: String,
        symbol_name: String,
        quantity: u64,
        reason: String,
        total_balance: i64,
        exchange: Option<String>,
        tick_price: u64,
    ) -> Result<()> {
        // ⑪ 잔고 부족 매수 정지 체크
        if self.buy_suspended {
            tracing::debug!(
                "매수 스킵 — 잔고 부족 정지 중: {} (사유: {})",
                symbol,
                self.buy_suspended_reason.as_deref().unwrap_or("알 수 없음")
            );
            return Ok(());
        }

        // ③ 중복 주문 방지
        if self.symbol_to_odno.contains_key(&symbol) {
            tracing::debug!("매수 스킵 — {} 미체결 주문 이미 존재", symbol);
            return Ok(());
        }

        // ⑧ 리스크 검증
        {
            let risk = self.risk_manager.lock().await;
            if !risk.can_trade() {
                tracing::warn!(
                    "리스크 한도 초과 — 매수 거부: {} (비상정지 or 손실한도)",
                    symbol
                );
                return Ok(());
            }
            // 포지션 비중 검사 (추정 주문금액 = quantity × 100,000원 보수적 추정)
            if total_balance > 0 {
                let est_amount = (quantity as i64).saturating_mul(100_000);
                if !risk.check_position_size(est_amount, total_balance) {
                    tracing::warn!(
                        "포지션 비중 초과 — 매수 거부: {} (추정 {}원 / 총잔고 {}원)",
                        symbol,
                        est_amount,
                        total_balance
                    );
                    return Ok(());
                }
            }
        }

        // ① 주문 실행 — 해외(지정가 USD) / 국내(시장가 KRW) 분기
        let order_result = if let Some(ref exch) = exchange {
            // 해외 지정가 주문 (KIS 해외는 시장가 미지원)
            // fetch_overseas_tick 반환 코드(NAS/NYS/AMS) → 주문 코드(NASD/NYSE/AMEX) 변환
            let order_exch = match exch.as_str() {
                "NAS" => "NASD",
                "NYS" => "NYSE",
                "AMS" => "AMEX",
                other => other,
            };
            let usd_price = tick_price as f64 / 100.0;
            let req = OverseasOrderRequest {
                symbol: symbol.clone(),
                exchange: order_exch.to_string(),
                side: RestOrderSide::Buy,
                quantity,
                price: usd_price,
            };
            tracing::info!(
                "해외 매수 주문 시도: {} {}주 @ ${:.2} ({}) — {}",
                symbol, quantity, usd_price, order_exch, reason
            );
            self.place_overseas_with_retry(&req).await
        } else {
            // 국내 시장가 매수
            let req = OrderRequest {
                symbol: symbol.clone(),
                side: RestOrderSide::Buy,
                order_type: OrderType::Market,
                quantity,
                price: 0,
            };
            self.place_with_retry(&req).await
        };

        // ⑪ 잔고 부족 에러 감지 — 이후 매수 주문 정지
        let response = match order_result {
            Ok(resp) => resp,
            Err(e) => {
                let msg = e.to_string();
                if is_insufficient_balance_error(&msg) {
                    self.buy_suspended = true;
                    self.buy_suspended_reason = Some(msg.clone());
                    tracing::warn!(
                        "잔고 부족 감지 — 매수 주문 정지: {} (매도 체결 또는 수동 해제 시 재개) | {}",
                        symbol, msg
                    );
                    return Ok(()); // 에러 전파 없이 정상 종료 (상위 루프가 계속 실행되도록)
                }
                return Err(e);
            }
        };
        tracing::info!(
            "매수 주문 접수: {} {}주 — {} (odno: {})",
            symbol,
            quantity,
            reason,
            response.odno
        );

        self.register_pending(symbol, symbol_name, OrderSide::Buy, quantity, reason, response)
            .await
    }

    async fn process_sell(
        &mut self,
        symbol: String,
        symbol_name: String,
        quantity: u64,
        reason: String,
        exchange: Option<String>,
        tick_price: u64,
    ) -> Result<()> {
        // ③ 중복 주문 방지
        if self.symbol_to_odno.contains_key(&symbol) {
            tracing::debug!("매도 스킵 — {} 미체결 주문 이미 존재", symbol);
            return Ok(());
        }

        // 보유 포지션 확인
        // - 국내: position_tracker 확인 필수
        // - 해외: tracker 미동기화 상태일 수 있으므로 수량 그대로 신뢰
        let sell_qty = if exchange.is_none() {
            let position_qty = {
                let tracker = self.position_tracker.lock().await;
                tracker.get(&symbol).map(|p| p.quantity).unwrap_or(0)
            };
            if position_qty == 0 {
                tracing::debug!("매도 스킵 — {} 보유 포지션 없음", symbol);
                return Ok(());
            }
            quantity.min(position_qty)
        } else {
            quantity
        };

        // ⑧ 비상정지 확인 (매도는 손실한도와 무관하게 실행 허용)
        if self.risk_manager.lock().await.is_emergency_stop() {
            tracing::warn!("비상정지 상태 — 매도 거부: {}", symbol);
            return Ok(());
        }

        // ① 주문 실행 — 해외(지정가 USD) / 국내(시장가 KRW) 분기
        let response = if let Some(ref exch) = exchange {
            let order_exch = match exch.as_str() {
                "NAS" => "NASD",
                "NYS" => "NYSE",
                "AMS" => "AMEX",
                other => other,
            };
            let usd_price = tick_price as f64 / 100.0;
            let req = OverseasOrderRequest {
                symbol: symbol.clone(),
                exchange: order_exch.to_string(),
                side: RestOrderSide::Sell,
                quantity: sell_qty,
                price: usd_price,
            };
            tracing::info!(
                "해외 매도 주문 시도: {} {}주 @ ${:.2} ({}) — {}",
                symbol, sell_qty, usd_price, order_exch, reason
            );
            self.place_overseas_with_retry(&req).await?
        } else {
            let req = OrderRequest {
                symbol: symbol.clone(),
                side: RestOrderSide::Sell,
                order_type: OrderType::Market,
                quantity: sell_qty,
                price: 0,
            };
            self.place_with_retry(&req).await?
        };
        tracing::info!(
            "매도 주문 접수: {} {}주 — {} (odno: {})",
            symbol,
            sell_qty,
            reason,
            response.odno
        );

        self.register_pending(symbol, symbol_name, OrderSide::Sell, sell_qty, reason, response)
            .await
    }

    /// 미체결 풀 등록 + ⑥ 주문 기록 저장 (Pending)
    async fn register_pending(
        &mut self,
        symbol: String,
        symbol_name: String,
        side: OrderSide,
        quantity: u64,
        reason: String,
        response: OrderResponse,
    ) -> Result<()> {
        let mut record = OrderRecord::new(
            symbol.clone(),
            symbol_name,
            side,
            quantity,
            0, // price — 체결 시점(on_fill)에 avg_price로 갱신
            "Market".to_string(),
        );
        // KIS 모의투자 환경에서 ondo가 빈 문자열로 반환될 수 있음 → 로컬 UUID로 대체
        let odno = if response.odno.is_empty() {
            format!("LOCAL-{}", uuid::Uuid::new_v4())
        } else {
            response.odno
        };
        record.kis_order_id = Some(odno.clone());

        if let Err(e) = self.order_store.append(record.clone()).await {
            tracing::error!("주문 기록 저장 실패 (Pending): {}", e);
        }

        self.symbol_to_odno.insert(symbol, odno.clone());
        self.pending.insert(odno, PendingOrder { record, signal_reason: reason });

        Ok(())
    }

    /// ⑩ 재시도 로직 — KIS rate-limit 오류 시 2초 대기 × 최대 3회
    /// - EGW00133: 초당 주문건수 초과
    /// - EGW00201: 초당 거래건수 초과 (price/order 공통)
    async fn place_with_retry(&self, req: &OrderRequest) -> Result<OrderResponse> {
        const MAX_RETRIES: u32 = 3;
        let mut last_err = anyhow::anyhow!("주문 최대 재시도 횟수(3회) 초과");

        for attempt in 0..MAX_RETRIES {
            let client = self.rest_client.read().await.clone();
            match client.place_order(req).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    let msg = e.to_string();
                    let is_rate_limit = msg.contains("EGW00133") || msg.contains("EGW00201");
                    if is_rate_limit && attempt < MAX_RETRIES - 1 {
                        tracing::warn!(
                            "KIS rate-limit ({}) — 2초 후 재시도 ({}/{})",
                            if msg.contains("EGW00201") { "EGW00201" } else { "EGW00133" },
                            attempt + 1,
                            MAX_RETRIES
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        last_err = e;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Err(last_err)
    }

    /// ⑩ 해외 주문 재시도 — 국내와 동일한 rate-limit 재시도 로직
    async fn place_overseas_with_retry(&self, req: &OverseasOrderRequest) -> Result<OrderResponse> {
        const MAX_RETRIES: u32 = 3;
        let mut last_err = anyhow::anyhow!("해외 주문 최대 재시도 횟수(3회) 초과");

        for attempt in 0..MAX_RETRIES {
            let client = self.rest_client.read().await.clone();
            match client.place_overseas_order(req).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    let msg = e.to_string();
                    let is_rate_limit = msg.contains("EGW00133") || msg.contains("EGW00201");
                    if is_rate_limit && attempt < MAX_RETRIES - 1 {
                        tracing::warn!(
                            "KIS rate-limit (해외, {}) — 2초 후 재시도 ({}/{})",
                            if msg.contains("EGW00201") { "EGW00201" } else { "EGW00133" },
                            attempt + 1,
                            MAX_RETRIES
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        last_err = e;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Err(last_err)
    }

    /// 종목명으로 미체결 주문을 체결 처리 (시장가 주문 자동 확인용)
    ///
    /// 폴링 루프에서 주문 접수 후 다음 틱에 호출 — 시장가 주문은 즉시 체결 가정
    pub async fn confirm_fill_by_symbol(&mut self, symbol: &str, fill_price: u64) -> Result<()> {
        let ondo = match self.symbol_to_odno.get(symbol).cloned() {
            Some(o) => o,
            None => return Ok(()), // 미체결 주문 없음
        };
        let qty = self.pending.get(&ondo).map(|p| p.record.quantity).unwrap_or(1);
        self.on_fill(&ondo, qty, fill_price).await
    }
}

// ────────────────────────────────────────────────────────────────────
// 잔고 부족 에러 감지 헬퍼
// ────────────────────────────────────────────────────────────────────

/// KIS API 응답에서 잔고 부족 오류인지 판별.
///
/// 알려진 KIS 에러코드/메시지:
/// - `APBK0013`: 주문가능금액 부족
/// - `APBK0915`: 잔고 부족
/// - `APBK0017`: 주문가능금액이 없습니다
/// - msg1 키워드: "잔고부족", "잔고 부족", "주문가능금액부족", "주문가능금액 부족"
fn is_insufficient_balance_error(msg: &str) -> bool {
    msg.contains("잔고부족")
        || msg.contains("잔고 부족")
        || msg.contains("주문가능금액부족")
        || msg.contains("주문가능금액 부족")
        || msg.contains("APBK0013")
        || msg.contains("APBK0915")
        || msg.contains("APBK0017")
}
