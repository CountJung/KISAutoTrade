import { expect, test } from '@playwright/test'

async function mockSettingsApi(page: import('@playwright/test').Page) {
  const budget = {
    scope: { brokerId: 'kis', accountId: '12345678-01' },
    krw: { allocatedAmount: 0, cashAmount: 0, reservedAmount: 0, availableAmount: 0, ownedPositionCount: 0, blockedReason: null, feeBufferBps: 100 },
    usd: { allocatedAmount: 0, cashAmount: 0, reservedAmount: 0, availableAmount: 0, ownedPositionCount: 0, blockedReason: null, feeBufferBps: 100 },
  }
  await page.route('**/api/**', async (route) => {
    const path = new URL(route.request().url()).pathname
    if (!path.startsWith('/api/')) {
      await route.fallback()
      return
    }
    if (path === '/api/auto-trading-budget') {
      if (route.request().method() === 'POST') {
        const input = route.request().postDataJSON()
        for (const [key, value] of [['krw', input.krwAmount], ['usd', input.usdAmount]] as const) {
          budget[key] = { ...budget[key], allocatedAmount: value, cashAmount: value, availableAmount: value }
        }
      }
      await route.fulfill({ json: budget })
      return
    }
    const payloads: Record<string, unknown> = {
      '/api/app-config': {
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
      '/api/trading/status': {
        isRunning: false,
        activeStrategies: [],
        positionCount: 0,
        totalUnrealizedPnl: 0,
        wsConnected: false,
      },
      '/api/check-config': {
        broker_id: 'kis',
        broker_account_id: '12345678-01',
        real_key_set: false,
        real_account_set: false,
        paper_key_set: true,
        active_mode: 'paper',
        is_ready: true,
        discord_configured: false,
        base_url: 'https://example.invalid',
        issues: [],
      },
      '/api/log-config': { retention_days: 5, max_size_mb: 100, api_debug: false },
      '/api/archive-config': { retention_days: 90, max_size_mb: 500 },
      '/api/archive-stats': { total_files: 0, size_bytes: 0, oldest_date: null, newest_date: null },
      '/api/profiles': [],
      '/api/web-config': { runningPort: 7474, accessUrl: 'http://localhost:7474', distPath: '', distFound: true },
      '/api/stock-list-stats': { count: 0, lastUpdatedAt: null, filePath: 'data/stocklist/stocklist.json', updateIntervalHours: 24 },
      '/api/risk-config': {
        enabled: true,
        dailyLossLimit: 500000,
        maxPositionRatio: 0.2,
        maxDailyBuyOrdersPerSymbol: 0,
        maxDailySellOrdersPerSymbol: 1,
        maxConsecutiveLossesPerStrategySymbol: 3,
        volatilitySizingEnabled: false,
        riskPerTradeBps: 100,
        atrStopMultiplier: 2,
        currentLoss: 0,
        dailyProfit: 0,
        netLoss: 0,
        emergencyStop: false,
        canTrade: true,
        lossRatio: 0,
        blockedStrategySymbolCount: 0,
        atrReadySymbolCount: 0,
      },
      '/api/check-update': { hasUpdate: false, currentVersion: '0.2.0', latestVersion: '0.2.0', releaseUrl: '' },
      '/api/recent-logs': [],
    }
    await route.fulfill({ json: payloads[path] ?? {} })
  })
}

test('Database administration stays desktop-only in unauthenticated web mode', async ({ page }) => {
  await mockSettingsApi(page)
  await page.goto('/settings')

  await expect(page.getByText('데이터베이스 및 데이터 이관')).toBeVisible()
  await expect(page.getByText(/Tauri 데스크톱 앱 Settings에서만/)).toBeVisible()
  await expect(page.getByLabel('Password')).toHaveCount(0)
  await expect(page.getByRole('button', { name: '앱 테이블 삭제' })).toHaveCount(0)
})

test('Database Settings safety notice does not overflow a narrow viewport', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await mockSettingsApi(page)
  await page.goto('/settings')

  await expect(page.getByText('데이터베이스 및 데이터 이관')).toBeVisible()
  await expect(page.getByText(/Tauri 데스크톱 앱 Settings에서만/)).toBeVisible()
  const metrics = await page.evaluate(() => ({ scrollWidth: document.documentElement.scrollWidth, width: window.innerWidth }))
  expect(metrics.scrollWidth).toBeLessThanOrEqual(metrics.width)
})

test('Dedicated trading budget saves scoped KRW and USD cents and validates input', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await mockSettingsApi(page)
  await page.goto('/settings')
  const krw = page.getByLabel('자동매매 원화 배정액 (원)')
  const usd = page.getByLabel('자동매매 달러 배정액 (USD)')
  const save = page.getByRole('button', { name: '자동매매 예산 저장' })
  await expect(krw).toHaveValue('0')
  await expect(save).toBeDisabled()
  await krw.fill('-1')
  await expect(save).toBeDisabled()
  await krw.fill('1000000')
  await usd.fill('100.501')
  await expect(save).toBeDisabled()
  await usd.fill('100.50')
  const request = page.waitForRequest((r) => r.url().includes('/api/auto-trading-budget') && r.method() === 'POST')
  await save.click()
  expect((await request).postDataJSON()).toEqual({ brokerId: 'kis', brokerAccountId: '12345678-01', krwAmount: 1000000, usdAmount: 10050 })
  await expect(page.getByText('자동매매 예산을 저장했습니다.')).toBeVisible()
  await expect(page.getByText('사용 가능액: ₩1,000,000')).toBeVisible()
  await page.reload()
  await expect(krw).toHaveValue('1000000')
  await expect(usd).toHaveValue('100.50')
  const metrics = await page.evaluate(() => ({ width: window.innerWidth, content: document.documentElement.scrollWidth }))
  expect(metrics.content).toBeLessThanOrEqual(metrics.width)
})

test('Dedicated budget cannot be changed while trading or when status is unavailable', async ({ page }) => {
  await mockSettingsApi(page)
  await page.route('**/api/trading/status', (route) => route.fulfill({ json: { isRunning: true } }))
  await page.goto('/settings')
  await expect(page.getByLabel('자동매매 원화 배정액 (원)')).toBeDisabled()
  await expect(page.getByRole('button', { name: '자동매매 예산 저장' })).toBeDisabled()
})
