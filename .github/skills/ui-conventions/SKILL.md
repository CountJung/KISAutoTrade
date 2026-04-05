---
name: ui-conventions
description: "AutoConditionTrade 프로젝트 UI 컨벤션. MUI v6 컴포넌트 사용 규칙, 차트(lightweight-charts v5) 패턴, 레이아웃 규칙, 색상 시스템, 금융 데이터 표시 규칙. Keywords: MUI, chart, lightweight-charts, 차트, 레이아웃, 색상, 금융, React, Trading, StockChart"
---
# UI 컨벤션 스킬 — AutoConditionTrade

> 이 스킬은 `react-best-practices` 스킬과 분리된 **프로젝트 전용** 규칙입니다.

---

## 1. 기본 원칙

| 원칙 | 내용 |
|------|------|
| 컴포넌트 임포트 | MUI 아이콘은 **직접 경로** 사용 (`@mui/icons-material/TrendingUp`) |
| 스타일링 | MUI `sx` prop 우선, `styled()` 사용 금지 (인라인 sx로 충분) |
| 색상 참조 | 하드코딩 금지 → 항상 `theme.palette.*` 또는 `'primary.main'` 등 semantic 색상 사용 |
| 폰트 | `"Noto Sans KR"` (한글), `"Roboto"` (영문) — `theme.typography` 자동 적용 |
| 다크/라이트 | `useTheme()` 훅으로 `theme.palette.mode` 감지, CSS 변수 사용 금지 |

---

## 2. 레이아웃 시스템

### 페이지 기본 구조

```tsx
export default function MyPage() {
  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={3}>
        페이지 제목
      </Typography>
      {/* 콘텐츠 */}
    </Box>
  )
}
```

### Grid 2-컬럼 패턴 (주문 패널 + 차트)

```tsx
<Grid container spacing={2}>
  {/* 좌: 좁은 작업 패널 */}
  <Grid item xs={12} md={4}>
    <Paper sx={{ p: 3, height: '100%' }}>...</Paper>
  </Grid>

  {/* 우: 넓은 데이터/차트 패널 */}
  <Grid item xs={12} md={8}>
    <Paper sx={{ overflow: 'hidden' }}>...</Paper>
  </Grid>
</Grid>
```

### 사이드바

- 너비: localStorage `act:panel:sidebar:width` (기본 220px, 범위 160~400)
- `LayoutResizer` 컴포넌트로 드래그 리사이즈 지원
- 스크롤 방지: Drawer paper에 `overflowX: 'hidden'`

---

## 3. 색상 시스템 (금융 UI)

### 상승/하락 색상

```tsx
// 항상 MUI semantic 색상 사용
<Typography color={isPositive ? 'success.main' : 'error.main'}>
  {value}
</Typography>

// 차트용 캔들 색상 (라이브러리 직접 지정)
const UP_COLOR   = '#26a69a'   // 초록
const DOWN_COLOR = '#ef5350'   // 빨강
const UP_VOL     = 'rgba(38,166,154,0.45)'
const DOWN_VOL   = 'rgba(239,83,80,0.45)'
```

### 배경 색상 (다크/라이트)

| 용도 | 라이트 | 다크 |
|------|--------|------|
| Paper | `#ffffff` | `#1e1e1e` |
| 기본 배경 | `#f5f5f5` | `#121212` |
| 차트 배경 | `#ffffff` | `#1e1e1e` |
| 차트 그리드 | `#eeeeee` | `#2a2a2a` |
| 텍스트 | `#333333` | `#c8c8c8` |

---

## 4. 금융 데이터 표시

### 숫자 포맷

```tsx
// 원화 포맷 (항상 ko-KR 로케일)
function fmt(n: number) {
  return n.toLocaleString('ko-KR')
}
// 사용: fmt(75000) → "75,000"

// 퍼센트 (소수점 2자리)
changeRt.toFixed(2) + '%'

// 부호 표시 (손익)
(positive ? '+' : '') + fmt(value) + '원'
```

### 상승/하락 아이콘

```tsx
import TrendingUpIcon   from '@mui/icons-material/TrendingUp'
import TrendingDownIcon from '@mui/icons-material/TrendingDown'

{isPositive
  ? <TrendingUpIcon   fontSize="small" color="success" />
  : <TrendingDownIcon fontSize="small" color="error" />
}
```

### 모의/실전 뱃지

```tsx
<Chip
  size="small"
  label={isPaper ? '모의' : '실전'}
  color={isPaper ? 'warning' : 'success'}
  sx={{ height: 16, fontSize: '0.6rem' }}
/>
```

---

## 5. 차트 컴포넌트 — StockChart

### 라이브러리 선택 근거

