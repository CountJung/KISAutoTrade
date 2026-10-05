import type { ChartCandle, StrategyPreviewPreparation } from '../../../src/api/types'

const strategyEntries = [
  {
    leveraged_symbol: 'SOXL',
    leveraged_symbol_name: 'Direxion Daily Semiconductor Bull 3X',
    inverse_leveraged_symbol: 'SOXS',
    inverse_leveraged_symbol_name: 'Legacy inverse',
    base_symbols: ['SOXX'],
    base_symbol_names: { SOXX: 'iShares Semiconductor ETF' },
    base_symbol_roles: { SOXX: 'underlying' },
    quantity: 3,
    inverse_quantity: 1,
    is_overseas: true,
  },
  {
    leveraged_symbol: 'KORU',
    leveraged_symbol_name: 'Direxion Daily South Korea Bull 3X',
    inverse_leveraged_symbol: '',
    inverse_leveraged_symbol_name: '',
    base_symbols: [],
    base_symbol_names: {},
    base_symbol_roles: {},
    quantity: 1,
    inverse_quantity: 1,
    is_overseas: true,
  },
]

function strategy(id: string, name: string, index: number) {
  const isLeveraged = id === 'leveraged_trend_hold_default'
  const symbol = String(index).padStart(6, '0')
  return {
    id,
    name,
    enabled: false,
    brokerId: 'kis',
    brokerAccountId: '12345678-01',
    targetSymbols: isLeveraged ? ['SOXL', 'KORU'] : [symbol],
    targetSymbolNames: isLeveraged
      ? {
        SOXL: 'Direxion Daily Semiconductor Bull 3X',
        KORU: 'Direxion Daily South Korea Bull 3X',
      }
      : { [symbol]: `Mock Strategy ${index}` },
    orderQuantity: 1,
    params: isLeveraged
      ? {
        entries: strategyEntries,
        upward_sensitivity: 1,
        trailing_stop_pct: 1.5,
        trailing_activation_profit_pct: 1,
        breakeven_buffer_pct: 0.2,
        min_hold_observations: 2,
        initial_stop_loss_pct: 1,
        entry_failure_observations: 3,
        rapid_rebound_enabled: false,
        rapid_rebound_lookback_ticks: 8,
        rapid_rebound_drop_pct: 2,
        rapid_rebound_recovery_pct: 1.2,
        rapid_rebound_max_low_age_ticks: 3,
      }
      : {},
  }
}

export function historyFixture(count: number): ChartCandle[] {
  return Array.from({ length: count }, (_, index) => {
    const date = new Date(Date.UTC(2025, 0, 1 + index)).toISOString().slice(0, 10).replaceAll('-', '')
    return { date, open: '100', high: '105', low: '99', close: '104', volume: '1000' }
  })
}

export const readyPreparation: StrategyPreviewPreparation = {
  requiredHistoryBars: 252,
  providedHistoryBars: 252,
  availableHistoryBars: 252,
  readyAtStart: true,
  readyAtEnd: true,
  firstReadyTime: '20250910',
  unreadyEvaluationBars: 0,
  evaluatedBars: 50,
  historyStatus: 'sufficient',
  indicatorStatus: 'ready',
  outcome: 'conditionsNotMet',
}

type MockOptions = {
  strategyDelayMs?: number
  genericPreviewDelayMs?: number
  leveragedPreviewDelayMs?: number
  leveragedLegacy?: boolean
  leveragedZero?: boolean
  activeBroker?: 'kis' | 'toss'
  previewRequests?: unknown[]
  genericPreviewRequests?: unknown[]
  chartRequests?: string[]
  chartCandles?: ChartCandle[]
  genericPreparation?: StrategyPreviewPreparation
  updateRequests?: unknown[]
  scopeController?: { current: 'A' | 'B' }
}

