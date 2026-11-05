//! Bounded observation diagnostics; never resets or changes the trading ledger.
use super::*;

/// A phase is an observation of one continuous ledger, not an independent run.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseDiagnosticsView {
    pub version: u8,
    pub policy: &'static str,
    pub observed_bars: usize,
    pub held_bars: usize,
    pub exposure_pct: f64,
    pub start_quantity: u64,
    pub end_quantity: u64,
    pub pending_at_start: bool,
    pub pending_at_end: bool,
    pub realized_pnl_krw: f64,
    pub start_residual_unrealized_pnl_krw: f64,
    pub end_residual_unrealized_pnl_krw: f64,
    pub dividend_net_krw: f64,
    pub equity_change_krw: f64,
}

pub(super) struct PhaseObservation {
    start: Snapshot,
    end: Snapshot,
    bars: usize,
    held_bars: usize,
}

#[derive(Clone, Copy)]
struct Snapshot {
    quantity: u64,
    pending: bool,
    equity: f64,
    residual: f64,
    realized: f64,
    dividend: f64,
}

impl ReplaySession {
    fn phase_slot(&self) -> usize {
        usize::from(self.index >= self.split_index && self.split_index < self.total_bars)
    }

    fn observation_snapshot(&self, equity: f64) -> Snapshot {
        Snapshot {
            quantity: self.position.as_ref().map_or(0, |p| p.quantity),
            pending: self.pending.is_some(),
            equity,
            residual: self.position.as_ref().map_or(0.0, |p| {
                (self.krw(self.last_close) - p.average_fill_krw) * p.quantity as f64
                    - p.allocated_buy_cost_krw
            }),
            realized: self.realized_pnl,
            dividend: self.accounting.dividend_net_krw,
        }
    }

    /// Capture the prior close before this bar changes FX, credits dividends or
    /// executes a queued open. Entry costs and pending signals cross the split.
    pub(super) fn begin_phase_observation(&mut self) {
        let slot = self.phase_slot();
        if self.phase_observations[slot].is_none() {
            let equity =
                current_equity(self.cash, self.position.as_ref(), self.krw(self.last_close));
            let snapshot = self.observation_snapshot(equity);
            self.phase_observations[slot] = Some(PhaseObservation {
                start: snapshot,
                end: snapshot,
                bars: 0,
                held_bars: 0,
            });
        }
    }

    pub(super) fn end_phase_observation(&mut self, equity: f64) {
        let snapshot = self.observation_snapshot(equity);
        let slot = self.phase_slot();
        if let Some(observation) = &mut self.phase_observations[slot] {
            observation.end = snapshot;
            observation.bars += 1;
            observation.held_bars += usize::from(snapshot.quantity > 0);
        }
    }

    pub(super) fn attach_phase_diagnostics(&self, phases: &mut [BacktestPhaseView]) {
        for phase in phases {
            let slot = usize::from(phase.phase == "outOfSample");
            let Some(observation) = &self.phase_observations[slot] else {
                continue;
            };
            let start = observation.start;
            let end = observation.end;
            phase.diagnostics = Some(PhaseDiagnosticsView {
                version: 1,
                policy: "sameLedgerObservationSplit",
                observed_bars: observation.bars,
                held_bars: observation.held_bars,
                exposure_pct: round_metric(percent(observation.held_bars, observation.bars)),
                start_quantity: start.quantity,
                end_quantity: end.quantity,
                pending_at_start: start.pending,
                pending_at_end: end.pending,
                realized_pnl_krw: round_money(end.realized - start.realized),
                start_residual_unrealized_pnl_krw: round_money(start.residual),
                end_residual_unrealized_pnl_krw: round_money(end.residual),
                dividend_net_krw: round_money(end.dividend - start.dividend),
                equity_change_krw: round_money(end.equity - start.equity),
            });
        }
    }
}
