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

8. **장기 실행/OOM 방지**
   - polling API, 로그 reader, daemon, cache, per-symbol map은 데이터 크기 상한을 명시합니다.
   - 최근 로그 조회처럼 주기적으로 호출되는 파일 reader는 `read_to_string()`으로 전체 파일을 읽지 말고 bounded tail reader 또는 ring buffer를 사용합니다.
   - async lock은 네트워크 await 전에 해제합니다. fallback 값이 필요하면 필요한 필드만 복사한 뒤 lock guard를 drop합니다.
   - 활성 broker가 KIS가 아닌 경우 KIS balance/order polling을 실행하지 않는 등 provider scope를 daemon에서도 확인합니다.
   - 외부 HTTP client는 명시 timeout을 설정하고, response body는 `Content-Length`와 실제 bytes 길이 상한을 모두 검사합니다.
   - 토큰 cache mutex는 cache 확인/저장에만 사용하고, 네트워크 token 발급은 별도 refresh mutex로 직렬화합니다.

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

## 프로젝트 전략 구현 메모

- 레버리지 단일 티커 추세 전략(`LeveragedTrendHoldStrategy`)은 기본적으로 장 시작 후 `entry_window_start_min`~`entry_window_end_min` 사이의 추세추종 진입만 수행한다.
- 장중 급락 후 반등을 실험할 때는 기본값으로 꺼진 `intraday_rebound_enabled`를 켜고, `rebound_baseline_ticks`, `rebound_confirm_ticks`, `rebound_pullback_pct`, `rebound_buy_pressure_pct`, `rebound_rsi_min`을 함께 저장한다.
- 장중 반동 진입은 절대 시각을 파라미터로 받지 않는다. 자동매매가 켜진 뒤 누적한 가격 관측치를 기준 구간과 바로 다음 확인 구간으로 나누고, 기준 구간 하락 후 확인 구간의 가격 회복이 충분히 강할 때 매수세 반동으로 본다.
- 장중 반동 진입은 추세 진입용 `snapshot_for()` 전체 조건(EMA short/long, ADX no-trade gate)을 요구하지 않는다. 가격 반등 조건을 먼저 판단하고, RSI는 계산 가능할 때만 `rebound_rsi_min` 필터로 사용한다. 이렇게 해야 Toss 1분봉 미리보기처럼 장중 데이터가 짧거나 EMA60/ADX가 준비되지 않은 구간에서도 강한 반등 후보를 확인할 수 있다.
- 급반등 단독 진입(`rapid_rebound_enabled`)은 기존 장중 반동과 별도 옵션이다. 최근 `rapid_rebound_lookback_ticks` 관측치 안에서 선행 고점 대비 저점 하락률(`rapid_rebound_drop_pct`)과 저점 대비 현재가 회복률(`rapid_rebound_recovery_pct`)을 보고, 저점 후 `rapid_rebound_max_low_age_ticks` 안에 회복했을 때만 진입한다. 이 경로도 EMA/ADX 추세 조건을 요구하지 않으며, 실시간 `on_tick()`과 `preview_signals()`가 같은 `rapid_rebound_entry_ok()` helper를 사용해야 한다.
- 장중 반동/급반등 관측 버퍼는 `rebound_price_cap()`에서 두 옵션의 필요 길이 중 큰 값을 `bounded_window_with_extra()`로 감싼다. user-param을 그대로 `VecDeque` capacity로 쓰지 않는다.
- Toss 실행 scope에서 자동매매를 시작하면 Toss `1d` candles로 일봉 OHLC를 초기화하고, Toss `1m` candles의 OHLC를 레버리지 전략 장중 상태에도 주입한다. 실시간 현재가 polling은 같은 분 안에서는 마지막 1분봉과 반동 관측값을 갱신하고, 분이 바뀔 때만 새 관측치를 추가한다. 공개 데이터와 Toss 데이터가 섞이지 않게 strategy preview/진단도 가능하면 Toss candles 경로를 우선 사용한다.
- 레버리지 미리보기 입력은 `interval=1m|1d`와 `count=20..200`을 검증한다. 1분봉은 replay 첫 거래일보다 엄격히 이전인 완료 일봉만 지표 warmup으로 사용한다. 일봉은 표시 구간 이전 일봉만 warmup에 사용하며, 각 표시 일자는 세션 진입 시각의 시가-only 관측과 장 종료 시각의 완성 OHLC 관측으로 나눠 당일 고가·저가·종가 look-ahead를 막는다. 미국 정규장처럼 자정을 넘는 세션의 종료 시각은 다음 KST 날짜로 기록하고, signal 원본 시각은 상세 표시용으로 보존하되 별도 `chartTime` 거래일 키로 일봉 marker를 정렬한다.
- 레버리지 전략 청산은 초기 손절, 반등 실패 손절, 수익 보호 청산을 분리한다. `initial_stop_loss_pct`는 보호 활성 전에도 진입가 대비 손실을 즉시 제한한다. 반등 실패 손절은 `entry_failure_observations`와 `min_hold_observations`를 모두 지난 뒤에도 고점 수익률이 `trailing_activation_profit_pct`에 닿지 못한 채 진입가 아래일 때만 발생한다. 이렇게 해야 매수 직후 작은 음수 흔들림이 “실패”로 과도하게 해석되지 않는다. 이후 고점 수익률이 `trailing_activation_profit_pct` 이상일 때만 본전 보호/추적손절/추세 이탈 청산을 검사한다. 보호 활성 후 `breakeven_buffer_pct` 이하로 내려오면 본전 보호 청산, 고점 대비 `trailing_stop_pct` 이상 밀리면 수익 보호 추적손절, EMA/RSI 추세 이탈은 현재 수익률이 버퍼보다 높을 때만 청산한다. 미리보기와 실시간 `on_tick()`은 같은 초기 손절/보호 청산 helper를 사용해야 한다.
- `RiskManager`의 전략/종목별 일일 매수 제한은 재진입 전략을 막을 수 있어 차단 조건으로 사용하지 않는다. 하위 호환 필드는 남기되 view/update 경로는 0으로 노출·저장하고, 매도 일일 제한과 연속 손실 차단은 별도 방어로 유지한다.

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

