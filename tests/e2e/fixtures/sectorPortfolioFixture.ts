import type { SectorPortfolioInput, SectorPortfolioReport, SectorStrategy } from '../../../src/shared/api/sectorPortfolioTypes'

const strategies: SectorStrategy[] = ['base', 'defensive', 'aggressive', 'spy', 'cash']
export function fixtureReport(input: SectorPortfolioInput): SectorPortfolioReport {
  const daily = input.evaluation.map(day => ({
    date: day.date,
    plan: { strategy: 'base' as SectorStrategy, signalDate: input.history[input.history.length - 1].date, defensiveState: 'normal' as const, selectedSymbols: [], targetWeight: 0, referenceEquityUsd: 10_000, rebalance: false, reasons: ['UI 응답 fixture'], targets: [] },
    cashUsd: 10_000, equityUsd: 10_000, dailyReturnPct: 0, drawdownPct: 0, exposurePct: 0, holdings: [],
  }))
  return {
    engineVersion: 'sector-portfolio-ui-fixture', inputHash: 'a'.repeat(64), source: input.source,
    historyBars: input.history.length, evaluationBars: input.evaluation.length,
    startDate: daily[0].date, endDate: daily[daily.length - 1].date, executionModel: 'nextOpen',
    warnings: ['합성 응답은 Rust 전략 성과 검증을 대신하지 않습니다.'], rankings: [],
    portfolios: strategies.map(strategy => ({
      strategy, daily: daily.map(day => ({ ...day, plan: { ...day.plan, strategy } })), trades: [], endingHoldings: [],
      summary: { initialCashUsd: 10_000, endingEquityUsd: 10_000, netReturnPct: 0, samePathReturnBeforeCostsPct: 0, maxDrawdownPct: 0, maxDailyLossPct: 0, dailyVolatilityPct: 0, totalCostUsd: 0, realizedPnlUsd: 0, unrealizedPnlUsd: 0, averageExposurePct: 0, turnoverPct: 0, filledTradeCount: 0, cashLimitedCount: 0, endingCashUsd: 10_000 },
    })),
  }
}

