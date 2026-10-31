import { expect, test, type Page } from '@playwright/test'
import { keyboardProfileFixture, fakeProfiles, addPath, updatePath, deletePath, newTossAccountsPath, savedTossAccountsPath,
  openAddProfile, openEditProfile, openDeleteProfile, ordinaryProfileClosePositive, profileTick, waitProfileTicket,
  finishProfileTicket, addProfileTrigger, profileCard, captureProfiles, safeProfiles, oldAccounts,
  fillSyntheticAdd, expectedAddBody, expectedEditBody, attemptMainClose, currentError, startKisDetection, type ProfileFixture } from './fixtures/keyboardProfileFixture'

const states = new WeakMap<Page, ProfileFixture>()
test.afterEach(async ({ page }, info) => { const state = states.get(page); if (state) try { await captureProfiles(page, state, info.title, info.status) } finally { safeProfiles(state) } })
async function setup(page: Page, retainDeletedProfiles = false, narrow = false) {
  await page.setViewportSize(narrow ? { width: 375, height: 812 } : { width: 1280, height: 900 }); await page.emulateMedia({ colorScheme: narrow ? 'dark' : 'light' })
  const state = await keyboardProfileFixture(page, { retainDeletedProfiles }); states.set(page, state)
  await page.goto('/settings'); await expect(profileCard(page)).toBeVisible(); await expect(profileCard(page, fakeProfiles[2])).toBeVisible()
  return state
}

test('P2-KBD-05 minimum Add pending Escape matches disabled Cancel after ordinary close and input Enter positives', async ({ page }) => {
  const state = await setup(page), open = () => openAddProfile(page, 'kis')
  await ordinaryProfileClosePositive(page, state, open)
  const { dialog } = await open()
  await dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill('합성 새 KIS')
  await dialog.getByLabel('APP KEY', { exact: true }).fill('SYNTHETIC-KIS-KEY')
  await dialog.getByLabel('APP SECRET', { exact: true }).fill('SYNTHETIC-KIS-SECRET')
  await dialog.getByLabel('계좌번호 (10자리 입력, 예: 12345678-01)', { exact: true }).fill('00000002-01')
  const realModeChips = dialog.getByText('실전투자', { exact: true })
  await expect(realModeChips).toHaveCount(2) // Existing detection-result and mode chips both describe the same mode.
  await expect(realModeChips.nth(0)).toBeVisible(); await expect(realModeChips.nth(1)).toBeVisible()
  state.hold(addPath); await dialog.getByRole('button', { name: '추가', exact: true }).click()
  const held = await waitProfileTicket(state, addPath)
  expect(held.body).toEqual({ broker_id: 'kis', name: '합성 새 KIS', is_paper_trading: false, live_trading_consent: false,
    app_key: 'SYNTHETIC-KIS-KEY', app_secret: 'SYNTHETIC-KIS-SECRET', account_no: '00000002-01' })
  await expect(dialog.getByRole('button', { name: '취소', exact: true })).toBeDisabled()
  const pendingName = dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })
  await pendingName.focus(); await expect(pendingName).toBeFocused()
  const escapeTarget = await pendingName.elementHandle()
  if (!escapeTarget) throw new Error('No pending Add input for the real Dialog Escape path')
  const focusBeforeEscape = await escapeTarget.evaluate(element => {
    Reflect.set(element, '__kbd05EscapeEvent', null)
    element.addEventListener('keydown', event => Reflect.set(element, '__kbd05EscapeEvent', event), { once: true, passive: true, capture: true })
    return { isActualActiveElement: element === document.activeElement, activeTag: document.activeElement?.tagName,
      activeId: document.activeElement?.id, dialogContainsActive: element.closest('[role="dialog"]')?.contains(document.activeElement) ?? false }
  })
  await page.keyboard.press('Escape'); await profileTick(page)
  const observedEscape = await escapeTarget.evaluate(element => {
    const event = Reflect.get(element, '__kbd05EscapeEvent') as KeyboardEvent | null
    return { key: event?.key ?? null, isTrusted: event?.isTrusted ?? null, defaultPrevented: event?.defaultPrevented ?? null,
      eventTargetWasInput: event?.target === element, inputStillConnected: element.isConnected,
      activeTag: document.activeElement?.tagName, activeId: document.activeElement?.id,
      dialogContainsActive: element.closest('[role="dialog"]')?.contains(document.activeElement) ?? false }
  })
  state.notes.push({ stage: 'Add-main-held-Escape', pending: !held.bodyFinished, stillOpen: await dialog.count(), focusBeforeEscape, observedEscape })
  expect(focusBeforeEscape.dialogContainsActive).toBe(true)
  expect(observedEscape).toMatchObject({ key: 'Escape', isTrusted: true, eventTargetWasInput: true })
  await expect(dialog).toBeVisible(); await expect(dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })).toHaveValue('합성 새 KIS')
  state.release(addPath); await finishProfileTicket(page, held); await expect(dialog).toHaveCount(0)
  expect(state.reads(addPath)).toHaveLength(1)
})

