import { useEffect, useMemo, useRef, useState } from 'react'

import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import CircularProgress from '@mui/material/CircularProgress'
import MenuItem from '@mui/material/MenuItem'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import RefreshIcon from '@mui/icons-material/Refresh'

import { usePreviewStrategy, usePreviewStrategyContinuous } from '../../../api/hooks'
import type {
  BrokerId,
  CmdError,
  SimulationAssumptions,
  StrategyPreviewPreparation,
  StrategyPreviewView,
  ReplayCalculationMode,
} from '../../../api/types'
import { collectPreviewCandles, collectContinuousCandles, continuousRangeError, type CandleCollection, type ContinuousCandleCollection, type ContinuousCollectionProgress } from '../model/candlePageCollector'
import { maCrossEntryFiltersDisabled } from './maCrossEntryControls'
import { defaultContinuousRange, ReplayRangeControls, ContinuousCollectionNotice } from './continuousReplayPanel'
import { StrategyPreviewChart } from './leveragedTrendHoldPreviewChart'
import {
  defaultSimulationAssumptions,
  rebaseDefaultMarketCosts,
  SimulationAssumptionsEditor,
  StrategyResearchResults,
} from './strategyResearchPanel'

type PreviewInterval = '1m' | 'D' | 'W' | 'M'
type PreviewCount = 50 | 100 | 200

const KIS_INTERVALS: Array<{ value: PreviewInterval; label: string }> = [
  { value: 'D', label: '일봉' },
  { value: 'W', label: '주봉' },
  { value: 'M', label: '월봉' },
]
const TOSS_INTERVALS: Array<{ value: PreviewInterval; label: string }> = [
  { value: '1m', label: '1분봉' },
  { value: 'D', label: '일봉' },
]
const PREVIEW_COUNTS: PreviewCount[] = [50, 100, 200]
function PreparationStatus({ value }: { value: StrategyPreviewPreparation }) {
  const historyLabel = value.historyStatus === 'unsupported' ? '사전 자료 요구량 미지원'
    : value.historyStatus === 'notRequired' ? '사전 자료 불필요'
      : value.historyStatus === 'insufficient' ? '평가 시작 사전 자료 부족' : '평가 시작 사전 자료 충분'
  const indicatorLabel = value.indicatorStatus === 'unsupported' ? '지표 준비 상태 미지원'
    : value.indicatorStatus === 'warmingUp' ? '지표 준비 중' : '지표 준비 완료'
  const outcomeLabel = value.outcome === 'conditionsNotMet' ? '매매 조건 불충족'
    : value.outcome === 'noTrades' ? '신호 발생 · 체결 없음'
      : value.outcome === 'traded' ? '체결 가정 거래 발생' : '평가 불가'
  const severity = value.historyStatus === 'insufficient' || value.outcome === 'noTrades' ? 'warning'
    : value.outcome === 'traded' ? 'success' : 'info'
  return (
    <Alert severity={severity} data-testid="strategy-preparation-status" sx={{ py: 0.75 }}>
      <Typography variant="body2">{historyLabel} · {indicatorLabel} · {outcomeLabel}</Typography>
      <Typography variant="caption" display="block">
        사전 자료 {value.providedHistoryBars}봉 / {value.requiredHistoryBars == null ? '요구량 미지원' : `필요 ${value.requiredHistoryBars}봉`}
        {' · '}지표 버퍼 {value.availableHistoryBars == null ? '미지원' : `${value.availableHistoryBars}봉`}
        {' · '}시작 준비 {value.readyAtStart == null ? '미지원' : value.readyAtStart ? '완료' : '미완료'}
        {' · '}종료 준비 {value.readyAtEnd == null ? '미지원' : value.readyAtEnd ? '완료' : '미완료'}
      </Typography>
      <Typography variant="caption" display="block">
        {value.evaluatedBars == null ? '조건 평가 확인 미지원' : `실제 조건 평가 ${value.evaluatedBars}봉`}
      </Typography>
      {value.unreadyEvaluationBars > 0 && (
        <Typography variant="caption" display="block">
          평가 구간 중 봉 처리 후 지표 준비 미확인 {value.unreadyEvaluationBars}봉
        </Typography>
      )}
      {value.firstReadyTime && <Typography variant="caption" display="block">봉 처리 후 첫 준비 완료 시점 {value.firstReadyTime}</Typography>}
    </Alert>
  )
}

