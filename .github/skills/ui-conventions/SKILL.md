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
| 색상 참조 | 하드코딩 금지 → 항상 `theme.palette.*` 또는 `'primary.main'`, `'background.paper'`, `'text.secondary'` 등 MUI semantic 색상 사용 |
| 폰트 | `"Noto Sans KR"` (한글), `"Roboto"` (영문) — `theme.typography` 자동 적용 |
| 다크/라이트 | `useTheme()` 훅으로 `theme.palette.mode` 감지, CSS 변수 사용 금지 |

---

## 2. 레이아웃 시스템

### Playwright/browser 검증 트리거

다음 UI 작업은 Playwright 또는 실제 브라우저 확인 대상이다.

- 앱 shell, sidebar, main scroll container, panel width/height, overflow, sticky/header/footer 레이아웃 변경
- Dialog, Drawer, ResizableDialog, LayoutResizer, drag/resize/rail click/pointer interaction 변경
- 차트, canvas, lightweight-charts, ResizeObserver, theme 전환 시각 상태 변경
- 주문·전략·설정처럼 사용자가 버튼/입력/상태를 한 화면에서 판단해야 하는 workflow 변경
- loading/error/empty 상태, disabled/enabled gate, broker/account scope, live-trading consent 표시 변경

검증은 `npm run test:e2e`를 기본으로 하고, 기존 spec이 없으면 route와 상호작용을 mock API로 고정한 focused spec을 `tests/e2e/`에 추가한다. 실행할 수 없으면 최종 보고에 미실행 사유와 남은 UI 리스크를 적는다.

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

- 너비: localStorage `act:panel:sidebar:width` (기본 220px, 범위 160~400). 읽기/쓰기는 `src/shared/lib/persistentLayout.ts` helper를 사용한다.
- `LayoutResizer` 컴포넌트로 드래그 리사이즈 지원
- 스크롤 방지: Drawer paper에 `overflowX: 'hidden'`
- 자동매매 시작/정지처럼 앱 어디서든 즉시 접근해야 하는 전역 동작은 좌측 사이드바의 실행 상태 chip 옆에 icon button으로 둔다. 버튼은 `useTradingStatus`와 같은 Query cache를 갱신하는 `useStartTrading`/`useStopTrading` 훅을 사용하고, pending 상태에서는 spinner와 disabled 상태를 표시한다.

### 조절 가능한 패널

- Log 화면 높이처럼 사용자가 드래그로 조절하는 내부 패널은 `LayoutResizer`와 `persistentLayout` helper를 함께 사용한다.
- 저장 키는 `act:panel:{panelName}:width|height` 형식을 유지한다. 예: `act:panel:log:height`.
- Tauri 네이티브 창 위치/크기는 React localStorage가 아니라 `tauri-plugin-window-state`가 담당한다.
- 주문/전략/설정처럼 카드 안의 컨트롤을 사용자가 조작해야 하는 영역은 기본 레이아웃에서 버튼, 입력값, 상태, 핵심 금액/수량이 모두 보여야 한다. 리사이저는 가려진 UI를 보정하는 보조 수단이지 기본 가시성의 대체재가 아니다.
- 좌측 주문 패널, 접수 주문 카드, 전략 편집 카드처럼 화면 폭·높이에 따라 내용이 눌릴 수 있는 카드/패널은 `LayoutResizer` 적용을 먼저 검토한다. 사용자가 카드 영역을 직접 넓히거나 높일 수 있게 할 때는 `persistentLayout`으로 `act:panel:{scope}:{name}:width|height` 키를 저장하고, min/max 범위를 정해 다른 주요 화면을 밀어내지 않게 한다.
- 좁은 영역에서 다열 테이블, 긴 주문번호, 긴 종목명, 여러 버튼이 동시에 필요한 경우는 카드형 세로 레이아웃, 줄임표+tooltip, full-width primary action을 먼저 적용한다. 그래도 운영상 더 많은 정보를 한 번에 확인해야 하는 영역은 리사이저와 내부 스크롤을 함께 제공한다.

---

## 3. 색상 시스템 (금융 UI)

### 기본 컴포넌트 색상