test('P2-KBD-05 minimum Edit pending backdrop matches disabled Cancel after ordinary close positives', async ({ page }) => {
  const state = await setup(page), open = () => openEditProfile(page)
  await ordinaryProfileClosePositive(page, state, open)
  // Prove this exact native pointer coordinate is a real normal backdrop close first.
  let view = await open(); await page.mouse.click(5, 5); await expect(view.dialog).toHaveCount(0); await expect(view.trigger).toBeFocused()
  view = await open(); const { dialog } = view
  await expect(dialog).toContainText('실거래 동의 없음')
  await dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill('합성 Toss 수정')
  state.hold(updatePath); await dialog.getByRole('button', { name: '저장', exact: true }).click()
  const held = await waitProfileTicket(state, updatePath)
  expect(held.body).toEqual({ id: fakeProfiles[2].id, broker_id: 'toss', name: '합성 Toss 수정', is_paper_trading: false, live_trading_consent: false, account_no: '7101' })
  await expect(dialog.getByRole('button', { name: '취소', exact: true })).toBeDisabled()
  await page.mouse.click(5, 5); await profileTick(page)
  state.notes.push({ stage: 'Edit-main-held-backdrop', pending: !held.bodyFinished, stillOpen: await dialog.count() })
  await expect(dialog).toBeVisible(); await expect(dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })).toHaveValue('합성 Toss 수정')
  state.release(updatePath); await finishProfileTicket(page, held); await expect(dialog).toHaveCount(0)
  expect(state.reads(updatePath)).toHaveLength(1)
})

test('P2-KBD-05 minimum Delete Enter Space repeats remain one request after the shared scope queue finishes', async ({ page }) => {
  const state = await setup(page, true), open = () => openDeleteProfile(page)
  await ordinaryProfileClosePositive(page, state, open, false)
  const { dialog } = await open(), button = dialog.getByRole('button', { name: '삭제', exact: true })
  state.hold(deletePath); await button.focus(); await page.keyboard.press('Enter')
  const first = await waitProfileTicket(state, deletePath)
  await page.keyboard.press('Space'); await page.keyboard.press('Enter')
  expect(state.reads(deletePath)).toHaveLength(1) // Serialization alone can satisfy this, so it is not the terminal assertion.
  state.notes.push({ stage: 'Delete-first-held-requests', count: state.reads(deletePath).length, sharedScopeIsNotDedup: true })
  state.release(deletePath); await finishProfileTicket(page, first)
  // Repeated mutate switches the observer to the final queued operation. Its onSuccess closes only
  // after its own account publication completes, so await the close before counting every HTTP write.
  await expect(dialog).toHaveCount(0)
  await Promise.all(state.reads(deletePath).map(ticket => finishProfileTicket(page, ticket))); await profileTick(page)
  state.notes.push({ stage: 'Delete-after-final-observed-success-and-publication', count: state.reads(deletePath).length,
    statuses: state.reads(deletePath).map(ticket => ticket.status), retainedSyntheticCardForQueueProof: true })
  expect(state.reads(deletePath)).toHaveLength(1)
  expect(state.reads(deletePath)[0].body).toEqual({ id: fakeProfiles[1].id })
})

