import { expect, type Locator, type Page } from '@playwright/test'
import { mkdir, writeFile } from 'node:fs/promises'
import { createHash } from 'node:crypto'

export const settingsPairs = [
  { section: '데이터 갱신 주기', label: '갱신 주기', unit: '초', min: 5, max: 300, step: 5, initial: 30, draft: '37', rounded: 35, family: 'refresh', command: 'set_refresh_config', field: 'interval_sec' },
  { section: '로그 설정', label: '보관 기간', unit: '일', min: 1, max: 30, step: 1, initial: 5, draft: '7', rounded: 7, family: 'log', command: 'set_log_config', field: 'retention_days' },
  { section: '로그 설정', label: '최대 용량', unit: 'MB', min: 10, max: 500, step: 10, initial: 100, draft: '136', rounded: 140, family: 'log', command: 'set_log_config', field: 'max_size_mb' },
  { section: '체결 기록 보관', label: '보관 기간', unit: '일', min: 1, max: 365, step: 1, initial: 90, draft: '95', rounded: 95, family: 'archive', command: 'set_trade_archive_config', field: 'retention_days' },
  { section: '체결 기록 보관', label: '최대 용량', unit: 'MB', min: 50, max: 2000, step: 50, initial: 500, draft: '523', rounded: 500, family: 'archive', command: 'set_trade_archive_config', field: 'max_size_mb' },
  { section: '리스크 관리', label: '일일 순손실 한도', unit: '원', min: 0, max: 5000000, step: 100000, initial: 500000, draft: '151000', rounded: 200000, family: 'risk', command: 'update_risk_config', field: 'dailyLossLimit' },
  { section: '리스크 관리', label: '종목당 최대 비중', unit: '%', min: 1, max: 100, step: 1, initial: 20, draft: '22.6', rounded: 23, family: 'risk', command: 'update_risk_config', field: 'maxPositionRatio' },
  { section: '리스크 관리', label: '전략/종목별 일일 매도 제한', unit: '회', min: 0, max: 10, step: 1, initial: 1, draft: '4', rounded: 4, family: 'risk', command: 'update_risk_config', field: 'maxDailySellOrdersPerSymbol' },
  { section: '리스크 관리', label: '전략/종목별 연속 손실 차단', unit: '회', min: 0, max: 10, step: 1, initial: 3, draft: '5', rounded: 5, family: 'risk', command: 'update_risk_config', field: 'maxConsecutiveLossesPerStrategySymbol' },
  { section: '리스크 관리', label: '거래당 위험 한도', unit: '%', min: 0, max: 10, step: 0.1, initial: 1, draft: '1.26', rounded: 1.3, family: 'risk', command: 'update_risk_config', field: 'riskPerTradeBps' },
  { section: '리스크 관리', label: 'ATR 손절 배수', unit: '배', min: 0.5, max: 10, step: 0.5, initial: 2, draft: '2.26', rounded: 2.5, family: 'risk', command: 'update_risk_config', field: 'atrStopMultiplier' },
] as const
export type SettingsPair = typeof settingsPairs[number]
export type SettingsWriteCommand = 'set_refresh_config' | 'set_log_config' | 'set_trade_archive_config' | 'update_risk_config'
export interface SettingsNativeCall { command: string; args: Record<string, unknown>; kind: 'read' | 'mock-write' | 'subscription' | 'denied'; id?: number; status?: 'pending' | 'success' | 'failure'; statusCode?: number }
export interface SettingsNativeSnapshot { calls: SettingsNativeCall[]; mockWrites: SettingsNativeCall[]; unknown: string[]; intentionalFailures: number[]; controlsAudit: unknown[] }
export interface SettingsFixtureOptions { riskEnabled?: boolean; volatility?: boolean; holdRisk?: boolean; riskFailures?: number }
export const baseRiskPayload = { dailyLossLimit: 500000, maxPositionRatio: 0.2, maxDailyBuyOrdersPerSymbol: 0, maxDailySellOrdersPerSymbol: 1,
  maxConsecutiveLossesPerStrategySymbol: 3, volatilitySizingEnabled: true, riskPerTradeBps: 100, atrStopMultiplier: 2 }