### Tauri window-state 플러그인 패턴

앱 창의 크기, 위치, maximize 상태는 직접 JSON 저장소를 만들지 말고 `tauri-plugin-window-state`를 사용한다.

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_window_state::Builder::default().build())
```

- `src-tauri/tauri.conf.json`의 main window는 `visible:false`로 두어 복원 전 기본 크기 창이 보이는 flash를 줄인다.
- WebView 내부 패널 크기나 Log 높이 같은 레이아웃 상태는 이 플러그인 대상이 아니므로 React의 `src/shared/lib/persistentLayout.ts` localStorage helper로 저장한다.
- window close 시 자동매매 정지용 `on_window_event`는 그대로 유지한다.

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

> 마지막 업데이트: 2026-07-03T17:17:37+09:00

---

## 11. 전략 신호 공통 TradeGuard 패턴

전략별 `on_tick()`에서 반복 매매 방어를 각각 구현하면 전략마다 기준이 달라지고 누락이 생긴다.
전략 신호와 주문 실행 사이의 `OrderManager::submit_signal()`에서 공통 `TradeGuard`를 먼저 통과시킨다.

### 구현 위치

| 역할 | 위치 |
|------|------|
| guard 상태/판단 | `src-tauri/src/trading/guard.rs` |
| 주문 전 평가 | `src-tauri/src/trading/order.rs::submit_signal()`, `src-tauri/src/trading/order/submission.rs::OrderManager::submit_signal_shared()` |
| 실행 scope 고정 | `src-tauri/src/trading/order.rs::OrderManager::set_execution_scope()` |
| 일별 초기화 | `OrderManager::reset_day()` |

### 검사 항목

1. 동일 종목/동일 방향 쿨다운
2. 매도 후 재매수 쿨다운
3. 손절 후 당일 재진입 금지
4. 최근 반대 방향 신호 반복 시 종목 단위 휩소 쿨다운
5. 익절 매도에서 수수료·세금·슬리피지 차감 후 기대 순익 확인
6. `RiskManager`의 broker/account scope + 전략 ID + 종목 + 방향 + 날짜별 일일 주문 접수 횟수 제한
7. `RiskManager`의 broker/account scope + 전략 ID + 종목별 연속 손실 신규 진입 차단

```rust
let broker_scope = self.execution_scope.clone();

if let Some(reason) =
    risk.daily_order_limit_reason_for_scope(&broker_scope, strategy_id, symbol, side)
{
    tracing::info!("리스크 주문 횟수 제한 — {}", reason);
    return Ok(());
}