test('P2-KBD-05 minimum Delete failure keeps target readable error and one explicit retry with safe focus', async ({ page }) => {
  const state = await setup(page), open = () => openDeleteProfile(page, fakeProfiles[0])
  await ordinaryProfileClosePositive(page, state, open, false)
  const { dialog } = await open()
  await expect(dialog).toContainText('현재 활성 프로파일입니다. 삭제 시 다른 프로파일로 자동 전환됩니다.')
  const configBefore = state.reads('/api/app-config').length
  state.reply(deletePath, 500, { code: 'SYNTHETIC_DELETE_500', message: '합성 삭제 실패: 현재 대상을 유지하세요.' })
  await dialog.getByRole('button', { name: '삭제', exact: true }).click()
  await finishProfileTicket(page, await waitProfileTicket(state, deletePath))
  await finishProfileTicket(page, await waitProfileTicket(state, '/api/app-config', configBefore + 1))
  await expect(dialog).toBeVisible(); await expect(dialog).toContainText(fakeProfiles[0].name)
  const deleteError = dialog.getByRole('alert').filter({ hasText: '합성 삭제 실패: 현재 대상을 유지하세요.' })
  await expect(deleteError).toHaveCount(1); await expect(deleteError).toBeVisible()
  await expect(dialog).toContainText('현재 활성 프로파일입니다. 삭제 시 다른 프로파일로 자동 전환됩니다.')
  state.reply(deletePath, 200, null)
  await dialog.getByRole('button', { name: '삭제', exact: true }).click()
  await finishProfileTicket(page, await waitProfileTicket(state, deletePath, 2))
  await expect(dialog).toHaveCount(0); expect(state.reads(deletePath)).toHaveLength(2)
  await expect(profileCard(page)).toHaveCount(0); await expect(addProfileTrigger(page, 'kis')).toBeFocused()
})

test('P2-KBD-05 minimum Add Toss auxiliary success after close and same broker reopen cannot select an old account', async ({ page }) => {
  const state = await setup(page), open = () => openAddProfile(page, 'toss')
  await ordinaryProfileClosePositive(page, state, open)
  let { dialog, trigger } = await open()
  await dialog.getByLabel('Client ID', { exact: true }).fill('SYNTHETIC-TOSS-ID')
  await dialog.getByLabel('Client Secret', { exact: true }).fill('SYNTHETIC-TOSS-SECRET')
  state.hold(newTossAccountsPath); await dialog.getByRole('button', { name: '계좌 조회', exact: true }).click()
  const old = await waitProfileTicket(state, newTossAccountsPath)
  expect(old.body).toEqual({ client_id: 'SYNTHETIC-TOSS-ID', client_secret: 'SYNTHETIC-TOSS-SECRET' })
  await expect(dialog.getByRole('button', { name: '취소', exact: true })).toBeEnabled()
  await dialog.getByRole('button', { name: '취소', exact: true }).click(); await expect(dialog).toHaveCount(0); await expect(trigger).toBeFocused()
  ;({ dialog, trigger } = await open())
  await dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill('합성 새 세션 이름')
  await expect(dialog.getByRole('textbox', { name: 'accountSeq', exact: true })).toHaveValue('')
  state.release(newTossAccountsPath); await finishProfileTicket(page, old)
  state.notes.push({ stage: 'Add-Toss-previous-session-body-finished', ticket: old.id, sameBrokerReopened: true })
  await expect(dialog).toBeVisible(); await expect(dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })).toHaveValue('합성 새 세션 이름')
  await expect(dialog.getByRole('textbox', { name: 'accountSeq', exact: true })).toHaveValue('')
  await expect(dialog.getByRole('combobox', { name: 'accountSeq', exact: true })).toHaveCount(0)
  await expect(dialog).not.toContainText('1개 계좌를 조회했습니다.')
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
})

test('P2-KBD-05 minimum Edit Toss auxiliary error after close and same ID reopen cannot leak into a new draft', async ({ page }) => {
  const state = await setup(page), open = () => openEditProfile(page)
  await ordinaryProfileClosePositive(page, state, open)
  let { dialog, trigger } = await open()
  state.reply(savedTossAccountsPath, 500, { code: 'SYNTHETIC_AUX_500', message: '합성 이전 세션 조회 실패' })
  state.hold(savedTossAccountsPath); await dialog.getByRole('button', { name: '계좌 조회', exact: true }).click()
  const old = await waitProfileTicket(state, savedTossAccountsPath)
  expect(old.body).toBeNull()
  await expect(dialog.getByRole('button', { name: '취소', exact: true })).toBeEnabled()
  await page.keyboard.press('Escape'); await expect(dialog).toHaveCount(0); await expect(trigger).toBeFocused()
  ;({ dialog, trigger } = await open())
  await dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill('합성 다시 연 편집')
  state.release(savedTossAccountsPath); await finishProfileTicket(page, old)
  state.notes.push({ stage: 'Edit-Toss-previous-session-error-body-finished', ticket: old.id, sameProfileId: fakeProfiles[2].id })
  await expect(dialog).toBeVisible(); await expect(dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })).toHaveValue('합성 다시 연 편집')
  await expect(dialog).not.toContainText('합성 이전 세션 조회 실패')
  await expect(dialog.getByRole('textbox', { name: 'accountSeq', exact: true })).toHaveValue('7101')
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
})


