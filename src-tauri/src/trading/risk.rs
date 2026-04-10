use serde::{Deserialize, Serialize};

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
    /// 오늘 누적 총 손실 (음수 누적). 손실 체결만 반영
    current_loss: i64,
    /// 오늘 누적 총 수익 (양수 누적). 수익 체결만 반영
    daily_profit: i64,
    /// 비상 정지 여부
    emergency_stop: bool,
    /// 일별 초기화 기준 날짜
    #[serde(default)]
    last_reset_date: Option<chrono::NaiveDate>,
}

fn default_true() -> bool { true }

impl RiskManager {
    pub fn new(daily_loss_limit: i64, max_position_ratio: f64) -> Self {
        Self {
            enabled: true,
            daily_loss_limit,
            max_position_ratio,
            current_loss: 0,
            daily_profit: 0,
            emergency_stop: false,
            last_reset_date: Some(chrono::Local::now().date_naive()),
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
        if !self.enabled { return; }
        // 순손실이 한도 이상이면 비상 정지
        if self.net_loss() >= self.daily_loss_limit {
            self.emergency_stop = true;
            tracing::warn!(
                "일일 순손실 한도 초과 — 손실{}원 - 수익{}원 = 순손실{}원 / 한도{}원 → 비상 정지",
                self.current_loss.abs(), self.daily_profit, self.net_loss(), self.daily_loss_limit
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

    /// 리스크 관리 활성화 여부
    pub fn is_enabled(&self) -> bool { self.enabled }

    /// 리스크 관리 활성화 토글
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        tracing::info!("리스크 관리 {}", if enabled { "활성화" } else { "비활성화" });
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
        self.last_reset_date = Some(chrono::Local::now().date_naive());
        // 비상 정지는 수동 해제 필요
    }

    /// 날짜가 바뀌었으면 자동으로 일별 손익 초기화
    pub fn reset_if_new_day(&mut self) {
        let today = chrono::Local::now().date_naive();
        if self.last_reset_date != Some(today) {
            self.current_loss = 0;
            self.daily_profit = 0;
            self.last_reset_date = Some(today);
            tracing::info!("리스크 관리자 일별 초기화 완료 (날짜: {})", today);
        }
    }

    /// 오늘 누적 총 손실 (음수)
    pub fn current_loss(&self) -> i64 { self.current_loss }

    /// 오늘 누적 총 수익 (양수)
    pub fn daily_profit(&self) -> i64 { self.daily_profit }
}

impl Default for RiskManager {
    fn default() -> Self {
        // 기본값: 리스크 관리 활성, 50만원 손실 한도, 종목당 20% 비중
        Self::new(500_000, 0.20)
    }
}