export async function keyboardSettingsFixture(page: Page, options: SettingsFixtureOptions = {}) {
  await page.clock.install()
  const external: string[] = [], apiRequests: string[] = [], pageErrors: string[] = [], consoleMessages: string[] = [], webSockets: string[] = []
  const notes: unknown[] = []
  page.on('pageerror', error => pageErrors.push(error.message))
  page.on('console', message => { if (['warning', 'error'].includes(message.type())) consoleMessages.push(message.text()) })
  await page.route('**/*', async route => {
    const url = new URL(route.request().url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { external.push(url.origin); return route.abort() }
    if (url.pathname.startsWith('/api/')) { apiRequests.push(`${route.request().method()} ${url.pathname}`); return route.abort() }
    return route.fallback()
  })
  await page.routeWebSocket('**/*', socket => { webSockets.push(socket.url()); socket.close() })
  await page.addInitScript(options => {
    const calls: SettingsNativeCall[] = []
    const mockWrites: typeof calls = [], unknown: string[] = [], intentionalFailures: number[] = [], controlsAudit: unknown[] = []
    const writeCommands = new Set(['set_refresh_config', 'set_log_config', 'set_trade_archive_config', 'update_risk_config'])
    const controls = new Map<string, { hold: boolean; failures: number }>([['get_risk_config', { hold: options.holdRisk ?? false, failures: options.riskFailures ?? 0 }]])
    const held = new Map<number, { command: string; release: () => void }>()
    let callbackId = 0, eventId = 0
    const callbacks = new Map<number, unknown>(), subscriptions = new Map<number, string>()
    const events = new Set(['exchange-rate-updated', 'exchange-rate-status-updated', 'balance-updated', 'overseas-balance-updated'])
    const log = { retention_days: 5, max_size_mb: 100, api_debug: false }
    const refresh = { interval_sec: 30 }, archive = { retention_days: 90, max_size_mb: 500 }
    const risk = { enabled: options.riskEnabled ?? true, dailyLossLimit: 500000, maxPositionRatio: 0.2, maxDailyBuyOrdersPerSymbol: 0,
      maxDailySellOrdersPerSymbol: 1, maxConsecutiveLossesPerStrategySymbol: 3, volatilitySizingEnabled: options.volatility ?? true,
      riskPerTradeBps: 100, atrStopMultiplier: 2, currentLoss: 0, dailyProfit: 0, netLoss: 0, emergencyStop: false,
      canTrade: true, lossRatio: 0, blockedStrategySymbolCount: 0, atrReadySymbolCount: 0 }
    // All values below are literals in this browser. No native backend, file, DB, profile or account is read.
    const reads: Record<string, unknown> = {
      get_app_config: { active_broker_id: 'kis', active_broker_account_id: null, active_profile_id: null,
        active_profile_name: '키보드 합성 검증', kis_app_key_masked: '', kis_account_no: '', kis_is_paper_trading: true,
        kis_configured: false, active_broker_configured: false, discord_enabled: false, notification_levels: [] },
      get_trading_status: { isRunning: false, activeStrategies: [], positionCount: 0, totalUnrealizedPnl: 0, wsConnected: false,
        tradingProfileId: null, tradingBrokerId: null, tradingAccountId: null, buySuspended: false, buySuspendedReason: null },
      check_config: { broker_id: 'kis', broker_account_id: null, real_key_set: false, real_account_set: false,
        paper_key_set: false, active_mode: '모의투자', is_ready: false, discord_configured: false, base_url: 'synthetic', issues: [] },
      list_profiles: [], get_refresh_config: refresh, get_refresh_interval: 30,
      get_log_config: log, get_trade_archive_config: archive,
      get_trade_archive_stats: { total_files: 0, size_bytes: 0, oldest_date: null, newest_date: null },
      get_risk_config: risk, get_broker_rate_limit_status: [],
      get_web_config: { runningPort: 7474, accessUrl: 'http://localhost:7474', distPath: '', distFound: false, lanEnabled: false, apiTokenConfigured: false },
      get_stock_list_stats: { count: 0, lastUpdatedAt: null, filePath: 'synthetic-only', updateIntervalHours: 24 },
      get_database_config: { provider: 'postgresql', host: '127.0.0.1', port: 5432, database: 'synthetic-only', username: 'synthetic-only',
        passwordConfigured: false, tlsMode: 'prefer', maxConnections: 5, activeBackend: 'json', configured: false },
      inspect_json_storage: { fileCount: 0, sizeBytes: 0, categories: [] },
      check_for_update: { hasUpdate: false, currentVersion: '0.0.0-fixture', latestVersion: '0.0.0-fixture', releaseUrl: '', releaseNotes: null },
    }
    const deny = (command: string, args: Record<string, unknown>, reason: string): never => {
      calls.push({ command, args: structuredClone(args), kind: 'denied' }); unknown.push(`${command}: ${reason}`)
      throw { code: 'KEYBOARD_FIXTURE_DENIED', message: `Synthetic fixture denied ${command}: ${reason}` }
    }
    const execute = async (command: string, args: Record<string, unknown>, kind: 'read' | 'mock-write', produce: () => unknown) => {
      const call: SettingsNativeCall = { id: calls.length + 1, command, args: structuredClone(args), kind, status: 'pending' }
      calls.push(call); if (kind === 'mock-write') mockWrites.push(call)
      const control = controls.get(command)
      if (control?.hold) await new Promise<void>(release => held.set(call.id!, { command, release }))
      if (control && control.failures > 0) {
        control.failures--; call.status = 'failure'; call.statusCode = 500; intentionalFailures.push(call.id!)
        throw { code: 'SYNTHETIC_SETTINGS_500', message: 'Intentional browser-only settings failure' }
      }
      const result = produce(); call.status = 'success'; call.statusCode = 200
      return structuredClone(result)
    }
    const release = (command: string) => {
      const control = controls.get(command); if (control) control.hold = false
      for (const [id, pending] of held) if (pending.command === command) { held.delete(id); pending.release() }
    }
    const finite = (value: unknown, min: number, max: number, integer = false) => typeof value === 'number' && Number.isFinite(value)
      && value >= min && value <= max && (!integer || Number.isInteger(value))
    const inputFor = (command: string, args: Record<string, unknown>, keys: string[]) => {
      const input = args.input
      if (Object.keys(args).join(',') !== 'input' || !input || typeof input !== 'object' || Array.isArray(input)
        || Object.keys(input).sort().join(',') !== [...keys].sort().join(',')) return deny(command, args, 'unexpected input shape')
      return input as Record<string, unknown>
    }
    Object.assign(window, {
      __kbd04Settings: { calls, mockWrites, unknown, intentionalFailures, controlsAudit,
        configure: (command: string, input: { hold?: boolean; failures?: number }) => {
          if ((!writeCommands.has(command) && command !== 'get_risk_config') || Object.keys(input).some(key => !['hold', 'failures'].includes(key))
            || (input.hold !== undefined && typeof input.hold !== 'boolean') || (input.failures !== undefined && !finite(input.failures, 0, 3, true))) return deny(command, {}, 'invalid synthetic control')
          controls.set(command, { hold: input.hold ?? false, failures: input.failures ?? 0 }); controlsAudit.push({ command, ...input })
        }, release, releaseAll: () => { for (const command of controls.keys()) release(command) },
        setRisk: (patch: Record<string, unknown>) => {
          if (Object.keys(patch).some(key => !Object.prototype.hasOwnProperty.call(risk, key))) return deny('setRisk-fixture-only', {}, 'unknown synthetic risk field')
          Object.assign(risk, patch); controlsAudit.push({ fixtureOnlyRiskPatch: structuredClone(patch) })
        },
      },
      __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => {} },
      __TAURI_INTERNALS__: {
        transformCallback: (callback: unknown) => { callbacks.set(++callbackId, callback); return callbackId },
        unregisterCallback: (id: number) => { callbacks.delete(id) },
        invoke: async (command: string, args: Record<string, unknown> = {}) => {
          if (Object.prototype.hasOwnProperty.call(reads, command)) {
            if (Object.keys(args).length) return deny(command, args, 'read arguments are not allowed')
            return execute(command, args, 'read', () => reads[command])
          }
          if (command === 'plugin:event|listen') {
            const target = args.target as { kind?: string } | undefined
            if (!events.has(String(args.event)) || target?.kind !== 'Any' || !callbacks.has(Number(args.handler))
              || Object.keys(args).sort().join(',') !== 'event,handler,target') return deny(command, args, 'unexpected subscription')
            calls.push({ command, args: structuredClone(args), kind: 'subscription' })
            subscriptions.set(++eventId, String(args.event)); return eventId
          }
          if (command === 'plugin:event|unlisten') {
            if (subscriptions.get(Number(args.eventId)) !== args.event || Object.keys(args).sort().join(',') !== 'event,eventId') return deny(command, args, 'unexpected unsubscribe')
            calls.push({ command, args: structuredClone(args), kind: 'subscription' }); subscriptions.delete(Number(args.eventId)); return undefined
          }
          if (command === 'set_refresh_config') {
            if (Object.keys(args).join(',') !== 'interval_sec' || !finite(args.interval_sec, 5, 300, true)) return deny(command, args, 'unexpected refresh payload')
            return execute(command, args, 'mock-write', () => { refresh.interval_sec = Number(args.interval_sec); reads.get_refresh_interval = refresh.interval_sec; return refresh })
          }
          if (command === 'set_log_config') {
            const input = inputFor(command, args, ['retention_days', 'max_size_mb', 'api_debug'])
            if (!finite(input.retention_days, 1, 30, true) || !finite(input.max_size_mb, 10, 500, true) || input.api_debug !== false) return deny(command, args, 'unexpected log payload')
            return execute(command, args, 'mock-write', () => { Object.assign(log, input); return log })
          }
          if (command === 'set_trade_archive_config') {
            const input = inputFor(command, args, ['retention_days', 'max_size_mb'])
            if (!finite(input.retention_days, 1, 365, true) || !finite(input.max_size_mb, 50, 2000, true)) return deny(command, args, 'unexpected archive payload')
            return execute(command, args, 'mock-write', () => { Object.assign(archive, input); return archive })
          }
          if (command === 'update_risk_config') {
            const input = inputFor(command, args, ['dailyLossLimit', 'maxPositionRatio', 'maxDailyBuyOrdersPerSymbol', 'maxDailySellOrdersPerSymbol',
              'maxConsecutiveLossesPerStrategySymbol', 'volatilitySizingEnabled', 'riskPerTradeBps', 'atrStopMultiplier'])
            if (!finite(input.dailyLossLimit, 0, 5000000, true) || !finite(input.maxPositionRatio, .01, 1)
              || input.maxDailyBuyOrdersPerSymbol !== 0 || !finite(input.maxDailySellOrdersPerSymbol, 0, 10, true)
              || !finite(input.maxConsecutiveLossesPerStrategySymbol, 0, 10, true) || typeof input.volatilitySizingEnabled !== 'boolean'
              || !finite(input.riskPerTradeBps, 0, 1000, true) || !finite(input.atrStopMultiplier, .5, 10)) return deny(command, args, 'unexpected risk payload')
            return execute(command, args, 'mock-write', () => { Object.assign(risk, input); return risk })
          }
          return deny(command, args, 'not on the explicit allowlist')
        },
      },
    })
  }, options)
  const snapshot = () => page.evaluate(() => {
    const state = Reflect.get(window, '__kbd04Settings') as SettingsNativeSnapshot
    return structuredClone({ calls: state.calls, mockWrites: state.mockWrites, unknown: state.unknown, intentionalFailures: state.intentionalFailures, controlsAudit: state.controlsAudit })
  })
  const configure = (command: SettingsWriteCommand | 'get_risk_config', input: { hold?: boolean; failures?: number }) => page.evaluate(({ command, input }) =>
    (Reflect.get(window, '__kbd04Settings') as { configure: (command: string, input: { hold?: boolean; failures?: number }) => void }).configure(command, input), { command, input })
  const release = (command: SettingsWriteCommand | 'get_risk_config') => page.evaluate(command =>
    (Reflect.get(window, '__kbd04Settings') as { release: (command: string) => void }).release(command), command)
  const setRisk = (patch: Record<string, unknown>) => page.evaluate(patch =>
    (Reflect.get(window, '__kbd04Settings') as { setRisk: (patch: Record<string, unknown>) => void }).setRisk(patch), patch)
  const drain = async () => {
    await page.evaluate(() => (Reflect.get(window, '__kbd04Settings') as { releaseAll: () => void }).releaseAll())
    await expect.poll(async () => (await snapshot()).calls.filter(call => call.status === 'pending').length).toBe(0)
    await page.clock.runFor(1); await page.evaluate(() => Promise.resolve())
  }
  return { notes, external, apiRequests, pageErrors, consoleMessages, webSockets, snapshot, configure, release, setRisk, drain }
}
export type SettingsFixture = Awaited<ReturnType<typeof keyboardSettingsFixture>>

