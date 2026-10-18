import type { BrokerId, ChartCandle } from './types'

export type ReplayExecutionModel = 'sameClose' | 'nextOpen'
export type ReplayIntrabarExitPolicy = 'unverified' | 'stopFirst'

export interface ReplayEventTime {
  barTime: string
  phase: 'open' | 'close' | 'intrabar'
}

export interface FxScenarioPoint { barTime: string; rateKrw: number }
export interface DividendScenarioPoint { barTime: string; amountPerShare: number; withholdingTaxBps: number }

export interface ReplayComparison {
  sameInputQuantityBuyHold: {
    requestedQuantity: number
    actualEntryQuantity: number
    remainingQuantity: number
    finalEquityKrw: number
    cumulativeReturnPct: number
    terminalLiquidationReturnPct: number
    realizedPnlKrw: number
    residualUnrealizedPnlKrw: number
    dividendNetKrw: number
    mddPct: number
    averageInvestedCapitalPct: number
    averageInvestedEquityPct: number
    peakInvestedCapitalPct?: number
    peakInvestedEquityPct?: number
    holdingBarPct: number
    filledOrderCount: number
    blockedOrderCount: number
    unfilledOrderCount: number
  }
  sameExecutionPathCostReference: {
    restoredCostKrw: number
    finalEquityKrw: number
    cumulativeReturnPct: number
  }
  capitalExposureDifferencePct: number
  holdingBarDifferencePct: number
  limitations: string[]
}

export interface ReplayAccounting {
  realizedPnlKrw: number
  residualUnrealizedPnlKrw: number
  remainingQuantity: number
  remainingCostBasisKrw: number
  remainingBuyFeeKrw: number
  realizedPricePnlKrw: number
  realizedFxPnlKrw: number
  residualPricePnlKrw: number
  residualFxPnlKrw: number
  totalFeeKrw: number
  totalTaxKrw: number
  totalSlippageKrw: number
  dividendGrossKrw: number
  dividendTaxKrw: number
  dividendNetKrw: number
  terminalLiquidationEquityKrw: number
  terminalLiquidationReturnPct: number
  terminalExitFeeKrw: number
  terminalExitTaxKrw: number
  terminalExitSlippageKrw: number
  averageInvestedNotionalKrw: number
  averageInvestedCapitalPct: number
  averageInvestedEquityPct: number
  peakInvestedCapitalPct: number
  peakInvestedEquityPct: number
  realizedEquitySignMismatch: boolean
  warnings: string[]
}

export interface SimulationAssumptions {
  initialCapitalKrw: number
  feeBps: number
  taxBps: number
  slippageBps: number
  exchangeRateKrw: number
  maxPositionRatio: number
  volatilitySizingEnabled: boolean
  riskPerTradeBps: number
  atrStopMultiplier: number
  dailyLossLimitKrw: number
  inSamplePercent: number
  /** Optional only for saved results from older engines. New requests set these explicitly. */
  executionModel?: ReplayExecutionModel
  intrabarExitPolicy?: ReplayIntrabarExitPolicy
  stopLossPct?: number
  takeProfitPct?: number
  dividendPriceBasis?: 'unverified' | 'unadjusted' | 'dividendAdjusted'
  costSource?: 'estimated' | 'brokerVerifiedManual'
  costNote?: string
  buyFeeBps?: number | null
  sellFeeBps?: number | null
  minimumFeeNative?: number
  fxScenario?: FxScenarioPoint[]
  dividendScenario?: DividendScenarioPoint[]
}

export interface ReplayAssessment {
  model: 'dailyDiagnostic' | 'intradaySample'
  performanceStatus: 'notEvaluable' | 'sampleOnly'
  dailyContextBars: number
  intradayBars: number
  timestampStatus: 'unverified'
  sessionStatus: 'unverified'
  limitations: string[]
}

export interface ReplayMetadata {
  engineVersion: string
  strategyVersion: string
  sourceInterval: string
  replayCadence: 'minuteClose' | 'dailyClose' | 'dailyCloseWithDayBoundary' | 'dailyOpenAndClose' | 'weeklyClose' | 'monthlyClose' | 'candleClose' | string
  liveCadenceSeconds: number
  warmupCount: number
  dataStart: string
  dataEnd: string
  dataSource: string
  deterministic: boolean
  lookAheadSafe: boolean
  inputHash: string
  assessment?: ReplayAssessment
}