match self.trade_guard.evaluate_for_scope(
    &broker_scope,
    &signal,
    held_qty,
    avg_price,
    tick_price,
    exchange.is_some(),
) {
    GuardDecision::Allow => {}
    GuardDecision::Block { reason } => {
        tracing::info!("TradeGuard 차단 — {}", reason);
        return Ok(());
    }
}
```

`TradeGuard::evaluate()`와 `RiskManager::daily_order_limit_reason()` 같은 scope 없는 메서드는 하위 호환 래퍼로만 사용한다.
자동매매 주문 경로에서는 반드시 `BrokerScope`를 받는 `*_for_scope()` 메서드를 호출해야 한다. 그렇지 않으면 KIS 계좌와 Toss `accountSeq`, 또는 같은 broker의 서로 다른 계좌 사이에 쿨다운·일일 주문 횟수·연속 손실 차단 상태가 섞인다.

주문 횟수 카운터는 신호 발생 시점이 아니라 KIS가 주문을 접수해 pending에 등록된 뒤 증가시킨다. 기본값은 전략/종목별 매수 1회, 매도 1회이며 0은 제한 없음으로 취급한다.

연속 손실 차단은 매도 체결로 PnL이 확정된 뒤 `record_strategy_symbol_pnl_for_scope()`에서 갱신한다. 손실이면 카운터를 증가시키고, 수익이면 해당 scope/전략/종목의 카운터와 차단 상태를 해제한다. 포지션 청산을 막지 않기 위해 차단은 신규 `Buy` 신호에만 적용한다.

> 마지막 업데이트: 2026-07-03T13:27:56

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
자동매매 시작 전에 활성 broker의 실제 잔고를 읽고 `Strategy::sync_position_for_broker()` 훅으로 전략별 상태를 복원한다.

```rust
pub struct BrokerPositionSnapshot {
    pub broker_id: BrokerId,
    pub market: BrokerMarket,
    pub symbol: String,
    pub quantity: u64,
    pub avg_price: u64,
}

pub trait Strategy: Send + Sync {
    fn sync_position(&mut self, _symbol: &str, _quantity: u64, _avg_price: u64) {}
    fn sync_position_for_broker(&mut self, snapshot: &BrokerPositionSnapshot) {
        self.sync_position(&snapshot.symbol, snapshot.quantity, snapshot.avg_price);
    }
}
```

- KIS 국내: `get_balance()` → `PositionTracker::load_if_empty()` + `StrategyManager::sync_position_for_broker()`
- KIS 해외: `get_overseas_balance()` → `OverseasPositionTracker::load_if_empty()` + `StrategyManager::sync_position_for_broker()`
- Toss: `TossBrokerAdapter::list_holdings()` → market/currency 기준 국내/해외 tracker 분리 복원 + `StrategyManager::sync_position_for_broker()`
- 해외 또는 USD 평균단가/현재가는 USD × 100(cents)로 변환해서 전략에 전달한다.
- 해외 잔고와 체결은 국내 `PositionTracker`에 혼입하지 않는다.
- Toss decimal 수량은 전략의 in-position 복원 목적상 양수면 최소 1 단위로 snapshot에 반영한다. 실제 주문 수량으로 쓰기 전에는 Toss 주문 adapter에서 decimal 지원 여부를 별도 검증해야 한다.
- Toss 자동매매 실행은 주문/체결 adapter가 연결되어 있으므로, `start_trading()`에서 활성 Toss 프로파일 설정과 `live_trading_consent`를 확인한 뒤 holdings 기반 전략 상태 복원을 수행한다.

> 마지막 업데이트: 2026-07-03T15:25:46

---

## 13. 레버리지 단일 티커 추세 전략 패턴

레버리지 전략은 롱/숏 방향을 별도 모델로 해석하지 않는다. 사용자가 선택한 ETF 자체가 상승 추세이면 매수하고, 해당 ETF 자체의 상승 추세가 훼손되면 청산한다. SOXL 같은 롱 레버리지와 SOXS 같은 숏 레버리지 모두 같은 규칙으로 처리한다.

```rust
pub struct LeveragedTrendHoldEntry {
    pub leveraged_symbol: String,          // 실행 대상 ticker. 예: SOXL 또는 SOXS
    pub quantity: u64,
    // inverse_* / base_* 필드는 기존 저장 JSON 호환용 legacy 필드다.
}
```

- `target_symbols`에는 `leveraged_symbol`만 포함한다. 기존 저장 JSON에 `inverse_leveraged_symbol`이나 `base_symbols`가 있어도 폴링 대상에 넣지 않는다.
- 진입 조건은 대상 ETF 자체의 OHLC로 판단한다. 현재가 > EMA20, EMA20 > EMA60, RSI 상단 기준 이상, ADX 기준 이상, 최근 3봉 중 2개 이상 양봉.
- 청산 조건도 대상 ETF 자체의 OHLC로 판단한다. 고점 대비 trailing stop, 현재가 EMA20 하향 이탈, EMA20 < EMA60, RSI 약화, 장마감 청산.
- `upward_sensitivity`는 1.0~5.0 범위로 관리한다. 기본값 1.0은 기존 RSI 진입 기준을 유지하고, 값이 높을수록 진입 RSI 기준을 완화해 더 이른 신호를 허용한다. `downward_sensitivity`는 legacy 저장값 호환 필드로 남기되 새 UI에는 노출하지 않는다.
- 전략 상태는 `states: HashMap<String, ...>`와 `positions: HashMap<String, ...>`에 ticker별로 독립 저장한다.
- 설정창 미리보기는 `LeveragedTrendHoldStrategy::preview_signals_with_execution()`을 사용한다. 활성 Toss 프로파일의 `1m`/`1d` candles를 과거 시각 기준으로 replay하고 raw 신호마다 simulated 체결/차단 상태를 되먹임해 다음 신호를 평가한다. 실제 주문은 만들지 않으며 비용·리스크 backtest와 재현 메타데이터를 함께 반환한다.

> 마지막 업데이트: 2026-07-07T17:35:00+09:00

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

KIS 체결 조회 API로 주문번호를 확인할 때 `tot_ccld_qty`는 주문의 누적 체결 수량으로 취급한다. `src-tauri/src/trading/order/fills.rs`의 `OrderManager::on_fill()`은 누적 수량과 `PendingOrder.filled_quantity`의 차이만 포지션, 수수료, 거래 기록에 반영해야 한다.

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

주문 제출 전에는 `OrderManager`가 현재 실행 `BrokerScope` + symbol 기준으로 로컬 pending을 scan한다. 같은 방향 pending은 중복 주문으로, 반대 방향 pending은 반대 미체결 충돌로 분리해 차단 사유를 남긴다. 로컬 pending 충돌 helper는 `src-tauri/src/trading/order/conflicts.rs`에 둔다. Toss 주문 adapter를 연결할 때 provider의 `opposite-pending-order-exists` 응답도 같은 conflict 계열로 매핑한다.

> 마지막 업데이트: 2026-07-04T13:13:49+09:00

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

## 18. Provider 주문/체결 trace 기록 패턴

provider 문의와 장애 분석에 필요한 원본 식별자는 주문 제출 시점에 캡처해 체결 기록까지 전파한다.

| 필드 | 저장 위치 | 의미 |
|------|-----------|------|
| `provider` | `OrderRecord`, `TradeRecord` | `kis`, `toss` 등 원천 provider |
| `provider_order_id` | `OrderRecord`, `TradeRecord` | KIS `odno`, Toss order id |
| `provider_request_id` | `OrderRecord`, `TradeRecord` | Toss `requestId`, `X-Request-Id` 등 요청 추적 ID |
| `provider_tr_id` | `OrderRecord`, `TradeRecord` | KIS 주문/조회 TR-ID |

- 새 필드는 기존 JSON 하위 호환을 위해 `#[serde(default)]` optional 필드로 추가한다.
- KIS 주문은 `OrderResponse.tr_id`와 `odno`를 `OrderRecord::with_provider_trace()`에 넣고, `on_fill()`에서 `TradeRecord::with_provider_trace()`로 복사한다.
- 로그에는 `provider=kis tr_id=... odno=...`처럼 token 형태로 남겨 Log UI가 trace chip으로 감지할 수 있게 한다.

