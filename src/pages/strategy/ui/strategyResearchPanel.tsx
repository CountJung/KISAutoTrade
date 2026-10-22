import { useEffect, useMemo, useState } from 'react'

import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import Grid from '@mui/material/Grid'
import MenuItem from '@mui/material/MenuItem'
import FormControlLabel from '@mui/material/FormControlLabel'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Switch from '@mui/material/Switch'
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
  EntryFilterPreparation,
  VolatilityExpansionPreparation,
  BrokerId,
  ReplayMetadata,
  ReplayEventTime,
  ReplayCalculationMode,
  ReplayContinuityView,
  SimulationAssumptions,
  StrategyExperimentSnapshot,
} from '../../../api/types'
import {
  clearExperimentSlots,
  loadExperimentSlots,
  saveExperimentSlot,
  type ExperimentScope,
} from '../model/experimentStore'
import { AccountingResults, ComparisonResults, AccountingAssumptionSummary, accountingComparisonKey, SimulationAccountingEditor } from './strategyAccountingPanel'
import { calculationModeLabel, ContinuityResults } from './continuousReplayPanel'
import { FailedBreakoutPolicySummary } from './failedBreakoutControls'
import { FailedBreakoutPreparation } from './failedBreakoutPreparation'
import { RsiPolicySummary } from './rsiControls'
import { StrongClosePolicySummary } from './strongCloseControls'
import { StrongClosePreparation } from './strongClosePreparation'
import { VolatilityExpansionSummary } from './volatilityExpansionControls'
import { VolatilityPreparation } from './volatilityExpansionPreparation'
import { MaCrossEntrySummary } from './maCrossEntryControls'
import { MaCrossEntryPreparation } from './maCrossEntryPreparation'
import { MeanReversionPolicySummary } from './meanReversionControls'
import { DeviationPolicySummary } from './deviationControls'
import { MomentumEntrySummary } from './momentumEntryControls'
import { ConsecutiveEntrySummary } from './consecutiveEntryControls'
import { ConsecutiveEntryPreparation } from './consecutiveEntryPreparation'
import { LthMinuteTimingPanel, lthTimingComparisonKey } from './lthMinuteTimingPanel'
import { fmtRoundedKrw as fmtMoney } from '../../../shared/lib/formatters'

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
    executionModel: 'sameClose',
    intrabarExitPolicy: 'unverified',
    stopLossPct: 0,
    takeProfitPct: 0,
    dividendPriceBasis: 'unverified',
    costSource: 'estimated',
    costNote: '',
    buyFeeBps: null,
    sellFeeBps: null,
    minimumFeeNative: 0,
    fxScenario: [],
    dividendScenario: [],
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
  isOverseas: boolean
  continuousMode?: boolean
  onValidationChange: (error: string | null) => void
  onDraftChange: () => void
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

