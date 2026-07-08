import { expect, test } from '@playwright/test'

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
  return {
    id,
    name,
    enabled: false,
    brokerId: 'kis',
    brokerAccountId: '12345678-01',
    targetSymbols: isLeveraged ? ['SOXL', 'KORU'] : [`0000${index}`],
    targetSymbolNames: isLeveraged
      ? {
        SOXL: 'Direxion Daily Semiconductor Bull 3X',
        KORU: 'Direxion Daily South Korea Bull 3X',
      }
      : {},
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
      }
      : {},
  }
}

type MockOptions = {
  strategyDelayMs?: number
  activeBroker?: 'kis' | 'toss'
  previewRequests?: unknown[]
}

async function mockApi(page: import('@playwright/test').Page, options: MockOptions = {}) {
  const activeBroker = options.activeBroker ?? 'kis'
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
  ]

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
          active_broker_account_id: activeBroker === 'toss' ? '1' : '12345678-01',
          kis_app_key_masked: '***',
          kis_account_no: '12345678-01',
          kis_is_paper_trading: true,
          kis_configured: true,
          active_broker_configured: true,
          discord_enabled: false,
          notification_levels: [],
          active_profile_id: activeBroker === 'toss' ? 'toss-live' : 'paper',
          active_profile_name: activeBroker === 'toss' ? 'Toss 실전' : '모의',
        },
      })
      return
    }
    if (url.pathname === '/api/trading/status') {
      await route.fulfill({
        json: {
          isRunning: false,
          activeStrategies: [],
          positionCount: 0,
          totalUnrealizedPnl: 0,
          wsConnected: false,
          tradingProfileId: null,
          tradingBrokerId: null,
          tradingAccountId: null,
          buySuspended: false,
          buySuspendedReason: null,
        },
      })
      return
    }
    if (url.pathname === '/api/strategies') {
      if (options.strategyDelayMs) {
        await new Promise((resolve) => setTimeout(resolve, options.strategyDelayMs))
      }
      await route.fulfill({ json: strategies })
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
    if (url.pathname === '/api/strategy/leveraged-trend-hold/preview') {
      const body = route.request().postDataJSON()
      options.previewRequests?.push(body)
      await route.fulfill({
        json: {
          symbol: body.symbol,
          candles: [
            { date: '20260707170100', open: '100', high: '101', low: '99', close: '100', volume: '1200' },
            { date: '20260707170200', open: '100', high: '104', low: '100', close: '103', volume: '1500' },
            { date: '20260707170300', open: '103', high: '106', low: '102', close: '105', volume: '1800' },
          ],
          signals: [
            {
              time: '20260707170200',
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
              time: '20260707170300',
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
        },
      })
      return
    }
    await route.fulfill({ json: {} })
  })
}

test('Strategy initial viewport keeps visible main scrollbar gutter', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  const main = page.getByTestId('app-main-scroll')
  await expect(main).toBeVisible()
  await expect(page.getByText('레버리지 대상 ETF')).toBeVisible()
  await expect(page.getByTestId('app-main-scroll-rail')).toBeVisible()
  await expect(page.getByTestId('app-main-scroll-thumb')).toBeVisible()

  const metrics = await main.evaluate((el) => {
    const style = window.getComputedStyle(el)
    return {
      overflowY: style.overflowY,
      scrollbarGutter: style.scrollbarGutter,
      scrollHeight: el.scrollHeight,
      clientHeight: el.clientHeight,
    }
  })
  const thumbHeight = await page.getByTestId('app-main-scroll-thumb').evaluate((el) => el.getBoundingClientRect().height)

  expect(metrics.overflowY).toBe('scroll')
  expect(metrics.scrollbarGutter).toContain('stable')
  expect(metrics.scrollHeight).toBeGreaterThan(metrics.clientHeight)
  expect(thumbHeight).toBeGreaterThan(0)
})

test('Strategy scrollbar appears after delayed strategy content loads', async ({ page }) => {
  await mockApi(page, { strategyDelayMs: 250 })
  await page.goto('/strategy')

  const main = page.getByTestId('app-main-scroll')
  await expect(main).toBeVisible()
  await expect(page.getByText('레버리지 대상 ETF')).toBeVisible()
  await expect(page.getByTestId('app-main-scroll-rail')).toBeVisible()
  await expect(page.getByTestId('app-main-scroll-thumb')).toBeVisible()

  await expect
    .poll(() => main.evaluate((el) => el.scrollHeight > el.clientHeight))
    .toBe(true)
})

test('main scrollbar thumb can be dragged with a pointer', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  const main = page.getByTestId('app-main-scroll')
  const thumb = page.getByTestId('app-main-scroll-thumb')
  await expect(main).toBeVisible()
  await expect(thumb).toBeVisible()

  const before = await main.evaluate((el) => el.scrollTop)
  const box = await thumb.boundingBox()
  expect(box).not.toBeNull()
  if (!box) return

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 160, { steps: 8 })
  await page.mouse.up()

  await expect
    .poll(() => main.evaluate((el) => el.scrollTop))
    .toBeGreaterThan(before + 20)
})

test('Leveraged strategy editor uses single target ticker model', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  await expect(page.getByText('레버리지 대상 ETF')).toBeVisible()
  await expect(page.getByText('SOXL', { exact: true })).toBeVisible()
  await expect(page.getByLabel('초기 손절(%)')).toBeVisible()
  await expect(page.getByLabel('실패 판정 관측치')).toBeVisible()
  await expect(page.getByLabel('추적손절(%)')).toBeVisible()
  await expect(page.getByLabel('추적 활성 수익(%)')).toBeVisible()
  await expect(page.getByLabel('본전 보호 버퍼(%)')).toBeVisible()
  await expect(page.getByLabel('최소 보유 관측치')).toBeVisible()
  await expect(page.getByRole('button', { name: '대상 추가' }).first()).toBeVisible()
  await expect(page.getByText('운용 모드')).toHaveCount(0)
  await expect(page.getByText('기초지수')).toHaveCount(0)
  await expect(page.getByText('숏 실험')).toHaveCount(0)
})

test('Leveraged strategy preview can switch between configured tickers', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  await expect(page.getByText('전략 미리보기')).toBeVisible()
  const tickerSelect = page.getByRole('combobox', { name: '시뮬레이션 티커' })
  await expect(tickerSelect).toContainText('SOXL')

  await tickerSelect.click()
  await page.getByRole('option', { name: /KORU/ }).click()

  await expect(tickerSelect).toContainText('KORU')
})

test('Leveraged strategy preview runs selected ticker and renders signal chart', async ({ page }) => {
  const previewRequests: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', previewRequests })
  await page.goto('/strategy')

  await page.getByRole('button', { name: '미리보기 계산' }).click()

  await expect(page.getByText('mock preview signals')).toBeVisible()
  await expect(page.getByTestId('lth-preview-chart')).toBeVisible()
  await expect(page.getByTestId('lth-preview-chart').locator('canvas').first()).toBeVisible()
  await expect(page.getByText(/매수 07\/07 17:02/)).toBeVisible()
  await expect(page.getByText(/매도 07\/07 17:03/)).toBeVisible()
  expect(previewRequests).toHaveLength(1)
  expect(previewRequests[0]).toMatchObject({ symbol: 'SOXL' })
})