> 마지막 업데이트: 2026-07-03T16:35:00

---

## 19. Broker rate-limit scheduler 패턴

provider API 호출 간격과 429 backoff는 `src-tauri/src/broker/rate_limit.rs`의 `RateLimitScheduler`로 모은다.

| provider | group | 적용 범위 |
|----------|-------|-----------|
| Toss | `toss:auth` | `POST /oauth2/token` |
| Toss | `toss:account` | accounts, holdings, orders, buying-power, sellable-quantity, commissions |
| Toss | `toss:market` | prices, orderbook, trades, candles, stocks, warnings, market-calendar, exchange-rate |
| KIS | `kis:order` | 국내/해외 주문 제출 |

- 새 provider client는 요청 전 `wait(group)`을 호출하고, 응답 헤더를 받은 뒤 `apply_response_headers(group, headers)`를 호출한다.
- `Retry-After`, `X-RateLimit-Remaining=0`, `X-RateLimit-Reset`은 scheduler pause로 반영한다.
- 개별 호출부마다 임의 `sleep()`을 흩뿌리지 말고 group key와 scheduler 기본 간격을 조정한다.

> 마지막 업데이트: 2026-07-03T16:35:00

---

## 20. Broker-aware 프로파일/AppState 패턴

다중 증권사 확장 시 KIS 계좌번호와 Toss `accountSeq`를 같은 의미로 취급하지 않는다.

구현 위치:

