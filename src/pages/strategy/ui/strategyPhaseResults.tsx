import Alert from '@mui/material/Alert'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Typography from '@mui/material/Typography'
import type { BacktestPhaseDiagnostics, BacktestReport } from '../../../api/types'
import { fmtRoundedKrw } from '../../../shared/lib/formatters'

function validDiagnostics(value: unknown): value is BacktestPhaseDiagnostics {
  if (!value || typeof value !== 'object') return false
  const d = value as Record<string, unknown>
  if (d.version !== 1 || d.policy !== 'sameLedgerObservationSplit') return false
  const counts = ['observedBars', 'heldBars', 'startQuantity', 'endQuantity']
  if (!counts.every((key) => typeof d[key] === 'number' && Number.isSafeInteger(d[key]) && (d[key] as number) >= 0)) return false
  if ((d.heldBars as number) > (d.observedBars as number)) return false
  const money = ['realizedPnlKrw', 'startResidualUnrealizedPnlKrw', 'endResidualUnrealizedPnlKrw', 'dividendNetKrw', 'equityChangeKrw', 'exposurePct']
  return money.every((key) => typeof d[key] === 'number' && Number.isFinite(d[key]))
    && (d.exposurePct as number) >= 0 && (d.exposurePct as number) <= 100
    && typeof d.pendingAtStart === 'boolean' && typeof d.pendingAtEnd === 'boolean'
}

const phaseName = (phase: string) => phase === 'inSample' ? '앞 구간 · 동일 장부' : '뒤 구간 · 동일 장부'

export function StrategyPhaseResults({ report }: { report: BacktestReport }) {
  const phases = report.phases ?? []
  const allDiagnosed = phases.length > 0 && phases.every((phase) => validDiagnostics(phase.diagnostics))
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%', mt: 1 }} data-testid="strategy-phase-results">
    <Alert severity="info" data-testid="phase-policy">
      평가 관측 봉 수로 나눈 동일 장부의 앞/뒤 구간입니다. 경계에서 보유·대기 주문을 초기화하거나 재학습하지 않으며 후반을 독립 검증 성과로 해석하지 않습니다.
      길이·노출·거래 표본이 다른 구간의 단순 수익률 비교는 과최적화 판정이 아닙니다.
    </Alert>
    {allDiagnosed && report.overfitWarning && <Alert severity="warning">{report.overfitWarning}</Alert>}
    {phases.length === 0 ? <Typography variant="body2" color="text.secondary">표시할 구간 관측이 없습니다.</Typography> : <>
      <TableContainer component={Paper} variant="outlined" sx={{ maxWidth: '100%' }}>
        <Table size="small" aria-label="동일 장부 구간 성과">
          <TableHead><TableRow><TableCell>구간</TableCell><TableCell>기간</TableCell><TableCell align="right">수익률</TableCell><TableCell align="right">MDD</TableCell><TableCell align="right">완료 매도/승률</TableCell></TableRow></TableHead>
          <TableBody>{phases.map((phase) => <TableRow key={phase.phase}>
            <TableCell>{phaseName(phase.phase)}</TableCell><TableCell sx={{ overflowWrap: 'anywhere' }}>{phase.start} → {phase.end}</TableCell>
            <TableCell align="right">{phase.returnPct.toFixed(2)}%</TableCell><TableCell align="right">{phase.mddPct.toFixed(2)}%</TableCell>
            <TableCell align="right">{phase.completedTrades}회 / {phase.winRatePct.toFixed(1)}%</TableCell>
          </TableRow>)}</TableBody>
        </Table>
      </TableContainer>
      {phases.map((phase) => {
        const d = phase.diagnostics
        return <Paper key={phase.phase} variant="outlined" sx={{ p: 1, overflowWrap: 'anywhere' }}>
          <Typography variant="caption" fontWeight={700}>{phaseName(phase.phase)} 진단</Typography>
          {!validDiagnostics(d) ? <Alert severity="warning">구간 진단 정보 없음 · 이전 또는 지원하지 않는 결과입니다. 재계산 전에는 표본·경계·손익 귀속을 확인할 수 없습니다.</Alert> : <>
            <Typography variant="body2">관측 {d.observedBars}봉 · 종가 보유 {d.heldBars}봉 · 보유 관측 비율 {d.exposurePct.toFixed(2)}%</Typography>
            <Typography variant="body2">경계 보유 {d.startQuantity} → {d.endQuantity}주 · 대기 주문 {d.pendingAtStart ? '있음' : '없음'} → {d.pendingAtEnd ? '있음' : '없음'}</Typography>
            <Typography variant="body2">실현 {fmtRoundedKrw(d.realizedPnlKrw)} · 잔여 평가손익 {fmtRoundedKrw(d.startResidualUnrealizedPnlKrw)} → {fmtRoundedKrw(d.endResidualUnrealizedPnlKrw)} · 순배당 {fmtRoundedKrw(d.dividendNetKrw)} · 자산 증감 {fmtRoundedKrw(d.equityChangeKrw)}</Typography>
          </>}
          {phase.completedTrades === 0 ? <Typography variant="body2" color="warning.main">완료 매도 0회 · 승률과 실현 거래 표본으로 전략 효능을 평가할 수 없습니다. 보유 평가손익은 남을 수 있습니다.</Typography>
            : phase.completedTrades < 20 ? <Typography variant="body2" color="warning.main">완료 매도 {phase.completedTrades}회 · 작은 표본입니다. 20회 이상도 신뢰성이나 독립성을 보장하지 않습니다.</Typography> : null}
        </Paper>
      })}
      <Typography variant="caption" color="text.secondary">완료 매도는 부분매도도 각각 셉니다. 경계를 넘어 보유한 매도는 매도 수량에 배분된 원래 진입원가·매수비용을 차감한 실현손익이 매도 구간에 귀속됩니다. 자산 증감 = 실현손익 + 잔여 평가손익 증감 + 순배당이며 표시 반올림 차이가 있을 수 있습니다. 저장 A/B의 진단은 원래 서버 결과이며 축약된 거래·곡선으로 다시 계산하지 않습니다.</Typography>
    </>}
  </Stack>
}
