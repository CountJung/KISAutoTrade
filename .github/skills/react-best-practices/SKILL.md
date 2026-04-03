---
name: react-best-practices
description: "React/Tauri 성능 최적화 규칙. waterfall 제거, 번들 최적화(MUI/icons-material 직접 import), 리렌더 최적화, useMemo/useCallback, Suspense, lazy loading, TanStack Query 중복 제거, localStorage 버전 관리. Keywords: React, performance, waterfall, bundle, useMemo, useCallback, Suspense, lazy, TanStack Query, MUI barrel import"
---
# React Best Practices

> 이 스킬은 Vercel Engineering 가이드를 기반으로 이 프로젝트에 맞게 적용된 React 성능 최적화 규칙입니다.  
> **작업 중 새로운 패턴이 추가되면 이 파일을 업데이트합니다.**

---

## 1. 배럴 Import 금지 — 번들 최적화 (CRITICAL)

MUI와 아이콘 라이브러리는 배럴 import 시 수천 개의 모듈을 로드합니다.  
**직접 경로 import**만 사용합니다.

```tsx
// ❌ 금지 — 전체 라이브러리 로드 (2,000+ 모듈)
import { Button, TextField } from '@mui/material';
import { PlayArrow, Stop } from '@mui/icons-material';

// ✅ 올바른 방법 — 직접 경로 (필요한 것만 로드)
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';
```

**예외:** `@mui/material`에서 여러 컴포넌트를 한 줄로 import하는 것은 `vite`가 트리셰이킹하므로 허용됩니다. 아이콘(`@mui/icons-material`)은 반드시 직접 경로를 사용합니다.

---

## 2. Waterfall 제거 — 병렬 데이터 페칭 (CRITICAL)

순차적 `await`는 최대 성능 저하를 유발합니다. 독립적인 요청은 항상 병렬로 실행합니다.

```typescript
// ❌ 금지 — 순차 실행 (3 round trips)
const balance = await getBalance();
const stats = await getTodayStats();
const trades = await getTodayTrades();

// ✅ 올바른 방법 — 병렬 실행 (1 round trip)
const [balance, stats, trades] = await Promise.all([
  getBalance(),
  getTodayStats(),
  getTodayTrades(),
]);
```

TanStack Query에서는 `useQueries`로 병렬 쿼리를 관리합니다:

```typescript
import { useQueries } from '@tanstack/react-query';
import { KEYS } from '@/api/hooks';

const results = useQueries({
  queries: [
    { queryKey: KEYS.balance, queryFn: getBalance, refetchInterval: 60_000 },
    { queryKey: KEYS.todayStats, queryFn: getTodayStats, refetchInterval: 60_000 },
  ],
});
```

---

## 3. TanStack Query 중복 제거 패턴 (MEDIUM-HIGH)

같은 `queryKey`를 사용하면 여러 컴포넌트에서 단 한 번만 fetch됩니다.

```typescript
// ✅ src/api/hooks.ts — queryKey를 KEYS 상수로 관리
export const KEYS = {
  balance: ['balance'] as const,
  price: (symbol: string) => ['price', symbol] as const,
  todayStats: ['stats', 'today'] as const,
} as const;

// 같은 KEYS를 사용하면 어디서 호출해도 단 하나의 요청만 발생
export const useBalance = () =>
  useQuery({ queryKey: KEYS.balance, queryFn: getBalance, refetchInterval: 60_000 });
```

새 훅은 반드시 `src/api/hooks.ts`에 추가하고 `KEYS`에 키를 등록합니다.

---

## 4. 파생 상태 계산 — useState/useEffect 남용 금지 (MEDIUM)

props나 state에서 계산 가능한 값은 별도 state로 저장하지 않습니다.

```tsx
// ❌ 금지 — 불필요한 state와 effect
const [totalPnl, setTotalPnl] = useState(0);
useEffect(() => {
  setTotalPnl(trades.reduce((sum, t) => sum + (t.pnl ?? 0), 0));
}, [trades]);

// ✅ 올바른 방법 — render 중 계산
const totalPnl = trades.reduce((sum, t) => sum + (t.pnl ?? 0), 0);
```

---

## 5. useMemo/useCallback 사용 기준 (MEDIUM)

