import { useEffect, useMemo, useState } from 'react'

import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import Grid from '@mui/material/Grid'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import DeleteOutlineIcon from '@mui/icons-material/DeleteOutline'
import ScienceOutlinedIcon from '@mui/icons-material/ScienceOutlined'

import type {
  BacktestReport,
  BrokerId,
  ReplayMetadata,
  SimulationAssumptions,
  StrategyExperimentSnapshot,
} from '../../../api/types'
import {
  clearExperimentSlots,
  loadExperimentSlots,
  saveExperimentSlot,
  type ExperimentScope,
} from '../model/experimentStore'

export function defaultSimulationAssumptions(isOverseas: boolean): SimulationAssumptions {
  return {
    initialCapitalKrw: 10_000_000,
    feeBps: isOverseas ? 10 : 1.5,
    taxBps: isOverseas ? 0 : 20,
    slippageBps: isOverseas ? 10 : 5,
    exchangeRateKrw: 1450,
    maxPositionRatio: 0.3,
    volatilitySizingEnabled: false,
    riskPerTradeBps: 100,
    atrStopMultiplier: 2,
    dailyLossLimitKrw: 1_000_000,
    inSamplePercent: 70,
  }
}

export function rebaseDefaultMarketCosts(
  current: SimulationAssumptions,
  previousIsOverseas: boolean,
  nextIsOverseas: boolean,
): SimulationAssumptions {
  if (previousIsOverseas === nextIsOverseas) return current
  const previous = defaultSimulationAssumptions(previousIsOverseas)
  const next = defaultSimulationAssumptions(nextIsOverseas)
  return {
    ...current,
    feeBps: current.feeBps === previous.feeBps ? next.feeBps : current.feeBps,
    taxBps: current.taxBps === previous.taxBps ? next.taxBps : current.taxBps,
    slippageBps: current.slippageBps === previous.slippageBps ? next.slippageBps : current.slippageBps,
  }
}

type AssumptionsProps = {
  value: SimulationAssumptions
  onChange: (value: SimulationAssumptions) => void
  disabled?: boolean
}

const assumptionFields: Array<{
  key: keyof SimulationAssumptions
  label: string
  min: number
  max: number
  step: number
  helper?: string
}> = [
  { key: 'initialCapitalKrw', label: '초기자본(원)', min: 10_000, max: 1_000_000_000_000, step: 100_000 },
  { key: 'feeBps', label: '수수료(bps)', min: 0, max: 1000, step: 0.1 },
  { key: 'taxBps', label: '세금(bps)', min: 0, max: 1000, step: 0.1 },
  { key: 'slippageBps', label: '슬리피지(bps)', min: 0, max: 1000, step: 0.1 },
  { key: 'exchangeRateKrw', label: 'USD/KRW 환율', min: 100, max: 10_000, step: 1 },
  { key: 'maxPositionRatio', label: '종목 최대 비중', min: 0.01, max: 1, step: 0.01, helper: '0.3 = 자본의 30%' },
  { key: 'dailyLossLimitKrw', label: '일일 손실 한도(원)', min: 0, max: 1_000_000_000_000, step: 10_000 },
  { key: 'inSamplePercent', label: '학습 구간(%)', min: 50, max: 100, step: 5, helper: '나머지는 out-of-sample 검증' },
]

export function SimulationAssumptionsEditor({ value, onChange, disabled }: AssumptionsProps) {
  const updateNumber = (key: keyof SimulationAssumptions, next: number) => {
    onChange({ ...value, [key]: Number.isFinite(next) ? next : 0 })
  }

  return (
    <Box sx={{ p: 1.5, border: 1, borderColor: 'divider', borderRadius: 1 }}>
      <Stack direction="row" alignItems="center" spacing={0.75} sx={{ mb: 1.25 }}>
        <ScienceOutlinedIcon fontSize="small" color="primary" />
        <Typography variant="caption" fontWeight={700}>백테스트 실행 가정</Typography>
      </Stack>
      <Grid container spacing={1.25}>
        {assumptionFields.map((field) => (
          <Grid item xs={12} sm={6} md={3} key={field.key}>
            <TextField
              size="small"
              type="number"
              label={field.label}
              value={value[field.key] as number}
              onChange={(event) => updateNumber(field.key, Number(event.target.value))}
              disabled={disabled}
              fullWidth
              helperText={field.helper}
              inputProps={{ min: field.min, max: field.max, step: field.step }}
            />
          </Grid>
        ))}
      </Grid>
      <Typography variant="caption" color="text.secondary" display="block" sx={{ mt: 1 }}>
        신호 수와 별도로 공통 cooldown·손절 재진입 금지·포지션 비중·비용·현금 조건을 통과한 주문/체결만 성과에 반영합니다.
      </Typography>
    </Box>
  )
}