const mainMatrix = [
  { kind: 'Add', broker: 'kis', activation: 'same-turn', status: 200 },
  { kind: 'Add', broker: 'toss', activation: 'Space', status: 200 },
  { kind: 'Edit', broker: 'kis', activation: 'Enter', status: 500 },
  { kind: 'Edit', broker: 'toss', activation: 'same-turn', status: 500 },
] as const
async function mainContract(page: Page, item: { kind: 'Add' | 'Edit'; broker: 'kis' | 'toss'; activation: string; status: number }) {
  const state = await setup(page, false, item.broker === 'toss')
  const profile = item.broker === 'kis' ? fakeProfiles[0] : fakeProfiles[2]
  const open = () => item.kind === 'Add' ? openAddProfile(page, item.broker) : openEditProfile(page, profile)
  await ordinaryProfileClosePositive(page, state, open)
  const view = await open(), { dialog } = view
  const name = `합성 ${item.kind} 첫 초안`, edited = `합성 ${item.kind} pending 변경`
  if (item.kind === 'Add') await fillSyntheticAdd(view, item.broker, name)
  else await dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill(name)
  if (item.broker === 'toss') await expect(dialog).toContainText('실거래 동의 없음')
  if (item.kind === 'Add' && item.broker === 'kis') {
    const account = dialog.getByLabel('계좌번호 (10자리 입력, 예: 12345678-01)', { exact: true })
    await account.fill('00000002-22'); await expect(dialog).toContainText('퇴직연금 계좌')
    await account.fill('00000002-01')
  }
  const nameInput = dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })
  await nameInput.focus(); await page.keyboard.press('Tab')
  expect(await dialog.evaluate(element => element.contains(document.activeElement))).toBe(true)
  await page.keyboard.press('Shift+Tab'); await expect(nameInput).toBeFocused()
  const path = item.kind === 'Add' ? addPath : updatePath, label = item.kind === 'Add' ? '추가' : '저장'
  const submit = dialog.getByRole('button', { name: label, exact: true })
  const original = await submit.elementHandle(); if (!original) throw new Error('No named main submit button')
  const configBefore = state.reads('/api/app-config').length
  state.hold(path); state.hold('/api/app-config')
  const message = `합성 ${item.kind} 저장 실패: 현재 초안을 유지하세요.`
  if (item.status === 500) state.reply(path, 500, { code: 'SYNTHETIC_MAIN_500', message })
  if (item.activation === 'same-turn') await submit.evaluate(element => {
    for (let index = 0; index < 3; index++) (element as HTMLButtonElement).click()
  })
  else { await submit.focus(); await page.keyboard.press(item.activation) }
  const first = await waitProfileTicket(state, path)
  const geometry = await dialog.boundingBox(); if (!geometry) throw new Error('No visible profile dialog geometry')
  const viewport = page.viewportSize()!
  expect(geometry.x).toBeGreaterThanOrEqual(0); expect(geometry.x + geometry.width).toBeLessThanOrEqual(viewport.width + 1)
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
  await page.screenshot({ path: `.cache/kbd05-implementation-20261031/main-pending-${item.kind}-${item.broker}-${item.status}.png`, animations: 'disabled' })
  expect(first.body).toEqual(item.kind === 'Add' ? expectedAddBody(item.broker, name) : expectedEditBody(profile, name))
  await expect(submit).toHaveAccessibleName(label); await expect(submit).toHaveAttribute('aria-busy', 'true')
  expect(await submit.evaluate((element, previous) => element === previous, original)).toBe(true)
  await submit.focus(); await expect(submit).toBeFocused()
  await page.keyboard.press('Enter'); await page.keyboard.press('Space')
  const bounds = await submit.boundingBox(); if (!bounds) throw new Error('No named pending submit bounds')
  await page.mouse.click(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2)
  await expect(nameInput).toBeEnabled(); await nameInput.fill(edited)
  await attemptMainClose(page, view)
  await expect(nameInput).toHaveValue(edited)
  state.release(path); await finishProfileTicket(page, first)
  const publication = await waitProfileTicket(state, '/api/app-config', configBefore + 1)
  expect(publication.held).toBe(true); expect(publication.bodyFinished).toBe(false)
  await attemptMainClose(page, view)
  expect(state.reads(path)).toHaveLength(1)
  state.notes.push({ stage: 'main-HTTP-finished-but-publication-pending', ...item, publication: publication.id,
    mainTicket: first.id, bodyFinished: first.bodyFinished, draftEditablePolicyPreserved: true, geometry, viewport,
    activationKind: item.activation === 'same-turn' ? 'untrusted HTMLElement.click burst plus subsequent native keyboard/pointer' : 'native keyboard' })
  state.release('/api/app-config'); await finishProfileTicket(page, publication)
  if (item.status === 500) {
    await currentError(view, message); await expect(nameInput).toHaveValue(edited)
    await expect(submit).toBeEnabled(); expect(state.reads(path)).toHaveLength(1)
    state.clearReply(path)
    await submit.focus(); await page.keyboard.press('Enter')
    const retry = await waitProfileTicket(state, path, 2); await finishProfileTicket(page, retry)
    expect(retry.body).toEqual(item.kind === 'Add' ? expectedAddBody(item.broker, edited) : expectedEditBody(profile, edited))
  }
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await Promise.all(state.reads(path).map(ticket => finishProfileTicket(page, ticket)))
  expect(state.reads(path)).toHaveLength(item.status === 500 ? 2 : 1)
  if (item.kind === 'Add') await expect(view.trigger).toBeFocused()
  else await expect(profileCard(page, { ...profile, name: edited }).getByRole('button', { name: '편집', exact: true })).toBeFocused()
  state.notes.push({ stage: 'main-current-owner-internal-close', kind: item.kind, broker: item.broker,
    exactProfileWrites: state.reads(path).length, sourceAwaitBoundary: item.kind === 'Add' ? 'success publication' : 'error reconciliation',
    inactiveEditSuccessVoidInvalidationNotAwaited: item.kind === 'Edit' && !profile.is_active })
}
for (const item of mainMatrix) test(`P2-KBD-05 ${item.kind} ${item.broker} pending close repeats and ${item.status === 200 ? 'publication success' : 'reconciliation error retry'}`, async ({ page }) => mainContract(page, item))
test('P2-KBD-05 Add toss current failure preserves edited draft and explicit retry owns success close', async ({ page }) => mainContract(page, { kind: 'Add', broker: 'toss', activation: 'Enter', status: 500 }))

