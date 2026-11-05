import { useMemo, useState } from 'react'
import Accordion from '@mui/material/Accordion'
import AccordionDetails from '@mui/material/AccordionDetails'
import AccordionSummary from '@mui/material/AccordionSummary'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TableContainer from '@mui/material/TableContainer'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'
import type { SectorPortfolioReport, SectorStrategy } from '../../../shared/api/sectorPortfolioTypes'
import { sectorStrategyLabel } from './SectorPortfolioResults'

type PaperMark = { date: string; strategy: SectorStrategy; equityUsd: number; priceSource: string }
type PaperSnapshot = { initialCashUsd: number; costBps: 10 | 20; startDate: string; marks: PaperMark[] }

function parseSnapshot(text: string): PaperSnapshot {
  if (text.length > 100_000) throw new Error('모의 원장 요약은 100KB 이내로 입력하세요.')
  const value: unknown = JSON.parse(text)
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('모의 원장 요약은 JSON 객체여야 합니다.')
  const data = value as Partial<PaperSnapshot>
  if (!Number.isFinite(data.initialCashUsd) || (data.initialCashUsd ?? 0) <= 0 || ![10, 20].includes(data.costBps ?? 0) || typeof data.startDate !== 'string'
    || !Array.isArray(data.marks) || data.marks.length === 0 || data.marks.length > 500) {
    throw new Error('initialCashUsd, costBps(10 또는 20), startDate, marks(1~500개)가 필요합니다.')
  }
  const keys = new Set<string>()
  for (const mark of data.marks) {
    if (!mark || typeof mark !== 'object' || !Object.prototype.hasOwnProperty.call(sectorStrategyLabel, mark.strategy)
      || typeof mark.date !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(mark.date)
      || !Number.isFinite(mark.equityUsd) || mark.equityUsd < 0
      || typeof mark.priceSource !== 'string' || !mark.priceSource.trim() || mark.priceSource.length > 500) {
      throw new Error('각 marks에는 날짜·전략(base/defensive/aggressive/spy/cash)·유한 equityUsd·priceSource가 필요합니다.')
    }
    const key = `${mark.strategy}:${mark.date}`
    if (keys.has(key)) throw new Error('같은 전략·날짜의 모의 평가가 중복되어 있습니다.')
    keys.add(key)
  }
  return data as PaperSnapshot
}

export function SectorPaperComparison({ report, costBps }: { report: SectorPortfolioReport; costBps: number }) {
  const [text, setText] = useState('')
  const parsed = useMemo(() => {
    if (!text.trim()) return null
    try {
      const snapshot = parseSnapshot(text)
      if (snapshot.costBps !== costBps) throw new Error('비용 가정이 달라 직접 비교할 수 없습니다. 모의 원장과 코드 재생의 편도 costBps를 일치시키세요.')
      if (snapshot.startDate !== report.startDate || report.portfolios.some(item => item.summary.initialCashUsd !== snapshot.initialCashUsd)) {
        throw new Error('직접 비교하려면 모의 원장의 시작일과 초기자금이 이 재생 결과와 같아야 합니다. 다른 시작 실험은 합산하지 않습니다.')
      }
      return { snapshot, error: null }
    } catch (error) {
      return { snapshot: null, error: error instanceof Error ? error.message : '모의 원장 요약을 확인하세요.' }
    }
  }, [text, report, costBps])
  return (
    <Accordion disableGutters>
      <AccordionSummary expandIcon={<ExpandMoreIcon />}><Typography>현재 모의 원장과 대조 (선택)</Typography></AccordionSummary>
      <AccordionDetails>
        <Alert severity="info" sx={{ mb: 2 }}>직접 입력한 모의 평가와 같은 날짜의 코드 재생을 대조합니다. 원장의 09:50 조회 표시가격과 일봉 다음 시가 근사는 다릅니다. 차이는 전략 효과만으로 해석할 수 없습니다. 계좌·원장은 조회하거나 수정하지 않습니다.</Alert>
        <Typography variant="caption" component="p" sx={{ mb: 1, overflowWrap: 'anywhere' }}>
          형식: {'{"initialCashUsd":10000,"costBps":10,"startDate":"평가 시작 YYYY-MM-DD","marks":[{"date":"YYYY-MM-DD","strategy":"base","equityUsd":10000,"priceSource":"09:50 표시가격·출처·조회시각"}]}'}
        </Typography>
        <TextField label="모의 원장 평가 JSON" multiline minRows={3} maxRows={8} fullWidth value={text} onChange={event => setText(event.target.value)} error={Boolean(parsed?.error)} helperText={parsed?.error ?? '실제 기록이 있을 때만 입력하세요. 예제 수익률이나 모의 기록을 자동 생성하지 않습니다.'} />
        {parsed?.snapshot && <TableContainer component={Box} tabIndex={0} aria-label="모의 원장과 코드 재생 차이" sx={{ mt: 2 }}>
          <Table size="small" sx={{ minWidth: 600 }}>
            <TableHead><TableRow>{['날짜', '전략', '모의 USD', '재생 USD', '모의−재생 USD', '모의 가격 출처'].map(label => <TableCell key={label}>{label}</TableCell>)}</TableRow></TableHead>
            <TableBody>{parsed.snapshot.marks.map(mark => {
              const equity = report.portfolios.find(item => item.strategy === mark.strategy)?.daily.find(day => day.date === mark.date)?.equityUsd
              return <TableRow key={`${mark.strategy}-${mark.date}`}><TableCell>{mark.date}</TableCell><TableCell>{sectorStrategyLabel[mark.strategy]}</TableCell>
                <TableCell>{mark.equityUsd.toFixed(2)}</TableCell><TableCell>{equity?.toFixed(2) ?? '같은 평가일 없음'}</TableCell>
                <TableCell>{equity == null ? '비교 불가' : (mark.equityUsd - equity).toFixed(2)}</TableCell><TableCell sx={{ overflowWrap: 'anywhere', maxWidth: 280 }}>{mark.priceSource}</TableCell>
              </TableRow>
            })}</TableBody>
          </Table>
        </TableContainer>}
      </AccordionDetails>
    </Accordion>
  )
}
