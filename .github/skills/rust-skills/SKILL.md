---
name: rust-skills
description: "Rust 코딩 범용 스킬. 데이터 구조 설계, trait 구현, 매크로, 모듈 구조, 소유권/라이프타임, 에러 처리, clippy/fmt 규칙. Keywords: rust, struct, enum, trait, ownership, lifetime, Result, clippy, cargo, module, thiserror, serde, tokio, async"
---
# Rust Coding Skill

## Instructions

1. **요청 완전 이해**  
   데이터 구조 설계, trait 구현, 매크로 작성, 도메인 로직 모델링, 모듈 구성 중 어느 작업인지 파악합니다.  
   가변성 필요성, 소유권 흐름, async 컨텍스트, 내부 가변성, 동시성 경계 등 핵심 제약을 식별합니다.

2. **데이터 구조 정밀 설계**
   - 도메인 요구에 따라 `struct`, `enum`, `newtype` 중 선택합니다.
   - 각 필드의 소유권을 고려합니다: `&str` vs `String`, 슬라이스 vs 벡터, 공유 시 `Arc<T>`, 유연한 소유권에 `Cow<'a, T>`.
   - 타입으로 불변성을 명시합니다 (예: `NonZeroU32`, `Duration`, 커스텀 enum).
   - 상태 머신에는 boolean 플래그 대신 `enum`을 사용합니다.

3. **관용적 Rust 구현**
   - `impl` 블록은 struct/enum 바로 아래에 배치합니다.
   - 관련 메서드를 그룹화합니다: 생성자, 게터, 변이 메서드, 도메인 로직, 헬퍼.
   - 적절한 생성자(`new`, `with_capacity`, 빌더)를 제공합니다.
   - 변환 단순화를 위해 trait(`Display`, `Debug`, `From`, `Into`, `TryFrom`)을 구현합니다.
   - 패닉 대신 `Result<T, E>` 반환을 선호합니다.
   - 함수는 짧게 유지하여 라이프타임 추론과 명확성을 향상시킵니다.

4. **문서화 및 코드 스타일 규칙**
   - struct, enum, 필드, 메서드에 `///` 문서 주석을 사용합니다.
   - 설계나 아키텍처 설명 시 `//!` 모듈 수준 문서를 사용합니다.
   - `cargo fmt` 와 `cargo clippy --all-targets --all-features` 를 실행합니다.
   - 논리적으로 구분되는 메서드와 섹션 사이에 빈 줄을 사용합니다.

5. **매크로 효과적 활용**
   - `derive` 매크로(`Debug`, `Clone`, `Serialize`, `Deserialize`)로 보일러플레이트를 줄입니다.
   - 반복 패턴 제거를 위해 작고 집중된 선언적 매크로를 만듭니다.
   - `unwrap()` / `expect()` 사용 최소화 — `?` 연산자와 `Result` 타입 활용.

6. **빌드 속도 최적화**

   ```toml
   # .cargo/config.toml (Windows msvc)
   [target.x86_64-pc-windows-msvc]
   linker = "rust-lld"

   # Cargo.toml — 빠른 개발 빌드
   [profile.dev]
   opt-level = 0
   debug = true
   ```

   - `cargo check` 로 빠른 반복 — `cargo build` 대신 우선 사용
   - `sccache` 로 컴파일 아티팩트 캐싱
   - 불필요한 의존성과 feature 플래그 최소화

7. **모듈 및 프로젝트 구조**
   - 소유권과 도메인 경계를 반영하여 모듈을 구성합니다.
   - 가능하면 `pub(crate)` 사용 — 노출이 필요한 것만 `pub` 처리.
   - API를 작고 표현적으로 유지하고 내부 타입 노출을 피합니다.
   - 기능과 일치하는 의미 있는 파일 및 모듈 이름을 사용합니다.

---

## 패턴 예시

### thiserror 기반 에러 정의

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type AppResult<T> = Result<T, AppError>;
```

### Tauri IPC Command 패턴 (이 프로젝트)

```rust
// CmdResult<T> = Result<T, CmdError> — 모든 IPC 커맨드 반환 타입
#[derive(Debug, Serialize)]
pub struct CmdError {
    pub code: String,
    pub message: String,
}
pub type CmdResult<T> = Result<T, CmdError>;

// IPC 핸들러는 얇게 — 비즈니스 로직은 도메인 모듈에 위임
#[tauri::command]
pub async fn get_balance(state: State<'_, AppState>) -> CmdResult<BalanceResult> {
    state.rest_client
        .get_balance()
        .await
        .map_err(|e| CmdError { code: "BALANCE_ERR".into(), message: e.to_string() })
}
```

### AppState 공유 패턴 (이 프로젝트)

```rust
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub rest_client: Arc<KisRestClient>,        // 읽기 많음 → Arc
    pub trade_store: Arc<TradeStore>,
    pub stats_store: Arc<StatsStore>,
    pub discord: Option<Arc<DiscordNotifier>>,
}