function fmtMoney(value: number) {
  return `${Math.round(value).toLocaleString('ko-KR')}원`
}

function fmtPct(value: number) {
  return `${value > 0 ? '+' : ''}${value.toFixed(2)}%`
}

function Metric({ label, value, tone }: { label: string; value: string; tone?: 'positive' | 'negative' }) {
  return (
    <Paper variant="outlined" sx={{ p: 1.25, height: '100%' }}>
      <Typography variant="caption" color="text.secondary" display="block">{label}</Typography>
      <Typography
        variant="subtitle2"
        fontWeight={700}
        color={tone === 'positive' ? 'success.main' : tone === 'negative' ? 'error.main' : 'text.primary'}
      >
        {value}
      </Typography>
    </Paper>
  )
}

function EquityCurve({ report }: { report: BacktestReport }) {
  const points = report.equityCurve
  const polyline = useMemo(() => {
    if (points.length === 0) return ''
    const values = points.map((point) => point.equityKrw)
    const min = Math.min(...values)
    const max = Math.max(...values)
    const span = Math.max(1, max - min)
    return values.map((value, index) => {
      const x = points.length === 1 ? 0 : index / (points.length - 1) * 100
      const y = 38 - ((value - min) / span) * 34
      return `${x.toFixed(2)},${y.toFixed(2)}`
    }).join(' ')
  }, [points])

  if (points.length === 0) return null
  return (
    <Box
      role="img"
      aria-label={`자산 곡선 ${fmtMoney(points[0].equityKrw)}에서 ${fmtMoney(points[points.length - 1].equityKrw)}`}
      sx={{ width: '100%', height: 190, border: 1, borderColor: 'divider', borderRadius: 1, p: 1 }}
    >
      <svg viewBox="0 0 100 40" preserveAspectRatio="none" width="100%" height="100%">
        <line x1="0" x2="100" y1="38" y2="38" stroke="currentColor" opacity="0.18" />
        <polyline points={polyline} fill="none" stroke="currentColor" strokeWidth="1.2" vectorEffect="non-scaling-stroke" />
      </svg>
    </Box>
  )
}

type ResultsProps = {
  report: BacktestReport
  replay: ReplayMetadata
  strategyId: string
  brokerId: BrokerId
  brokerAccountId: string | null
  symbol: string
  params: Record<string, unknown>
  orderQuantity: number
  generatedAt: string
}

