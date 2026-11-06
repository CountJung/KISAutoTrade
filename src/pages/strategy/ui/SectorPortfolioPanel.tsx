import { useEffect, useMemo, useRef, useState } from 'react'
import type { ChangeEvent } from 'react'
import Accordion from '@mui/material/Accordion'
import AccordionDetails from '@mui/material/AccordionDetails'
import AccordionSummary from '@mui/material/AccordionSummary'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'
import { usePreviewSectorPortfolio } from '../../../api/sectorPortfolioHooks'
import type { SectorPortfolioInput, SectorPortfolioReport } from '../../../shared/api/sectorPortfolioTypes'
import { parseSectorPortfolioInput, SECTOR_INPUT_MAX_BYTES, syntheticSectorPortfolioInput } from '../model/sectorPortfolioInput'
import { SectorPortfolioResults } from './SectorPortfolioResults'
import { SectorPaperComparison } from './SectorPaperComparison'
import { SectorDataPreparationPanel } from './SectorDataPreparationPanel'

type Completed = { input: SectorPortfolioInput; report: SectorPortfolioReport }

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') return error.message
  return '자료 계산을 완료하지 못했습니다. 입력과 서버 연결을 확인하세요.'
}

export function SectorPortfolioPanel({ researchOnly = false }: { researchOnly?: boolean }) {
  const [text, setText] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [completed, setCompleted] = useState<Completed | null>(null)
  const mutation = usePreviewSectorPortfolio()
  const revision = useRef(0)
  const running = useRef(false)
  const mounted = useRef(true)
  useEffect(() => {
    mounted.current = true
    return () => { mounted.current = false; revision.current += 1 }
  }, [])
  const parsed = useMemo(() => {
    if (!text.trim()) return { input: null, error: null }
    try { return { input: parseSectorPortfolioInput(text), error: null } }
    catch (failure) { return { input: null, error: failure instanceof Error ? failure.message : 'JSON을 확인하세요.' } }
  }, [text])

  function changeText(next: string) {
    revision.current += 1
    setText(next)
    setError(null)
    setCompleted(null)
  }

  async function loadFile(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0]
    event.target.value = ''
    if (!file) return
    const ticket = ++revision.current
    setCompleted(null)
    setError(null)
    if (file.size > SECTOR_INPUT_MAX_BYTES) { setError('JSON 파일은 1.5MB 이내로 선택하세요.'); return }
    try {
      const content = await file.text()
      if (revision.current === ticket) changeText(content)
    } catch (failure) { if (revision.current === ticket) setError(errorMessage(failure)) }
  }

  async function run() {
    if (!parsed.input || running.current) return
    const ticket = revision.current
    const input = parsed.input
    running.current = true
    setError(null)
    setCompleted(null)
    try {
      const report = await mutation.mutateAsync(input)
      if (mounted.current && ticket === revision.current) setCompleted({ input, report })
    } catch (failure) {
      if (mounted.current && ticket === revision.current) setError(errorMessage(failure))
    } finally { running.current = false }
  }

  function download() {
    if (!completed) return
    const url = URL.createObjectURL(new Blob([JSON.stringify(completed, null, 2)], { type: 'application/json' }))
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = `sector-portfolio-${completed.report.startDate}-${completed.report.endDate}.json`
    anchor.click()
    URL.revokeObjectURL(url)
  }

  return (
    <Accordion disableGutters defaultExpanded={researchOnly} sx={{ mb: 3, minWidth: 0 }} data-testid="sector-portfolio-panel">
      <AccordionSummary expandIcon={<ExpandMoreIcon />}><Typography component="h2" variant="subtitle1" sx={{ fontWeight: 700 }}>섹터 ETF 3전략 비교</Typography></AccordionSummary>
      <AccordionDetails sx={{ p: { xs: 1.5, sm: 2.5 }, minWidth: 0 }}>
        <Stack spacing={2}>
          <Alert severity="info">계좌 연결 없이 입력 자료로 계산합니다. 기본형 최대3개×30%, 방어형 같은 종목×15%/30%, 공격형 최대2개×47.5%를 같은 기간·자본·비용으로 비교합니다. 이 패널은 자동매매를 시작하거나 주문하지 않습니다.</Alert>
          <Typography variant="body2">전일 완료 자료로 선정·예산을 고정하고 다음 거래일 시가에 체결하는 일봉 근사입니다. 현재 모의투자의 09:50 조회 표시가격과 다릅니다. 30일 결과만으로 우월성이나 급락 방어를 확정할 수 없습니다.</Typography>
          <Typography variant="body2">SPY와 11개 섹터의 조정 신호종가·비조정 시가/종가, 출처·수집시각·거래일 달력을 JSON으로 입력하세요. 사전자료 최소22봉과 평가자료를 분리하며 합계500봉·1.5MB 이내입니다. 날짜·조정·기업행위·누락 검증 실패는 0% 성과로 표시하지 않습니다.</Typography>
          <SectorDataPreparationPanel researchOnly={researchOnly} onPrepared={input => changeText(JSON.stringify(input, null, 2))} onInvalidate={() => changeText('')} />
          <Stack direction="row" spacing={1} useFlexGap sx={{ flexWrap: 'wrap' }}>
            <Button component="label" variant="outlined">자료 JSON 불러오기<input type="file" accept=".json,application/json" aria-label="섹터 자료 JSON 파일" hidden onChange={event => void loadFile(event)} /></Button>
            <Button variant="outlined" onClick={() => changeText(JSON.stringify(syntheticSectorPortfolioInput(), null, 2))}>합성 예제 입력</Button>
          </Stack>
          <TextField label="섹터 비교 자료 JSON" value={text} onChange={event => changeText(event.target.value)} multiline minRows={5} maxRows={10} fullWidth error={Boolean(parsed.error)} helperText={parsed.error ?? '파일은 서버나 계좌에 저장하지 않습니다. 상세 금융·달력 검증은 Rust 엔진에서 수행합니다.'} slotProps={{ htmlInput: { spellCheck: false } }} />
          <Box><Button variant="contained" disabled={!parsed.input || mutation.isPending} onClick={() => void run()} aria-busy={mutation.isPending} startIcon={mutation.isPending ? <CircularProgress size={16} color="inherit" /> : undefined}>{mutation.isPending ? '세 전략 계산 중' : '세 전략 비교 계산'}</Button></Box>
          {error && <Alert severity="error">{error}</Alert>}
          {completed && <>
            <SectorPortfolioResults report={completed.report} input={completed.input} />
            <Box><Button variant="outlined" onClick={download}>입력과 결과 JSON 다운로드</Button></Box>
            <SectorPaperComparison report={completed.report} costBps={completed.input.costBps ?? 10} />
          </>}
        </Stack>
      </AccordionDetails>
    </Accordion>
  )
}