// JSON Storage 공유가 필요한 경우
pub struct TradeStore {
    data_dir: PathBuf,
    // 동시 쓰기 시 Mutex 추가
}
```

### Serde camelCase 매핑 (TypeScript 1:1)

```rust
// IPC 반환 struct → camelCase (TypeScript interface와 1:1 매핑)
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeRecord {
    pub id: String,
    pub symbol_name: String,  // JSON: "symbolName"
    pub executed_at: String,  // JSON: "executedAt"
    pub pnl: Option<i64>,
}

// 내부 enum → camelCase variant
#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TradeSide { Buy, Sell }
// JSON: "buy", "sell"
```

---

### ⚠️ 웹 핸들러(axum)에서 내부 Struct 직렬화 금지 패턴

> **2026-04-16 실제 버그에서 추출** — Strategy 페이지 `Cannot read properties of undefined (reading 'length')` 오류

이 프로젝트는 **Tauri IPC + axum 웹 REST** 듀얼 모드를 지원한다.
내부 도메인 struct(`StrategyConfig`, `TradeRecord` 등)는 디스크 저장 포맷이 snake_case이므로
`#[serde(rename_all = "camelCase")]`가 붙어 있지 않다.

axum 핸들러에서 이런 struct를 `serde_json::to_value(struct)` 로 직접 직렬화하면
**snake_case JSON이 프론트엔드에 전달**되고, TypeScript가 `undefined`로 읽어 런타임 크래시가 발생한다.

#### ❌ 잘못된 패턴 (snake_case 노출)

```rust
async fn strategies_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let mgr = s.strategy_manager.lock().await;
    let configs = mgr.all_configs();                      // Vec<&StrategyConfig>
    Json(serde_json::to_value(configs).unwrap_or_default()) // ← snake_case 출력!
    // → { "target_symbols": [...], "order_quantity": 1 }
    // TypeScript는 targetSymbols를 찾으므로 undefined → 런타임 크래시
}
```

#### ✅ 올바른 패턴 A: serde_json::json! 매크로로 직접 조립

```rust
async fn strategies_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let configs: Vec<StrategyConfig> = {
        let mgr = s.strategy_manager.lock().await;
        mgr.all_configs().into_iter().cloned().collect()
    };
    let views: Vec<_> = configs.iter().map(|cfg| serde_json::json!({
        "id":             cfg.id,
        "name":           cfg.name,
        "enabled":        cfg.enabled,
        "targetSymbols":  cfg.target_symbols,   // ← 직접 camelCase 키 지정
        "orderQuantity":  cfg.order_quantity,
        "params":         cfg.params,
    })).collect();
    Json(serde_json::Value::Array(views))
}
```

#### ✅ 올바른 패턴 B: 별도 View struct 사용 (필드가 많을 때 권장)

```rust
/// axum 응답 전용 View — camelCase로 직렬화
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StrategyView {
    id: String,
    name: String,
    enabled: bool,
    target_symbols: Vec<String>,          // JSON: "targetSymbols"
    target_symbol_names: HashMap<String, String>, // JSON: "targetSymbolNames"
    order_quantity: u64,                  // JSON: "orderQuantity"
    params: serde_json::Value,
}

// StrategyConfig → StrategyView 변환 후 직렬화
let view = StrategyView { target_symbols: cfg.target_symbols.clone(), ... };
Json(serde_json::to_value(view)?)
```

> **판단 기준**: 필드 3개 이하 → json! 매크로, 4개 이상 또는 중첩 타입 포함 → View struct

#### 체크리스트 — axum 핸들러 작성 시

- [ ] 내부 struct를 `serde_json::to_value()`로 직접 직렬화하지 않는다
- [ ] `serde_json::json!{}` 매크로로 camelCase 키를 명시하거나 View struct를 별도 정의한다
- [ ] Tauri IPC `commands.rs`의 동일 커맨드 응답 필드와 **키 이름이 일치하는지** 확인한다
- [ ] TypeScript `types.ts`의 interface 필드명과 한 번 더 대조한다

---

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TokenManager {
    token: Arc<Mutex<Option<AccessToken>>>,
    base_url: String,
    app_key: String,
    app_secret: String,
}

