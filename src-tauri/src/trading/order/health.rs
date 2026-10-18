//! 주문 실행 상태·동기화 장애·매수 차단과 일 초기화.

use super::*;

impl OrderManager {
    pub fn health_status(&self) -> TradingHealthStatus {
        let mut status = self.health.clone();
        status.pending_order_count = self.pending.len();
        status.oldest_pending_at = self
            .pending
            .values()
            .map(|pending| pending.record.timestamp.as_str())
            .min()
            .map(str::to_string);
        status.persistence_blocked = self.persistence_blocked;
        status
    }

    pub fn mark_holdings_sync(&mut self, error: Option<String>) {
        let now = chrono::Local::now().to_rfc3339();
        self.health.last_holdings_sync_attempt_at = Some(now.clone());
        self.health.last_holdings_sync_error = error.clone();
        if let Some(error) = error {
            self.health.holdings_consecutive_failures =
                self.health.holdings_consecutive_failures.saturating_add(1);
            self.health.daemon_last_error = Some(error);
        } else {
            self.health.last_holdings_sync_at = Some(now);
            self.health.holdings_consecutive_failures = 0;
        }
        self.refresh_daemon_failure_summary();
    }

    pub(in crate::trading) fn mark_reconciliation(&mut self, error: Option<String>) {
        let now = chrono::Local::now().to_rfc3339();
        let previous_failures = self.health.reconciliation_consecutive_failures;
        self.health.last_reconciliation_attempt_at = Some(now.clone());
        self.health.last_reconciliation_error = error.clone();
        if let Some(error) = error {
            self.health.reconciliation_consecutive_failures = self
                .health
                .reconciliation_consecutive_failures
                .saturating_add(1);
            self.health.daemon_last_error = Some(error);
            if self.health.reconciliation_consecutive_failures == 3 {
                if let Some(discord) = self.discord.clone() {
                    let detail = self
                        .health
                        .last_reconciliation_error
                        .clone()
                        .unwrap_or_default();
                    tokio::spawn(async move {
                        let _ = discord
                            .send(NotificationEvent::error(
                                "자동매매 체결 대조 장애",
                                format!("{detail} broker 연결과 계좌 인증을 확인하세요. 미체결 주문의 신규 제출은 신중히 판단해야 합니다."),
                            ))
                            .await;
                    });
                }
            }
        } else {
            self.health.last_reconciliation_at = Some(now);
            self.health.reconciliation_consecutive_failures = 0;
            if previous_failures >= 3 {
                if let Some(discord) = self.discord.clone() {
                    tokio::spawn(async move {
                        let _ = discord
                            .send(NotificationEvent::info(
                                "자동매매 체결 대조 복구",
                                "broker 미체결 주문 대조가 정상화되었습니다.",
                            ))
                            .await;
                    });
                }
            }
        }
        self.refresh_daemon_failure_summary();
    }

    fn refresh_daemon_failure_summary(&mut self) {
        self.health.daemon_consecutive_failures = self
            .health
            .holdings_consecutive_failures
            .max(self.health.reconciliation_consecutive_failures);
        if self.health.daemon_consecutive_failures == 0 {
            self.health.daemon_last_error = None;
        }
    }

    /// 잔고 부족으로 매수가 정지됐는지 여부
    pub fn is_buy_suspended(&self) -> bool {
        self.buy_suspended
    }

    /// 잔고 부족 매수 정지 수동 해제 (예: 입금 후 사용자 요청)
    pub fn clear_buy_suspension(&mut self) {
        if self.persistence_blocked {
            tracing::warn!("영속화 장애 차단은 수동 매수 정지 해제로 해제할 수 없습니다.");
            return;
        }
        if self.buy_suspended {
            self.buy_suspended = false;
            self.buy_suspended_reason = None;
            tracing::info!("잔고 부족 매수 정지 해제 (수동)");
        }
    }

    pub fn block_for_persistence_failure(&mut self, reason: String) {
        let should_notify = !self.persistence_blocked;
        self.persistence_blocked = true;
        self.buy_suspended = true;
        self.buy_suspended_reason = Some(reason);
        self.health.persistence_error = self.buy_suspended_reason.clone();
        if should_notify {
            if let Some(discord) = self.discord.clone() {
                let detail = self.buy_suspended_reason.clone().unwrap_or_default();
                tokio::spawn(async move {
                    let _ = discord
                        .send(NotificationEvent::error(
                            "자동매매 영속화 장애",
                            format!("{detail} 데이터 저장소를 복구하고 앱을 재시작할 때까지 모든 신규 주문을 차단합니다."),
                        ))
                        .await;
                });
            }
        }
    }

    /// 리스크 runtime 스냅샷을 저장한다.
    /// 저장 실패 시 재시작으로 한도·차단 상태를 우회할 수 있으므로 fail-closed로 신규 주문을 차단한다.
    pub async fn persist_risk_runtime(&mut self) {
        let snapshot = self.risk_manager.lock().await.runtime_state();
        if let Err(error) = self.risk_store.save_runtime(&snapshot).await {
            tracing::error!("리스크 runtime 상태 저장 실패: {error}");
            self.block_for_persistence_failure(format!("리스크 runtime 상태 저장 실패: {error}"));
        }
    }

    pub async fn suspend_buying_for_account_sync(&mut self, detail: String) {
        let reason = format!("계좌 동기화 실패: {detail}");
        let should_notify = self.buy_suspended_reason.as_deref() != Some(reason.as_str());
        self.buy_suspended = true;
        self.buy_suspended_reason = Some(reason.clone());
        if should_notify {
            if let Some(discord) = &self.discord {
                let _ = discord
                    .send(NotificationEvent::error(
                        "자동매매 신규 매수 중단",
                        format!(
                            "{reason} 계좌 조회가 정상화될 때까지 신규 매수를 제출하지 않습니다."
                        ),
                    ))
                    .await;
            }
        }
    }

    pub fn clear_account_sync_suspension(&mut self) {
        if self
            .buy_suspended_reason
            .as_deref()
            .is_some_and(|reason| reason.starts_with("계좌 동기화 실패:"))
        {
            self.buy_suspended = false;
            self.buy_suspended_reason = None;
            tracing::info!("계좌 동기화 정상화 — 신규 매수 차단 해제");
        }
    }

    /// 일 초기화 — 자동매매 시작 시 또는 자정 리셋 시 호출
    pub fn reset_day(&mut self) {
        // Provider 접수 전 예약도 실행 재시작을 넘어 유지하고 결과 반영 때만 해제한다.
        self.trade_guard.reset_day();
        // 매수 중단 사유는 자정만으로 해소되지 않는다. 계좌 재동기화, 매도 체결,
        // 영속화 복구 또는 사용자의 명시적 해제에서만 상태를 바꾼다.
        if !self.pending.is_empty() {
            tracing::info!(
                "일 초기화: 미체결 주문 {}건은 provider reconciliation을 위해 유지",
                self.pending.len()
            );
        }
    }
}
