import { expect, test, type Page, type Route } from '@playwright/test'

const harnessViolations = new WeakMap<Page, string[]>()
test.afterEach(async ({ page }) => { expect(harnessViolations.get(page) ?? []).toEqual([]) })

function eventPath(route: Route, expected?: typeof accountConfig): string {
  const request = route.request(), url = new URL(request.url())
  const origin = new URL(String(test.info().project.use.baseURL)).origin
  const path = url.pathname.split('/').pop() ?? ''
  const methods: Record<string, string> = { config: 'GET', balance: 'GET', overseas: 'GET', broker: 'GET',
    budget: 'GET', recovery: 'GET', 'set-active': 'POST', delete: 'POST', 'save-budget': 'POST' }
  if (url.origin !== origin || methods[path] !== request.method()) throw new Error('Unexpected fixture endpoint/method: ' + request.method() + ' ' + url.href)
  if (['budget', 'recovery', 'save-budget'].includes(path)) {
    const input = path === 'save-budget' ? request.postDataJSON().input : JSON.parse(url.searchParams.get('input') ?? 'null')
    if (!input || !['kis', 'toss'].includes(input.brokerId) || !input.brokerAccountId ||
      (expected && (input.brokerId !== expected.active_broker_id || input.brokerAccountId !== expected.active_broker_account_id))) {
      throw new Error('Unexpected fixture budget account')
    }
    if (path === 'save-budget' && (typeof input.expectedIsPaper !== 'boolean' || !input.expectedProfileId ||
      (expected && (input.expectedIsPaper !== expected.kis_is_paper_trading || input.expectedProfileId !== expected.active_profile_id)))) {
      throw new Error('Unexpected fixture budget profile/mode')
    }
  }
  return path
}

type EventHarness = {
  emit: (event: string, payload: unknown) => Promise<void>
  switchAccount: (account: string) => Promise<void>
  seen: string[]
  callbackCount: () => number
  queueThenUnmount: () => void
  deliverQueued: (payload: unknown) => void
  exchangeRate: () => unknown
  balance: () => unknown
  refreshHoldings: () => void
  enableHoldings: () => void
  scopeQueries: () => { key: unknown[], data: unknown, fetchStatus: string }[]
  setOnline: (online: boolean) => void
  refreshConfig: () => Promise<void>
  deleteActive: () => Promise<void>
  config: () => { active_profile_id: string | null }
  pendingProfileMutations: () => { paused: boolean }[]
  saveBudget: () => Promise<unknown>
}

/** Mount the production event hook with real React Query observers and Tauri's
 * event mock. Only the application entry point is replaced; all HTTP is local
 * Playwright fixtures and no trading commands or provider endpoints are used. */