- 앱 전역 색상은 `src/shared/config/theme/index.ts`의 MUI theme가 source of truth다. 페이지/컴포넌트는 가능하면 `bgcolor: 'background.default'`, `bgcolor: 'background.paper'`, `color: 'text.primary'`, `borderColor: 'divider'`처럼 palette token을 사용한다.
- 다크 모드에서 `#000`, `black`, `#111` 같은 순수 검정 배경/스크롤바를 직접 지정하지 않는다. 필요하면 `alpha(theme.palette.text.primary, n)` 또는 `alpha(theme.palette.background.paper, n)`처럼 MUI palette 기반으로 만든다.
- 스크롤바는 `MuiCssBaseline` 전역 styleOverrides에서 palette 기반 색상으로 관리한다. 개별 컴포넌트가 `::-webkit-scrollbar`를 직접 덮어써야 할 때도 thumb/track은 반드시 `theme.palette.text.primary`, `background.default`, `background.paper`, `divider`에서 파생한다.
- 앱의 주 스크롤 컨테이너는 초기 렌더부터 스크롤바 영역이 잡히도록 `overflowY: 'scroll'`, `scrollbarGutter: 'stable both-edges'`, flex item `minHeight: 0`을 함께 둔다. macOS/Chromium overlay scrollbar처럼 native 폭이 0일 수 있는 환경에서는 AppShell의 palette 기반 scroll rail/thumb를 함께 표시한다. 보이는 custom rail/thumb는 장식으로만 두지 말고 pointer drag와 rail click으로 실제 `scrollTop`을 제어할 수 있어야 하며, Playwright에서 overflow, rail/thumb 가시성, thumb 드래그 후 `scrollTop` 증가를 함께 검증한다.
- AppShell의 주 스크롤 컨테이너는 route별 `scrollTop`을 기억하고 복원한다. 짧은 페이지로 이동하며 같은 컨테이너가 0으로 clamp되어도 긴 페이지로 돌아오면 이전 위치가 복원되어야 한다. 비동기 콘텐츠로 높이가 늦게 늘어나는 페이지는 ResizeObserver 갱신 중 pending restore를 재시도하고, Playwright에서 긴 페이지 → 짧은 페이지 → 긴 페이지 복귀 시나리오를 검증한다.
- Alert, Chip, Button, ToggleButton, TextField, Paper, Table, Drawer 등 MUI 컴포넌트는 기본 variant/color를 우선 사용한다. 금융 상승/하락, 경고/성공/오류 외 색상은 semantic token으로 표현하고, 브랜드 장식용 임의 색상은 피한다.

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

| 용도 | MUI token |
|------|-----------|
| Paper | `theme.palette.background.paper` |
| 기본 배경 | `theme.palette.background.default` |
| 차트 배경 | `theme.palette.background.paper` |
| 차트 그리드 | `alpha(theme.palette.text.primary, 0.10~0.14)` |
| 구분선/차트 축 | `theme.palette.divider` |
| 텍스트 | `theme.palette.text.primary` / `theme.palette.text.secondary` |

`src/widgets/stock-chart`처럼 MUI 컴포넌트가 아닌 라이브러리에 색상을 직접 전달해야 하는 경우도 `useTheme()`로 palette token을 읽어 전달한다. 상승/하락 캔들 색상처럼 금융 차트 표준색이 필요한 경우만 예외적으로 고정 색상을 허용한다.

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

### Broker scope 표시

- Dashboard, Trading, Strategy, History처럼 broker/account 혼동이 주문·기록 해석에 영향을 주는 화면은 제목 영역에 `src/shared/ui/BrokerScopeIndicator`를 배치한다.
- 표시값은 `useAppConfig()`의 `active_broker_id`, `active_profile_name`, `active_broker_account_id`, `kis_is_paper_trading`을 사용한다.
- 페이지별로 임의 Chip 조합을 새로 만들지 말고 공통 컴포넌트를 재사용해 broker/profile/account 표시 순서와 색상을 유지한다.
- Strategy 카드처럼 저장 데이터 자체에 broker/account scope가 있는 경우 카드 header에 해당 scope chip을 표시하고, 현재 활성 scope와 다르면 `warning` 색상으로 표시한다.
- Settings의 계좌 프로파일 관리는 KIS/Toss 섹션을 분리한다. KIS 계좌번호와 Toss `accountSeq`는 같은 문자열 필드에 저장되더라도 UI에서는 같은 목록·같은 broker 선택 폼으로 섞지 않는다.
- 활성 broker가 Toss인 Dashboard/Trading/Strategy는 KIS 잔고·시세·주문 흐름을 호출하지 않고 Toss 보유종목/시세, 주문 전 검증, 실거래 동의 상태를 명확히 표시한다.
- Dashboard의 Toss 소액 수동매매 검증 gate는 `src/features/manual-order`의 공유 컴포넌트를 사용한다. Dashboard는 검색 종목 1주 시장가 매수 조건을 사전검증하고, 실거래 동의·최종 확인 checkbox·최대 허용금액을 같은 화면에서 보여준 뒤 Dashboard 전용 버튼으로만 실제 소액매매 검증을 제출한다. Trading은 일반 주문 패널에서 `TossOrderPreflightPanel`을 표시하고 `canSubmit=true`일 때 수동 주문 버튼을 활성화한다. Strategy/자동매매 화면에는 소액매매 검증 UI를 두지 않는다.
- Trading의 Toss 수동 주문창은 접수 주문 목록을 같은 패널 안에서 보여준다. 좁은 좌측 주문 패널에서는 다열 테이블을 쓰지 말고 주문별 카드로 종목, 매수/매도, 주문가, 수량, 체결수량, 상태를 표시한다. 가격 정정 UI는 카드 안의 전체 폭 입력 폼으로 제공해 주문 ID나 컬럼 폭 때문에 가격/정정 버튼이 가려지지 않게 한다. 접수 주문 정정 성공/실패는 Alert로 표시하고, 토큰·secret·계좌 원문은 노출하지 않는다.