export function StrategyResearchResults({
  report,
  replay,
  strategyId,
  brokerId,
  brokerAccountId,
  symbol,
  params,
  orderQuantity,
  generatedAt,
}: ResultsProps) {
  const scope = useMemo<ExperimentScope>(() => ({
    brokerId,
    brokerAccountId,
    strategyId,
    symbol,
  }), [brokerAccountId, brokerId, strategyId, symbol])
  const scopeKey = `${brokerId}:${brokerAccountId ?? 'none'}:${strategyId}:${symbol}`
  const [slots, setSlots] = useState<Partial<Record<'A' | 'B', StrategyExperimentSnapshot>>>({})
  const [storageError, setStorageError] = useState<string | null>(null)

  useEffect(() => {
    setSlots(loadExperimentSlots(scope))
  }, [scopeKey]) // eslint-disable-line react-hooks/exhaustive-deps -- serialized scope is the persistence boundary

  const saveSlot = (slot: 'A' | 'B') => {
    const snapshot: StrategyExperimentSnapshot = {
      id: `${replay.inputHash}:${slot}`,
      slot,
      strategyId,
      strategyVersion: replay.strategyVersion,
      brokerId,
      brokerAccountId,
      symbol,
      params,
      orderQuantity,
      replay,
      backtest: report,
      generatedAt,
    }
    if (!saveExperimentSlot(scope, snapshot)) {
      setStorageError('브라우저 저장 공간이 부족하거나 비활성화되어 A/B 결과를 저장하지 못했습니다.')
      return
    }
    setStorageError(null)
    setSlots(loadExperimentSlots(scope))
  }

  const clearSlots = () => {
    clearExperimentSlots(scope)
    setSlots({})
  }

  const comparable = slots.A && slots.B
    ? slots.A.replay.dataSource === slots.B.replay.dataSource
      && slots.A.replay.sourceInterval === slots.B.replay.sourceInterval
      && slots.A.replay.dataStart === slots.B.replay.dataStart
      && slots.A.replay.dataEnd === slots.B.replay.dataEnd
    : true
  const summary = report.summary

  return (
    <Stack spacing={1.5} data-testid="strategy-backtest-results">
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} alignItems={{ xs: 'stretch', sm: 'center' }}>
        <Typography variant="subtitle2" fontWeight={700} sx={{ mr: 'auto' }}>주문 가능성 반영 백테스트</Typography>
        <Button size="small" variant="outlined" onClick={() => saveSlot('A')}>현재 결과를 A로 저장</Button>
        <Button size="small" variant="outlined" onClick={() => saveSlot('B')}>현재 결과를 B로 저장</Button>
        {(slots.A || slots.B) && (
          <Button size="small" color="inherit" startIcon={<DeleteOutlineIcon />} onClick={clearSlots}>A/B 초기화</Button>
        )}
      </Stack>
      {storageError && <Alert severity="error" sx={{ py: 0.5 }}>{storageError}</Alert>}

      <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
        <Chip size="small" label={`${replay.sourceInterval} · ${replay.replayCadence}`} />
        <Chip size="small" label={`live ${replay.liveCadenceSeconds}초 tick`} variant="outlined" />
        <Chip size="small" label={`warmup ${replay.warmupCount}봉`} variant="outlined" />
        <Chip size="small" label={`${replay.dataStart} → ${replay.dataEnd}`} variant="outlined" />
        <Chip size="small" label={`재현 ID ${replay.inputHash.slice(0, 10)}`} color="primary" variant="outlined" />
      </Stack>

      <Alert severity="info" sx={{ py: 0.5 }}>
        봉 종가 replay와 실제 10초 tick의 시간축을 분리 표시합니다. deterministic/무미래참조 fixture를 사용하지만 봉 내부 가격 경로와 provider pending·체결 지연은 재현하지 않으며, 슬리피지는 체결가격 영향만 근사합니다.
      </Alert>
      {report.overfitWarning && <Alert severity="warning" sx={{ py: 0.5 }}>{report.overfitWarning}</Alert>}

      <Grid container spacing={1}>
        <Grid item xs={6} sm={4} md={2}><Metric label="누적 수익률" value={fmtPct(summary.cumulativeReturnPct)} tone={summary.cumulativeReturnPct >= 0 ? 'positive' : 'negative'} /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="MDD" value={fmtPct(-summary.mddPct)} tone="negative" /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="승률" value={`${summary.winRatePct.toFixed(1)}%`} /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="손익비" value={summary.profitFactor == null ? '—' : summary.profitFactor.toFixed(2)} /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="Turnover" value={`${summary.turnoverPct.toFixed(1)}%`} /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="Exposure" value={`${summary.exposurePct.toFixed(1)}%`} /></Grid>
      </Grid>

      <Alert severity={summary.blockedOrderCount > 0 ? 'warning' : 'success'} sx={{ py: 0.5 }}>
        원시 신호 {summary.signalCount}개 · 주문 가능 {summary.orderEligibleCount}개 · 체결 가정 {summary.filledOrderCount}개 · 차단 {summary.blockedOrderCount}개
      </Alert>

      <EquityCurve report={report} />

      {report.phases.length > 0 && (
        <TableContainer component={Paper} variant="outlined">
          <Table size="small" aria-label="학습 및 검증 구간 성과">
            <TableHead><TableRow><TableCell>구간</TableCell><TableCell>기간</TableCell><TableCell align="right">수익률</TableCell><TableCell align="right">MDD</TableCell><TableCell align="right">거래/승률</TableCell></TableRow></TableHead>
            <TableBody>
              {report.phases.map((phase) => (
                <TableRow key={phase.phase}>
                  <TableCell>{phase.phase === 'inSample' ? 'In-sample' : 'Out-of-sample'}</TableCell>
                  <TableCell>{phase.start} → {phase.end}</TableCell>
                  <TableCell align="right">{fmtPct(phase.returnPct)}</TableCell>
                  <TableCell align="right">{phase.mddPct.toFixed(2)}%</TableCell>
                  <TableCell align="right">{phase.completedTrades}회 / {phase.winRatePct.toFixed(1)}%</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      {(slots.A || slots.B) && (
        <Box sx={{ p: 1.25, border: 1, borderColor: 'divider', borderRadius: 1 }} data-testid="strategy-ab-comparison">
          <Typography variant="caption" fontWeight={700} display="block" sx={{ mb: 0.75 }}>저장된 A/B 실험 비교</Typography>
          {!comparable && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 데이터 source/봉/기간이 달라 직접 비교할 수 없습니다.</Alert>}
          <Grid container spacing={1}>
            {(['A', 'B'] as const).map((slot) => {
              const saved = slots[slot]
              return (
                <Grid item xs={12} sm={6} key={slot}>
                  <Paper variant="outlined" sx={{ p: 1 }}>
                    <Typography variant="caption" fontWeight={700}>실험 결과 {slot}</Typography>
                    {saved ? (
                      <Typography variant="body2">
                        {fmtPct(saved.backtest.summary.cumulativeReturnPct)} · MDD {saved.backtest.summary.mddPct.toFixed(2)}% · {saved.replay.dataStart}~{saved.replay.dataEnd}
                      </Typography>
                    ) : <Typography variant="body2" color="text.secondary">저장된 결과 없음</Typography>}
                  </Paper>
                </Grid>
              )
            })}
          </Grid>
        </Box>
      )}

      <TableContainer component={Paper} variant="outlined" sx={{ maxHeight: 320 }}>
        <Table size="small" stickyHeader aria-label="백테스트 거래 목록">
          <TableHead><TableRow><TableCell>시각</TableCell><TableCell>상태</TableCell><TableCell>방향</TableCell><TableCell align="right">수량</TableCell><TableCell align="right">체결가</TableCell><TableCell align="right">비용</TableCell><TableCell align="right">실현손익</TableCell><TableCell>사유</TableCell></TableRow></TableHead>
          <TableBody>
            {report.trades.slice(0, 100).map((trade, index) => (
              <TableRow key={`${trade.time}:${trade.side}:${index}`}>
                <TableCell>{trade.time}</TableCell>
                <TableCell><Chip size="small" label={trade.status === 'filled' ? '체결 가정' : '차단'} color={trade.status === 'filled' ? 'success' : 'warning'} variant="outlined" /></TableCell>
                <TableCell>{trade.side === 'buy' ? '매수' : '매도'}</TableCell>
                <TableCell align="right">{trade.quantity.toLocaleString('ko-KR')}</TableCell>
                <TableCell align="right">{trade.fillPrice == null ? '—' : trade.fillPrice.toLocaleString('ko-KR')}</TableCell>
                <TableCell align="right">{fmtMoney(trade.costKrw)}</TableCell>
                <TableCell align="right">{trade.realizedPnlKrw == null ? '—' : fmtMoney(trade.realizedPnlKrw)}</TableCell>
                <TableCell>{trade.blockedReason ?? trade.reason}</TableCell>
              </TableRow>
            ))}
            {report.trades.length === 0 && <TableRow><TableCell colSpan={8}>거래 신호가 없습니다.</TableCell></TableRow>}
          </TableBody>
        </Table>
      </TableContainer>
      {report.trades.length > 100 && <Typography variant="caption" color="text.secondary">UI 반응성을 위해 최근 계산의 거래 목록은 처음 100건만 표시합니다.</Typography>}
    </Stack>
  )
}
