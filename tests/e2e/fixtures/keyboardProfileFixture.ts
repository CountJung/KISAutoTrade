import { expect, type Locator, type Page } from '@playwright/test'
import { mkdir, writeFile } from 'node:fs/promises'
import { createHash } from 'node:crypto'
import type { AccountProfileView, AppConfigView } from '../../../src/shared/api/types'
import { mockUx } from './uxReviewFixture'

export const fakeProfiles: AccountProfileView[] = [
  { id: 'kbd05-kis-a', broker_id: 'kis', broker_account_id: '00000000-01', name: '합성 KIS A', is_paper_trading: true,
    live_trading_consent: false, app_key_masked: 'SYNTHETIC-***', account_no: '00000000-01', is_active: true, is_configured: true },
  { id: 'kbd05-kis-b', broker_id: 'kis', broker_account_id: '00000001-01', name: '합성 KIS B', is_paper_trading: true,
    live_trading_consent: false, app_key_masked: 'SYNTHETIC-***', account_no: '00000001-01', is_active: false, is_configured: true },
  { id: 'kbd05-toss-a', broker_id: 'toss', broker_account_id: '7101', name: '합성 Toss A', is_paper_trading: false,
    live_trading_consent: false, app_key_masked: 'SYNTHETIC-***', account_no: '7101', is_active: false, is_configured: true },
  { id: 'kbd05-toss-b', broker_id: 'toss', broker_account_id: '7102', name: '합성 Toss B', is_paper_trading: false,
    live_trading_consent: false, app_key_masked: 'SYNTHETIC-***', account_no: '7102', is_active: false, is_configured: true },
]
export const addPath = '/api/profiles/add', updatePath = '/api/profiles/update', deletePath = '/api/profiles/delete'
export const newTossAccountsPath = '/api/toss-accounts', savedTossAccountsPath = '/api/profiles/kbd05-toss-a/toss-accounts'
export const savedAccountsPath = (id: string) => `/api/profiles/${id}/toss-accounts`
export const oldAccounts = [{ account_seq: '7991', account_no_masked: 'SYNTHETIC-***', account_type: 'synthetic', label: '합성 이전 계좌' }]
interface Reply { status: number; json: unknown }
export interface ProfileTicket { id: number; path: string; method: string; body: unknown; kind: 'mock-profile-write' | 'aux-read-post' | 'synthetic-get';
  held: boolean; released: boolean; status: number; bodyFinished: boolean; browserError?: string; done: Promise<void>; complete: () => void }

