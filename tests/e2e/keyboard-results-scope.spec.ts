import { expect, test, type Page } from '@playwright/test'
import { keyboardResultsFixture, installResultsSourceHarness, openResultsSurface, showResults, resultItems, resultButton, resultRow,
  focusFirstResult, assertNoResultsSelection, expectResultsSelection, resultsTerm, safeResults, captureResults,
  type ResultsFixture, type ResultsView } from './fixtures/keyboardResultsFixture'

const states = new WeakMap<Page, ResultsFixture>()
test.afterEach(async ({ page }, info) => { const state = states.get(page); if (state) try { await captureResults(page, state, info.title, info.status) } finally { safeResults(state) } })

async function replaceFocusedResult(page: Page, state: ResultsFixture, view: ResultsView,
  harness: Awaited<ReturnType<typeof installResultsSourceHarness>>) {
  await showResults(view); await focusFirstResult(view)
  await page.keyboard.press('Tab'); await expect(resultButton(view, resultItems[1])).toBeFocused()
  const identity = await harness.identity()
  const before = state.reads.filter(read => read.term === resultsTerm).length
  state.setSearch(resultsTerm, { items: [resultItems[0]] })
  await harness.refresh('search')
  expect(await harness.identity()).toBe(identity)
  expect(state.reads.filter(read => read.term === resultsTerm)).toHaveLength(before + 1)
  const response = state.reads.filter(read => read.term === resultsTerm).slice(-1)[0]!
  await response.done
  expect(response.error).toBeUndefined(); expect(response.bodyFinished).toBe(true)
  await expect(resultButton(view, resultItems[1])).toHaveCount(0)
  await assertNoResultsSelection(view, resultItems[0]); await assertNoResultsSelection(view, resultItems[1])
  await expect(view.input).toHaveValue(resultsTerm)
  state.notes.push({ stage: 'focused-second-result-removed-by-same-query-GET', oldSymbol: resultItems[1].pdno,
    remainingSymbol: resultItems[0].pdno, responseTicket: response.id, identity,
    focusRecoveryPolicy: 'No arbitrary input-focus requirement on DOM removal; no automatic selection. Explicit search resumes below.' })
  state.setSearch(resultsTerm, { items: resultItems })
  await harness.refresh('search')
  await view.input.focus(); await view.input.press('Enter')
  for (const item of resultItems) await expect(resultButton(view, item)).toBeVisible()
  await expect(view.input).toBeFocused()
  return identity
}

for (const broker of ['kis', 'toss'] as const) test(`P2-KBD-03 source ${broker.toUpperCase()} LTH replacement and synthetic enabled GET prevent row bypass then unlock keyboard selection`, async ({ page }) => {
  await page.setViewportSize(broker === 'kis' ? { width: 1280, height: 900 } : { width: 375, height: 812 })
  await page.emulateMedia({ colorScheme: broker === 'kis' ? 'light' : 'dark' })
  const state = await keyboardResultsFixture(page, broker, { controlledStrategy: true }); states.set(page, state)
  const harness = await installResultsSourceHarness(page, state)
  const view = await openResultsSurface(page, 'LTH')
  expect(harness.intercepted()).toBe(true)
  const originalInput = await view.input.elementHandle()
  if (!originalInput) throw new Error('No initial LTH input')
  const identity = await replaceFocusedResult(page, state, view, harness)
  await focusFirstResult(view)
  state.strategy.enabled = true
  await harness.refresh('strategies') // Synthetic GET only: do not click toggle/save/add/start.
  expect(await harness.identity()).toBe(identity)
  expect(await view.input.evaluate((element, original) => element === original, originalInput)).toBe(true)
  await expect(view.input).toBeDisabled(); await expect(view.search).toBeDisabled()
  const add = view.area.getByRole('button', { name: '대상 추가', exact: true })
  await expect(add).toHaveCount(2)
  await expect(add.nth(0)).toBeDisabled(); await expect(add.nth(1)).toBeDisabled()
  let pointerPathPresent = false
  for (const item of resultItems) {
    const button = resultButton(view, item)
    if (await button.count()) await expect(button).toBeDisabled()
    const row = resultRow(view, item)
    if (await row.count()) {
      pointerPathPresent = true
      await row.getByRole('cell', { name: item.pdno, exact: true }).click()
      await assertNoResultsSelection(view, item)
    }
  }
  await page.keyboard.press('Enter'); await page.keyboard.press('Space')
  await expect(view.input).toHaveValue(resultsTerm)
  for (const item of resultItems) await assertNoResultsSelection(view, item)
  await page.clock.runFor(600)
  for (const item of resultItems) await assertNoResultsSelection(view, item)
  state.notes.push({ stage: 'synthetic-enabled-row-lock', broker, pointerPathPresent,
    removedResultsAlsoCountAsLocked: true, noToggleOrSave: true, identity })

  state.strategy.enabled = false
  await harness.refresh('strategies')
  expect(await harness.identity()).toBe(identity)
  await expect(view.input).toBeEnabled()
  await showResults(view); await focusFirstResult(view)
  await page.keyboard.press('Space')
  await expectResultsSelection(page, view); await expect(view.input).toBeFocused()
  await expect(add.nth(0)).toBeEnabled(); await expect(add.nth(1)).toBeEnabled()
  await page.clock.runFor(600); await expect(resultRow(view)).toHaveCount(0)
  expect(state.audit.writes).toEqual([])
})

for (const [surface, broker] of [['Trading', 'toss'], ['Strategy', 'kis']] as const) test(`P2-KBD-03 source ${broker.toUpperCase()} ${surface} focused result replacement never selects another row and explicit keyboard recovers`, async ({ page }) => {
  const state = await keyboardResultsFixture(page, broker); states.set(page, state)
  const harness = await installResultsSourceHarness(page, state)
  const view = await openResultsSurface(page, surface)
  expect(harness.intercepted()).toBe(true)
  const identity = await replaceFocusedResult(page, state, view, harness)
  await focusFirstResult(view); await page.keyboard.press('Tab')
  await expect(resultButton(view, resultItems[1])).toBeFocused()
  await page.keyboard.press('Enter')
  await expectResultsSelection(page, view, resultItems[1]); await expect(view.input).toBeFocused()
  await page.clock.runFor(600); await expect(resultRow(view, resultItems[1])).toHaveCount(0)
  expect(await harness.identity()).toBe(identity)
})
