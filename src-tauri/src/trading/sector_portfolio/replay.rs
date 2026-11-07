use super::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Default)]
struct Ledger {
    cash: f64,
    positions: BTreeMap<String, SectorHolding>,
    realized: f64,
    costs: f64,
    turnover: f64,
    state: Option<DefensiveState>,
    selection: Vec<String>,
    trades: Vec<SectorTrade>,
    daily: Vec<SectorDaily>,
    peak: f64,
    previous_equity: f64,
}

impl Ledger {
    fn holdings(&self) -> Vec<SectorHolding> {
        self.positions.values().cloned().collect()
    }

    fn equity(&self, day: &SectorDay) -> f64 {
        self.cash
            + self
                .positions
                .values()
                .map(|row| row.quantity as f64 * day.prices[&row.symbol].close)
                .sum::<f64>()
    }

    fn execute(&mut self, plan: &SectorPlan, day: &SectorDay, fee: f64) -> Result<(), String> {
        if !plan.rebalance {
            return Ok(());
        }
        let targets: BTreeMap<_, _> = plan.targets.iter().map(|target| {
            let quantity = (target.budget_usd / day.prices[&target.symbol].open).floor();
            if !quantity.is_finite() || !(0.0..=9_007_199_254_740_991.0).contains(&quantity) {
                return Err("SECTOR_QUANTITY_LIMIT: JSON 정수 안전범위를 초과하는 목표수량은 평가하지 않습니다.".into());
            }
            Ok((target.symbol.clone(), quantity as u64))
        }).collect::<Result<_, String>>()?;
        // Confirmed holdings are reduced before any new purchase; all books use
        // the same immutable observed opening price for a symbol/day.
        let existing: Vec<_> = self.positions.keys().cloned().collect();
        for symbol in existing {
            let held = self.positions[&symbol].quantity;
            let target = *targets.get(&symbol).unwrap_or(&0);
            if held > target {
                self.sell(&symbol, held - target, day, fee);
            }
        }
        for symbol in &plan.selected_symbols {
            let held = self.positions.get(symbol).map_or(0, |row| row.quantity);
            let target = targets[symbol];
            if target > held {
                self.buy(symbol, target - held, day, fee);
            }
        }
        Ok(())
    }

    fn sell(&mut self, symbol: &str, quantity: u64, day: &SectorDay, fee: f64) {
        let price = day.prices[symbol].open;
        let amount = price * quantity as f64;
        let cost = amount * fee;
        let row = self.positions.get_mut(symbol).expect("owned holding");
        let allocated_buy_cost = row.remaining_buy_cost_usd * quantity as f64 / row.quantity as f64;
        self.realized +=
            amount - cost - row.average_cost_usd * quantity as f64 - allocated_buy_cost;
        row.remaining_buy_cost_usd -= allocated_buy_cost;
        row.quantity -= quantity;
        if row.quantity == 0 {
            self.positions.remove(symbol);
        }
        self.cash += amount - cost;
        self.costs += cost;
        self.turnover += amount;
        self.trades.push(SectorTrade {
            date: day.date.clone(),
            symbol: symbol.into(),
            side: "sell".into(),
            requested_quantity: quantity,
            quantity,
            price_usd: price,
            cost_usd: cost,
            cash_after_usd: self.cash,
            status: "filled".into(),
        });
    }

    fn buy(&mut self, symbol: &str, requested: u64, day: &SectorDay, fee: f64) {
        let price = day.prices[symbol].open;
        let unit_cost = price * (1.0 + fee);
        let affordable = (self.cash / unit_cost).floor() as u64;
        let mut quantity = requested.min(affordable);
        // Floating arithmetic must never turn a positive cash book negative.
        while quantity > 0 && price * quantity as f64 + price * quantity as f64 * fee > self.cash {
            quantity -= 1;
        }
        let amount = price * quantity as f64;
        let cost = amount * fee;
        if quantity > 0 {
            let row = self
                .positions
                .entry(symbol.into())
                .or_insert(SectorHolding {
                    symbol: symbol.into(),
                    quantity: 0,
                    average_cost_usd: 0.0,
                    remaining_buy_cost_usd: 0.0,
                });
            row.average_cost_usd = (row.average_cost_usd * row.quantity as f64 + amount)
                / (row.quantity + quantity) as f64;
            row.quantity += quantity;
            row.remaining_buy_cost_usd += cost;
            self.cash -= amount + cost;
            self.costs += cost;
            self.turnover += amount;
        }
        self.trades.push(SectorTrade {
            date: day.date.clone(),
            symbol: symbol.into(),
            side: "buy".into(),
            requested_quantity: requested,
            quantity,
            price_usd: price,
            cost_usd: cost,
            cash_after_usd: self.cash,
            status: if quantity == requested {
                "filled"
            } else if quantity == 0 {
                "notAffordable"
            } else {
                "cashLimited"
            }
            .into(),
        });
    }