impl TokenManager {
    pub async fn get_token(&self) -> Result<String, ApiError> {
        let mut guard = self.token.lock().await;
        if guard.as_ref().map_or(true, |t| t.is_expired()) {
            *guard = Some(self.issue_token().await?);
        }
        Ok(guard.as_ref().unwrap().token.clone())
    }
}
```

### 입력 검증 (보안)

```rust
/// 종목 코드 형식 검증 — Shell Injection 방지
fn validate_symbol(symbol: &str) -> Result<(), CmdError> {
    if symbol.len() != 6 || !symbol.chars().all(|c| c.is_ascii_digit()) {
        return Err(CmdError {
            code: "INVALID_SYMBOL".into(),
            message: format!("종목코드는 6자리 숫자여야 합니다: {symbol}"),
        });
    }
    Ok(())
}
```

### Strategy trait 패턴 (이 프로젝트)

```rust
#[async_trait]
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn is_enabled(&self) -> bool;
    fn on_tick(&mut self, symbol: &str, price: u64) -> Signal;
    fn reset(&mut self);
}

// StrategyManager — Vec<Box<dyn Strategy>>
pub struct StrategyManager {
    strategies: Vec<Box<dyn Strategy>>,
}

impl StrategyManager {
    pub fn on_tick(&mut self, symbol: &str, price: u64) -> Vec<Signal> {
        self.strategies.iter_mut()
            .filter(|s| s.is_enabled())
            .map(|s| s.on_tick(symbol, price))
            .filter(|sig| !matches!(sig, Signal::Hold))
            .collect()
    }
}
```

---

## 설계 결정 지침

| 상황 | 권장 패턴 |
|------|-----------|
| 상태 표현 | `enum` (boolean 플래그 대신) |
| 공유 소유권 | `Arc<T>` (clone 가능한 핸들) |
| 내부 가변성 (write) | `Mutex<T>` (async) |
| 내부 가변성 (read-heavy) | `RwLock<T>` |
| 에러 전파 | `thiserror` + `?` 연산자 |
| IPC 에러 | `CmdError { code, message }` |
| 비동기 실행 | `tokio::spawn` + `async fn` |
| 파일 I/O | `tokio::fs` + `serde_json` |
| 설정 영속화 | JSON 파일 (`secure_config.json`) |

---

## ⚠️ 외부 문자열 값 → 내부 enum 매핑

```rust
// ✅ 안전: toLowerCase + alias 포함
fn parse_order_side(s: &str) -> Result<OrderSide, CmdError> {
    match s.to_lowercase().as_str() {
        "buy" | "매수"  => Ok(OrderSide::Buy),
        "sell" | "매도" => Ok(OrderSide::Sell),
        other => Err(CmdError {
            code: "INVALID_SIDE".into(),
            message: format!("알 수 없는 주문 방향: {other}"),
        }),
    }
}
```

---

## Runtime 상태 영구 저장 패턴 (sync load + async save)

AppState 구조체가 sync 초기화(`fn new`)를 사용할 때 파일 I/O 처리:

```rust
// 저장소 구조체
pub struct StrategyStore { data_dir: PathBuf }

impl StrategyStore {
    // AppState::new() (sync) — std::fs 사용
    pub fn load_sync(&self, profile_id: &str) -> Vec<StrategyConfig> {
        let path = self.config_path(profile_id);
        if !path.exists() { return vec![]; }
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    // IPC 커맨드 (async) — tokio::fs 사용
    pub async fn save(&self, profile_id: &str, configs: &[StrategyConfig]) -> anyhow::Result<()> {
        let path = self.config_path(profile_id);
        if let Some(parent) = path.parent() { tokio::fs::create_dir_all(parent).await?; }
        tokio::fs::write(&path, serde_json::to_string_pretty(configs)?).await?;
        Ok(())
    }
}
```

핵심: `AppState::new()`는 sync → `std::fs`, IPC 커맨드는 async → `tokio::fs`

---

## 10. 전략별 종목 독립 상태 패턴 (Per-Symbol HashMap State)

자동매매 전략처럼 **여러 종목을 하나의 전략 인스턴스에서 처리**할 때, 종목별 상태를 독립 관리하는 패턴.

### 문제: 단일 상태 공유

```rust
// ❌ 잘못된 패턴 — 모든 종목이 prices/in_position 공유
pub struct SomeStrategy {
    prices: VecDeque<u64>,  // KODEX+ACE 가격이 뒤섞임
    in_position: bool,       // 한 종목이 매수하면 다른 종목 매수 불가
}
```

### 해결: 종목코드 키 HashMap

```rust
// ✅ 올바른 패턴 — 종목별 독립 상태
struct SomeState {
    prices: VecDeque<u64>,
    in_position: bool,
    // 전략별 추가 필드 (last_buy_price, avg_range 등)
}

pub struct SomeStrategy {
    config: StrategyConfig,
    params: SomeParams,
    states: std::collections::HashMap<String, SomeState>,  // 종목코드 → 상태
}

impl SomeStrategy {
    fn on_tick(&mut self, symbol: &str, price: u64, ...) -> Option<Signal> {
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| SomeState {
            prices: VecDeque::new(),
            in_position: false,
        });
        // state를 통해 종목별 독립 판단
    }
}
```

### reset() 패턴별 구현

```rust
// 패턴 A: 단순 초기화 (initialize_*에서 복원 가능한 경우)
fn reset(&mut self) {
    self.states.clear();
}

