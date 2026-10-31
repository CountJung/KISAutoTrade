import { expect, test, type Page } from '@playwright/test'
import { keyboardResultsFixture, pointerPositive, showResults, resultItems, resultButton, resultRow,
  expectResultsSelection, safeResults, captureResults, resultsTerm, openResultsSurface, focusFirstResult, resultsOutline,
  nativeActivationCounter, outsideResults, assertNoResultsSelection, waitResultsRead, settleResultsRead, observeNativeCaret, compareSameContextCaret,
  type NativeCaretObservation, type ResultsFixture, type ResultItem } from './fixtures/keyboardResultsFixture'

const states = new WeakMap<Page, ResultsFixture>()
test.afterEach(async ({ page }, info) => { const state = states.get(page); if (state) try { await captureResults(page, state, info.title, info.status) } finally { safeResults(state) } })
async function setup(page: Page, broker: 'kis' | 'toss' = 'kis', options: { items?: ResultItem[]; narrow?: boolean } = {}) {
  const narrow = options.narrow ?? broker === 'toss'
  await page.setViewportSize(narrow ? { width: 375, height: 812 } : { width: 1280, height: 900 })
  await page.emulateMedia({ colorScheme: narrow ? 'dark' : 'light' })
  const state = await keyboardResultsFixture(page, broker, options); states.set(page, state)
  return state
}

test('P2-KBD-03 minimum KIS LTH native named result buttons keep Tab order and select with Enter', async ({ page }) => {
  const state = await setup(page), view = await pointerPositive(page, state, 'LTH')
  await showResults(view)
  const first = resultButton(view), second = resultButton(view, resultItems[1])
  // This fails for the product's missing native control, after the original pointer path has passed.
  await expect(first).toBeVisible(); await expect(second).toBeVisible()
  expect(await first.evaluate(element => element.tagName)).toBe('BUTTON')
  await view.input.focus(); await view.input.press('Tab'); await expect(view.search).toBeFocused()
  await page.keyboard.press('Tab'); await expect(first).toBeFocused()
  await page.clock.runFor(250); await expect(first).toBeFocused()
  await page.keyboard.press('Tab'); await expect(second).toBeFocused()
  await page.keyboard.press('Shift+Tab'); await expect(first).toBeFocused()
  await page.keyboard.press('Enter')
  await expectResultsSelection(page, view)
  await expect(view.input).toBeFocused()
  await expect(resultRow(view)).toHaveCount(0)
})

test('P2-KBD-03 minimum KIS LTH Escape closes domestic results and returns input focus without reopening', async ({ page }) => {
  const state = await setup(page), view = await pointerPositive(page, state, 'LTH')
  await showResults(view)
  await view.input.focus(); await view.input.press('Escape')
  await expect(view.input).toBeFocused()
  await expect(resultRow(view)).toHaveCount(0)
  await page.clock.runFor(600)
  await expect(resultRow(view)).toHaveCount(0)
  await expect(view.input).toBeFocused()
})

for (const surface of ['Trading', 'Strategy'] as const) test(`P2-KBD-03 minimum KIS ${surface} result ArrowDown moves focus from first to second without selection`, async ({ page }) => {
  const state = await setup(page), view = await pointerPositive(page, state, surface)
  await showResults(view)
  const first = resultButton(view), second = resultButton(view, resultItems[1])
  await expect(first).toBeVisible(); await expect(second).toBeVisible()
  await view.input.focus(); await view.input.press('Tab'); await expect(view.search).toBeFocused()
  await page.keyboard.press('Tab'); await expect(first).toBeFocused()
  await page.clock.runFor(250); await expect(first).toBeFocused()
  const before = state.audit.counts.get(`/api/search/${encodeURIComponent('합성결과탐색용')}`) ?? 0
  await page.keyboard.press('ArrowDown')
  await expect(second).toBeFocused()
  await expect(view.input).toHaveValue('합성결과탐색용')
  await expect(first).toBeVisible(); await expect(second).toBeVisible()
  expect(state.audit.counts.get(`/api/search/${encodeURIComponent('합성결과탐색용')}`) ?? 0).toBe(before)
})