    fn mark(&mut self, plan: SectorPlan, day: &SectorDay) -> Result<(), String> {
        let equity = self.equity(day);
        if !equity.is_finite() || !self.cash.is_finite() || self.cash < 0.0 || equity <= 0.0 {
            return Err(
                "INVALID_SECTOR_ACCOUNTING: 비유한 자산 또는 음수 현금을 허용하지 않습니다.".into(),
            );
        }
        self.peak = self.peak.max(equity);
        self.state = Some(plan.defensive_state);
        self.selection.clone_from(&plan.selected_symbols);
        self.daily.push(SectorDaily {
            date: day.date.clone(),
            plan,
            cash_usd: self.cash,
            equity_usd: equity,
            daily_return_pct: (equity / self.previous_equity - 1.0) * 100.0,
            drawdown_pct: (1.0 - equity / self.peak) * 100.0,
            exposure_pct: (equity - self.cash) / equity * 100.0,
            holdings: self.holdings(),
        });
        self.previous_equity = equity;
        Ok(())
    }

    fn finish(
        self,
        strategy: SectorStrategy,
        initial: f64,
        last: &SectorDay,
    ) -> SectorPortfolioResult {
        let equity = self.equity(last);
        let unrealized = self
            .positions
            .values()
            .map(|row| {
                row.quantity as f64 * (last.prices[&row.symbol].close - row.average_cost_usd)
                    - row.remaining_buy_cost_usd
            })
            .sum();
        let mean = self
            .daily
            .iter()
            .map(|row| row.daily_return_pct)
            .sum::<f64>()
            / self.daily.len() as f64;
        let variance = if self.daily.len() > 1 {
            self.daily
                .iter()
                .map(|row| (row.daily_return_pct - mean).powi(2))
                .sum::<f64>()
                / (self.daily.len() - 1) as f64
        } else {
            0.0
        };
        let summary = SectorSummary {
            initial_cash_usd: initial,
            ending_equity_usd: equity,
            net_return_pct: (equity / initial - 1.0) * 100.0,
            same_path_return_before_costs_pct: ((equity + self.costs) / initial - 1.0) * 100.0,
            max_drawdown_pct: self
                .daily
                .iter()
                .map(|row| row.drawdown_pct)
                .fold(0.0, f64::max),
            max_daily_loss_pct: self
                .daily
                .iter()
                .map(|row| (-row.daily_return_pct).max(0.0))
                .fold(0.0, f64::max),
            daily_volatility_pct: variance.sqrt(),
            total_cost_usd: self.costs,
            realized_pnl_usd: self.realized,
            unrealized_pnl_usd: unrealized,
            average_exposure_pct: self.daily.iter().map(|row| row.exposure_pct).sum::<f64>()
                / self.daily.len() as f64,
            turnover_pct: self.turnover / initial * 100.0,
            filled_trade_count: self.trades.iter().filter(|row| row.quantity > 0).count(),
            cash_limited_count: self
                .trades
                .iter()
                .filter(|row| row.quantity < row.requested_quantity)
                .count(),
            ending_cash_usd: self.cash,
        };
        SectorPortfolioResult {
            strategy,
            summary,
            ending_holdings: self.holdings(),
            daily: self.daily,
            trades: self.trades,
        }
    }
}

/// Historical approximation only. This has no provider, account, order, risk
/// mutation, runtime registration or persistence side effects.
pub fn preview_sector_portfolio(
    input: &SectorPortfolioInput,
) -> Result<SectorPortfolioReport, String> {
    validation::input(input)?;
    let mut report = replay_validated(input)?;
    report.warnings = strict_warnings(input);
    Ok(report)
}