| 역할 | 위치 |
|------|------|
| 공통 broker 타입 | `src-tauri/src/broker/domain.rs` |
| broker adapter trait | `src-tauri/src/broker/adapter.rs` |
| Toss read-only client/adapter | `src-tauri/src/broker/toss/{mod,adapter,client,http,error,support,types,orders}.rs` |
| 프로파일 저장 broker scope | `src-tauri/src/config/mod.rs::AccountProfile` |
| 활성 broker/account IPC 표시 | `src-tauri/src/commands.rs::AppConfigView`, `ProfileView` |
| 프로파일/잔고 IPC | `src-tauri/src/commands/accounts.rs` |
| 설정/환율 IPC | `src-tauri/src/commands/settings.rs` |
| KIS 시세/차트/종목검색 IPC | `src-tauri/src/commands/market.rs` |
| 수동 주문 IPC | `src-tauri/src/commands/orders.rs` |
| 체결/통계/로그 IPC | `src-tauri/src/commands/records.rs` |
| trade archive IPC | `src-tauri/src/commands/archive.rs` |
| Toss 진단/preflight IPC | `src-tauri/src/commands/toss.rs` |
| Toss read-only 시세/장운영 IPC | `src-tauri/src/commands/toss_market.rs` |
| KIS market/order REST 핸들러 | `src-tauri/src/server/market.rs` |
| 기록/로그/보관 REST 핸들러 | `src-tauri/src/server/records.rs` |
| 프로파일 REST 핸들러 | `src-tauri/src/server/profiles.rs` |
| Toss read-only REST 핸들러 | `src-tauri/src/server/toss.rs` |
| 자동매매/전략 REST 핸들러 | `src-tauri/src/server/trading.rs` |
| 자동매매 실행 scope 고정 | `src-tauri/src/commands/trading.rs::TradingStatus` |
| 자동매매 daemon/status/sync IPC | `src-tauri/src/commands/trading.rs` |
| 자동매매 시작 히스토리/ATR 초기화 | `src-tauri/src/commands/trading/history.rs` |
| 포지션/전략 IPC | `src-tauri/src/commands/strategy.rs` |
| 리스크/pending 주문 IPC | `src-tauri/src/commands/risk.rs` |
| 전략 facade | `src-tauri/src/trading/strategy.rs` |
| 전략 core/manager/state | `src-tauri/src/trading/strategy/{core,manager,state}.rs` |
| 개별 전략 구현 | `src-tauri/src/trading/strategy/{classic,breakout,mean_trend,leveraged_trend_hold,price_condition}.rs` |
| 전략 설정 저장 scope | `src-tauri/src/trading/strategy/core.rs::StrategyConfig` |
| 전략 view builder | `src-tauri/src/trading/views.rs::build_strategy_view()` |
| 시작 전 broker-aware 포지션 복원 | `src-tauri/src/commands/trading.rs::sync_strategy_positions_from_active_broker()` |
| 주문 전 broker read-only 검증 | `src-tauri/src/trading/preflight.rs`, `src-tauri/src/commands.rs::check_toss_order_preflight()` |
| 자동매매 주문 scope | `src-tauri/src/trading/order.rs::OrderManager::execution_scope` |
| 로컬 pending 충돌 차단 | `src-tauri/src/trading/order/conflicts.rs::pending_conflict_reason_for_scope()` |
| 주문 방어 scope | `src-tauri/src/trading/guard.rs`, `src-tauri/src/trading/risk.rs` |

규칙:

- 기존 `profiles.json` 하위 호환을 위해 `AccountProfile.broker_id`에는 `#[serde(default = "default_broker_id")]`를 둔다.
- 자동매매 시작 시점에 `trading_profile_id`, `trading_broker_id`, `trading_account_id`를 모두 저장한다.
- `StrategyConfig`에는 `broker_id`와 `broker_account_id`를 저장하고, `StrategyManager::apply_saved_configs_for_scope()`와 `update_strategy` 경로에서 현재 활성 broker/account scope로 stamp한다.
- 전략 파라미터 또는 대상 종목이 바뀌면 `StrategyManager::update_config()`로 config를 수정하고 `build_strategy()`로 인스턴스를 재빌드한다. 개별 전략은 생성자에서 params를 파싱하므로 config map만 수정하면 런타임 전략이 낡은 값을 계속 사용할 수 있다.
- `StrategyManager::apply_saved_configs_for_scope()`는 저장 config를 적용한 뒤 전략 인스턴스를 재빌드한다. 프로파일/account scope 전환 시 이전 종목의 per-symbol state가 잔류하지 않게 하기 위해서다.
- tick hot path에서 대상 종목 여부를 확인할 때 `target_symbols.contains(&symbol.to_string())`처럼 매번 할당하지 말고 `StrategyConfig::targets_symbol(symbol)`을 사용한다.
- user-param이 직접 `VecDeque` capacity 또는 per-symbol buffer 길이가 되지 않게 `src-tauri/src/trading/strategy/state.rs`의 bounded helper를 사용한다. 비정상 저장 JSON이 있어도 전략별 버퍼가 상한 없이 커지면 안 된다.
- IPC/REST 전략 목록 응답은 `src-tauri/src/trading/views.rs::build_strategy_view()`를 사용한다. `StrategyManager` lock은 config clone까지만 잡고, 종목명 조회 같은 async lookup은 lock 밖에서 수행한다.
- IPC/REST 프로파일 목록 응답은 `src-tauri/src/commands/accounts.rs::profile_to_view()`를 사용해 masking과 broker/account view drift를 막는다.
- IPC/REST risk view, pending order view, archive stats는 `src-tauri/src/commands/{risk,archive}.rs`의 shared builder/service를 재사용한다. REST handler에서 같은 JSON shape를 직접 hand-build하지 않는다.
- 프로파일 전환 시 저장 전략이 없더라도 `apply_saved_configs_for_scope(&[], ...)`를 호출해 이전 프로파일의 활성 전략/종목이 잔류하지 않게 reset한다.
- 자동매매 실행 중 활성 프로파일을 전환해도 실행 중 주문 경로는 시작 시점 scope를 유지한다.
- `OrderManager::set_execution_scope(BrokerScope)`는 `start_trading()`에서만 설정하고, 주문 전 `TradeGuard`/`RiskManager`/`PendingOrder`에 같은 scope를 전달한다.
- `BrokerScope`는 `broker_id`와 `Option<BrokerAccountId>`를 함께 들고, KIS 계좌번호와 Toss `accountSeq`가 같은 키 공간을 공유하지 않게 한다.
- 아직 실제 주문/체결 adapter가 없는 broker는 자동매매 시작 시 `BROKER_NOT_SUPPORTED`로 차단한다. 현재 Toss는 주문/체결 adapter가 연결되어 있으므로 `live_trading_consent`와 accountSeq 설정을 통과하면 `start_trading()`을 허용한다.
- 웹 REST `/api/trading/start`도 IPC `start_trading`과 같은 broker gate를 유지해야 한다. `is_trading`만 true로 바꾸지 말고 KIS 설정 검증, 미지원 broker 차단, `OrderManager::set_execution_scope()` 설정, 시작 전 KIS 잔고 기반 전략 포지션 복원을 수행한다.
- 웹 REST `/api/archive-config` 변경도 IPC `set_trade_archive_config`와 같이 파일 저장 후 `purge_old_trade_files()`를 즉시 예약한다. IPC/REST 관리 경로가 다른 보관 정책을 가지면 장기 운영 중 디스크 사용량 예측이 어긋난다.
- KIS REST client는 주문뿐 아니라 read API도 `RateLimitScheduler`를 거친다. 계좌/잔고는 `kis:account`, 체결 조회는 `kis:execution`, 시세/차트는 `kis:quote`, 주문은 `kis:order` group을 사용하고 응답 rate-limit header를 group별로 반영한다.
- Toss client는 token 발급/401 1회 재시도/accounts/holdings/buying-power/sellable-quantity/commissions 및 주문 생성/조회 API를 제공한다. HTTP timeout과 response body 상한을 두고, `Content-Length`가 없거나 부정확한 응답도 chunk 누적 중 상한 초과 전에 중단한다. 파싱/에러 메시지에는 전체 body를 싣지 말고 snippet만 포함한다. token cache mutex를 잡은 채 네트워크 token 발급을 기다리지 않는다. holdings는 `BrokerHolding`으로 매핑하며 자동매매 시작 전 전략 포지션 복원에 사용한다.
- Toss 주문 client surface는 `create_order`, `list_orders`, `get_order`, `modify_order`, `cancel_order`로 둔다. `TossOrderCreateRequest::with_generated_client_order_id()`는 공식 idempotency key 제약을 만족하는 `clientOrderId`를 생성한다. 소액 검증 IPC(`submit_toss_small_buy_verification`)는 실거래 동의, 최종 확인, 최대 허용 주문금액, 직전 preflight, open-order scan 후 1주 시장가 매수만 제출한다. Dashboard는 수동거래 페이지로 안내만 표시하고 소액매매 검증 UI를 두지 않는다. Trading 수동 주문은 `place_order`에서 활성 Toss 프로파일이면 live consent/preflight/local pending/provider open-order scan 후 Toss order API로 분기한다. 자동매매 주문은 `OrderManager::submit_signal_shared()`에서 시작 시점 `BrokerScope`와 일치하는 Toss profile credential을 찾아 제출하고, `confirm_pending_fills_from_broker_shared()`는 Toss `get_order` detail의 누적 체결수량/평균가를 `on_fill()`로 반영한다. Strategy/자동매매 화면에는 소액매매 검증 UI를 두지 않는다.
- Toss 모듈을 분리할 때 내부 DTO/validation/helper는 `pub(super)`로 제한하고, 외부에서 필요한 client/adapter와 공식 응답 타입만 `broker/toss/mod.rs`에서 re-export한다.
- `src-tauri/src/trading/order/fills.rs::confirm_pending_fills_from_broker()`는 pending `OrderRecord.provider` trace로 provider를 판정한다. 자동매매 daemon은 `OrderManager::confirm_pending_fills_from_broker_shared()`를 호출해 KIS/Toss 체결 조회 네트워크 await를 `order_manager` mutex 밖에서 수행한다.
- 자동매매 daemon의 전략 신호 주문 제출은 `src-tauri/src/trading/order/submission.rs`의 `OrderManager::submit_signal_shared()`를 사용한다. 이 경로는 `order_manager` mutex를 짧게 잡아 guard/pending/submitting 예약만 수행하고, provider 주문 API와 order store append는 mutex 밖에서 await한다. provider 호출 중 같은 broker scope/symbol의 중복/반대 주문은 `submitting` 예약 맵으로 차단한다.
- 주문 전 금액/수량 판정은 `trading/preflight.rs`의 공통 함수에 모은다. Toss `commissionRate`는 percent 문자열로 해석하고, `BrokerMoney`/`BrokerQuantity` 문자열 precision은 응답 view까지 보존한다.
- 실제 주문 제출 전에는 로컬 pending scan을 먼저 수행해 같은 scope/symbol의 같은 방향 중복 주문과 반대 방향 미체결 주문을 모두 차단한다.
- 수동/자동 및 IPC/REST 주문은 `OrderManager::{submit_manual_order_shared,submit_signal_shared}`가 공유하는 scoped order service를 거친다. provider 응답을 받은 뒤 pending snapshot 저장에 실패하면 신규 매수를 중단한다.
- pending 주문은 `storage::PendingOrderStore`에 broker/account scope, provider/client order ID, 누적 체결 watermark, provider 상태와 함께 저장한다. 자정/stop에서 지우지 않고 앱 시작 및 자동매매 시작 장벽에서 broker 체결/detail과 대조한다.
- 체결 적용은 scope가 포함된 deterministic fill event ID로 OrderStore/TradeStore/StatsStore를 idempotent하게 갱신한다. provider 누적 평균가는 이전 체결 notional을 빼 delta 체결가로 변환한다. 자동매매 시작 장벽은 오늘 TradeStore의 `realized_pnl_krw`를 RiskManager에 재생해 crash 중간 지점과 무관하게 손실 한도를 복원하며, `TradeRecord::matches_scope`로 실행 scope 체결만 반영한다(스코프 미기록 레거시 레코드는 KIS로 간주).
- `OrderRecord`/`TradeRecord`는 `with_broker_scope()`로 broker/account를 기록한다. PositionTracker/OverseasPositionTracker는 단일 활성 scope 스냅샷만 보유한다 — 프로파일 전환(비실행 중) 시 `clear()`하고 다음 holdings `replace()` 또는 수동 주문 직전 refresh로 재동기화하며, 실행 중에는 `execution_scope`가 주문 흐름을 시작 시점 scope에 고정한다.
- 리스크 설정과 일별 runtime 상태는 `storage::RiskStore`가 `risk/config.json`(설정)과 `risk/runtime.json`(날짜·손익·비상정지·주문 횟수·연속 손실 차단)으로 분리 저장한다. 주문 접수·체결 반영·비상정지 변경·일별 초기화 시점마다 스냅샷을 저장하고, OrderManager 경로의 저장 실패는 fail-closed로 신규 주문을 차단한다. 시작 시 복원은 스냅샷 날짜가 오늘일 때만 카운터를 적용하고 비상정지는 날짜와 무관하게 유지한다(수동 해제 대상). runtime 파일을 읽지 못하면 비상정지 상태로 시작한다.
- 계좌 잔고·holdings 조회는 fail-closed다. 시작 전 하나라도 실패하거나 총평가액이 0 이하이면 시작하지 않고, 운용 중 갱신 실패는 `buy_suspended_reason`과 Discord에 노출한 뒤 신규 매수를 막는다.
- Toss 연결 진단은 `check_toss_profile_connection` IPC에서 프로파일 lock을 짧게 잡아 clone한 뒤 실행한다. 진단 단계는 OpenAPI spec, token, accounts, holdings, order preflight read-only 순서이며 토큰 문자열은 응답에 포함하지 않는다.
- broker rate limiter는 `broker/rate_limit.rs`의 `shared_scheduler(scope, init)`로 credential scope별 process-wide 공유한다 (Toss `toss|base_url|client_id`, KIS `kis|base_url|app_key|paper=...`). 짧게 생성되는 client나 프로파일 전환에도 pacing/429 pause/운영 상태가 유지된다. 요청 결과는 `record_outcome`으로 기록하고, `get_broker_rate_limit_status` IPC와 Settings "Broker 요청 상태" 섹션이 scope/그룹별 pause 잔여·rate limit 누적·마지막 성공/실패·연속 실패를 노출한다(scope 레이블의 credential은 마스킹).
- 모든 broker HTTP 경로에 timeout과 응답 크기 상한을 둔다: KIS connect 10s/전체 30s/8MB(`read_kis_response_text`), Toss 15s/4MB, KIS token 15s, detect/exchange 10s. 새 HTTP 호출을 추가할 때 `Client::new()`를 그대로 쓰지 않는다.
- Tauri IPC와 axum 웹 REST 응답 필드는 같이 갱신한다. 웹 핸들러에서 내부 struct를 그대로 직렬화하지 말고 `serde_json::json!`으로 camel/snake 응답 키를 명시한다.

