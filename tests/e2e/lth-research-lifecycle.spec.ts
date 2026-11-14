import { expect, test, type Page } from '@playwright/test'
import { mockApi as fixtureMockApi, syntheticCollectionSummary } from './fixtures/strategyResearchFixture'

type ApiAudit = { observed: string[]; expected: string[] }
const apiAudits = new WeakMap<Page, ApiAudit>()
async function mockApi(page: Page, options: NonNullable<Parameters<typeof fixtureMockApi>[1]> = {}) {
  const audit = apiAudits.get(page)
  if (!audit) throw new Error('Research API audit was not initialized')
  // Preserve the original mutable options object used by cancellation retry controls.
  await fixtureMockApi(page, Object.assign(options, { strictResearchApi: true, unexpectedApiRequests: audit.observed }))
}

test.beforeEach(async ({ page }) => {
  apiAudits.set(page, { observed: [], expected: [] })
  await page.route('**/*', (route) => {
    const url = new URL(route.request().url())
    return ['127.0.0.1', 'localhost'].includes(url.hostname) ? route.continue() : route.abort()
  })
})

test.afterEach(async ({ page }) => {
  const audit = apiAudits.get(page)
  expect(audit?.observed ?? []).toEqual(audit?.expected ?? [])
})

const card = (page: Page) => page.locator('.MuiPaper-root').filter({ hasText: 'LeveragedTrendHoldStrategy' }).first()
const calculate = (page: Page) => card(page).getByRole('button', { name: '미리보기 계산', exact: true })
function gate() { let release!: () => void; const promise = new Promise<void>((resolve) => { release = resolve }); return { promise, release } }
type Event = { action: string; body: Record<string, unknown> }

test('research fixture rejects unknown API wrong method financial mutation and wrong scope', async ({ page }) => {
  const events: Event[] = []; const previews: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', researchEvents: events, previewRequests: previews })
  await page.goto('/strategy'); await expect(calculate(page)).toBeEnabled()
  const audit = apiAudits.get(page)!
  audit.expected.push('GET /api/not-registered-research-probe', 'POST /api/trading/start', 'GET /api/strategy/leveraged-trend-hold/research/run')
  for (const [method, path] of [
    ['GET', '/api/not-registered-research-probe'],
    ['POST', '/api/trading/start'],
    ['GET', '/api/strategy/leveraged-trend-hold/research/run'],
  ]) {
    const result = await page.evaluate(async ({ method, path }) => {
      const response = await fetch(path, { method })
      return { status: response.status, body: await response.json() }
    }, { method, path })
    expect(result.status).toBe(501)
    expect(result.body.code).toBe('UNEXPECTED_MOCK_API')
  }
  const wrongScope = await page.evaluate(async () => {
    const response = await fetch('/api/strategy/leveraged-trend-hold/research/reserve', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ expectedPaperTrading: true, expectedBrokerId: 'toss',
        preview: { expectedProfileId: 'not-active', expectedBrokerAccountId: 'wrong-account', symbol: 'KORU', params: {} } }),
    })
    return response.json()
  })
  expect(wrongScope.code).toBe('LTH_RESEARCH_REJECTED')
  expect(wrongScope.jobId).toBeUndefined()
  expect(events.map((event) => event.action)).toEqual(['reserve'])
  expect(previews).toHaveLength(0)
})

test('US1m reserve/run uses owner-only DTO and same-task double click owns one job', async ({ page }) => {
  const events: Event[] = []; const run = gate()
  await mockApi(page, { activeBroker: 'toss', researchEvents: events, researchRunGate: run.promise })
  await page.goto('/strategy')
  await calculate(page).evaluate((button: HTMLButtonElement) => { button.click(); button.click() })
  await expect(card(page).getByRole('status')).toHaveText('수집·계산 중...')
  expect(events.map((e) => e.action)).toEqual(['reserve', 'run'])
  expect(events[0].body.expectedPaperTrading).toBe(true)
  expect(Object.keys(events[1].body).sort()).toEqual(['jobId', 'owner'])
  run.release()
  await expect(card(page).getByTestId('replay-assessment-notice')).toBeVisible()
  await expect(calculate(page)).toBeEnabled()
})