function isDomesticSymbol(symbol: string) {
  return symbol.length === 6 && /^[0-9]/.test(symbol)
}

type Props = {
  strategyId: string
  strategyName: string
  brokerId: BrokerId
  brokerAccountId: string | null
  profileId: string | null
  symbols: string[]
  symbolNames: Record<string, string>
  orderQuantity: number
  params: Record<string, unknown>
  paramsError?: string | null
}

export function StrategyPreviewPanel({
  strategyId,
  strategyName,
  brokerId,
  brokerAccountId,
  profileId,
  symbols,
  symbolNames,
  orderQuantity,
  params,
  paramsError,
}: Props) {
  const dailyEventStrategy = strategyId.startsWith('strong_close') || strategyId.startsWith('volatility_expansion')
  const previewMutation = usePreviewStrategy()
  const continuousMutation = usePreviewStrategyContinuous()
  const [calculationMode, setCalculationMode] = useState<ReplayCalculationMode>('boundedWindow')
  const [continuousRange, setContinuousRange] = useState(defaultContinuousRange)
  const [continuousCollection, setContinuousCollection] = useState<ContinuousCandleCollection | null>(null)
  const [continuousProgress, setContinuousProgress] = useState<ContinuousCollectionProgress | null>(null)
  const rangeError = calculationMode === 'continuous' ? continuousRangeError(continuousRange.start, continuousRange.end) : null
  const [selectedSymbol, setSelectedSymbol] = useState(symbols[0] ?? '')
  const [previewInterval, setPreviewInterval] = useState<PreviewInterval>('D')
  const maCadenceError = strategyId.startsWith('ma_cross') && !maCrossEntryFiltersDisabled(params) && calculationMode !== 'continuous' && previewInterval !== 'D' ? 'MA 교차 진입 필터는 이전 완료 일봉이 필요합니다. 일봉을 선택하세요.' : null
  const filterCadenceError = maCadenceError ?? (strategyId.startsWith('consecutive_move') && calculationMode !== 'continuous' && previewInterval !== 'D' && (Number(params.min_move_atr_multiplier ?? 0) > 0 || Number(params.long_trend_period ?? 0) > 0) ? 'ATR·장기 MA 진입 필터는 이전 완료 일봉이 필요합니다. 일봉을 선택하세요.' : null)
  const [previewCount, setPreviewCount] = useState<PreviewCount>(50)
  const [preview, setPreview] = useState<StrategyPreviewView | null>(null)
  const initialIsOverseas = !isDomesticSymbol(symbols[0] ?? '')
  const [assumptions, setAssumptions] = useState<SimulationAssumptions>(() => defaultSimulationAssumptions(initialIsOverseas))
  const assumptionsMarket = useRef(initialIsOverseas)
  const [accountingError, setAccountingError] = useState<string | null>(null)
  const [accountingDraftRevision, setAccountingDraftRevision] = useState(0)
  const [sourceLabel, setSourceLabel] = useState('일봉 캔들')
  const [collecting, setCollecting] = useState(false)
  const [collection, setCollection] = useState<CandleCollection | null>(null)
  const [localError, setLocalError] = useState<string | null>(null)
  const previewGeneration = useRef(0)
  const previewInputKey = JSON.stringify({
    strategyId,
    brokerId,
    brokerAccountId,
    profileId,
    symbols,
    selectedSymbol,
    previewInterval,
    previewCount,
    orderQuantity,
    params, paramsError,
    assumptions,
    accountingDraftRevision,
    calculationMode, continuousRange,
  })

  const currentInputKey = useRef(previewInputKey)
  currentInputKey.current = previewInputKey

  useEffect(() => () => { previewGeneration.current += 1 }, [])

  useEffect(() => {
    previewGeneration.current += 1
    setCollecting(false)
    setCollection(null)
    setContinuousCollection(null)
    setContinuousProgress(null)
    setPreview(null)
    setLocalError(null)
    previewMutation.reset()
    continuousMutation.reset()
  }, [previewInputKey]) // eslint-disable-line react-hooks/exhaustive-deps -- serialized preview input is the invalidation boundary

  useEffect(() => {
    if (!symbols.includes(selectedSymbol)) {
      setSelectedSymbol(symbols[0] ?? '')
      setPreview(null)
    }
  }, [selectedSymbol, symbols])

  useEffect(() => {
    const nextIsOverseas = !isDomesticSymbol(selectedSymbol)
    setAssumptions((current) => rebaseDefaultMarketCosts(current, assumptionsMarket.current, nextIsOverseas))
    assumptionsMarket.current = nextIsOverseas
  }, [selectedSymbol])

  useEffect(() => {
    if (calculationMode === 'continuous' && previewInterval !== 'D') {
      setPreviewInterval('D')
    } else if (dailyEventStrategy && previewInterval !== 'D') {
      setPreviewInterval('D')
    } else if (brokerId === 'toss' && (previewInterval === 'W' || previewInterval === 'M')) {
      setPreviewInterval('D')
    } else if (brokerId !== 'toss' && previewInterval === '1m') {
      setPreviewInterval('D')
    }
  }, [brokerId, previewInterval, dailyEventStrategy, calculationMode])

  const selectedLabel = useMemo(() => {
    if (!selectedSymbol) return ''
    const name = symbolNames[selectedSymbol]
    return name && name !== selectedSymbol ? `${selectedSymbol} · ${name}` : selectedSymbol
  }, [selectedSymbol, symbolNames])

  const error = (calculationMode === 'continuous' ? continuousMutation.error : previewMutation.error) as CmdError | Error | null

  const handlePreview = async () => {
    if (!selectedSymbol || !profileId || accountingError || rangeError || paramsError || filterCadenceError) return
    const requestGeneration = ++previewGeneration.current
    const requestInputKey = previewInputKey
    const cancelled = () => requestGeneration !== previewGeneration.current || requestInputKey !== currentInputKey.current
    setLocalError(null)
    setPreview(null)
    setCollection(null)
    setContinuousCollection(null)
    setContinuousProgress(null)
    setCollecting(true)
    try {
      const source = { symbol: selectedSymbol, brokerId, isOverseas: !isDomesticSymbol(selectedSymbol), expectedProfileId: profileId, expectedBrokerAccountId: brokerAccountId ?? undefined }
      const loaded = calculationMode === 'continuous'
        ? await collectContinuousCandles(source, continuousRange, cancelled, (progress) => { if (!cancelled()) setContinuousProgress(progress) })
        : await collectPreviewCandles({ ...source, interval: previewInterval }, previewCount, cancelled)
      if (cancelled()) return
      if (calculationMode === 'continuous') {
        const continuous = loaded as ContinuousCandleCollection
        setContinuousCollection(continuous)
        if (!continuous.ready) throw new Error('연속기간 자료 준비를 확인하지 못해 계산을 보류했습니다.')
      } else setCollection(loaded as CandleCollection)
      if (!loaded.candles.length) throw new Error(`완료된 평가 봉이 없습니다. ${loaded.sourceLabel}`)
      setSourceLabel(loaded.sourceLabel)
      const common = {
        strategyId,
        strategyName,
        symbol: selectedSymbol,
        isOverseas: loaded.isOverseas,
        orderQuantity,
        params,
        historyCandles: loaded.historyCandles,
        interval: previewInterval,
        dataSource: loaded.dataSource,
        strategyVersion: 'strategy-config-v1',
        brokerId,
        brokerAccountId,
        assumptions,
      }
      const result = calculationMode === 'continuous'
        ? await continuousMutation.mutateAsync({ ...common, chunks: (loaded as ContinuousCandleCollection).chunks })
        : await previewMutation.mutateAsync({ ...common, candles: loaded.candles })
      if (cancelled()) return
      if (calculationMode === 'continuous' && (result.continuity?.mode !== 'continuous' || result.continuity.evaluationBars !== loaded.candles.length || result.continuity.historyBars !== loaded.historyCandles.length || result.continuity.chunkCount !== (loaded as ContinuousCandleCollection).chunks.length || result.continuity.chunks.length !== (loaded as ContinuousCandleCollection).chunks.length)) throw new Error('연속 실행 응답의 평가/사전자료 계약이 입력과 일치하지 않습니다.')
      setPreview(result)
    } catch (caught) {
      if (cancelled()) return
      setPreview(null)
      setLocalError(caught instanceof Error ? caught.message : typeof caught === 'object' && caught !== null && 'message' in caught ? String(caught.message) : '미리보기 차트 데이터를 가져오지 못했습니다.')
    } finally {
      if (!cancelled()) setCollecting(false)
    }
  }

  const busy = collecting || previewMutation.isPending || continuousMutation.isPending
  const cancelPreview = () => {
    previewGeneration.current += 1
    setCollecting(false)
    setCollection(null)
    setPreview(null)
    previewMutation.reset()
    continuousMutation.reset()
    setContinuousProgress(null)
    setLocalError('미리보기 계산을 취소했습니다.')
  }

  return (
    <Box sx={{ pt: 1.5, borderTop: 1, borderColor: 'divider' }}>
      {(paramsError || filterCadenceError) && <Alert severity="error">{paramsError ?? filterCadenceError}</Alert>}
      <ReplayRangeControls mode={calculationMode} onModeChange={setCalculationMode} range={continuousRange} onRangeChange={setContinuousRange} disabled={busy} error={rangeError} />
      <SimulationAssumptionsEditor
        continuousMode={calculationMode === 'continuous'}
        key={`${profileId}:${brokerId}:${brokerAccountId}:${selectedSymbol}`}
        isOverseas={!isDomesticSymbol(selectedSymbol)} onValidationChange={setAccountingError}
        onDraftChange={() => { previewGeneration.current += 1; setPreview(null); setAccountingDraftRevision((revision) => revision + 1) }}
        value={assumptions}
        onChange={setAssumptions}
        disabled={busy}
      />
      <Stack
        direction={{ xs: 'column', sm: 'row' }}
        alignItems={{ xs: 'stretch', sm: 'center' }}
        justifyContent="space-between"
        spacing={1}
        sx={{ my: 1 }}
      >
        <Box>
          <Typography variant="caption" fontWeight={700}>
            전략 미리보기
          </Typography>
          <Typography variant="caption" color="text.secondary" display="block">
            저장 전 편집값으로 선택한 봉 단위와 분석 구간의 매수/청산 신호를 재계산합니다.
          </Typography>
        </Box>
        <Stack
          direction={{ xs: 'column', sm: 'row' }}
          spacing={1}
          alignItems={{ xs: 'stretch', sm: 'center' }}
        >
          <TextField
            select
            size="small"
            label="시뮬레이션 티커"
            value={selectedSymbol}
            onChange={(event) => {
              setSelectedSymbol(event.target.value)
              setPreview(null)
              setLocalError(null)
            }}
            disabled={symbols.length === 0 || busy}
            fullWidth
            sx={{ minWidth: { sm: 220 } }}
          >
            {symbols.map((symbol) => (
              <MenuItem key={symbol} value={symbol}>
                {symbolNames[symbol] ? `${symbol} · ${symbolNames[symbol]}` : symbol}
              </MenuItem>
            ))}
          </TextField>
          <TextField
            select
            size="small"
            label="봉 단위"
            value={previewInterval}
            onChange={(event) => {
              setPreviewInterval(event.target.value as PreviewInterval)
              setPreview(null)
              setLocalError(null)
            }}
            disabled={busy || calculationMode === 'continuous'}
            fullWidth
            sx={{ minWidth: { sm: 110 } }}
          >
            {(brokerId === 'toss' ? TOSS_INTERVALS : KIS_INTERVALS).filter((option) => !dailyEventStrategy || option.value === 'D').map((option) => (
              <MenuItem key={option.value} value={option.value}>{option.label}</MenuItem>
            ))}
          </TextField>
          <TextField
            select
            size="small"
            label="분석 구간"
            value={previewCount}
            onChange={(event) => {
              setPreviewCount(Number(event.target.value) as PreviewCount)
              setPreview(null)
              setLocalError(null)
            }}
            disabled={busy || calculationMode === 'continuous'}
            helperText={calculationMode === 'continuous' ? '단기 모드 전용' : undefined}
            fullWidth
            sx={{ minWidth: { sm: 120 } }}
          >
            {PREVIEW_COUNTS.map((count) => (
              <MenuItem key={count} value={count}>최근 {count}봉</MenuItem>
            ))}
          </TextField>
          <Button
            size="small"
            variant="outlined"
            startIcon={busy ? <CircularProgress size={14} /> : <RefreshIcon />}
            onClick={handlePreview}
            disabled={!selectedSymbol || !profileId || busy || !!accountingError || !!rangeError || !!paramsError || !!filterCadenceError}
            sx={{ width: { xs: '100%', sm: 'auto' }, whiteSpace: 'nowrap' }}
          >
            {collecting ? '캔들 페이지 조회 중' : '미리보기 계산'}
          </Button>
          {busy && <Button size="small" onClick={cancelPreview}>취소</Button>}
        </Stack>
      </Stack>

      {calculationMode === 'boundedWindow' && <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 1 }}>
        요청 분석 구간 최근 {previewCount}봉을 평가하고, 그 이전 봉만 사전 자료로 사용합니다.
        {' '}{brokerId === 'toss' ? 'Toss는 한 페이지 최대 200봉' : 'KIS는 한 페이지 최대 100봉'}으로 사전 자료 252봉까지 순차 조회합니다.
        최대 8페이지·보관 500봉이며, 제공 자료가 끝나면 실제 확보량과 부족 사유를 표시합니다.
      </Typography>}

      {collection && (
        <Alert severity={collection.historyCandles.length < 252 || collection.candles.length < previewCount ? 'warning' : 'info'}
          data-testid="candle-collection-status" sx={{ mb: 1, overflowWrap: 'anywhere' }}>
          {collection.sourceLabel}
          {(collection.historyCandles.length < 252 || collection.candles.length < previewCount) && ' · 자료 부족: 요청 기간과 사전 자료 기준을 줄이지 않습니다.'}
        </Alert>
      )}

      {calculationMode === 'continuous' && <ContinuousCollectionNotice progress={continuousProgress} collection={continuousCollection} />}
      {dailyEventStrategy && (
        <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 1 }} data-testid="daily-event-replay-note">
          강한 종가는 전일 완료봉으로 다음 평가일을 판단하고, 변동성 확장은 이전 일봉 범위와 당일 OHLC를 비교합니다.
          {' '}일봉만 지원하며 선택한 같은 종가/다음 관측 시가 체결 가정을 적용합니다. 실제 장중 순서·체결은 검증하지 않습니다.
        </Typography>
      )}

      {symbols.length === 0 ? (
        <Alert severity="info" sx={{ py: 0.75 }}>
          대상 종목을 추가하면 이 카드 안에서 바로 전략 신호를 미리볼 수 있습니다.
        </Alert>
      ) : localError || error ? (
        <Alert severity="warning" sx={{ py: 0.75 }}>
          {localError ?? ('message' in error! ? error!.message : '미리보기 계산 중 오류가 발생했습니다.')}
        </Alert>
      ) : preview ? (
        <Stack spacing={1}>
          <Alert severity={preview.signals.length > 0 ? 'success' : 'info'} sx={{ py: 0.75 }}>
            {preview.message}
          </Alert>
          <Typography variant="caption" color="text.secondary" data-testid="strategy-evaluation-range">
            {calculationMode === 'continuous' ? `연속 요청 ${continuousRange.start}~${continuousRange.end}` : `요청 분석 ${previewCount}봉`} · 실제 평가 {preview.candles.length}봉
            {' · '}사전 자료 {continuousCollection?.historyCandles.length ?? collection?.historyCandles.length ?? preview.preparation?.providedHistoryBars ?? 0}봉
          </Typography>
          {preview.preparation && <PreparationStatus value={preview.preparation} />}
          <StrategyPreviewChart
            candles={preview.candles}
            signals={preview.signals}
            sourceLabel={`${sourceLabel} · ${selectedLabel}`}
            emptyLabel={`${selectedLabel} 차트 데이터가 아직 없습니다.`}
          />
          {preview.backtest && preview.replay && (
            <StrategyResearchResults
              report={preview.backtest}
              calculationMode={preview.continuity ? 'continuous' : 'boundedWindow'}
              continuity={preview.continuity}
              entryFilterPreparation={preview.entryFilterPreparation}
              replay={preview.replay}
              strategyId={strategyId}
              brokerId={brokerId}
              brokerAccountId={brokerAccountId}
              symbol={selectedSymbol}
              params={params}
              orderQuantity={orderQuantity}
              generatedAt={preview.generatedAt}
            />
          )}
        </Stack>
      ) : (
        <Alert severity="info" sx={{ py: 0.75 }}>
          티커를 선택하고 미리보기 계산을 누르면 이 카드의 현재 세팅으로 신호를 표시합니다.
        </Alert>
      )}
    </Box>
  )
}