async function installHarness(page: Page, scoped = false, initiallyEnabled = true, includeBudget = false) {
  const origin = new URL(String(test.info().project.use.baseURL)).origin
  const violations: string[] = []
  harnessViolations.set(page, violations)
  // Exact test origin only; an unknown API must not reach Vite or another port.
  await page.route('**/*', route => {
    const request = route.request(), url = new URL(request.url())
    if (url.origin === origin && request.method() === 'GET' && (['/', '/@vite/client', '/@react-refresh', '/favicon.ico'].includes(url.pathname) || ['/src/', '/node_modules/', '/@fs/'].some(prefix => url.pathname.startsWith(prefix)))) return route.fallback()
    violations.push(request.method() + ' ' + url.href)
    return route.abort()
  })
  await page.routeWebSocket('**/*', socket => {
    const url = new URL(socket.url())
    const httpOrigin = url.origin.replace(/^ws/, 'http')
    if (httpOrigin !== origin || url.pathname !== '/') violations.push('WS ' + socket.url())
    socket.close()
  })
  await page.route('**/src/main.tsx', async (route) => {
    const request = route.request(), url = new URL(request.url())
    if (url.origin !== origin || request.method() !== 'GET' || url.pathname !== '/src/main.tsx') {
      violations.push('entry ' + request.method() + ' ' + url.href); return route.abort()
    }
    const response = await route.fetch()
    const entry = await response.text()
    const dependency = (name: string) => {
      const path = [...entry.matchAll(/(?:from|import)\s*["']([^"']+)["']/g)]
        .map((match) => match[1])
        .find((path) => path.includes(`/deps/${name}.js`))
      if (!path) throw new Error(`Vite entry dependency missing: ${name}`)
      return JSON.stringify(path)
    }
    await route.fulfill({ response, contentType: 'text/javascript', body: `
      import React from ${dependency('react')};
      import ReactDOM from ${dependency('react-dom_client')};
      import { QueryClient, QueryClientProvider, useQuery, onlineManager } from ${dependency('@tanstack_react-query')};
      import { useBackendEvents } from '/src/api/backendEvents.ts';
      import { KEYS } from '/src/api/queryKeys.ts';
      import { useBalance, useOverseasBalance, useBrokerHoldings, useSetActiveProfile, useDeleteProfile, useAppConfig } from '/src/api/accountHooks.ts';
      import { useAutoTradingBudget, useUpdateAutoTradingBudget } from '/src/api/budgetHooks.ts';
      import { useBudgetRecovery } from '/src/api/budgetRecoveryHooks.ts';
      import { mockIPC } from '/node_modules/@tauri-apps/api/mocks.js';
      import { emit } from '/node_modules/@tauri-apps/api/event.js';
      const scoped = ${scoped};
      const json = async response => {
        const data = await response.json();
        if (!response.ok) throw new Error(data.error || 'Synthetic API failure');
        return data;
      };
      mockIPC((command, args) => {
        if (!scoped) throw new Error('Unexpected IPC: ' + command);
        const paths = { get_app_config: 'config', get_balance: 'balance', get_overseas_balance: 'overseas', get_broker_holdings: 'broker', get_auto_trading_budget: 'budget', get_budget_recovery: 'recovery' };
        if (paths[command]) return fetch('/api/event-scope/' + paths[command] + (args?.input ? '?input=' + encodeURIComponent(JSON.stringify(args.input)) : '')).then(json);
        if (command === 'set_active_profile') return fetch('/api/event-scope/set-active', { method: 'POST', body: JSON.stringify(args) }).then(json);
        if (command === 'delete_profile') return fetch('/api/event-scope/delete', { method: 'POST', body: JSON.stringify(args) }).then(json);
        if (command === 'update_auto_trading_budget') return fetch('/api/event-scope/save-budget', { method: 'POST', body: JSON.stringify(args) }).then(json);
        throw new Error('Unexpected IPC: ' + command);
      }, { shouldMockEvents: true });
      const client = new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } });
      let account = 'A';
      const seen = [];
      client.getQueryCache().subscribe(() => {
        seen.push(JSON.stringify(client.getQueryData(KEYS.balance)));
        seen.push(JSON.stringify(client.getQueryData(KEYS.overseasBalance)));
      });
      let switchProfile;
      let deleteActive;
      let enableHoldings;
      let saveBudget;
      function Harness() {
        useBackendEvents();
        const [enabled, setEnabled] = React.useState(${initiallyEnabled});
        enableHoldings = () => setEnabled(true);
        const options = { enabled: () => enabled, placeholderData: previous => previous };
        const domestic = scoped ? useBalance(options) : useQuery({ queryKey: KEYS.balance, queryFn: () => fetch('/api/balance?account=' + account).then(r => r.json()) });
        const overseas = scoped ? useOverseasBalance(options) : useQuery({ queryKey: KEYS.overseasBalance, queryFn: () => fetch('/api/overseas-balance?account=' + account).then(r => r.json()) });
        const broker = scoped ? useBrokerHoldings(options) : { data: null };
        const mutation = useSetActiveProfile();
        const deletion = useDeleteProfile();
        const config = scoped ? useAppConfig() : { error: null };
        const budgetInput = config.data?.active_broker_account_id ? { brokerId: config.data.active_broker_id, brokerAccountId: config.data.active_broker_account_id } : undefined;
        const budget = ${includeBudget} ? useAutoTradingBudget(budgetInput) : { data: null };
        const recovery = ${includeBudget} ? useBudgetRecovery(budgetInput, { profileId: config.data?.active_profile_id ?? null, isPaper: config.data?.kis_is_paper_trading ?? false }, true) : { data: null };
        const budgetUpdate = useUpdateAutoTradingBudget();
        saveBudget = () => budgetUpdate.mutateAsync({ ...budgetInput, expectedProfileId: config.data.active_profile_id, expectedIsPaper: config.data.kis_is_paper_trading, krwAmount: 1234, usdAmount: 0 });
        switchProfile = next => mutation.mutateAsync(next);
        deleteActive = () => deletion.mutateAsync('profile-a');
        return React.createElement('main', null,
          React.createElement('pre', { 'data-testid': 'domestic' }, JSON.stringify(domestic.data)),
          React.createElement('pre', { 'data-testid': 'overseas' }, JSON.stringify(overseas.data)),
          React.createElement('pre', { 'data-testid': 'broker' }, JSON.stringify(broker.data)),
          React.createElement('pre', { 'data-testid': 'budget' }, JSON.stringify(budget.data)),
          React.createElement('pre', { 'data-testid': 'save-status' }, budgetUpdate.status + ':' + (budgetUpdate.error?.code ?? '')),
          React.createElement('pre', { 'data-testid': 'recovery' }, JSON.stringify(recovery.data)),
          React.createElement('pre', { 'data-testid': 'holdings-error' }, domestic.error?.message || ''),
          config.error ? React.createElement('div', { role: 'alert' }, config.error.message) : null);
      }
      const root = ReactDOM.createRoot(document.getElementById('root'));
      let queued = [];
      window.__balanceEvents = {
        emit,
        seen,
        switchAccount(next) {
          if (scoped) return switchProfile(next);
          account = next;
          return Promise.all([
            client.invalidateQueries({ queryKey: KEYS.balance }),
            client.invalidateQueries({ queryKey: KEYS.overseasBalance }),
          ]);
        },
        refreshConfig: () => client.invalidateQueries({ queryKey: KEYS.appConfig }, { throwOnError: true }),
        deleteActive: () => deleteActive(),
        saveBudget: () => saveBudget(),
        config: () => client.getQueryData(KEYS.appConfig),
        pendingProfileMutations: () => client.getMutationCache().getAll().filter(mutation => mutation.state.status === 'pending').map(mutation => ({ paused: mutation.state.isPaused })),
        refreshHoldings: () => { [KEYS.balance, KEYS.overseasBalance, KEYS.brokerHoldings, ['autoTradingBudget'], ['budgetRecovery']].forEach(queryKey => { void client.invalidateQueries({ queryKey }); }); },
        enableHoldings: () => enableHoldings(),
        setOnline: online => onlineManager.setOnline(online),
        scopeQueries: () => client.getQueryCache().getAll().filter(query => ['balance', 'overseasBalance', 'brokerHoldings', 'autoTradingBudget', 'budgetRecovery'].includes(query.queryKey[0])).map(query => ({ key: query.queryKey, data: query.state.data, fetchStatus: query.state.fetchStatus })),
        callbackCount: () => window.__TAURI_INTERNALS__.callbacks.size,
        queueThenUnmount() {
          queued = [...window.__TAURI_INTERNALS__.callbacks.values()];
          root.unmount();
        },
        deliverQueued: (payload) => queued.forEach(callback => callback({ payload })),
        exchangeRate: () => client.getQueryData(KEYS.exchangeRate),
        balance: () => client.getQueryData(KEYS.balance),
      };
      root.render(React.createElement(React.StrictMode, null,
        React.createElement(QueryClientProvider, { client }, React.createElement(Harness))));
    ` })
  })
}