// 패턴 B: 가격 버퍼 유지, 포지션만 초기화 (MA 계산에 히스토리 필요한 경우)
fn reset(&mut self) {
    for state in self.states.values_mut() {
        state.in_position = false;
        state.entry_price = None;
    }
}

// 패턴 C: 일부 필드만 유지 (VolatilityExpansionStrategy — avg_range 유지)
fn reset(&mut self) {
    for state in self.states.values_mut() {
        // avg_range는 유지 (초기화하면 다음 날 첫 틱까지 신호 없음)
        state.day_open = None;
        state.day_high = None;
        state.day_low = None;
        state.in_position = false;
        state.entry_price = None;
    }
}
```

### 이 프로젝트에서의 적용

`strategy.rs`의 11개 전략 모두 이 패턴 사용:

| 전략 | State 구조체 | 핵심 유지 필드 |
|------|------------|--------------|
| MA교차(1) | `MaCrossState` | `prev_short_ma`, `prev_long_ma` |
| RSI(2) | 인라인 튜플 | `(VecDeque<u64>, Option<f64>)` |
| 모멘텀(3) | `MomentumState` | `last_buy_price`, `last_sell_price` |
| 이격도(4) | 인라인 | `VecDeque<u64>` |
| 52주신고가(5) | `FiftyTwoWeekState` | `prev_price`, `high_52w`, `buy_price` |
| 연속상승(6) | `ConsecutiveMoveState` | `prices`, `in_position` |
| 돌파실패(7) | `FailedBreakoutState` | `breakout_prev_high` |
| 강한종가(8) | `StrongCloseState` | `pending_buy`, `entry_price` |
| 변동성확장(9) | `VolatilityExpansionState` | `avg_range` (reset 후 유지) |
| 평균회귀(10) | `MeanReversionState` | `prices` (reset 후 유지) |
| 추세필터(11) | `TrendFilterState` | `prices` (reset 후 유지) |

---

## 레이블 루프 금지 원칙

### ❌ 잘못된 패턴 — goto 유사 레이블 루프

```rust
// 이 프로젝트에서 발견된 실제 버그 패턴
'main_loop: loop {
    // ...
    'inner: for item in &items {
        if condition {
            break 'main_loop;      // 외부 루프로 jump — 비선형 흐름
        }
        if other {
            continue 'main_loop;   // 외부 루프로 jump — goto와 동일
        }
    }
    // 내부 루프 실행 이후 흐름이 불명확
    if some_flag { continue 'main_loop; }
}
```

**문제**: 내부 루프에서 `break 'outer` / `continue 'outer` 를 사용하면:
- 코드를 위→아래로 읽을 때 흐름 추적이 불가능
- 중간에 상태(`market_pause_until` 등)를 변경한 채로 루프를 탈출하면 디버깅 어려움
- C/C++의 `goto`와 동일한 가독성 문제 발생

### ✅ 올바른 패턴 — 함수 분리 + return

내부 루프 로직을 별도 `async fn`으로 추출하고, 조기 탈출은 `return`으로 처리한다.
호출자는 반환값(enum)을 보고 후속 동작을 결정한다.

```rust
/// 처리 결과를 enum으로 명시 — 호출자가 결과에 따라 분기
#[derive(Debug, PartialEq)]
enum TickCycleResult {
    Done,
    MarketClosed,   // → 호출자가 pause 설정
    Stopped,        // → 호출자가 다음 사이클 skip
}

/// 내부 루프를 별도 함수로 추출 — 조기 탈출은 return
async fn process_items(items: &[String], flag: &Arc<Mutex<bool>>) -> TickCycleResult {
    for item in items {
        if !*flag.lock().await {
            return TickCycleResult::Stopped;    // break 'outer 대신
        }
        if is_closed(item) {
            return TickCycleResult::MarketClosed; // break 'outer 대신
        }
        // 정상 처리
    }
    TickCycleResult::Done
}