for (const broker of ['kis', 'toss'] as const) for (const surface of ['Trading', 'Strategy', 'LTH'] as const) {
  test(`P2-KBD-03 ${broker.toUpperCase()} ${surface} Enter Space native activation preserves Tab ShiftTab focus and closes once`, async ({ page }) => {
    const state = await setup(page, broker)
    for (const key of ['Enter', 'Space']) {
      const view = await openResultsSurface(page, surface)
      if (surface !== 'Trading') await expect(page.getByRole('spinbutton', { name: '1회 수량', exact: true }).first()).toHaveValue('1')
      await showResults(view)
      const numbers = await page.locator('input[type="number"]').evaluateAll(elements => elements.map(element => (element as HTMLInputElement).value))
      await focusFirstResult(view); await resultsOutline(resultButton(view))
      await page.clock.runFor(250); await expect(resultButton(view)).toBeFocused()
      await page.keyboard.press('Tab'); await expect(resultButton(view, resultItems[1])).toBeFocused()
      await page.keyboard.press('Shift+Tab'); await expect(resultButton(view)).toBeFocused()
      const target = key === 'Enter' ? resultItems[0] : resultItems[1]
      if (key === 'Space') { await page.keyboard.press('Tab'); await expect(resultButton(view, target)).toBeFocused() }
      const focusedGeometry = await resultButton(view, target).evaluate(element => {
        const rect = element.getBoundingClientRect()
        return { left: rect.left, right: rect.right, viewport: innerWidth, overflow: document.documentElement.scrollWidth - innerWidth }
      })
      expect(focusedGeometry.left).toBeGreaterThanOrEqual(-1)
      expect(focusedGeometry.right).toBeLessThanOrEqual(focusedGeometry.viewport + 1)
      expect(focusedGeometry.overflow).toBeLessThanOrEqual(1)
      await page.screenshot({ path: `.cache/kbd03-implementation-20261031/focus-${surface}-${broker}-${key}.png`, animations: 'disabled' })
      const count = await nativeActivationCounter(resultButton(view, target))
      expect(await count()).toBe(0)
      await page.keyboard.press(key)
      await expectResultsSelection(page, view, target)
      expect(await count()).toBe(1)
      await expect(view.input).toBeFocused()
      for (const item of resultItems) await expect(resultButton(view, item)).toHaveCount(0)
      await page.clock.runFor(600)
      for (const item of resultItems) await expect(resultButton(view, item)).toHaveCount(0)
      await expect(view.input).toBeFocused()
      expect(await page.locator('input[type="number"]').evaluateAll(elements => elements.map(element => (element as HTMLInputElement).value))).toEqual(numbers)
      state.notes.push({ stage: 'native-key-selection', surface, key, focusedGeometry, nativeClicks: await count(), target: target.pdno,
        limitation: 'Native click count plus DOM result and preserved source closure; not an internal callback invocation spy' })
    }
  })

  test(`P2-KBD-03 ${broker.toUpperCase()} ${surface} four result keys clamp focus while input caret and Tab remain native`, async ({ page }) => {
    const state = await setup(page, broker), view = await openResultsSurface(page, surface)
    await showResults(view); await focusFirstResult(view)
    const first = resultButton(view), second = resultButton(view, resultItems[1])
    const before = state.reads.length
    for (const [key, item] of [['ArrowUp', resultItems[0]], ['ArrowDown', resultItems[1]], ['ArrowDown', resultItems[1]],
      ['Home', resultItems[0]], ['End', resultItems[1]], ['End', resultItems[1]], ['ArrowUp', resultItems[0]]] as const) {
      await page.keyboard.press(key); await expect(resultButton(view, item)).toBeFocused()
      await expect(view.input).toHaveValue(resultsTerm)
      await assertNoResultsSelection(view, item)
    }
    await view.input.focus()
    const appCaret: NativeCaretObservation[] = []
    for (const key of ['End', 'Home', 'ArrowRight', 'ArrowLeft', 'ArrowDown', 'ArrowUp']) {
      const observation = await observeNativeCaret(view.input, key)
      appCaret.push(observation)
      state.notes.push({ stage: 'app-caret-key-observed-before-neutral-control', surface, broker, ...observation })
      expect(observation.before.focused).toBe(true); expect(observation.after.focused).toBe(true)
      expect(observation.after.value).toBe(resultsTerm)
      expect(state.reads.length).toBe(before)
    }
    await page.keyboard.press('Tab'); await expect(view.search).toBeFocused()
    await page.keyboard.press('Tab'); await expect(first).toBeFocused()
    await page.keyboard.press('Tab'); await expect(second).toBeFocused()
    await page.keyboard.press('Shift+Tab'); await expect(first).toBeFocused()
    await page.keyboard.press('Shift+Tab'); await expect(view.search).toBeFocused()
    await page.keyboard.press('Shift+Tab'); await expect(view.input).toBeFocused()
    await page.clock.runFor(600)
    expect(state.reads.length).toBe(before)
    await expect(first).toBeVisible(); await expect(second).toBeVisible()
    await assertNoResultsSelection(view, resultItems[0]); await assertNoResultsSelection(view, resultItems[1])
    state.notes.push({ stage: 'focus-only-keys-and-native-caret-app-contract-complete', surface, before, after: state.reads.length })
    await page.screenshot({ path: `.cache/kbd03-implementation-20261031/caret-app-${surface}-${broker}.png`, animations: 'disabled' })
    const contextProbe = await compareSameContextCaret(view.input, appCaret, `${surface}-${broker}`)
    state.notes.push({ stage: 'same-context-neutral-caret-regression', surface, broker, ...contextProbe })
    expect(contextProbe.insertion.sameParent).toBe(true)
    expect(contextProbe.insertion.cloneOwnReactKeys).toBe(0)
    expect(contextProbe.insertion.originalAfter).toEqual(contextProbe.insertion.originalBefore)
    for (const dimension of ['x', 'y', 'width', 'height'] as const) expect(Math.abs(contextProbe.insertion.cloneRect[dimension] - contextProbe.insertion.originalBefore[dimension])).toBeLessThanOrEqual(1)
    expect(contextProbe.insertion.cloneType).toBe(contextProbe.insertion.originalType)
    expect(contextProbe.insertion.cloneFont).toBe(contextProbe.insertion.originalFont)
    expect(contextProbe.insertion.cloneDirection).toBe(contextProbe.insertion.originalDirection)
    const scrollability = (nodes: typeof contextProbe.originalBefore.scrollContext) => nodes.map(({ tag, id, scrollHeight, scrollWidth, clientHeight, clientWidth }) => ({ tag, id, scrollHeight, scrollWidth, clientHeight, clientWidth }))
    expect(scrollability(contextProbe.originalAfterInsertion.scrollContext)).toEqual(scrollability(contextProbe.originalBefore.scrollContext))
    expect(contextProbe.cleanup.neutralNodesRemaining).toBe(0)
    expect(contextProbe.cleanup.focusouts.filter(event => !event.staysInsideParent)).toEqual([])
    expect(contextProbe.originalAfter).toMatchObject({ value: contextProbe.originalBefore.value,
      start: contextProbe.originalBefore.start, end: contextProbe.originalBefore.end, direction: contextProbe.originalBefore.direction, focused: true })
    expect(state.reads.length).toBe(before)
    await expect(first).toBeVisible(); await expect(second).toBeVisible()
    for (const { app, neutral } of contextProbe.pairs) {
      expect(neutral.before).toMatchObject({ value: app.before.value, start: app.before.start, end: app.before.end,
        direction: app.before.direction, focused: true })
      expect(neutral.before.scrollContext).toHaveLength(app.before.scrollContext.length)
      for (const [index, expected] of app.before.scrollContext.entries()) {
        expect(neutral.before.scrollContext[index]).toMatchObject({ tag: expected.tag,
          clientHeight: expected.clientHeight, clientWidth: expected.clientWidth, scrollHeight: expected.scrollHeight, scrollWidth: expected.scrollWidth })
      }
      expect(app.after.event?.key).toBe(app.key); expect(neutral.after.event?.key).toBe(app.key)
      expect(app.after.event).not.toBeNull(); expect(neutral.after.event).not.toBeNull()
      expect(app.after.event?.isTrusted).toBe(true); expect(neutral.after.event?.isTrusted).toBe(true)
      expect(app.after.event?.defaultPrevented).toBe(neutral.after.event?.defaultPrevented)
      expect(neutral.after.event?.defaultPrevented).toBe(false)
      expect(app.after).toMatchObject({ value: neutral.after.value, focused: true })
      expect(app.after.value).toBe(app.before.value); expect(neutral.after.value).toBe(neutral.before.value)
      expect(neutral.after.focused).toBe(true)
      if (['ArrowRight', 'ArrowLeft', 'ArrowDown', 'ArrowUp'].includes(app.key)) {
        expect(app.after).toMatchObject({ start: neutral.after.start, end: neutral.after.end, direction: neutral.after.direction })
      } else {
        expect(['Home', 'End']).toContain(app.key)
        // Native document-scroll commands can animate asynchronously. Retain their numeric caret/scroll
        // observations without imposing frame equality; result-button Home/End remains strict above.
      }
    }
    await expect(view.input).toBeFocused()
    await expect(view.input).toHaveValue(resultsTerm)
    expect(state.reads.length).toBe(before)
  })

  test(`P2-KBD-03 ${broker.toUpperCase()} ${surface} Escape external180 and pointer blur race preserve focus and remain closed600`, async ({ page }) => {
    const state = await setup(page, broker), view = await openResultsSurface(page, surface)
    await showResults(view)
    await view.input.focus(); await view.input.press('Escape')
    await expect(resultRow(view)).toHaveCount(0); await expect(view.input).toBeFocused()
    await page.clock.runFor(600); await expect(resultRow(view)).toHaveCount(0)
    await showResults(view); await focusFirstResult(view)
    await page.keyboard.press('Escape')
    await expect(resultRow(view)).toHaveCount(0); await expect(view.input).toBeFocused()
    await page.clock.runFor(600); await expect(resultRow(view)).toHaveCount(0)
    await showResults(view); await focusFirstResult(view)
    await page.clock.pauseAt(await page.evaluate(() => Date.now() + 1000))
    const outside = outsideResults(view)
    await outside.focus(); await expect(outside).toBeFocused()
    await page.clock.runFor(179); await expect(resultRow(view)).toBeVisible()
    await page.clock.runFor(1); await expect(resultRow(view)).toHaveCount(0)
    await showResults(view); await focusFirstResult(view)
    await outside.focus(); await expect(outside).toBeFocused()
    await resultRow(view).getByRole('cell', { name: resultItems[0].pdno, exact: true }).click()
    await expectResultsSelection(page, view); await expect(view.input).toBeFocused()
    await page.clock.runFor(600)
    await expect(resultRow(view)).toHaveCount(0); await expect(view.input).toBeFocused()
    state.notes.push({ stage: 'Escape-input-and-result-external180-pointer-race', surface, externalVisibleAt179: true, closedAt180: true })
  })

  test(`P2-KBD-03 ${broker.toUpperCase()} ${surface} held loading empty error and recovery do not invent result focus or selection`, async ({ page }) => {
    const state = await setup(page, broker), view = await openResultsSurface(page, surface)
    const heldTerm = '합성보류목록', emptyTerm = '합성빈목록', errorTerm = '합성오류목록', recoverTerm = '합성복구목록'
    state.setSearch(heldTerm, { items: resultItems, hold: true })
    await view.input.fill(heldTerm); await view.input.press('Enter')
    const held = await waitResultsRead(state, heldTerm)
    await expect(view.search).toHaveAttribute('aria-busy', 'true')
    for (const item of resultItems) await expect(resultButton(view, item)).toHaveCount(0)
    await page.keyboard.press('Home'); await page.keyboard.press('End'); await page.keyboard.press('ArrowDown')
    await expect(view.input).toBeFocused()
    await settleResultsRead(page, held)
    for (const item of resultItems) await expect(resultButton(view, item)).toBeVisible()
    await expect(view.input).toBeFocused(); await assertNoResultsSelection(view)

    state.setSearch(emptyTerm, { items: [] })
    await view.input.fill(emptyTerm); await view.input.press('Enter')
    await settleResultsRead(page, await waitResultsRead(state, emptyTerm))
    await expect(view.search).toHaveAttribute('aria-busy', 'false')
    for (const item of resultItems) await expect(resultButton(view, item)).toHaveCount(0)
    await page.keyboard.press('ArrowDown'); await page.keyboard.press('End'); await expect(view.input).toBeFocused()

    state.setSearch(errorTerm, { items: [], status: 503 })
    await view.input.fill(errorTerm); await view.input.press('Enter')
    await settleResultsRead(page, await waitResultsRead(state, errorTerm))
    // Preserve the real retry contract (count < 2), progressing each timer after its exact response.
    await page.clock.runFor(1000); await settleResultsRead(page, await waitResultsRead(state, errorTerm, 2))
    await page.clock.runFor(2000); await settleResultsRead(page, await waitResultsRead(state, errorTerm, 3))
    await expect(view.search).toHaveAttribute('aria-busy', 'false')
    expect(state.reads.filter(read => read.term === errorTerm)).toHaveLength(3)
    for (const item of resultItems) await expect(resultButton(view, item)).toHaveCount(0)
    await expect(view.input).toBeFocused(); await assertNoResultsSelection(view)
    state.setSearch(recoverTerm, { items: [resultItems[1]] })
    await showResults(view, recoverTerm, [resultItems[1]])
    await expect(resultButton(view)).toHaveCount(0); await expect(resultButton(view, resultItems[1])).toBeVisible()
    await expect(view.input).toBeFocused()
    state.notes.push({ stage: 'held-empty-error-recovery', surface, errorReplies: 3, oldRowsGone: true, focusNotStolen: true })
  })
}