### Provider trace 표시

- History/Log처럼 provider 문의·디버깅 식별자를 보여주는 화면은 `src/shared/ui/ProviderTraceChips`를 재사용한다.
- KIS `tr_id`/`odno`, Toss `requestId`/order id는 짧은 chip으로 표시하고 full value는 tooltip에 둔다.
- 토큰, app secret, 계좌 원문 등 민감 정보는 chip과 로그 메시지에 표시하지 않는다.

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
    background: { type: ColorType.Solid, color: theme.palette.background.paper },
    textColor: theme.palette.text.primary,
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
- 다크/라이트 전환은 별도 `useEffect([theme])`에서 palette 기반 `applyOptions`를 호출
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
// 6자리 영숫자 코드 입력 → KIS API 현재가 조회 (종목코드 직접 조회, 0005A0 등 ETF 포함)
if query.len() == 6 && query.chars().all(|c| c.is_ascii_alphanumeric()) { ... }
// 그 외(부분 코드 포함) → 로컬 KRX 캐시 이름 검색
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

---

## 해외 잔고 통화 표시 패턴

KIS TTTS3012R은 USD 기준 잔고를 반환한다. KRW 환산은 **하드코딩 상수 금지** — `useExchangeRateStatus()` 또는 호환용 `useExchangeRate()` 훅으로 실시간 환율을 가져온다.

```tsx
// ❌ 잘못된 패턴: KRW_RATE 상수 하드코딩
const KRW_RATE = 1450  // ← 절대 사용 금지

// ✅ 올바른 패턴: 동적 환율 + 출처 표시
const { data: exchangeRateKrwLegacy = 1450 } = useExchangeRate()
const { data: exchangeRateStatus } = useExchangeRateStatus()
const exchangeRateKrw = exchangeRateStatus?.rate ?? exchangeRateKrwLegacy

<Chip
  size="small"
  label={`USD/KRW ${Math.round(exchangeRateKrw).toLocaleString('ko-KR')} · ${exchangeRateSourceLabel(exchangeRateStatus?.source)}`}
  color={exchangeRateStatus?.fallbackUsed ? 'warning' : 'default'}
/>

// USD/KRW 토글 상태 — 상위 컴포넌트에서 관리
const [overseasCurrency, setOverseasCurrency] = useState<'USD' | 'KRW'>('USD')

// 토글 버튼: ButtonGroup 없이 인접 Button 2개로 구현
<Box sx={{ ml: 'auto', display: 'flex', gap: 0.5 }}>
  <Button size="small" variant={overseasCurrency === 'USD' ? 'contained' : 'outlined'}
    onClick={() => setOverseasCurrency('USD')} sx={{ minWidth: 48, px: 1 }}>USD</Button>
  <Button size="small" variant={overseasCurrency === 'KRW' ? 'contained' : 'outlined'}
    onClick={() => setOverseasCurrency('KRW')} sx={{ minWidth: 48, px: 1 }}>KRW</Button>
</Box>

// 값 표시: 컴포넌트 내부 헬퍼 함수 (state + 동적 환율 클로저 활용)
const fmtFx = (usdStr: string) => {
  const v = parseFloat(usdStr)
  return overseasCurrency === 'USD'
    ? `$${v.toFixed(2)}`
    : `${Math.round(v * exchangeRateKrw).toLocaleString('ko-KR')}원`
}
```

❌ **잘못된 패턴**: KRW 환산 시 환율 상수 하드코딩 또는 KRW_RATE 상수 사용  
✅ **올바른 패턴**: `useExchangeRateStatus()` 훅 → Toss 활성 프로파일은 Toss `exchange-rate` 우선, 실패 시 open.er-api.com, 둘 다 실패하면 캐시/기본값 1450. 기존 숫자만 필요한 곳은 `useExchangeRate()` 호환 훅 사용 가능.

---

## 공통 갱신 주기 패턴 (REFRESH_INTERVAL_SEC)

`REFRESH_INTERVAL_SEC` 환경변수(기본 30초, 최소 5초)로 가격/잔고/환율 전체 갱신 주기를 제어한다. 환율은 숫자 이벤트 `exchange-rate-updated`와 출처/유효시간 이벤트 `exchange-rate-status-updated`를 함께 발행한다.

```tsx
// Dashboard에서 동적 인터벌 사용
const { data: refreshIntervalSec = 30 } = useRefreshInterval()
const intervalMs = refreshIntervalSec * 1000

const { data: balance } = useBalance({ refetchInterval: intervalMs })
const { data: overseasBalance } = useOverseasBalance({ refetchInterval: intervalMs })
const { data: stats } = useTodayStats({ refetchInterval: intervalMs })
```