test('manual cancel keeps gate closed until cancellation ACK and ignores late run', async ({ page }) => {
  const events: Event[] = []; const run = gate(); const cancel = gate()
  await mockApi(page, { activeBroker: 'toss', researchEvents: events, researchRunGate: run.promise, researchCancelGate: cancel.promise })
  await page.goto('/strategy'); await calculate(page).click()
  await expect(card(page).getByRole('status')).toHaveText('수집·계산 중...')
  await card(page).getByRole('button', { name: '연구 취소', exact: true }).click()
  await expect(card(page).getByRole('status')).toHaveText('취소 확인 중...')
  await expect(card(page).getByRole('button', { name: '취소 확인 중...', exact: true })).toBeDisabled()
  cancel.release(); await expect(calculate(page)).toBeEnabled()
  const completed = page.waitForResponse((response) => new URL(response.url()).pathname.endsWith('/research/run'))
  run.release(); await (await completed).finished()
  await expect(card(page).getByTestId('replay-assessment-notice')).toHaveCount(0)
  expect(events.map((e) => e.action)).toEqual(['reserve', 'run', 'cancel'])
})

for (const change of ['input', 'draft', 'unmount'] as const) {
  test(`late reserve after ${change} is cancelled without run`, async ({ page }) => {
    const events: Event[] = []; const reserve = gate()
    await mockApi(page, { activeBroker: 'toss', researchEvents: events, researchReserveGate: reserve.promise })
    await page.goto('/strategy'); await calculate(page).click()
    await expect(card(page).getByRole('status')).toHaveText('예약 중...')
    if (change === 'input') {
      await card(page).getByLabel('분석 구간').click()
      await page.getByRole('option', { name: '최근 50봉', exact: true }).click()
    } else if (change === 'draft') {
      await card(page).getByLabel('수수료(bps)', { exact: true }).fill('12')
    } else {
      await page.getByRole('button', { name: 'Dashboard', exact: true }).click()
    }
    reserve.release()
    await expect.poll(() => events.map((e) => e.action)).toEqual(['reserve', 'cancel'])
    if (change !== 'unmount') {
      await expect(calculate(page)).toBeEnabled()
      await expect(card(page).getByTestId('replay-assessment-notice')).toHaveCount(0)
    }
  })
}

test('cancel failure is explicit and old success cannot release a newer pending owner', async ({ page }) => {
  const events: Event[] = []; const oldRun = gate(); const newRun = gate()
  const oldEntered = gate(); const newEntered = gate(); const jobs: string[] = []
  const options = {
    activeBroker: 'toss' as const, researchEvents: events, researchCancelFails: true,
    researchLateSuccessAfterCancel: true,
    researchRunGateForJob: (id: string) => {
      jobs.push(id)
      if (jobs.length === 1) { oldEntered.release(); return oldRun.promise }
      newEntered.release(); return newRun.promise
    },
  }
  await mockApi(page, options); await page.goto('/strategy'); await calculate(page).click()
  await oldEntered.promise
  await expect(card(page).getByRole('status')).toHaveText('수집·계산 중...')
  await card(page).getByRole('button', { name: '연구 취소', exact: true }).click()
  await expect(card(page).getByText(/취소 확인 실패: 기존 작업/)).toBeVisible()
  await expect(calculate(page)).toBeDisabled()
  expect(events.filter((e) => e.action === 'reserve')).toHaveLength(1)
  options.researchCancelFails = false
  await card(page).getByRole('button', { name: '취소 다시 확인', exact: true }).click()
  await expect(calculate(page)).toBeEnabled()
  await calculate(page).click()
  await newEntered.promise
  await expect(card(page).getByRole('status')).toHaveText('수집·계산 중...')
  expect(jobs).toHaveLength(2)
  const oldResponse = page.waitForResponse((response) => new URL(response.url()).pathname.endsWith('/research/run') && response.request().postDataJSON().jobId === jobs[0])
  oldRun.release()
  const response = await oldResponse
  await response.finished()
  expect((await response.json()).message).toBe('mock preview signals')
  // Two render frames after body completion are a scheduling barrier, not direct
  // instrumentation of the application's response consumer or native IPC.
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(card(page).getByRole('status')).toHaveText('수집·계산 중...')
  const pendingCalculate = card(page).getByRole('button', { name: '수집·계산 중...', exact: true })
  await expect(pendingCalculate).toBeDisabled()
  await pendingCalculate.evaluate((button: HTMLButtonElement) => button.click())
  expect(events.filter((e) => e.action === 'reserve')).toHaveLength(2)
  await expect(card(page).getByTestId('replay-assessment-notice')).toHaveCount(0)
  newRun.release()
  await expect(card(page).getByTestId('replay-assessment-notice')).toBeVisible()
  expect(events.filter((e) => e.action === 'cancel')).toHaveLength(2)
  expect(jobs[0]).not.toBe(jobs[1])
})