function mockResearchResult(body: Record<string, unknown>, interval: string) {
  const assumptions = body.assumptions ?? {
    initialCapitalKrw: 10_000_000,
    feeBps: 1.5,
    taxBps: 20,
    slippageBps: 5,
    exchangeRateKrw: 1450,
    maxPositionRatio: 0.3,
    volatilitySizingEnabled: false,
    riskPerTradeBps: 100,
    atrStopMultiplier: 2,
    dailyLossLimitKrw: 1_000_000,
    inSamplePercent: 70,
  }
  return {
    replay: {
      engineVersion: 'strategy-replay-v7',
      strategyVersion: 'mock-v1',
      sourceInterval: interval,
      replayCadence: interval === '1m' ? 'minuteClose' : 'dailyCloseWithDayBoundary',
      liveCadenceSeconds: 10,
      warmupCount: 1,
      dataStart: '20260701',
      dataEnd: '20260704',
      dataSource: String(body.dataSource ?? `mock:${interval}`),
      deterministic: true,
      lookAheadSafe: true,
      inputHash: `mock-${interval}-research-hash`,
    },
    backtest: {
      assumptions,
      summary: {
        initialCapitalKrw: 10_000_000,
        finalEquityKrw: 10_010_000,
        cumulativeReturnPct: 0.1,
        mddPct: 0.02,
        completedTrades: 1,
        winningTrades: 1,
        losingTrades: 0,
        winRatePct: 100,
        profitFactor: null,
        turnoverPct: 0.02,
        exposurePct: 50,
        signalCount: 3,
        orderEligibleCount: 2,
        filledOrderCount: 2,
        blockedOrderCount: 1,
      },
      phases: [
        { phase: 'inSample', start: '20260701', end: '20260702', returnPct: 0.08, mddPct: 0, completedTrades: 0, winRatePct: 0 },
        { phase: 'outOfSample', start: '20260703', end: '20260704', returnPct: 0.02, mddPct: 0.02, completedTrades: 1, winRatePct: 100 },
      ],
      trades: [
        { time: '20260702', side: 'buy', signalPrice: 104, fillPrice: 104.1, quantity: 1, grossAmountKrw: 104.1, costKrw: 0.1, realizedPnlKrw: null, status: 'filled', reason: 'mock buy', phase: 'inSample' },
        { time: '20260703', side: 'buy', signalPrice: 105, fillPrice: null, quantity: 1, grossAmountKrw: 0, costKrw: 0, realizedPnlKrw: null, status: 'blocked', reason: 'duplicate', blockedReason: '이미 보유 중인 포지션', phase: 'outOfSample' },
        { time: '20260704', side: 'sell', signalPrice: 106, fillPrice: 105.9, quantity: 1, grossAmountKrw: 105.9, costKrw: 0.2, realizedPnlKrw: 1.6, status: 'filled', reason: 'mock sell', phase: 'outOfSample' },
      ],
      equityCurve: [
        { time: '20260701', equityKrw: 10_000_000, drawdownPct: 0, phase: 'inSample' },
        { time: '20260702', equityKrw: 10_008_000, drawdownPct: 0, phase: 'inSample' },
        { time: '20260703', equityKrw: 10_006_000, drawdownPct: 0.02, phase: 'outOfSample' },
        { time: '20260704', equityKrw: 10_010_000, drawdownPct: 0, phase: 'outOfSample' },
      ],
      overfitWarning: null,
    },
  }
}