/// Private accounting core. Each public mode must validate its own complete input
/// before calling this; this function does not certify source metadata.
pub(super) fn replay_validated(
    input: &SectorPortfolioInput,
) -> Result<SectorPortfolioReport, String> {
    let normalized = serde_json::to_vec(input).map_err(|error| error.to_string())?;
    let input_hash = hex::encode(Sha256::digest(&normalized));
    let days: Vec<_> = input
        .history
        .iter()
        .chain(&input.evaluation)
        .cloned()
        .collect();
    let strategies = [
        SectorStrategy::Base,
        SectorStrategy::Defensive,
        SectorStrategy::Aggressive,
        SectorStrategy::Spy,
        SectorStrategy::Cash,
    ];
    let mut books: Vec<_> = strategies
        .iter()
        .map(|_| Ledger {
            cash: input.initial_cash_usd,
            peak: input.initial_cash_usd,
            previous_equity: input.initial_cash_usd,
            ..Ledger::default()
        })
        .collect();
    let mut rankings = Vec::with_capacity(input.evaluation.len());
    for index in input.history.len()..days.len() {
        let history = &days[..index];
        let day = &days[index];
        let (spy_r20, previous_spy_r20) = planner::spy_returns(history);
        rankings.push(SectorRankingDay {
            date: day.date.clone(),
            signal_date: history.last().expect("validated prefix").date.clone(),
            spy_r20,
            previous_spy_r20,
            rankings: rank_sectors(history)?,
        });
        for (strategy, book) in strategies.iter().zip(&mut books) {
            let plan = plan_sector_portfolio(
                history,
                *strategy,
                book.state.unwrap_or(DefensiveState::Normal),
                &book.holdings(),
                book.cash,
                &book.selection,
            )?;
            book.execute(&plan, day, input.cost_bps / 10_000.0)?;
            book.mark(plan, day)?;
        }
    }
    let last = days.last().expect("validated window");
    let portfolios = strategies
        .into_iter()
        .zip(books)
        .map(|(strategy, book)| book.finish(strategy, input.initial_cash_usd, last))
        .collect();
    Ok(SectorPortfolioReport {
        engine_version: SECTOR_ENGINE_VERSION.into(),
        input_hash,
        source: input.source.clone(),
        history_bars: input.history.len(),
        evaluation_bars: input.evaluation.len(),
        start_date: input.evaluation[0].date.clone(),
        end_date: last.date.clone(),
        execution_model: "nextOpenApproximation".into(),
        warnings: Vec::new(),
        rankings,
        portfolios,
    })
}

fn strict_warnings(input: &SectorPortfolioInput) -> Vec<String> {
    let mut warnings = vec![
        "일봉 nextOpen 사후 근사입니다. 원장의 뉴욕09:50 조회 표시가격·동시 실체결을 복원하지 않습니다.".into(),
        "출처·거래일 달력·coverage·평가창 기업행위 부재·조정 기준은 입력자의 선언이며 프로그램이 공급원/공식 달력을 자동 인증하지 않았습니다.".into(),
        "inputHash는 serde로 정규화한 전체 입력의 SHA256이며 원본 파일 bytes의 hash가 아닙니다.".into(),
        "새 독립 가상실험입니다. 첫 평가 전 방어상태는 보통이며 사전자료는 지표만 준비합니다. 첫 두 신호가 혼합이면 보통을 유지하고 기존 모의 원장의 상태/보유를 복원하지 않습니다.".into(),
        "corporateActionsVerified는 평가창의 분할·배당/미수배당 사건 부재 선언입니다. 무보유 사전자료의 과거 기업행위는 조정종가 출처에 위임합니다. corporateActions 목록으로 전달된 사건은 시점과 무관하게 현재 미지원으로 거부합니다.".into(),
        "신호는 이전 완료봉으로 확정하고 관측 시가에서 정수 수량을 산출합니다. OHLC는 실제 호가·유동성·부분체결/거절 가능성을 입증하지 않습니다.".into(),
        "독립 연구 현금장부이며 운영 OrderManager·TradeGuard·RiskManager를 호출하지 않습니다. 실거래/기존전략 guard 통과 성과가 아닙니다.".into(),
        "기말 보유는 마지막 비조정 종가로 평가하며 가상 청산비용을 추가 차감하지 않았습니다. 세금·현금이자는0입니다.".into(),
        "정산은 USD f64 전체 정밀도로 계산하고 센트 단위 반올림을 매 거래에 적용하지 않습니다. 현금/수량 한도를 별도 검사하며 독립 재정산 오차 허용범위를 구분해야 합니다.".into(),
        "비용 제거 수익률은 같은 체결 경로에 비용을 더한 산술값이며 비용0에서 재실행한 전략 결과가 아닙니다. 노출은 봉말 평가비중입니다.".into(),
        "회전율=매수대금+매도대금 합계/초기자금×100(양방향, 연율화 없음), 변동성=일간 수익률 표본 표준편차(비연율화)입니다. 5%p 비교는 부동소수 경계 보호1e-12를 적용합니다.".into(),
        "짧은 회고 구간으로 전략 우월성·급락방어·실거래 안전성을 확정할 수 없습니다. 분할/배당 이벤트가 있거나 부재 검증이 없는 입력은 평가를 거부합니다.".into(),
    ];
    if input.source.synthetic {
        warnings.insert(
            0,
            "합성 fixture 결과입니다. 실제 시장자료 또는 실제 모의투자 성과가 아닙니다.".into(),
        );
    }
    if input.cost_bps == 20.0 {
        warnings.push("편도 총20bp 스트레스입니다. 기준10bp에20bp를 가산한 값이 아닙니다.".into());
    }
    warnings
}
