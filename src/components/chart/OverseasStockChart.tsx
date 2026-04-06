/**
 * OverseasStockChart — lightweight-charts v5 기반 해외주식 캔들 차트
 *
 * 프리셋: 1월(D/1M) | 3월(D/3M) | 6월(W/6M) | 1년(W/1Y) | 5년(M/5Y)
 * 기능  : 캔들 + 거래량, 확대/축소/전체맞춤, MUI 다크/라이트 자동 동기화
 *
 * KIS API: /uapi/overseas-price/v1/quotations/dailyprice (HHDFS76200200)
 * 약 100건/호출 — 더 긴 기간은 여러 번 호출이 필요하나 현재 미구현
 */
import { useEffect, useRef, useState, useCallback } from 'react'
import {
  createChart,
  CandlestickSeries,
  HistogramSeries,
  ColorType,
  CrosshairMode,
  type IChartApi,
  type ISeriesApi,
  type Time,
} from 'lightweight-charts'
import Box from '@mui/material/Box'
import Stack from '@mui/material/Stack'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import Typography from '@mui/material/Typography'
import Skeleton from '@mui/material/Skeleton'
import Alert from '@mui/material/Alert'
import IconButton from '@mui/material/IconButton'
import Tooltip from '@mui/material/Tooltip'
import ZoomInIcon from '@mui/icons-material/ZoomIn'
import ZoomOutIcon from '@mui/icons-material/ZoomOut'
import FitScreenIcon from '@mui/icons-material/FitScreen'
import { useTheme } from '@mui/material/styles'
import { useOverseasChartData } from '../../api/hooks'
import type { ChartCandle } from '../../api/types'

// ─── 프리셋 정의 ────────────────────────────────────────────────────

interface OverseaChartPreset {
  key: string
  label: string
}

export const OVERSEAS_CHART_PRESETS: OverseaChartPreset[] = [
  { key: '1M', label: '1월' },
  { key: '3M', label: '3월' },
  { key: '6M', label: '6월' },
  { key: '1Y', label: '1년' },
  { key: '5Y', label: '5년' },
]

// ─── 날짜 변환 유틸 ─────────────────────────────────────────────────

/** "20260403" → "2026-04-03" */
function toISODate(yyyymmdd: string): string {
  return `${yyyymmdd.slice(0, 4)}-${yyyymmdd.slice(4, 6)}-${yyyymmdd.slice(6, 8)}`
}

// ─── 데이터 변환 ────────────────────────────────────────────────────

interface CandlePoint { time: Time; open: number; high: number; low: number; close: number }
interface VolumePoint { time: Time; value: number; color: string }

function toChartPoints(candles: ChartCandle[]): { candleData: CandlePoint[]; volumeData: VolumePoint[] } {
  const candleData: CandlePoint[] = []
  const volumeData: VolumePoint[] = []

  for (const c of candles) {
    if (!c.date || c.date.length !== 8) continue
    const time  = toISODate(c.date) as Time
    const open  = parseFloat(c.open)  || 0
    const high  = parseFloat(c.high)  || 0
    const low   = parseFloat(c.low)   || 0
    const close = parseFloat(c.close) || 0
    const vol   = parseFloat(c.volume) || 0
    if (!open || !close) continue

    candleData.push({ time, open, high, low, close })
    volumeData.push({
      time,
      value: vol,
      color: close >= open ? 'rgba(38,166,154,0.45)' : 'rgba(239,83,80,0.45)',
    })
  }

  return { candleData, volumeData }
}

// ─── 차트 옵션 ──────────────────────────────────────────────────────

function buildChartOptions(width: number, height: number, isDark: boolean) {
  const bg      = isDark ? '#1e1e1e' : '#ffffff'
  const textClr = isDark ? '#c8c8c8' : '#333333'
  const grid    = isDark ? '#2a2a2a' : '#eeeeee'
  const border  = isDark ? '#333333' : '#dddddd'

  return {
    width,
    height,
    layout: {
      background: { type: ColorType.Solid, color: bg },
      textColor: textClr,
      fontSize: 11,
    },
    grid: {
      vertLines: { color: grid },
      horzLines: { color: grid },
    },
    crosshair: { mode: CrosshairMode.Normal },
    rightPriceScale: { borderColor: border },
    timeScale: {
      borderColor: border,
      timeVisible: false,
      fixLeftEdge: true,
      fixRightEdge: true,
    },
  }
}