## 21. 전략 deterministic replay / 실행 피드백 패턴

- `src-tauri/src/trading/simulation.rs`의 replay 엔진은 raw signal과 `filled`/`blocked` 실행 결과를 분리한다. 성과 지표에는 TradeGuard, RiskManager, 현금·보유 수량·포지션 비중과 수수료/세금/슬리피지/환율을 통과한 체결 가정만 반영한다.
- 과거 시각 replay는 `TradeGuard::{evaluate_for_scope_at,record_submitted_for_scope_at}`을 사용한다. 실시간 wrapper 내부의 `Local::now()`를 test clock처럼 바꾸지 않는다.
- live 시작과 preview warmup은 `strategy::initialize_strategy_warmup()`을 공유한다. 일봉과 장중 봉을 별도 인자로 전달하고 replay 시작 이후 봉을 warmup에 넣지 않는다.
- 일봉 replay는 정보 공개 시점을 지킨다. 장 시작 event는 시가-only OHLC, 장 종료 event만 완성 OHLC를 사용한다. 입력은 최대 500봉으로 제한하고 결과에는 engine/strategy version, source/interval/range, warmup count, input hash를 남긴다.
- 전략이 raw signal 생성 시 내부 `in_position`을 선반영했는데 주문이 skip되면 `SubmissionOutcome::Skipped`의 실제 `held_quantity`/`avg_price`를 `StrategyManager::sync_position()`에 되먹임한다. 수량 0 snapshot도 flat 상태로 반영해야 한다.
- 자동매매 시작 플래그 lock은 broker reconcile/risk restore/warmup이 완료될 때까지 daemon 첫 tick을 막아야 한다.

