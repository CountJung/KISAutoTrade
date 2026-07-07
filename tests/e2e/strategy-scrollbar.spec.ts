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
]

function strategy(id: string, name: string, index: number) {
  const isLeveraged = id === 'leveraged_trend_hold_default'
  return {
    id,
    name,
    enabled: false,
    brokerId: 'kis',
    brokerAccountId: '12345678-01',
    targetSymbols: isLeveraged ? ['SOXL'] : [`0000${index}`],
    targetSymbolNames: isLeveraged ? { SOXL: 'Direxion Daily Semiconductor Bull 3X' } : {},
    orderQuantity: 1,
    params: isLeveraged
      ? { entries: strategyEntries, upward_sensitivity: 1 }
      : {},
  }
}

async function mockApi(page: import('@playwright/test').Page, options: { strategyDelayMs?: number } = {}) {
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
          active_broker_id: 'kis',
          active_broker_account_id: '12345678-01',
          kis_app_key_masked: '***',
          kis_account_no: '12345678-01',
          kis_is_paper_trading: true,
          kis_configured: true,
          active_broker_configured: true,
          discord_enabled: false,
          notification_levels: [],
          active_profile_id: 'paper',
          active_profile_name: '모의',
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
  await expect(page.getByText('SOXL')).toBeVisible()
  await expect(page.getByRole('button', { name: '대상 추가' }).first()).toBeVisible()
  await expect(page.getByText('운용 모드')).toHaveCount(0)
  await expect(page.getByText('기초지수')).toHaveCount(0)
  await expect(page.getByText('숏 실험')).toHaveCount(0)
})