test('scope budget save cannot succeed while its post-save money read is paused offline', async ({ page }) => {
  await installHarness(page, true, true, true)
  let releaseSave!: () => void
  const saveGate = new Promise<void>(resolve => { releaseSave = resolve })
  let posted = false, saved = false, budgetReads = 0, writes = 0
  await page.route('**/api/event-scope/*', async route => {
    const path = eventPath(route, accountConfig)
    if (path === 'config') return route.fulfill({ json: accountConfig })
    if (path === 'save-budget') {
      writes++
      posted = true
      await saveGate
      saved = true
      return route.fulfill({ json: { marker: 'POST_ACK_ONLY' } })
    }
    if (path === 'budget') budgetReads++
    return route.fulfill({ json: { marker: saved ? 'CONFIRMED_AFTER_SAVE' : 'BEFORE_SAVE' } })
  })
  await page.goto('/')
  await expect(page.getByTestId('budget')).toContainText('BEFORE_SAVE')
  const before = budgetReads
  await page.evaluate(() => {
    const harness = (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents
    void harness.saveBudget().catch(() => {})
  })
  await expect.poll(() => posted).toBe(true)
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.setOnline(false))
  try {
    releaseSave()
    await expect.poll(() => saved).toBe(true)
    await expect.poll(() => page.evaluate(() =>
      (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries()
        .some(query => query.key[0] === 'autoTradingBudget' && query.fetchStatus === 'paused'))).toBe(true)
    await expect(page.getByTestId('save-status')).toHaveText('pending:')
    await expect(page.getByTestId('budget')).not.toContainText('CONFIRMED_AFTER_SAVE')
    expect(budgetReads).toBe(before)
    expect(writes).toBe(1)
    await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.setOnline(true))
    await expect(page.getByTestId('budget')).toContainText('CONFIRMED_AFTER_SAVE')
    await expect(page.getByTestId('save-status')).toHaveText('success:')
    expect(budgetReads).toBeGreaterThan(before)
    expect(writes).toBe(1)
  } finally {
    releaseSave()
    await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.setOnline(true))
  }
})

for (const market of ['domestic', 'overseas'] as const) {
  test(`queued old-account ${market} event refetches current account without putting its payload in cache`, async ({ page }) => {
    await installHarness(page)
    let revision = 1
    let block = false
    let release: (() => void) | undefined
    const requests: string[] = []
    await page.route('**/api/*balance?*', async (route) => {
      const url = new URL(route.request().url())
      const currentMarket = url.pathname.includes('overseas') ? 'overseas' : 'domestic'
      requests.push(`${currentMarket}:${url.searchParams.get('account')}`)
      if (block && currentMarket === market) await new Promise<void>((resolve) => { release = resolve })
      await route.fulfill({ json: { account: url.searchParams.get('account'), market: currentMarket, marker: `current-${revision}` } })
    })
    await page.goto('/')
    await expect(page.getByTestId(market)).toContainText('"account":"A"')
    await expect.poll(() => page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.callbackCount())).toBe(4)
    await page.evaluate(() => { void (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('B') })
    await expect(page.getByTestId(market)).toContainText('"account":"B"')
    revision = 2
    block = true
    const before = requests.filter((request) => request === `${market}:B`).length
    await page.evaluate(async (kind) => {
      const harness = (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents
      await harness.emit(kind === 'domestic' ? 'balance-updated' : 'overseas-balance-updated', {
        account: 'A', marker: 'QUEUED_OLD_ACCOUNT_PAYLOAD', items: [],
      })
    }, market)
    await expect.poll(() => requests.filter((request) => request === `${market}:B`).length).toBe(before + 1)
    await expect(page.getByTestId(market)).toContainText('"account":"B"')
    await expect(page.getByTestId(market)).not.toContainText('QUEUED_OLD_ACCOUNT_PAYLOAD')
    block = false
    release?.()
    await expect(page.getByTestId(market)).toContainText('current-2')
    expect(await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.seen.some((data) => data?.includes('QUEUED_OLD_ACCOUNT_PAYLOAD')))).toBe(false)
  })
}