test('daily preview retains the old endpoint and has no reservation', async ({ page }) => {
  const events: Event[] = []; const previews: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', researchEvents: events, previewRequests: previews })
  await page.goto('/strategy'); await card(page).getByLabel('봉 단위').click()
  await page.getByRole('option', { name: '일봉', exact: true }).click()
  await calculate(page).click()
  await expect(card(page).getByTestId('strategy-replay-diagnostic')).toBeVisible()
  expect(events).toEqual([]); expect(previews).toHaveLength(1)
})

for (const change of ['mode', 'profile'] as const) {
  test(`${change} A→B→A invalidates late reserve and cannot revive an old result`, async ({ page }) => {
    const events: Event[] = []; const reserve = gate()
    const scope = { current: 'A' as 'A' | 'B' }
    const options = { activeBroker: 'toss' as const, researchEvents: events, researchReserveGate: reserve.promise,
      scopeController: scope, appConfigOverride: { kis_is_paper_trading: true } }
    await mockApi(page, options); await page.goto('/strategy'); await calculate(page).click()
    await expect(card(page).getByRole('status')).toHaveText('예약 중...')
    const publish = async (patch: Record<string, unknown>) => {
      // Synthetic query publication through the actual mounted provider. No source
      // replacement: this harness also runs against CI dist, and does not prove native.
      await card(page).evaluate((button, patch) => {
        type Client = { setQueryData: (key: string[], update: (old: Record<string, unknown>) => Record<string, unknown>) => void }
        type Fiber = { return?: Fiber; memoizedProps?: { client?: Client } }
        const entry = Object.entries(button).find(([key]) => key.startsWith('__reactFiber$'))
        let fiber = entry?.[1] as Fiber | undefined
        while (fiber && !fiber.memoizedProps?.client?.setQueryData) fiber = fiber.return
        const client = fiber?.memoizedProps?.client
        if (!client) throw new Error('Mounted QueryClient provider not found')
        client.setQueryData(['appConfig'], (old) => ({ ...old, ...patch }))
      }, patch)
    }
    if (change === 'mode') await publish({ kis_is_paper_trading: false })
    else await publish({ active_profile_id: 'profile-b', active_broker_account_id: '87654321-01' })
    if (change === 'mode') await expect(card(page).getByRole('status')).toHaveText('취소 확인 중...')
    else {
      // The production strategy card is keyed by profile scope and remounts B.
      await expect(card(page).getByTestId('replay-assessment-notice')).toHaveCount(0)
      await expect(calculate(page)).toBeEnabled()
    }
    await publish({ kis_is_paper_trading: true, active_profile_id: 'toss-live', active_broker_account_id: '1' })
    reserve.release()
    await expect.poll(() => events.map((e) => e.action)).toEqual(['reserve', 'cancel'])
    await expect(calculate(page)).toBeEnabled()
    await expect(card(page).getByTestId('replay-assessment-notice')).toHaveCount(0)
    const cancel = events.find((e) => e.action === 'cancel')!
    expect(Object.keys(cancel.body).sort()).toEqual(['jobId', 'owner'])
  })
}