비싼 계산과 콜백의 안정적인 참조가 필요할 때만 사용합니다.

```tsx
// ✅ useMemo — 리스트 필터링/정렬 등 연산 비용이 있을 때
const filteredTrades = useMemo(
  () => trades.filter(t => t.symbol.includes(search)),
  [trades, search]
);

// ✅ useCallback — 자식 컴포넌트에 콜백을 prop으로 전달할 때
const handlePlaceOrder = useCallback(
  (input: PlaceOrderInput) => placeOrderMutation.mutate(input),
  [placeOrderMutation]
);

// ❌ 불필요한 메모이제이션
const count = useMemo(() => trades.length, [trades]); // 불필요
```

---

## 6. React.lazy + Suspense — 코드 스플리팅 (MEDIUM)

무거운 페이지는 lazy load합니다. 초기 화면에 불필요한 페이지에 적용합니다.

```tsx
import { lazy, Suspense } from 'react';
import CircularProgress from '@mui/material/CircularProgress';

const History = lazy(() => import('../pages/History'));

function App() {
  return (
    <Suspense fallback={<CircularProgress />}>
      <History />
    </Suspense>
  );
}
```

---

## 7. localStorage 버전 관리 (MEDIUM)

Zustand persist 스토어 스키마 변경 시 버전 prefix를 사용합니다.

```typescript
// src/store/settingsStore.ts
const STORAGE_KEY = 'v1:settings-storage';

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({ /* ... */ }),
    { name: STORAGE_KEY }
  )
);
```

---

## 8. Tauri IPC 에러 처리 패턴 (프로젝트 특이사항)

`invoke()` 래퍼에서 `CmdError` 타입을 처리합니다.

```typescript
// src/api/commands.ts 패턴
export const getBalance = (): Promise<BalanceResult> =>
  invoke('get_balance'); // 에러 시 TanStack Query가 error state로 전환

// React에서 에러 표시
const { data, error, isLoading } = useBalance();
if (error) return <Alert severity="error">{(error as CmdError).message}</Alert>;
```

---

> 마지막 업데이트: 2026-04-03

---

## 9. 드래그 리사이저 컴포넌트 (LayoutResizer)

**파일**: `src/components/LayoutResizer.tsx` (ArielTauriGUI에서 1:1 포팅 완료)

Holy Grail 레이아웃의 패널 크기를 마우스 드래그로 조절합니다.

```tsx
import { LayoutResizer } from '../components/LayoutResizer'

// 수평 분할 (좌/우 패널 사이)
<Box sx={{ display: 'flex', height: '100%' }}>
  <LeftPanel style={{ width: leftWidth }} />
  <LayoutResizer
    direction="horizontal"
    onResize={(delta) => setLeftWidth(w => Math.max(200, w + delta))}
    onResizeEnd={() => localStorage.setItem('leftWidth', String(leftWidth))}
  />
  <RightPanel sx={{ flex: 1 }} />
</Box>

// 수직 분할 (상/하 패널 사이)
<LayoutResizer direction="vertical" onResize={(delta) => setTopHeight(h => h + delta)} />
```

**Props**:
- `direction`: `"horizontal"` (col-resize) | `"vertical"` (row-resize)
- `onResize(delta: number)`: 드래그 중 픽셀 단위 delta
- `onResizeEnd?()`: 드래그 완료 시 (로컬스토리지 저장 시점)

---

## 10. 리사이즈 + 드래그 가능 Dialog (ResizableDialog)

**파일**: `src/components/ResizableDialog.tsx` (ArielTauriGUI에서 포팅 완료)

MUI Dialog를 확장한 컴포넌트. 8방향 리사이즈 핸들 + AppBar 드래그 이동 지원.

```tsx
import { ResizableDialog } from '../components/ResizableDialog'

<ResizableDialog
  open={open}
  onClose={() => setOpen(false)}
  dialogTitle="종목 상세 정보"
  defaultWidth={800}
  defaultHeight={600}
  minWidth={400}
  minHeight={300}
  storageKey="stock-detail-dialog"  // SPA 세션 동안 크기/위치 유지
>
  <DialogContent>...</DialogContent>
</ResizableDialog>
```