const escapeRegex = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
export function settingsControl(page: Page, pair: SettingsPair) {
  const region = page.getByRole('region', { name: pair.section, exact: true })
  // Baseline has no accessible control name. Scope by the existing visible label until that red is fixed.
  const label = region.getByText(new RegExp(`^${escapeRegex(pair.label)}:`))
  const block = label.locator('..')
  return { pair, region, label, block, slider: block.getByRole('slider'), input: block.getByRole('textbox') }
}
export async function openSettingsContract(page: Page) {
  await page.goto('/settings')
  await expect(page.getByRole('heading', { name: 'Settings', exact: true })).toBeVisible()
  await expect(page.getByRole('region', { name: '리스크 관리', exact: true }).getByRole('slider')).toHaveCount(6)
  await expect(settingsControl(page, settingsPairs[10]).input).toHaveValue('2')
}
export async function controlNamingObservation(control: Locator) {
  const count = await control.count()
  if (count !== 1) return { count, ariaSnapshot: null, links: null }
  return { count, ariaSnapshot: await control.ariaSnapshot(), links: await control.evaluate(element => {
    const resolve = (attribute: string) => (element.getAttribute(attribute) ?? '').split(/\s+/).filter(Boolean)
      .map(id => ({ id, text: document.getElementById(id)?.textContent ?? null }))
    const field = element as HTMLInputElement
    return { id: element.id, ariaLabel: element.getAttribute('aria-label'), labelledBy: resolve('aria-labelledby'),
      describedBy: resolve('aria-describedby'), labels: [...(field.labels ?? [])].map(label => label.textContent ?? ''), valueText: element.getAttribute('aria-valuetext') }
  }) }
}
export async function nativeBlurCount(input: Locator) {
  const element = await input.elementHandle()
  if (!element) throw new Error('No numeric input for blur observation')
  await element.evaluate(field => { Reflect.set(field, '__kbd04Blurs', 0); field.addEventListener('blur', () => Reflect.set(field, '__kbd04Blurs', Number(Reflect.get(field, '__kbd04Blurs')) + 1), { passive: true }) })
  return () => element.evaluate(field => Number(Reflect.get(field, '__kbd04Blurs')))
}
export async function safeSettings(state: SettingsFixture) {
  expect(state.external).toEqual([]); expect(state.apiRequests).toEqual([]); expect(state.pageErrors).toEqual([]); expect(state.webSockets).toEqual([])
  const native = await state.snapshot()
  expect(native.unknown).toEqual([])
  expect(native.mockWrites.every(call => ['set_refresh_config', 'set_log_config', 'set_trade_archive_config', 'update_risk_config'].includes(call.command))).toBe(true)
}
export async function captureSettings(page: Page, state: SettingsFixture, title: string, status: string | undefined) {
  await state.drain()
  const directory = '.cache/kbd04-implementation-20261031/evidence'
  await mkdir(directory, { recursive: true })
  const prefix = title.replace(/[^a-zA-Z0-9-]/g, '_').slice(0, 100)
  const hash = createHash('sha256').update(title, 'utf8').digest('hex').slice(0, 12)
  const name = `${prefix}-${hash}`
  await writeFile(`${directory}/${name}.json`, JSON.stringify({ title, status, notes: state.notes, native: await state.snapshot(), external: state.external,
    apiRequests: state.apiRequests, pageErrors: state.pageErrors, webSockets: state.webSockets, consoleMessages: state.consoleMessages,
    safetyMeaning: 'Explicit browser mock settings writes are counted; actual native/API/settings/account/DB/trades are never invoked' }, null, 2))
  await page.screenshot({ path: `${directory}/${name}.png`, animations: 'disabled' })
}