test('exchange events keep their global cache behavior and queued callbacks cannot mutate after cleanup', async ({ page }) => {
  await installHarness(page)
  await page.route('**/api/*balance?*', (route) => route.fulfill({ json: { account: 'B', marker: 'current' } }))
  await page.goto('/')
  await expect(page.getByTestId('domestic')).toContainText('current')
  await expect.poll(() => page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.callbackCount())).toBe(4)
  await page.evaluate(async () => {
    const harness = (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents
    await harness.emit('exchange-rate-updated', 1400)
  })
  expect(await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.exchangeRate())).toBe(1400)
  await page.evaluate(async () => {
    const harness = (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents
    await harness.emit('exchange-rate-status-updated', { rate: 1450, source: 'synthetic fixture' })
    harness.queueThenUnmount()
    harness.deliverQueued({ rate: 9999, marker: 'QUEUED_AFTER_UNMOUNT' })
  })
  await expect.poll(() => page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.callbackCount())).toBe(0)
  expect(await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.exchangeRate())).toBe(1450)
  expect(await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.balance())).toEqual({ account: 'B', marker: 'current' })
})

const accountConfig = {
  active_profile_id: 'profile-a', active_profile_name: 'A', active_broker_id: 'kis',
  active_broker_account_id: '11111111-01', kis_is_paper_trading: true,
  kis_configured: true, active_broker_configured: true,
}

for (const identity of ['profile', 'account', 'broker', 'mode'] as const) {
  test(`production holdings hooks isolate ${identity} changes and discard late previous-scope responses`, async ({ page }) => {
    await installHarness(page, true)
    const nextConfig = { ...accountConfig,
      ...(identity === 'profile' ? { active_profile_id: 'profile-b' } : {}),
      ...(identity === 'account' ? { active_broker_account_id: '22222222-01' } : {}),
      ...(identity === 'broker' ? { active_broker_id: 'toss' } : {}),
      ...(identity === 'mode' ? { kis_is_paper_trading: false } : {}),
    }
    let active = 'A'
    let blockOld = false
    let blockNew = true
    const requests: string[] = []
    const oldReleases: (() => void)[] = []
    const newReleases: (() => void)[] = []
    await page.route('**/api/event-scope/*', async (route) => {
      const path = eventPath(route)
      if (path === 'config') return route.fulfill({ json: active === 'A' ? accountConfig : nextConfig })
      if (path === 'set-active') {
        active = 'B'
        return route.fulfill({ json: nextConfig })
      }
      const requestedAccount = active
      requests.push(`${path}:${requestedAccount}`)
      if (blockOld && requestedAccount === 'A') await new Promise<void>((resolve) => { oldReleases.push(resolve) })
      if (blockNew && requestedAccount === 'B') await new Promise<void>((resolve) => { newReleases.push(resolve) })
      await route.fulfill({ json: { account: requestedAccount, marker: `SCOPE_${requestedAccount}_${path}` } })
    })
    await page.goto('/')
    for (const market of ['domestic', 'overseas', 'broker']) {
      await expect(page.getByTestId(market)).toContainText('"account":"A"')
    }
    blockOld = true
    await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshHoldings())
    await expect.poll(() => oldReleases.length).toBe(3)
    await page.evaluate(() => { void (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('B') })
    await expect.poll(() => requests.filter(request => request.endsWith(':B')).length).toBeGreaterThanOrEqual(3)
    // Even an explicit keep-previous-data option cannot bridge the account scope.
    for (const market of ['domestic', 'overseas', 'broker']) {
      await expect(page.getByTestId(market)).not.toContainText('"account":"A"')
    }
    blockNew = false
    newReleases.splice(0).forEach(release => release())
    for (const market of ['domestic', 'overseas', 'broker']) {
      await expect(page.getByTestId(market)).toContainText('"account":"B"')
    }
    blockOld = false
    oldReleases.splice(0).forEach(release => release())
    await page.evaluate(() => new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
    for (const market of ['domestic', 'overseas', 'broker']) {
      await expect(page.getByTestId(market)).toContainText('"account":"B"')
    }
    const queries = await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries())
    for (const prefix of ['balance', 'overseasBalance', 'brokerHoldings']) {
      const query = queries.find(query => query.key[0] === prefix && query.key[query.key.length - 4] === nextConfig.active_profile_id &&
        query.key[query.key.length - 3] === nextConfig.active_broker_id && query.key[query.key.length - 2] === nextConfig.active_broker_account_id && query.key[query.key.length - 1] === nextConfig.kis_is_paper_trading)
      expect(query?.data).toMatchObject({ account: 'B' })
    }
  })
}

