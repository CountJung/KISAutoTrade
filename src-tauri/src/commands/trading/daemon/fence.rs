//! Serialize daemon mutations with start/stop and configuration publication.

use super::*;
use crate::trading::{order::ExecutionRunContext, strategy::StrategyManager};

pub(super) struct ExecutionCycle {
    pub context: ExecutionRunContext,
    pub rest: Arc<KisRestClient>,
    pub profile: Option<AccountProfile>,
    pub symbols: Vec<String>,
}

/// Provider requests use snapshots captured under the same gate as the run identity.
pub(super) async fn capture_cycle(
    running: &Arc<Mutex<bool>>,
    strategies: &Arc<Mutex<StrategyManager>>,
    orders: &Arc<Mutex<OrderManager>>,
) -> Option<ExecutionCycle> {
    let running = running.lock().await;
    if !*running {
        return None;
    }
    let strategies = strategies.lock().await;
    let orders = orders.lock().await;
    let context = orders.execution_run_context()?;
    if strategies.history_generation() != context.configuration_revision {
        return None;
    }
    let (rest, profile) = orders.execution_run_snapshots(&context)?;
    Some(ExecutionCycle {
        context,
        rest,
        profile,
        symbols: strategies.active_symbols(),
    })
}

pub(super) struct RunFence<'a> {
    pub running: &'a Arc<Mutex<bool>>,
    pub strategies: &'a Arc<Mutex<StrategyManager>>,
    pub orders: &'a Arc<Mutex<OrderManager>>,
    pub expected: &'a ExecutionRunContext,
}

impl RunFence<'_> {
    async fn current_orders(&self) -> Option<tokio::sync::MutexGuard<'_, OrderManager>> {
        let running = self.running.lock().await;
        if !*running {
            return None;
        }
        let strategies = self.strategies.lock().await;
        let orders = self.orders.lock().await;
        if strategies.history_generation() != self.expected.configuration_revision
            || !orders.execution_run_matches(self.expected)
        {
            return None;
        }
        Some(orders)
    }

    /// No provider or storage future is polled with these three guards held.
    pub async fn with_current<R>(
        &self,
        apply: impl FnOnce(&mut StrategyManager) -> R,
    ) -> Option<R> {
        let running = self.running.lock().await;
        if !*running {
            return None;
        }
        let mut strategies = self.strategies.lock().await;
        let orders = self.orders.lock().await;
        if strategies.history_generation() != self.expected.configuration_revision
            || !orders.execution_run_matches(self.expected)
        {
            return None;
        }
        Some(apply(&mut strategies))
    }

    pub async fn is_current(&self) -> bool {
        self.with_current(|_| ()).await.is_some()
    }

    /// Await outside the gate, then reject stale output before its consumer runs.
    pub async fn after<R>(&self, future: impl std::future::Future<Output = R>) -> Option<R> {
        let result = future.await;
        self.with_current(|_| result).await
    }

    /// Keep only the order guard through notification/storage awaits. A following
    /// run cannot begin until the old health mutation finishes, and start/stop can
    /// obtain the flag and strategy guards without waiting for provider I/O here.
    pub async fn apply_balance(&self, result: Result<i64, String>) -> Option<i64> {
        let mut orders = self.current_orders().await?;
        match result {
            Ok(total) if total > 0 => {
                orders.clear_account_sync_suspension();
                orders.mark_holdings_sync(None);
                Some(total)
            }
            result => {
                let error = result
                    .err()
                    .unwrap_or_else(|| "총평가액이 0원 이하입니다.".into());
                tracing::error!(
                    "계좌 스냅샷 갱신 실패로 이번 자동주문 주기를 차단합니다: {}",
                    error
                );
                orders.mark_holdings_sync(Some(error.clone()));
                orders.suspend_buying_for_account_sync(error).await;
                None
            }
        }
    }

    pub async fn reset_day(&self, risk: &Arc<Mutex<crate::trading::risk::RiskManager>>) -> bool {
        let Some(mut orders) = self.current_orders().await else {
            return false;
        };
        let reset = risk.lock().await.reset_if_new_day();
        orders.reset_day();
        if reset {
            orders.persist_risk_runtime().await;
        }
        true
    }
}

#[cfg(test)]
mod tests;
