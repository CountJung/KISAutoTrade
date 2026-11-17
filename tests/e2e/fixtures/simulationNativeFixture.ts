import { expect, type Page } from '@playwright/test'
import { syntheticSectorPortfolioInput } from '../../../src/pages/strategy/model/sectorPortfolioInput'
import { requestFixture } from './sectorDataFixture'
import { fixtureReport } from './sectorPortfolioFixture'

type DelayedCommand = 'get_sector_quote_scope' | 'collect_sector_data' | 'preview_sector_portfolio'
export type NativeCall = { command: string; args: Record<string, unknown>; status: 'pending' | 'success' | 'denied' }
export type NativeSnapshot = { calls: NativeCall[]; denied: string[]; fakeBalanceUsd: number }
type QueryState = { profileId: string; dataUpdateCount: number; dataUpdatedAt: number; publishing: number }
type TestControl = { switchProfile: (id: string) => Promise<unknown>; republish: (releaseCommand?: string) => { before: QueryState; after: QueryState }; queryState: () => QueryState }
type NativeControl = NativeSnapshot & { configure: (command: string) => void; release: (command: string) => void }

export async function simulationNativeFixture(page: Page) {
  await page.clock.setFixedTime(new Date('2026-11-06T12:00:00Z'))
  const violations: string[] = [], errors: string[] = [], http: string[] = []
  page.on('pageerror', error => errors.push(error.message))
  await page.route('**/*', async route => {
    const request = route.request(), url = new URL(request.url())
    if (!['localhost', '127.0.0.1'].includes(url.hostname) || request.method() !== 'GET' || url.pathname.startsWith('/api/')) {
      http.push(`${request.method()} ${url.origin}${url.pathname}`)
      return route.abort()
    }
    return route.fallback()
  })
  await page.routeWebSocket('**/*', socket => {
    const url = new URL(socket.url())
    if (!(['localhost', '127.0.0.1'].includes(url.hostname) && url.pathname === '/' && url.searchParams.has('token'))) violations.push(`WebSocket ${url.origin}${url.pathname}`)
    socket.close()
  })
  const request = requestFixture()
  request.bundle.manifest.synthetic = false
  const bundle = structuredClone(request.bundle)
  bundle.manifest.calendarSource = 'LATE_COLLECT_MUST_NOT_APPLY'
  const input = syntheticSectorPortfolioInput()
  await page.addInitScript(({ bundle, report }) => {
    let active = 'A', callbackId = 0, eventId = 0
    const calls: NativeCall[] = [], denied: string[] = []
    const holds = new Set<string>(), pending = new Map<number, { command: string; release: () => void }>()
    const callbacks = new Map<number, unknown>(), subscriptions = new Map<number, string>()
    const events = new Set(['exchange-rate-updated', 'exchange-rate-status-updated', 'balance-updated', 'overseas-balance-updated'])
    const config = () => ({ active_profile_id: active, active_profile_name: `합성 프로필 ${active}`, active_broker_id: 'kis',
      active_broker_account_id: null, kis_app_key_masked: '', kis_account_no: '', kis_is_paper_trading: true,
      kis_configured: false, active_broker_configured: false, discord_enabled: false, notification_levels: [] })
    const reject = (command: string, args: Record<string, unknown>): never => {
      denied.push(command); calls.push({ command, args: structuredClone(args), status: 'denied' })
      throw new Error(`SIMULATION_FIXTURE_DENIED: ${command}`)
    }
    const execute = async (command: string, args: Record<string, unknown>, result: unknown) => {
      const call: NativeCall = { command, args: structuredClone(args), status: 'pending' }
      calls.push(call)
      if (holds.has(command)) await new Promise<void>(release => pending.set(calls.length, { command, release }))
      call.status = 'success'
      return structuredClone(result)
    }
    const release = (command: string) => {
      holds.delete(command)
      for (const [id, value] of pending) if (value.command === command) { pending.delete(id); value.release() }
    }
    Reflect.set(window, '__simulationNative', {
      calls, denied, fakeBalanceUsd: 99_999,
      configure: (command: string) => {
        if (!['get_sector_quote_scope', 'collect_sector_data', 'preview_sector_portfolio'].includes(command)) return reject(command, {})
        holds.add(command)
      }, release,
    })
    Reflect.set(window, '__TAURI_EVENT_PLUGIN_INTERNALS__', { unregisterListener: () => {} })
    Reflect.set(window, '__TAURI_INTERNALS__', {
      transformCallback: (callback: unknown) => { callbacks.set(++callbackId, callback); return callbackId },
      unregisterCallback: (id: number) => { callbacks.delete(id) },
      invoke: async (command: string, args: Record<string, unknown> = {}) => {
        if (['get_app_config', 'get_trading_status', 'check_for_update', 'get_sector_quote_scope'].includes(command)) {
          if (Object.keys(args).length) return reject(command, args)
          if (command === 'get_app_config') return execute(command, args, config())
          if (command === 'get_sector_quote_scope') return execute(command, args, { profileId: active, provider: 'kis', paper: true })
          if (command === 'check_for_update') return execute(command, args, { hasUpdate: false, currentVersion: '0.0.0-fixture', latestVersion: '0.0.0-fixture', releaseUrl: '', releaseNotes: null })
          return execute(command, args, { isRunning: false, activeStrategies: [], positionCount: 0, totalUnrealizedPnl: 0, wsConnected: false,
            tradingProfileId: null, tradingBrokerId: null, tradingAccountId: null, buySuspended: false, buySuspendedReason: null })
        }
        if (command === 'set_active_profile') {
          if (Object.keys(args).join(',') !== 'id' || !['A', 'B'].includes(String(args.id))) return reject(command, args)
          active = String(args.id)
          return execute(command, args, config())
        }
        if (command === 'collect_sector_data') {
          const input = args.input as { expectedProfileId?: string; authenticatedQuotesAcknowledged?: boolean; manifest?: unknown } | undefined
          if (Object.keys(args).join(',') !== 'input' || input?.expectedProfileId !== active || input.authenticatedQuotesAcknowledged !== true || !input.manifest) return reject(command, args)
          return execute(command, args, bundle)
        }
        if (command === 'preview_sector_portfolio') {
          if (Object.keys(args).join(',') !== 'input' || !args.input) return reject(command, args)
          return execute(command, args, report)
        }
        if (command === 'plugin:event|listen') {
          const target = args.target as { kind?: string } | undefined
          if (Object.keys(args).sort().join(',') !== 'event,handler,target' || !events.has(String(args.event)) || target?.kind !== 'Any' || !callbacks.has(Number(args.handler))) return reject(command, args)
          subscriptions.set(++eventId, String(args.event)); return execute(command, args, eventId)
        }
        if (command === 'plugin:event|unlisten') {
          if (Object.keys(args).sort().join(',') !== 'event,eventId' || subscriptions.get(Number(args.eventId)) !== args.event) return reject(command, args)
          subscriptions.delete(Number(args.eventId)); return execute(command, args, undefined)
        }
        // Includes balances, credentials, profile lists, orders, start/stop and all unknowns.
        return reject(command, args)
      },
    })
  }, { bundle, report: fixtureReport(input) })
  const snapshot = () => page.evaluate(() => {
    const native = Reflect.get(window, '__simulationNative') as NativeControl
    return structuredClone({ calls: native.calls, denied: native.denied, fakeBalanceUsd: native.fakeBalanceUsd })
  })
  const configure = (command: DelayedCommand) => page.evaluate(command => (Reflect.get(window, '__simulationNative') as NativeControl).configure(command), command)
  const release = (command: DelayedCommand) => page.evaluate(command => (Reflect.get(window, '__simulationNative') as NativeControl).release(command), command)
  const switchProfile = (id: 'A' | 'B') => page.evaluate(id => (Reflect.get(window, '__simulationTest') as TestControl).switchProfile(id), id)
  const republish = (releaseCommand?: DelayedCommand) => page.evaluate(command => (Reflect.get(window, '__simulationTest') as TestControl).republish(command), releaseCommand)
  const queryState = () => page.evaluate(() => (Reflect.get(window, '__simulationTest') as TestControl).queryState())
  await page.goto('/tests/e2e/fixtures/simulationAppHarness.html')
  await expect(page.getByRole('heading', { name: '시뮬레이션', exact: true })).toBeVisible()
  await expect(page.getByText('시세 조회 프로필: 합성 프로필 A', { exact: false })).toBeVisible()
  await expect.poll(async () => (await queryState()).publishing).toBe(0)
  return { request, input, snapshot, configure, release, switchProfile, republish, queryState, violations, errors, http }
}

export async function assertSimulationSafe(state: Awaited<ReturnType<typeof simulationNativeFixture>>) {
  const native = await state.snapshot()
  expect(native.denied).toEqual([])
  expect(native.calls.filter(call => /balance|holdings|order|start_trading|stop_trading|profiles|credential/.test(call.command))).toEqual([])
  expect(state.http).toEqual([])
  expect(state.violations).toEqual([])
  expect(state.errors).toEqual([])
}
