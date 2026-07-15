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

function previewRangeLabel(actual: number, requested: PreviewCount) {
  return actual === requested ? `최근 ${requested}봉` : `실제 ${actual}봉 / 요청 ${requested}봉`
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
): Promise<{ candles: ChartCandle[]; sourceLabel: string; isOverseas: boolean }> {
  const isOverseas = !isDomesticSymbol(symbol)
  const intervalLabel = interval === '1m' ? '1분봉' : interval === 'D' ? '일봉' : interval === 'W' ? '주봉' : '월봉'

  if (brokerId === 'toss') {
    const tossInterval = interval === '1m' ? '1m' : '1d'
    const candles = await cmd.getTossChartData(symbol, tossInterval, count)
    return {
      candles,
      sourceLabel: `Toss ${intervalLabel} · ${previewRangeLabel(candles.length, count)}`,
      isOverseas,
    }
  }

  if (!isOverseas) {
    const end = new Date()
    const start = new Date(end)
    const calendarDaysPerCandle = interval === 'D' ? 2 : interval === 'W' ? 8 : 32
    start.setDate(start.getDate() - count * calendarDaysPerCandle)
    const candles = await cmd.getChartData({
      symbol,
      period_code: interval,
      start_date: toYmd(start),
      end_date: toYmd(end),
      count,
    })
    return {
      candles: candles.slice(-count),
      sourceLabel: `KIS ${intervalLabel} · ${previewRangeLabel(Math.min(candles.length, count), count)}`,
      isOverseas: false,
    }
  }

  let lastError: unknown = null
  for (const exchange of OVERSEAS_EXCHANGES) {
    try {
      const candles = await cmd.getOverseasChartData(symbol, exchange, interval, '', count)
      return {
        candles: candles.slice(-count),
        sourceLabel: `KIS ${exchange} ${intervalLabel} · ${previewRangeLabel(Math.min(candles.length, count), count)}`,
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
    if (brokerId === 'toss' && (previewInterval === 'W' || previewInterval === 'M')) {
      setPreviewInterval('D')
    } else if (brokerId !== 'toss' && previewInterval === '1m') {
      setPreviewInterval('D')
    }
  }, [brokerId, previewInterval])

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
      setSourceLabel(loaded.sourceLabel)
      const result = await previewMutation.mutateAsync({
        strategyId,
        strategyName,
        symbol: selectedSymbol,
        isOverseas: loaded.isOverseas,
        orderQuantity,
        params,
        candles: loaded.candles,
        interval: previewInterval,
        dataSource: loaded.sourceLabel,
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
            {(brokerId === 'toss' ? TOSS_INTERVALS : KIS_INTERVALS).map((option) => (
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
