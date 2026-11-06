import { useEffect, useMemo, useRef, useState } from 'react'
import type { ChangeEvent } from 'react'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Checkbox from '@mui/material/Checkbox'
import CircularProgress from '@mui/material/CircularProgress'
import FormControlLabel from '@mui/material/FormControlLabel'
import MenuItem from '@mui/material/MenuItem'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { useBrokerChartScope, useCollectBrokerChartData, usePreviewBrokerChartPortfolio } from '../../../api/brokerChartHooks'
import type { BrokerChartCollection, BrokerChartInput, BrokerChartReport, BrokerChartScope } from '../../../shared/api/brokerChartTypes'
import { brokerChartResultsView, parseBrokerChartInput, syntheticBrokerChartInput } from '../model/brokerChartInput'
import { SECTOR_INPUT_MAX_BYTES } from '../model/sectorPortfolioInput'
import { SectorPortfolioResults } from './SectorPortfolioResults'

type Props = { researchOnly: boolean; quoteScopeKey: string; quoteScopeReady: boolean; quoteScopeCurrent?: (key: string) => boolean }
type QuoteRequest = { key: string; token: number }
type Completed = { input: BrokerChartInput; report: BrokerChartReport }
function message(error: unknown) {
  return error && typeof error === 'object' && 'message' in error && typeof error.message === 'string'
    ? error.message : '차트 자료 요청을 완료하지 못했습니다. 입력과 실행 환경을 확인하세요.'
}
function download(value: unknown, name: string) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2)], { type: 'application/json' }))
  const anchor = document.createElement('a')
  anchor.href = url; anchor.download = name; anchor.click(); URL.revokeObjectURL(url)
}

