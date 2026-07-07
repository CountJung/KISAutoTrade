import { useEffect, useMemo, useRef } from 'react'

import Box from '@mui/material/Box'
import Chip from '@mui/material/Chip'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import { useTheme } from '@mui/material/styles'

import {
  CandlestickSeries,
  ColorType,
  createChart,
  createSeriesMarkers,
  type CandlestickData,
  type IChartApi,
  type ISeriesApi,
  type ISeriesMarkersPluginApi,
  type SeriesMarker,
  type Time,
  type UTCTimestamp,
} from 'lightweight-charts'

import type {
  ChartCandle,
  LeveragedTrendHoldPreviewSignal,
} from '../../../api/types'

type Props = {
  candles: ChartCandle[]
  signals: LeveragedTrendHoldPreviewSignal[]
}

function toNumber(value: string) {
  const parsed = Number(value.replace(/,/g, ''))
  return Number.isFinite(parsed) ? parsed : 0
}

function toChartTime(value: string, fallbackIndex = 0): Time {
  const digits = value.replace(/\D/g, '')
  if (digits.length >= 12) {
    const year = Number(digits.slice(0, 4))
    const month = Number(digits.slice(4, 6)) - 1
    const day = Number(digits.slice(6, 8))
    const hour = Number(digits.slice(8, 10))
    const minute = Number(digits.slice(10, 12))
    const second = digits.length >= 14 ? Number(digits.slice(12, 14)) : 0
    const ms = new Date(year, month, day, hour, minute, second).getTime()
    if (Number.isFinite(ms)) return Math.floor(ms / 1000) as UTCTimestamp
  }
  const parsed = Date.parse(value)
  if (Number.isFinite(parsed)) return Math.floor(parsed / 1000) as UTCTimestamp
  return (fallbackIndex + 1) as UTCTimestamp
}

function toLabel(time: string) {
  const digits = time.replace(/\D/g, '')
  if (digits.length >= 12) {
    return `${digits.slice(4, 6)}/${digits.slice(6, 8)} ${digits.slice(8, 10)}:${digits.slice(10, 12)}`
  }
  return time
}

export function LeveragedTrendHoldPreviewChart({ candles, signals }: Props) {
  const theme = useTheme()
  const containerRef = useRef<HTMLDivElement | null>(null)
  const chartRef = useRef<IChartApi | null>(null)
  const candleRef = useRef<ISeriesApi<'Candlestick', Time> | null>(null)
  const markersRef = useRef<ISeriesMarkersPluginApi<Time> | null>(null)

  const chartData = useMemo<CandlestickData<Time>[]>(
    () => candles
      .map((candle, index) => ({
        time: toChartTime(candle.date, index),
        open: toNumber(candle.open),
        high: toNumber(candle.high),
        low: toNumber(candle.low),
        close: toNumber(candle.close),
      }))
      .filter((candle) => candle.open > 0 && candle.high > 0 && candle.low > 0 && candle.close > 0),
    [candles],
  )

  const markers = useMemo<SeriesMarker<Time>[]>(
    () => signals.map((signal, index) => ({
      time: toChartTime(signal.time, index),
      position: signal.side === 'buy' ? 'belowBar' : 'aboveBar',
      color: signal.side === 'buy' ? theme.palette.success.main : theme.palette.error.main,
      shape: signal.side === 'buy' ? 'arrowUp' : 'arrowDown',
      text: signal.side === 'buy' ? `매수 ${signal.price.toFixed(2)}` : `매도 ${signal.price.toFixed(2)}`,
      size: 1.6,
    })),
    [signals, theme.palette.error.main, theme.palette.success.main],
  )

  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const chart = createChart(container, {
      width: container.clientWidth,
      height: 300,
      layout: {
        background: { type: ColorType.Solid, color: theme.palette.background.paper },
        textColor: theme.palette.text.secondary,
        fontSize: 11,
      },
      grid: {
        vertLines: { color: theme.palette.divider },
        horzLines: { color: theme.palette.divider },
      },
      rightPriceScale: { borderVisible: false },
      timeScale: { borderVisible: false, timeVisible: true, secondsVisible: false },
      crosshair: {
        vertLine: { color: theme.palette.text.disabled },
        horzLine: { color: theme.palette.text.disabled },
      },
    })
    const candleSeries = chart.addSeries(CandlestickSeries, {
      upColor: theme.palette.success.main,
      downColor: theme.palette.error.main,
      borderVisible: false,
      wickUpColor: theme.palette.success.main,
      wickDownColor: theme.palette.error.main,
    })
    const markerApi = createSeriesMarkers(candleSeries, [])

    chartRef.current = chart
    candleRef.current = candleSeries
    markersRef.current = markerApi

    const ro = new ResizeObserver(() => {
      chart.applyOptions({ width: container.clientWidth })
    })
    ro.observe(container)

    return () => {
      ro.disconnect()
      chart.remove()
      chartRef.current = null
      candleRef.current = null
      markersRef.current = null
    }
  }, [theme])

  useEffect(() => {
    candleRef.current?.setData(chartData)
    markersRef.current?.setMarkers(markers)
    chartRef.current?.timeScale().fitContent()
  }, [chartData, markers])

  if (candles.length === 0) {
    return (
      <Box sx={{ minHeight: 180, display: 'grid', placeItems: 'center', border: 1, borderColor: 'divider', borderRadius: 1 }}>
        <Typography variant="caption" color="text.secondary">
          Toss 1분봉 데이터가 아직 없습니다.
        </Typography>
      </Box>
    )
  }

  return (
    <Stack spacing={1}>
      <Box ref={containerRef} data-testid="lth-preview-chart" sx={{ width: '100%', minHeight: 300 }} />
      <Stack direction="row" gap={0.75} flexWrap="wrap">
        {signals.length === 0 ? (
          <Chip size="small" variant="outlined" label="신호 없음" />
        ) : signals.map((signal) => (
          <Chip
            key={`${signal.side}-${signal.time}-${signal.price}`}
            size="small"
            color={signal.side === 'buy' ? 'success' : 'error'}
            variant="outlined"
            label={`${signal.side === 'buy' ? '매수' : '매도'} ${toLabel(signal.time)} · ${signal.price.toFixed(2)}`}
          />
        ))}
      </Stack>
    </Stack>
  )
}