**Props**:
- `dialogTitle`: AppBar에 표시할 타이틀. 생략하면 AppBar 없이 children만 렌더
- `defaultWidth/Height`: 초기 크기 (px). storageKey 캐시가 있으면 무시
- `minWidth/Height`: 리사이즈 최솟값 (px)
- `storageKey`: SPA 세션 동안 크기/위치 유지할 고유 키
- `titleBarSx`: AppBar에 추가 SxProps

**동작**:
- AppBar/DialogTitle 클릭+드래그 → 다이얼로그 이동
- 변/모서리 핸들 드래그 → 리사이즈 (8방향)
- `storageKey` 설정 시 SPA 세션 동안 크기·위치 기억 (세션 내 캐시)

---

## 11. 앱 UI 상태 localStorage 영속성 규칙

**핵심 원칙**: 사용자가 조작한 크기/위치/패널 분할 상태는 `localStorage`에 저장하여 앱 재시작 후에도 유지합니다.

### 저장 대상

| 상태 종류 | 저장 키 패턴 | 저장 시점 |
|----------|-------------|----------|
| 패널 너비 | `act:panel:{panelName}:width` | `onResizeEnd` |
| 패널 높이 | `act:panel:{panelName}:height` | `onResizeEnd` |
| Dialog 크기/위치 | `ResizableDialog.storageKey` (SPA 세션) | 리사이즈/이동 중 자동 |
| Sidebar 접힘 여부 | `act:sidebar:collapsed` | 토글 시 |
| 테이블 열 너비 | `act:table:{tableId}:columns` | 드래그 완료 시 |

### 패턴: LayoutResizer 크기 영속성

```tsx
const SIDEBAR_KEY = 'act:panel:sidebar:width'
const DEFAULT_WIDTH = 240

function AppLayout() {
  const [sidebarWidth, setSidebarWidth] = useState(() => {
    const saved = localStorage.getItem(SIDEBAR_KEY)
    return saved ? Number(saved) : DEFAULT_WIDTH
  })

  return (
    <Box sx={{ display: 'flex', height: '100vh' }}>
      <Sidebar style={{ width: sidebarWidth }} />
      <LayoutResizer
        direction="horizontal"
        onResize={(delta) => setSidebarWidth(w => Math.max(160, Math.min(480, w + delta)))}
        onResizeEnd={() => localStorage.setItem(SIDEBAR_KEY, String(sidebarWidth))}
      />
      <MainContent sx={{ flex: 1 }} />
    </Box>
  )
}
```

> **주의**: `onResizeEnd` 클로저에서 최신 state를 읽으려면 `useRef`로 추적하거나  
> `useCallback`의 deps에 width를 포함합니다.

### 패턴: localStorage 초기값 + 범위 제한

```typescript
function readStoredSize(key: string, defaultVal: number, min: number, max: number): number {
  const raw = localStorage.getItem(key)
  if (!raw) return defaultVal
  const n = Number(raw)
  return Number.isFinite(n) ? Math.max(min, Math.min(max, n)) : defaultVal
}
```

### Zustand persist 연동

앱 수준 UI 설정(테마, 사이드바 상태 등)은 Zustand `persist` 미들웨어를 사용합니다.  
패널 크기처럼 빈번히 변경되는 값은 `localStorage` 직접 접근이 성능상 유리합니다.

```typescript
// 자주 변경 → localStorage 직접
localStorage.setItem('act:panel:sidebar:width', String(width))

// 드물게 변경 → Zustand persist
useSettingsStore.getState().setTheme('dark')
```

### ResizableDialog의 영속성

`ResizableDialog`의 `storageKey`는 **SPA 세션** 동안만 유지됩니다 (인메모리 Map).  
앱 재시작 후에도 크기를 유지하려면:

```tsx
// localStorage 기반 DialogGeometry 영속화 예시
const DIALOG_KEY = 'act:dialog:stock-detail'

const savedGeometry = useMemo(() => {
  const raw = localStorage.getItem(DIALOG_KEY)
  return raw ? JSON.parse(raw) : undefined
}, [])

// onClose 시 현재 크기 저장 (ResizableDialog의 내부 state를 ref로 노출 필요 시 확장)
```

> ResizableDialog는 현재 SPA 세션 캐시만 지원합니다.  
> localStorage 영속이 필요하면 `storageKey`를 기반으로 별도 save/restore 로직을 추가합니다.
