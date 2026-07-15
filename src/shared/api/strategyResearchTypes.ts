import type { BrokerId, ChartCandle } from './types'

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
}

export interface ReplayMetadata {
  engineVersion: string
  strategyVersion: string
  sourceInterval: string
  replayCadence: 'minuteClose' | 'dailyClose' | 'dailyOpenAndClose' | 'weeklyClose' | 'monthlyClose' | 'candleClose' | string
  liveCadenceSeconds: number
  warmupCount: number
  dataStart: string
  dataEnd: string
  dataSource: string
  deterministic: boolean
  lookAheadSafe: boolean
  inputHash: string
}

export interface BacktestTrade {
  time: string
  side: 'buy' | 'sell'
  signalPrice: number
  fillPrice: number | null
  quantity: number
  grossAmountKrw: number
  costKrw: number
  realizedPnlKrw: number | null
  status: 'filled' | 'blocked'
  reason: string
  blockedReason?: string
  phase: 'inSample' | 'outOfSample'
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
}

export interface BacktestReport {
  assumptions: SimulationAssumptions
  summary: BacktestSummary
  phases: BacktestPhase[]
  trades: BacktestTrade[]
  equityCurve: EquityPoint[]
  overfitWarning: string | null
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
}
