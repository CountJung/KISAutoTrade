import { expect, test, type Page } from '@playwright/test'

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
  scopeQueries: () => { key: unknown[], data: unknown }[]
  refreshConfig: () => Promise<void>
  deleteActive: () => Promise<void>
  config: () => { active_profile_id: string | null }
  pendingProfileMutations: () => { paused: boolean }[]
}

/** Mount the production event hook with real React Query observers and Tauri's
 * event mock. Only the application entry point is replaced; all HTTP is local
 * Playwright fixtures and no trading commands or provider endpoints are used. */
async function installHarness(page: Page, scoped = false, initiallyEnabled = true) {
  await page.route('**/src/main.tsx', async (route) => {
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
      import { QueryClient, QueryClientProvider, useQuery } from ${dependency('@tanstack_react-query')};
      import { useBackendEvents } from '/src/api/backendEvents.ts';
      import { KEYS } from '/src/api/queryKeys.ts';
      import { useBalance, useOverseasBalance, useBrokerHoldings, useSetActiveProfile, useDeleteProfile, useAppConfig } from '/src/api/accountHooks.ts';
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
        const paths = { get_app_config: 'config', get_balance: 'balance', get_overseas_balance: 'overseas', get_broker_holdings: 'broker' };
        if (paths[command]) return fetch('/api/event-scope/' + paths[command]).then(json);
        if (command === 'set_active_profile') return fetch('/api/event-scope/set-active', { method: 'POST', body: JSON.stringify(args) }).then(json);
        if (command === 'delete_profile') return fetch('/api/event-scope/delete', { method: 'POST', body: JSON.stringify(args) }).then(json);
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
        switchProfile = next => mutation.mutateAsync(next);
        deleteActive = () => deletion.mutateAsync('profile-a');
        return React.createElement('main', null,
          React.createElement('pre', { 'data-testid': 'domestic' }, JSON.stringify(domestic.data)),
          React.createElement('pre', { 'data-testid': 'overseas' }, JSON.stringify(overseas.data)),
          React.createElement('pre', { 'data-testid': 'broker' }, JSON.stringify(broker.data)),
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
        config: () => client.getQueryData(KEYS.appConfig),
        pendingProfileMutations: () => client.getMutationCache().getAll().filter(mutation => mutation.state.status === 'pending').map(mutation => ({ paused: mutation.state.isPaused })),
        refreshHoldings: () => { [KEYS.balance, KEYS.overseasBalance, KEYS.brokerHoldings].forEach(queryKey => { void client.invalidateQueries({ queryKey }); }); },
        enableHoldings: () => enableHoldings(),
        scopeQueries: () => client.getQueryCache().getAll().filter(query => ['balance', 'overseasBalance', 'brokerHoldings'].includes(query.queryKey[0])).map(query => ({ key: query.queryKey, data: query.state.data })),
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
      const path = new URL(route.request().url()).pathname.split('/').pop()
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
      const query = queries.find(query => query.key[0] === prefix && query.key[1] === nextConfig.active_profile_id &&
        query.key[2] === nextConfig.active_broker_id && query.key[3] === nextConfig.active_broker_account_id && query.key[4] === nextConfig.kis_is_paper_trading)
      expect(query?.data).toMatchObject({ account: 'B' })
    }
  })
}

test('production holdings hooks wait for config and preserve disabled function options with legacy null profile', async ({ page }) => {
  await installHarness(page, true, false)
  let releaseConfig: (() => void) | undefined
  const holdings: string[] = []
  await page.route('**/api/event-scope/*', async (route) => {
    const path = new URL(route.request().url()).pathname.split('/').pop()
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
  await expect.poll(() => page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries().some(query => query.key[1] === null && query.key[2] === 'kis'))).toBe(true)
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
    const path = new URL(route.request().url()).pathname.split('/').pop()
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
  expect(afterB.every(query => query.key[1] === 'profile-b')).toBe(true)

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
    expect(query.key.slice(1)).toEqual(['profile-a', 'kis', '11111111-01', true])
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
    const path = new URL(route.request().url()).pathname.split('/').pop()
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
    const path = new URL(route.request().url()).pathname.split('/').pop()
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
  expect(queries.some(query => query.key[1] === 'profile-a')).toBe(false)
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
    const path = new URL(route.request().url()).pathname.split('/').pop()
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
    const path = new URL(route.request().url()).pathname.split('/').pop()
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
  expect((await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries())).some(query => query.key[1] === 'profile-a')).toBe(false)
  failRecovery = false
  await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.refreshConfig())
  for (const market of ['domestic', 'overseas', 'broker']) await expect(page.getByTestId(market)).toContainText('CONFIRMED_B')
  const queries = await page.evaluate(() => (window as unknown as { __balanceEvents: EventHarness }).__balanceEvents.scopeQueries())
  for (const prefix of ['balance', 'overseasBalance', 'brokerHoldings']) {
    expect(queries.find(query => query.key[0] === prefix && query.key[1] === 'profile-b')?.data).toMatchObject({ marker: 'CONFIRMED_B' })
  }
})