| 환경변수 | 기본값 | 최소값 | 역할 |
|-----------|------|------|------|
| `REFRESH_INTERVAL_SEC` | 30 | 5 | 가격/잔고/환율 전체 갱신 주기(초) |
| `WEB_PORT` | 7474 | — | 모바일 웹서버 포트 |

---

## 대시보드 패널 확장/축소 패턴

**리스크 관리처럼 항상 표시해야 하는 패널**은 `Collapse`를 사용하지 않는다.  
대신 기능 ON/OFF 버튼을 패널 내부에 배치한다.

```tsx
// ❌ 잘못된 패턴 — 리스크 관리 같은 중요 패널에 Collapse 사용
<Stack onClick={() => setExpanded(v => !v)} sx={{ cursor: 'pointer' }}>
  <Typography>리스크 관리</Typography>
  <IconButton><ExpandMoreIcon /></IconButton>
</Stack>
<Collapse in={expanded}><RiskPanel /></Collapse>

// ✅ 올바른 패턴 — 항상 펼쳐서 표시, 기능은 버튼으로 제어
<Stack direction="row" alignItems="center" spacing={1} mb={1.5}>
  <Typography variant="subtitle1" fontWeight={600}>리스크 관리</Typography>
  <Tooltip title="..."><InfoOutlinedIcon /></Tooltip>
</Stack>
<Divider sx={{ mb: 1.5 }} />
<RiskPanel />  {/* 항상 표시 */}
```

비상정지 상태 토글 버튼 패턴:
```tsx
// 비상정지 상태에 따라 다른 버튼 표시
<Stack direction="row" justifyContent="space-between" alignItems="center" mt={1.5}>
  <Typography variant="caption" color={
    risk.isEmergencyStop ? 'error.main' : risk.canTrade ? 'success.main' : 'warning.main'
  }>
    {risk.isEmergencyStop ? '🚫 비상정지 활성' : risk.canTrade ? '✅ 거래 가능' : '⚠️ 거래 불가'}
  </Typography>
  {!risk.isEmergencyStop && (
    <Button variant="outlined" color="warning" size="small"
      startIcon={<WarningAmberIcon fontSize="small" />}
      onClick={() => activateStop()}
    >비상정지 발동</Button>
  )}
</Stack>
```

> 마지막 업데이트: 2026-07-04T12:48:33+09:00

---

## 레버리지 ETF 단일 티커 편집 UI

`LeveragedTrendHoldStrategy` 설정은 단일 ticker 목록으로 보여준다. 롱/숏 레버리지 ETF 모두 해당 ETF 자체가 상승 추세이면 매수하고, 자체 추세가 훼손되면 청산한다.

UI 규칙:

- 레버리지 전략 섹션 안에 전용 ETF 검색기와 `대상 추가` 버튼을 둔다. 상단 공용 종목 선택 패널을 거치지 않고 국내 ETF 검색 또는 미국 티커 조회로 바로 대상 ticker를 추가한다.
- 미국 티커 조회는 KIS 해외 현재가로 이름/가격을 우선 검증하되, 현재가 API 실패가 대상 등록을 막지 않게 한다. 영문으로 시작하고 영문/숫자/`.`/`-` 형식을 통과하면 직접 선택 fallback을 제공하고 warning으로 검증 실패를 알린다.
- `운용 모드`, `기초지수`, `롱`, `숏`, `숏 실험`, `유사기초` 슬롯은 표시하지 않는다. 방향성은 ticker 자체 가격 추세로만 판단한다.
- 세팅된 대상 테이블은 시장, ticker, 종목명, 1회 수량만 표시한다.
- 진입 민감도는 레버리지 섹션 내부 파라미터로 노출한다. 기본 1.0, 범위 1.0~5.0, 높을수록 상승 진입 신호가 더 민감해진다.
- 전략 파라미터 중 퍼센트/배율/민감도처럼 소수 조정이 필요한 숫자 입력은 spinner 화살표 `step`을 0.1로 둔다. 기간, 관측치, 수량처럼 정수 의미가 있는 입력만 `step: 1`을 사용한다.
- 청산 파라미터는 같은 영역에 모아 노출한다. `initial_stop_loss_pct`/`entry_failure_observations`는 반등 실패 시 작게 손절하는 초기 방어이고, `trailing_activation_profit_pct`/`breakeven_buffer_pct`/`trailing_stop_pct`/`min_hold_observations`는 반등 성공 후 수익 보호 방어다.
- 청산 도움말은 “반등이 틀리면 초기 손절/실패 판정으로 먼저 빠지고, 활성 수익을 넘긴 뒤에는 본전 보호와 추적손절로 수익을 지킨다”는 순서를 설명한다.
- 급반등 단독 진입은 기존 장중 반동 진입과 다른 블록으로 노출한다. 라벨은 `급반등 단독 진입 사용`, 입력은 `최근 관측치`, `선행 급락(%)`, `저점 회복(%)`, `저점 후 허용 관측치`를 사용한다. 도움말에는 “추세/ADX 조건 없이 최근 관측 구간의 급락 후 빠른 회복만으로 진입한다”는 점과 초기 손절/수익 보호를 함께 확인해야 한다는 점을 적는다.
- 저장 버튼은 대상 ticker가 하나 이상 있고 비어 있는 ticker가 없을 때만 활성화한다.
- 기존 저장 JSON 호환 때문에 `inverse_*`, `base_*`, `base_symbol_roles` 필드는 타입에 남아 있을 수 있으나 새 UI에서는 노출하지 않는다.
- 전략 미리보기 차트에서 Toss `YYYYMMDDHHmmss` 형태의 1분봉 시간은 한국시간(KST)으로 명시 파싱하고, lightweight-charts `timeFormatter`/`tickMarkFormatter`도 KST 표시로 고정한다. 브라우저 로컬 타임존이나 UTC 기본 formatter에 맡기면 데이마켓/프리마켓 시간표가 사용자가 보는 Toss 앱 시간과 어긋날 수 있다.
- 레버리지 전략 미리보기 차트는 Toss 1분봉 캔들 위에 종가 선 그래프를 함께 표시할 수 있어야 한다. 캔들 몸통/꼬리와 Toss 앱 선 그래프의 체감 차이가 신호 해석에 영향을 주므로, 기본은 종가선을 켜고 사용자가 스위치로 끌 수 있게 한다.

