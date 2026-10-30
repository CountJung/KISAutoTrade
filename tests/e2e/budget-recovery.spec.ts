import { expect, test, type Page } from '@playwright/test'
import type { BudgetRecoveryItem } from '../../src/shared/api/budgetRecoveryTypes'

const original: BudgetRecoveryItem = {
  reservationId: 'auto-budget-fixture', currency: 'KRW', market: 'kr', exchange: null, symbol: '005930', side: 'buy',
  quantity: 10, orderPrice: 100, filledQuantity: 0, filledNotional: 0, reservedAmount: 1010,
  submittedAt: '2026-10-29T10:00:00+09:00', profileId: 'fixture-profile', isPaper: true,
  clientOrderId: 'fixture-client-id', providerOrderId: null, evidenceReference: null, snapshotRevision: 'fixture-revision',
  state: 'unresolved', blockedReason: null,
}

async function mockNative(page: Page, options: { item?: Partial<BudgetRecoveryItem>; running?: boolean; failFirst?: boolean; failAfterIntent?: boolean; totalCount?: number } = {}) {
  // All native invocations are intercepted in the browser. No desktop app or broker is started.
  await page.route('**/api/**', (route) => new URL(route.request().url()).pathname.startsWith('/api/') ? route.abort() : route.fallback())
  await page.addInitScript(({ item, running, failFirst, failAfterIntent, totalCount }) => {
    const calls: Array<{ command: string; args: Record<string, unknown> }> = []
    const view = { scope: { brokerId: 'kis', accountId: 'fixture-account' }, items: [item], totalCount }
    let attempts = 0
    const config = { active_broker_id: 'kis', active_broker_account_id: 'fixture-account', active_profile_id: 'fixture-profile',
      active_profile_name: '검증 전용 프로파일', kis_app_key_masked: '', kis_account_no: '', kis_is_paper_trading: true,
      kis_configured: false, active_broker_configured: false, discord_enabled: false, notification_levels: [] }
    const currency = { allocatedAmount: 2020, cashAmount: 2020, reservedAmount: 1010, availableAmount: 0,
      ownedPositionCount: 0, blockedReason: '응답 불명 주문', feeBufferBps: 100 }
    Object.assign(window, {
      __recoveryCalls: calls,
      __TAURI_INTERNALS__: {
        transformCallback: () => 1,
        unregisterCallback: () => {},
        invoke: async (command: string, args: Record<string, unknown> = {}) => {
          calls.push({ command, args })
          if (command === 'get_app_config') return structuredClone(config)
          if (command === 'get_trading_status') return { isRunning: running, activeStrategies: [], positionCount: 0, wsConnected: false, totalUnrealizedPnl: 0 }
          if (command === 'get_auto_trading_budget') return { scope: view.scope, krw: currency, usd: { ...currency, reservedAmount: 0 } }
          if (command === 'get_budget_recovery') return structuredClone(view)
          if (command === 'approve_budget_recovery') {
            attempts += 1
            if (failAfterIntent && attempts === 1) {
              const input = args.input as { providerOrderId: string; evidenceReference: string }
              view.items[0] = { ...item, state: 'awaitingReconciliation', providerOrderId: input.providerOrderId, evidenceReference: input.evidenceReference }
              throw { code: 'FIXTURE_SAVE_FAILURE', message: '검증용 pending 저장 실패: 승인 의도 보존' }
            }
            if (failFirst && attempts === 1) throw { code: 'FIXTURE_SAVE_FAILURE', message: '검증용 저장 실패: 보류 유지' }
            const input = args.input as { providerOrderId: string }
            view.items[0] = { ...item, state: 'awaitingReconciliation', providerOrderId: input.providerOrderId }
            return structuredClone(view)
          }
          if (['list_profiles', 'get_profiles', 'get_broker_rate_limit_status', 'get_positions', 'get_pending_orders', 'get_strategies', 'get_recent_logs'].includes(command)) return []
          if (command === 'get_refresh_config') return { interval_sec: 30 }
          if (command === 'get_refresh_interval') return 30
          if (command === 'get_exchange_rate') return 1400
          if (command === 'get_log_config') return { retention_days: 5, max_size_mb: 100, api_debug: false }
          if (command === 'get_trade_archive_config') return { retention_days: 90, max_size_mb: 500 }
          if (command.startsWith('plugin:')) return 1
          return null
        },
      },
    })
  }, { item: { ...original, ...options.item }, running: options.running ?? false, failFirst: options.failFirst ?? false, failAfterIntent: options.failAfterIntent ?? false, totalCount: options.totalCount ?? 1 })
  await page.goto('/settings')
  await expect(page.getByRole('heading', { name: '응답 불명 주문 대조·복구', exact: true })).toBeVisible()
}

