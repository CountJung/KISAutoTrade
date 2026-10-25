import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle, EntryFilterPreparation } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { long_period: 200, short_period: 5, mid_period: 20 }
const policyParams = { exit_confirm_observations: 2, profit_activation_pct: 5, profit_trailing_pct: 3, max_hold_observations: 63 }
function response(body: any, continuous: boolean, diagnostic?: EntryFilterPreparation, engineVersion = 'strategy-replay-v22') {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-22T12:00:00+09:00', message: 'trend_filter fixture',
    ...(diagnostic ? { entryFilterPreparation: diagnostic } : {}),
    replay: { engineVersion: engineVersion, strategyVersion: 'fixture', sourceInterval: body.interval, replayCadence: 'dailyCloseWithDayBoundary', liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last, dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: JSON.stringify(body.params) },
    backtest: { assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0 },
      trades: [], equityCurve: [], phases: [], overfitWarning: null,
    },
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: any, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'TrendFilterStrategy' }) }
async function setup(page: Page, options: { params?: Params; engineVersion?: string; enabled?: boolean; failSave?: boolean; delay?: Promise<void>; started?: () => void; saveDelay?: Promise<void>; saveStarted?: () => void; diagnostic?: EntryFilterPreparation; toss?: boolean } = {}) {
  const requests: any[] = [], updates: any[] = [], unsafe: string[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { activeBroker: options.toss ? 'toss' : 'kis', chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  page.on('request', (request) => { if (/\/api\/(?:order|trading\/(?:start|stop))/.test(new URL(request.url()).pathname) && request.method() !== 'GET') unsafe.push(request.url()) })
  const strategy = () => ({ id: 'trend_filter_default', name: 'TrendFilterStrategy', enabled: options.enabled ?? false, brokerId: options.toss ? 'toss' : 'kis', brokerAccountId: options.toss ? '1' : '12345678-01', targetSymbols: ['000005'], targetSymbolNames: { '000005': 'Mock trend_filter' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/trend_filter_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    options.saveStarted?.(); if (options.saveDelay) await options.saveDelay
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: '추세 필터 설정 저장 실패' } }); return }
    stored = body.params; await route.fulfill({ json: strategy() })
  })
  await page.route('**/api/strategy/preview{,/continuous}', async (route) => {
    const body = route.request().postDataJSON(), continuous = route.request().url().endsWith('/continuous')
    requests.push(body); options.started?.()
    if (options.delay) await options.delay
    await route.fulfill({ json: response(body, continuous, options.diagnostic, options.engineVersion) })
  })
  await page.goto('/strategy')
  return { requests, updates, unsafe }
}
const labels = ['장기 MA 이탈 연속 확인 수', '수익 보호 시작 (%)', '수익 보호 되밀림 (%)', '최대 보유 관측 수'] as const
const coreLabels = ['장기 MA 관측 수', '단기 MA 관측 수', '중기 MA 관측 수'] as const
const runButton = (page: Page) => card(page).getByRole('button', { name: '미리보기 계산', exact: true })
const saveButton = (page: Page) => card(page).getByRole('button', { name: '변경사항 저장', exact: true })
async function setPolicy(page: Page) {
  for (const [index, value] of ['2', '5', '3', '63'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
}
async function continuous(page: Page) {
  await card(page).getByRole('combobox', { name: /^계산 범위 모드/ }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await card(page).getByLabel('연속 시작일').fill('2025-01-01')
  await card(page).getByLabel('연속 종료일').fill('2025-12-31')
}
async function saveAndWait(page: Page) {
  const response = page.waitForResponse((res) => res.url().endsWith('/api/strategies/trend_filter_default'))
  await saveButton(page).click(); await (await response).finished(); await expect(saveButton(page)).toHaveCount(0)
}

test('omitted options preserve defaults and explain current-inclusive ownership protections', async ({ page }) => {
  const { requests, updates, unsafe } = await setup(page), panel = card(page)
  for (const [i,v] of ['200','5','20'].entries()) await expect(panel.getByLabel(coreLabels[i],{ exact:true })).toHaveValue(v)
  for (const [i,v] of ['1','0','0','0'].entries()) await expect(panel.getByLabel(labels[i],{ exact:true })).toHaveValue(v)
  for (const text of ['산술평균에 현재 관측을 포함', '세 MA가 모두 준비된 뒤', '등호는 진입하지 않습니다', '이탈 확인은 최소 1', '진입 전 가격·현재 봉 전체 고가는 사용하지 않습니다', '옵션이 꺼져 있어도', '나이 불명 복원', '재시작·과거 지표 재적재·핵심 MA 기간 변경', '일수', '신호는 체결이 아니며']) await expect(panel).toContainText(text)
  await runButton(page).click(); await expect(panel.getByTestId('trend-filter-policy-summary')).toContainText('이탈 확인 1 (기본)연속 관측')
  expect(requests[0].params).toMatchObject(legacyParams); for (const key of Object.keys(policyParams)) expect(requests[0].params).not.toHaveProperty(key)
  await panel.getByLabel('1회 수량',{ exact:true }).fill('2'); await saveAndWait(page)
  expect(updates[0].params).toMatchObject(legacyParams); expect(unsafe).toEqual([])
})
test('scoped save reload and both preview payloads retain seven fields and A/B summaries', async ({ page }) => {
  const { requests, updates, unsafe } = await setup(page), panel=card(page)
  await runButton(page).click(); await panel.getByRole('button',{name:'현재 결과를 A로 저장',exact:true}).click()
  await setPolicy(page); await saveAndWait(page)
  expect(updates[0]).toMatchObject({ expectedProfileId:'paper',expectedBrokerId:'kis',expectedBrokerAccountId:'12345678-01',params:{...legacyParams,...policyParams} })
  await page.reload(); for(const [i,v] of ['2','5','3','63'].entries()) await expect(panel.getByLabel(labels[i],{exact:true})).toHaveValue(v)
  await continuous(page); await runButton(page).click(); await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[1].params).toMatchObject({...legacyParams,...policyParams}); expect(requests[1]).not.toHaveProperty('candles')
  await panel.getByRole('button',{name:'현재 결과를 B로 저장',exact:true}).click()
  const ab=panel.getByTestId('strategy-ab-comparison'); await expect(ab).toContainText('A와 B의 추세 필터 파라미터가 다릅니다')
  await expect(ab).toContainText('이탈 확인 2연속 관측'); await expect(ab).toContainText('수익 보호 시작 5% / 되밀림 3%'); expect(unsafe).toEqual([])
})
test('invalid raw drafts block save and both previews without numeric coercion', async ({page})=>{
  const {requests}=await setup(page); await runButton(page).click()
  for(const [label,bads,repair] of [[coreLabels[0],['','NaN','Infinity','0','513','2.5'],'200'],[labels[0],['0','1.5','1000001'],'1'],[labels[1],['1000.1'],'0'],[labels[2],['99.91'],'0'],[labels[3],['-1','1.5','1000001'],'0']] as const){
    for(const raw of bads){ const input=card(page).getByLabel(label,{exact:true}); await input.fill(raw); await expect(input).toHaveValue(raw); await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled(); await expect(card(page).getByRole('button',{name:'현재 결과를 A로 저장',exact:true})).toHaveCount(0); await input.fill(repair); await expect(runButton(page)).toBeEnabled() }
  }
  await card(page).getByLabel(labels[1],{exact:true}).fill('5'); await expect(runButton(page)).toBeDisabled(); await card(page).getByLabel(labels[2],{exact:true}).fill('3'); await expect(runButton(page)).toBeEnabled()
  await continuous(page); await card(page).getByLabel(labels[0],{exact:true}).fill('0'); await expect(runButton(page)).toBeDisabled(); expect(requests).toHaveLength(1)
})
for(const [name,invalid] of [['string',{long_period:'200'}],['null',{exit_confirm_observations:null}],['fraction',{max_hold_observations:1.5}],['boolean',{profit_activation_pct:false}],['unpaired',{profit_activation_pct:5}]] as const){
  test(`stored ${name} remains invalid until explicit repair`,async({page})=>{
    const {updates}=await setup(page,{params:{...legacyParams,...invalid}}); await expect(runButton(page)).toBeDisabled()
    await card(page).getByLabel('1회 수량',{exact:true}).fill('2'); await expect(saveButton(page)).toBeDisabled()
    for(const [i,v] of ['200','5','20'].entries()) { const input=card(page).getByLabel(coreLabels[i],{exact:true}); await input.fill(''); await input.fill(v) }
    await setPolicy(page); await saveAndWait(page); expect(updates[0].params).toMatchObject({...legacyParams,...policyParams})
  })
}
test('legacy engine and damaged saved params remain explicitly unverified',async({page})=>{
  await setup(page,{engineVersion:'strategy-replay-v21'}); await runButton(page).click(); await expect(card(page).getByTestId('trend-filter-policy-summary')).toContainText('이전 또는 미확인 엔진')
  await card(page).getByRole('button',{name:'현재 결과를 A로 저장',exact:true}).click()
  await page.evaluate(()=>{for(const [k,v] of Object.entries(localStorage)){if(!v.includes('trend_filter_default'))continue;const s=JSON.parse(v);if(s.A){s.A.params.exit_confirm_observations=null;localStorage.setItem(k,JSON.stringify(s))}}})
  await page.reload(); await runButton(page).click(); await expect(card(page).getByTestId('strategy-ab-comparison')).toContainText('확인 필요 (null)')
})
test('save failure keeps draft for identical retry',async({page})=>{
  const {updates}=await setup(page,{failSave:true}); await setPolicy(page); await saveButton(page).click(); await expect(page.getByRole('alert').filter({hasText:'추세 필터 설정 저장 실패'})).toBeVisible()
  await expect(card(page).getByLabel(labels[2],{exact:true})).toHaveValue('3'); await saveButton(page).click(); await expect.poll(()=>updates.length).toBe(2); expect(updates[1]).toEqual(updates[0])
})
test('late preview and mobile invalid edits cannot publish stale results',async({page})=>{
  let release!:()=>void,started!:()=>void
  const delay=new Promise<void>(r=>{release=r}),entered=new Promise<void>(r=>{started=r})
  await page.setViewportSize({width:375,height:844}); await setup(page,{delay,started})
  const finished=page.waitForResponse(r=>r.url().endsWith('/api/strategy/preview')); await runButton(page).click(); await entered
  await card(page).getByLabel(labels[0],{exact:true}).fill('0'); release(); await(await finished).finished()
  await expect(card(page).getByRole('button',{name:'현재 결과를 A로 저장',exact:true})).toHaveCount(0)
  expect(await card(page).getByTestId('trend-filter-policy-controls').evaluate(el=>el.scrollWidth>el.clientWidth+1)).toBe(false)
  await page.screenshot({path:'.cache/trend-filter-mobile.png',fullPage:true})
})
test('active strategy locks all seven controls',async({page})=>{await setup(page,{enabled:true});for(const label of [...coreLabels,...labels])await expect(card(page).getByLabel(label,{exact:true})).toBeDisabled()})
test('decimal keyboard and inclusive bounds save finite numeric values',async({page})=>{
  const {updates}=await setup(page,{params:{...legacyParams,profit_activation_pct:5,profit_trailing_pct:3}}),input=card(page).getByLabel(labels[2],{exact:true})
  await input.fill(''); await input.pressSequentially('0.'); await expect(input).toHaveValue('0.'); await input.pressSequentially('5'); await expect(input).toHaveValue('0.5'); await saveAndWait(page); expect(updates[0].params.profit_trailing_pct).toBe(0.5)
  for(const [i,v] of ['1','512','200'].entries()) await card(page).getByLabel(coreLabels[i],{exact:true}).fill(v)
  for(const [i,v] of ['1000000','1000','99.9','1000000'].entries()) await card(page).getByLabel(labels[i],{exact:true}).fill(v)
  await saveAndWait(page); expect(updates[1].params).toMatchObject({long_period:1,short_period:512,mid_period:200,exit_confirm_observations:1000000,profit_activation_pct:1000,profit_trailing_pct:99.9,max_hold_observations:1000000})
})
test('late save completion preserves newer draft for a second save',async({page})=>{
  let release!:()=>void,started!:()=>void
  const saveDelay=new Promise<void>(r=>{release=r}),entered=new Promise<void>(r=>{started=r})
  const {updates}=await setup(page,{saveDelay,saveStarted:started}); await setPolicy(page); const finished=page.waitForResponse(r=>r.url().endsWith('/api/strategies/trend_filter_default'))
  await saveButton(page).click(); await entered; await card(page).getByLabel(labels[3],{exact:true}).fill('64'); release(); await(await finished).finished()
  await expect(card(page).getByLabel(labels[3],{exact:true})).toHaveValue('64'); await expect(saveButton(page)).toBeEnabled(); await saveAndWait(page); expect(updates[1].params.max_hold_observations).toBe(64)
})