## 전략 카드 연구 작업 공간

- Strategy 페이지의 모든 전략 카드는 데스크톱에서도 `Grid item xs={12}` 전체 너비를 사용한다. 일반 전략만 `md={6}`으로 줄이면 파라미터와 미리보기 차트를 동시에 읽기 어렵다.
- 일반 전략 파라미터는 `xs={12} sm={6} md={4}`로 배치하고 TextField에 `fullWidth`를 적용한다.
- 미리보기 티커 Select와 계산 버튼은 좁은 화면에서 세로로 쌓고, 데스크톱에서만 가로로 배치한다.
- 단기 구간 미리보기는 KIS 일/주/월봉, Toss 1분/일봉을 제공하고 분석 구간은 최근 50/100/200봉으로 분리한다. `봉 단위`와 `분석 구간`을 한 프리셋 이름으로 합쳐 사용자가 일봉과 1년 범위를 혼동하게 하지 않는다.
- 강한 종가·변동성 확장의 generic 미리보기는 일봉만 선택할 수 있다. 전일 조건→다음 평가일과 당일 OHLC→이전 N일 평균 범위의 차이를 안내하고, 같은 종가/다음 관측 시가 실행 모델과 장중 순서 미검증을 표시하며 실제 체결 재현으로 표현하지 않는다.
- 티커·봉 단위·분석 구간·파라미터·수량·broker가 바뀌면 이전 미리보기 결과를 무효화한다. 진행 중 요청의 결과도 generation/key가 현재 입력과 일치할 때만 표시해 stale chart를 막는다.
- lightweight-charts 미리보기는 `horzTouchDrag: true`, `vertTouchDrag: false`, `pinch: true`로 설정하고 컨테이너 `touchAction: pan-y`를 유지한다. 모바일에서 세로 한 손가락 이동은 페이지 스크롤, 가로 이동은 차트 패닝, 두 손가락은 확대/축소로 분리하며 확대·축소·전체 맞춤 버튼도 제공한다.
- Playwright에서 모든 전략 카드의 x/width가 동일한지와 좁은 viewport에서 입력/Select/버튼이 카드 밖으로 넘치지 않는지 검증한다.
- 연구 패널은 초기자본, 수수료/세금/슬리피지 bps, USD/KRW, 종목 최대 비중, 일일 손실 한도, in-sample 비율을 카드 전체 너비에서 입력받는다. 입력·티커·broker/account가 바뀌면 기존 결과와 진행 중 응답을 무효화한다.
- 결과는 raw signal, 주문 가능, 체결 가정, 차단을 분리하고 누적 수익률/MDD/승률/손익비/turnover/exposure, equity curve, in/out-of-sample, 거래 목록을 표시한다. 큰 거래 목록은 UI에서 100건 이하로 제한해 jank를 막는다.
- replay cadence와 live 10초 tick cadence, warmup, 데이터 source/range, 재현 hash를 함께 표시해 일봉/1분봉 근사와 실제 tick을 혼동하지 않게 한다.
- 전용 LTH의 `replay.assessment`로 성과 표시 가능 여부를 결정한다. `dailyDiagnostic/notEvaluable` 일봉과 유효 분봉 0개는 자료 진단만 표시하고 수익률·승률·자산 곡선·거래 성과·A/B 저장을 숨긴다. assessment 없는 기존 LTH 응답도 평가 불가로 처리하며 빈 backtest를 0% 성과로 표시하지 않는다. generic 성과 패널은 기존 계약을 유지한다.
- `intradaySample/sampleOnly`는 “분봉 표본 모의 결과”로 제목/경고를 표시한다. 별도 일봉 context 수와 실제 장중 관측 수, 최대 200봉/3개월 대표성 부족, 원본 시간대·DST·세션 미검증과 고정 KST 근사를 명시한다. `warmupCount=0`을 일봉 context 없음으로 해석하지 않는다. A/B snapshot은 assessment를 replay에 보존하며 기존 미검증 LTH 저장 결과의 성과도 숨긴다.
- 단기 generic preview는 모든 전략에 동일한 요청 평가 봉 수를 우선 적용한다. 공통 완료봉 페이지를 최대8회·보관500봉 안에서 조회해 평가50/100/200봉과 이전252봉을 별도 `historyCandles`로 보낸다. 준비 부족을 감추기 위해 요청 평가 봉을 history로 옮기거나200·252 기준을 줄이지 않고 요청·실제 평가·사전자료 수와 수집 중단 사유를 표시한다. 연속 일봉의 별도 상한과 날짜 범위는 아래 v12 항목을 따른다.
- `preparation`에서 자료 부족·지표 준비 중·조건 불충족·신호 무체결·체결 가정을 구분한다. 제공 자료와 실제 지표 버퍼 수, 시작/종료 준비, 처리 후 첫 준비 시각, 조건 평가 봉 수를 함께 읽는다. seed-only 또는 미지원 구간은 평가 불가로 표시하며 null을 false/0으로 단정하지 않는다.
- A/B 실험은 broker/account/strategy/symbol scope별 localStorage에 두 slot만 저장한다. 전략 버전, params, data source/interval/range, 비용 가정, 생성 시각을 포함하고 credential/secret은 저장하지 않는다. source/기간이 다르면 직접 비교 경고를 표시한다.
- equity SVG는 `role="img"`와 시작/종료 자산을 설명하는 `aria-label`을 제공한다. 좁은 화면에서는 실행 가정 입력과 preview control이 1열로 쌓여 카드 밖으로 넘치지 않아야 한다.