export async function keyboardProfileFixture(page: Page, options: { retainDeletedProfiles?: boolean } = {}) {
  await page.clock.install()
  const profiles = structuredClone(fakeProfiles)
  const config: AppConfigView = { active_broker_id: 'kis', active_broker_account_id: fakeProfiles[0].broker_account_id,
    active_profile_id: fakeProfiles[0].id, active_profile_name: fakeProfiles[0].name, kis_app_key_masked: 'SYNTHETIC-***',
    kis_account_no: fakeProfiles[0].account_no, kis_is_paper_trading: true, kis_configured: true, active_broker_configured: true,
    discord_enabled: false, notification_levels: [] }
  const audit = await mockUx(page, { activeBroker: 'kis', appConfigOverride: config })
  const tickets: ProfileTicket[] = [], notes: unknown[] = [], webSockets: string[] = [], consoleMessages: string[] = []
  page.on('console', message => { if (['warning', 'error'].includes(message.type())) consoleMessages.push(message.text()) })
  const heldPaths = new Set<string>(), replies = new Map<string, Reply>()
  page.on('websocket', socket => webSockets.push(socket.url()))
  const mutations = new Set([addPath, updatePath, deletePath])
  const savedTossPaths = new Set(fakeProfiles.filter(profile => profile.broker_id === 'toss').map(profile => savedAccountsPath(profile.id)))
  const auxPaths = new Set(['/api/detect-trading-type', newTossAccountsPath, ...savedTossPaths])
  replies.set(newTossAccountsPath, { status: 200, json: oldAccounts })
  for (const path of savedTossPaths) replies.set(path, { status: 200, json: oldAccounts })
  replies.set('/api/detect-trading-type', { status: 200, json: { is_paper_trading: false, message: '합성 KIS 감지' } })
  const run = async (route: import('@playwright/test').Route, kind: ProfileTicket['kind'], produce: () => unknown) => {
    const request = route.request(), path = new URL(request.url()).pathname
    const selected = replies.get(path)
    let release!: () => void
    const gate = new Promise<void>(resolve => { release = resolve })
    const ticket: ProfileTicket = { id: tickets.length + 1, path, method: request.method(), body: request.postData() ? request.postDataJSON() : null,
      kind, held: heldPaths.has(path), released: false, status: selected?.status ?? 200, bodyFinished: false, done: Promise.resolve(),
      complete: () => { ticket.released = true; release() } }
    tickets.push(ticket)
    ticket.done = page.waitForResponse(response => response.headers()['x-kbd05-ticket'] === String(ticket.id)).then(async response => {
      const failure = await response.finished(); if (failure) throw failure
      expect(response.status()).toBe(ticket.status); ticket.bodyFinished = true
    }).catch(error => { ticket.browserError = String(error) })
    if (ticket.held) await gate
    let json: unknown
    if (selected) { if (kind === 'mock-profile-write' && ticket.status < 400) produce(); json = selected.json }
    else json = produce()
    await route.fulfill({ status: ticket.status, json, headers: { 'x-kbd05-ticket': String(ticket.id) } })
  }
  for (const [path, produce] of [
    ['/api/profiles', () => profiles], ['/api/app-config', () => config],
    ['/api/check-config', () => ({ broker_id: config.active_broker_id, broker_account_id: config.active_broker_account_id,
      real_key_set: false, real_account_set: false, paper_key_set: true, active_mode: '모의투자', is_ready: true, discord_configured: false, base_url: 'synthetic', issues: [] })],
    ['/api/strategies', () => []],
  ] as const) audit.responses.set(path, route => run(route, 'synthetic-get', produce))
  const currency = { allocatedAmount: 0, cashAmount: 0, reservedAmount: 0, availableAmount: 0, ownedPositionCount: 0, blockedReason: null, feeBufferBps: 100 }
  audit.responses.set('/api/auto-trading-budget', async route => { await route.fulfill({ json: { scope: { brokerId: config.active_broker_id, accountId: config.active_broker_account_id }, krw: currency, usd: currency } }) })
  const safeCredential = (value: unknown) => typeof value === 'string' && value.startsWith('SYNTHETIC-')
  const safeAccount = (value: unknown) => typeof value === 'string' && (/^0000000\d-01$/.test(value) || /^7\d{3}$/.test(value))
  const deny = async (route: import('@playwright/test').Route, why: string) => {
    audit.unknown.push(`${new URL(route.request().url()).pathname}: ${why}`)
    await route.fulfill({ status: 403, json: { code: 'SYNTHETIC_DENIED', message: '합성 fixture가 예상하지 않은 요청을 거부했습니다.' } })
  }
  await page.route('**/api/**', async route => {
    const request = route.request(), url = new URL(request.url()), path = url.pathname
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
    if (request.method() === 'GET') return route.fallback()
    if (request.method() !== 'POST' || (!mutations.has(path) && !auxPaths.has(path))) return route.fallback()
    const body = request.postData() ? request.postDataJSON() as Record<string, unknown> : null
    if (path === addPath) {
      if (!body || Object.keys(body).sort().join(',') !== 'account_no,app_key,app_secret,broker_id,is_paper_trading,live_trading_consent,name'
        || !['kis', 'toss'].includes(String(body.broker_id)) || !String(body.name).startsWith('합성')
        || !safeCredential(body.app_key) || !safeCredential(body.app_secret) || !safeAccount(body.account_no)
        || typeof body.is_paper_trading !== 'boolean' || body.live_trading_consent !== false) return deny(route, 'unexpected add payload')
      return run(route, 'mock-profile-write', () => {
        const added: AccountProfileView = { id: `kbd05-added-${tickets.length}`,  broker_id: body.broker_id as 'kis' | 'toss', broker_account_id: String(body.account_no),
          name: String(body.name), account_no: String(body.account_no), is_paper_trading: Boolean(body.is_paper_trading), live_trading_consent: false,
          app_key_masked: 'SYNTHETIC-***', is_active: false, is_configured: true }
        profiles.push(added); return added
      })
    }
    if (path === updatePath) {
      const profile = profiles.find(profile => profile.id === body?.id)
      if (!body || !profile || Object.keys(body).some(key => !['id', 'broker_id', 'name', 'is_paper_trading', 'live_trading_consent', 'app_key', 'app_secret', 'account_no'].includes(key))
        || body.broker_id !== profile.broker_id || !String(body.name).startsWith('합성') || body.live_trading_consent !== false
        || typeof body.is_paper_trading !== 'boolean' || (body.app_key !== undefined && !safeCredential(body.app_key))
        || (body.app_secret !== undefined && !safeCredential(body.app_secret)) || (body.account_no !== undefined && !safeAccount(body.account_no))) return deny(route, 'unexpected update payload')
      return run(route, 'mock-profile-write', () => {
        profile.name = String(body.name); profile.is_paper_trading = Boolean(body.is_paper_trading)
        if (body.account_no !== undefined) profile.account_no = String(body.account_no)
        if (profile.is_active) Object.assign(config, { active_profile_name: profile.name, kis_account_no: profile.account_no, kis_is_paper_trading: profile.is_paper_trading })
        return profile
      })
    }
    if (path === deletePath) {
      if (!body || Object.keys(body).join(',') !== 'id' || !fakeProfiles.some(profile => profile.id === body.id)) return deny(route, 'unexpected delete id')
      return run(route, 'mock-profile-write', () => {
        if (!options.retainDeletedProfiles) {
          const index = profiles.findIndex(profile => profile.id === body.id)
          if (index >= 0) profiles.splice(index, 1)
          if (config.active_profile_id === body.id) {
            const next = profiles.find(profile => profile.broker_id === 'kis')!
            Object.assign(config, { active_profile_id: next.id, active_profile_name: next.name, active_broker_account_id: next.broker_account_id, kis_account_no: next.account_no })
            for (const profile of profiles) profile.is_active = profile.id === next.id
          }
        }
        return null // The exact delete endpoint's existing void JSON response, not an unknown fallback.
      })
    }
    if (path === '/api/detect-trading-type') {
      if (!body || Object.keys(body).sort().join(',') !== 'appKey,appSecret' || !safeCredential(body.appKey) || !safeCredential(body.appSecret)) return deny(route, 'unexpected synthetic detect')
    } else if (path === newTossAccountsPath) {
      if (!body || Object.keys(body).sort().join(',') !== 'client_id,client_secret' || !safeCredential(body.client_id) || !safeCredential(body.client_secret)) return deny(route, 'unexpected new-key lookup')
    } else if (savedTossPaths.has(path) && body !== null) return deny(route, 'saved lookup must have no body')
    return run(route, 'aux-read-post', () => [])
  })
  const reads = (path: string) => tickets.filter(ticket => ticket.path === path)
  const release = (path: string) => { heldPaths.delete(path); for (const ticket of reads(path)) if (ticket.held && !ticket.released) ticket.complete() }
  const drain = async () => {
    heldPaths.clear()
    for (let pass = 0; pass < 12; pass++) {
      const snapshot = [...tickets]
      for (const ticket of snapshot) if (ticket.held && !ticket.released) ticket.complete()
      await Promise.all(snapshot.map(ticket => ticket.done)); await profileTick(page)
      if (snapshot.length === tickets.length && tickets.every(ticket => ticket.bodyFinished || ticket.browserError)) return
    }
    throw new Error('Synthetic profile requests did not quiesce')
  }
  return { audit, notes, tickets, webSockets, consoleMessages, reads, hold: (path: string) => heldPaths.add(path), release,
    reply: (path: string, status: number, json: unknown) => replies.set(path, { status, json }),
    clearReply: (path: string) => replies.delete(path), drain }
}
export type ProfileFixture = Awaited<ReturnType<typeof keyboardProfileFixture>>
export async function profileTick(page: Page, milliseconds = 1) { await page.clock.runFor(milliseconds); await page.evaluate(() => Promise.resolve()) }
export async function waitProfileTicket(state: ProfileFixture, path: string, ordinal = 1) {
  await expect.poll(() => state.reads(path).length).toBeGreaterThanOrEqual(ordinal)
  return state.reads(path)[ordinal - 1]
}
export async function finishProfileTicket(page: Page, ticket: ProfileTicket) {
  if (ticket.held && !ticket.released) ticket.complete()
  await ticket.done; expect(ticket.browserError).toBeUndefined(); expect(ticket.bodyFinished).toBe(true); await profileTick(page)
}
export function profileCard(page: Page, profile = fakeProfiles[0]) {
  const region = page.getByRole('region', { name: `${profile.broker_id === 'kis' ? '한국투자증권' : '토스증권'} 계좌 프로파일`, exact: true })
  return region.getByText(profile.name, { exact: true }).locator('xpath=ancestor::*[contains(@class,"MuiPaper-root")][1]')
}
export function addProfileTrigger(page: Page, broker: 'kis' | 'toss') {
  return page.getByRole('button', { name: `${broker === 'kis' ? '한국투자증권' : '토스증권'} 계좌 추가`, exact: true })
}
export async function openAddProfile(page: Page, broker: 'kis' | 'toss') {
  const trigger = addProfileTrigger(page, broker); await trigger.scrollIntoViewIfNeeded(); await trigger.focus(); await page.keyboard.press('Enter')
  const dialog = page.getByRole('dialog', { name: `${broker === 'kis' ? '한국투자증권' : '토스증권'} 계좌 프로파일 추가`, exact: true })
  await expect(dialog).toBeVisible(); return { dialog, trigger }
}
export async function openEditProfile(page: Page, profile = fakeProfiles[2]) {
  const trigger = profileCard(page, profile).getByRole('button', { name: '편집', exact: true }); await trigger.scrollIntoViewIfNeeded(); await trigger.focus(); await page.keyboard.press('Enter')
  const dialog = page.getByRole('dialog', { name: `프로파일 편집 — ${profile.name}`, exact: true })
  await expect(dialog).toBeVisible(); return { dialog, trigger }
}
export async function openDeleteProfile(page: Page, profile = fakeProfiles[1]) {
  const trigger = profileCard(page, profile).getByRole('button', { name: '삭제', exact: true }); await trigger.scrollIntoViewIfNeeded(); await trigger.focus(); await page.keyboard.press('Enter')
  const dialog = page.getByRole('dialog', { name: '프로파일 삭제', exact: true })
  await expect(dialog).toBeVisible(); await expect(dialog).toContainText(profile.name); return { dialog, trigger }
}
export async function ordinaryProfileClosePositive(page: Page, state: ProfileFixture, open: () => Promise<{ dialog: Locator; trigger: Locator }>, withInput = true) {
  for (const mode of ['Escape', 'Cancel']) {
    const { dialog, trigger } = await open()
    if (withInput) { await dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill('합성 정상닫기 초안'); await page.keyboard.press('Enter'); await profileTick(page); expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0) }
    if (mode === 'Escape') await page.keyboard.press('Escape')
    else await dialog.getByRole('button', { name: '취소', exact: true }).click()
    await expect(dialog).toHaveCount(0); await expect(trigger).toBeFocused()
  }
  state.notes.push({ stage: 'ordinary-Cancel-Escape-focus-positive', inputEnterNeverSubmits: withInput, mockProfileWrites: 0 })
}
export async function captureProfiles(page: Page, state: ProfileFixture, title: string, status: string | undefined) {
  await state.drain()
  const directory = '.cache/kbd05-implementation-20261031/evidence'; await mkdir(directory, { recursive: true })
  const name = `${title.replace(/[^a-zA-Z0-9-]/g, '_').slice(0, 100)}-${createHash('sha256').update(title, 'utf8').digest('hex').slice(0, 12)}`
  await writeFile(`${directory}/${name}.json`, JSON.stringify({ title, status, notes: state.notes,
    requests: state.tickets.map(({ done: _done, complete: _complete, ...ticket }) => ticket),
    mockProfileWrites: state.tickets.filter(ticket => ticket.kind === 'mock-profile-write').length,
    auxReadPosts: state.tickets.filter(ticket => ticket.kind === 'aux-read-post').length,
    actualProfileAccountBrokerWrites: 0, deniedWrites: state.audit.writes, external: state.audit.external, unknown: state.audit.unknown,
    pageErrors: state.audit.errors, webSockets: state.webSockets, consoleMessages: state.consoleMessages,
    intentionalHttp500: state.tickets.filter(ticket => ticket.status === 500).map(ticket => ({ id: ticket.id, path: ticket.path, kind: ticket.kind })) }, null, 2))
  await page.screenshot({ path: `${directory}/${name}.png`, animations: 'disabled' })
}
export function safeProfiles(state: ProfileFixture) {
  expect(state.audit.writes).toEqual([]); expect(state.audit.external).toEqual([]); expect(state.audit.unknown).toEqual([]); expect(state.audit.errors).toEqual([]); expect(state.webSockets).toEqual([])
  expect(state.tickets.flatMap(ticket => ticket.browserError ? [ticket.browserError] : [])).toEqual([])
}

