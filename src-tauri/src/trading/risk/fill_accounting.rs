use super::*;
use sha2::{Digest, Sha256};

const MAX_DAILY_FILL_EVENTS: usize = 65_536;

impl RiskManager {
    /// Apply a fill's PnL and strategy loss protection atomically at most once.
    ///
    /// Persist `runtime_state()` after both `Ok(true)` and `Ok(false)` before
    /// acknowledging a pending fill: a previous save may have failed after this
    /// in-memory mutation. Other dates never affect today's protection. Daily
    /// reset/rebuild clears the bounded receipt set together with daily totals.
    pub fn record_fill_pnl_once(
        &mut self,
        execution_date: chrono::NaiveDate,
        event_id: &str,
        scope: &BrokerScope,
        strategy_id: &str,
        symbol: &str,
        pnl: i64,
    ) -> anyhow::Result<bool> {
        // Reconciliation and desktop/manual fills can arrive before the daemon's
        // first new-day cycle. Never retire today's first loss as an old date.
        self.reset_if_new_day();
        let runtime_date = self
            .last_reset_date
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        if execution_date != runtime_date {
            return Ok(false);
        }
        anyhow::ensure!(
            !event_id.is_empty(),
            "체결 리스크 이벤트 ID가 비어 있습니다"
        );
        let event_key = hex::encode(Sha256::digest(serde_json::to_vec(&(scope, event_id))?));
        let payload = hex::encode(Sha256::digest(serde_json::to_vec(&(
            strategy_id,
            symbol,
            pnl,
        ))?));
        if let Some(previous) = self.processed_fill_pnl.get(&event_key) {
            anyhow::ensure!(
                previous == &payload,
                "체결 리스크 이벤트 내용이 변경되었습니다"
            );
            return Ok(false);
        }
        anyhow::ensure!(
            self.processed_fill_pnl.len() < MAX_DAILY_FILL_EVENTS,
            "당일 체결 리스크 이벤트 보관 한도에 도달했습니다"
        );

        // Validate all arithmetic before changing either PnL or its receipt.
        // record_pnl/net_loss use a signed loss magnitude, so i64::MIN cannot
        // safely be represented by their existing absolute-value contract.
        if pnl < 0 {
            anyhow::ensure!(
                self.current_loss
                    .checked_add(pnl)
                    .is_some_and(|loss| loss > i64::MIN),
                "체결 리스크 손실 누적값이 범위를 초과했습니다"
            );
            if self.max_consecutive_losses_per_strategy_symbol > 0 {
                let key = (scope.clone(), strategy_id.to_owned(), symbol.to_owned());
                anyhow::ensure!(
                    self.consecutive_loss_counts.get(&key).copied().unwrap_or(0) < u32::MAX,
                    "체결 리스크 연속 손실 횟수가 범위를 초과했습니다"
                );
            }
        } else if pnl > 0 {
            anyhow::ensure!(
                self.daily_profit.checked_add(pnl).is_some(),
                "체결 리스크 수익 누적값이 범위를 초과했습니다"
            );
        }
        self.record_pnl(pnl);
        self.record_strategy_symbol_pnl_for_scope(scope, strategy_id, symbol, pnl);
        self.processed_fill_pnl.insert(event_key, payload);
        Ok(true)
    }

    /// 재시작 후 복원용 일별 runtime 스냅샷. ATR 캐시는 시세에서 재구축되므로 제외한다.
    pub fn runtime_state(&self) -> RiskRuntimeState {
        RiskRuntimeState {
            date: self
                .last_reset_date
                .unwrap_or_else(|| chrono::Local::now().date_naive()),
            current_loss: self.current_loss,
            daily_profit: self.daily_profit,
            emergency_stop: self.emergency_stop,
            daily_order_counts: self
                .daily_order_counts
                .iter()
                .map(
                    |((date, scope, strategy_id, symbol, side), count)| DailyOrderCountEntry {
                        date: *date,
                        scope: scope.clone(),
                        strategy_id: strategy_id.clone(),
                        symbol: symbol.clone(),
                        side: *side,
                        count: *count,
                    },
                )
                .collect(),
            consecutive_loss_counts: self
                .consecutive_loss_counts
                .iter()
                .map(
                    |((scope, strategy_id, symbol), count)| StrategySymbolCountEntry {
                        scope: scope.clone(),
                        strategy_id: strategy_id.clone(),
                        symbol: symbol.clone(),
                        count: *count,
                    },
                )
                .collect(),
            processed_fill_pnl: self.processed_fill_pnl.clone(),
            blocked_strategy_symbols: self
                .blocked_strategy_symbols
                .iter()
                .map(
                    |((scope, strategy_id, symbol), count)| StrategySymbolCountEntry {
                        scope: scope.clone(),
                        strategy_id: strategy_id.clone(),
                        symbol: symbol.clone(),
                        count: *count,
                    },
                )
                .collect(),
        }
    }

    /// 저장된 일별 runtime 스냅샷을 적용한다.
    ///
    /// - 비상정지는 수동 해제 대상이므로 날짜와 무관하게 복원한다.
    /// - 손익·주문 횟수·연속 손실 상태는 스냅샷 날짜가 오늘일 때만 복원한다.
    pub fn apply_runtime_state(&mut self, state: RiskRuntimeState) {
        self.reset_daily();
        self.emergency_stop = state.emergency_stop;
        let today = chrono::Local::now().date_naive();
        if state.date != today {
            tracing::info!(
                "리스크 runtime 스냅샷이 이전 날짜({})라 비상정지 상태만 복원했습니다.",
                state.date
            );
            self.last_reset_date = Some(today);
            return;
        }
        self.processed_fill_pnl = state.processed_fill_pnl;
        self.current_loss = state.current_loss;
        self.daily_profit = state.daily_profit;
        self.last_reset_date = Some(state.date);
        self.daily_order_counts = state
            .daily_order_counts
            .into_iter()
            .filter(|entry| entry.date == today)
            .map(|entry| {
                (
                    (
                        entry.date,
                        entry.scope,
                        entry.strategy_id,
                        entry.symbol,
                        entry.side,
                    ),
                    entry.count,
                )
            })
            .collect();
        self.consecutive_loss_counts = state
            .consecutive_loss_counts
            .into_iter()
            .map(|entry| ((entry.scope, entry.strategy_id, entry.symbol), entry.count))
            .collect();
        self.blocked_strategy_symbols = state
            .blocked_strategy_symbols
            .into_iter()
            .map(|entry| ((entry.scope, entry.strategy_id, entry.symbol), entry.count))
            .collect();
        tracing::info!(
            "리스크 runtime 복원 완료 — 순손실 {}원, 주문횟수 {}건, 연속손실 차단 {}건, 비상정지 {}",
            self.net_loss(),
            self.daily_order_counts.len(),
            self.blocked_strategy_symbols.len(),
            self.emergency_stop
        );
    }
}

#[cfg(test)]
mod tests;
