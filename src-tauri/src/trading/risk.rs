use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DailyOrderSide {
    Buy,
    Sell,
}

/// 리스크 관리자
/// - 일일 최대 순손실 한도 감시 (순손실 = 총 손실 - 당일 수익)
/// - 최대 단일 종목 비중 검사
/// - 비상 정지(Emergency Stop) 기능
#[derive(Debug, Serialize, Deserialize)]
pub struct RiskManager {
    /// 리스크 관리 활성화 여부. false이면 자동 비상정지·한도 검사 비활성
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 일일 최대 순손실 한도 (원, 양수)
    pub daily_loss_limit: i64,
    /// 단일 종목 최대 투자 비중 (0.0 ~ 1.0)
    pub max_position_ratio: f64,
    /// 전략/종목별 일일 매수 주문 제한. 0이면 제한 없음.
    #[serde(default = "default_daily_buy_order_limit")]
    pub max_daily_buy_orders_per_symbol: u32,
    /// 전략/종목별 일일 매도 주문 제한. 0이면 제한 없음.
    #[serde(default = "default_daily_sell_order_limit")]
    pub max_daily_sell_orders_per_symbol: u32,
    /// 전략/종목별 연속 손실 차단 기준. 0이면 제한 없음.
    #[serde(default = "default_max_consecutive_losses")]
    pub max_consecutive_losses_per_strategy_symbol: u32,
    /// ATR 기반 주문 수량 산정 활성화 여부.
    #[serde(default)]
    pub volatility_sizing_enabled: bool,
    /// 거래당 허용 위험 한도(bps). 100 = 계좌 평가액의 1%.
    #[serde(default = "default_risk_per_trade_bps")]
    pub risk_per_trade_bps: u32,
    /// ATR 손절폭 배수. 수량 = 위험금액 / (ATR × 배수).
    #[serde(default = "default_atr_stop_multiplier")]
    pub atr_stop_multiplier: f64,
    /// 오늘 누적 총 손실 (음수 누적). 손실 체결만 반영
    current_loss: i64,
    /// 오늘 누적 총 수익 (양수 누적). 수익 체결만 반영
    daily_profit: i64,
    /// 비상 정지 여부
    emergency_stop: bool,
    /// 일별 초기화 기준 날짜
    #[serde(default)]
    last_reset_date: Option<chrono::NaiveDate>,
    /// (날짜, 전략 ID, 종목, 매수/매도)별 주문 접수 횟수
    #[serde(skip)]
    daily_order_counts: HashMap<(chrono::NaiveDate, String, String, DailyOrderSide), u32>,
    /// (전략 ID, 종목)별 연속 손실 횟수
    #[serde(skip)]
    consecutive_loss_counts: HashMap<(String, String), u32>,
    /// 연속 손실 기준에 도달해 신규 진입이 차단된 (전략 ID, 종목)
    #[serde(skip)]
    blocked_strategy_symbols: HashMap<(String, String), u32>,
    /// 종목별 최근 ATR. 국내=KRW, 해외=USD cents.
    #[serde(skip)]
    atr_by_symbol: HashMap<String, u64>,
}

fn default_true() -> bool {
    true
}

fn default_daily_buy_order_limit() -> u32 {
    1
}

fn default_daily_sell_order_limit() -> u32 {
    1
}

fn default_max_consecutive_losses() -> u32 {
    3
}

fn default_risk_per_trade_bps() -> u32 {
    100
}

fn default_atr_stop_multiplier() -> f64 {
    2.0
}

impl RiskManager {
    pub fn new(daily_loss_limit: i64, max_position_ratio: f64) -> Self {
        Self {
            enabled: true,
            daily_loss_limit,
            max_position_ratio,
            max_daily_buy_orders_per_symbol: default_daily_buy_order_limit(),
            max_daily_sell_orders_per_symbol: default_daily_sell_order_limit(),
            max_consecutive_losses_per_strategy_symbol: default_max_consecutive_losses(),
            volatility_sizing_enabled: false,
            risk_per_trade_bps: default_risk_per_trade_bps(),
            atr_stop_multiplier: default_atr_stop_multiplier(),
            current_loss: 0,
            daily_profit: 0,
            emergency_stop: false,
            last_reset_date: Some(chrono::Local::now().date_naive()),
            daily_order_counts: HashMap::new(),
            consecutive_loss_counts: HashMap::new(),
            blocked_strategy_symbols: HashMap::new(),
            atr_by_symbol: HashMap::new(),
        }
    }