**lightweight-charts v5** (TradingView OSS) 선택:
- 금융 차트 전용, OHLCV 캔들 + 거래량 네이티브 지원
- 마우스 휠 줌/패닝 내장, TypeScript-first
- 번들 크기 ~40KB (Recharts 대비 1/3)
- MUI / React 의존성 없음 → 충돌 없음

### 컴포넌트 위치

```
src/components/chart/StockChart.tsx
```

### Props

```tsx
interface StockChartProps {
  symbol: string      // 종목코드 6자리 (미만이면 빈 상태 표시)
  stockName?: string  // 종목명 (차트 툴바에 표시)
}
```

### 프리셋 정의

```typescript
// src/components/chart/StockChart.tsx 내 CHART_PRESETS
export const CHART_PRESETS = [
  { key: '1D', label: '일',  periodCode: 'D', months: 1  }, // 1달치 일봉
  { key: '3M', label: '3월', periodCode: 'D', months: 3  }, // 3달치 일봉
  { key: '6M', label: '6월', periodCode: 'W', months: 6  }, // 6달치 주봉
  { key: '1Y', label: '년',  periodCode: 'W', months: 12 }, // 1년치 주봉
  { key: '5Y', label: '5년', periodCode: 'M', months: 60 }, // 5년치 월봉
]
```

### v5 API 패턴

```typescript
import {
  createChart,
  CandlestickSeries,
  HistogramSeries,
  ColorType,
  CrosshairMode,
} from 'lightweight-charts'

// 차트 생성
const chart = createChart(containerElement, options)

// 시리즈 추가 (v5: addSeries() 사용, addCandlestickSeries() 없음)
const candleSeries = chart.addSeries(CandlestickSeries, {
  upColor:       '#26a69a',
  downColor:     '#ef5350',
  borderVisible: false,
  wickUpColor:   '#26a69a',
  wickDownColor: '#ef5350',
})

const volumeSeries = chart.addSeries(HistogramSeries, {
  priceFormat:  { type: 'volume' },
  priceScaleId: 'vol',
})
chart.priceScale('vol').applyOptions({
  scaleMargins: { top: 0.85, bottom: 0 },
  drawTicks: false,
})

// 데이터 포맷 (time은 반드시 "YYYY-MM-DD" 문자열)
candleSeries.setData([
  { time: '2026-04-01', open: 75000, high: 76500, low: 74800, close: 76000 },
])

// 다크/라이트 동기화
chart.applyOptions({
  layout: {
    background: { type: ColorType.Solid, color: isDark ? '#1e1e1e' : '#ffffff' },
    textColor: isDark ? '#c8c8c8' : '#333333',
  },
})

// 줌 컨트롤
const ts = chart.timeScale()
const range = ts.getVisibleLogicalRange()
if (range) {
  const center = (range.from + range.to) / 2
  const half   = (range.to - range.from) / 4
  ts.setVisibleLogicalRange({ from: center - half, to: center + half })  // 확대
}
ts.fitContent() // 전체 맞춤

// 언마운트 정리
chart.remove()
```

### 주의사항

- `useEffect`에서 차트를 생성할 때 의존성 배열은 `[]` (1회만 생성)
- 다크/라이트 전환은 별도 `useEffect([isDark])`로 `applyOptions` 호출
- 데이터 업데이트는 `setData()`. `update()`는 단일 캔들 업데이트용
- KIS API 응답은 최신순(내림차순) → `candles.reverse()` 필수 (Rust side에서 처리)
- `containerRef.current`에 `ResizeObserver` 연결, 언마운트 시 `disconnect()`

---

## 6. KIS 차트 API

### 엔드포인트

```
GET /uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice
TR-ID: FHKST03010100  (실전/모의 공통)
```

### 파라미터

| 파라미터 | 값 |
|---------|---|
| `FID_COND_MRKT_DIV_CODE` | `J` |
| `FID_INPUT_ISCD` | 종목코드 6자리 |
| `FID_INPUT_DATE_1` | 시작일 YYYYMMDD |
| `FID_INPUT_DATE_2` | 종료일 YYYYMMDD |
| `FID_PERIOD_DIV_CODE` | `D`(일)/`W`(주)/`M`(월) |
| `FID_ORG_ADJ_PRC` | `0` (수정주가 미적용) |

### 응답 output2 필드

| 필드 | 설명 |
|------|------|
| `stck_bsop_date` | 영업일자 YYYYMMDD |
| `stck_oprc` | 시가 |
| `stck_hgpr` | 고가 |
| `stck_lwpr` | 저가 |
| `stck_clpr` | 종가 |
| `acml_vol` | 누적거래량 |

---

## 7. IPC 데이터 흐름 (차트)