> 마지막 업데이트: 2026-10-03T23:21:04+09:00

## DB 관리 Settings UI

- PostgreSQL/MariaDB만 provider 선택지로 제공하고, 연결 설정 저장 전에는 연결 테스트·테이블·이관 action을 비활성화한다. password는 조회값으로 채우지 않고 비워 두면 기존 값을 유지한다.
- 연결/서버 버전/latency/schema/table 행 수, 현재 JSON/DB backend, JSON 파일 수·크기, 마지막 이관 checksum/경로를 같은 섹션에서 확인할 수 있게 한다.
- table clear/drop은 자동매매 중이거나 DB backend가 활성일 때 금지한다. 별도 Dialog에서 정확한 확인 문구 입력과 복구 불가 확인 checkbox를 모두 요구한다.
- backend 전환은 자동매매 정지와 schema/import 검증을 통과한 경우만 허용하며, stale 연결 설정으로 작업하지 않도록 form dirty 상태도 action을 잠근다.
- DB password와 파괴적 관리는 Tauri desktop Settings에서만 렌더링한다. 웹 모드에는 보안 안내만 보여주고 관리 form/control을 노출하지 않는다.

> 마지막 업데이트: 2026-10-05T23:54:16+09:00


### Toss 수수료 비율 표시 (2026-10-18)

`TossOrderPreflightView.commissionRate`는 공식 소수 비율 문자열이다. `0.00015`를 `0.015%`로 표시하며 원본에 `%`만 붙이지 않는다. 주문 계산은 Rust preflight에서 비율을 그대로 적용한다. 공식 계약 근거와100배 오류 회귀는 `toss-api/SKILL.md`를 따른다.

> 마지막 업데이트: 2026-10-18T14:30:00+09:00

## 연구 체결 가정 (replay v10)

- 공통 실행 가정에 같은 종가/다음 관측봉 시가, 봉 내부 미검증/손절 우선 가정과 연구 손절·익절(0비활성)을 제공한다. 입력 변경은 기존 preview와 진행 응답을 무효화한다.
- signalAvailableAt와 fillAt의 봉 label·open/close/intrabar 단계를 분리 표시한다. 마지막 unfilled는 차단과 구분하고 순서 불명·갭·정확한 거래소 시각/체결 미검증 경고를 남긴다. raw 전략 신호 수와 별도 보호 주문 수는 일치하지 않을 수 있다.
- 엔진·체결 모델·보호 가정을 현재 결과와 A/B 양쪽에 표시한다. 기존 저장 응답의 선택 필드가 없으면 확인 불가로 표시하고 새 결과로 둔갑시키지 않는다. LTH 일봉은 계속 자료 진단만 표시한다.

> 마지막 업데이트: 2026-10-18T15:21:57+09:00


## 연구 비용·현금흐름과 회계 결과 (replay v11)