for (const userMoves of [false, true]) test(`P2-KBD-05 Delete delayed profile removal ${userMoves ? 'does not steal a new user focus' : 'restores a safe fallback after trigger disappears'}`, async ({ page }) => {
  const state = await setup(page), target = userMoves ? fakeProfiles[1] : fakeProfiles[0]
  const view = await openDeleteProfile(page, target), { dialog, trigger } = view
  if (target.is_active) await expect(dialog).toContainText('현재 활성 프로파일입니다.')
  const profilesBefore = state.reads('/api/profiles').length, configBefore = state.reads('/api/app-config').length
  state.hold(deletePath); state.hold('/api/app-config'); state.hold('/api/profiles')
  const submit = dialog.getByRole('button', { name: '삭제', exact: true })
  await submit.focus(); await page.keyboard.press('Enter')
  const deletion = await waitProfileTicket(state, deletePath)
  expect(deletion.body).toEqual({ id: target.id })
  await expect(submit).toHaveAccessibleName('삭제'); await expect(submit).toHaveAttribute('aria-busy', 'true')
  await page.keyboard.press('Space'); await page.keyboard.press('Enter')
  await attemptMainClose(page, view, submit)
  state.release(deletePath); await finishProfileTicket(page, deletion)
  const publication = await waitProfileTicket(state, '/api/app-config', configBefore + 1)
  await attemptMainClose(page, view, submit)
  state.release('/api/app-config'); await finishProfileTicket(page, publication)
  await expect(page.getByRole('dialog')).toHaveCount(0)
  const list = await waitProfileTicket(state, '/api/profiles', profilesBefore + 1)
  expect(list.held).toBe(true)
  await expect(trigger).toHaveCount(1)
  const userControl = addProfileTrigger(page, 'toss')
  if (userMoves) { await userControl.focus(); await expect(userControl).toBeFocused() }
  state.release('/api/profiles'); await finishProfileTicket(page, list)
  await expect(profileCard(page, target)).toHaveCount(0)
  await expect(userMoves ? userControl : addProfileTrigger(page, target.broker_id)).toBeFocused()
  expect(state.reads(deletePath)).toHaveLength(1)
  state.notes.push({ stage: 'delayed-list-removal-focus', userMoves, deletedId: target.id, publication: publication.id, listTicket: list.id })
})