// ─── 컴포넌트 ──────────────────────────────────────────────────────

interface OverseasStockChartProps {
  symbol: string
  exchange: string  // NAS, NYS, AMS 등
  stockName?: string
}

interface CrosshairData {
  time: string
  open: number
  high: number
  low: number
  close: number
  volume: number
}

export function OverseasStockChart({ symbol, exchange, stockName }: OverseasStockChartProps) {
  const theme  = useTheme()
  const isDark = theme.palette.mode === 'dark'

  const [presetKey, setPresetKey] = useState('3M')
  const [crosshair, setCrosshair] = useState<CrosshairData | null>(null)

  const containerRef = useRef<HTMLDivElement>(null)
  const chartRef     = useRef<IChartApi | null>(null)
  const candleRef    = useRef<ISeriesApi<'Candlestick'> | null>(null)
  const volumeRef    = useRef<ISeriesApi<'Histogram'> | null>(null)

  const { data: candles, isLoading, isError, error } = useOverseasChartData(
    symbol,
    exchange,
    presetKey,
  )

  // ── 차트 초기화 (마운트 1회) ────────────────────────────────────
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const height = 380
    const chart = createChart(container, buildChartOptions(container.clientWidth, height, isDark))

    const cs = chart.addSeries(CandlestickSeries, {
      upColor:       '#26a69a',
      downColor:     '#ef5350',
      borderVisible: false,
      wickUpColor:   '#26a69a',
      wickDownColor: '#ef5350',
    })

    const vs = chart.addSeries(HistogramSeries, {
      priceFormat: { type: 'volume' },
      priceScaleId: 'vol',
    })
    chart.priceScale('vol').applyOptions({
      scaleMargins: { top: 0.85, bottom: 0 },
    })

    chartRef.current  = chart
    candleRef.current = cs
    volumeRef.current = vs

    // ── 크로스헤어 이동 → 툴팁 데이터 업데이트 ─────────────────────
    chart.subscribeCrosshairMove((param) => {
      if (!param.time || param.point === undefined || param.point.x < 0 || param.point.y < 0) {
        setCrosshair(null)
        return
      }
      const c = param.seriesData.get(cs) as CandlePoint | undefined
      const v = param.seriesData.get(vs) as { value: number } | undefined
      if (!c) { setCrosshair(null); return }
      setCrosshair({
        time: param.time as string,
        open: c.open,
        high: c.high,
        low: c.low,
        close: c.close,
        volume: v?.value ?? 0,
      })
    })

    const ro = new ResizeObserver(() => {
      if (container && chartRef.current) {
        chartRef.current.applyOptions({ width: container.clientWidth })
      }
    })
    ro.observe(container)

    return () => {
      ro.disconnect()
      chart.remove()
      chartRef.current  = null
      candleRef.current = null
      volumeRef.current = null
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // ── 다크/라이트 동기화 ──────────────────────────────────────────
  useEffect(() => {
    const container = containerRef.current
    if (!chartRef.current || !container) return
    chartRef.current.applyOptions(buildChartOptions(container.clientWidth, 380, isDark))
  }, [isDark])

  // ── 데이터 업데이트 ─────────────────────────────────────────────
  useEffect(() => {
    if (!candles || !candleRef.current || !volumeRef.current) return
    const { candleData, volumeData } = toChartPoints(candles)
    candleRef.current.setData(candleData)
    volumeRef.current.setData(volumeData)
    chartRef.current?.timeScale().fitContent()
  }, [candles])

  // ── 줌 / 피트 컨트롤 ──────────────────────────────────────────
  const handleZoomIn = useCallback(() => {
    const ts    = chartRef.current?.timeScale()
    const range = ts?.getVisibleLogicalRange()
    if (!ts || !range) return
    const center = (range.from + range.to) / 2
    const half   = (range.to - range.from) / 4
    ts.setVisibleLogicalRange({ from: center - half, to: center + half })
  }, [])

  const handleZoomOut = useCallback(() => {
    const ts    = chartRef.current?.timeScale()
    const range = ts?.getVisibleLogicalRange()
    if (!ts || !range) return
    const center = (range.from + range.to) / 2
    const span   = (range.to - range.from)
    ts.setVisibleLogicalRange({ from: center - span, to: center + span })
  }, [])

  const handleFit = useCallback(() => {
    chartRef.current?.timeScale().fitContent()
  }, [])

  // ── 렌더 ────────────────────────────────────────────────────────
  const errMsg = (error as { message?: string } | null)?.message ?? '차트 데이터 조회 실패'
  const isEmpty = !isLoading && !isError && (!candles || candles.length === 0)

  return (
    <Box>
      {/* 툴바 */}
      <Stack direction="row" alignItems="center" spacing={1} mb={1} flexWrap="wrap">
        {stockName && (
          <Typography variant="subtitle2" fontWeight={600} sx={{ mr: 1 }}>
            {stockName}
          </Typography>
        )}

        <ToggleButtonGroup
          value={presetKey}
          exclusive
          onChange={(_, v) => { if (v) setPresetKey(v) }}
          size="small"
        >
          {OVERSEAS_CHART_PRESETS.map((p) => (
            <ToggleButton key={p.key} value={p.key} sx={{ px: 1.5, minWidth: 44 }}>
              {p.label}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>

        <Box sx={{ flex: 1 }} />

        <Tooltip title="확대">
          <IconButton size="small" onClick={handleZoomIn}><ZoomInIcon fontSize="small" /></IconButton>
        </Tooltip>
        <Tooltip title="축소">
          <IconButton size="small" onClick={handleZoomOut}><ZoomOutIcon fontSize="small" /></IconButton>
        </Tooltip>
        <Tooltip title="전체 맞춤">
          <IconButton size="small" onClick={handleFit}><FitScreenIcon fontSize="small" /></IconButton>
        </Tooltip>
      </Stack>

      {/* 상태 표시 */}
      {isError ? (
        <Alert severity="error" sx={{ height: 380, alignItems: 'center' }}>
          {errMsg}
        </Alert>
      ) : isLoading ? (
        <Skeleton variant="rectangular" height={380} sx={{ borderRadius: 1 }} />
      ) : isEmpty ? (
        <Box
          sx={{
            height: 380,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            bgcolor: 'action.hover',
            borderRadius: 1,
          }}
        >
          <Typography variant="body2" color="text.secondary">
            차트 데이터가 없습니다 (장외 시간이거나 API 미지원 심볼)
          </Typography>
        </Box>
      ) : null}

      {/* lightweight-charts 렌더 타겟 + 툴팁 오버레이 */}
      <Box sx={{ position: 'relative' }}>
        <Box
          ref={containerRef}
          sx={{
            width: '100%',
            height: 380,
            display: isLoading || isError || isEmpty ? 'none' : 'block',
            borderRadius: 1,
            overflow: 'hidden',
          }}
        />
        {crosshair && !isLoading && !isError && !isEmpty && (
          <Box sx={{
            position: 'absolute', top: 8, left: 8,
            bgcolor: 'rgba(0,0,0,0.72)',
            color: '#fff',
            borderRadius: 1,
            px: 1.5, py: 0.75,
            pointerEvents: 'none',
            zIndex: 10,
            fontSize: '11px',
            lineHeight: 1.7,
            minWidth: 160,
          }}>
            <Box sx={{ fontWeight: 600, mb: 0.2 }}>{crosshair.time}</Box>
            <Box>O&nbsp;{crosshair.open.toFixed(2)}&ensp;H&nbsp;{crosshair.high.toFixed(2)}</Box>
            <Box>L&nbsp;{crosshair.low.toFixed(2)}&ensp;C&nbsp;<Box component="span" sx={{ color: crosshair.close >= crosshair.open ? '#26a69a' : '#ef5350', fontWeight: 600 }}>{crosshair.close.toFixed(2)}</Box></Box>
            <Box sx={{ color: '#aaa' }}>Vol&nbsp;{Math.round(crosshair.volume).toLocaleString()}</Box>
          </Box>
        )}
      </Box>
    </Box>
  )
}