- 비용·시나리오 입력/응답의 정본은 저장소 `docs/ipc-commands.md`의 v11 회계·비교 계약이다. 비용 출처는 추정/사용자 수동 확인이며 후자에 최대120자 메모를 요구한다. 브로커 API 자동 검증이라고 표시하지 않는다. 방향별 bps의 빈칸은 공통 요율 사용, 최소 수수료는 국내 원/해외 USD 단위를 라벨에 표시한다.
- FX/배당 JSON은 각각64행·24,000자로 제한하고 자료·범위 오류를 빈 배열로 바꾸지 않는다. 잘못된 draft 편집도 현재 preview와 진행 generation을 즉시 무효화하고 계산을 차단한다. 시나리오는 정규화 뒤 실제 평가 봉에 정확히 일치해야 하며 국내 FX와 LTH 일봉의 비어 있지 않은 시나리오를 지원되는 성과로 표시하지 않는다.
- 배당 가격 자료는 미확인/미조정 선언/조정 선언을 구분한다. 배당 조정 가격+현금 배당은 이중 계산으로 거부하고 미확인은 경고한다. FX는 지정 봉부터 유지하고 배당은 봉 시작 직전 보유 수량에 즉시 세후 원화 지급한다는 연구 가정과 유휴 KRW 현금 정책을 안내한다.
- `strategyAccountingPanel.tsx`는 비용/시나리오 편집과 손익/비교 표시를 맡는다. 긴 JSON 예시·원문은 `overflowWrap:anywhere`/`whiteSpace:pre-wrap`로 모바일 카드 밖에 넘치지 않게 한다. 공통 수수료와 매수/매도 수수료의 accessible name이 부분 일치하므로 Playwright 공통 수수료 선택에는 `exact:true`를 사용한다.
- 실현·잔여 평가·순배당을 분리하고 전체 자산 증감과 부호가 다르면 경고한다. 가격/환율 효과와 수수료·거래세·슬리피지 영향을 따로 읽으며 슬리피지를 이중 차감하지 않는다. 완료 청산 표본 수와 부분 매도도 각1건이라는 한계를 표시한다.
- 종료 시 가상 청산을 실제 체결·승률과 분리한다. 봉말 평균 보유 평가액, 초기자본/당시 순자산 대비 관측 평균·최대 비중, 보유 관측 봉 비율을 서로 구분하고 시간 가중·장중 최대 노출이라고 표시하지 않는다.
- 단순 보유는 같은 요청 수량이어도 실제 진입 수량·노출이 다를 수 있음을 표시한다. 같은 체결 경로의 비용 제거 값은 발생 당시 원화 비용 환급 산술이며 원가/FX 재분해나 비용 변경 전략 재실행이 아니다. 겹친 A/B는 독립 검증이 아니며 부분기간을 전체 분기/연속 운용 성과로 합산하지 않는다.
- A/B에 비용 출처·요율·최소 수수료·FX/배당 시나리오·배당 가격 선언과 서버 집계를 보존한다. 보관 거래를100건으로 잘라도 집계는 다시 계산하지 않는다. legacy `accounting`/`comparison` 누락은 unknown으로 표시하고 LTH 일봉·기존 평가 불가 슬롯에는 새 회계 성과도 숨긴다.
- UI 검증은 `docs/HARNESS_MAP.md`의 P2-15 focused 조합을 따른다. 2026-10-18 고정 소스47/47(mock API) 통과는 입력·상태·표시 계약의 근거이며 실제 비용/주문/시장 성과 검증이 아니다.

> 마지막 업데이트: 2026-10-18T15:44:21+09:00

- 연구 원화 정수 표시는 `shared/lib/formatters.ts::fmtRoundedKrw`를 재사용한다. Math.round의 음수 half-tie를 보존하며 통화/누락/decimal-string 의미가 다른 fmtBrokerMoney로 대체하지 않는다.

> 마지막 업데이트: 2026-10-18T15:51:29+09:00


## 연속 일봉 연구 UI (replay v12)