const detectPath = '/api/detect-trading-type'
for (const kind of ['Add', 'Edit'] as const) test(`P2-KBD-05 KIS ${kind} permitted auxiliary close and same target reopen fence old detect success`, async ({ page }) => {
  const state = await setup(page), open = () => kind === 'Add' ? openAddProfile(page, 'kis') : openEditProfile(page, fakeProfiles[0])
  let view = await open()
  const oldMode = kind === 'Add', expectedMode = kind === 'Edit'
  state.reply(detectPath, 200, { is_paper_trading: oldMode, message: '합성 이전 세션 감지' }); state.hold(detectPath)
  await startKisDetection(view, kind, 'OLD')
  const old = await waitProfileTicket(state, detectPath)
  await expect(view.dialog.getByRole('button', { name: '취소', exact: true })).toBeEnabled()
  await view.dialog.getByRole('button', { name: '취소', exact: true }).click(); await expect(view.dialog).toHaveCount(0)
  view = await open(); await view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill('합성 새 KIS 세션')
  await finishProfileTicket(page, old)
  await expect(view.dialog.getByLabel(/자동 감지 또는 직접 선택/)).toBeChecked({ checked: expectedMode })
  await expect(view.dialog).not.toContainText('합성 이전 세션 감지')
  await expect(view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })).toHaveValue('합성 새 KIS 세션')
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
})

test('P2-KBD-05 Add broker ABA cannot revive an old Toss lookup', async ({ page }) => {
  const state = await setup(page)
  let view = await openAddProfile(page, 'toss')
  await view.dialog.getByLabel('Client ID', { exact: true }).fill('SYNTHETIC-OLD-ID')
  await view.dialog.getByLabel('Client Secret', { exact: true }).fill('SYNTHETIC-OLD-SECRET')
  state.hold(newTossAccountsPath); await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click()
  const old = await waitProfileTicket(state, newTossAccountsPath)
  await view.dialog.getByRole('button', { name: '취소', exact: true }).click()
  view = await openAddProfile(page, 'kis'); await view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill('합성 중간 KIS')
  await page.keyboard.press('Escape'); await expect(view.dialog).toHaveCount(0)
  view = await openAddProfile(page, 'toss'); await view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill('합성 마지막 Toss')
  await finishProfileTicket(page, old)
  await expect(view.dialog.getByRole('textbox', { name: 'accountSeq', exact: true })).toHaveValue('')
  await expect(view.dialog).not.toContainText('1개 계좌를 조회했습니다.')
  await expect(view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })).toHaveValue('합성 마지막 Toss')
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
})

test('P2-KBD-05 Edit profile ABA cannot revive old saved-key account selection', async ({ page }) => {
  const state = await setup(page)
  let view = await openEditProfile(page, fakeProfiles[2])
  state.hold(savedTossAccountsPath); await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click()
  const old = await waitProfileTicket(state, savedTossAccountsPath)
  await view.dialog.getByRole('button', { name: '취소', exact: true }).click()
  view = await openEditProfile(page, fakeProfiles[3]); await expect(view.dialog.getByRole('textbox', { name: 'accountSeq', exact: true })).toHaveValue('7102')
  await page.keyboard.press('Escape'); await expect(view.dialog).toHaveCount(0)
  view = await openEditProfile(page, fakeProfiles[2]); await view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true }).fill('합성 A 재진입')
  await finishProfileTicket(page, old)
  await expect(view.dialog.getByRole('textbox', { name: 'accountSeq', exact: true })).toHaveValue('7101')
  await expect(view.dialog.getByRole('textbox', { name: '프로파일 이름', exact: true })).toHaveValue('합성 A 재진입')
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
  await expect(view.dialog).not.toContainText('1개 계좌를 조회했습니다.')
})