test('failed research error is invalidated by input change and never revived', async ({ page }) => {
  let runs = 0
  await mockApi(page, { activeBroker: 'toss' })
  await page.route('**/api/strategy/leveraged-trend-hold/research/run', (route) => {
    runs += 1
    return route.fulfill({ json: { code: 'LTH_RESEARCH_FAILED', error: 'synthetic failure' } })
  })
  await page.goto('/strategy'); await calculate(page).click()
  await expect(card(page).getByText('US 분봉 연구 예약·수집·계산에 실패했습니다. 실제 공급·발행·coverage는 미검증입니다.')).toBeVisible()
  await expect(calculate(page)).toBeEnabled()
  await calculate(page).click()
  await expect.poll(() => runs).toBe(2)
  await expect(calculate(page)).toBeEnabled()
  await card(page).getByLabel('분석 구간').click()
  await page.getByRole('option', { name: '최근 50봉', exact: true }).click()
  await expect(card(page).getByText(/US 분봉 연구 예약·수집·계산에 실패했습니다/)).toHaveCount(0)
  await card(page).getByLabel('분석 구간').click()
  await page.getByRole('option', { name: '최근 200봉', exact: true }).click()
  await expect(card(page).getByText(/US 분봉 연구 예약·수집·계산에 실패했습니다/)).toHaveCount(0)
})

test('KR minute preview retains old endpoint without reserve/run', async ({ page }) => {
  const events: Event[] = []; const previews: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', leveragedDomestic: true, researchEvents: events, previewRequests: previews })
  await page.goto('/strategy'); await calculate(page).click()
  await expect(card(page).getByTestId('replay-assessment-notice')).toBeVisible()
  expect(events).toEqual([]); expect(previews).toHaveLength(1)
})

test('lost run response retains owner until cancellation ACK', async ({ page }) => {
  const events: Event[] = []; const cancel = gate()
  await mockApi(page, { activeBroker: 'toss', researchEvents: events, researchCancelGate: cancel.promise })
  await page.route('**/api/strategy/leveraged-trend-hold/research/run', (route) => route.abort())
  await page.goto('/strategy'); await calculate(page).click()
  await expect(card(page).getByRole('status')).toHaveText('취소 확인 중...')
  expect(events.map((e) => e.action)).toEqual(['reserve', 'cancel'])
  await expect(card(page).getByRole('button', { name: '취소 확인 중...', exact: true })).toBeDisabled()
  cancel.release()
  await expect(calculate(page)).toBeEnabled()
  await expect(card(page).getByText(/US 분봉 연구 예약·수집·계산에 실패했습니다/)).toBeVisible()
})

for (const change of ['mode', 'profile'] as const) {
  test(`same-task cache ${change} ABA cancels late reserve without run`, async ({ page }) => {
    const events: Event[] = []; const reserve = gate()
    await mockApi(page, { activeBroker: 'toss', researchEvents: events, researchReserveGate: reserve.promise })
    await page.goto('/strategy'); await calculate(page).click()
    await expect(card(page).getByRole('status')).toHaveText('예약 중...')
    await card(page).evaluate((button, change) => {
      type Client = { getQueryData: (key: string[]) => Record<string, unknown>; setQueryData: (key: string[], value: Record<string, unknown>) => void }
      type Fiber = { return?: Fiber; memoizedProps?: { client?: Client } }
      const entry = Object.entries(button).find(([key]) => key.startsWith('__reactFiber$'))
      let fiber = entry?.[1] as Fiber | undefined
      while (fiber && !fiber.memoizedProps?.client?.setQueryData) fiber = fiber.return
      const client = fiber?.memoizedProps?.client
      if (!client) throw new Error('Mounted QueryClient provider not found')
      const original = client.getQueryData(['appConfig'])
      client.setQueryData(['appConfig'], { ...original, ...(change === 'mode'
        ? { kis_is_paper_trading: !original.kis_is_paper_trading }
        : { active_profile_id: 'profile-b', active_broker_account_id: '87654321-01' }) })
      client.setQueryData(['appConfig'], original)
    }, change)
    reserve.release()
    await expect.poll(() => events.map((event) => event.action)).toEqual(['reserve', 'cancel'])
    await expect(calculate(page)).toBeEnabled()
    expect(events.filter((event) => event.action === 'run')).toHaveLength(0)
    expect(events.filter((event) => event.action === 'cancel')).toHaveLength(1)
  })
}