async function evidence(page: Page) {
  await page.getByLabel('대조한 증권사 주문번호', { exact: true }).fill('fixture-provider-1')
  await page.getByLabel('원본과 대조한 clientOrderId', { exact: true }).fill('fixture-client-id')
  await page.getByLabel('대조 근거 참조', { exact: true }).fill('비식별 합성 fixture 1')
  await page.getByRole('checkbox', { name: /원래 계좌·프로파일/ }).check()
}

async function approvals(page: Page) {
  return page.evaluate(() => (window as unknown as { __recoveryCalls: Array<{ command: string; args: unknown }> }).__recoveryCalls.filter((call) => call.command === 'approve_budget_recovery'))
}

test('desktop requires matching evidence and explicit confirmation; binding does not claim a fill', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  await mockNative(page)
  const approve = page.getByRole('button', { name: '대조 승인 후 미체결 복원' })
  await expect(approve).toBeDisabled()
  await evidence(page)
  await expect(approve).toBeEnabled()
  await page.getByLabel('대조한 증권사 주문번호', { exact: true }).fill('fixture-provider-2')
  await expect(approve).toBeDisabled()
  await page.getByRole('checkbox', { name: /원래 계좌·프로파일/ }).check()
  await approve.click()
  await expect(page.getByText(/미체결 대조 대상으로 연결됐습니다/)).toBeVisible()
  expect(await approvals(page)).toEqual([{ command: 'approve_budget_recovery', args: { input: {
    brokerId: 'kis', brokerAccountId: 'fixture-account', reservationId: original.reservationId,
    snapshotRevision: original.snapshotRevision, providerOrderId: 'fixture-provider-2',
    clientOrderId: original.clientOrderId, evidenceReference: '비식별 합성 fixture 1', confirmed: true,
  } } }])
  expect(errors).toEqual([])
})

test('storage failure keeps review inputs and permits the same explicit retry', async ({ page }) => {
  await mockNative(page, { failFirst: true })
  await evidence(page)
  const approve = page.getByRole('button', { name: '대조 승인 후 미체결 복원' })
  await approve.click()
  await expect(page.getByText('검증용 저장 실패: 보류 유지')).toBeVisible()
  await expect(page.getByLabel('대조한 증권사 주문번호', { exact: true })).toHaveValue('fixture-provider-1')
  await approve.click()
  await expect(page.getByText(/미체결 대조 대상으로 연결됐습니다/)).toBeVisible()
  const calls = await approvals(page)
  expect(calls).toHaveLength(2)
  expect(calls[0].args).toEqual(calls[1].args)
})

test('legacy entries never expose approval', async ({ page }) => {
  await mockNative(page, { item: { snapshotRevision: null, profileId: null, isPaper: null, blockedReason: '기존 예약에 복구 근거가 없습니다.' } })
  await expect(page.getByText('기존 예약에 복구 근거가 없습니다.')).toBeVisible()
  await expect(page.getByRole('button', { name: '대조 승인 후 미체결 복원' })).toHaveCount(0)
  expect(await approvals(page)).toHaveLength(0)
})

test('running trading disables recovery approval and evidence editing', async ({ page }) => {
  await mockNative(page, { running: true })
  await expect(page.getByLabel('대조한 증권사 주문번호', { exact: true })).toBeDisabled()
  await expect(page.getByRole('button', { name: '대조 승인 후 미체결 복원' })).toBeDisabled()
  expect(await approvals(page)).toHaveLength(0)
})

test('mobile recovery cards remain readable and mismatched client ID cannot be approved', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await mockNative(page, { totalCount: 101 })
  await page.getByLabel('대조한 증권사 주문번호', { exact: true }).fill('fixture-provider-1')
  await page.getByLabel('원본과 대조한 clientOrderId', { exact: true }).fill('wrong')
  await page.getByLabel('대조 근거 참조', { exact: true }).fill('fixture')
  await expect(page.getByRole('checkbox', { name: /원래 계좌·프로파일/ })).toBeDisabled()
  await expect(page.getByText(/표시 한도를 넘는 기록/)).toBeVisible()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
  expect(await approvals(page)).toHaveLength(0)
})