/// 호출자 — 레이블 없는 단순 loop
loop {
    // ...
    let result = process_items(&items, &flag).await;
    if result == TickCycleResult::MarketClosed {
        pause_until = Some(Instant::now() + Duration::from_secs(300));
        continue;   // 레이블 없는 continue — 외부 루프만
    }
    // ...
}
```

**실제 적용**: `commands.rs`의 `run_trading_daemon` + `poll_symbols_tick`

---

## 8. Tauri 백그라운드 데몬 패턴

### watch::Sender로 런타임 interval 변경

설정값(간격 등)이 UI에서 실시간으로 바뀌어야 할 때 `tokio::sync::watch` 채널 사용.

```rust
// AppState 초기화 시
let (interval_tx, _) = tokio::sync::watch::channel(initial_interval);
// AppState 필드: refresh_interval_tx: Arc<watch::Sender<u64>>

// 데몬 내부
let mut interval_rx = interval_tx.subscribe();
let mut current_interval = *interval_rx.borrow_and_update();
loop {
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(current_interval)) => {
            // 주기 작업 수행
        }
        _ = interval_rx.changed() => {
            current_interval = *interval_rx.borrow_and_update();
            tracing::info!("간격 변경: {}초", current_interval);
            // 즉시 새 간격으로 재시작
        }
    }
}

// IPC 커맨드에서 간격 변경
let _ = state.refresh_interval_tx.send(new_interval);
```

### on_window_event — 앱 종료 시 안전 처리

Tauri v2 Builder에 `on_window_event` 등록으로 종료 시 자동매매 정지:

```rust
.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { .. } = event {
        let state: tauri::State<commands::AppState> = window.state();
        let is_trading = state.is_trading.clone();
        // sync 컨텍스트 → spawn으로 비동기 작업 예약
        tauri::async_runtime::spawn(async move {
            let mut guard = is_trading.lock().await;
            if *guard {
                *guard = false;
                tracing::info!("앱 종료 — 자동매매 정지 신호 전송");
            }
        });
        tracing::info!("앱 종료 요청");
    }
})
```

> ⚠️ `on_window_event` 콜백은 동기 컨텍스트. `tokio::sync::Mutex` 의 `.lock().await` 는 직접 호출 불가 → `spawn` 으로 예약.

### .env 파일 단일 값 저장/로드 패턴

`WEB_PORT`, `REFRESH_INTERVAL_SEC` 등 단순 값은 `.env` 에 저장 (JSON 파일 불필요):

```rust
// 저장 (기존 줄 교체)
use std::io::Write;
let env_path = std::env::current_dir().unwrap_or_default().join(".env");
let existing = std::fs::read_to_string(&env_path).unwrap_or_default();
let mut lines: Vec<String> = existing
    .lines()
    .filter(|l| !l.starts_with("MY_KEY="))
    .map(String::from)
    .collect();
lines.push(format!("MY_KEY={}", value));
std::fs::OpenOptions::new()
    .write(true).create(true).truncate(true)
    .open(&env_path)
    .and_then(|mut f| f.write_all(lines.join("\n").as_bytes()))?;

// 로드
let value = std::fs::read_to_string(&env_path)
    .unwrap_or_default()
    .lines()
    .find(|l| l.starts_with("MY_KEY="))
    .and_then(|l| l["MY_KEY=".len()..].parse::<u64>().ok())
    .unwrap_or(default_value);
```

> ❌ 단순 설정 값에 별도 JSON 파일(`refresh_config.json`) 사용 — 복잡도만 증가  
> ✅ `.env` 에 `KEY=value` 형식으로 통일 (WEB_PORT, REFRESH_INTERVAL_SEC 등)

---

## 9. 자동매매 반복 매매 방지 패턴

등락 반복 구간에서 전략이 매수/매도를 빠르게 반복하면 수수료, 세금, 슬리피지 때문에 손실이 누적된다. 이 문제는 개별 전략 파라미터만으로 해결하지 말고, 전략 신호와 주문 실행 사이의 공통 방어 계층으로 모델링한다.

### 권장 구조

```rust
pub enum GuardDecision {
    Allow,
    Block { reason: String },
}

