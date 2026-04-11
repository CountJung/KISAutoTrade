# AutoConditionTrade — 코딩 가이드

> 이 문서는 프로젝트 유지보수·신규 기여자를 위한 실전 참고서입니다.  
> 추상적 원칙이 아닌 **이 프로젝트의 실제 파일·코드 패턴**을 기준으로 작성합니다.

---

## 목차

1. [프로젝트 기술 스택 및 아키텍처](#1-프로젝트-기술-스택-및-아키텍처)
2. [설정 변수(Config) 추가하는 법](#2-설정-변수config-추가하는-법)
3. [전역 AppState 값 추가 및 관리](#3-전역-appstate-값-추가-및-관리)
4. [IPC 커맨드 추가 (Rust → React 연결)](#4-ipc-커맨드-추가-rust--react-연결)
5. [UI에서 버튼 → 백그라운드 작업 → 결과 표시 패턴](#5-ui에서-버튼--백그라운드-작업--결과-표시-패턴)
6. [데이터 저장 (JSON 파일)](#6-데이터-저장-json-파일)
7. [백그라운드 데몬 작성 원칙](#7-백그라운드-데몬-작성-원칙)
8. [Rust 제어 흐름 원칙 (goto 유사 패턴 금지)](#8-rust-제어-흐름-원칙-goto-유사-패턴-금지)
9. [에러 처리 패턴](#9-에러-처리-패턴)
10. [TypeScript 타입 미러링](#10-typescript-타입-미러링)
11. [React Query 캐시 무효화 패턴](#11-react-query-캐시-무효화-패턴)
12. [기능 구현 완료 체크리스트](#12-기능-구현-완료-체크리스트)

---

## 1. 프로젝트 기술 스택 및 아키텍처

```
React 18 (TypeScript)  →  Tauri v2 IPC  →  Rust Backend
                                         ↓
                              KIS Open API (REST + WebSocket)
                              Discord Bot API
                              JSON 파일 Storage
```

| 역할 | 경로 |
|------|------|
| Tauri IPC 커맨드 | `src-tauri/src/commands.rs` |
| Tauri 앱 진입점 / 데몬 초기화 | `src-tauri/src/lib.rs` |
| KIS REST 클라이언트 | `src-tauri/src/api/rest.rs` |
| 전략 엔진 | `src-tauri/src/trading/strategy.rs` |
| React 훅 (TanStack Query) | `src/api/hooks.ts` |
| Tauri invoke 래퍼 | `src/api/commands.ts` |
| TypeScript 타입 미러 | `src/api/types.ts` |
| Zustand 스토어 | `src/store/*.ts` |

---

## 2. 설정 변수(Config) 추가하는 법

> **예시**: 로그 보관 기간(`retention_days`) 설정 추가 과정

### Step 1 — Rust 구조체 정의 (`src-tauri/src/logging/mod.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub retention_days: u32,   // 보관 기간 (일)
    pub max_size_mb: u64,      // 최대 저장 용량 (MB)
    pub api_debug: bool,       // API 디버그 로깅 여부
}

impl Default for LogConfig {
    fn default() -> Self {
        Self { retention_days: 7, max_size_mb: 500, api_debug: false }
    }
}

impl LogConfig {
    /// 파일에서 로드, 없으면 기본값
    pub fn load_or_default(log_dir: &Path) -> Self {
        let path = log_dir.join("log_config.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// 파일에 동기 저장
    pub fn save_sync(&self, log_dir: &Path) -> std::result::Result<(), anyhow::Error> {
        std::fs::create_dir_all(log_dir)?;
        let path = log_dir.join("log_config.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

### Step 2 — AppState에 필드 추가 (`src-tauri/src/commands.rs`)

```rust
pub struct AppState {
    // ... 기존 필드 ...
    pub log_config: Arc<RwLock<LogConfig>>,   // ← 추가
}

impl AppState {
    pub fn new(...) -> Self {
        let log_config = Arc::new(RwLock::new(
            LogConfig::load_or_default(&log_dir)
        ));
        // ...
        Self {
            // ...
            log_config,
        }
    }
}
```

### Step 3 — IPC 커맨드 추가 (`src-tauri/src/commands.rs`)

```rust
/// 설정 조회
#[tauri::command]
pub async fn get_log_config(state: State<'_, AppState>) -> CmdResult<LogConfig> {
    Ok(state.log_config.read().await.clone())
}

/// 설정 변경 + 즉시 파일 저장 + 즉시 적용
#[tauri::command]
pub async fn set_log_config(
    input: SetLogConfigInput,
    state: State<'_, AppState>,
) -> CmdResult<LogConfig> {
    let new_cfg = LogConfig { retention_days: input.retention_days, ... };
    *state.log_config.write().await = new_cfg.clone();
    new_cfg.save_sync(&state.log_dir).map_err(CmdError::from)?;
    crate::logging::cleanup(&state.log_dir, &new_cfg); // 즉시 적용
    Ok(new_cfg)
}
```

### Step 4 — `lib.rs` `generate_handler!` 에 등록

```rust
// src-tauri/src/lib.rs
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // ... 기존 커맨드 ...
        commands::get_log_config,
        commands::set_log_config,   // ← 추가
    ])
```

### Step 5 — TypeScript 타입 및 훅 추가

```typescript
// src/api/types.ts
export interface LogConfig {
  retentionDays: number;
  maxSizeMb: number;
  apiDebug: boolean;
}

// src/api/commands.ts
export const getLogConfig = () => invoke<LogConfig>('get_log_config');
export const setLogConfig = (input: Partial<LogConfig>) =>
  invoke<LogConfig>('set_log_config', { input });

// src/api/hooks.ts
export const useLogConfig = () =>
  useQuery({ queryKey: KEYS.logConfig, queryFn: getLogConfig });

export const useSetLogConfig = () =>
  useMutation({
    mutationFn: setLogConfig,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: KEYS.logConfig }),
  });
```

### Step 6 — UI에서 표시 (`src/pages/Settings.tsx`)

```tsx
const { data: logConfig } = useLogConfig();
const { mutate: setLogConfig } = useSetLogConfig();

<TextField
  label="로그 보관 기간 (일)"
  type="number"
  value={logConfig?.retentionDays ?? 7}
  onChange={(e) => setLogConfig({ retentionDays: Number(e.target.value) })}
/>
```

---

## 3. 전역 AppState 값 추가 및 관리

### Arc 래핑 타입 선택 기준

| 상황 | 타입 | 이유 |
|------|------|------|
| 읽기 빈번, 쓰기 드물 | `Arc<RwLock<T>>` | 다수 reader 동시 허용 |
| 쓰기 빈번 또는 간단한 배타적 접근 | `Arc<Mutex<T>>` | 단순 exclusive lock |
| 원자적 bool 플래그 | `Arc<AtomicBool>` | lock-free read |

### 읽기 / 쓰기 패턴

```rust
// 읽기
let value = state.log_config.read().await.clone();

// 쓰기
*state.log_config.write().await = new_value;

// 잠금 최소화: clone 후 릴리스
let snapshot = {
    let guard = state.log_config.read().await;
    guard.clone()
}; // guard 여기서 drop
// snapshot으로 긴 처리 수행
```

### ❌ 금지 패턴 — 데드락 위험

```rust
// 두 개의 lock을 동시에 유지하면 데드락 위험
let _guard1 = state.strategy_manager.lock().await;
let _guard2 = state.order_manager.lock().await;  // 다른 곳에서 역순으로 lock하면 데드락
```

### ✅ 올바른 패턴 — 최소 범위 lock

```rust
// 필요한 데이터를 빠르게 읽고 lock 해제
let symbols = state.strategy_manager.lock().await.active_symbols();
// lock 해제 후 독립 처리
let rest = state.rest_client.read().await.clone();
```

---

## 4. IPC 커맨드 추가 (Rust → React 연결)

### 완전한 추가 절차 (누락 없이 4개 파일 수정)

#### ① Rust 커맨드 정의 (`src-tauri/src/commands.rs`)

```rust
/// 반환 타입은 항상 CmdResult<T>
#[tauri::command]
pub async fn my_new_command(
    input: MyInput,
    state: State<'_, AppState>,
) -> CmdResult<MyOutput> {
    // ... 처리 ...
    Ok(result)
}
```

#### ② `lib.rs` `generate_handler!` 등록 (반드시!)

```rust
// src-tauri/src/lib.rs
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        commands::my_new_command,  // ← 여기 빠지면 런타임 에러
    ])
```

#### ③ TypeScript invoke 래퍼 (`src/api/commands.ts`)

```typescript
export const myNewCommand = (input: MyInput) =>
  invoke<MyOutput>('my_new_command', { input });
```

#### ④ React Query 훅 (`src/api/hooks.ts`)

```typescript
// 조회형 (query)
export const useMyData = () =>
  useQuery({ queryKey: KEYS.myData, queryFn: myNewCommand });

// 변경형 (mutation)
export const useMyMutation = () =>
  useMutation({
    mutationFn: myNewCommand,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: KEYS.myData });
    },
  });
```

### 커맨드 반환 타입 규칙

```rust
// src-tauri/src/commands.rs
pub struct CmdError {
    pub code: String,     // 에러 식별자 (예: "CONFIG_NOT_READY")
    pub message: String,  // 사람이 읽을 수 있는 메시지
}

type CmdResult<T> = Result<T, CmdError>;

// 에러 생성
return Err(CmdError {
    code: "NOT_FOUND".into(),
    message: format!("찾을 수 없습니다: {}", id),
});

// anyhow::Error → CmdError 변환
.map_err(CmdError::from)?
```

---

## 5. UI에서 버튼 → 백그라운드 작업 → 결과 표시 패턴

### 기본 패턴: useMutation + 로딩 상태

```tsx
// src/pages/Settings.tsx (예시)
import { useSetLogConfig } from '../api/hooks';

function LogSettings() {
    const { mutate, isPending, isSuccess, isError, error } = useSetLogConfig();

    const handleSave = () => {
        mutate({ retentionDays: 30, maxSizeMb: 500, apiDebug: false });
    };

    return (
        <>
            <Button
                onClick={handleSave}
                disabled={isPending}
                startIcon={isPending ? <CircularProgress size={16} /> : <SaveIcon />}
            >
                {isPending ? '저장 중...' : '저장'}
            </Button>

            {isSuccess && <Alert severity="success">저장되었습니다.</Alert>}
            {isError && (
                <Alert severity="error">
                    {(error as CmdError)?.message ?? '저장 실패'}
                </Alert>
            )}
        </>
    );
}
```

### 토스트 알림 패턴 (onSuccess/onError 콜백)

```tsx
const { mutate: startTrading } = useMutation({
    mutationFn: () => invoke('start_trading'),
    onSuccess: () => {
        enqueueSnackbar('자동 매매를 시작했습니다.', { variant: 'success' });
        queryClient.invalidateQueries({ queryKey: KEYS.tradingStatus });
    },
    onError: (err: CmdError) => {
        enqueueSnackbar(err.message, { variant: 'error' });
    },
});
```

### 주기적 자동 갱신 패턴 (refetchInterval)

```tsx
// src/api/hooks.ts
export const useTradingStatus = () =>
    useQuery({
        queryKey: KEYS.tradingStatus,
        queryFn: getTradingStatus,
        refetchInterval: (query) => {
            // 자동매매 실행 중에만 빠른 갱신
            return query.state.data?.isRunning ? 3000 : 10000;
        },
    });
```

### 장시간 작업 진행률 표시 (Tauri Event 활용)

```tsx
// Rust에서 이벤트 발행
app_handle.emit("progress", ProgressPayload { percent: 50, message: "처리 중...".into() });

// React에서 이벤트 구독
useEffect(() => {
    const unlisten = listen<ProgressPayload>('progress', (event) => {
        setProgress(event.payload.percent);
    });
    return () => { unlisten.then(f => f()); };
}, []);
```

---

## 6. 데이터 저장 (JSON 파일)

### 저장 경로 규칙

```
실행 위치 기준:
  ./logs/           — 로그 파일 (app.log.YYYY-MM-DD, error.log.YYYY-MM-DD)
  ./data/           — 모든 데이터
    trades/YYYY/MM/DD/trades.json   — 체결 기록 (일별)
    stats/YYYY/MM/DD/stats.json     — 통계 (일별)
    orders/YYYY/MM/DD/orders.json   — 주문 이력 (일별)
    stocklist/stocklist.json        — 종목 캐시
    profiles.json                   — 계좌 프로파일
    trade_archive_config.json       — 체결 기록 보관 설정

macOS ~/Library/Application Support/... 는 사용하지 않는다.
```

### 비동기 저장 패턴

```rust
// src-tauri/src/storage/trade_store.rs 패턴
pub async fn append(&self, record: TradeRecord) -> anyhow::Result<()> {
    let path = self.path_for_date(record.date);
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    let mut records = self.get_by_date(record.date).await.unwrap_or_default();
    records.push(record);
    let json = serde_json::to_string_pretty(&records)?;
    tokio::fs::write(&path, json).await?;
    Ok(())
}
```

### 날짜별 경로 생성 패턴

```rust
fn path_for_date(&self, date: chrono::NaiveDate) -> PathBuf {
    self.data_dir
        .join("trades")
        .join(date.format("%Y").to_string())
        .join(date.format("%m").to_string())
        .join(date.format("%d").to_string())
        .join("trades.json")
}
```

---

## 7. 백그라운드 데몬 작성 원칙

### 시작 방법 (`src-tauri/src/lib.rs`)

```rust
// 앱 시작 시 영구 데몬 spawn — Tauri async runtime 사용
tauri::async_runtime::spawn(commands::run_trading_daemon(
    Arc::clone(&app_state.is_trading),
    Arc::clone(&app_state.strategy_manager),
    // ... 필요한 Arc 클론 ...
));

// 동기 컨텍스트에서 시작하는 경우 (setup() 내부) — std::thread::spawn 사용
// ⚠️ setup()은 동기 컨텍스트 — tokio::task::spawn_blocking 사용 시 패닉
std::thread::spawn(move || {
    commands::purge_old_trade_files(&data_dir, &trade_cfg);
});
```

### 데몬 구조 원칙

```rust
pub async fn my_daemon(flag: Arc<Mutex<bool>>, ...) {
    loop {
        // Phase 1: 비활성 체크 → continue (레이블 없음)
        if !*flag.lock().await {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        // Phase 2~N: 순차 처리
        // ...

        // 마지막: 다음 사이클까지 대기
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
```

### 중단 가능한 대기 (100ms 단위 poll)

```rust
// 최대 10초 대기하지만 flag가 false 되면 즉시 탈출
for _ in 0u32..100 {
    if !*flag.lock().await { break; }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

---

## 8. Rust 제어 흐름 원칙 (goto 유사 패턴 금지)

> Rust의 레이블 루프(`'label: loop`)는 C의 `goto`처럼 흐름을 비선형으로 만든다.  
> **이 프로젝트에서는 레이블 루프를 금지한다.**

### ❌ 금지 패턴 — 레이블 루프

```rust
'outer: loop {
    // ...
    'inner: for item in &items {
        if some_condition {
            break 'outer;      // ← goto와 동일한 비선형 흐름
        }
        if other_condition {
            continue 'outer;   // ← 외부 루프로 점프
        }
    }
}
```

### ✅ 올바른 패턴 — 함수 분리 + return

```rust
/// 내부 루프 로직을 별도 함수로 추출
async fn process_items(items: &[Item], flag: &Arc<Mutex<bool>>) -> ProcessResult {
    for item in items {
        if !*flag.lock().await {
            return ProcessResult::Stopped;   // break 'outer 대신 return
        }
        if some_condition {
            return ProcessResult::NeedsRetry; // continue 'outer 대신 return
        }
        // 정상 처리
    }
    ProcessResult::Done
}

// 호출 측 — 단순 순차 흐름
loop {
    let result = process_items(&items, &flag).await;
    if result == ProcessResult::NeedsRetry {
        continue;   // 레이블 없는 continue — 외부 루프만
    }
    // ...
}
```

### 실제 예시: `poll_symbols_tick` (`src-tauri/src/commands.rs`)

```rust
/// TickCycleResult — 함수 반환값으로 호출자에게 상태 전달
#[derive(Debug, PartialEq)]
enum TickCycleResult {
    Done,
    MarketClosed,
    Stopped,
}

async fn poll_symbols_tick(symbols: &[String], ...) -> TickCycleResult {
    for symbol in symbols {
        if !*is_trading.lock().await {
            return TickCycleResult::Stopped;  // break 'symbol_loop 대신
        }
        // ...
        if is_market_closed_error(&msg) {
            return TickCycleResult::MarketClosed;  // break 'symbol_loop 대신
        }
    }
    TickCycleResult::Done
}

// 호출자: run_trading_daemon
loop {
    // ...
    let result = poll_symbols_tick(&symbols, ...).await;
    if result == TickCycleResult::MarketClosed {
        market_pause_until = Some(Instant::now() + Duration::from_secs(300));
        continue;  // 레이블 없는 continue
    }
    // ...
}
```

---

## 9. 에러 처리 패턴

### Rust — CmdResult

```rust
// IPC 커맨드의 표준 에러 타입
pub struct CmdError {
    pub code: String,    // 에러 코드 (대문자 스네이크: "NOT_FOUND")
    pub message: String, // 사람이 읽는 메시지
}

// anyhow 에러를 CmdError로 변환
fn from(e: anyhow::Error) -> Self {
    Self { code: "INTERNAL".into(), message: e.to_string() }
}

// 사용 예
pub async fn get_balance(state: State<'_, AppState>) -> CmdResult<BalanceResult> {
    let client = state.rest_client.read().await.clone();
    client.get_balance().await.map_err(CmdError::from)
}
```

### TypeScript — 에러 타입

```typescript
// src/api/types.ts
export interface CmdError {
    code: string;
    message: string;
}

// 훅에서 에러 처리
const { error } = useMutation({ ... });
const cmdError = error as CmdError;
console.error(cmdError?.code, cmdError?.message);
```

---

## 10. TypeScript 타입 미러링

Rust `#[serde(rename_all = "camelCase")]` ↔ TypeScript 인터페이스는 1:1 대응한다.

### Rust 구조체

```rust
// src-tauri/src/commands.rs
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingStatus {
    pub is_running: bool,
    pub active_strategies: Vec<String>,
    pub position_count: usize,
    pub total_unrealized_pnl: i64,
    pub ws_connected: bool,
    pub trading_profile_id: Option<String>,
    pub buy_suspended: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_suspended_reason: Option<String>,
}
```

### TypeScript 인터페이스 (1:1 미러)

```typescript
// src/api/types.ts
export interface TradingStatus {
    isRunning: boolean;
    activeStrategies: string[];
    positionCount: number;
    totalUnrealizedPnl: number;
    wsConnected: boolean;
    tradingProfileId: string | null;
    buySuspended: boolean;
    buySuspendedReason?: string;  // skip_serializing_if → optional
}
```

### 규칙 요약

| Rust | TypeScript |
|------|-----------|
| `snake_case` 필드 + `camelCase` serde | `camelCase` 인터페이스 필드 |
| `Option<T>` | `T \| null` 또는 `T?` (skip_serializing_if 시) |
| `u64`, `i64` | `number` |
| `Vec<T>` | `T[]` |
| `bool` | `boolean` |

---

## 11. React Query 캐시 무효화 패턴

### KEYS 상수 정의 (`src/api/hooks.ts`)

```typescript
export const KEYS = {
    appConfig:          ['appConfig'] as const,
    tradingStatus:      ['tradingStatus'] as const,
    balance:            ['balance'] as const,
    overseasBalance:    ['overseasBalance'] as const,
    positions:          ['positions'] as const,
    strategies:         ['strategies'] as const,
    logConfig:          ['logConfig'] as const,
    tradeArchiveConfig: ['tradeArchiveConfig'] as const,
    // ... 새 쿼리 추가 시 여기에
};
```

### mutation 후 관련 쿼리 무효화

```typescript
export const usePlaceOrder = () =>
    useMutation({
        mutationFn: placeOrder,
        onSuccess: () => {
            // 주문 후 잔고·포지션·당일 통계 즉시 갱신
            queryClient.invalidateQueries({ queryKey: KEYS.balance });
            queryClient.invalidateQueries({ queryKey: KEYS.positions });
            queryClient.invalidateQueries({ queryKey: KEYS.todayStats });
        },
    });
```

---

## 12. 기능 구현 완료 체크리스트

기능을 완전히 구현했다고 판단하기 전에 아래 항목을 확인한다.

### Rust 백엔드

- [ ] `cargo check` 경고 0개
- [ ] 새 커맨드가 `lib.rs` `generate_handler!`에 등록됨
- [ ] `AppState` 필드 추가 시 `AppState::new()`에 초기화 코드 있음
- [ ] 에러는 `CmdResult<T>` 반환 (panic/unwrap 최소화)

### TypeScript / React

- [ ] `npx tsc --noEmit` 경고 0개
- [ ] `src/api/types.ts` 에 Rust 구조체와 1:1 인터페이스 추가
- [ ] `src/api/commands.ts` 에 invoke 래퍼 추가
- [ ] `src/api/hooks.ts` 에 Query/Mutation 훅 추가
- [ ] `KEYS` 상수에 새 쿼리 키 추가

### UI 동기화 (순수 인프라 제외)

- [ ] 새 설정값 → Settings 페이지에서 수정 가능
- [ ] 새 상태 플래그 → Dashboard 또는 관련 화면에 표시
- [ ] 새 IPC 커맨드 → 호출하는 UI 버튼/훅 존재
- [ ] 에러 상태 → 사용자에게 알림(토스트·배지·색상) 표시

### 문서

- [ ] `agent.md` 디렉토리 맵 및 IPC 커맨드 목록 업데이트
- [ ] `.github/skills/` 관련 스킬 파일 업데이트
- [ ] 이 `coding-guide.md`에 새 패턴이 생긴 경우 반영

---

> 마지막 업데이트: 2026-04-11T00:00:00