```
React useChartData(symbol, periodCode, startDate, endDate, presetKey)
  → TanStack Query (KEYS.chartData(symbol, presetKey))
  → cmd.getChartData({ symbol, period_code, start_date, end_date })
  → invoke('get_chart_data', { input })
  → Rust commands::get_chart_data()
  → KisRestClient::get_chart_data()
  → KIS REST API
  → Vec<ChartCandle> (시간순 정렬, 오름차순)
```

---

## 8. 빈 상태 / 로딩 처리

| 상태 | 처리 방법 |
|------|----------|
| 로딩 중 | `<Skeleton variant="rectangular" height={380} />` |
| 에러 | `<Alert severity="error">` + 에러 메시지 |
| 종목 미선택 | 안내 텍스트 `Box` (bgcolor: action.hover) |
| 데이터 없음 | 빈 차트 (lightweight-charts가 자동으로 빈 뷰 표시) |

---

---

## 9. TextField + Button 인라인 정렬

TextField에 `helperText`가 있으면 컴포넌트 전체 높이가 늘어나 `alignItems="center"`만으로는 Button이 입력 필드 중앙에 정렬되지 않는다.

### 원인

```
┌─ TextField ─────────────────┐   ┌─ Button ─┐
│ label                       │   │          │  ← center = helperText 위쪽
│ [ input field         ]     │   │  저장     │
│ helperText                  │   └──────────┘
└─────────────────────────────┘
```

`alignItems="center"` 시 Button의 수직 중심이 helperText 위로 올라가 시각적으로 입력 필드보다 낮게 보임.

### 해결 — helperText를 Stack 밖으로 분리

```tsx
// ✅ 권장 패턴
<Box>
  <Stack direction="row" spacing={1} alignItems="center">
    <TextField
      label="레이블"
      size="small"
      sx={{ width: 140 }}
      // helperText 제거 — Stack 밖으로 분리
    />
    <Button variant="outlined">저장</Button>
  </Stack>
  {/* 도움말은 TextField 너비 이하에 별도 표시 */}
  <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
    기본값: 7474 (재시작 후 적용)
  </Typography>
</Box>

// ❌ 잘못된 패턴 — helperText가 포함된 상태로 alignItems="center"
<Box display="flex" alignItems="center">
  <TextField helperText="..." />  {/* Button이 입력 필드보다 낮게 표시됨 */}
  <Button>저장</Button>
</Box>
```

### 요약 규칙

| 상황 | alignItems | helperText 위치 |
|------|-----------|----------------|
| TextField에 helperText 없음 | `"center"` | TextField 내부 |
| TextField에 helperText 있음 | `"center"` | Stack **밖** `<Typography variant="caption">` |

---

## 10. TanStack Query `enabled` 조건 — 검색 필터 주의사항

### 실제 발생 버그 (반복 패턴)

**증상**: "한국 주식 종목이 이름으로 검색되지 않는다" — 숫자를 포함한 이름(예: "200", "코스피200")은 검색되지 않고, 6자리 완전한 코드만 동작.

**근본 원인**: `hooks.ts`의 `useStockSearch` `enabled` 조건에 숫자 전용 쿼리 차단 필터가 포함되어 있었음.

```typescript
// ❌ 잘못된 패턴 — 숫자만으로 구성된 검색어를 차단 → 이름 검색 불가
enabled: query.length >= 2 && !/^\d+$/.test(query)
//                           ^^^^^^^^^^^^^^^^^^^^^^^^ 이 조건이 "200", "005" 등을 막음

// ✅ 올바른 패턴 — 길이만 체크, 숫자/문자 구분 없이 허용
enabled: query.length >= 2
```

**백엔드 `search_stock` 동작 (변경 불필요)**:
```rust
// 6자리 숫자 → KIS API 현재가 조회 (종목코드 직접 조회)
if query.len() == 6 && query.chars().all(|c| c.is_ascii_digit()) { ... }
// 그 외(부분 숫자 포함) → 로컬 KRX 캐시 이름 검색
// → "200" 입력 시 "KODEX200", "코스피200 ETF" 등 반환됨
```

**함께 수정해야 하는 프론트엔드 패턴**:
```tsx
// ❌ Trading.tsx useEffect — 숫자 전용 입력 시 searchQuery 설정 차단
useEffect(() => {
  if (market !== 'KR' || !inputValue || /^\d+$/.test(inputValue)) {  // ← 이 조건 제거
    setSearchQuery('')
    return
  }
  ...
}, [inputValue, market])

// ✅ 올바른 패턴
useEffect(() => {
  if (market !== 'KR' || !inputValue) {
    setSearchQuery('')
    return
  }
  const t = setTimeout(() => setSearchQuery(inputValue), 400)
  return () => clearTimeout(t)
}, [inputValue, market])
```

> 마지막 업데이트: 2026-04-05