    /// 순손실 = 총 손실 - 당일 수익 (양수 = 순손실, 0 이하 = 순수익)
    pub fn net_loss(&self) -> i64 {
        let gross_loss = self.current_loss.abs();
        if gross_loss > self.daily_profit {
            gross_loss - self.daily_profit
        } else {
            0
        }
    }

    /// 추가 거래 가능 여부
    pub fn can_trade(&self) -> bool {
        if !self.enabled {
            // 리스크 관리 비활성 시에도 수동 비상정지는 유효
            return !self.emergency_stop;
        }
        !self.emergency_stop && self.net_loss() < self.daily_loss_limit
    }

    /// 순손실 한도 도달 비율 (0.0 ~ 1.0+)
    pub fn loss_ratio(&self) -> f64 {
        if self.daily_loss_limit == 0 {
            return 0.0;
        }
        self.net_loss() as f64 / self.daily_loss_limit as f64
    }

    /// 체결 손익 반영 (positive = 수익, negative = 손실)
    /// - 손실: current_loss에 누적
    /// - 수익: daily_profit에 누적
    /// - 순손실이 한도 이상이면 비상정지 (enabled인 경우에만)
    pub fn record_pnl(&mut self, pnl: i64) {
        if pnl < 0 {
            self.current_loss += pnl; // current_loss는 음수 누적
        } else if pnl > 0 {
            self.daily_profit += pnl;
        }
        // 리스크 관리 비활성 시 자동 비상정지 스킵
        if !self.enabled {
            return;
        }
        // 순손실이 한도 이상이면 비상 정지
        if self.net_loss() >= self.daily_loss_limit {
            self.emergency_stop = true;
            tracing::warn!(
                "일일 순손실 한도 초과 — 손실{}원 - 수익{}원 = 순손실{}원 / 한도{}원 → 비상 정지",
                self.current_loss.abs(),
                self.daily_profit,
                self.net_loss(),
                self.daily_loss_limit
            );
        }
    }

    /// 하위 호환 alias
    pub fn record_loss(&mut self, amount: i64) {
        self.record_pnl(-amount.abs());
    }

    /// 단일 종목 주문 금액이 허용 비중 이내인지 검사
    pub fn check_position_size(&self, order_amount: i64, total_balance: i64) -> bool {
        if total_balance == 0 {
            return false;
        }
        let ratio = order_amount as f64 / total_balance as f64;
        ratio <= self.max_position_ratio
    }

    /// 종목별 ATR을 갱신한다. 가격 단위는 현재가와 동일하다.
    pub fn set_symbol_atr(&mut self, symbol: &str, atr: u64) {
        if atr > 0 {
            self.atr_by_symbol.insert(symbol.to_string(), atr);
        }
    }

    /// ATR이 준비된 종목 수.
    pub fn atr_symbol_count(&self) -> usize {
        self.atr_by_symbol.len()
    }

    /// 계좌 위험 한도와 ATR 손절폭으로 매수 수량을 계산한다.
    ///
    /// - `tick_price`: 국내=KRW, 해외=USD cents
    /// - `total_balance`: KRW 기준 총 평가액
    /// - `exchange_rate_krw`: 해외 가격을 KRW 위험금액으로 환산할 때 사용
    pub fn volatility_adjusted_quantity(
        &self,
        symbol: &str,
        requested_quantity: u64,
        tick_price: u64,
        total_balance: i64,
        is_overseas: bool,
        exchange_rate_krw: f64,
    ) -> u64 {
        if !self.volatility_sizing_enabled
            || requested_quantity == 0
            || tick_price == 0
            || total_balance <= 0
            || self.risk_per_trade_bps == 0
            || self.atr_stop_multiplier <= 0.0
        {
            return requested_quantity;
        }

        let Some(atr) = self.atr_by_symbol.get(symbol).copied().filter(|v| *v > 0) else {
            tracing::debug!("ATR 수량 산정 스킵 — ATR 미준비: {}", symbol);
            return requested_quantity;
        };

        let fx = if is_overseas {
            exchange_rate_krw.max(0.0)
        } else {
            1.0
        };
        if fx <= 0.0 {
            return requested_quantity;
        }

        let price_krw = if is_overseas {
            tick_price as f64 / 100.0 * fx
        } else {
            tick_price as f64
        };
        let stop_distance_krw = if is_overseas {
            atr as f64 / 100.0 * fx * self.atr_stop_multiplier
        } else {
            atr as f64 * self.atr_stop_multiplier
        };
        if price_krw <= 0.0 || stop_distance_krw <= 0.0 {
            return requested_quantity;
        }

        let risk_amount = total_balance as f64 * self.risk_per_trade_bps as f64 / 10_000.0;
        let risk_qty = (risk_amount / stop_distance_krw).floor();
        let max_position_amount = total_balance as f64 * self.max_position_ratio;
        let position_qty = (max_position_amount / price_krw).floor();
        let adjusted = risk_qty.min(position_qty).max(0.0) as u64;

        if adjusted == 0 {
            tracing::warn!(
                "ATR 수량 산정 결과 0주 — 매수 스킵 예정: {} (ATR={}, 위험한도={}bps, 총잔고={}원)",
                symbol,
                atr,
                self.risk_per_trade_bps,
                total_balance
            );
            return 0;
        }

        tracing::info!(
            "ATR 수량 산정: {} 요청{}주 → {}주 (ATR={}, 손절배수={:.2}, 위험한도={}bps)",
            symbol,
            requested_quantity,
            adjusted,
            atr,
            self.atr_stop_multiplier,
            self.risk_per_trade_bps
        );
        adjusted
    }

