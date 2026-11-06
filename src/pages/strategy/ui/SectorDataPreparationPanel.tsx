import { useEffect, useMemo, useRef, useState } from 'react'
import type { ChangeEvent } from 'react'
import Accordion from '@mui/material/Accordion'
import AccordionDetails from '@mui/material/AccordionDetails'
import AccordionSummary from '@mui/material/AccordionSummary'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Checkbox from '@mui/material/Checkbox'
import FormControlLabel from '@mui/material/FormControlLabel'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'
import { useCollectSectorData, useSectorQuoteScope, useValidateSectorData } from '../../../api/sectorDataHooks'
import type { SectorDataValidation, SectorQuoteScope } from '../../../shared/api/sectorDataTypes'
import type { SectorPortfolioInput } from '../../../shared/api/sectorPortfolioTypes'
import { parseSectorDataPreparation, SECTOR_DATA_MAX_BYTES, SECTOR_DATA_SYMBOLS, sectorDataTemplate } from '../model/sectorDataPreparation'

function message(error: unknown): string {
  return error && typeof error === 'object' && 'message' in error && typeof error.message === 'string'
    ? error.message : '자료 준비 요청을 완료하지 못했습니다. 입력과 실행 환경을 확인하세요.'
}
function downloadJson(value: unknown, filename: string) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2)], { type: 'application/json' }))
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
}