test('scope budgets and recovery discard mode ABA reads and late saves while untrusted config hides money', async ({ page }) => {
  await page.clock.install()
  await installHarness(page, true, true, true)
  let active: 'paper' | 'live' = 'paper'
  let phase = 'initial-paper'
  let configFails = false
  let saveFinished = false
  let saveMutations = 0
  const releases: (() => void)[] = []
  let releaseSave!: () => void
  const saveGate = new Promise<void>(resolve => { releaseSave = resolve })
  const recoveryGates: (() => void)[] = []
  let completedOld = 0
  let recoveryReads = 0
  let budgetReads = 0
  const configs = { paper: accountConfig, live: { ...accountConfig, kis_is_paper_trading: false } }
  await page.route('**/api/event-scope/*', async route => {
    const path = eventPath(route, configs[active])
    if (path === 'recovery') recoveryReads++
    if (path === 'budget') budgetReads++
    if (path === 'config') return route.fulfill(configFails
      ? { status: 503, json: { error: 'Synthetic scope unknown' } } : { json: configs[active] })
    if (path === 'set-active') {
      active = (route.request().postDataJSON() as { id: 'paper' | 'live' }).id
      phase = active === 'live' ? 'current-live' : 'fresh-paper'
      return route.fulfill({ json: configs[active] })
    }
    if (path === 'save-budget') {
      saveMutations++
      await saveGate
      saveFinished = true
      return route.fulfill({ json: { marker: 'STALE_SAVE_PAPER' } })
    }
    const marker = phase
    if (marker === 'recovered-paper' && ['budget', 'recovery'].includes(path)) {
      await new Promise<void>(resolve => recoveryGates.push(resolve))
    }
    if (marker === 'old-paper' && ['budget', 'recovery'].includes(path ?? '')) {
      await new Promise<void>(resolve => { releases.push(resolve) })
    }
    await route.fulfill({ json: { marker, market: path } })
    if (marker === 'old-paper' && ['budget', 'recovery'].includes(path ?? '')) completedOld++
  })
  await page.goto('/')
  for (const panel of ['budget', 'recovery']) await expect(page.getByTestId(panel)).toContainText('initial-paper')
  await page.evaluate(() => { void (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.saveBudget().catch(() => {}) })
  phase = 'old-paper'
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshHoldings())
  await expect.poll(() => releases.length).toBe(2)
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('live'))
  for (const panel of ['budget', 'recovery']) await expect(page.getByTestId(panel)).toContainText('current-live')
  releaseSave()
  await expect.poll(() => saveFinished).toBe(true)
  for (const panel of ['budget', 'recovery']) {
    await expect(page.getByTestId(panel)).toContainText('current-live')
    await expect(page.getByTestId(panel)).not.toContainText('STALE_SAVE_PAPER')
  }
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('paper'))
  for (const panel of ['budget', 'recovery']) await expect(page.getByTestId(panel)).toContainText('fresh-paper')
  releases.forEach(release => release())
  await expect.poll(() => completedOld).toBe(2)
  await page.evaluate(() => new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  for (const panel of ['budget', 'recovery']) await expect(page.getByTestId(panel)).toContainText('fresh-paper')
  configFails = true
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshConfig().catch(() => {}))
  await expect(page.getByRole('alert')).toContainText('Synthetic scope unknown')
  for (const panel of ['budget', 'recovery']) await expect(page.getByTestId(panel)).toBeEmpty()
  configFails = false
  phase = 'recovered-paper'
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshConfig())
  await expect.poll(() => recoveryGates.length).toBe(2)
  for (const panel of ['budget', 'recovery']) await expect(page.getByTestId(panel)).toBeEmpty()
  recoveryGates.forEach(release => release())
  for (const panel of ['budget', 'recovery']) await expect(page.getByTestId(panel)).toContainText('recovered-paper')
  // Recovery stays an explicit review, even though the shared account query polls balances.
  const readsBefore = recoveryReads
  const budgetReadsBefore = budgetReads
  await page.clock.runFor(61_000)
  await expect.poll(() => budgetReads).toBeGreaterThan(budgetReadsBefore)
  expect(recoveryReads).toBe(readsBefore)
  expect(saveMutations).toBe(1)
})

test('production holdings hooks wait for config and preserve disabled function options with legacy null profile', async ({ page }) => {
  await installHarness(page, true, false)
  let releaseConfig: (() => void) | undefined
  const holdings: string[] = []
  await page.route('**/api/event-scope/*', async (route) => {
    const path = eventPath(route)
    if (path === 'config') {
      await new Promise<void>(resolve => { releaseConfig = resolve })
      return route.fulfill({ json: { ...accountConfig, active_profile_id: null } })
    }
    holdings.push(path ?? '')
    return route.fulfill({ json: { marker: 'LEGACY_CURRENT_ACCOUNT' } })
  })
  await page.goto('/')
  await expect.poll(() => !!releaseConfig).toBe(true)
  expect(holdings).toHaveLength(0)
  releaseConfig?.()
  await expect.poll(() => page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries().some(query => query.key[query.key.length - 4] === null && query.key[query.key.length - 3] === 'kis'))).toBe(true)
  expect(holdings).toHaveLength(0)
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.enableHoldings())
  for (const market of ['domestic', 'overseas', 'broker']) {
    await expect(page.getByTestId(market)).toContainText('LEGACY_CURRENT_ACCOUNT')
  }
  expect(holdings.sort()).toEqual(['balance', 'broker', 'overseas'])
})