for (const [surface, broker] of [['Trading', 'kis'], ['Strategy', 'toss'], ['LTH', 'toss']] as const) test(`P2-KBD-03 ${broker.toUpperCase()} ${surface} 40 long results keep End Home inside real scroll bounds`, async ({ page }) => {
  const items = Array.from({ length: 40 }, (_, index) => ({ pdno: `0006${String(index).padStart(2, '0')}`,
    prdt_name: `합성 긴목록 ${index + 1} ${'접근 가능한 긴 이름 '.repeat(8)}`.trim() }))
  const state = await setup(page, broker, { items }), view = await openResultsSurface(page, surface)
  await showResults(view, resultsTerm, items)
  await focusFirstResult(view, items[0]); await resultsOutline(resultButton(view, items[0]))
  const geometry = async (item: ResultItem) => resultButton(view, item).evaluate(element => {
    const paper = element.closest('.MuiPaper-root')
    if (!(paper instanceof HTMLElement)) throw new Error('No actual result scroll boundary')
    const button = element.getBoundingClientRect(), bounds = paper.getBoundingClientRect()
    return { button: { top: button.top, bottom: button.bottom, left: button.left, right: button.right },
      paper: { top: bounds.top, bottom: bounds.bottom }, scrollTop: paper.scrollTop, scrollHeight: paper.scrollHeight,
      clientHeight: paper.clientHeight, viewportHeight: innerHeight, documentOverflow: document.documentElement.scrollWidth - innerWidth }
  })
  const firstBefore = await geometry(items[0])
  await page.keyboard.press('End'); await expect(resultButton(view, items[39])).toBeFocused()
  const last = await geometry(items[39])
  expect(last.scrollTop).toBeGreaterThan(firstBefore.scrollTop)
  expect(last.scrollHeight).toBeGreaterThan(last.clientHeight)
  expect(last.button.top).toBeGreaterThanOrEqual(Math.max(0, last.paper.top) - 2)
  expect(last.button.bottom).toBeLessThanOrEqual(Math.min(last.viewportHeight, last.paper.bottom) + 2)
  await resultsOutline(resultButton(view, items[39]))
  await page.screenshot({ path: `.cache/kbd03-implementation-20261031/long-${surface}-${broker}.png`, animations: 'disabled' })
  await page.keyboard.press('Home'); await expect(resultButton(view, items[0])).toBeFocused()
  const firstAfter = await geometry(items[0])
  expect(firstAfter.scrollTop).toBeLessThanOrEqual(2)
  expect(firstAfter.button.top).toBeGreaterThanOrEqual(Math.max(0, firstAfter.paper.top) - 2)
  expect(firstAfter.button.bottom).toBeLessThanOrEqual(Math.min(firstAfter.viewportHeight, firstAfter.paper.bottom) + 2)
  expect(firstAfter.documentOverflow).toBeLessThanOrEqual(1)
  await expect(view.input).toHaveValue(resultsTerm); await assertNoResultsSelection(view, items[0])
  state.notes.push({ stage: 'long-list-actual-geometry', surface, count: 40, firstBefore, last, firstAfter })
})