test('captured DOM button same-task cache ABA click cannot dispatch reserve', async ({ page }) => {
  const events: Event[] = []
  await mockApi(page, { activeBroker: 'toss', researchEvents: events })
  await page.goto('/strategy'); await expect(calculate(page)).toBeEnabled()
  const dispatched = await calculate(page).evaluate((button: HTMLButtonElement) => {
    type Client = { getQueryData: (key: string[]) => Record<string, unknown>; setQueryData: (key: string[], value: Record<string, unknown>) => void }
    type Fiber = { return?: Fiber; memoizedProps?: { client?: Client } }
    const entry = Object.entries(button).find(([key]) => key.startsWith('__reactFiber$'))
    let fiber = entry?.[1] as Fiber | undefined
    while (fiber && !fiber.memoizedProps?.client?.setQueryData) fiber = fiber.return
    const client = fiber?.memoizedProps?.client
    if (!client) throw new Error('Mounted QueryClient provider not found')
    const original = client.getQueryData(['appConfig'])
    const click = button.click.bind(button)
    const originalFetch = window.fetch
    const calls: string[] = []
    window.fetch = (input, init) => {
      const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url
      if (new URL(url, location.href).pathname.endsWith('/research/reserve')) calls.push(url)
      return originalFetch.call(window, input, init)
    }
    try {
      client.setQueryData(['appConfig'], { ...original, kis_is_paper_trading: !original.kis_is_paper_trading })
      client.setQueryData(['appConfig'], original)
      // Native DOM click enters React on the retained node. Observe synchronous
      // browser REST fetch creation, not a potentially later route-arrival count.
      // This does not directly invoke an old React closure or prove native IPC.
      click()
      return calls
    } finally { window.fetch = originalFetch }
  })
  expect(dispatched).toEqual([])
  await expect(calculate(page)).toBeEnabled()
  expect(events.filter((event) => event.action === 'reserve')).toHaveLength(0)
})

test('cancel transport loss retains ownership until a new confirmed cancellation', async ({ page }) => {
  const events: Event[] = []; const run = gate(); let attempts = 0
  await mockApi(page, { activeBroker: 'toss', researchEvents: events, researchRunGate: run.promise })
  await page.route('**/api/strategy/leveraged-trend-hold/research/cancel', (route) => {
    attempts += 1
    return attempts === 1 ? route.abort() : route.fallback()
  })
  await page.goto('/strategy'); await calculate(page).click()
  await expect(card(page).getByRole('status')).toHaveText('수집·계산 중...')
  await expect.poll(() => events.filter((event) => event.action === 'run').length).toBe(1)
  await card(page).getByRole('button', { name: '연구 취소', exact: true }).click()
  await expect(card(page).getByText(/취소 확인 실패: 기존 작업/)).toBeVisible()
  await expect(calculate(page)).toBeDisabled()
  expect(events.filter((event) => event.action === 'reserve')).toHaveLength(1)
  await card(page).getByRole('button', { name: '취소 다시 확인', exact: true }).click()
  await expect(calculate(page)).toBeEnabled()
  expect(attempts).toBe(2)
  expect(events.filter((event) => event.action === 'cancel')).toHaveLength(1)
  const completed = page.waitForResponse((response) => new URL(response.url()).pathname.endsWith('/research/run'))
  run.release()
  const response = await completed
  await response.finished()
  expect((await response.json()).code).toBe('LTH_RUNTIME_REJECTED')
  await expect(card(page).getByTestId('replay-assessment-notice')).toHaveCount(0)
})


const summaryRegion = (page: Page) => card(page).getByRole('region', { name: '수집 상세 기록', exact: true })