export function SectorDataPreparationPanel({ researchOnly, onPrepared, onInvalidate }: {
  researchOnly: boolean
  onPrepared: (input: SectorPortfolioInput) => void
  onInvalidate: () => void
}) {
  const [text, setText] = useState('')
  const [scope, setScope] = useState<SectorQuoteScope | null>(null)
  const [acknowledged, setAcknowledged] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [report, setReport] = useState<SectorDataValidation | null>(null)
  const scopeMutation = useSectorQuoteScope()
  const collectMutation = useCollectSectorData()
  const validateMutation = useValidateSectorData()
  const revision = useRef(0)
  const running = useRef(false)
  const mounted = useRef(true)
  const native = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const pending = scopeMutation.isPending || collectMutation.isPending || validateMutation.isPending
  useEffect(() => {
    mounted.current = true
    return () => { mounted.current = false; revision.current += 1 }
  }, [])
  const parsed = useMemo(() => {
    if (!text.trim()) return { input: null, error: null }
    try { return { input: parseSectorDataPreparation(text), error: null } }
    catch (failure) { return { input: null, error: failure instanceof Error ? failure.message : 'JSON을 확인하세요.' } }
  }, [text])

  function invalidate() {
    revision.current += 1
    setError(null)
    setReport(null)
    setAcknowledged(false)
    onInvalidate()
    return revision.current
  }
  function changeText(next: string) { invalidate(); setText(next) }
  function current(ticket: number) { return mounted.current && revision.current === ticket }
  async function loadFile(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0]
    event.target.value = ''
    if (!file) return
    const ticket = invalidate()
    setText('')
    if (file.size > SECTOR_DATA_MAX_BYTES) { setError('자료 JSON 파일은 1.5MB 이내로 선택하세요.'); return }
    try { const next = await file.text(); if (current(ticket)) changeText(next) }
    catch (failure) { if (current(ticket)) setError(message(failure)) }
  }
  async function lookupScope() {
    if (researchOnly || !native || running.current) return
    running.current = true
    const ticket = invalidate()
    setScope(null)
    try {
      const next = await scopeMutation.mutateAsync()
      if (current(ticket)) setScope(next)
    } catch (failure) { if (current(ticket)) setError(message(failure)) }
    finally { running.current = false }
  }
  async function collect() {
    if (researchOnly || !native || !scope || scope.provider !== 'kis' || !acknowledged || !parsed.input || running.current) return
    const input = parsed.input
    const expectedProfileId = scope.profileId
    running.current = true
    const ticket = invalidate()
    try {
      const bundle = await collectMutation.mutateAsync({ expectedProfileId, authenticatedQuotesAcknowledged: true, manifest: input.bundle.manifest })
      if (current(ticket)) changeText(JSON.stringify({ ...input, bundle, verification: {
        ...input.verification, coverageVerified: false, adjustmentVerified: false, corporateActionsVerified: false,
        adjustmentEvidence: '', corporateActionsEvidence: '',
      } }, null, 2))
    } catch (failure) { if (current(ticket)) { setScope(null); setError(message(failure)) } }
    finally { running.current = false }
  }
  async function validate() {
    if (!parsed.input || running.current) return
    const input = parsed.input
    running.current = true
    const ticket = invalidate()
    try {
      const next = await validateMutation.mutateAsync(input)
      if (current(ticket)) setReport(next)
    } catch (failure) { if (current(ticket)) setError(message(failure)) }
    finally { running.current = false }
  }

  return <Accordion disableGutters data-testid="sector-data-preparation" sx={{ minWidth: 0 }}>
    <AccordionSummary expandIcon={<ExpandMoreIcon />}><Typography component="h3">자료 수집·검증 준비</Typography></AccordionSummary>
    <AccordionDetails sx={{ px: { xs: 1, sm: 2 }, minWidth: 0 }}><Stack spacing={2}>
      <Typography variant="body2">고정12종목: {SECTOR_DATA_SYMBOLS.join(', ')}. 비교 목표는 미국 뉴욕 시간(America/New_York)의 완료된 평가30거래일과 앞선22거래일입니다. 달력30일과 다릅니다. 공식 거래일 목록·마감 여부·근거를 직접 입력하세요.</Typography>
      <Alert severity="warning">수집 완료만으로 계산 가능 판정을 내리지 않습니다. 비조정 시가·종가(MODP=0)와 조정 신호종가(MODP=1)를 분리합니다. MODP=1의 배당 포함 총수익 의미와 KIS 거래소 코드 매핑은 별도 근거 확인이 필요합니다. 기업행위·달력·조정 근거가 없으면 계산을 보류합니다.</Alert>
      <Typography variant="body2">템플릿에는 확인되지 않은 선언을 모두 false로 둡니다. sessionDates는 22개 사전 거래일 뒤 평가 거래일을 이어 적고, exchangeMappings의 NAS/NYS/AMS와 evidence를 종목별로 확인하세요. verification의 확인 값·adjustmentEvidence·corporateActionsEvidence는 사용자의 근거 선언이며 공급원의 자동 인증이 아닙니다.</Typography>
      <Stack direction="row" spacing={1} useFlexGap sx={{ flexWrap: 'wrap' }}>
        <Button variant="outlined" onClick={() => downloadJson(sectorDataTemplate(), 'sector-data-template.json')}>미확인 자료 템플릿 다운로드</Button>
        <Button variant="outlined" onClick={() => changeText(JSON.stringify(sectorDataTemplate(), null, 2))}>템플릿 편집 시작</Button>
        <Button component="label" variant="outlined">자료 준비 JSON 불러오기<input type="file" accept=".json,application/json" aria-label="자료 준비 JSON 파일" hidden onChange={event => void loadFile(event)} /></Button>
      </Stack>
      <TextField label="자료 준비 JSON" multiline minRows={5} maxRows={12} fullWidth value={text} onChange={event => changeText(event.target.value)} error={Boolean(parsed.error)} helperText={parsed.error ?? 'bundle(원본·조정 자료/수집근거), verification, initialCashUsd, costBps를 함께 입력합니다. 최대1.5MB, 로컬 검증만으로는 시세를 조회하지 않습니다.'} slotProps={{ htmlInput: { spellCheck: false } }} />
      {parsed.input && <Typography variant="body2" sx={{ overflowWrap: 'anywhere' }}>
        기준일 {parsed.input.bundle.manifest.endDate || '미입력'} · 선언 거래일 {parsed.input.bundle.manifest.sessionDates.length}개 · 평가 {Math.max(0, parsed.input.bundle.manifest.sessionDates.length - 22)}거래일 · 자료 {parsed.input.bundle.series.length}/12종목<br />
        달력 출처 {parsed.input.bundle.manifest.calendarSource || '미입력'} · 달력 근거 {parsed.input.bundle.manifest.calendarEvidence || '미입력'} · 수집 {parsed.input.bundle.retrievedAt || '미수집'} · {parsed.input.bundle.manifest.synthetic ? '합성 자료 선언' : '실제 자료 선언(자동 인증 아님)'}<br />
        수집 페이지 {parsed.input.bundle.pages?.length ?? 0}개 · rowsHash는 보관한 시세 필드의 canonical SHA-256이며 HTTP 원문 해시가 아닙니다. 해시는 재현성 비교용이며 출처·진위 인증이 아닙니다.
      </Typography>}
      {!researchOnly && <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 2 }}>
        <Typography component="h4" variant="subtitle2">일반 데스크톱 앱의 명시적 KIS 시세 수집</Typography>
        <Typography variant="body2">새 수집 후 확인 선언과 조정·기업행위 근거는 초기화합니다. 새 자료를 검토한 뒤 다시 확인하세요.</Typography>
        {!native && <Typography variant="body2">KIS 수집은 일반 데스크톱 앱에서만 가능합니다. 브라우저에서는 파일을 불러와 검증하세요.</Typography>}
        <Button variant="outlined" onClick={() => void lookupScope()} disabled={!native || pending}>KIS 조회 범위 확인</Button>
        {scope && <Typography variant="body2" sx={{ overflowWrap: 'anywhere', mt: 1 }}>선택 범위: {scope.provider} · {scope.paper ? '모의' : '실전'} · 프로필 {scope.profileId}. 계좌 변경 시 다시 확인하세요.</Typography>}
        <FormControlLabel control={<Checkbox checked={acknowledged} disabled={!native || !scope || pending} onChange={event => { invalidate(); setAcknowledged(event.target.checked) }} />} label="선택 프로필의 인증된 KIS 시세 조회 요청을 승인합니다. 주문은 하지 않습니다." />
        <Box><Button variant="contained" onClick={() => void collect()} disabled={!native || !scope || scope.provider !== 'kis' || !acknowledged || !parsed.input || pending}>KIS 12종목 자료 수집</Button></Box>
      </Box>}
      {researchOnly && <Alert severity="info">독립 연구 모드에서는 KIS 수집·계좌 범위 조회를 제공하지 않습니다. 파일 자료의 순수 검증과 계산만 요청합니다.</Alert>}
      <Stack direction="row" spacing={1} useFlexGap sx={{ flexWrap: 'wrap' }}>
        <Button variant="contained" onClick={() => void validate()} disabled={!parsed.input || pending} aria-busy={pending}>{pending ? '자료 준비 요청 중' : '파일 자료 검증'}</Button>
        <Button variant="outlined" disabled={!parsed.input || pending} onClick={() => parsed.input && downloadJson(parsed.input, 'sector-data-bundle.json')}>자료·근거 JSON 다운로드</Button>
      </Stack>
      {error && <Alert severity="error">{error}</Alert>}
      {report && <Box data-testid="sector-data-validation">
        <Alert severity={report.ready && report.input ? 'success' : 'warning'}>{report.ready && report.input ? '자료 검증 통과 · 아래 버튼으로 비교 입력에 적용하세요.' : '평가 보류 · 확인할 근거와 자료를 보완하세요. 수익률은 계산하지 않았습니다.'}</Alert>
        <Typography variant="caption" component="p" sx={{ my: 1, overflowWrap: 'anywhere' }}>bundle SHA-256 {report.bundleHash}<br />verification SHA-256 {report.verificationHash}</Typography>
        {report.diagnostics.length > 0 && <Box component="ul" sx={{ pl: 3 }}>{report.diagnostics.map((item, index) => <li key={`${item.code}-${index}`}>{item.code}: {item.message}</li>)}</Box>}
        {report.warnings.length > 0 && <Alert severity="warning"><Box component="ul" sx={{ m: 0, pl: 2 }}>{report.warnings.map((item, index) => <li key={`${index}-${item}`}>{item}</li>)}</Box></Alert>}
        <Box><Button variant="outlined" sx={{ mt: 2 }} disabled={!parsed.input || pending} onClick={() => parsed.input && downloadJson({ request: parsed.input, report }, 'sector-data-evidence.json')}>자료와 검증 보고서 다운로드</Button></Box>
        <Button variant="contained" sx={{ mt: 2 }} disabled={!report.ready || !report.input || pending} onClick={() => report.ready && report.input && onPrepared(report.input)}>검증된 자료를 비교 입력으로 적용</Button>
      </Box>}
    </Stack></AccordionDetails>
  </Accordion>
}