export function SimulationAssumptionsEditor({ value, onChange, disabled, isOverseas, onValidationChange, onDraftChange, continuousMode = false }: AssumptionsProps) {
  const updateNumber = (key: keyof SimulationAssumptions, next: number) => {
    onChange({ ...value, [key]: Number.isFinite(next) ? next : 0 })
  }

  return (
    <Box sx={{ p: 1.5, border: 1, borderColor: 'divider', borderRadius: 1 }}>
      <Stack direction="row" alignItems="center" spacing={0.75} sx={{ mb: 1.25 }}>
        <ScienceOutlinedIcon fontSize="small" color="primary" />
        <Typography variant="caption" fontWeight={700}>백테스트 실행 가정</Typography>
      </Stack>
      <Grid container spacing={1.25} sx={{ mb: 1.25 }}>
        <Grid item xs={12} sm={6}>
          <TextField select size="small" label="체결 모델" value={value.executionModel ?? 'sameClose'} fullWidth disabled={disabled}
            onChange={(event) => onChange({ ...value, executionModel: event.target.value as SimulationAssumptions['executionModel'] })}>
            <MenuItem value="sameClose">신호 봉 종가 체결 가정</MenuItem>
            <MenuItem value="nextOpen">다음 관측 봉 시가 체결 가정</MenuItem>
          </TextField>
        </Grid>
        <Grid item xs={12} sm={6}>
          <TextField select size="small" label="봉 내부 청산 정책" value={value.intrabarExitPolicy ?? 'unverified'} fullWidth disabled={disabled}
            onChange={(event) => onChange({ ...value, intrabarExitPolicy: event.target.value as SimulationAssumptions['intrabarExitPolicy'] })}>
            <MenuItem value="unverified">봉 내부 경로 미검증 · 시가 갭 청산만</MenuItem>
            <MenuItem value="stopFirst">손절 우선 · 보수적 모의 청산</MenuItem>
          </TextField>
        </Grid>
        <Grid item xs={12} sm={6}>
          <TextField size="small" type="number" label="연구 손절(%)" value={value.stopLossPct ?? 0} fullWidth disabled={disabled}
            onChange={(event) => updateNumber('stopLossPct', Number(event.target.value))}
            inputProps={{ min: 0, max: 99.9, step: 0.1 }} helperText="0 = 비활성 · 백테스트 전용" />
        </Grid>
        <Grid item xs={12} sm={6}>
          <TextField size="small" type="number" label="연구 익절(%)" value={value.takeProfitPct ?? 0} fullWidth disabled={disabled}
            onChange={(event) => updateNumber('takeProfitPct', Number(event.target.value))}
            inputProps={{ min: 0, max: 1000, step: 0.1 }} helperText="0 = 비활성 · 백테스트 전용" />
        </Grid>
      </Grid>
      <Alert severity="info" sx={{ mb: 1.25 }}>
        {value.executionModel === 'nextOpen'
          ? '종가에서 확인한 신호는 다음 관측 봉의 시가로 체결을 가정합니다. 마지막 봉 신호는 다음 봉이 없으면 미체결로 남습니다.'
          : '종가로 신호를 확인한 같은 봉의 종가 체결은 낙관적 가정입니다.'}
        {' '}{value.intrabarExitPolicy === 'stopFirst'
          ? '손절·익절이 같은 봉에서 함께 닿으면 손절을 먼저 적용합니다. OHLC만으로 실제 도달 순서를 확인할 수 없습니다.'
          : '봉 내부 손절·익절 경로를 검증하지 않습니다. 연구 손절·익절은 갭 시가 청산만 가정하며 봉 내부 닿음으로 체결하지 않습니다.'}
        {' '}실제 주문·자동매매 설정에는 적용되지 않습니다.
      </Alert>
      <Grid container spacing={1.25}>
        {assumptionFields.map((field) => (
          <Grid item xs={12} sm={6} md={3} key={field.key}>
            <TextField
              size="small"
              type="number"
              label={continuousMode && field.key === 'inSamplePercent' ? '연속 구간 경계(%)' : field.label}
              value={value[field.key] as number}
              onChange={(event) => updateNumber(field.key, Number(event.target.value))}
              disabled={disabled}
              fullWidth
              helperText={continuousMode && field.key === 'inSamplePercent' ? '동일 장부의 전후 구간 · 독립 학습/검증 아님' : field.helper}
              inputProps={{ min: field.min, max: field.max, step: field.step }}
            />
          </Grid>
        ))}
      </Grid>
      <SimulationAccountingEditor value={value} onChange={onChange} disabled={disabled} isOverseas={isOverseas} onValidationChange={onValidationChange} onDraftChange={onDraftChange} />
      <FormControlLabel
        control={<Switch checked={value.volatilitySizingEnabled} disabled={disabled} onChange={(event) => onChange({ ...value, volatilitySizingEnabled: event.target.checked })} />}
        label="백테스트 ATR 기반 수량 산정"
      />
      {value.volatilitySizingEnabled && (
        <Grid container spacing={1.25}>
          <Grid item xs={12} sm={6}>
            <TextField size="small" type="number" label="거래당 위험(bps)" value={value.riskPerTradeBps} onChange={(event) => updateNumber('riskPerTradeBps', Number(event.target.value))} disabled={disabled} fullWidth inputProps={{ min: 0, max: 10000, step: 1 }} />
          </Grid>
          <Grid item xs={12} sm={6}>
            <TextField size="small" type="number" label="ATR 손절 배수" value={value.atrStopMultiplier} onChange={(event) => updateNumber('atrStopMultiplier', Number(event.target.value))} disabled={disabled} fullWidth inputProps={{ min: 0.1, max: 20, step: 0.1 }} />
          </Grid>
          <Grid item xs={12}>
            <Typography variant="caption" color="text.secondary">
              이 백테스트에만 적용합니다. 같은 주기의 관측 15봉으로 이전 종가를 포함한 True Range 14개를 준비합니다.
              자료 부족 또는 ATR 0이면 신규 매수를 차단하며, 보유분 매도는 ATR 준비로 차단하지 않습니다. 누락 봉을 보간하지 않습니다.
            </Typography>
          </Grid>
        </Grid>
      )}
      <Typography variant="caption" color="text.secondary" display="block" sx={{ mt: 1 }}>
        신호 수와 별도로 공통 cooldown·손절 재진입 금지·포지션 비중·비용·현금 조건을 통과한 주문/체결만 성과에 반영합니다.
      </Typography>
    </Box>
  )
}

