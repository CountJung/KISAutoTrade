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

import { usePreviewStrategy } from '../../../api/hooks'
import * as cmd from '../../../api/commands'
import type {
  BrokerId,
  ChartCandle,
  CmdError,
  SimulationAssumptions,
  StrategyPreviewPreparation,
  StrategyPreviewView,
} from '../../../api/types'
import { StrategyPreviewChart } from './leveragedTrendHoldPreviewChart'
import {
  defaultSimulationAssumptions,
  rebaseDefaultMarketCosts,
  SimulationAssumptionsEditor,
  StrategyResearchResults,
} from './strategyResearchPanel'

const OVERSEAS_EXCHANGES = ['NAS', 'NYS', 'AMS'] as const
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
const HISTORY_REQUEST_BARS = 252

function previewRangeLabel(actual: number, requested: PreviewCount) {
  return actual === requested ? `최근 ${requested}봉` : `실제 ${actual}봉 / 요청 ${requested}봉`
}

function splitPreviewCandles(candles: ChartCandle[], count: PreviewCount, dataSource: string) {
  const ordered = [...new Map(candles.map((candle) => [candle.date, candle])).values()]
    .sort((left, right) => left.date.localeCompare(right.date))
  const evaluationStart = Math.max(0, ordered.length - count)
  const evaluation = ordered.slice(evaluationStart)
  const historyCandles = ordered.slice(0, evaluationStart)
  return {
    candles: evaluation,
    historyCandles,
    dataSource,
    sourceLabel: `${dataSource} · ${previewRangeLabel(evaluation.length, count)} · 사전 자료 ${historyCandles.length}봉`,
  }
}

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