- 정본은 [IPC v12 연속 재생 계약](../../../docs/ipc-commands.md)이다. 기존 단기50/100/200봉 모드·flat 합계500봉 경로와 전용 LTH UI는 유지한다. 연속 모드는 일반 전략의 일봉 날짜 범위에만 제공하고 어제까지 최근3년을 기본 요청한다. API의 기존 주/월/분봉 허용을 UI 선택 지원으로 확대하지 않는다.
- `pages/strategy/ui/continuousReplayPanel.tsx`가 모드·날짜·수집 안내·공급 경계 표를, `pages/strategy/model/candlePageCollector.ts`가 bounded 수집을 맡는다. strict 시작 이전252봉을 한 번 확보하고48회 요청·보관4608/평가4096을 제한한다. 요청 평가 봉을 history로 옮기지 않고 준비 완료 후 오래된 순서로500봉씩 나누어 연속 API를 한 번 호출한다.
- 요청/실제 관측 범위·asOf·history252 확보량·페이지·제외/중복 수·중단 사유·최대 관측 간격을 보인다. 요청 경계 차이 또는 관측 간격7달력일 초과는 계산 보류다. 이 gate는 앱의 보수적 자료 경계이며 검증된 거래소 달력이 아니고 긴 휴장도 보류할 수 있음을 명시한다. 준비 완료여도 `coverageVerified=false`를 유지하며 누락 봉을 합성하거나 일부 자료를3년 성과로 표시하지 않는다.
- 날짜가 비었거나 역전·무효이면 계산을 차단한다. 모드·날짜·가정·scope 변경과 취소는 기존 결과/두 mutation 및 진행 generation을 무효화한다. 늦게 온 연속 응답이 이후 단기 결과를 덮지 않게 한다. 응답의 continuity mode·평가/history 수·묶음 수와 원장 길이를 보낸 입력과 확인한다.
- 결과와 A/B에 실제 호출 모드(`boundedWindow`/`continuous`)·최상위 `continuity`를 서버 집계와 함께 보존한다. 이전 보관값에 계산 모드가 없으면 정보 없음으로 표시하며 엔진 이름으로 추정하지 않는다. 모드가 다른 A/B는 가정 차이를 표시한다. hash는 모드를 구분하고 같은 연속 경제 입력의 공급 분할은 구분하지 않는다.
- 공급 경계의 시작/종료 보유·pending은 같은 장부의 상태 진단이다. 묶음·분기마다 자본·원가·리스크가 초기화되거나 종료 청산된다고 표시하지 않는다. 다음 시가 대기는 다음 묶음으로 보존하고 전체 마지막에만 미체결로 확정한다. 연속70/30은 **앞/뒤 구간·동일 장부**와 **연속 구간 경계**로 표시하며 독립 훈련/검증·분기 수익 합산으로 설명하지 않는다. 기존 v11 비용 환급·단순 보유 실제 노출 차이 설명도 유지한다.
- 모바일에서 날짜 입력은 한 열로 쌓고 긴 범위/경계 표는 카드 안에서 스크롤하게 한다. focused 검증은 [Harness P2-18](../../../docs/HARNESS_MAP.md)의 연속10개+인접47개 조합을 따르며 최초56개 통과+수정한1개 재실행 통과를 단일 전체 통과 실행으로 부풀리지 않는다. 실제 provider coverage·주문/체결 검증과 구분한다.

> 마지막 업데이트: 2026-10-18T16:10:33+09:00

## Consecutive v13 선택 진입 필터

- 관측 횟수를 일수로 표시하지 않는다. 누적 변화0..100, ATR배수0..20, MA정수0..512의 raw 입력을 유한수/범위/정수로 검증하고 빈칸·잘못된 stored string/null은 draft를 보존하며 저장·preview를 막는다. 잘못된 편집도 이전 응답을 무효화한다. 실제 키보드 소수 입력과 모바일 줄바꿈/overflow를 검증한다.
- ATR/MA는 일봉만 지원한다는 안내와 live 약10초 quote+완료 일봉 맥락 차이를 명시한다. 기존 매도/guard가 유지됨을 안내하며 수량 조절 ATR 준비와 선택 진입 준비를 별도 표시한다.
- 모든 필터가 유효 숫자0/누락인 요청·A/B 기록은 비활성으로 중립 표시한다. 활성 필터의 entryFilterPreparation 없는 legacy만 준비 정보 없음이며 통과를 실제 체결로 표시하지 않는다. A/B에 서버 진단과 params를 보존하고 전 필터0에서는 기존 기본 경로를 유지한다. 정본은 docs/ipc-commands.md의 v13 계약이다.

> 마지막 업데이트: 2026-10-18T16:31:45+09:00


## Momentum v14 진입 교차 UI

- entry_mode level/crossing 문자열을 일반 numeric param 변환으로 훼손하지 않는다. 새두필드의 stored enum/null/string 원형을 보존해 엄격히 검증하고 저장/두preview를 차단한다. 실제 소수 키 입력·실패 재시도·활성 잠금·이전 응답 폐기와 모바일을 확인한다.
- 간격은 %p이고0..100/crossing에서threshold이하이다. level에서는 유효값을보관하되미사용을명시한다. 기존>=T/<=−T와entrycross이전<T→현재>=T,재무장<=T−band를안내한다. 보유청산은모드와독립이며 신호·pending을확정소유/체결로표시하지않는다.
- A/B는서버성과와함께params의모드/%p를보존·표시하고차이를알린다. live약10초와선택봉관측차이를숨기지않는다. 새DTO/계좌query/polling을추가하지않는다.

> 마지막 업데이트: 2026-10-20T19:42:13+09:00

숫자 초안의 공통 문자열 해석은 `src/shared/lib/numberDraft.ts::parseDecimalDraft`를 shared/lib 공개 진입점으로 사용한다. 빈칸·단독 부호/점·잘못된 문자열·비유한수는 raw를 보존하고 유한한 십진 입력만 숫자로 바꾼다. 전략별 범위·정수·mode 관계와 input raw-state 보존은 각 controls에 남긴다. Graphify 후보/Serena 실제 caller와 도메인 차이를 확인한 뒤 승격한 범위이며, 소유/가격 anchor 같은 전략 상태와 혼합하지 않는다.

> 마지막 업데이트: 2026-10-20T19:49:11+09:00