export function namedSettingsControl(page: Page, pair: SettingsPair) {
  const region = page.getByRole('region', { name: pair.section, exact: true })
  return { pair, region, slider: region.getByRole('slider', { name: pair.label, exact: true }), input: region.getByRole('textbox', { name: pair.label, exact: true }) }
}
export async function settingsTick(page: Page, milliseconds = 1) { await page.clock.runFor(milliseconds); await page.evaluate(() => Promise.resolve()) }
export const riskSave = (page: Page) => page.getByRole('region', { name: '리스크 관리', exact: true }).getByRole('button', { name: '저장', exact: true })
export function expectedSettingsArgs(pair: SettingsPair, value: number) {
  if (pair.family === 'refresh') return { interval_sec: value }
  if (pair.family === 'log') return { input: { retention_days: 5, max_size_mb: 100, api_debug: false, [pair.field]: value } }
  if (pair.family === 'archive') return { input: { retention_days: 90, max_size_mb: 500, [pair.field]: value } }
  return { input: { ...baseRiskPayload, [pair.field]: pair.field === 'maxPositionRatio' ? value / 100 : pair.field === 'riskPerTradeBps' ? Math.round(value * 100) : value } }
}
export async function waitSettingsWrite(state: SettingsFixture, command: SettingsWriteCommand, ordinal: number, status?: 'pending' | 'success' | 'failure') {
  await expect.poll(async () => (await state.snapshot()).mockWrites.filter(call => call.command === command).length).toBe(ordinal)
  if (status) await expect.poll(async () => (await state.snapshot()).mockWrites.filter(call => call.command === command)[ordinal - 1]?.status).toBe(status)
  return (await state.snapshot()).mockWrites.filter(call => call.command === command)[ordinal - 1]
}
