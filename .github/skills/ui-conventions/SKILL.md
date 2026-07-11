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
- 일반 미리보기는 KIS 일/주/월봉, Toss 1분/일봉을 제공하고 분석 구간은 최근 50/100/200봉으로 분리한다. `봉 단위`와 `분석 구간`을 한 프리셋 이름으로 합쳐 사용자가 일봉과 1년 범위를 혼동하게 하지 않는다.
- 티커·봉 단위·분석 구간·파라미터·수량·broker가 바뀌면 이전 미리보기 결과를 무효화한다. 진행 중 요청의 결과도 generation/key가 현재 입력과 일치할 때만 표시해 stale chart를 막는다.
- lightweight-charts 미리보기는 `horzTouchDrag: true`, `vertTouchDrag: false`, `pinch: true`로 설정하고 컨테이너 `touchAction: pan-y`를 유지한다. 모바일에서 세로 한 손가락 이동은 페이지 스크롤, 가로 이동은 차트 패닝, 두 손가락은 확대/축소로 분리하며 확대·축소·전체 맞춤 버튼도 제공한다.
- Playwright에서 모든 전략 카드의 x/width가 동일한지와 좁은 viewport에서 입력/Select/버튼이 카드 밖으로 넘치지 않는지 검증한다.

## DB 관리 Settings UI

- PostgreSQL/MariaDB만 provider 선택지로 제공하고, 연결 설정 저장 전에는 연결 테스트·테이블·이관 action을 비활성화한다. password는 조회값으로 채우지 않고 비워 두면 기존 값을 유지한다.
- 연결/서버 버전/latency/schema/table 행 수, 현재 JSON/DB backend, JSON 파일 수·크기, 마지막 이관 checksum/경로를 같은 섹션에서 확인할 수 있게 한다.
- table clear/drop은 자동매매 중이거나 DB backend가 활성일 때 금지한다. 별도 Dialog에서 정확한 확인 문구 입력과 복구 불가 확인 checkbox를 모두 요구한다.
- backend 전환은 자동매매 정지와 schema/import 검증을 통과한 경우만 허용하며, stale 연결 설정으로 작업하지 않도록 form dirty 상태도 action을 잠근다.
- DB password와 파괴적 관리는 Tauri desktop Settings에서만 렌더링한다. 웹 모드에는 보안 안내만 보여주고 관리 form/control을 노출하지 않는다.

> 마지막 업데이트: 2026-07-11T23:27:54+09:00
