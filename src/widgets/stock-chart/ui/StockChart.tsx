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
import { alpha, useTheme, type Theme } from '@mui/material/styles'
import { useChartData, useTossChartData } from '../../../api/hooks'
import type { ChartCandle } from '../../../api/types'

// ─── 프리셋 정의 ────────────────────────────────────────────────────

type PeriodCode = 'D' | 'W' | 'M'
type TossInterval = '1d' | '1m'
type ChartSource = 'kis' | 'toss'

interface ChartPreset {
  key: string
  label: string
  periodCode?: PeriodCode
  tossInterval?: TossInterval
  /** today 기준 몇 달 전 */
  months?: number
}

export const CHART_PRESETS: ChartPreset[] = [
  { key: '1D', label: '일',  periodCode: 'D', months: 1  }, // 1달치 일봉
  { key: '3M', label: '3월', periodCode: 'D', months: 3  }, // 3달치 일봉
  { key: '6M', label: '6월', periodCode: 'W', months: 6  }, // 6달치 주봉
  { key: '1Y', label: '년',  periodCode: 'W', months: 12 }, // 1년치 주봉
  { key: '5Y', label: '5년', periodCode: 'M', months: 60 }, // 5년치 월봉
]

export const TOSS_CHART_PRESETS: ChartPreset[] = [
  { key: '1D', label: '일', tossInterval: '1d' },
  { key: '1M', label: '1분', tossInterval: '1m' },
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

function toChartTime(value: string): Time | null {
  if (/^\d{8}$/.test(value)) {
    return toISODate(value) as Time
  }
  if (/^\d{12,14}$/.test(value)) {
    const year = value.slice(0, 4)
    const month = value.slice(4, 6)
    const day = value.slice(6, 8)
    const hour = value.slice(8, 10)
    const minute = value.slice(10, 12)
    const second = value.slice(12, 14) || '00'
    const parsed = Date.parse(`${year}-${month}-${day}T${hour}:${minute}:${second}+09:00`)
    return Number.isFinite(parsed) ? Math.floor(parsed / 1000) as Time : null
  }
  const parsed = Date.parse(value)
  return Number.isFinite(parsed) ? Math.floor(parsed / 1000) as Time : null
}

function formatChartTime(value: Time): string {
  if (typeof value === 'number') {
    return new Date(value * 1000).toLocaleString('ko-KR', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }
  if (typeof value === 'string') return value
  return `${value.year}-${String(value.month).padStart(2, '0')}-${String(value.day).padStart(2, '0')}`
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
    if (!c.date) continue
    const time = toChartTime(c.date)
    if (!time) continue
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
  theme: Theme,
  timeVisible = false
) {
  const bg      = theme.palette.background.paper
  const textClr = theme.palette.text.primary
  const grid    = alpha(theme.palette.text.primary, theme.palette.mode === 'dark' ? 0.14 : 0.1)
  const border  = theme.palette.divider

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
      timeVisible,
      secondsVisible: false,
      fixLeftEdge: true,
      fixRightEdge: true,
    },
  }
}

// ─── 컴포넌트 ──────────────────────────────────────────────────────

interface StockChartProps {
  symbol: string
  stockName?: string
  source?: ChartSource
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

export function StockChart({ symbol, stockName, source = 'kis' }: StockChartProps) {
  const theme = useTheme()

  const [presetKey, setPresetKey] = useState('1D')
  const [chartType, setChartType] = useState<ChartType>('candle')
  const [crosshair, setCrosshair] = useState<CrosshairData | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const chartRef     = useRef<IChartApi | null>(null)
  const candleRef    = useRef<ISeriesApi<'Candlestick'> | null>(null)
  const lineRef      = useRef<ISeriesApi<'Line'> | null>(null)
  const volumeRef    = useRef<ISeriesApi<'Histogram'> | null>(null)

  const presets = source === 'toss' ? TOSS_CHART_PRESETS : CHART_PRESETS
  const preset = presets.find((item) => item.key === presetKey) ?? presets[0]
  const startDate = monthsAgoYYYYMMDD(preset.months ?? 1)
  const endDate   = todayYYYYMMDD()
  const kisPeriodCode = preset.periodCode ?? 'D'
  const tossInterval = preset.tossInterval ?? '1d'
  const hasValidSymbol = source === 'toss' ? !!symbol : symbol.length === 6

  useEffect(() => {
    if (!presets.some((item) => item.key === presetKey)) {
      setPresetKey(presets[0].key)
    }
  }, [presetKey, presets])

  const kisChart = useChartData(
    symbol,
    kisPeriodCode,
    startDate,
    endDate,
    preset.key,
    { enabled: source === 'kis' && symbol.length === 6 },
  )
  const tossChart = useTossChartData(
    symbol,
    tossInterval,
    preset.key,
    { enabled: source === 'toss' && !!symbol },
  )
  const { data: candles, isLoading, isError, error } = source === 'toss' ? tossChart : kisChart

  // ── 차트 초기화 (마운트 1회) ────────────────────────────────────
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const height = 380
    const chart = createChart(container, buildChartOptions(container.clientWidth, height, theme))

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
      color: theme.palette.primary.main,
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
        time: formatChartTime(param.time as Time),
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
      buildChartOptions(container.clientWidth, 380, theme)
    )
    lineRef.current?.applyOptions({ color: theme.palette.primary.main })
  }, [theme])

  // ── 데이터 업데이트 ─────────────────────────────────────────────
  useEffect(() => {
    if (!candles || !candleRef.current || !volumeRef.current || !lineRef.current) return
    const { candleData, volumeData, lineData } = toChartPoints(candles)
    candleRef.current.setData(candleData)
    lineRef.current.setData(lineData)
    volumeRef.current.setData(volumeData)
    chartRef.current?.applyOptions({
      timeScale: {
        timeVisible: candleData.some((point) => typeof point.time === 'number'),
        secondsVisible: false,
      },
    })
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
            if (presets.some((x) => x.key === v)) setPresetKey(v)
          }}
          size="small"
        >
          {presets.map((p) => (
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
      {!hasValidSymbol && !isLoading && !isError && (
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