function toYmd(date: Date) {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}${month}${day}`
}

async function loadPreviewCandles(
  symbol: string,
  brokerId: BrokerId,
  interval: PreviewInterval,
  count: PreviewCount,
): Promise<{ candles: ChartCandle[]; historyCandles: ChartCandle[]; sourceLabel: string; dataSource: string; isOverseas: boolean }> {
  const isOverseas = !isDomesticSymbol(symbol)
  const intervalLabel = interval === '1m' ? '1분봉' : interval === 'D' ? '일봉' : interval === 'W' ? '주봉' : '월봉'

  if (brokerId === 'toss') {
    const tossInterval = interval === '1m' ? '1m' : '1d'
    const candles = await cmd.getTossChartData(symbol, tossInterval, Math.min(count + HISTORY_REQUEST_BARS, 200))
    return {
      ...splitPreviewCandles(candles, count, `Toss ${intervalLabel}`),
      isOverseas,
    }
  }

  if (!isOverseas) {
    const end = new Date()
    const start = new Date(end)
    const calendarDaysPerCandle = interval === 'D' ? 2 : interval === 'W' ? 8 : 32
    const requestedBars = count + HISTORY_REQUEST_BARS
    start.setDate(start.getDate() - requestedBars * calendarDaysPerCandle)
    const candles = await cmd.getChartData({
      symbol,
      period_code: interval,
      start_date: toYmd(start),
      end_date: toYmd(end),
      count: requestedBars,
    })
    return {
      ...splitPreviewCandles(candles, count, `KIS ${intervalLabel}`),
      isOverseas: false,
    }
  }

  let lastError: unknown = null
  for (const exchange of OVERSEAS_EXCHANGES) {
    try {
      const candles = await cmd.getOverseasChartData(symbol, exchange, interval, '', count + HISTORY_REQUEST_BARS)
      return {
        ...splitPreviewCandles(candles, count, `KIS ${exchange} ${intervalLabel}`),
        isOverseas: true,
      }
    } catch (error) {
      lastError = error
    }
  }
  throw lastError ?? new Error(`${symbol} 해외 일봉 차트를 조회할 수 없습니다.`)
}

type Props = {
  strategyId: string
  strategyName: string
  brokerId: BrokerId
  brokerAccountId: string | null
  symbols: string[]
  symbolNames: Record<string, string>
  orderQuantity: number
  params: Record<string, unknown>
}

export function StrategyPreviewPanel({
  strategyId,
  strategyName,
  brokerId,
  brokerAccountId,
  symbols,
  symbolNames,
  orderQuantity,
  params,
}: Props) {
  const dailyEventStrategy = strategyId.startsWith('strong_close') || strategyId.startsWith('volatility_expansion')
  const previewMutation = usePreviewStrategy()
  const [selectedSymbol, setSelectedSymbol] = useState(symbols[0] ?? '')
  const [previewInterval, setPreviewInterval] = useState<PreviewInterval>('D')
  const [previewCount, setPreviewCount] = useState<PreviewCount>(50)
  const [preview, setPreview] = useState<StrategyPreviewView | null>(null)
  const initialIsOverseas = !isDomesticSymbol(symbols[0] ?? '')
  const [assumptions, setAssumptions] = useState<SimulationAssumptions>(() => defaultSimulationAssumptions(initialIsOverseas))
  const assumptionsMarket = useRef(initialIsOverseas)
  const [sourceLabel, setSourceLabel] = useState('일봉 캔들')
  const [localError, setLocalError] = useState<string | null>(null)
  const previewGeneration = useRef(0)
  const previewInputKey = JSON.stringify({
    strategyId,
    brokerId,
    brokerAccountId,
    symbols,
    selectedSymbol,
    previewInterval,
    previewCount,
    orderQuantity,
    params,
    assumptions,
  })

  useEffect(() => {
    previewGeneration.current += 1
    setPreview(null)
    setLocalError(null)
    previewMutation.reset()
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
    if (dailyEventStrategy && previewInterval !== 'D') {
      setPreviewInterval('D')
    } else if (brokerId === 'toss' && (previewInterval === 'W' || previewInterval === 'M')) {
      setPreviewInterval('D')
    } else if (brokerId !== 'toss' && previewInterval === '1m') {
      setPreviewInterval('D')
    }
  }, [brokerId, previewInterval, dailyEventStrategy])

  const selectedLabel = useMemo(() => {
    if (!selectedSymbol) return ''
    const name = symbolNames[selectedSymbol]
    return name && name !== selectedSymbol ? `${selectedSymbol} · ${name}` : selectedSymbol
  }, [selectedSymbol, symbolNames])

  const error = previewMutation.error as CmdError | Error | null

  const handlePreview = async () => {
    if (!selectedSymbol) return
    const requestGeneration = previewGeneration.current
    setLocalError(null)
    try {
      const loaded = await loadPreviewCandles(selectedSymbol, brokerId, previewInterval, previewCount)
      if (requestGeneration !== previewGeneration.current) return
      setSourceLabel(loaded.sourceLabel)
      const result = await previewMutation.mutateAsync({
        strategyId,
        strategyName,
        symbol: selectedSymbol,
        isOverseas: loaded.isOverseas,
        orderQuantity,
        params,
        candles: loaded.candles,
        historyCandles: loaded.historyCandles,
        interval: previewInterval,
        dataSource: loaded.dataSource,
        strategyVersion: 'strategy-config-v1',
        brokerId,
        brokerAccountId,
        assumptions,
      })
      if (requestGeneration !== previewGeneration.current) return
      setPreview(result)
    } catch (caught) {
      if (requestGeneration !== previewGeneration.current) return
      setPreview(null)
      setLocalError(caught instanceof Error ? caught.message : '미리보기 차트 데이터를 가져오지 못했습니다.')
    }
  }

  return (
    <Box sx={{ pt: 1.5, borderTop: 1, borderColor: 'divider' }}>
      <SimulationAssumptionsEditor
        value={assumptions}
        onChange={setAssumptions}
        disabled={previewMutation.isPending}
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
            disabled={symbols.length === 0 || previewMutation.isPending}
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
            disabled={previewMutation.isPending}
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
            disabled={previewMutation.isPending}
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
            startIcon={previewMutation.isPending ? <CircularProgress size={14} /> : <RefreshIcon />}
            onClick={handlePreview}
            disabled={!selectedSymbol || previewMutation.isPending}
            sx={{ width: { xs: '100%', sm: 'auto' }, whiteSpace: 'nowrap' }}
          >
            미리보기 계산
          </Button>
        </Stack>
      </Stack>

      <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 1 }}>
        요청 분석 구간 최근 {previewCount}봉을 평가하고, 그 이전 봉만 사전 자료로 사용합니다.
        {' '}{brokerId === 'toss' ? 'Toss는 한 페이지 최대 200봉' : 'KIS는 한 페이지 응답 범위'} 내에서 조회해
        사전 자료가 부족할 수 있으며, 추가 페이지 조회는 하지 않습니다.
      </Typography>

      {dailyEventStrategy && (
        <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 1 }} data-testid="daily-event-replay-note">
          강한 종가는 전일 완료봉으로 다음 평가일을 판단하고, 변동성 확장은 이전 일봉 범위와 당일 OHLC를 비교합니다.
          {' '}일봉만 지원하며 체결은 종가 가정입니다. 장중 신호·익일 시가 체결을 재현하지 않습니다.
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
            요청 분석 {previewCount}봉 · 실제 평가 {preview.candles.length}봉
            {' · '}사전 자료 {preview.preparation?.providedHistoryBars ?? preview.replay?.warmupCount ?? 0}봉
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