/** Independent price ledger: no strict-input promotion, paper ledger merge or automatic authentication. */
export function BrokerChartPortfolioPanel({ researchOnly, quoteScopeKey, quoteScopeReady, quoteScopeCurrent }: Props) {
  const [text, setText] = useState('')
  const [endDate, setEndDate] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [completed, setCompleted] = useState<Completed | null>(null)
  const [collection, setCollection] = useState<BrokerChartCollection | null>(null)
  const [authorizedScope, setAuthorizedScope] = useState<{ key: string; value: BrokerChartScope } | null>(null)
  const [sourceAck, setSourceAck] = useState<string | null>(null)
  const [quoteRequest, setQuoteRequest] = useState<QuoteRequest | null>(null)
  const scopeMutation = useBrokerChartScope()
  const collectMutation = useCollectBrokerChartData()
  const previewMutation = usePreviewBrokerChartPortfolio()
  const revision = useRef(0)
  const sourceRevision = useRef(0)
  const mounted = useRef(true)
  const running = useRef(false)
  const activeKey = useRef(quoteScopeKey)
  activeKey.current = quoteScopeKey
  const observedKey = useRef(quoteScopeKey)
  const sequence = useRef(0)
  const activeQuote = useRef<QuoteRequest | null>(null)
  const scope = quoteScopeReady && authorizedScope?.key === quoteScopeKey ? authorizedScope.value : null
  const acknowledgedSource = scope !== null && sourceAck === quoteScopeKey
  const native = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const pendingQuote = quoteRequest !== null
  const canAuthenticate = native && !researchOnly
  useEffect(() => {
    mounted.current = true
    return () => { mounted.current = false; revision.current += 1 }
  }, [])
  useEffect(() => {
    if (observedKey.current === quoteScopeKey) return
    observedKey.current = quoteScopeKey
    setAuthorizedScope(null); setSourceAck(null)
    // Existing independent raw-input experiments remain valid historical inputs.
    // Authorization and in-flight responses alone are bound to the live profile publication.
  }, [quoteScopeKey])
  const parsed = useMemo(() => {
    if (!text.trim()) return { input: null, error: null }
    try { return { input: parseBrokerChartInput(text), error: null } }
    catch (failure) { return { input: null, error: message(failure) } }
  }, [text])
  function changeText(next: string, collected: BrokerChartCollection | null = null) {
    revision.current += 1; setText(next); setError(null); setCompleted(null); setCollection(collected)
  }
  function editInput(change: Partial<BrokerChartInput>) {
    if (parsed.input) {
      const next = { ...parsed.input, ...change }
      // Preserve collection evidence beside changed assumptions, with its original input intact.
      changeText(JSON.stringify(next, null, 2), collection)
    }
  }
  function current(ticket: number) { return mounted.current && revision.current === ticket }
  function quoteCurrent(request: QuoteRequest) {
    return mounted.current && activeQuote.current?.token === request.token && activeKey.current === request.key
      && (quoteScopeCurrent?.(request.key) ?? true)
  }
  function startQuote() {
    const request = { key: quoteScopeKey, token: ++sequence.current }
    activeQuote.current = request; setQuoteRequest(request); setSourceAck(null); setError(null)
    return request
  }
  function finishQuote(request: QuoteRequest) {
    if (activeQuote.current?.token === request.token) activeQuote.current = null
    if (mounted.current) setQuoteRequest(previous => previous?.token === request.token ? null : previous)
  }
  async function lookupScope() {
    if (!canAuthenticate || !quoteScopeReady || activeQuote.current || !(quoteScopeCurrent?.(quoteScopeKey) ?? true)) return
    const request = startQuote()
    setAuthorizedScope(null)
    try {
      const value = await scopeMutation.mutateAsync()
      if (quoteCurrent(request)) setAuthorizedScope({ key: request.key, value })
    } catch (failure) { if (quoteCurrent(request)) setError(message(failure)) }
    finally { finishQuote(request) }
  }
  async function collect() {
    if (!canAuthenticate || !quoteScopeReady || !scope || !acknowledgedSource || activeQuote.current
      || !(quoteScopeCurrent?.(quoteScopeKey) ?? true) || (endDate && !/^\d{4}-\d{2}-\d{2}$/.test(endDate))) return
    const ticket = revision.current
    const optionsTicket = sourceRevision.current
    const knownCorporateActions = parsed.input?.knownCorporateActions ?? []
    const request = startQuote()
    try {
      const value = await collectMutation.mutateAsync({ expectedProfileId: scope.profileId, authenticatedQuotesAcknowledged: true,
        evaluationBars: 20, ...(endDate ? { endDate } : {}) })
      if (current(ticket) && optionsTicket === sourceRevision.current && quoteCurrent(request)) {
        const input = { ...value.input, acknowledgedPriceOnly: false,
          knownCorporateActions: [...value.input.knownCorporateActions, ...knownCorporateActions] }
        changeText(JSON.stringify(input, null, 2), value)
      }
    } catch (failure) { if (quoteCurrent(request)) { setAuthorizedScope(null); setError(message(failure)) } }
    finally { finishQuote(request) }
  }
  async function loadFile(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0]; event.target.value = ''
    if (!file) return
    changeText('')
    const ticket = revision.current
    if (file.size > SECTOR_INPUT_MAX_BYTES) { setError('차트 JSON 파일은 1.5MB 이내로 선택하세요.'); return }
    try { const next = await file.text(); if (current(ticket)) changeText(next) }
    catch (failure) { if (current(ticket)) setError(message(failure)) }
  }
  async function run() {
    if (!parsed.input?.acknowledgedPriceOnly || running.current) return
    const input = parsed.input; const ticket = revision.current
    running.current = true; setError(null); setCompleted(null)
    try {
      const report = await previewMutation.mutateAsync(input)
      if (current(ticket)) setCompleted({ input, report })
    } catch (failure) { if (current(ticket)) setError(message(failure)) }
    finally { running.current = false }
  }
  const input = parsed.input
  const source = input?.source
  return <Stack spacing={2} data-testid="broker-chart-panel" sx={{ minWidth: 0 }}>
    <Typography component="h3" variant="h6">차트 가격수익률 근사</Typography>
    <Alert severity="warning">배당재투자 총수익이 아닙니다. 비수정 요청으로 반환된 가격을 사용하며 공급자 조정 범위·과거 기수정 가능성은 검증하지 않았습니다. 비조정 종가 신호·평가에는 분배락 가격 영향이 포함되며, 미확인 배당·분할 등 권리를 정산하지 않습니다. 알려진 기업행위는 계산을 거부합니다.</Alert>
    <Typography variant="body2">독립 가상 USD 장부입니다. 이전 완료봉의 신호·예산과 다음 시가(nextOpen) 정수주, 매도 후 매수·현금 한도·편도 총10/20bp를 가정합니다. 실제 09:50 모의 표시가격·실제 체결과 다르며 검증형 결과나 실제·모의 원장과 자동 합산하지 않습니다.</Typography>
    <Typography variant="body2">브로커에는 비수정(unadjusted) USD 일봉 OHLCV를 요청합니다. SPY와11개 섹터의 사전22봉·평가20봉 예제를 제공하며, 관측 SPY 날짜는 공식 거래일 달력 완전성의 인증이 아닙니다. 초기자금 기본10,000 USD에 실제 잔고를 반영하지 않습니다.</Typography>
    <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 2 }}>
      <Stack spacing={1.5}>
        <Typography component="h4" variant="subtitle2">인증 차트 자료 수집 · 선택 사항</Typography>
        {!canAuthenticate && <Alert severity="info">일반 데스크톱 앱에서만 명시적으로 인증 시세를 수집합니다. 이 실행 환경에서는 JSON 입력과 순수 계산을 사용할 수 있습니다.</Alert>}
        {canAuthenticate && !quoteScopeReady && <Alert severity="info">KIS 또는 Toss 조회 프로필 확인이 완료될 때까지 인증 수집을 보류합니다.</Alert>}
        <TextField label="차트 조회 종료일 (선택)" value={endDate} onChange={event => {
          sourceRevision.current += 1; setEndDate(event.target.value); setSourceAck(null)
        }} size="small" placeholder="YYYY-MM-DD" helperText="비워두면 뉴욕 전일을 조회 기준으로 고정합니다. 최종 평가 종료일은 수신한 SPY 완료봉 기준입니다." />
        <Box><Button variant="outlined" disabled={!canAuthenticate || !quoteScopeReady || pendingQuote} onClick={() => void lookupScope()}>차트 조회 프로필 확인</Button></Box>
        {scope && <Typography variant="body2" sx={{ overflowWrap: 'anywhere' }}>확인한 시세 범위: {scope.profileId} · {scope.provider.toUpperCase()} · {scope.provider === 'kis' ? scope.paper ? '모의' : '실전' : '실전 시세'}</Typography>}
        {scope?.provider === 'kis' && <Typography variant="body2">KIS 기본 매핑은 2026-11-06 공식 마스터 스냅샷의 고정12종목을 사용합니다. 오류·빈 자료면 수집을 중단하며 매핑 최신성을 자동 인증하지 않습니다.</Typography>}
        <FormControlLabel control={<Checkbox checked={acknowledgedSource} disabled={!canAuthenticate || !scope || pendingQuote} onChange={(_, checked) => setSourceAck(checked ? quoteScopeKey : null)} />} label="확인한 프로필의 인증 시세 조회에 동의합니다 (주문·잔고 동의 아님)" />
        <Box><Button variant="outlined" disabled={!canAuthenticate || !scope || !acknowledgedSource || pendingQuote || Boolean(endDate && !/^\d{4}-\d{2}-\d{2}$/.test(endDate))} onClick={() => void collect()} aria-busy={pendingQuote}>차트 자료 20봉 수집</Button></Box>
        {pendingQuote && <Typography role="status" variant="body2">{quoteRequest?.key === quoteScopeKey ? '차트 조회 요청 중입니다.' : '이전 프로필 요청 종료를 기다립니다. 이전 응답은 적용하지 않습니다.'}</Typography>}
      </Stack>
    </Box>
    <Stack direction="row" spacing={1} useFlexGap sx={{ flexWrap: 'wrap' }}>
      <Button component="label" variant="outlined">차트 JSON 불러오기<input hidden type="file" accept=".json,application/json" aria-label="차트 가격 자료 JSON 파일" onChange={event => void loadFile(event)} /></Button>
      <Button variant="outlined" onClick={() => changeText(JSON.stringify(syntheticBrokerChartInput(), null, 2))}>합성 차트 예제 입력</Button>
    </Stack>
    <TextField label="차트 가격 자료 JSON" multiline minRows={5} maxRows={10} fullWidth value={text} onChange={event => changeText(event.target.value)} error={Boolean(parsed.error)} helperText={parsed.error ?? 'BrokerChartInput만 입력합니다. strict 입력·성과 원장과 자동 변환하지 않습니다. 합계500봉·1.5MB 이내.'} slotProps={{ htmlInput: { spellCheck: false } }} />
    {input && <>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
        <TextField label="가격 근사 초기자금 USD" type="number" size="small" value={input.initialCashUsd} onChange={event => editInput({ initialCashUsd: Number(event.target.value) })} />
        <TextField select label="가격 근사 편도 총비용" size="small" value={input.costBps} onChange={event => editInput({ costBps: Number(event.target.value) as 10 | 20 })} sx={{ minWidth: 180 }}>
          <MenuItem value={10}>10bp · 0.1%</MenuItem><MenuItem value={20}>20bp · 0.2% 스트레스</MenuItem>
        </TextField>
      </Stack>
      <Typography variant="body2" sx={{ overflowWrap: 'anywhere' }}>출처 {source?.name} · {source?.brokerId.toUpperCase()} · {source?.synthetic ? '합성 자료' : '브로커 자료라는 입력 선언'} · 수집 {source?.retrievedAt}<br />요청 {source?.adjustment} / {source?.currency} / {source?.interval} · 날짜 기준 {source?.sessionBasis} · {source?.calendarEvidence}<br />사전 {input.history.length}봉 / 평가 {input.evaluation.length}봉 · 알려진 기업행위 {input.knownCorporateActions.length}건 (빈 목록도 사건 부재 확인이 아님)</Typography>
      <FormControlLabel control={<Checkbox checked={input.acknowledgedPriceOnly} onChange={(_, checked) => editInput({ acknowledgedPriceOnly: checked })} />} label="배당 포함 총수익·권리 정산 미검증인 가격 근사 한계를 확인했습니다" />
    </>}
    <Box><Button variant="contained" disabled={!input?.acknowledgedPriceOnly || previewMutation.isPending} onClick={() => void run()} aria-busy={previewMutation.isPending} startIcon={previewMutation.isPending ? <CircularProgress size={16} color="inherit" /> : undefined}>{previewMutation.isPending ? '가격 근사 계산 중' : '가격 근사 비교 계산'}</Button></Box>
    {error && <Alert severity="error">{error}</Alert>}
    {collection && <Stack spacing={1}>
      <Typography variant="body2" sx={{ overflowWrap: 'anywhere' }}>수집 정규화 SHA-256 {collection.collectionHash} · 원시 요청 {collection.receipts.length}페이지{collection.exchangeMappings ? ` · 사용한 거래소 매핑 ${collection.exchangeMappings.length}건` : ''}. hash는 공급원 진위·총수익 인증이 아닙니다. 원본 수집 입력과 현재 계산 가정은 내보내기에서 구분합니다.</Typography>
      {collection.diagnostics.map((diagnostic, index) => <Typography variant="body2" key={index}>{diagnostic}</Typography>)}
      <Box><Button variant="outlined" onClick={() => download({ mode: 'brokerChartPriceApproximation', input, collection }, 'broker-chart-collection.json')}>차트 수집 근거 JSON 다운로드</Button></Box>
    </Stack>}
    {completed && <Stack spacing={2} data-testid="broker-chart-results">
      <Alert severity="warning">가격 근사 결과 · 신호 {completed.report.signalBasis} · 배당재투자 총수익 아님 · 경제적 수익 미검증. 비용은 편도 총{completed.input.costBps}bp 가정입니다.</Alert>
      <Typography variant="body2" sx={{ overflowWrap: 'anywhere' }}>가격 근사 입력 SHA-256 {completed.report.inputHash}. 정규화 DTO와 모드 태그의 hash이며 원본 파일·공급원 진위 인증이 아닙니다.</Typography>
      <SectorPortfolioResults input={brokerChartResultsView(completed.input)} report={completed.report.result} />
      <Box><Button variant="outlined" onClick={() => download({ ...completed, collection }, `broker-chart-price-${completed.report.result.startDate}-${completed.report.result.endDate}.json`)}>가격 근사 입력과 결과 JSON 다운로드</Button></Box>
    </Stack>}
  </Stack>
}