test('successful synthetic summary separates counts ranges fingerprints and entry budget at narrow width', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()) })
  await page.setViewportSize({ width: 390, height: 844 })
  await mockApi(page, { activeBroker: 'toss', researchCollectionSummary: syntheticCollectionSummary() })
  await page.goto('/strategy'); await calculate(page).click()
  const region = summaryRegion(page)
  await expect(region).toBeVisible()
  for (const text of ['요청 200봉', '요청 시작 지정 안 됨', '예약 고정 asOf 2026-07-07T21:00:00Z',
    '실제 평가 UTC 2026-07-07T13:31:00Z', '일봉 context 원래 join label 20260706',
    '일봉 관측 UTC 2026-07-02T04:00:00Z', '분봉 관측 UTC 2026-07-07T13:30:00Z',
    '페이지 dispatch 15 / returned 15 / validated 15', 'typed rows returned 202 / validated 202 / unique 201 / dedup 1',
    'selectionLimit: 1', 'collector dedup은 Prepared DuplicateInstant', '정규화 수집입력 SHA',
    '수집 진입 시 남은 예산 27123ms', 'maxPages 16', 'maxTypedRows 3200', 'pageSize 200', 'pageTimeoutSeconds 15',
    'source complete', 'coverageVerified=false', 'publication unknown', 'dailyOHLC/adjustment unverified']) {
    await expect(region).toContainText(text)
  }
  await expect(region).toContainText('ef'.repeat(32))
  await expect(region).toContainText('ab'.repeat(32))
  await expect(region).toContainText('cd'.repeat(32))
  await expect(region.getByText(/raw SHA/)).toHaveCount(16)
  await expect(region).toContainText('수신시각 collectedAt 2026-07-07T21:01:00Z')
  expect(await region.evaluate((node) => node.scrollWidth <= node.clientWidth)).toBe(true)
  expect(errors).toEqual([])
})

for (const damage of ['missing', 'object', 'negative', 'future', 'reversed', 'certified', 'invalid-label', 'source'] as const) {
  test(`successful ${damage} metadata uses safe unavailable display`, async ({ page }) => {
    const summary = syntheticCollectionSummary()
    let value: unknown = summary
    if (damage === 'missing') value = undefined
    if (damage === 'object') value = { daily: null }
    if (damage === 'negative') summary.minute.uniqueRows = -1
    if (damage === 'future') summary.admission!.evaluationEnd = '2026-07-08T20:00:00Z'
    if (damage === 'reversed') summary.minute.observedStart = '2026-07-07T20:01:00Z'
    if (damage === 'certified') value = { ...summary, coverageVerified: true }
    if (damage === 'invalid-label') summary.admission!.dailyContextStart = '99999999'
    if (damage === 'source') summary.minute.sources[0].sha256 = 'broken'
    const errors: string[] = []; page.on('pageerror', (error) => errors.push(error.message))
    await mockApi(page, { activeBroker: 'toss', researchCollectionSummary: value })
    await page.goto('/strategy'); await calculate(page).click()
    await expect(card(page).getByText('수집 상세 기록 확인 불가', { exact: true })).toBeVisible()
    await expect(card(page).getByTestId('replay-assessment-notice')).toBeVisible()
    await expect(summaryRegion(page)).toHaveCount(0)
    expect(errors).toEqual([])
  })
}

test('success followed by deterministic terminal failure clears previous metadata', async ({ page }) => {
  const options = { activeBroker: 'toss' as const, researchCollectionSummary: syntheticCollectionSummary(), researchRunFails: false }
  await mockApi(page, options); await page.goto('/strategy'); await calculate(page).click()
  await expect(summaryRegion(page)).toBeVisible()
  options.researchRunFails = true
  await calculate(page).click()
  await expect(card(page).getByText(/수집 상세 수치·원인 내역은 제공되지 않음/)).toBeVisible()
  await expect(calculate(page)).toBeEnabled()
  await expect(summaryRegion(page)).toHaveCount(0)
  await expect(card(page).getByText('ab'.repeat(32), { exact: false })).toHaveCount(0)
})