pub struct TradeGuard {
    cooldowns: std::collections::HashMap<(String, GuardSide), chrono::DateTime<chrono::Utc>>,
    min_expected_profit_bps: i32,
}
```

### 검사 순서

1. 동일 종목/동일 방향 미체결 주문 존재 여부
2. 손절 후 재진입 금지 시간
3. 매도 직후 재매수 쿨다운
4. 예상 순손익이 수수료·세금·슬리피지보다 큰지 확인
5. 최근 N틱 내 반대 신호 반복 횟수 확인

### ❌ 잘못된 패턴 — 전략 신호 즉시 주문

```rust
let signals = strategy_mgr.lock().await.on_tick(symbol, price, volume);
for signal in signals {
    order_mgr.lock().await.submit_signal(signal, &symbol_name, 0, exchange, price).await?;
}
```

### ✅ 올바른 패턴 — 공통 guard 통과 후 주문

```rust
let decision = trade_guard.lock().await.evaluate(&signal, symbol, price, now);
if matches!(decision, GuardDecision::Allow) {
    order_mgr.lock().await.submit_signal(signal, &symbol_name, total_balance, exchange, price).await?;
}
```

### 상태 복원 주의

전략 내부 `in_position` 플래그는 프로세스 재시작 또는 수동 매매 후 실제 잔고와 어긋날 수 있다. 자동매매 시작 시 국내는 `PositionTracker`/KIS 잔고, 해외는 `get_overseas_balance()`를 기준으로 전략 상태를 복원하는 API를 별도로 설계한다.

> 마지막 업데이트: 2026-06-30T00:00:00

---

## 11. 전략 신호 공통 TradeGuard 패턴

전략별 `on_tick()`에서 반복 매매 방어를 각각 구현하면 전략마다 기준이 달라지고 누락이 생긴다.
전략 신호와 주문 실행 사이의 `OrderManager::submit_signal()`에서 공통 `TradeGuard`를 먼저 통과시킨다.

### 구현 위치

| 역할 | 위치 |
|------|------|
| guard 상태/판단 | `src-tauri/src/trading/guard.rs` |
| 주문 전 평가 | `src-tauri/src/trading/order.rs::submit_signal()` |
| 일별 초기화 | `OrderManager::reset_day()` |

### 검사 항목

1. 동일 종목/동일 방향 쿨다운
2. 매도 후 재매수 쿨다운
3. 손절 후 당일 재진입 금지
4. 최근 반대 방향 신호 반복 시 종목 단위 휩소 쿨다운
5. 익절 매도에서 수수료·세금·슬리피지 차감 후 기대 순익 확인
6. `RiskManager`의 전략 ID + 종목 + 방향 + 날짜별 일일 주문 접수 횟수 제한
7. `RiskManager`의 전략 ID + 종목별 연속 손실 신규 진입 차단

```rust
let strategy_signal = StrategySignal {
    strategy_id: s.id().to_string(),
    signal,
};

if let Some(reason) = risk.daily_order_limit_reason(strategy_id, symbol, side) {
    tracing::info!("리스크 주문 횟수 제한 — {}", reason);
    return Ok(());
}

