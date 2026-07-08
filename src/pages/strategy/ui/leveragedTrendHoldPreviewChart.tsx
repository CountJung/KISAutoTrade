import { useEffect, useMemo, useRef, useState } from 'react'

import Box from '@mui/material/Box'
import Chip from '@mui/material/Chip'
import FormControlLabel from '@mui/material/FormControlLabel'
import Stack from '@mui/material/Stack'
import Switch from '@mui/material/Switch'
import Typography from '@mui/material/Typography'
import { useTheme } from '@mui/material/styles'

import {
  CandlestickSeries,
  ColorType,
  createChart,
  createSeriesMarkers,
  LineSeries,
  type CandlestickData,
  type IChartApi,
  type ISeriesApi,
  type ISeriesMarkersPluginApi,
  type LineData,
  type SeriesMarker,
  type Time,
  type UTCTimestamp,
} from 'lightweight-charts'

import type {
  ChartCandle,
  LeveragedTrendHoldPreviewSignal,
  StrategyPreviewSignal,
} from '../../../api/types'

export type StrategyPreviewChartSignal =
  | LeveragedTrendHoldPreviewSignal
  | StrategyPreviewSignal

type Props = {
  candles: ChartCandle[]
  signals: StrategyPreviewChartSignal[]
  sourceLabel?: string
  emptyLabel?: string
}

const KST_OFFSET_MS = 9 * 60 * 60 * 1000

function toNumber(value: string) {
  const parsed = Number(value.replace(/,/g, ''))
  return Number.isFinite(parsed) ? parsed : 0
}

function parseTimeMs(value: string, fallbackIndex = 0): number {
  const digits = value.replace(/\D/g, '')
  if (digits.length === 8) {
    const year = Number(digits.slice(0, 4))
    const month = Number(digits.slice(4, 6)) - 1
    const day = Number(digits.slice(6, 8))
    const ms = Date.UTC(year, month, day) - KST_OFFSET_MS
    if (Number.isFinite(ms)) return ms
  }
  if (digits.length >= 12) {
    const year = Number(digits.slice(0, 4))
    const month = Number(digits.slice(4, 6)) - 1
    const day = Number(digits.slice(6, 8))
    const hour = Number(digits.slice(8, 10))
    const minute = Number(digits.slice(10, 12))
    const second = digits.length >= 14 ? Number(digits.slice(12, 14)) : 0
    const ms = Date.UTC(year, month, day, hour, minute, second) - KST_OFFSET_MS
    if (Number.isFinite(ms)) return ms
  }
  const parsed = Date.parse(value)
  if (Number.isFinite(parsed)) return parsed
  return (fallbackIndex + 1) * 1000
}

function toChartTime(value: string, fallbackIndex = 0): Time {
  const ms = parseTimeMs(value, fallbackIndex)
  if (Number.isFinite(ms)) return Math.floor(ms / 1000) as UTCTimestamp
  return (fallbackIndex + 1) as UTCTimestamp
}

function chartTimeToMs(time: Time): number | null {
  if (typeof time === 'number') return time * 1000
  if (typeof time === 'string') {
    const parsed = Date.parse(time)
    return Number.isFinite(parsed) ? parsed : null
  }
  if (typeof time === 'object' && time !== null && 'year' in time && 'month' in time && 'day' in time) {
    return Date.UTC(time.year, time.month - 1, time.day) - KST_OFFSET_MS
  }
  return null
}

function pad2(value: number) {
  return String(value).padStart(2, '0')
}

function formatKstMs(ms: number, includeDate: boolean) {
  const kst = new Date(ms + KST_OFFSET_MS)
  const month = pad2(kst.getUTCMonth() + 1)
  const day = pad2(kst.getUTCDate())
  const hour = pad2(kst.getUTCHours())
  const minute = pad2(kst.getUTCMinutes())
  return includeDate ? `${month}/${day} ${hour}:${minute}` : `${hour}:${minute}`
}

function formatChartTimeKst(time: Time, includeDate = false) {
  const ms = chartTimeToMs(time)
  return ms === null ? String(time) : formatKstMs(ms, includeDate)
}

function toLabel(time: string) {
  return formatKstMs(parseTimeMs(time), true)
}

export function StrategyPreviewChart({
  candles,
  signals,
  sourceLabel = 'Toss 1분봉 캔들',
  emptyLabel = '차트 데이터가 아직 없습니다.',
}: Props) {
  const theme = useTheme()
  const containerRef = useRef<HTMLDivElement | null>(null)
  const chartRef = useRef<IChartApi | null>(null)
  const candleRef = useRef<ISeriesApi<'Candlestick', Time> | null>(null)
  const closeLineRef = useRef<ISeriesApi<'Line', Time> | null>(null)
  const markersRef = useRef<ISeriesMarkersPluginApi<Time> | null>(null)
  const [showCloseLine, setShowCloseLine] = useState(true)

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

  const closeLineData = useMemo<LineData<Time>[]>(
    () => chartData.map((candle) => ({
      time: candle.time,
      value: candle.close,
    })),
    [chartData],
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
      localization: {
        timeFormatter: (time: Time) => formatChartTimeKst(time, true),
      },
      timeScale: {
        borderVisible: false,
        timeVisible: true,
        secondsVisible: false,
        tickMarkFormatter: (time: Time) => formatChartTimeKst(time, false),
      },
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
    const closeLineSeries = chart.addSeries(LineSeries, {
      color: theme.palette.info.main,
      lineWidth: 2,
      priceLineVisible: false,
      lastValueVisible: false,
      crosshairMarkerVisible: true,
    })
    const markerApi = createSeriesMarkers(candleSeries, [])

    chartRef.current = chart
    candleRef.current = candleSeries
    closeLineRef.current = closeLineSeries
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
      closeLineRef.current = null
      markersRef.current = null
    }
  }, [theme])

  useEffect(() => {
    candleRef.current?.setData(chartData)
    closeLineRef.current?.setData(showCloseLine ? closeLineData : [])
    markersRef.current?.setMarkers(markers)
    chartRef.current?.timeScale().fitContent()
  }, [chartData, closeLineData, markers, showCloseLine])

  if (candles.length === 0) {
    return (
      <Box sx={{ minHeight: 180, display: 'grid', placeItems: 'center', border: 1, borderColor: 'divider', borderRadius: 1 }}>
        <Typography variant="caption" color="text.secondary">
          {emptyLabel}
        </Typography>
      </Box>
    )
  }

  return (
    <Stack spacing={1}>
      <Stack direction="row" alignItems="center" gap={1} flexWrap="wrap">
        <Chip size="small" variant="outlined" label={sourceLabel} />
        <FormControlLabel
          control={
            <Switch
              size="small"
              checked={showCloseLine}
              onChange={(event) => setShowCloseLine(event.target.checked)}
              inputProps={{ 'aria-label': '종가 선 그래프 표시' }}
            />
          }
          label="종가 선 그래프"
          sx={{
            ml: 0,
            '& .MuiFormControlLabel-label': {
              fontSize: '0.75rem',
              color: 'text.secondary',
            },
          }}
        />
      </Stack>
      <Box ref={containerRef} data-testid="lth-preview-chart" sx={{ width: '100%', minHeight: 300 }} />
      <Stack direction="row" gap={0.75} flexWrap="wrap">
        <Chip size="small" variant="outlined" label="시간축 KST" />
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

export const LeveragedTrendHoldPreviewChart = StrategyPreviewChart