test('set-active-profile mutation fences all three holdings caches through A to B to A and late A completion', async ({ page }) => {
  await installHarness(page, true)
  const configs = {
    A: accountConfig,
    B: { ...accountConfig, active_profile_id: 'profile-b', active_profile_name: 'B', active_broker_account_id: '22222222-01' },
  }
  let active: 'A' | 'B' = 'A'
  let phase = 'initial-A'
  let holdReturn = true
  let oldResponsesCompleted = 0
  const mutations: string[] = []
  const requests: { market: string, phase: string }[] = []
  const oldReleases: (() => void)[] = []
  const returnReleases: (() => void)[] = []
  await page.route('**/api/event-scope/*', async (route) => {
    const path = eventPath(route)
    if (path === 'config') return route.fulfill({ json: configs[active] })
    if (path === 'set-active') {
      const { id } = route.request().postDataJSON() as { id: string }
      mutations.push(id)
      active = id === 'profile-a' ? 'A' : 'B'
      phase = active === 'A' ? 'return-A' : 'current-B'
      return route.fulfill({ json: configs[active] })
    }
    const requestedPhase = phase
    requests.push({ market: path ?? '', phase: requestedPhase })
    if (requestedPhase === 'old-A') await new Promise<void>((resolve) => { oldReleases.push(resolve) })
    if (requestedPhase === 'return-A' && holdReturn) await new Promise<void>((resolve) => { returnReleases.push(resolve) })
    await route.fulfill({ json: { marker: requestedPhase, market: path } })
    if (requestedPhase === 'old-A') oldResponsesCompleted += 1
  })
  await page.goto('/')
  for (const market of ['domestic', 'overseas', 'broker']) {
    await expect(page.getByTestId(market)).toContainText('initial-A')
  }
  phase = 'old-A'
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshHoldings())
  await expect.poll(() => oldReleases.length).toBe(3)
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('profile-b'))
  for (const market of ['domestic', 'overseas', 'broker']) {
    await expect(page.getByTestId(market)).toContainText('current-B')
  }
  const afterB = await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries())
  expect(afterB).toHaveLength(3)
  expect(afterB.every(query => query.key[query.key.length - 4] === 'profile-b')).toBe(true)

  // Use the same production mutation again. Returning A must fetch fresh data,
  // and even explicit keepPreviousData must not show B or the first A result.
  await page.evaluate(() => { void (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('profile-a') })
  await expect.poll(() => returnReleases.length).toBeGreaterThanOrEqual(3)
  for (const market of ['domestic', 'overseas', 'broker']) {
    await expect(page.getByTestId(market)).not.toContainText('current-B')
    await expect(page.getByTestId(market)).not.toContainText('initial-A')
  }
  holdReturn = false
  returnReleases.splice(0).forEach(release => release())
  for (const market of ['domestic', 'overseas', 'broker']) {
    await expect(page.getByTestId(market)).toContainText('return-A')
  }
  // A's original cancelled request now finishes after the same A key exists again.
  // It must remain detached from the new query rather than overwriting fresh A.
  oldReleases.splice(0).forEach(release => release())
  await expect.poll(() => oldResponsesCompleted).toBe(3)
  await page.evaluate(() => new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  for (const market of ['domestic', 'overseas', 'broker']) {
    await expect(page.getByTestId(market)).toContainText('return-A')
    await expect(page.getByTestId(market)).not.toContainText('old-A')
  }
  expect(mutations).toEqual(['profile-b', 'profile-a'])
  for (const market of ['balance', 'overseas', 'broker']) {
    expect(requests.some(request => request.market === market && request.phase === 'current-B')).toBe(true)
    expect(requests.some(request => request.market === market && request.phase === 'return-A')).toBe(true)
  }
  const afterA = await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries())
  expect(afterA).toHaveLength(3)
  for (const query of afterA) {
    expect(query.key.slice(-4)).toEqual(['profile-a', 'kis', '11111111-01', true])
    const publication = query.key[query.key.length - 5]
    expect(typeof publication).toBe('number')
    expect(Number(publication)).toBeGreaterThan(Math.max(...afterB.map(previous =>
      Number(previous.key[previous.key.length - 5]))))
    expect(query.data).toMatchObject({ marker: 'return-A' })
  }
})


test('late A config refresh cannot overwrite a successful B profile publication', async ({ page }) => {
  await installHarness(page, true)
  const configB = { ...accountConfig, active_profile_id: 'profile-b', active_broker_account_id: '22222222-01' }
  let active = 'A'
  let delayConfig = false
  let releaseOldConfig: (() => void) | undefined
  let oldConfigCompleted = false
  await page.route('**/api/event-scope/*', async (route) => {
    const path = eventPath(route)
    if (path === 'set-active') {
      active = 'B'
      return route.fulfill({ json: configB })
    }
    if (path === 'config') {
      const captured = active === 'A' ? accountConfig : configB
      if (delayConfig) {
        await new Promise<void>(resolve => { releaseOldConfig = resolve })
        await route.fulfill({ json: captured })
        oldConfigCompleted = true
        return
      }
      return route.fulfill({ json: captured })
    }
    return route.fulfill({ json: { marker: `ACCOUNT_${active}` } })
  })
  await page.goto('/')
  await expect(page.getByTestId('domestic')).toContainText('ACCOUNT_A')
  delayConfig = true
  await page.evaluate(() => { void (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshConfig() })
  await expect.poll(() => !!releaseOldConfig).toBe(true)
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('profile-b'))
  for (const market of ['domestic', 'overseas', 'broker']) await expect(page.getByTestId(market)).toContainText('ACCOUNT_B')
  releaseOldConfig?.()
  await expect.poll(() => oldConfigCompleted).toBe(true)
  await page.evaluate(() => new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  expect(await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.config().active_profile_id)).toBe('profile-b')
  for (const market of ['domestic', 'overseas', 'broker']) await expect(page.getByTestId(market)).toContainText('ACCOUNT_B')
})