for (const change of ['cancel', 'input-ABA', 'mode-ABA'] as const) {
  test(`late success after ${change} cannot replace newer summary`, async ({ page }) => {
    const oldRun = gate(); const entered = gate(); const events: Event[] = []; const jobs: string[] = []
    const options = { activeBroker: 'toss' as const, researchEvents: events,
      researchCollectionSummary: syntheticCollectionSummary(), researchLateSuccessAfterCancel: true,
      researchRunGateForJob: (id: string) => { jobs.push(id); if (jobs.length === 1) { entered.release(); return oldRun.promise } return Promise.resolve() } }
    await mockApi(page, options); await page.goto('/strategy'); await calculate(page).click(); await entered.promise
    await expect(card(page).getByRole('status')).toHaveText('수집·계산 중...')
    if (change === 'cancel') await card(page).getByRole('button', { name: '연구 취소', exact: true }).click()
    else if (change === 'input-ABA') {
      for (const n of [50, 200]) {
        await card(page).getByLabel('분석 구간').click()
        await page.getByRole('option', { name: `최근 ${n}봉`, exact: true }).click()
      }
    } else {
      await card(page).evaluate((node) => {
        type Client = { getQueryData: (key: string[]) => Record<string, unknown>; setQueryData: (key: string[], value: Record<string, unknown>) => void }
        type Fiber = { return?: Fiber; memoizedProps?: { client?: Client } }
        let fiber = Object.entries(node).find(([key]) => key.startsWith('__reactFiber$'))?.[1] as Fiber | undefined
        while (fiber && !fiber.memoizedProps?.client?.setQueryData) fiber = fiber.return
        const client = fiber?.memoizedProps?.client
        if (!client) throw new Error('Mounted QueryClient provider not found')
        const old = client.getQueryData(['appConfig'])
        client.setQueryData(['appConfig'], { ...old, kis_is_paper_trading: !old.kis_is_paper_trading })
        client.setQueryData(['appConfig'], old)
      })
    }
    await expect(calculate(page)).toBeEnabled()
    const next = syntheticCollectionSummary(); next.daily.sources[0].sha256 = '12'.repeat(32); next.minute.sources.forEach((source) => { source.sha256 = '34'.repeat(32) })
    options.researchCollectionSummary = next
    await calculate(page).click()
    await expect(summaryRegion(page)).toContainText('12'.repeat(32))
    expect(jobs).toHaveLength(2)
    // Register exact old response waiter before releasing the held command.
    const completed = page.waitForResponse((response) => new URL(response.url()).pathname.endsWith('/research/run') && response.request().postDataJSON().jobId === jobs[0])
    oldRun.release()
    const response = await completed; expect(await response.finished()).toBeNull()
    expect((await response.json()).collectionSummary.daily.sources[0].sha256).toBe('ab'.repeat(32))
    // Positive current product state after body completion; absence alone is not consumption proof.
    await expect(summaryRegion(page)).toContainText('12'.repeat(32))
    await expect(calculate(page)).toBeEnabled()
    await expect(summaryRegion(page)).not.toContainText('ab'.repeat(32))
    expect(events.filter((e) => e.action === 'cancel')).toHaveLength(1)
  })
}


for (const availability of ['partial', 'unavailable'] as const) {
  test(`source ${availability} records metadata presence without fabricated hashes`, async ({ page }) => {
    const summary = syntheticCollectionSummary()
    summary.minute.sources = []
    if (availability === 'unavailable') summary.daily.sources = []
    summary.sourceAvailability = availability
    await mockApi(page, { activeBroker: 'toss', researchCollectionSummary: summary })
    await page.goto('/strategy'); await calculate(page).click()
    await expect(summaryRegion(page)).toContainText(`source ${availability}`)
    await expect(summaryRegion(page).getByText(/raw SHA/)).toHaveCount(availability === 'partial' ? 1 : 0)
    await expect(summaryRegion(page)).toContainText('coverageVerified=false')
    await expect(summaryRegion(page)).not.toContainText('0'.repeat(64))
  })
}