function executionModelLabel(assumptions: SimulationAssumptions) {
  if (assumptions.executionModel === 'nextOpen') return '다음 관측 봉 시가'
  if (assumptions.executionModel === 'sameClose') return '신호 봉 종가'
  return '체결 모델 정보 없음 · 이전 결과'
}

function executionAssumptionsLabel(assumptions: SimulationAssumptions) {
  const policy = assumptions.intrabarExitPolicy === 'stopFirst' ? '손절 우선' : assumptions.intrabarExitPolicy === 'unverified' ? '봉 내부 미검증' : '봉 내부 정책 정보 없음'
  const stop = assumptions.stopLossPct == null ? '정보 없음' : `${assumptions.stopLossPct}%`
  const take = assumptions.takeProfitPct == null ? '정보 없음' : `${assumptions.takeProfitPct}%`
  return `${executionModelLabel(assumptions)} · ${policy} · 손절 ${stop} · 익절 ${take}`
}

function intrabarPathLabel(status: string) {
  if (status === 'gapAtOpen') return '시가 갭 청산 가정'
  if (status === 'ambiguousStopFirst') return '손절·익절 동시 도달 · 손절 우선 가정'
  if (status === 'ohlcUnverified') return 'OHLC 경로 미검증'
  return status
}

function eventTimeLabel(time?: ReplayEventTime) {
  if (!time) return '정보 없음'
  const phase = { open: '시가', close: '종가', intrabar: '봉 내부' }[time.phase]
  return `${time.barTime} · ${phase}`
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

function AtrPreparationNotice({ report }: { report: BacktestReport }) {
  if (!report.assumptions.volatilitySizingEnabled) return null
  const preparation = report.atrPreparation
  if (!preparation) {
    return (
      <Alert severity="warning" data-testid="backtest-atr-preparation">
        이전 응답에는 ATR 준비 정보가 없습니다. ATR 기반 수량 산정의 준비 여부를 확인하려면 다시 계산하세요.
      </Alert>
    )
  }
  const hadUnreadyBars = !preparation.readyAtStart || !preparation.readyAtEnd || preparation.unreadyEvaluationBars > 0
  return (
    <Alert severity={hadUnreadyBars ? 'warning' : 'info'} data-testid="backtest-atr-preparation">
      <Typography variant="body2" fontWeight={700}>ATR 기반 수량 산정 준비</Typography>
      <Typography variant="body2">
        ATR 기간 {preparation.period} · 사전 관측 {preparation.historyBars}봉 · 시작 {preparation.readyAtStart ? '준비' : '미준비'} · 종료 {preparation.readyAtEnd ? '준비' : '미준비'}
      </Typography>
      <Typography variant="body2">
        ATR 미준비 평가 {preparation.unreadyEvaluationBars}봉 · 최초 준비 {preparation.firstReadyTime ?? '확인되지 않음'}
      </Typography>
      <Typography variant="caption" display="block">
        전략 지표 준비 상태와 별도입니다. 자료 부족 또는 ATR 0이면 신규 매수를 차단하며 보유분 매도는 ATR 준비로 차단하지 않습니다.
        True Range는 이전 종가와 관측 OHLC로 계산하고 누락 봉은 보간하지 않습니다.
      </Typography>
    </Alert>
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
  calculationMode?: ReplayCalculationMode
  continuity?: ReplayContinuityView
  entryFilterPreparation?: EntryFilterPreparation
  volatilityExpansionPreparation?: VolatilityExpansionPreparation
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
  calculationMode,
  continuity,
  entryFilterPreparation,
  volatilityExpansionPreparation,
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
  const isLeveragedReplay = strategyId.startsWith('leveraged_trend_hold')
  const assessment = replay.assessment
  const performanceUnavailable = assessment?.model === 'dailyDiagnostic'
    || assessment?.performanceStatus === 'notEvaluable'
    || (isLeveragedReplay && !assessment)
  const isIntradaySample = assessment?.model === 'intradaySample'

  useEffect(() => {
    setSlots(loadExperimentSlots(scope))
  }, [scopeKey]) // eslint-disable-line react-hooks/exhaustive-deps -- serialized scope is the persistence boundary

  const saveSlot = (slot: 'A' | 'B') => {
    if (performanceUnavailable) return
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
      calculationMode,
      continuity,
      entryFilterPreparation,
      volatilityExpansionPreparation,
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
      && slots.A.replay.assessment?.model === slots.B.replay.assessment?.model
    : true
  const differentTiming = slots.A && slots.B && lthTimingComparisonKey(slots.A.replay.assessment?.minuteTiming) !== lthTimingComparisonKey(slots.B.replay.assessment?.minuteTiming)
  const differentExecution = slots.A && slots.B && (
    slots.A.replay.engineVersion !== slots.B.replay.engineVersion
    || slots.A.calculationMode !== slots.B.calculationMode
    || slots.A.backtest.assumptions.executionModel !== slots.B.backtest.assumptions.executionModel
    || slots.A.backtest.assumptions.intrabarExitPolicy !== slots.B.backtest.assumptions.intrabarExitPolicy
    || slots.A.backtest.assumptions.stopLossPct !== slots.B.backtest.assumptions.stopLossPct
    || slots.A.backtest.assumptions.takeProfitPct !== slots.B.backtest.assumptions.takeProfitPct
    || accountingComparisonKey(slots.A.backtest.assumptions) !== accountingComparisonKey(slots.B.backtest.assumptions)
  )
  const summary = report.summary
  const metadata = (
    <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
      <Chip size="small" label={calculationModeLabel(calculationMode)} color={calculationMode === 'continuous' ? 'primary' : 'default'} variant="outlined" />
      <Chip size="small" label={`엔진 ${replay.engineVersion}`} variant="outlined" />
      <Chip size="small" label={`체결 ${executionModelLabel(report.assumptions)}`} color="primary" variant="outlined" sx={{ maxWidth: '100%', height: 'auto', '& .MuiChip-label': { whiteSpace: 'normal', py: 0.5 } }} />
      <Chip size="small" label={`${replay.sourceInterval} · ${replay.replayCadence}`} />
      <Chip size="small" label={`live ${replay.liveCadenceSeconds}초 tick`} variant="outlined" />
      <Chip size="small" label={`사전 자료 ${replay.warmupCount}봉`} variant="outlined" />
      <Chip size="small" label={`${replay.dataStart} → ${replay.dataEnd}`} variant="outlined" />
      <Chip size="small" label={`재현 ID ${replay.inputHash.slice(0, 10)}`} color="primary" variant="outlined" />
    </Stack>
  )
  const assessmentNotice = (assessment || performanceUnavailable) && (
    <Alert severity="warning" data-testid="replay-assessment-notice" sx={{ py: 0.75 }}>
      <Typography variant="body2" fontWeight={700}>
        {performanceUnavailable
          ? assessment?.model === 'intradaySample' ? '성과 평가 불가 · 분봉 자료 진단' : '성과 평가 불가 · 일봉/자료 진단'
          : '분봉 표본 모의 결과'}
      </Typography>
      <Typography variant="body2">
        {assessment
          ? `일봉 맥락 ${assessment.dailyContextBars}봉 · 실제 장중 관측 ${assessment.intradayBars}봉`
          : '이전 응답에는 평가 모델 정보가 없어 성과를 표시하거나 A/B에 저장할 수 없습니다. 다시 계산하세요.'}
      </Typography>
      {assessment && (
        <Typography variant="body2">
          {performanceUnavailable
            ? assessment.model === 'intradaySample'
              ? '유효한 분봉 관측이 없어 장중 신호·체결·투자 성과를 평가할 수 없습니다.'
              : '일봉 OHLC만으로 장중 진입·관측 횟수·청산 경로를 재현할 수 없어 수익률과 거래 성과를 계산하지 않습니다.'
            : '최대 200봉의 분봉 표본입니다. 3개월 성과를 대표하지 않으며 실제 10초 관측과 다릅니다.'}
          {' '}시각의 시간대·DST와 거래 세션은 검증되지 않았습니다.
        </Typography>
      )}
      {assessment?.limitations.map((limitation, index) => (
        <Typography key={`${index}:${limitation}`} variant="caption" display="block">{limitation}</Typography>
      ))}
      {isLeveragedReplay && replay.sourceInterval === '1m' && <LthMinuteTimingPanel value={assessment?.minuteTiming} />}
    </Alert>
  )

  if (performanceUnavailable) {
    return (
      <Stack spacing={1.5} data-testid="strategy-replay-diagnostic">
        {assessmentNotice}
        {metadata}
      </Stack>
    )
  }

  return (
    <Stack spacing={1.5} data-testid="strategy-backtest-results">
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} alignItems={{ xs: 'stretch', sm: 'center' }}>
        <Typography variant="subtitle2" fontWeight={700} sx={{ mr: 'auto' }}>{isIntradaySample ? '분봉 표본 모의 결과' : '주문 가능성 반영 백테스트'}</Typography>
        <Button size="small" variant="outlined" onClick={() => saveSlot('A')}>현재 결과를 A로 저장</Button>
        <Button size="small" variant="outlined" onClick={() => saveSlot('B')}>현재 결과를 B로 저장</Button>
        {(slots.A || slots.B) && (
          <Button size="small" color="inherit" startIcon={<DeleteOutlineIcon />} onClick={clearSlots}>A/B 초기화</Button>
        )}
      </Stack>
      {storageError && <Alert severity="error" sx={{ py: 0.5 }}>{storageError}</Alert>}

      {assessmentNotice}
      {metadata}
      {strategyId.startsWith('failed_breakout') && <><FailedBreakoutPolicySummary params={params} /><FailedBreakoutPreparation value={entryFilterPreparation} params={params} /></>}
      {strategyId.startsWith('volatility_expansion') && <><VolatilityExpansionSummary params={params} /><VolatilityPreparation value={volatilityExpansionPreparation} /></>}
      {strategyId.startsWith('strong_close') && <><StrongClosePolicySummary params={params} /><StrongClosePreparation value={entryFilterPreparation} params={params} /></>}
      {strategyId.startsWith('rsi') && <RsiPolicySummary params={params} />}
      {strategyId.startsWith('ma_cross') && <><MaCrossEntrySummary params={params} /><MaCrossEntryPreparation value={entryFilterPreparation} params={params} /></>}
      {strategyId.startsWith('mean_reversion') && <MeanReversionPolicySummary params={params} />}
      {strategyId.startsWith('deviation') && <DeviationPolicySummary params={params} />}
      {strategyId.startsWith('momentum') && <MomentumEntrySummary params={params} />}
      {strategyId.startsWith('consecutive_move') && <ConsecutiveEntryPreparation value={entryFilterPreparation} params={params} />}
      <AtrPreparationNotice report={report} />

      <Alert severity="warning" sx={{ py: 0.5 }} data-testid="backtest-execution-assumptions">
        <Typography variant="body2">{executionAssumptionsLabel(report.assumptions)}</Typography>
        <Typography variant="caption" display="block">
          신호 확인과 체결 시점은 봉 라벨과 시가·종가·봉 내부 단계로 표시하며 정확한 거래소 체결 시각을 뜻하지 않습니다.
          실제 10초 관측, 봉 내부 가격 경로, provider pending·체결 지연은 검증하지 않습니다. 슬리피지는 가격 영향만 근사합니다.
        </Typography>
        {(report.executionWarnings ?? []).map((warning, index) => <Typography key={`${index}:${warning}`} variant="body2">{warning}</Typography>)}
        {!report.executionWarnings && <Typography variant="body2">이전 응답에는 체결 진단 정보가 없습니다. 다시 계산해 확인하세요.</Typography>}
      </Alert>
      {report.overfitWarning && <Alert severity="warning" sx={{ py: 0.5 }}>{report.overfitWarning}</Alert>}

      <Grid container spacing={1}>
        <Grid item xs={6} sm={4} md={2}><Metric label="누적 수익률" value={fmtPct(summary.cumulativeReturnPct)} tone={summary.cumulativeReturnPct >= 0 ? 'positive' : 'negative'} /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="MDD" value={fmtPct(-summary.mddPct)} tone="negative" /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="승률" value={`${summary.winRatePct.toFixed(1)}%`} /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="손익비" value={summary.profitFactor == null ? '—' : summary.profitFactor.toFixed(2)} /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="Turnover" value={`${summary.turnoverPct.toFixed(1)}%`} /></Grid>
        <Grid item xs={6} sm={4} md={2}><Metric label="보유 관측 봉 비율" value={`${summary.exposurePct.toFixed(1)}%`} /></Grid>
      </Grid>

      <Alert severity={summary.blockedOrderCount > 0 || (summary.unfilledOrderCount ?? 0) > 0 || (summary.ambiguousBarCount ?? 0) > 0 ? 'warning' : 'success'} sx={{ py: 0.5 }}>
        원시 신호 {summary.signalCount}개 · 주문 가능 {summary.orderEligibleCount}개 · 체결 가정 {summary.filledOrderCount}개 · 차단 {summary.blockedOrderCount}개 · 미체결 {summary.unfilledOrderCount == null ? '정보 없음' : `${summary.unfilledOrderCount}개`}
        {summary.ambiguousBarCount != null && ` · 손절·익절 동시 도달 ${summary.ambiguousBarCount}봉`}
      </Alert>

      {continuity && <ContinuityResults value={continuity} />}
      {calculationMode === 'continuous' && <Alert severity="warning" data-testid="continuous-phase-policy">앞/뒤 구간은 현금·보유·대기 주문·guard·risk를 보존한 한 연속 장부의 구간 표시입니다. 경계에서 재학습/리셋하지 않으며 후반을 독립 검증 성과로 해석하지 않습니다.</Alert>}
      <AccountingResults report={report} />
      <ComparisonResults report={report} />
      <EquityCurve report={report} />

      {report.phases.length > 0 && (
        <TableContainer component={Paper} variant="outlined">
          <Table size="small" aria-label={calculationMode === 'continuous' ? '연속 장부 구간 성과' : '학습 및 검증 구간 성과'}>
            <TableHead><TableRow><TableCell>구간</TableCell><TableCell>기간</TableCell><TableCell align="right">수익률</TableCell><TableCell align="right">MDD</TableCell><TableCell align="right">거래/승률</TableCell></TableRow></TableHead>
            <TableBody>
              {report.phases.map((phase) => (
                <TableRow key={phase.phase}>
                  <TableCell>{calculationMode === 'continuous' ? phase.phase === 'inSample' ? '앞 구간 · 동일 장부' : '뒤 구간 · 동일 장부' : phase.phase === 'inSample' ? 'In-sample' : 'Out-of-sample'}</TableCell>
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
          {differentTiming && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 시각 진단 버전·봉 표기·시간대 또는 검증 가정이 다릅니다. 진단 정보가 없는 이전 결과와 직접 비교하지 마세요.</Alert>}
          {strategyId.startsWith('failed_breakout') && slots.A && slots.B && JSON.stringify(slots.A.params) !== JSON.stringify(slots.B.params) && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 돌파 파라미터가 다릅니다. 기준 가격·완료 일봉 조건·수익 보호·보유 관측 상한을 확인하세요.</Alert>}
          {strategyId.startsWith('volatility_expansion') && slots.A && slots.B && (JSON.stringify(slots.A.params) !== JSON.stringify(slots.B.params) || slots.A.volatilityExpansionPreparation?.cadence !== slots.B.volatilityExpansionPreparation?.cadence) && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 변동성 확장 조건 또는 관측 주기가 다릅니다. 이전 완료 범위·배수·기존 손절과 선택 수익 보호·보유 관측 한도를 확인하세요.</Alert>}
          {strategyId.startsWith('strong_close') && slots.A && slots.B && JSON.stringify(slots.A.params) !== JSON.stringify(slots.B.params) && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 강한 종가 파라미터가 다릅니다. 이전 완료 일봉 추세·수익 보호·보유 관측 상한과 기존 손절을 확인하세요.</Alert>}
          {strategyId.startsWith('rsi') && slots.A && slots.B && JSON.stringify(slots.A.params) !== JSON.stringify(slots.B.params) && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 RSI 파라미터가 다릅니다. 반등 교차·추세·손절·보유 관측 조건을 확인하세요.</Alert>}
          {strategyId.startsWith('ma_cross') && slots.A && slots.B && JSON.stringify(slots.A.params) !== JSON.stringify(slots.B.params) && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 MA 교차 파라미터가 다릅니다. 입력 관측 MA와 완료 일봉 진입 조건을 함께 확인하세요.</Alert>}
          {strategyId.startsWith('consecutive_move') && slots.A && slots.B && JSON.stringify(slots.A.params) !== JSON.stringify(slots.B.params) && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 전략 파라미터가 다릅니다. 진입 필터 조건을 함께 확인하세요.</Alert>}
          {strategyId.startsWith('momentum') && slots.A && slots.B && JSON.stringify(slots.A.params) !== JSON.stringify(slots.B.params) && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 Momentum 파라미터가 다릅니다. 진입 모드·재무장 간격을 확인하세요.</Alert>}
          {strategyId.startsWith('mean_reversion') && slots.A && slots.B && JSON.stringify(slots.A.params) !== JSON.stringify(slots.B.params) && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 MeanReversion 파라미터가 다릅니다. 청산 모드·추세·보유 관측 상한을 확인하세요.</Alert>}
          {strategyId.startsWith('deviation') && slots.A && slots.B && JSON.stringify(slots.A.params) !== JSON.stringify(slots.B.params) && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 Deviation 파라미터가 다릅니다. 진입·추세·보유 관측·손절 조건을 확인하세요.</Alert>}
          {differentExecution && <Alert severity="warning" sx={{ mb: 1, py: 0.25 }}>A와 B의 엔진 또는 체결 가정이 다릅니다. 아래 조건 차이를 함께 확인하세요.</Alert>}
          <Typography variant="caption" display="block" sx={{ mb: 1 }}>같은 기간/겹친 기간의 A/B는 독립 검증이 아닙니다. 부분기간의 결과를 전체 분기나 연속 운용 성과로 합산하지 마세요.</Typography>
          <Grid container spacing={1}>
            {(['A', 'B'] as const).map((slot) => {
              const saved = slots[slot]
              const savedUnavailable = saved && (saved.replay.assessment?.model === 'dailyDiagnostic' || saved.replay.assessment?.performanceStatus === 'notEvaluable' || (isLeveragedReplay && !saved.replay.assessment))
              return (
                <Grid item xs={12} sm={6} key={slot}>
                  <Paper variant="outlined" sx={{ p: 1 }}>
                    <Typography variant="caption" fontWeight={700}>실험 결과 {slot}</Typography>
                    {saved && <Typography variant="caption" display="block" sx={{ overflowWrap: 'anywhere' }}>엔진 {saved.replay.engineVersion} · {executionAssumptionsLabel(saved.backtest.assumptions)}</Typography>}
                    {saved && <Typography variant="caption" display="block">{calculationModeLabel(saved.calculationMode)}{saved.continuity ? ` · 공급 ${saved.continuity.chunkCount}묶음 / 평가 ${saved.continuity.evaluationBars}봉 / 사전 ${saved.continuity.historyBars}봉 1회` : ''}</Typography>}
                    {saved?.strategyId.startsWith('failed_breakout') && <><FailedBreakoutPolicySummary params={saved.params} /><FailedBreakoutPreparation value={saved.entryFilterPreparation} params={saved.params} compact /></>}
                    {saved?.strategyId.startsWith('volatility_expansion') && <><VolatilityExpansionSummary params={saved.params} /><VolatilityPreparation value={saved.volatilityExpansionPreparation} compact /></>}
                    {saved?.strategyId.startsWith('strong_close') && <><StrongClosePolicySummary params={saved.params} /><StrongClosePreparation value={saved.entryFilterPreparation} params={saved.params} compact /></>}
                    {saved?.strategyId.startsWith('rsi') && <RsiPolicySummary params={saved.params} />}
                    {saved?.strategyId.startsWith('ma_cross') && <><MaCrossEntrySummary params={saved.params} /><MaCrossEntryPreparation value={saved.entryFilterPreparation} params={saved.params} compact /></>}
                    {saved?.strategyId.startsWith('consecutive_move') && <>
                      <ConsecutiveEntrySummary params={saved.params} />
                      <ConsecutiveEntryPreparation value={saved.entryFilterPreparation} params={saved.params} compact />
                    </>}
                    {saved?.strategyId.startsWith('momentum') && <MomentumEntrySummary params={saved.params} />}
                    {saved?.strategyId.startsWith('mean_reversion') && <MeanReversionPolicySummary params={saved.params} />}
                    {saved?.strategyId.startsWith('deviation') && <DeviationPolicySummary params={saved.params} />}
                    {saved && <AccountingAssumptionSummary value={saved.backtest.assumptions} />}
                    {saved && isLeveragedReplay && saved.replay.sourceInterval === '1m' && <LthMinuteTimingPanel value={saved.replay.assessment?.minuteTiming} compact />}
                    {savedUnavailable ? (
                      <Typography variant="body2" color="text.secondary">평가 모델 정보 부족 · 성과 비교 불가</Typography>
                    ) : saved ? (
                      <Typography variant="body2">
                        {saved.replay.assessment?.model === 'intradaySample' && '분봉 표본 · '}
                        {fmtPct(saved.backtest.summary.cumulativeReturnPct)} · MDD {saved.backtest.summary.mddPct.toFixed(2)}% · {saved.replay.dataStart}~{saved.replay.dataEnd}
                      </Typography>
                    ) : <Typography variant="body2" color="text.secondary">저장된 결과 없음</Typography>}
                    {saved && !savedUnavailable && <AccountingResults report={saved.backtest} compact />}
                  </Paper>
                </Grid>
              )
            })}
          </Grid>
        </Box>
      )}

      <TableContainer component={Paper} variant="outlined" sx={{ maxHeight: 320, maxWidth: '100%' }}>
        <Table size="small" stickyHeader aria-label="백테스트 거래 목록">
          <TableHead><TableRow><TableCell>기록 봉</TableCell><TableCell>신호 확인</TableCell><TableCell>체결 시점</TableCell><TableCell>상태</TableCell><TableCell>방향</TableCell><TableCell align="right">수량</TableCell><TableCell align="right">체결가</TableCell><TableCell align="right">비용</TableCell><TableCell align="right">실현손익</TableCell><TableCell>사유</TableCell></TableRow></TableHead>
          <TableBody>
            {report.trades.slice(0, 100).map((trade, index) => (
              <TableRow key={`${trade.time}:${trade.side}:${index}`}>
                <TableCell>{trade.time}</TableCell>
                <TableCell>{eventTimeLabel(trade.signalAvailableAt)}</TableCell>
                <TableCell>{trade.fillAt ? eventTimeLabel(trade.fillAt) : trade.status === 'filled' ? '정보 없음' : '—'}</TableCell>
                <TableCell><Chip size="small" label={trade.status === 'filled' ? '체결 가정' : trade.status === 'unfilled' ? '미체결' : '차단'} color={trade.status === 'filled' ? 'success' : 'warning'} variant="outlined" /></TableCell>
                <TableCell>{trade.side === 'buy' ? '매수' : '매도'}</TableCell>
                <TableCell align="right">{trade.quantity.toLocaleString('ko-KR')}</TableCell>
                <TableCell align="right">{trade.fillPrice == null ? '—' : trade.fillPrice.toLocaleString('ko-KR')}<Typography variant="caption" display="block">체결 기준가 {trade.executionReferencePrice ?? '정보 없음'} · 환율 {trade.exchangeRateKrw ?? '정보 없음'}</Typography></TableCell>
                <TableCell align="right">{fmtMoney(trade.costKrw)}<Typography variant="caption" display="block">수수료 {trade.feeKrw == null ? '정보 없음' : fmtMoney(trade.feeKrw)} · 거래세 {trade.taxKrw == null ? '정보 없음' : fmtMoney(trade.taxKrw)} · 슬리피지 영향 {trade.slippageKrw == null ? '정보 없음' : fmtMoney(trade.slippageKrw)}</Typography></TableCell>
                <TableCell align="right">{trade.realizedPnlKrw == null ? '—' : fmtMoney(trade.realizedPnlKrw)}</TableCell>
                <TableCell>{trade.blockedReason ?? trade.reason}{trade.intrabarPathStatus && <Typography variant="caption" display="block">봉 내부 경로: {intrabarPathLabel(trade.intrabarPathStatus)}</Typography>}</TableCell>
              </TableRow>
            ))}
            {report.trades.length === 0 && <TableRow><TableCell colSpan={10}>거래 신호가 없습니다.</TableCell></TableRow>}
          </TableBody>
        </Table>
      </TableContainer>
      {report.trades.length > 100 && <Typography variant="caption" color="text.secondary">UI 반응성을 위해 최근 계산의 거래 목록은 처음 100건만 표시합니다.</Typography>}
    </Stack>
  )
}