test('delete success with failed config refresh hides all old holdings and recovers only after a trusted config', async ({ page }) => {
  await installHarness(page, true)
  const configB = { ...accountConfig, active_profile_id: 'profile-b', active_broker_account_id: '22222222-01' }
  let deleted = false
  let failConfig = true
  let configRequests = 0
  const holdings: string[] = []
  await page.route('**/api/event-scope/*', async (route) => {
    const path = eventPath(route)
    if (path === 'delete') {
      deleted = true
      return route.fulfill({ json: null })
    }
    if (path === 'config') {
      configRequests += 1
      if (deleted && failConfig) return route.fulfill({ status: 500, json: { error: '현재 계좌 설정 조회 실패' } })
      return route.fulfill({ json: deleted ? configB : accountConfig })
    }
    holdings.push(`${path}:${deleted ? 'B' : 'A'}`)
    return route.fulfill({ json: { marker: deleted ? 'RECOVERED_B' : 'OLD_A' } })
  })
  await page.goto('/')
  for (const market of ['domestic', 'overseas', 'broker']) await expect(page.getByTestId(market)).toContainText('OLD_A')
  const error = await page.evaluate(async () => {
    try { await (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.deleteActive() }
    catch (error) { return (error as Error).message }
    return null
  })
  expect(error).toContain('현재 계좌 설정 조회 실패')
  expect(configRequests).toBe(2)
  await expect(page.getByRole('alert')).toContainText('현재 계좌 설정 조회 실패')
  await expect(page.getByTestId('holdings-error')).toContainText('현재 계좌 설정 조회 실패')
  for (const market of ['domestic', 'overseas', 'broker']) {
    await expect(page.getByTestId(market)).not.toContainText('OLD_A')
    await expect(page.getByTestId(market)).not.toContainText('RECOVERED_B')
  }
  expect(holdings.filter(request => request.endsWith(':B'))).toHaveLength(0)
  const queries = await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries())
  expect(queries.some(query => query.key[query.key.length - 4] === 'profile-a')).toBe(false)
  failConfig = false
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshConfig())
  for (const market of ['domestic', 'overseas', 'broker']) await expect(page.getByTestId(market)).toContainText('RECOVERED_B')
})

test('rapid B and C profile mutations serialize and only the final C scope supplies holdings', async ({ page }) => {
  await installHarness(page, true)
  let active = 'A'
  let releaseB: (() => void) | undefined
  let completedB = false
  let overlap = false
  const mutations: string[] = []
  const configFor = (id: string) => ({ ...accountConfig, active_profile_id: id, active_broker_account_id: `${id}-account` })
  await page.route('**/api/event-scope/*', async (route) => {
    const path = eventPath(route)
    if (path === 'config') return route.fulfill({ json: accountConfig })
    if (path === 'set-active') {
      const { id } = route.request().postDataJSON() as { id: string }
      mutations.push(id)
      if (id === 'profile-b') await new Promise<void>(resolve => { releaseB = resolve })
      if (id === 'profile-c' && !completedB) overlap = true
      active = id
      await route.fulfill({ json: configFor(id) })
      if (id === 'profile-b') completedB = true
      return
    }
    return route.fulfill({ json: { marker: `CURRENT_${active}` } })
  })
  await page.goto('/')
  await expect(page.getByTestId('domestic')).toContainText('CURRENT_A')
  await page.evaluate(() => {
    const harness = (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents
    void harness.switchAccount('profile-b')
    void harness.switchAccount('profile-c')
  })
  await expect.poll(() => !!releaseB).toBe(true)
  await expect.poll(() => page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.pendingProfileMutations())).toEqual([{ paused: false }, { paused: true }])
  expect(mutations).toEqual(['profile-b'])
  releaseB?.()
  for (const market of ['domestic', 'overseas', 'broker']) await expect(page.getByTestId(market)).toContainText('CURRENT_profile-c')
  expect(mutations).toEqual(['profile-b', 'profile-c'])
  expect(overlap).toBe(false)
  expect(await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.config().active_profile_id)).toBe('profile-c')
})


test('rejected profile publication that already applied B stays untrusted after recovery failure and later recovers B', async ({ page }) => {
  await installHarness(page, true)
  const configB = { ...accountConfig, active_profile_id: 'profile-b', active_broker_account_id: '22222222-01' }
  let appliedB = false
  let failRecovery = true
  let configRequests = 0
  const holdings: string[] = []
  await page.route('**/api/event-scope/*', async (route) => {
    const path = eventPath(route)
    if (path === 'set-active') {
      appliedB = true
      return route.fulfill({ status: 500, json: { error: '프로파일 적용 후 저장 실패' } })
    }
    if (path === 'config') {
      configRequests += 1
      if (appliedB && failRecovery) return route.fulfill({ status: 503, json: { error: '현재 계좌 복구 조회 실패' } })
      return route.fulfill({ json: appliedB ? configB : accountConfig })
    }
    holdings.push(`${path}:${appliedB ? 'B' : 'A'}`)
    return route.fulfill({ json: { marker: appliedB ? 'CONFIRMED_B' : 'PRIVATE_A' } })
  })
  await page.goto('/')
  for (const market of ['domestic', 'overseas', 'broker']) await expect(page.getByTestId(market)).toContainText('PRIVATE_A')
  const error = await page.evaluate(async () => {
    try { await (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('profile-b') }
    catch (error) { return (error as Error).message }
    return null
  })
  expect(error).toBe('프로파일 적용 후 저장 실패')
  expect(configRequests).toBe(2)
  await expect(page.getByRole('alert')).toContainText('현재 계좌 복구 조회 실패')
  await expect(page.getByTestId('holdings-error')).toContainText('현재 계좌 복구 조회 실패')
  for (const market of ['domestic', 'overseas', 'broker']) {
    await expect(page.getByTestId(market)).not.toContainText('PRIVATE_A')
    await expect(page.getByTestId(market)).not.toContainText('CONFIRMED_B')
  }
  expect(holdings.filter(request => request.endsWith(':B'))).toHaveLength(0)
  expect((await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries())).some(query => query.key[query.key.length - 4] === 'profile-a')).toBe(false)
  failRecovery = false
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshConfig())
  for (const market of ['domestic', 'overseas', 'broker']) await expect(page.getByTestId(market)).toContainText('CONFIRMED_B')
  const queries = await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries())
  for (const prefix of ['balance', 'overseasBalance', 'brokerHoldings']) {
    expect(queries.find(query => query.key[0] === prefix && query.key[query.key.length - 4] === 'profile-b')?.data).toMatchObject({ marker: 'CONFIRMED_B' })
  }
})


