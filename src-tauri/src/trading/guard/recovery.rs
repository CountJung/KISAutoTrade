//! Restore operator-bound submissions without moving their original protection window.
use super::*;

const MAX_RECOVERED_SUBMISSIONS: usize = 256;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct RecoveredSubmission {
    scope: BrokerScope,
    symbol: String,
    side: GuardSide,
    submitted: DateTime<FixedOffset>,
    stop_loss: bool,
}
impl TradeGuard {
    /// Live recovery retains the application's Local calendar policy, not the
    /// approval time or a provider/market timezone guessed from the symbol.
    pub(crate) fn restore_recovered_submission(
        &mut self,
        reservation_id: &str,
        scope: &BrokerScope,
        signal: &Signal,
        submitted: DateTime<FixedOffset>,
    ) -> anyhow::Result<()> {
        self.restore_recovered_submission_at(
            reservation_id,
            scope,
            signal,
            submitted.with_timezone(&Local).fixed_offset(),
            Local::now().fixed_offset(),
        )
    }
    fn restore_recovered_submission_at(
        &mut self,
        reservation_id: &str,
        scope: &BrokerScope,
        signal: &Signal,
        submitted: DateTime<FixedOffset>,
        now: DateTime<FixedOffset>,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            !reservation_id.is_empty() && reservation_id.len() <= 128,
            "복구 보호 기록의 예약 ID가 유효하지 않습니다."
        );
        let (symbol, side, reason) = match signal {
            Signal::Buy { symbol, reason, .. } => (symbol.clone(), GuardSide::Buy, reason),
            Signal::Sell { symbol, reason, .. } => (symbol.clone(), GuardSide::Sell, reason),
            Signal::Hold => anyhow::bail!("복구 주문의 매수·매도 방향이 없습니다."),
        };
        let restored = RecoveredSubmission {
            scope: scope.clone(),
            symbol: symbol.clone(),
            side,
            submitted,
            stop_loss: side == GuardSide::Sell && is_stop_loss_reason(reason),
        };
        if let Some(previous) = self.recovered_submissions.get(reservation_id) {
            anyhow::ensure!(
                previous == &restored,
                "복구 보호 기록의 원래 주문 정보가 변경되었습니다."
            );
            return Ok(());
        }
        anyhow::ensure!(
            self.recovered_submissions.len() < MAX_RECOVERED_SUBMISSIONS,
            "복구 보호 기록 한도(256)에 도달하여 보류합니다."
        );
        let symbol_key = (scope.clone(), symbol.clone());
        self.last_order_at
            .entry((scope.clone(), symbol, side))
            .and_modify(|last| *last = (*last).max(submitted))
            .or_insert(submitted);
        if restored.stop_loss {
            self.stop_loss_block_date
                .entry(symbol_key.clone())
                .and_modify(|date| *date = (*date).max(submitted.date_naive()))
                .or_insert(submitted.date_naive());
        }
        // Old receipts must not reintroduce already expired whipsaw observations.
        self.prune_recent(scope, &symbol_key.1, now);
        if submitted >= now - Duration::minutes(self.config.whipsaw_window_min) {
            let recent = self.recent_sides.entry(symbol_key).or_default();
            let position = recent
                .iter()
                .position(|(timestamp, _)| *timestamp > submitted)
                .unwrap_or(recent.len());
            recent.insert(position, (submitted, side));
        }
        // This bounded ID set survives reset_day: retries cannot resurrect a
        // cleared day's recent events. The ordinary last-side cooldown survives too.
        self.recovered_submissions
            .insert(reservation_id.to_string(), restored);
        Ok(())
    }
}
#[cfg(test)]
mod tests;