export interface BacktestTrade {
  time: string
  side: 'buy' | 'sell'
  signalPrice: number
  executionReferencePrice?: number
  exchangeRateKrw?: number
  feeKrw?: number
  taxKrw?: number
  slippageKrw?: number
  fillPrice: number | null
  quantity: number
  grossAmountKrw: number
  costKrw: number
  realizedPnlKrw: number | null
  status: 'filled' | 'blocked' | 'unfilled'
  reason: string
  blockedReason?: string
  phase: 'inSample' | 'outOfSample'
  signalAvailableAt?: ReplayEventTime
  fillAt?: ReplayEventTime
  intrabarPathStatus?: string
}

export interface EquityPoint {
  time: string
  equityKrw: number
  drawdownPct: number
  phase: 'inSample' | 'outOfSample'
}

export interface BacktestPhase {
  phase: 'inSample' | 'outOfSample'
  start: string
  end: string
  returnPct: number
  mddPct: number
  completedTrades: number
  winRatePct: number
}

export interface BacktestSummary {
  initialCapitalKrw: number
  finalEquityKrw: number
  cumulativeReturnPct: number
  mddPct: number
  completedTrades: number
  winningTrades: number
  losingTrades: number
  winRatePct: number
  profitFactor: number | null
  turnoverPct: number
  exposurePct: number
  signalCount: number
  orderEligibleCount: number
  filledOrderCount: number
  blockedOrderCount: number
  unfilledOrderCount?: number
  ambiguousBarCount?: number
}

export interface BacktestReport {
  assumptions: SimulationAssumptions
  summary: BacktestSummary
  phases: BacktestPhase[]
  trades: BacktestTrade[]
  equityCurve: EquityPoint[]
  overfitWarning: string | null
  atrPreparation?: BacktestAtrPreparation
  executionWarnings?: string[]
  accounting?: ReplayAccounting
  comparison?: ReplayComparison
}

export interface BacktestAtrPreparation {
  period: number
  historyBars: number
  readyAtStart: boolean
  readyAtEnd: boolean
  firstReadyTime: string | null
  unreadyEvaluationBars: number
  rangePolicy: 'observedBarsWithPreviousCloseNoInterpolation'
}

export interface StrategyExperimentSnapshot {
  id: string
  slot: 'A' | 'B'
  strategyId: string
  strategyVersion: string
  brokerId: BrokerId
  brokerAccountId: string | null
  symbol: string
  params: Record<string, unknown>
  orderQuantity: number
  replay: ReplayMetadata
  backtest: BacktestReport
  generatedAt: string
}

export interface LeveragedTrendHoldPreviewInput {
  symbol: string
  params: Record<string, unknown>
  interval?: '1m' | '1d'
  count?: number
  assumptions?: SimulationAssumptions
  expectedProfileId?: string | null
  expectedBrokerAccountId?: string | null
}

export interface LeveragedTrendHoldPreviewSignal {
  time: string
  chartTime?: string
  side: 'buy' | 'sell'
  price: number
  quantity: number
  reason: string
  emaShort?: number | null
  emaLong?: number | null
  rsi?: number | null
  adx?: number | null
}

export interface LeveragedTrendHoldPreviewView {
  symbol: string
  interval: '1m' | '1d'
  candleCount: number
  candles: ChartCandle[]
  signals: LeveragedTrendHoldPreviewSignal[]
  generatedAt: string
  message: string
  replay: ReplayMetadata
  backtest: BacktestReport
}

export interface StrategyPreviewInput {
  strategyId: string
  strategyName: string
  symbol: string
  isOverseas: boolean
  orderQuantity: number
  params: Record<string, unknown>
  candles: ChartCandle[]
  historyCandles?: ChartCandle[]
  warmupCount?: number
  interval?: string
  dataSource?: string
  strategyVersion?: string
  brokerId?: BrokerId
  brokerAccountId?: string | null
  assumptions?: SimulationAssumptions
}

export interface StrategyPreviewSignal {
  time: string
  chartTime?: string
  side: 'buy' | 'sell'
  price: number
  quantity: number
  reason: string
}

export interface StrategyPreviewView {
  strategyId: string
  symbol: string
  candles: ChartCandle[]
  signals: StrategyPreviewSignal[]
  generatedAt: string
  message: string
  replay: ReplayMetadata
  backtest: BacktestReport
  preparation?: StrategyPreviewPreparation
}

export interface StrategyPreviewPreparation {
  requiredHistoryBars: number | null
  providedHistoryBars: number
  availableHistoryBars: number | null
  readyAtStart: boolean | null
  readyAtEnd: boolean | null
  firstReadyTime: string | null
  unreadyEvaluationBars: number
  evaluatedBars: number | null
  historyStatus: 'sufficient' | 'insufficient' | 'notRequired' | 'unsupported'
  indicatorStatus: 'ready' | 'warmingUp' | 'unsupported'
  outcome: 'conditionsNotMet' | 'noTrades' | 'traded' | 'notEvaluable'
}