export async function mockApi(page: import('@playwright/test').Page, options: MockOptions = {}) {
  const activeBroker = options.activeBroker ?? 'kis'
  let tradingRunning = false
  const activeProfileId = () => options.scopeController?.current === 'B' ? 'profile-b' : (activeBroker === 'toss' ? 'toss-live' : 'paper')
  const activeAccountId = () => options.scopeController?.current === 'B' ? '87654321-01' : (activeBroker === 'toss' ? '1' : '12345678-01')
  const tradingStatus = () => ({
    isRunning: tradingRunning,
    activeStrategies: tradingRunning ? ['leveraged_trend_hold_default'] : [],
    positionCount: 0,
    totalUnrealizedPnl: 0,
    wsConnected: tradingRunning,
    tradingProfileId: tradingRunning ? (activeBroker === 'toss' ? 'toss-live' : 'paper') : null,
    tradingBrokerId: tradingRunning ? activeBroker : null,
    tradingAccountId: tradingRunning ? (activeBroker === 'toss' ? '1' : '12345678-01') : null,
    buySuspended: false,
    buySuspendedReason: null,
    health: {
      lastHoldingsSyncAt: null,
      lastHoldingsSyncAttemptAt: null,
      lastHoldingsSyncError: null,
      lastReconciliationAt: null,
      lastReconciliationAttemptAt: null,
      lastReconciliationError: null,
      oldestPendingAt: null,
      pendingOrderCount: 0,
      holdingsConsecutiveFailures: 0,
      reconciliationConsecutiveFailures: 0,
      persistenceBlocked: false,
      persistenceError: null,
      daemonConsecutiveFailures: 0,
      daemonLastError: null,
    },
  })
  const strategies = [
    strategy('leveraged_trend_hold_default', 'LeveragedTrendHoldStrategy', 1),
    strategy('ma_cross_default', 'MovingAverageCrossStrategy', 2),
    strategy('rsi_default', 'RsiStrategy', 3),
    strategy('momentum_default', 'MomentumStrategy', 4),
    strategy('deviation_default', 'DeviationStrategy', 5),
    strategy('fifty_two_week_high_default', 'FiftyTwoWeekHighStrategy', 6),
    strategy('consecutive_move_default', 'ConsecutiveMoveStrategy', 7),
    strategy('failed_breakout_default', 'FailedBreakoutStrategy', 8),
    strategy('strong_close_default', 'StrongCloseStrategy', 9),
    strategy('volatility_expansion_default', 'VolatilityExpansionStrategy', 10),
    strategy('mean_reversion_default', 'MeanReversionStrategy', 11),
    strategy('trend_filter_default', 'TrendFilterStrategy', 12),
    strategy('price_condition_default', 'PriceConditionStrategy', 13),
  ].map((item) => ({
    ...item,
    brokerId: activeBroker,
    brokerAccountId: activeBroker === 'toss' ? '1' : '12345678-01',
  }))

  await page.route('**/api/**', async (route) => {
    const url = new URL(route.request().url())
    if (!url.pathname.startsWith('/api/')) {
      await route.continue()
      return
    }
    if (url.pathname === '/api/check-update') {
      await route.fulfill({ json: { hasUpdate: false, currentVersion: '0.1.2', latestVersion: '0.1.2', releaseUrl: '' } })
      return
    }
    if (url.pathname === '/api/app-config') {
      await route.fulfill({
        json: {
          active_broker_id: activeBroker,
          active_broker_account_id: activeAccountId(),
          kis_app_key_masked: '***',
          kis_account_no: '12345678-01',
          kis_is_paper_trading: true,
          kis_configured: true,
          active_broker_configured: true,
          discord_enabled: false,
          notification_levels: [],
          active_profile_id: activeProfileId(),
          active_profile_name: options.scopeController?.current === 'B' ? '계좌 B' : (activeBroker === 'toss' ? 'Toss 실전' : '모의'),
        },
      })
      return
    }
    if (url.pathname === '/api/check-config') {
      await route.fulfill({ json: {
        broker_id: 'kis', broker_account_id: activeAccountId(), real_key_set: true, real_account_set: true, paper_key_set: true, active_mode: '모의투자', is_ready: true, discord_configured: false, base_url: 'mock', issues: [],
      } })
      return
    }
    if (url.pathname === '/api/trading/status') {
      await route.fulfill({ json: tradingStatus() })
      return
    }
    if (url.pathname === '/api/trading/start') {
      tradingRunning = true
      await route.fulfill({ json: tradingStatus() })
      return
    }
    if (url.pathname === '/api/trading/stop') {
      tradingRunning = false
      await route.fulfill({ json: tradingStatus() })
      return
    }
    if (url.pathname === '/api/recent-logs') {
      await route.fulfill({ json: [] })
      return
    }
    if (url.pathname === '/api/broker-rate-limit-status') {
      await route.fulfill({ json: [] })
      return
    }
    const settingsPayloads: Record<string, unknown> = {
      '/api/log-config': { retention_days: 5, max_size_mb: 100, api_debug: false },
      '/api/archive-config': { retention_days: 90, max_size_mb: 500 },
      '/api/archive-stats': { total_files: 0, size_bytes: 0, oldest_date: null, newest_date: null },
      '/api/web-config': { runningPort: 7474, accessUrl: 'http://localhost:7474', distPath: '', distFound: true },
      '/api/stock-list-stats': { count: 0, lastUpdatedAt: null, filePath: 'mock', updateIntervalHours: 24 },
      '/api/risk-config': { enabled: true, dailyLossLimit: 500000, maxPositionRatio: 0.2, maxDailyBuyOrdersPerSymbol: 0, maxDailySellOrdersPerSymbol: 1, maxConsecutiveLossesPerStrategySymbol: 3, volatilitySizingEnabled: false, riskPerTradeBps: 100, atrStopMultiplier: 2, currentLoss: 0, dailyProfit: 0, netLoss: 0, emergencyStop: false, canTrade: true, lossRatio: 0, blockedStrategySymbolCount: 0, atrReadySymbolCount: 0 },
    }
    if (url.pathname in settingsPayloads) {
      await route.fulfill({ json: settingsPayloads[url.pathname] })
      return
    }
    if (url.pathname === '/api/strategies') {
      if (options.strategyDelayMs) {
        await new Promise((resolve) => setTimeout(resolve, options.strategyDelayMs))
      }
      await route.fulfill({ json: strategies.map((item) => ({
        ...item,
        brokerAccountId: activeAccountId(),
        orderQuantity: options.scopeController?.current === 'B' ? 9 : item.orderQuantity,
      })) })
      return
    }
    if (url.pathname === '/api/profiles' && route.request().method() === 'GET' && options.scopeController) {
      await route.fulfill({ json: [
        { id: 'paper', broker_id: 'kis', broker_account_id: '12345678-01', name: '계좌 A', is_paper_trading: true, live_trading_consent: false, app_key_masked: '***', account_no: '12345678-01', is_active: options.scopeController.current === 'A', is_configured: true },
        { id: 'profile-b', broker_id: 'kis', broker_account_id: '87654321-01', name: '계좌 B', is_paper_trading: true, live_trading_consent: false, app_key_masked: '***', account_no: '87654321-01', is_active: options.scopeController.current === 'B', is_configured: true },
      ] })
      return
    }
    if (url.pathname === '/api/profiles/profile-b/set-active' && options.scopeController) {
      options.scopeController.current = 'B'
      await route.fulfill({ json: {
        active_broker_id: 'kis', active_broker_account_id: '87654321-01', kis_app_key_masked: '***', kis_account_no: '87654321-01', kis_is_paper_trading: true, kis_configured: true, active_broker_configured: true, discord_enabled: false, notification_levels: [], active_profile_id: 'profile-b', active_profile_name: '계좌 B',
      } })
      return
    }
    if (url.pathname.startsWith('/api/strategies/') && route.request().method() === 'POST') {
      options.updateRequests?.push(route.request().postDataJSON())
      await route.fulfill({ json: strategies.find((item) => url.pathname.endsWith(item.id)) ?? strategies[0] })
      return
    }
    if (url.pathname === '/api/toss-market-calendar') {
      const day = {
        date: '2026-07-07',
        daySession: { startTime: '09:00', endTime: '16:50' },
        preSession: { startTime: '17:00', endTime: '22:30' },
        regularSession: { startTime: '22:30', endTime: '05:00' },
        afterSession: { startTime: '05:00', endTime: '07:00' },
        isDayOpen: true,
        isPreOpen: false,
        isRegularOpen: false,
        isAfterOpen: false,
      }
      await route.fulfill({
        json: {
          brokerId: 'toss',
          kr: { ...day, preSession: null, afterSession: null },
          us: day,
          summary: 'mock toss calendar',
        },
      })
      return
    }
    if (url.pathname.startsWith('/api/chart/') || url.pathname.startsWith('/api/toss-chart/')) {
      options.chartRequests?.push(url.toString())
      await route.fulfill({
        json: options.chartCandles ?? [
          { date: '20260701', open: '100', high: '102', low: '99', close: '100', volume: '1000' },
          { date: '20260702', open: '100', high: '105', low: '100', close: '104', volume: '1500' },
          { date: '20260703', open: '104', high: '108', low: '103', close: '107', volume: '1800' },
          { date: '20260704', open: '107', high: '109', low: '105', close: '106', volume: '1200' },
        ],
      })
      return
    }
    if (url.pathname === '/api/strategy/preview') {
      const body = route.request().postDataJSON() as Record<string, unknown>
      options.genericPreviewRequests?.push(body)
      if (options.genericPreviewDelayMs) {
        await new Promise((resolve) => setTimeout(resolve, options.genericPreviewDelayMs))
      }
      await route.fulfill({
        json: {
          strategyId: body.strategyId,
          symbol: body.symbol,
          candles: body.candles,
          signals: [
            {
              time: '20260702',
              side: 'buy',
              price: 104,
              quantity: 1,
              reason: 'mock generic buy',
            },
            {
              time: '20260704',
              side: 'sell',
              price: 106,
              quantity: 1,
              reason: 'mock generic sell',
            },
          ],
          generatedAt: '2026-07-04T15:30:00+09:00',
          message: 'mock generic preview signals',
          ...mockResearchResult(body, String(body.interval ?? '1d')),
          preparation: options.genericPreparation,
        },
      })
      return
    }
    if (url.pathname === '/api/strategy/leveraged-trend-hold/preview') {
      const body = route.request().postDataJSON() as Record<string, unknown>
      options.previewRequests?.push(body)
      if (options.leveragedPreviewDelayMs) {
        await new Promise((resolve) => setTimeout(resolve, options.leveragedPreviewDelayMs))
      }
      await route.fulfill({
        json: {
          symbol: body.symbol,
          interval: body.interval ?? '1m',
          candleCount: options.leveragedZero ? 0 : 3,
          candles: options.leveragedZero ? [] : [
            { date: '20260707170100', open: '100', high: '101', low: '99', close: '100', volume: '1200' },
            { date: '20260707170200', open: '100', high: '104', low: '100', close: '103', volume: '1500' },
            { date: '20260707170300', open: '103', high: '106', low: '102', close: '105', volume: '1800' },
          ],
          signals: body.interval === '1d' || options.leveragedZero ? [] : [
            {
              time: body.interval === '1d' ? '20260707223500' : '20260707170200',
              chartTime: body.interval === '1d' ? '20260707' : undefined,
              side: 'buy',
              price: 103,
              quantity: 1,
              reason: 'mock rebound buy',
              emaShort: 101,
              emaLong: 100,
              rsi: 55,
              adx: 25,
            },
            {
              time: body.interval === '1d' ? '20260708050000' : '20260707170300',
              chartTime: body.interval === '1d' ? '20260707' : undefined,
              side: 'sell',
              price: 105,
              quantity: 1,
              reason: 'mock trend exit',
              emaShort: 103,
              emaLong: 101,
              rsi: 48,
              adx: 22,
            },
          ],
          generatedAt: '2026-07-07T17:03:00+09:00',
          message: 'mock preview signals',
          ...mockResearchResult(body, String(body.interval ?? '1m')),
          replay: {
            ...mockResearchResult(body, String(body.interval ?? '1m')).replay,
            ...(!options.leveragedLegacy ? { assessment: {
              model: body.interval === '1d' ? 'dailyDiagnostic' : 'intradaySample',
              performanceStatus: body.interval === '1d' || options.leveragedZero ? 'notEvaluable' : 'sampleOnly',
              dailyContextBars: body.interval === '1d' ? 3 : 0, intradayBars: body.interval === '1d' || options.leveragedZero ? 0 : 3,
              timestampStatus: 'unverified', sessionStatus: 'unverified', limitations: ['시각·세션 검증 전 표본'],
            } } : {}),
          },
        },
      })
      return
    }
    await route.fulfill({ json: {} })
  })
}