test('event fixture negative control rejects wrong endpoint method and budget account', async ({ page }) => {
  await installHarness(page, true, false)
  const denied: string[] = []
  await page.route('**/api/event-scope/*', async route => {
    try { eventPath(route, accountConfig) }
    catch (error) { denied.push(String(error)); return route.fulfill({ status: 400, json: { error: String(error) } }) }
    if (new URL(route.request().url()).pathname.endsWith('/config')) return route.fulfill({ json: accountConfig })
    throw new Error('Unexpected valid fixture request')
  })
  await page.goto('/')
  const statuses = await page.evaluate(async () => {
    const wrong = encodeURIComponent(JSON.stringify({ brokerId: 'kis', brokerAccountId: 'WRONG' }))
    return Promise.all([
      fetch('/api/event-scope/unknown').then(r => r.status),
      fetch('/api/event-scope/config', { method: 'POST' }).then(r => r.status),
      fetch('/api/event-scope/budget?input=' + wrong).then(r => r.status),
    ])
  })
  expect(statuses).toEqual([400, 400, 400])
  expect(denied).toHaveLength(3)
})


test('scope budget late save response after A to B to A cannot own success or revalidation', async ({ page }) => {
  await installHarness(page, true, true, true)
  let active: 'A' | 'B' = 'A', reads = 0, mutations = 0
  const configs = { A: accountConfig, B: { ...accountConfig, active_profile_id: 'profile-b', kis_is_paper_trading: false } }
  let release!: () => void
  await page.route('**/api/event-scope/*', async route => {
    const path = eventPath(route, configs[active])
    if (path === 'config') return route.fulfill({ json: configs[active] })
    if (path === 'set-active') {
      active = route.request().postDataJSON().id
      return route.fulfill({ json: configs[active] })
    }
    if (path === 'save-budget') {
      mutations++
      await new Promise<void>(resolve => { release = resolve })
      return route.fulfill({ json: { marker: 'OLD_A_SAVE' } })
    }
    if (path === 'budget') reads++
    return route.fulfill({ json: { marker: active + '-read-' + reads } })
  })
  await page.goto('/')
  await expect(page.getByTestId('budget')).toContainText('A-read')
  await page.evaluate(() => { void (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.saveBudget().catch(() => {}) })
  await expect.poll(() => mutations).toBe(1)
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('B'))
  await expect(page.getByTestId('budget')).toContainText('B-read')
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.switchAccount('A'))
  await expect(page.getByTestId('budget')).toContainText('A-read')
  const freshReads = reads
  release()
  await expect(page.getByTestId('save-status')).toHaveText('error:BUDGET_SCOPE_CHANGED')
  await expect(page.getByTestId('budget')).not.toContainText('OLD_A_SAVE')
  expect(reads).toBe(freshReads)
  expect(mutations).toBe(1)
})


test('scope config recovery with failed money reads never restores cached money', async ({ page }) => {
  await installHarness(page, true, true, true)
  let configFails = false, moneyFails = false, failedReads = 0
  await page.route('**/api/event-scope/*', async route => {
    const path = eventPath(route, accountConfig)
    if (path === 'config') return route.fulfill(configFails
      ? { status: 503, json: { error: 'Synthetic unknown config' } } : { json: accountConfig })
    if (['budget', 'recovery'].includes(path) && moneyFails) {
      await route.fulfill({ status: 503, json: { error: 'Synthetic post-recovery money failure' } })
      failedReads++
      return
    }
    return route.fulfill({ json: { marker: 'BEFORE_FAILURE' } })
  })
  await page.goto('/')
  for (const panel of ['budget', 'recovery']) await expect(page.getByTestId(panel)).toContainText('BEFORE_FAILURE')
  configFails = true
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshConfig().catch(() => {}))
  await expect(page.getByRole('alert')).toContainText('Synthetic unknown config')
  configFails = false; moneyFails = true
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshConfig())
  await expect(page.getByRole('alert')).toHaveCount(0)
  await expect.poll(() => failedReads).toBe(2)
  for (const panel of ['budget', 'recovery']) await expect(page.getByTestId(panel)).toBeEmpty()
})