    /// 전략/종목/방향별 일일 주문 제한 위반 사유를 반환한다.
    pub fn daily_order_limit_reason(
        &mut self,
        strategy_id: &str,
        symbol: &str,
        side: DailyOrderSide,
    ) -> Option<String> {
        self.reset_if_new_day();
        let limit = match side {
            DailyOrderSide::Buy => self.max_daily_buy_orders_per_symbol,
            DailyOrderSide::Sell => self.max_daily_sell_orders_per_symbol,
        };
        if limit == 0 {
            return None;
        }

        let today = chrono::Local::now().date_naive();
        let key = (today, strategy_id.to_string(), symbol.to_string(), side);
        let current = self.daily_order_counts.get(&key).copied().unwrap_or(0);
        if current >= limit {
            let side_name = match side {
                DailyOrderSide::Buy => "매수",
                DailyOrderSide::Sell => "매도",
            };
            Some(format!(
                "전략/종목별 일일 {} 주문 제한 초과: {}/{}회",
                side_name, current, limit
            ))
        } else {
            None
        }
    }

    /// KIS가 주문을 접수한 뒤 일일 주문 카운터를 증가시킨다.
    pub fn record_order_submitted(
        &mut self,
        strategy_id: &str,
        symbol: &str,
        side: DailyOrderSide,
    ) {
        self.reset_if_new_day();
        let today = chrono::Local::now().date_naive();
        let key = (today, strategy_id.to_string(), symbol.to_string(), side);
        *self.daily_order_counts.entry(key).or_insert(0) += 1;
    }

    /// 연속 손실 차단 상태라면 신규 진입 차단 사유를 반환한다.
    pub fn consecutive_loss_block_reason(&self, strategy_id: &str, symbol: &str) -> Option<String> {
        if self.max_consecutive_losses_per_strategy_symbol == 0 {
            return None;
        }
        let key = (strategy_id.to_string(), symbol.to_string());
        self.blocked_strategy_symbols.get(&key).map(|count| {
            format!(
                "전략/종목 연속 손실 차단: {}/{}회",
                count, self.max_consecutive_losses_per_strategy_symbol
            )
        })
    }

    /// 전략/종목별 확정 손익을 반영해 연속 손실 카운터와 신규 진입 차단 상태를 갱신한다.
    pub fn record_strategy_symbol_pnl(&mut self, strategy_id: &str, symbol: &str, pnl: i64) {
        if self.max_consecutive_losses_per_strategy_symbol == 0 {
            return;
        }

        let key = (strategy_id.to_string(), symbol.to_string());
        if pnl < 0 {
            let count = self.consecutive_loss_counts.entry(key.clone()).or_insert(0);
            *count += 1;
            if *count >= self.max_consecutive_losses_per_strategy_symbol {
                self.blocked_strategy_symbols.insert(key, *count);
                tracing::warn!(
                    "전략/종목 연속 손실 차단 — strategy={}, symbol={}, count={}/{}",
                    strategy_id,
                    symbol,
                    count,
                    self.max_consecutive_losses_per_strategy_symbol
                );
            }
        } else if pnl > 0 {
            self.consecutive_loss_counts.remove(&key);
            self.blocked_strategy_symbols.remove(&key);
        }
    }

    /// 현재 연속 손실로 신규 진입이 차단된 전략/종목 조합 수.
    pub fn blocked_strategy_symbol_count(&self) -> usize {
        self.blocked_strategy_symbols.len()
    }