for (const surface of ['Trading', 'Strategy', 'LTH'] as const) test(`P2-KBD-03 KIS ${surface} six-character RACE01 pointer return does not reopen after600`, async ({ page }) => {
  const items = [{ ...resultItems[0], prdt_name: 'RACE01' }]
  const state = await setup(page, 'kis', { items }), view = await openResultsSurface(page, surface)
  await showResults(view, resultsTerm, items)
  await page.clock.pauseAt(await page.evaluate(() => Date.now() + 1000))
  const outside = outsideResults(view)
  await outside.focus(); await expect(outside).toBeFocused()
  await resultRow(view, items[0]).getByRole('cell', { name: items[0].pdno, exact: true }).click()
  await expectResultsSelection(page, view, items[0]); await expect(view.input).toBeFocused()
  await page.clock.runFor(250); await expect(resultRow(view, items[0])).toHaveCount(0)
  await page.clock.runFor(350); await expect(resultRow(view, items[0])).toHaveCount(0)
  await expect(view.input).toHaveValue('RACE01'); await expect(view.input).toBeFocused()
  state.notes.push({ stage: 'six-character-name-pointer-race', surface,
    existingAutomaticSearchNotSuppressed: true, reads: state.reads.map(read => ({ term: read.term, id: read.id })) })
})
