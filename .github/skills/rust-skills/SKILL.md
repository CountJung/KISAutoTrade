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

### Arc<RwLock<T>> 읽기 많은 공유 상태

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

> 마지막 업데이트: 2026-04-11T00:00:00