    /// 리스크 관리 활성화 여부
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 리스크 관리 활성화 토글
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        tracing::info!(
            "리스크 관리 {}",
            if enabled { "활성화" } else { "비활성화" }
        );
    }

    /// 비상 정지 상태
    pub fn is_emergency_stop(&self) -> bool {
        self.emergency_stop
    }

    /// 비상 정지 수동 해제
    pub fn clear_emergency_stop(&mut self) {
        self.emergency_stop = false;
        tracing::info!("비상 정지 해제");
    }

    /// 비상 정지 수동 발동 (사용자 요청)
    pub fn trigger_emergency_stop(&mut self) {
        self.emergency_stop = true;
        tracing::warn!("비상 정지 수동 발동 (사용자 요청)");
    }

    /// 일 초기화 (매 거래일 시작 시 호출)
    pub fn reset_daily(&mut self) {
        self.current_loss = 0;
        self.daily_profit = 0;
        self.daily_order_counts.clear();
        self.consecutive_loss_counts.clear();
        self.blocked_strategy_symbols.clear();
        self.atr_by_symbol.clear();
        self.last_reset_date = Some(chrono::Local::now().date_naive());
        // 비상 정지는 수동 해제 필요
    }

    /// 날짜가 바뀌었으면 자동으로 일별 손익 초기화
    pub fn reset_if_new_day(&mut self) {
        let today = chrono::Local::now().date_naive();
        if self.last_reset_date != Some(today) {
            self.current_loss = 0;
            self.daily_profit = 0;
            self.daily_order_counts.clear();
            self.consecutive_loss_counts.clear();
            self.blocked_strategy_symbols.clear();
            self.atr_by_symbol.clear();
            self.last_reset_date = Some(today);
            tracing::info!("리스크 관리자 일별 초기화 완료 (날짜: {})", today);
        }
    }

    /// 오늘 누적 총 손실 (음수)
    pub fn current_loss(&self) -> i64 {
        self.current_loss
    }

    /// 오늘 누적 총 수익 (양수)
    pub fn daily_profit(&self) -> i64 {
        self.daily_profit
    }
}

impl Default for RiskManager {
    fn default() -> Self {
        // 기본값: 리스크 관리 활성, 50만원 손실 한도, 종목당 20% 비중
        Self::new(500_000, 0.20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volatility_sizing_uses_risk_amount_and_position_cap_for_domestic() {
        let mut risk = RiskManager::default();
        risk.volatility_sizing_enabled = true;
        risk.risk_per_trade_bps = 100;
        risk.atr_stop_multiplier = 2.0;
        risk.max_position_ratio = 0.20;
        risk.set_symbol_atr("005930", 100);

        let qty = risk.volatility_adjusted_quantity("005930", 1, 10_000, 1_000_000, false, 1.0);

        assert_eq!(qty, 20);
    }

    #[test]
    fn volatility_sizing_converts_overseas_cents_to_krw() {
        let mut risk = RiskManager::default();
        risk.volatility_sizing_enabled = true;
        risk.risk_per_trade_bps = 100;
        risk.atr_stop_multiplier = 2.0;
        risk.max_position_ratio = 0.20;
        risk.set_symbol_atr("VOO", 100);

        let qty = risk.volatility_adjusted_quantity("VOO", 1, 10_000, 1_000_000, true, 1_000.0);

        assert_eq!(qty, 2);
    }

    #[test]
    fn daily_order_limit_blocks_after_submission_count_reaches_limit() {
        let mut risk = RiskManager::default();
        assert!(risk
            .daily_order_limit_reason("strategy", "005930", DailyOrderSide::Buy)
            .is_none());

        risk.record_order_submitted("strategy", "005930", DailyOrderSide::Buy);

        assert!(risk
            .daily_order_limit_reason("strategy", "005930", DailyOrderSide::Buy)
            .is_some());
    }

    #[test]
    fn consecutive_loss_block_only_clears_after_profit() {
        let mut risk = RiskManager::default();
        risk.max_consecutive_losses_per_strategy_symbol = 2;

        risk.record_strategy_symbol_pnl("strategy", "005930", -1_000);
        assert!(risk
            .consecutive_loss_block_reason("strategy", "005930")
            .is_none());

        risk.record_strategy_symbol_pnl("strategy", "005930", -1_000);
        assert!(risk
            .consecutive_loss_block_reason("strategy", "005930")
            .is_some());

        risk.record_strategy_symbol_pnl("strategy", "005930", 1_000);
        assert!(risk
            .consecutive_loss_block_reason("strategy", "005930")
            .is_none());
    }
}
