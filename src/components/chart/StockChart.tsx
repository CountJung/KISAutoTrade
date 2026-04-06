/**
 * StockChart — lightweight-charts v5 기반 주식 캔들 차트
 *
 * 프리셋: 일(D/1M) | 주(D/3M) | 월(W/6M) | 3월(D/3M detail) | 년(W/1Y) | 5년(M/5Y)
 * 기능  : 캔들 + 거래량, 확대/축소/전체맞춤, MUI 다크/라이트 자동 동기화
 */
import { useEffect, useRef, useState, useCallback } from 'react'
import {
  createChart,
  CandlestickSeries,
  HistogramSeries,
  LineSeries,
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
import CandlestickChartIcon from '@mui/icons-material/CandlestickChart'
import ShowChartIcon from '@mui/icons-material/ShowChart'
import { useTheme } from '@mui/material/styles'
import { useChartData } from '../../api/hooks'
import type { ChartCandle } from '../../api/types'

// ─── 프리셋 정의 ────────────────────────────────────────────────────

type PeriodCode = 'D' | 'W' | 'M'

interface ChartPreset {
  key: string
  label: string
  periodCode: PeriodCode
  /** today 기준 몇 달 전 */
  months: number
}

export const CHART_PRESETS: ChartPreset[] = [
  { key: '1D', label: '일',  periodCode: 'D', months: 1  }, // 1달치 일봉
  { key: '3M', label: '3월', periodCode: 'D', months: 3  }, // 3달치 일봉
  { key: '6M', label: '6월', periodCode: 'W', months: 6  }, // 6달치 주봉
  { key: '1Y', label: '년',  periodCode: 'W', months: 12 }, // 1년치 주봉
  { key: '5Y', label: '5년', periodCode: 'M', months: 60 }, // 5년치 월봉
]

// ─── 날짜 유틸 ─────────────────────────────────────────────────────

function todayYYYYMMDD(): string {
  return new Date().toISOString().slice(0, 10).replace(/-/g, '')
}

function monthsAgoYYYYMMDD(months: number): string {
  const d = new Date()
  d.setMonth(d.getMonth() - months)
  return d.toISOString().slice(0, 10).replace(/-/g, '')
}

/** "20260403" → "2026-04-03" */
function toISODate(yyyymmdd: string): string {
  return `${yyyymmdd.slice(0, 4)}-${yyyymmdd.slice(4, 6)}-${yyyymmdd.slice(6, 8)}`
}

// ─── 데이터 변환 ────────────────────────────────────────────────────

interface CandlePoint {
  time: Time
  open: number
  high: number
  low: number
  close: number
}

interface VolumePoint {
  time: Time
  value: number
  color: string
}

interface LinePoint {
  time: Time
  value: number
}

function toChartPoints(candles: ChartCandle[]): { candleData: CandlePoint[]; volumeData: VolumePoint[]; lineData: LinePoint[] } {
  const candleData: CandlePoint[] = []
  const volumeData: VolumePoint[] = []
  const lineData: LinePoint[] = []

  for (const c of candles) {
    if (!c.date || c.date.length !== 8) continue
    const time = toISODate(c.date) as Time
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
    lineData.push({ time, value: close })
  }

  return { candleData, volumeData, lineData }
}

// ─── 차트 생성 옵션 ────────────────────────────────────────────────

function buildChartOptions(
  width: number,
  height: number,
  isDark: boolean
) {
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

interface StockChartProps {
  symbol: string
  stockName?: string
}

// ─── 툴팁 데이터 타입 ───────────────────────────────────────────────
interface CrosshairData {
  time: string
  open: number
  high: number
  low: number
  close: number
  volume: number
}

type ChartType = 'candle' | 'line'

export function StockChart({ symbol, stockName }: StockChartProps) {
  const theme = useTheme()
  const isDark = theme.palette.mode === 'dark'

  const [preset, setPreset] = useState<ChartPreset>(CHART_PRESETS[0])
  const [chartType, setChartType] = useState<ChartType>('candle')
  const [crosshair, setCrosshair] = useState<CrosshairData | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const chartRef     = useRef<IChartApi | null>(null)
  const candleRef    = useRef<ISeriesApi<'Candlestick'> | null>(null)
  const lineRef      = useRef<ISeriesApi<'Line'> | null>(null)
  const volumeRef    = useRef<ISeriesApi<'Histogram'> | null>(null)

  const startDate = monthsAgoYYYYMMDD(preset.months)
  const endDate   = todayYYYYMMDD()

  const { data: candles, isLoading, isError, error } = useChartData(
    symbol,
    preset.periodCode,
    startDate,
    endDate,
    preset.key,
  )

  // ── 차트 초기화 (마운트 1회) ────────────────────────────────────
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const height = 380
    const chart = createChart(container, buildChartOptions(container.clientWidth, height, isDark))

    // 캔들 시리즈
    const cs = chart.addSeries(CandlestickSeries, {
      upColor:      '#26a69a',
      downColor:    '#ef5350',
      borderVisible: false,
      wickUpColor:  '#26a69a',
      wickDownColor: '#ef5350',
    })

    // 라인 시리즈 (초기 숨김)
    const ls = chart.addSeries(LineSeries, {
      color: '#2196f3',
      lineWidth: 2,
      visible: false,
    })

    // 거래량 시리즈 (아래 15% 영역)
    const vs = chart.addSeries(HistogramSeries, {
      priceFormat: { type: 'volume' },
      priceScaleId: 'vol',
    })
    chart.priceScale('vol').applyOptions({
      scaleMargins: { top: 0.85, bottom: 0 },
    })

    chartRef.current  = chart
    candleRef.current = cs
    lineRef.current   = ls
    volumeRef.current = vs

    // ── 크로스헤어 이동 → 툴팁 데이터 업데이트 ─────────────────────
    chart.subscribeCrosshairMove((param) => {
      if (!param.time || param.point === undefined || param.point.x < 0 || param.point.y < 0) {
        setCrosshair(null)
        return
      }
      const c = param.seriesData.get(cs) as CandlePoint | undefined
      const v = param.seriesData.get(vs) as { value: number } | undefined
      // 라인 모드에서는 LineSeries 데이터로 폴백
      const lp = param.seriesData.get(ls) as LinePoint | undefined
      const closeVal = c?.close ?? lp?.value ?? 0
      if (!c && !lp) { setCrosshair(null); return }
      setCrosshair({
        time: param.time as string,
        open: c?.open ?? closeVal,
        high: c?.high ?? closeVal,
        low: c?.low ?? closeVal,
        close: closeVal,
        volume: v?.value ?? 0,
      })
    })

    // 리사이즈 감지
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
      lineRef.current   = null
      volumeRef.current = null
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // ── 다크/라이트 동기화 ──────────────────────────────────────────
  useEffect(() => {
    const container = containerRef.current
    if (!chartRef.current || !container) return
    chartRef.current.applyOptions(
      buildChartOptions(container.clientWidth, 380, isDark)
    )
  }, [isDark])

  // ── 데이터 업데이트 ─────────────────────────────────────────────
  useEffect(() => {
    if (!candles || !candleRef.current || !volumeRef.current || !lineRef.current) return
    const { candleData, volumeData, lineData } = toChartPoints(candles)
    candleRef.current.setData(candleData)
    lineRef.current.setData(lineData)
    volumeRef.current.setData(volumeData)
    chartRef.current?.timeScale().fitContent()
  }, [candles])

  // ── 차트 타입 전환 (캔들 ↔ 라인) ────────────────────────────────
  useEffect(() => {
    if (!candleRef.current || !lineRef.current) return
    candleRef.current.applyOptions({ visible: chartType === 'candle' })
    lineRef.current.applyOptions({ visible: chartType === 'line' })
  }, [chartType])

  // ── 줌 / 피트 컨트롤 ──────────────────────────────────────────
  const handleZoomIn = useCallback(() => {
    const ts = chartRef.current?.timeScale()
    const range = ts?.getVisibleLogicalRange()
    if (!ts || !range) return
    const center = (range.from + range.to) / 2
    const half   = (range.to - range.from) / 4
    ts.setVisibleLogicalRange({ from: center - half, to: center + half })
  }, [])

  const handleZoomOut = useCallback(() => {
    const ts = chartRef.current?.timeScale()
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

  return (
    <Box>
      {/* 툴바 */}
      <Stack direction="row" alignItems="center" spacing={1} mb={1} flexWrap="wrap">
        {stockName && (
          <Typography variant="subtitle2" fontWeight={600} sx={{ mr: 1 }}>
            {stockName}
          </Typography>
        )}

        {/* 프리셋 버튼 */}
        <ToggleButtonGroup
          value={preset.key}
          exclusive
          onChange={(_, v) => {
            if (!v) return
            const p = CHART_PRESETS.find((x) => x.key === v)
            if (p) setPreset(p)
          }}
          size="small"
        >
          {CHART_PRESETS.map((p) => (
            <ToggleButton key={p.key} value={p.key} sx={{ px: 1.5, minWidth: 44 }}>
              {p.label}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>

        <Box sx={{ flex: 1 }} />

        {/* 차트 타입 토글 */}
        <ToggleButtonGroup
          value={chartType}
          exclusive
          onChange={(_, v) => { if (v) setChartType(v as ChartType) }}
          size="small"
        >
          <ToggleButton value="candle" sx={{ px: 1 }}>
            <Tooltip title="캔들 차트">
              <CandlestickChartIcon fontSize="small" />
            </Tooltip>
          </ToggleButton>
          <ToggleButton value="line" sx={{ px: 1 }}>
            <Tooltip title="라인 차트">
              <ShowChartIcon fontSize="small" />
            </Tooltip>
          </ToggleButton>
        </ToggleButtonGroup>

        {/* 줌 컨트롤 */}
        <Tooltip title="확대">
          <IconButton size="small" onClick={handleZoomIn}>
            <ZoomInIcon fontSize="small" />
          </IconButton>
        </Tooltip>
        <Tooltip title="축소">
          <IconButton size="small" onClick={handleZoomOut}>
            <ZoomOutIcon fontSize="small" />
          </IconButton>
        </Tooltip>
        <Tooltip title="전체 맞춤">
          <IconButton size="small" onClick={handleFit}>
            <FitScreenIcon fontSize="small" />
          </IconButton>
        </Tooltip>
      </Stack>

      {/* 차트 컨테이너 */}
      {isError ? (
        <Alert severity="error" sx={{ height: 380, alignItems: 'center' }}>
          {errMsg}
        </Alert>
      ) : isLoading ? (
        <Skeleton variant="rectangular" height={380} sx={{ borderRadius: 1 }} />
      ) : null}

      {/* lightweight-charts 렌더 타겟 + 툴팁 오버레이 */}
      <Box sx={{ position: 'relative' }}>
        <Box
          ref={containerRef}
          sx={{
            width: '100%',
            height: 380,
            display: isLoading || isError ? 'none' : 'block',
            borderRadius: 1,
            overflow: 'hidden',
          }}
        />
        {crosshair && !isLoading && !isError && (
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
            <Box>시가&nbsp;{crosshair.open.toLocaleString('ko-KR')}&ensp;고가&nbsp;{crosshair.high.toLocaleString('ko-KR')}</Box>
            <Box>저가&nbsp;{crosshair.low.toLocaleString('ko-KR')}&ensp;종가&nbsp;<Box component="span" sx={{ color: crosshair.close >= crosshair.open ? '#26a69a' : '#ef5350', fontWeight: 600 }}>{crosshair.close.toLocaleString('ko-KR')}</Box></Box>
            <Box sx={{ color: '#aaa' }}>거래량&nbsp;{Math.round(crosshair.volume).toLocaleString('ko-KR')}</Box>
          </Box>
        )}
      </Box>

      {/* 심볼 미입력 빈 상태 */}
      {symbol.length !== 6 && !isLoading && !isError && (
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
            종목코드를 입력하면 차트가 표시됩니다
          </Typography>
        </Box>
      )}
    </Box>
  )
}