test('P2-KBD-05 new Toss lookup remains pending when an old session error and finally settle', async ({ page }) => {
  const state = await setup(page)
  let view = await openAddProfile(page, 'toss')
  await view.dialog.getByLabel('Client ID', { exact: true }).fill('SYNTHETIC-OLD-ID'); await view.dialog.getByLabel('Client Secret', { exact: true }).fill('SYNTHETIC-OLD-SECRET')
  state.reply(newTossAccountsPath, 500, { code: 'SYNTHETIC_OLD_ERROR', message: '합성 이전 조회 오류' }); state.hold(newTossAccountsPath)
  await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click(); const old = await waitProfileTicket(state, newTossAccountsPath)
  await view.dialog.getByRole('button', { name: '취소', exact: true }).click()
  view = await openAddProfile(page, 'toss')
  await view.dialog.getByLabel('Client ID', { exact: true }).fill('SYNTHETIC-NEW-ID'); await view.dialog.getByLabel('Client Secret', { exact: true }).fill('SYNTHETIC-NEW-SECRET')
  state.reply(newTossAccountsPath, 200, [{ ...oldAccounts[0], account_seq: '7882', label: '합성 현재 계좌' }])
  await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click(); const current = await waitProfileTicket(state, newTossAccountsPath, 2)
  await finishProfileTicket(page, old)
  expect(current.bodyFinished).toBe(false)
  await expect(view.dialog.getByRole('button', { name: '조회 중', exact: true })).toBeDisabled()
  await expect(view.dialog).not.toContainText('합성 이전 조회 오류')
  await expect(view.dialog.getByRole('textbox', { name: 'accountSeq', exact: true })).toHaveValue('')
  state.release(newTossAccountsPath); await finishProfileTicket(page, current)
  await expect(view.dialog.getByRole('combobox', { name: 'accountSeq', exact: true })).toContainText('합성 현재 계좌')
  expect(current.body).toEqual({ client_id: 'SYNTHETIC-NEW-ID', client_secret: 'SYNTHETIC-NEW-SECRET' })
  state.notes.push({ stage: 'old-finally-cannot-finish-new-auxiliary', old: old.id, current: current.id, independentPendingVerified: true })
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
})

for (const kind of ['Add', 'Edit'] as const) test(`P2-KBD-05 KIS ${kind} key revision rejects old detect then applies a fresh current read`, async ({ page }) => {
  const state = await setup(page), view = kind === 'Add' ? await openAddProfile(page, 'kis') : await openEditProfile(page, fakeProfiles[0])
  const currentMode = kind === 'Edit', oldMode = !currentMode
  state.reply(detectPath, 200, { is_paper_trading: oldMode, message: '합성 이전 키 감지' }); state.hold(detectPath)
  await startKisDetection(view, kind, 'OLD'); const old = await waitProfileTicket(state, detectPath)
  await view.dialog.getByLabel(kind === 'Edit' ? 'APP KEY (변경 시에만 입력)' : 'APP KEY', { exact: true }).fill('SYNTHETIC-CHANGED-KEY')
  await finishProfileTicket(page, old)
  await expect(view.dialog).not.toContainText('합성 이전 키 감지')
  await expect(view.dialog.getByLabel(/자동 감지 또는 직접 선택/)).toBeChecked({ checked: currentMode })
  state.reply(detectPath, 200, { is_paper_trading: oldMode, message: '합성 현재 키 감지' })
  await startKisDetection(view, kind, 'CURRENT'); const current = await waitProfileTicket(state, detectPath, 2)
  state.release(detectPath); await finishProfileTicket(page, current)
  await expect(view.dialog).toContainText('합성 현재 키 감지')
  await expect(view.dialog.getByLabel(/자동 감지 또는 직접 선택/)).toBeChecked({ checked: oldMode })
  expect(current.body).toEqual({ appKey: 'SYNTHETIC-CURRENT-KEY', appSecret: 'SYNTHETIC-CURRENT-SECRET' })
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
})

