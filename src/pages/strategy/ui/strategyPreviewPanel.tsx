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
  StrategyPreviewView,
} from '../../../api/types'
import { StrategyPreviewChart } from './leveragedTrendHoldPreviewChart'

const OVERSEAS_EXCHANGES = ['NAS', 'NYS', 'AMS'] as const

function isDomesticSymbol(symbol: string) {
  return symbol.length === 6 && /^[0-9]/.test(symbol)
}

function toYmd(date: Date) {
  return date.toISOString().slice(0, 10).replace(/-/g, '')
}

async function loadPreviewCandles(
  symbol: string,
  brokerId: BrokerId,
): Promise<{ candles: ChartCandle[]; sourceLabel: string; isOverseas: boolean }> {
  const isOverseas = !isDomesticSymbol(symbol)

  if (brokerId === 'toss') {
    return {
      candles: await cmd.getTossChartData(symbol, '1d', 200),
      sourceLabel: 'Toss 일봉 캔들',
      isOverseas,
    }
  }

  if (!isOverseas) {
    const end = new Date()
    const start = new Date(end)
    start.setFullYear(start.getFullYear() - 1)
    return {
      candles: await cmd.getChartData({
        symbol,
        period_code: 'D',
        start_date: toYmd(start),
        end_date: toYmd(end),
      }),
      sourceLabel: 'KIS 일봉 캔들',
      isOverseas: false,
    }
  }

  let lastError: unknown = null
  for (const exchange of OVERSEAS_EXCHANGES) {
    try {
      return {
        candles: await cmd.getOverseasChartData(symbol, exchange, 'D', ''),
        sourceLabel: `KIS ${exchange} 일봉 캔들`,
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
  symbols: string[]
  symbolNames: Record<string, string>
  orderQuantity: number
  params: Record<string, unknown>
}

export function StrategyPreviewPanel({
  strategyId,
  strategyName,
  brokerId,
  symbols,
  symbolNames,
  orderQuantity,
  params,
}: Props) {
  const previewMutation = usePreviewStrategy()
  const [selectedSymbol, setSelectedSymbol] = useState(symbols[0] ?? '')
  const [preview, setPreview] = useState<StrategyPreviewView | null>(null)
  const [sourceLabel, setSourceLabel] = useState('일봉 캔들')
  const [localError, setLocalError] = useState<string | null>(null)
  const previewGeneration = useRef(0)
  const previewInputKey = JSON.stringify({
    strategyId,
    brokerId,
    symbols,
    selectedSymbol,
    orderQuantity,
    params,
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
      const loaded = await loadPreviewCandles(selectedSymbol, brokerId)
      setSourceLabel(loaded.sourceLabel)
      const result = await previewMutation.mutateAsync({
        strategyId,
        strategyName,
        symbol: selectedSymbol,
        isOverseas: loaded.isOverseas,
        orderQuantity,
        params,
        candles: loaded.candles,
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
      <Stack
        direction={{ xs: 'column', sm: 'row' }}
        alignItems={{ xs: 'stretch', sm: 'center' }}
        justifyContent="space-between"
        spacing={1}
        sx={{ mb: 1 }}
      >
        <Box>
          <Typography variant="caption" fontWeight={700}>
            전략 미리보기
          </Typography>
          <Typography variant="caption" color="text.secondary" display="block">
            저장 전 편집값과 일봉 차트로 매수/청산 신호를 재계산합니다.
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
        </Stack>
      ) : (
        <Alert severity="info" sx={{ py: 0.75 }}>
          티커를 선택하고 미리보기 계산을 누르면 이 카드의 현재 세팅으로 신호를 표시합니다.
        </Alert>
      )}
    </Box>
  )
}