> 마지막 업데이트: 2026-07-15T00:00:00+09:00

---

## PostgreSQL/MariaDB 문서 저장 backend

- 거래·주문·통계·전략처럼 DB 전환 대상인 store는 `storage::read_json_or_default()`와 `storage::write_json()`만 사용한다. backend가 DB인데 실패하면 JSON fallback/dual-write로 성공을 가장하지 않고 오류를 반환한다.
- JSON 상대경로가 DB document key의 단일 원천이다. key는 `data_dir` 밖 경로, `..`, 절대경로, Windows prefix를 거부하고 import/export의 파일 수·개별 크기·전체 크기에 상한을 둔다.
- PostgreSQL/MariaDB 차이는 `storage/database.rs`의 pool/query 경계에서만 처리한다. 사용자 입력 table name이나 임의 SQL은 실행하지 않으며 앱 소유 allowlist 테이블만 관리한다.
- JSON → DB import는 한 transaction에서 전체 upsert한다. DB backend 활성화 전 현재 JSON key가 모두 존재하는지 검증하고, JSON backend 복귀 전 DB 문서를 live JSON 경로에 atomic restore한다.
- DB 설정 변경, schema mutation, import, backend 전환은 자동매매 중 거부한다. clear/drop은 별도 확인 문구를 요구하고 DB backend 활성 중에는 실행하지 않는다.
- DB password는 IPC view와 logical export에 포함하지 않는다. DB 관리 명령은 인증되지 않은 axum REST에 추가하지 않고 Tauri desktop command로만 제공한다.
- DB password는 `storage/database_keychain.rs`를 통해 OS keychain(macOS Keychain/Windows Credential Manager)에 저장하고 `database_config.json`에는 남기지 않는다. 파일에 남은 레거시 password는 `load_sync`가 시작 시 1회 keychain으로 이전하며, keychain을 쓸 수 없는 환경에서만 파일(0o600) 저장으로 fallback한다. 테스트는 `use_mock_keychain_for_tests()` + `keychain_test_lock()`으로 mock store를 직렬 사용한다(mock은 Entry 인스턴스별 독립 상태라 handle을 process-wide로 재사용해야 한다).
- 실서버 contract test는 `storage/database_contract_tests.rs`에 있다. `KISAT_PG_HOST/PORT/USER/PASSWORD` env가 없으면 skip하고, 있으면 매 실행 고유한 `kisautotrade_ct_*` database를 자동 생성 경로로 만들어 create/import/export/backend 전환/clear/drop 왕복을 검증한 뒤 DROP한다. 운영 database는 절대 사용하지 않는다.
- v1 schema는 기존 JSON 복원성을 위한 document store다. 주문/체결 복구·검색·retention을 위한 정규화 schema는 명시적인 schema version migration과 PostgreSQL/MariaDB contract test를 함께 추가한다.
- JSON 파일 저장은 경로별 lock과 store별 read-modify-write lock을 사용하고, temp write → file fsync → 정상본 `.bak` → atomic rename → parent directory fsync 순서를 지킨다. 역직렬화 실패 시 손상본을 격리하고 마지막 정상 백업으로 복구한다.

> 마지막 업데이트: 2026-07-15T00:00:00+09:00