export type ProfileView = Awaited<ReturnType<typeof openAddProfile>>
export async function fillSyntheticAdd(view: ProfileView, broker: 'kis' | 'toss', name: string) {
  await view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill(name)
  await view.dialog.getByLabel(broker === 'kis' ? 'APP KEY' : 'Client ID', { exact: true }).fill(`SYNTHETIC-${broker.toUpperCase()}-KEY`)
  await view.dialog.getByLabel(broker === 'kis' ? 'APP SECRET' : 'Client Secret', { exact: true }).fill(`SYNTHETIC-${broker.toUpperCase()}-SECRET`)
  await view.dialog.getByLabel(broker === 'kis' ? '계좌번호 (10자리 입력, 예: 12345678-01)' : 'accountSeq', { exact: true }).fill(broker === 'kis' ? '00000002-01' : '7103')
}
export function expectedAddBody(broker: 'kis' | 'toss', name: string) {
  return { broker_id: broker, name, is_paper_trading: false, live_trading_consent: false,
    app_key: `SYNTHETIC-${broker.toUpperCase()}-KEY`, app_secret: `SYNTHETIC-${broker.toUpperCase()}-SECRET`, account_no: broker === 'kis' ? '00000002-01' : '7103' }
}
export function expectedEditBody(profile: AccountProfileView, name: string) {
  return { id: profile.id, broker_id: profile.broker_id, name, is_paper_trading: profile.is_paper_trading,
    live_trading_consent: profile.live_trading_consent, account_no: profile.account_no }
}
export async function clickProfileBackdrop(page: Page, dialog: Locator, notes: unknown[], phase: string) {
  const viewport = page.viewportSize()
  const container = dialog.locator('..')
  await expect(container).toHaveClass(/MuiDialog-container/); await expect(container).toBeVisible()
  const handle = await container.elementHandle(), paper = await dialog.elementHandle()
  if (!handle || !paper) throw new Error('No current Dialog container/Paper for native backdrop hit')
  const inspect = () => handle.evaluate((element, paper) => {
    const box = element.getBoundingClientRect(), bounds = paper.getBoundingClientRect()
    const within = (value: number, min: number, max: number) => Math.max(min, Math.min(max, value))
    const centerX = within(bounds.x + bounds.width / 2, box.x + 1, box.right - 1)
    const centerY = within(bounds.y + bounds.height / 2, box.y + 1, box.bottom - 1)
    const candidates = [
      { x: (box.x + bounds.x) / 2, y: centerY }, { x: (bounds.right + box.right) / 2, y: centerY },
      { x: centerX, y: (box.y + bounds.y) / 2 }, { x: centerX, y: (bounds.bottom + box.bottom) / 2 },
    ].filter(point => point.x > box.x && point.x < box.right && point.y > box.y && point.y < box.bottom
      && point.x >= 0 && point.x < innerWidth && point.y >= 0 && point.y < innerHeight
      && (point.x < bounds.x || point.x >= bounds.right || point.y < bounds.y || point.y >= bounds.bottom))
      .map(point => { const hit = document.elementFromPoint(point.x, point.y); return { ...point,
        hitIsContainer: hit === element, hitTag: hit?.tagName ?? null, hitClass: hit ? String(hit.className) : null } })
    return { container: { x: box.x, y: box.y, width: box.width, height: box.height },
      paper: { x: bounds.x, y: bounds.y, width: bounds.width, height: bounds.height }, candidates }
  }, paper)
  let geometry = await inspect()
  // Synchronize to a real current hit, not a platform-fixed coordinate or an assumed fade duration.
  await expect.poll(async () => { geometry = await inspect(); return geometry.candidates.some(point => point.hitIsContainer) }).toBe(true)
  const point = geometry.candidates.find(point => point.hitIsContainer)
  if (!point) throw new Error('No backdrop point hits the current container itself')
  await handle.evaluate(element => {
    const events: Array<{ type: string; trusted: boolean; targetIsContainer: boolean; x: number; y: number; targetTag: string | null }> = []
    const listener = (event: Event) => { const mouse = event as MouseEvent; events.push({ type: event.type, trusted: event.isTrusted,
      targetIsContainer: event.target === element, x: mouse.clientX, y: mouse.clientY, targetTag: (event.target as Element | null)?.tagName ?? null }) }
    Reflect.set(element, '__kbd05BackdropEvents', events); Reflect.set(element, '__kbd05BackdropListener', listener)
    element.addEventListener('mousedown', listener, { once: true, passive: true, capture: true })
    element.addEventListener('click', listener, { once: true, passive: true, capture: true })
  })
  let observed: Array<{ type: string; trusted: boolean; targetIsContainer: boolean; x: number; y: number; targetTag: string | null }> = []
  try {
    // Locator actionability checks include the actual point receiving events; no force or synthetic click.
    await container.click({ position: { x: point.x - geometry.container.x, y: point.y - geometry.container.y } })
  } finally {
    observed = await handle.evaluate(element => {
      const events = Reflect.get(element, '__kbd05BackdropEvents') as Array<{ type: string; trusted: boolean; targetIsContainer: boolean; x: number; y: number; targetTag: string | null }>
      const listener = Reflect.get(element, '__kbd05BackdropListener') as (event: Event) => void
      element.removeEventListener('mousedown', listener, true); element.removeEventListener('click', listener, true)
      Reflect.deleteProperty(element, '__kbd05BackdropEvents'); Reflect.deleteProperty(element, '__kbd05BackdropListener')
      return events
    })
    notes.push({ stage: 'actual-container-native-backdrop', phase, viewport, geometry, point, observed })
  }
  expect(observed.map(event => event.type)).toEqual(['mousedown', 'click'])
  for (const event of observed) {
    expect(event.trusted).toBe(true); expect(event.targetIsContainer).toBe(true)
    expect(Math.abs(event.x - point.x)).toBeLessThanOrEqual(1); expect(Math.abs(event.y - point.y)).toBeLessThanOrEqual(1)
  }
}
export async function attemptMainClose(page: Page, view: ProfileView, state: ProfileFixture, focusTarget?: Locator) {
  const cancel = view.dialog.getByRole('button', { name: '취소', exact: true })
  await expect(cancel).toBeDisabled()
  const bounds = await cancel.boundingBox(); if (!bounds) throw new Error('No pending Cancel bounds')
  await page.mouse.click(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2)
  await expect(view.dialog).toBeVisible()
  const target = focusTarget ?? view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })
  await target.focus(); await expect(target).toBeFocused(); await page.keyboard.press('Escape')
  await expect(view.dialog).toBeVisible()
  await clickProfileBackdrop(page, view.dialog, state.notes, 'main-pending'); await expect(view.dialog).toBeVisible()
}
export async function currentError(view: ProfileView, text: string) {
  const alert = view.dialog.getByRole('alert').filter({ hasText: text })
  await expect(alert).toHaveCount(1); await expect(alert).toBeVisible()
  return alert
}
export async function startKisDetection(view: ProfileView, mode: 'Add' | 'Edit', tag: string) {
  const suffix = mode === 'Edit' ? ' (변경 시에만 입력)' : ''
  await view.dialog.getByLabel(`APP KEY${suffix}`, { exact: true }).fill(`SYNTHETIC-${tag}-KEY`)
  await view.dialog.getByLabel(`APP SECRET${suffix}`, { exact: true }).fill(`SYNTHETIC-${tag}-SECRET`)
  await view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).focus() // The original secret blur triggers the read once.
}
