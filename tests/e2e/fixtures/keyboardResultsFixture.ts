import { expect, type Page, type Locator } from '@playwright/test'
import { mkdir, writeFile } from 'node:fs/promises'
import { mockUx } from './uxReviewFixture'

export type ResultsSurface = 'Trading' | 'Strategy' | 'LTH'
export const resultsTerm = '합성결과탐색용'
export const resultItems = [
  { pdno: '0005A0', prdt_name: '합성 목록 첫 종목' },
  { pdno: '0005A1', prdt_name: '합성 목록 둘째 종목' },
]

export type ResultItem = typeof resultItems[number]
export interface ResultsReply { items: ResultItem[]; status?: number; hold?: boolean; errorCode?: string }
export interface ResultsRead { id: number; term: string; status: number; held: boolean; released: boolean; bodyFinished: boolean; error?: string; done: Promise<void>; complete: () => void }

export async function keyboardResultsFixture(page: Page, broker: 'kis' | 'toss' = 'kis', options: { items?: ResultItem[]; controlledStrategy?: boolean } = {}) {
  await page.clock.install()
  const audit = await mockUx(page, { activeBroker: broker, appConfigOverride: { kis_is_paper_trading: broker === 'kis' } })
  const notes: unknown[] = [], readPosts: string[] = []
  const items = options.items ?? resultItems
  const reads: ResultsRead[] = [], replies = new Map<string, ResultsReply>()
  const strategy = { enabled: false }
  if (options.controlledStrategy) audit.responses.set('/api/strategies', async route => { await route.fulfill({ json: [{
    id: 'leveraged_trend_hold_default', name: 'LeveragedTrendHoldStrategy', enabled: strategy.enabled,
    brokerId: broker, brokerAccountId: broker === 'kis' ? '12345678-01' : '1', targetSymbols: ['SOXL'],
    targetSymbolNames: { SOXL: 'Synthetic existing target' }, orderQuantity: 1,
    params: { entries: [{ leveraged_symbol: 'SOXL', leveraged_symbol_name: 'Synthetic existing target', inverse_leveraged_symbol: '',
      inverse_leveraged_symbol_name: '', base_symbols: [], base_symbol_names: {}, base_symbol_roles: {}, quantity: 1, inverse_quantity: 1, is_overseas: true }] },
  }] }) })
  const setSearch = (term: string, reply: ResultsReply) => {
    replies.set(term, reply)
    const path = `/api/search/${encodeURIComponent(term)}`
    audit.responses.set(path, async route => {
      const request = route.request(), url = new URL(request.url())
      if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
      if (url.pathname !== path || request.method() !== 'GET') return route.abort()
      const current = { ...replies.get(term)! }
      let release!: () => void
      const gate = new Promise<void>(resolve => { release = resolve })
      const read: ResultsRead = { id: reads.length + 1, term, status: current.status ?? 200, held: current.hold === true,
        released: false, bodyFinished: false, done: Promise.resolve(), complete: () => { read.released = true; release() } }
      reads.push(read)
      read.done = page.waitForResponse(response => response.headers()['x-kbd03-ticket'] === String(read.id)).then(async response => {
        const failure = await response.finished(); if (failure) throw failure
        expect(response.status()).toBe(read.status); read.bodyFinished = true
      }).catch(error => { read.error = String(error) })
      if (read.held) await gate
      await route.fulfill({ status: read.status, json: read.status < 400 ? current.items : { code: current.errorCode ?? 'SYNTHETIC_RESULTS_ERROR', message: 'Synthetic results unavailable' },
        headers: { 'x-kbd03-ticket': String(read.id) } })
    })
  }
  for (const term of [resultsTerm, ...items.flatMap(item => [item.pdno, item.prdt_name])]) setSearch(term, { items })
  for (const item of [...resultItems, ...items]) {
    audit.responses.set(`/api/price/${item.pdno}`, { json: { stck_prpr: '10000', hts_kor_isnm: item.prdt_name, prdy_vrss: '0', prdy_ctrt: '0', acml_vol: '10' } })
    audit.responses.set(`/api/chart/${item.pdno}`, { json: [] })
    audit.responses.set(`/api/toss-chart/${item.pdno}`, { json: [] })
    audit.responses.set(`/api/toss-stock-safety/${item.pdno}`, { json: { brokerId: 'toss', symbol: item.pdno, stockInfo: null,
      warnings: [], hasBlockingWarning: false, buyBlocked: false, buyBlockReason: null } })
    audit.responses.set(`/api/toss-market-snapshot/${item.pdno}`, { json: { brokerId: 'toss', market: 'kr', price: { amount: '10000', currency: 'KRW' },
      orderbook: { asks: [], bids: [], currency: 'KRW' }, trades: [], priceLimits: { upperLimitPrice: null, lowerLimitPrice: null, currency: 'KRW' } } })
  }
  if (broker === 'toss') await page.route('**/api/toss-open-orders', async route => {
    const request = route.request(), url = new URL(request.url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
    if (url.pathname !== '/api/toss-open-orders' || request.method() !== 'POST') return route.fallback()
    readPosts.push(url.pathname)
    return route.fulfill({ json: [] }) // Existing exact semantic read; no preflight/order/write exception.
  })
  return { audit, notes, readPosts, broker, items, reads, strategy, setSearch,
    drain: async () => {
      for (let pass = 0; pass < 8; pass++) {
        const snapshot = [...reads]
        for (const read of snapshot) if (read.held && !read.released) read.complete()
        await Promise.all(snapshot.map(read => read.done))
        await page.clock.runFor(1); await page.evaluate(() => Promise.resolve())
        if (reads.length === snapshot.length && reads.every(read => read.bodyFinished || read.error)) return
      }
      throw new Error('Synthetic result reads did not quiesce')
    },
  }
}
export type ResultsFixture = Awaited<ReturnType<typeof keyboardResultsFixture>>

export async function openResultsSurface(page: Page, surface: ResultsSurface) {
  await page.goto(surface === 'Trading' ? '/trading' : '/strategy')
  const area = surface === 'LTH' ? page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' }) : page.locator('body')
  if (surface === 'LTH') {
    // Preserve the real US default and explicitly select KR before testing domestic results.
    await expect(area.getByRole('textbox', { name: 'ETF 티커', exact: true })).toBeVisible()
    await area.getByRole('button', { name: '국내', exact: true }).click()
  }
  const input = area.getByRole('textbox', { name: surface === 'LTH' ? 'ETF 코드 또는 이름' : '6자리 종목코드 (예: 005930, 0005A0)', exact: true })
  const search = area.getByRole('button', { name: surface === 'LTH' ? '레버리지 대상 검색' : '국내 종목 검색', exact: true })
  await expect(input).toBeEnabled()
  return { page, input, search, area, surface }
}
export type ResultsView = Awaited<ReturnType<typeof openResultsSurface>>
export const resultButton = (view: ResultsView, item = resultItems[0]) => view.area.getByRole('button', { name: `${item.prdt_name} (${item.pdno}) 선택`, exact: true })
export const resultRow = (view: ResultsView, item = resultItems[0]) => view.area.getByRole('row').filter({ has: view.page.getByRole('cell', { name: item.pdno, exact: true }) })
export async function showResults(view: ResultsView, term = resultsTerm, items = resultItems) {
  await view.input.fill(term)
  await view.input.press('Enter')
  for (const item of items) await expect(resultRow(view, item)).toBeVisible()
  await expect(view.search).toHaveAttribute('aria-busy', 'false')
}
export async function expectResultsSelection(page: Page, view: ResultsView, item = resultItems[0]) {
  await expect(view.input).toHaveValue(item.prdt_name)
  if (view.surface === 'Trading') await expect(page.getByText(`선택됨: ${item.pdno}`, { exact: true })).toBeVisible()
  else if (view.surface === 'Strategy') await expect(page.getByText(`선택됨: ${item.prdt_name} (${item.pdno})`, { exact: true })).toBeVisible()
  else await expect(view.area.getByText(`${item.prdt_name} (${item.pdno})`, { exact: true })).toBeVisible()
}
export async function pointerPositive(page: Page, state: ResultsFixture, surface: ResultsSurface) {
  const view = await openResultsSurface(page, surface)
  await showResults(view)
  await resultRow(view).getByRole('cell', { name: resultItems[0].pdno, exact: true }).click()
  await expectResultsSelection(page, view)
  state.notes.push({ stage: 'existing-code-cell-pointer-positive-before-new-key-contract', surface, symbol: resultItems[0].pdno,
    searchGets: state.audit.counts.get(`/api/search/${encodeURIComponent(resultsTerm)}`) ?? 0 })
  // A fresh route load resets the previous selection before exercising the new key contract.
  return openResultsSurface(page, surface)
}
export function safeResults(state: ResultsFixture) {
  expect(state.audit.writes).toEqual([]); expect(state.audit.external).toEqual([])
  expect(state.audit.unknown).toEqual([]); expect(state.audit.errors).toEqual([])
  expect(state.reads.flatMap(read => read.error ? [read.error] : [])).toEqual([])
}
export async function captureResults(page: Page, state: ResultsFixture, title: string, status: string | undefined) {
  await state.drain()
  const directory = '.cache/kbd03-implementation-20261031/evidence'
  await mkdir(directory, { recursive: true })
  const name = title.replace(/[^a-zA-Z0-9-]/g, '_')
  await writeFile(`${directory}/${name}.json`, JSON.stringify({ status, broker: state.broker, notes: state.notes,
    reads: state.reads.map(({ done: _done, complete: _complete, ...read }) => read), counts: Object.fromEntries(state.audit.counts), readPosts: state.readPosts, writes: state.audit.writes,
    external: state.audit.external, unknown: state.audit.unknown, pageErrors: state.audit.errors }, null, 2))
  await page.screenshot({ path: `${directory}/${name}.png`, animations: 'disabled' })
}


export async function waitResultsRead(state: ResultsFixture, term: string, ordinal = 1) {
  await expect.poll(() => state.reads.filter(read => read.term === term).length).toBeGreaterThanOrEqual(ordinal)
  return state.reads.filter(read => read.term === term)[ordinal - 1]
}
export async function settleResultsRead(page: Page, read: ResultsRead) {
  if (read.held) read.complete()
  await read.done
  expect(read.error).toBeUndefined(); expect(read.bodyFinished).toBe(true)
  await page.clock.runFor(1); await page.evaluate(() => Promise.resolve())
}
export async function focusFirstResult(view: ResultsView, item = resultItems[0]) {
  await view.input.focus(); await view.input.press('Tab'); await expect(view.search).toBeFocused()
  await view.page.keyboard.press('Tab'); await expect(resultButton(view, item)).toBeFocused()
}
export async function resultsOutline(button: Locator) {
  await expect(button).toBeFocused()
  expect(await button.evaluate(element => ({ style: getComputedStyle(element).outlineStyle, width: getComputedStyle(element).outlineWidth })))
    .toEqual({ style: 'solid', width: '2px' })
}
export async function nativeActivationCounter(button: Locator) {
  const handle = await button.elementHandle()
  if (!handle) throw new Error('No native result control for activation counter')
  await handle.evaluate(element => {
    Reflect.set(element, '__kbd03Clicks', 0)
    element.addEventListener('click', () => Reflect.set(element, '__kbd03Clicks', Number(Reflect.get(element, '__kbd03Clicks')) + 1), { capture: true })
  })
  return () => handle.evaluate(element => Number(Reflect.get(element, '__kbd03Clicks')))
}
export function outsideResults(view: ResultsView) {
  return view.surface === 'Trading' ? view.page.getByRole('textbox', { name: '주문수량', exact: true })
    : view.area.getByRole('spinbutton', { name: '1회 수량', exact: true }).first()
}
export async function assertNoResultsSelection(view: ResultsView, item = resultItems[0]) {
  if (view.surface === 'Trading') await expect(view.page.getByText(`선택됨: ${item.pdno}`, { exact: true })).toHaveCount(0)
  else if (view.surface === 'Strategy') await expect(view.page.getByText(`선택됨: ${item.prdt_name} (${item.pdno})`, { exact: true })).toHaveCount(0)
  else await expect(view.area.getByText(`${item.prdt_name} (${item.pdno})`, { exact: true })).toHaveCount(0)
}

// Source-only bridge following 02's existing pattern; all evidence remains in the 03 directory.
export async function installResultsSourceHarness(page: Page, state: ResultsFixture) {
  let intercepted = false
  await page.route('**/src/main.tsx*', async route => {
    const request = route.request(), url = new URL(request.url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { state.audit.external.push(url.origin); return route.abort() }
    if (url.pathname !== '/src/main.tsx' || request.method() !== 'GET') return route.fallback()
    const response = await route.fetch(), original = await response.text()
    if (!original.includes('queryClient') || !original.includes('RouterProvider')) throw new Error('Original main QueryClient/router source no longer matches')
    intercepted = true
    await mkdir('.cache/kbd03-implementation-20261031', { recursive: true })
    await writeFile('.cache/kbd03-implementation-20261031/source-entry-transformed.txt', original)
    await route.fulfill({ response, body: original + `\nwindow.__kbd03Results = { identity: Math.random().toString(36), refresh: async (kind, term) => { await queryClient.invalidateQueries({ queryKey: kind === 'strategies' ? ['strategies'] : ['stockSearch', term] }); } };\n` })
  })
  return { intercepted: () => intercepted,
    identity: () => page.evaluate(() => (Reflect.get(window, '__kbd03Results') as { identity: string }).identity),
    refresh: async (kind: 'strategies' | 'search', term = resultsTerm) => {
      await page.evaluate(({ kind, term }) => (Reflect.get(window, '__kbd03Results') as { refresh: (kind: string, term: string) => Promise<void> }).refresh(kind, term), { kind, term })
      await page.clock.runFor(1); await page.evaluate(() => Promise.resolve())
    },
  }
}


export async function observeNativeCaret(input: Locator, key: string) {
  const before = await input.evaluate(element => {
    const field = element as HTMLInputElement
    Reflect.set(field, '__kbd03CaretEvent', null)
    field.addEventListener('keydown', event => {
      // Passive observation only: retain the native object; inspect it after Keyboard.press completes.
      Reflect.set(field, '__kbd03CaretEvent', event)
    }, { once: true, passive: true, capture: true })
    const scrollContext = []
    for (let node: HTMLElement | null = field; node; node = node.parentElement) scrollContext.push({ tag: node.tagName, id: node.id,
      className: node.className, top: node.scrollTop, left: node.scrollLeft, scrollHeight: node.scrollHeight,
      scrollWidth: node.scrollWidth, clientHeight: node.clientHeight, clientWidth: node.clientWidth })
    return { scrollContext, documentScroll: { x: scrollX, y: scrollY }, value: field.value, start: field.selectionStart, end: field.selectionEnd, direction: field.selectionDirection,
      focused: document.activeElement === field, documentFocused: document.hasFocus() }
  })
  // Keyboard.press preserves the actual current focus; Locator.press would focus the input implicitly.
  await input.page().keyboard.press(key)
  await input.page().evaluate(() => Promise.resolve())
  const after = await input.evaluate(element => {
    const field = element as HTMLInputElement
    const event = Reflect.get(field, '__kbd03CaretEvent') as KeyboardEvent | null
    const scrollContext = []
    for (let node: HTMLElement | null = field; node; node = node.parentElement) scrollContext.push({ tag: node.tagName, id: node.id,
      className: node.className, top: node.scrollTop, left: node.scrollLeft, scrollHeight: node.scrollHeight,
      scrollWidth: node.scrollWidth, clientHeight: node.clientHeight, clientWidth: node.clientWidth })
    return { scrollContext, documentScroll: { x: scrollX, y: scrollY }, value: field.value, start: field.selectionStart, end: field.selectionEnd, direction: field.selectionDirection,
      focused: document.activeElement === field, documentFocused: document.hasFocus(),
      event: event ? { key: event.key, code: event.code, keyCode: event.keyCode, defaultPrevented: event.defaultPrevented,
        isTrusted: event.isTrusted, isComposing: event.isComposing, altKey: event.altKey, ctrlKey: event.ctrlKey,
        metaKey: event.metaKey, shiftKey: event.shiftKey } : null }
  })
  return { key, before, after }
}
export type NativeCaretObservation = Awaited<ReturnType<typeof observeNativeCaret>>
export async function compareSameContextCaret(input: Locator, observations: NativeCaretObservation[], artifact: string) {
  const page = input.page(), id = `kbd03-context-neutral-${artifact}`
  const inspect = () => input.evaluate(fieldNode => {
    const field = fieldNode as HTMLInputElement, style = getComputedStyle(field), rect = field.getBoundingClientRect()
    const scrollContext = []
    for (let node: HTMLElement | null = field; node; node = node.parentElement) scrollContext.push({ tag: node.tagName, id: node.id,
      top: node.scrollTop, left: node.scrollLeft, scrollHeight: node.scrollHeight, scrollWidth: node.scrollWidth,
      clientHeight: node.clientHeight, clientWidth: node.clientWidth })
    return { value: field.value, start: field.selectionStart, end: field.selectionEnd, direction: field.selectionDirection,
      focused: document.activeElement === field, type: field.type, font: style.font, textDirection: style.direction,
      rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height }, scrollContext }
  })
  const originalBefore = await inspect()
  const insertion = await input.evaluate((fieldNode, id) => {
    const field = fieldNode as HTMLInputElement, parent = field.parentElement
    if (!parent || document.getElementById(id)) throw new Error('No unique same-parent neutral target')
    const style = getComputedStyle(field), originalRect = field.getBoundingClientRect()
    const clone = field.cloneNode(false) as HTMLInputElement
    for (const attribute of [...clone.attributes]) if (attribute.name.startsWith('on') || ['id', 'name', 'aria-label', 'aria-labelledby', 'aria-describedby', 'aria-controls', 'form', 'autofocus'].includes(attribute.name)) clone.removeAttribute(attribute.name)
    clone.id = id; clone.setAttribute('aria-label', '같은 문맥 중립 native 입력'); clone.tabIndex = -1
    // Same CSS classes/type/font/direction/scroll ancestors, removed from layout flow.
    Object.assign(clone.style, { position: 'absolute', left: `${field.offsetLeft}px`, top: `${field.offsetTop}px`,
      width: style.width, height: style.height, margin: style.margin, font: style.font, direction: style.direction,
      letterSpacing: style.letterSpacing, textAlign: style.textAlign, opacity: '0', pointerEvents: 'none' })
    const focusouts: Array<{ target: string; relatedTarget: string | null; staysInsideParent: boolean }> = []
    const listener = (event: FocusEvent) => focusouts.push({ target: (event.target as Element | null)?.id ?? '',
      relatedTarget: (event.relatedTarget as Element | null)?.id ?? null,
      staysInsideParent: event.relatedTarget instanceof Node && parent.contains(event.relatedTarget) })
    Reflect.set(parent, '__kbd03ContextFocusouts', focusouts); Reflect.set(parent, '__kbd03ContextFocusoutListener', listener)
    parent.addEventListener('focusout', listener, { passive: true, capture: true })
    parent.append(clone)
    const after = field.getBoundingClientRect(), cloneRect = clone.getBoundingClientRect(), cloneStyle = getComputedStyle(clone)
    return { sameParent: clone.parentElement === parent, cloneOwnReactKeys: Object.keys(clone).filter(key => key.startsWith('__react')).length,
      originalBefore: { x: originalRect.x, y: originalRect.y, width: originalRect.width, height: originalRect.height },
      originalAfter: { x: after.x, y: after.y, width: after.width, height: after.height },
      cloneRect: { x: cloneRect.x, y: cloneRect.y, width: cloneRect.width, height: cloneRect.height },
      originalType: field.type, cloneType: clone.type, originalFont: style.font, cloneFont: cloneStyle.font,
      originalDirection: style.direction, cloneDirection: cloneStyle.direction }
  }, id)
  const clone = page.getByRole('textbox', { name: '같은 문맥 중립 native 입력', exact: true })
  const originalAfterInsertion = await inspect()
  const pairs: Array<{ app: NativeCaretObservation; neutral: NativeCaretObservation }> = []
  try {
    for (const app of observations) {
      await clone.evaluate((element, before) => {
        const neutral = element as HTMLInputElement
        neutral.value = before.value
        neutral.focus({ preventScroll: true })
        if (before.start === null || before.end === null) throw new Error('Observed app text input has no selection')
        neutral.setSelectionRange(before.start, before.end, before.direction ?? 'none')
        // Reproduce only the neutral control's starting native scroll context, after the app measurements ended.
        let node: HTMLElement | null = neutral
        for (const expected of before.scrollContext) {
          if (!node) throw new Error('Same-parent neutral ancestor chain diverged')
          node.scrollTop = expected.top; node.scrollLeft = expected.left; node = node.parentElement
        }
        window.scrollTo(before.documentScroll.x, before.documentScroll.y)
      }, app.before)
      pairs.push({ app, neutral: await observeNativeCaret(clone, app.key) })
    }
  } finally {
    // Restore scroll and focus without ever assigning the original value/caret.
    await input.evaluate((fieldNode, before) => {
      const field = fieldNode as HTMLInputElement
      field.focus({ preventScroll: true })
      let node: HTMLElement | null = field
      for (const saved of before.scrollContext) {
        if (!node) break
        node.scrollTop = saved.top; node.scrollLeft = saved.left; node = node.parentElement
      }
    }, originalBefore)
  }
  const cleanup = await input.evaluate((fieldNode, id) => {
    const field = fieldNode as HTMLInputElement, parent = field.parentElement!
    const focusouts = Reflect.get(parent, '__kbd03ContextFocusouts') as Array<{ target: string; relatedTarget: string | null; staysInsideParent: boolean }>
    const listener = Reflect.get(parent, '__kbd03ContextFocusoutListener') as (event: FocusEvent) => void
    parent.removeEventListener('focusout', listener, true)
    document.getElementById(id)?.remove()
    Reflect.deleteProperty(parent, '__kbd03ContextFocusouts'); Reflect.deleteProperty(parent, '__kbd03ContextFocusoutListener')
    return { focusouts, neutralNodesRemaining: document.querySelectorAll(`[id="${id}"]`).length }
  }, id)
  const evidence = { kind: 'same-context native comparison: strict input arrows and non-interference for native Home/End',
    scopeLimits: { resultButtonHomeEnd: 'strict focus/clamp in the app test', inputArrows: 'strict native value/caret/direction/focus',
      inputHomeEnd: 'same starting selection; strict value/focus/native key/trusted/defaultPrevented; post-command numeric caret and animated scroll are observations only',
      nativeScrollFrameCoordinates: 'observed, not asserted equal across sequential native control runs' },
    platformMetadata: await page.evaluate(() => ({ platform: navigator.platform, userAgent: navigator.userAgent,
      language: navigator.language, webdriver: navigator.webdriver, width: innerWidth, height: innerHeight, dpr: devicePixelRatio })),
    originalCaretWasNeverSet: true, neutralOwnBehaviorHandlers: 0, sharedReactAncestorsPreserved: true,
    originalBefore, originalAfterInsertion, originalAfter: await inspect(), insertion, cleanup, pairs }
  await writeFile(`.cache/kbd03-implementation-20261031/caret-context-${artifact}.json`, JSON.stringify(evidence, null, 2))
  return evidence
}