test('unauthenticated web mode exposes no recovery action or request', async ({ page }) => {
  const recoveryRequests: string[] = []
  await page.route('**/api/**', async (route) => {
    const path = new URL(route.request().url()).pathname
    if (!path.startsWith('/api/')) { await route.fallback(); return }
    if (path.includes('recovery')) recoveryRequests.push(path)
    const payload = path === '/api/app-config' ? { active_broker_id: 'kis', active_broker_account_id: null, active_profile_id: null }
      : ['/api/profiles', '/api/broker-rate-limit-status', '/api/strategies', '/api/positions', '/api/recent-logs'].includes(path) ? [] : null
    await route.fulfill({ json: payload })
  })
  await page.goto('/settings')
  await expect(page.getByText('불명 주문 복구는 Tauri 데스크톱 앱에서만 사용할 수 있습니다.')).toBeVisible()
  await expect(page.getByRole('button', { name: '대조 승인 후 미체결 복원' })).toHaveCount(0)
  expect(recoveryRequests).toEqual([])
})

for (const mismatch of [{ profileId: 'another-profile' }, { isPaper: false }]) {
  test(`different identity blocks approval: ${JSON.stringify(mismatch)}`, async ({ page }) => {
    await mockNative(page, { item: mismatch })
    await expect(page.getByText('원래 프로파일 또는 투자 모드와 다릅니다. 보류를 유지합니다.')).toBeVisible()
    await expect(page.getByRole('button', { name: '대조 승인 후 미체결 복원' })).toHaveCount(0)
    expect(await approvals(page)).toHaveLength(0)
  })
}

test('terminal receipt cannot be approved again', async ({ page }) => {
  await mockNative(page, { item: { state: 'resolved', providerOrderId: 'fixture-provider-1' } })
  await expect(page.getByText(/복구 승인 기록이 보존된 종결 항목/)).toBeVisible()
  await expect(page.getByRole('button', { name: '대조 승인 후 미체결 복원' })).toHaveCount(0)
  expect(await approvals(page)).toHaveLength(0)
})

test('restart restores durable approval evidence without silently reconfirming it', async ({ page }) => {
  await mockNative(page, { item: { providerOrderId: 'fixture-provider-1', evidenceReference: '비식별 합성 fixture 1' } })
  await expect(page.getByText(/저장된 대조 승인이 있습니다/)).toBeVisible()
  await expect(page.getByLabel('대조한 증권사 주문번호', { exact: true })).toHaveValue('fixture-provider-1')
  await expect(page.getByLabel('원본과 대조한 clientOrderId', { exact: true })).toHaveValue('fixture-client-id')
  await expect(page.getByLabel('대조 근거 참조', { exact: true })).toHaveValue('비식별 합성 fixture 1')
  await expect(page.getByLabel('대조 근거 참조', { exact: true })).toBeDisabled()
  const approve = page.getByRole('button', { name: '대조 승인 후 미체결 복원' })
  await expect(approve).toBeDisabled()
  await page.getByRole('checkbox', { name: /원래 계좌·프로파일/ }).check()
  await approve.click()
  await expect(page.getByText(/미체결 대조 대상으로 연결됐습니다/)).toBeVisible()
  const calls = await approvals(page)
  expect(calls).toHaveLength(1)
  expect(calls[0].args).toMatchObject({ input: { providerOrderId: 'fixture-provider-1', clientOrderId: 'fixture-client-id', evidenceReference: '비식별 합성 fixture 1' } })
})

test('refresh after pending save failure keeps the durable intent retry available', async ({ page }) => {
  await mockNative(page, { failAfterIntent: true })
  await evidence(page)
  await page.getByRole('button', { name: '대조 승인 후 미체결 복원' }).click()
  await expect(page.getByText('검증용 pending 저장 실패: 승인 의도 보존')).toBeVisible()
  await page.getByRole('button', { name: '불명 주문 다시 조회' }).click()
  const retry = page.getByRole('button', { name: '같은 승인으로 미체결 저장 재시도' })
  await expect(retry).toBeDisabled()
  await expect(page.getByLabel('대조 근거 참조', { exact: true })).toHaveValue('비식별 합성 fixture 1')
  await page.getByRole('checkbox', { name: /원래 계좌·프로파일/ }).check()
  await retry.click()
  await expect.poll(async () => (await approvals(page)).length).toBe(2)
  const calls = await approvals(page)
  expect(calls[0].args).toEqual(calls[1].args)
})