for (const kind of ['Add', 'Edit'] as const) test(`P2-KBD-05 Toss ${kind} one key revision fences old account lookup and preserves its original branch`, async ({ page }) => {
  const state = await setup(page), view = kind === 'Add' ? await openAddProfile(page, 'toss') : await openEditProfile(page)
  if (kind === 'Add') {
    await view.dialog.getByLabel('Client ID', { exact: true }).fill('SYNTHETIC-OLD-ID'); await view.dialog.getByLabel('Client Secret', { exact: true }).fill('SYNTHETIC-OLD-SECRET')
  }
  const path = kind === 'Add' ? newTossAccountsPath : savedTossAccountsPath
  state.hold(path); await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click(); const old = await waitProfileTicket(state, path)
  await view.dialog.getByLabel(kind === 'Add' ? 'Client ID' : 'Client ID (변경 시에만 입력)', { exact: true }).fill('SYNTHETIC-ONE-CHANGED-ID')
  await finishProfileTicket(page, old)
  await expect(view.dialog.getByRole('textbox', { name: 'accountSeq', exact: true })).toHaveValue(kind === 'Add' ? '' : '7101')
  await expect(view.dialog).not.toContainText('1개 계좌를 조회했습니다.')
  state.reply(path, 200, [{ ...oldAccounts[0], account_seq: '7101', label: '합성 보존 계좌' }, oldAccounts[0]])
  state.release(path); await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click()
  const current = await waitProfileTicket(state, path, 2); await finishProfileTicket(page, current)
  if (kind === 'Edit') { expect(state.reads(newTossAccountsPath)).toHaveLength(0); expect(current.body).toBeNull(); await expect(view.dialog.getByRole('combobox', { name: 'accountSeq', exact: true })).toContainText('합성 보존 계좌') }
  else expect(current.body).toEqual({ client_id: 'SYNTHETIC-ONE-CHANGED-ID', client_secret: 'SYNTHETIC-OLD-SECRET' })
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
})

test('P2-KBD-05 Edit Toss saved and both-new-key branches keep existing account or apply one current account', async ({ page }) => {
  const state = await setup(page), view = await openEditProfile(page)
  state.reply(savedTossAccountsPath, 200, [{ ...oldAccounts[0], account_seq: '7101', label: '합성 현재 저장 계좌' }, oldAccounts[0]])
  await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click()
  const saved = await waitProfileTicket(state, savedTossAccountsPath); await finishProfileTicket(page, saved)
  expect(saved.body).toBeNull(); await expect(view.dialog.getByRole('combobox', { name: 'accountSeq', exact: true })).toContainText('합성 현재 저장 계좌')
  await view.dialog.getByLabel('Client ID (변경 시에만 입력)', { exact: true }).fill('SYNTHETIC-EDIT-NEW-ID')
  await view.dialog.getByLabel('Client Secret (변경 시에만 입력)', { exact: true }).fill('SYNTHETIC-EDIT-NEW-SECRET')
  state.reply(newTossAccountsPath, 200, [{ ...oldAccounts[0], account_seq: '7882', label: '합성 신규 키 계좌' }])
  await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click()
  const fresh = await waitProfileTicket(state, newTossAccountsPath); await finishProfileTicket(page, fresh)
  expect(fresh.body).toEqual({ client_id: 'SYNTHETIC-EDIT-NEW-ID', client_secret: 'SYNTHETIC-EDIT-NEW-SECRET' })
  await expect(view.dialog.getByRole('combobox', { name: 'accountSeq', exact: true })).toContainText('합성 신규 키 계좌')
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
})

test('P2-KBD-05 current Toss empty error retry and multiple accounts remain auxiliary reads only', async ({ page }) => {
  const state = await setup(page, false, true), view = await openAddProfile(page, 'toss')
  await view.dialog.getByLabel('Client ID', { exact: true }).fill('SYNTHETIC-CURRENT-ID'); await view.dialog.getByLabel('Client Secret', { exact: true }).fill('SYNTHETIC-CURRENT-SECRET')
  await view.dialog.getByRole('textbox', { name: 'accountSeq', exact: true }).fill('7103')
  state.reply(newTossAccountsPath, 200, [])
  await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click(); await finishProfileTicket(page, await waitProfileTicket(state, newTossAccountsPath))
  await expect(view.dialog).toContainText('조회된 토스증권 계좌가 없습니다.')
  state.reply(newTossAccountsPath, 500, { code: 'SYNTHETIC_CURRENT_500', message: '합성 현재 조회 실패' })
  await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click(); await finishProfileTicket(page, await waitProfileTicket(state, newTossAccountsPath, 2))
  await currentError(view, '합성 현재 조회 실패')
  state.reply(newTossAccountsPath, 200, [{ ...oldAccounts[0], account_seq: '7103', label: '합성 수동 계좌 보존' }, oldAccounts[0]])
  await view.dialog.getByRole('button', { name: '계좌 조회', exact: true }).click(); await finishProfileTicket(page, await waitProfileTicket(state, newTossAccountsPath, 3))
  await expect(view.dialog.getByRole('combobox', { name: 'accountSeq', exact: true })).toContainText('합성 수동 계좌 보존')
  await expect(view.dialog).not.toContainText('합성 현재 조회 실패')
  expect(state.tickets.filter(ticket => ticket.kind === 'mock-profile-write')).toHaveLength(0)
})
