import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Grid from '@mui/material/Grid'
import MenuItem from '@mui/material/MenuItem'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import type { ReplayCalculationMode, ReplayContinuityView } from '../../../api/types'
import type { ContinuousCandleCollection, ContinuousCollectionProgress } from '../model/candlePageCollector'

export function defaultContinuousRange() {
  const end = new Date()
  end.setDate(end.getDate() - 1)
  const iso = (date: Date) => `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
  const start = new Date(end)
  start.setFullYear(start.getFullYear() - 3)
  start.setDate(start.getDate() + 1)
  return { start: iso(start), end: iso(end) }
}

export function ReplayRangeControls({ mode, onModeChange, range, onRangeChange, disabled, error }: {
  mode: ReplayCalculationMode
  onModeChange: (mode: ReplayCalculationMode) => void
  range: { start: string; end: string }
  onRangeChange: (range: { start: string; end: string }) => void
  disabled: boolean
  error: string | null
}) {
  return <Stack spacing={1} sx={{ mb: 1.5, minWidth: 0 }}>
    <TextField select size="small" label="계산 범위 모드" value={mode} disabled={disabled} onChange={(event) => onModeChange(event.target.value as ReplayCalculationMode)} fullWidth>
      <MenuItem value="boundedWindow">단기 구간 · 최근 50/100/200봉</MenuItem>
      <MenuItem value="continuous">연속 일봉 · 날짜 범위</MenuItem>
    </TextField>
    {mode === 'continuous' && <>
      <Grid container spacing={1}>
        <Grid item xs={12} sm={6}><TextField fullWidth size="small" type="date" label="연속 시작일" value={range.start} disabled={disabled} onChange={(event) => onRangeChange({ ...range, start: event.target.value })} InputLabelProps={{ shrink: true }} /></Grid>
        <Grid item xs={12} sm={6}><TextField fullWidth size="small" type="date" label="연속 종료일" value={range.end} disabled={disabled} onChange={(event) => onRangeChange({ ...range, end: event.target.value })} InputLabelProps={{ shrink: true }} /></Grid>
      </Grid>
      <Alert severity="info" sx={{ py: 0.5 }}>
        기본 요청은 최근 3년의 완료 일봉입니다. 시작일 이전 사전자료 252봉을 한 번 준비한 뒤 관측을 시간순으로 이어 계산합니다.
        분기·공급 묶음 경계에서 현금·보유·대기 신호·쿨다운·리스크를 초기화하지 않습니다.
        최대 48페이지·평가 4096봉·보관 4608봉이며 필요한 시작 범위/사전자료를 확보하지 못하면 계산하지 않습니다.
      </Alert>
      {error && <Alert severity="error" sx={{ py: 0.5 }}>{error}</Alert>}
    </>}
  </Stack>
}

export function ContinuousCollectionNotice({ progress, collection }: { progress: ContinuousCollectionProgress | null; collection: ContinuousCandleCollection | null }) {
  if (!collection && !progress) return null
  return <Alert severity={collection && !collection.ready ? 'warning' : 'info'} sx={{ py: 0.75 }} data-testid="continuous-collection-status">
    <Typography variant="body2">연속 자료 수집 {collection?.pages ?? progress?.pages ?? 0}페이지 · 평가 {collection?.evaluationBars ?? progress?.evaluationBars ?? 0}봉 · 엄격한 이전 사전자료 {collection?.historyBars ?? progress?.historyBars ?? 0}/252봉</Typography>
    {collection && <>
      <Typography variant="body2">요청 {collection.requestedStart}~{collection.requestedEnd} · 실제 관측 {collection.actualStart ?? '없음'}~{collection.actualEnd ?? '없음'} · {collection.ready ? '계산 입력 준비' : '계산 보류'}</Typography>
      <Typography variant="caption" display="block">조회 기준 {collection.asOf ?? '정보 없음'}</Typography>
      <Typography variant="caption" display="block">미완료 제외 {collection.excludedUnfinishedBars} · 무효 제외 {collection.excludedInvalidBars} · 중복 제외 {collection.duplicateBars} · 최대 관측 간 달력 간격 {collection.maximumCalendarGapDays}일</Typography>
      <Typography variant="caption" display="block">coverageVerified=false · 거래소 휴장·중간 누락과 기간 coverage는 자동 검증되지 않았습니다. 없는 봉을 합성하거나 부분기간을 3년 성과로 표시하지 않습니다. 시작/종료 차이 또는 관측 간격 7달력일 상한은 앱의 보수적 자료 경계이며 검증된 거래소 달력이 아닙니다. 긴 휴장도 계산이 보류될 수 있습니다.</Typography>
      {collection.issues.map((issue, index) => <Typography variant="body2" key={`${index}:${issue}`}>{issue}</Typography>)}
    </>}
  </Alert>
}

export function calculationModeLabel(mode?: ReplayCalculationMode) {
  return mode === 'continuous' ? '연속 일봉 장부' : mode === 'boundedWindow' ? '단기 구간' : '계산 범위 정보 없음 · 이전 결과'
}

export function ContinuityResults({ value }: { value: ReplayContinuityView }) {
  return <Box data-testid="continuous-replay-continuity" sx={{ minWidth: 0, maxWidth: '100%' }}>
    <Alert severity="info" sx={{ mb: 1 }}>
      연속 공급 {value.chunkCount}묶음 · 평가 {value.evaluationBars}봉 · 사전자료 {value.historyBars}봉을 시작에 한 번 사용했습니다.
      아래 경계는 데이터 공급 경계이며 분기 리셋이나 독립 실험이 아닙니다. 자본·수량 규칙을 유지하는 한 장부에서 전체 공급 후 한 번 종료합니다.
    </Alert>
    <TableContainer sx={{ maxWidth: '100%', maxHeight: 260 }}>
      <Table size="small" aria-label="연속 replay 공급 경계">
        <TableHead><TableRow><TableCell>묶음</TableCell><TableCell>관측 범위</TableCell><TableCell>봉 수</TableCell><TableCell>시작/종료 보유</TableCell><TableCell>경계 대기 신호</TableCell></TableRow></TableHead>
        <TableBody>{value.chunks.map((chunk) => <TableRow key={chunk.index}><TableCell>{chunk.index + 1}</TableCell><TableCell>{chunk.start}~{chunk.end}</TableCell><TableCell>{chunk.bars}</TableCell><TableCell>{chunk.startQuantity} / {chunk.endQuantity}</TableCell><TableCell>{chunk.pendingAtEnd ? '보존' : '없음'}</TableCell></TableRow>)}</TableBody>
      </Table>
    </TableContainer>
  </Box>
}