match self.trade_guard.evaluate(&signal, held_qty, avg_price, tick_price, exchange.is_some()) {
    GuardDecision::Allow => {}
    GuardDecision::Block { reason } => {
        tracing::info!("TradeGuard 차단 — {}", reason);
        return Ok(());
    }
}
```

주문 횟수 카운터는 신호 발생 시점이 아니라 KIS가 주문을 접수해 pending에 등록된 뒤 증가시킨다. 기본값은 전략/종목별 매수 1회, 매도 1회이며 0은 제한 없음으로 취급한다.

연속 손실 차단은 매도 체결로 PnL이 확정된 뒤 `record_strategy_symbol_pnl()`에서 갱신한다. 손실이면 카운터를 증가시키고, 수익이면 해당 전략/종목의 카운터와 차단 상태를 해제한다. 포지션 청산을 막지 않기 위해 차단은 신규 `Buy` 신호에만 적용한다.

> 마지막 업데이트: 2026-07-01T16:45:00

---

## 18. ATR 기반 변동성 주문 수량 산정 패턴

고정 주문 수량은 종목 가격과 변동성이 달라질 때 계좌 위험을 일정하게 유지하지 못한다.
자동매매 매수 주문은 `RiskManager`가 보관한 ATR과 계좌 총잔고를 기준으로 주문 직전에 수량을 다시 계산한다.

### 구현 위치

| 역할 | 위치 |
|------|------|
| ATR 캐시와 수량 계산 | `src-tauri/src/trading/risk.rs::RiskManager` |
| 주문 직전 수량 조정 | `src-tauri/src/trading/order.rs::process_buy()` |
| ATR 초기화 | `src-tauri/src/commands.rs::start_trading()` |
| 총잔고 전달 | `src-tauri/src/commands.rs::run_trading_daemon()` → `poll_symbols_tick()` → `OrderManager::submit_signal()` |

### 계산 기준

```rust
let risk_amount = total_balance_krw * risk_per_trade_bps / 10_000;
let stop_distance = atr * atr_stop_multiplier;
let risk_qty = risk_amount / stop_distance;
let position_qty = (total_balance_krw * max_position_ratio) / tick_price;
let adjusted_qty = min(risk_qty, position_qty);
```

- 국내 가격과 ATR은 KRW 정수 단위다.
- 해외 가격과 ATR은 USD cents 단위이며, 수량 계산과 포지션 비중 검사는 체결 시점 환율로 KRW 환산한다.
- ATR이 아직 준비되지 않았거나 총잔고를 조회하지 못하면 기존 전략 수량을 유지한다.
- `risk_per_trade_bps == 0`이면 변동성 수량 산정을 사실상 우회한다.
- 계산 결과가 0주면 매수 주문을 스킵한다.

### UI/API 동기화

`RiskConfigView`와 `UpdateRiskConfigInput`에는 아래 필드를 함께 유지한다.

| 필드 | 의미 |
|------|------|
| `volatilitySizingEnabled` | ATR 기반 수량 산정 ON/OFF |
| `riskPerTradeBps` | 거래당 허용 위험 한도. 100 = 1% |
| `atrStopMultiplier` | ATR 기반 예상 손절폭 배수 |
| `atrSymbolCount` | 자동매매 시작 후 ATR이 준비된 종목 수 |

새 리스크 필드를 추가하면 Tauri IPC, axum `/api/risk-config`, `src/api/types.ts`, Settings UI를 동시에 갱신한다.

> 마지막 업데이트: 2026-07-01T17:20:00

---

## 12. 전략 상태와 실제 잔고 동기화 패턴

내부 `in_position` 플래그가 있는 전략은 앱 재시작 후 실제 잔고와 상태가 어긋날 수 있다.
자동매매 시작 전에 KIS 잔고를 읽고 `Strategy::sync_position()` 훅으로 전략별 상태를 복원한다.

```rust
pub trait Strategy: Send + Sync {
    fn sync_position(&mut self, _symbol: &str, _quantity: u64, _avg_price: u64) {}
}
```

- 국내: `get_balance()` → `PositionTracker::load_if_empty()` + `StrategyManager::sync_position()`
- 해외: `get_overseas_balance()` → `OverseasPositionTracker::load_if_empty()` + `StrategyManager::sync_position()`
- 해외 평균단가/현재가는 USD × 100(cents)로 변환해서 전략에 전달한다.
- 해외 잔고와 체결은 국내 `PositionTracker`에 혼입하지 않는다.

> 마지막 업데이트: 2026-07-01T16:15:13

---

## 13. 레버리지/역방향 레버리지 전략 매핑 패턴

기초 지수 ETF와 매매 대상 레버리지 ETF를 고정하지 말고 설정 데이터로 분리한다.
상승 추세는 정방향 레버리지, 하락 추세는 선택적 역방향 레버리지를 매수하도록 같은 row에서 관리한다.

```rust
pub struct LeveragedTrendHoldEntry {
    pub leveraged_symbol: String,          // 예: SOXL
    pub inverse_leveraged_symbol: String,  // 예: SOXS, 비어 있으면 비활성
    pub base_symbols: Vec<String>,         // 예: SOXX, SMH
    pub quantity: u64,
    pub inverse_quantity: u64,
}
```

- `target_symbols`에는 정방향, 역방향, 기초 종목을 모두 포함해야 폴링 루프가 필요한 시세를 수집한다.
- 기초 상승 조건: 현재가 > EMA20, EMA20 > EMA60, RSI 상단 기준 이상, ADX 기준 이상, 최근 3봉 중 2개 이상 양봉.
- 기초 하락 조건: 현재가 < EMA20, EMA20 < EMA60, RSI 하단 기준 이하, ADX 기준 이상, 최근 3봉 중 2개 이상 음봉.
- 정방향/역방향 포지션은 같은 `positions: HashMap<String, ...>`에 실제 매매 종목 코드별로 독립 저장한다.

> 마지막 업데이트: 2026-06-30T00:00:00

---

## 14. 국내/해외 포지션 트래커 분리 패턴

국내 원화 포지션과 해외 USD 포지션은 가격 단위, 수수료, 환율, 거래소 코드가 다르므로 같은 `PositionTracker`에 섞지 않는다.

| 구분 | 트래커 | 가격 단위 | 통계 반영 |
|------|--------|-----------|-----------|
| 국내 | `PositionTracker` | KRW 정수 | `StatsStore`/`RiskManager` 반영 |
| 해외 | `OverseasPositionTracker` | USD × 100 cents | 체결 시점 환율로 KRW 환산 후 `StatsStore`/`RiskManager` 반영 |

```rust
pub struct OverseasPosition {
    pub symbol: String,
    pub symbol_name: String,
    pub exchange: String,          // NASD / NYSE / AMEX
    pub quantity: u64,
    pub avg_price_cents: f64,
    pub current_price_cents: u64,
}
```

`OrderManager::submit_signal()`은 `exchange.is_some()`이면 `OverseasPositionTracker`에서 보유 수량과 평균가를 읽는다.
`OrderManager::on_fill()`은 pending 주문의 `exchange`를 기준으로 국내/해외 체결 경로를 분기한다.

❌ 잘못된 패턴:
```rust
// 해외 체결가 cents를 국내 원화 PositionTracker에 저장
position_tracker.on_buy("VOO".into(), "VOO".into(), qty, price_cents);
stats.gross_profit += overseas_pnl_cents; // 원화 통계 오염
```

✅ 올바른 패턴:
```rust
overseas_position_tracker.on_buy(
    symbol,
    symbol_name,
    exchange,      // NASD / NYSE / AMEX
    qty,
    price_cents,   // USD × 100
);
```

> 마지막 업데이트: 2026-07-01T16:15:13

---

## 15. 해외 체결 수수료/환율 기록 패턴

해외 자동매매 체결은 원본 USD 값과 KRW 환산값을 함께 저장한다. `TradeRecord`의 기존 `price`, `total_amount`, `fee`는 해외에서는 USD cents 단위이고, 화면/분석용 optional 필드에 USD와 KRW 값을 명시한다.

```rust
pub fn new_overseas(
    symbol: String,
    symbol_name: String,
    side: TradeSide,
    quantity: u64,
    price_cents: u64,
    fee_cents: u64,
    order_id: String,
    strategy_id: Option<String>,
    signal_reason: String,
    exchange: String,
    exchange_rate_krw: f64,
    realized_pnl_cents: Option<i64>,
) -> Self
```

기록 필드:

| 필드 | 단위 |
|------|------|
| `price_usd`, `total_amount_usd`, `fee_usd`, `realized_pnl_usd` | USD |
| `exchange_rate_krw` | 체결 시점 USD/KRW |
| `total_amount_krw`, `fee_krw`, `realized_pnl_krw` | KRW 환산 |

해외 수수료는 KIS 체결 응답에 건별 수수료가 없으므로 체결 금액의 10bps를 추정치로 기록한다. 원화 통계에는 `fee_krw`와 `realized_pnl_krw`만 반영한다.

> 마지막 업데이트: 2026-07-01T16:22:56

---

## 16. 주문번호 기반 부분체결 상태 패턴

KIS 체결 조회 API로 주문번호를 확인할 때 `tot_ccld_qty`는 주문의 누적 체결 수량으로 취급한다. `OrderManager::on_fill()`은 누적 수량과 `PendingOrder.filled_quantity`의 차이만 포지션, 수수료, 거래 기록에 반영해야 한다.

```rust
let cumulative_filled = filled_qty.min(pending.record.quantity);
if cumulative_filled <= pending.filled_quantity {
    return Ok(());
}
let delta_qty = cumulative_filled - pending.filled_quantity;
let is_complete = cumulative_filled >= pending.record.quantity;
```

상태 분리 규칙:

| 상태 | 처리 |
|------|------|
| 미체결 | pending map에 유지, `status = Pending`, `filled_quantity = 0` |
| 부분체결 | pending map에 유지, `status = PartiallyFilled`, `filled_quantity` 갱신 |
| 완전체결 | pending map과 `symbol_to_odno`에서 제거, 증가분을 `Filled` 기록으로 저장 |
| 주문 거부 | pending에 넣지 않고 `OrderStatus::Failed` 주문 이력으로 저장 |

Dashboard 미체결 주문 IPC(`PendingOrderView`)는 `status`, `filled_quantity`, `remaining_quantity`를 camelCase로 내려 UI가 부분체결과 잔여 수량을 표시할 수 있게 한다.

> 마지막 업데이트: 2026-07-01T16:33:25

---

## 17. 자동매매 슬리피지 기록 패턴

전략 성과 분석을 위해 자동매매 체결 기록은 신호가, 주문가, 체결가를 함께 저장한다. `PendingOrder`가 신호 발생 시점 가격과 주문 제출 가격을 보존하고, 체결 시 `TradeRecord::with_execution_prices()`로 슬리피지 필드를 채운다.

| 필드 | 의미 |
|------|------|
| `signal_price` | 전략 신호가 발생한 틱 가격 |
| `order_price` | 주문 제출 가격. 국내 시장가는 0, 해외 지정가는 USD cents |
| `price` | 실제 체결 평균가 |
| `slippage` | 슬리피지 비용. 국내 KRW, 해외 USD cents |
| `slippage_bps` | 신호가 대비 슬리피지 비용 bps |

슬리피지는 비용 관점으로 계산한다. 매수는 `체결가 - 신호가`, 매도는 `신호가 - 체결가`이며 양수면 불리한 체결, 음수면 유리한 체결이다.

```rust
let slippage = match self.side {
    TradeSide::Buy => self.price as i64 - signal_price as i64,
    TradeSide::Sell => signal_price as i64 - self.price as i64,
};
```

> 마지막 업데이트: 2026-07-01T16:45:00
