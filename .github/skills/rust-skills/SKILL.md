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
- 레버리지 미리보기 입력은 `interval=1m|1d`와 `count=20..200`을 검증한다. v7/LTH v3에서 이전 완료 일봉은 별도 bounded context이며 분봉 EMA/RSI/ADX·반동·볼린저 warmup에 넣지 않는다. `1d`는 장중 가격·진입/청산을 합성하지 않는 `dailyDiagnostic/notEvaluable` 자료 진단으로 전략·체결을 실행하지 않는다. `1m`은 실제 관측의 `intradaySample/sampleOnly`이며 유효 분봉 0개는 `notEvaluable`이다. 고정 KST 세션 근사와 원본 시간대·DST·세션 미검증을 assessment에 명시하고 `lookAheadSafe=false`로 둔다. `warmupCount=0`과 `dailyContextBars`를 구분하며 최대 200봉 표본을 3개월 성과로 표시하지 않는다.
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
| MA교차(1) | `MaCrossState` | `prices`, `prev_short_ma`, `prev_long_ma`, 별도 완료 일봉 `context` |
| RSI(2) | 인라인 튜플 | `(VecDeque<u64>, Option<f64>)` |
| 모멘텀(3) | `MomentumState` | `prices`, `owned_quantity`, `previous_momentum`, `armed`, `retry_candidate` |
| 이격도(4) | `DeviationState` | 가격/추세 버퍼, 실제 수량·평균가, 보유 관측·교차 후보 |
| 52주신고가(5) | `FiftyTwoWeekState` | `prev_price`, `high_52w`, `buy_price` |
| 연속상승(6) | `ConsecutiveMoveState` | `prices`, `in_position` |
| 돌파실패(7) | `FailedBreakoutState` | `breakout_prev_high` |
| 강한종가(8) | `StrongCloseState` | `pending_buy`, `entry_price` |
| 변동성확장(9) | `VolatilityExpansionState` | `avg_range` (reset 후 유지) |
| 평균회귀(10) | `MeanReversionState` | `prices`(reset 후 유지), actual q/native entry, 보유 age/candidate, live trend |
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
        let strategy_manager = state.strategy_manager.clone();
        let order_manager = state.order_manager.clone();
        // sync 컨텍스트 → spawn으로 비동기 작업 예약
        tauri::async_runtime::spawn(async move {
            let mut guard = is_trading.lock().await;
            if *guard {
                *guard = false;
                let mut manager = strategy_manager.lock().await;
                manager.advance_history_generation();
                manager.clear_history_execution_identity();
                order_manager.lock().await.end_execution_run();
                tracing::info!("앱 종료 — 자동매매 정지 신호 전송");
            }
        });
        tracing::info!("앱 종료 요청");
    }
})
```

> ⚠️ `on_window_event` 콜백은 동기 컨텍스트. `tokio::sync::Mutex` 의 `.lock().await` 는 직접 호출 불가 → `spawn` 으로 예약.

잠금 순서는 bool→strategy→order다. 이 `spawn` 예약만으로 프로세스 종료 전에 정지 작업의 완료를 보장하지는 않는다.

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

전략 내부 보유는 확인된 주문 소유권으로 복원한다. 시작 시 broker 잔고는 계좌 tracker에 게시하고, 전략에는 `sync_strategy_ownership`으로 exact owner 수량만 전달한다. 계좌 총잔고나 수동 매수를 모든 전략의 `in_position` 근거로 사용하지 않는다. 아래 §12를 따른다.

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

## 12. 전략 소유권과 계좌 잔고 동기화 패턴

계좌 전체 관측 수량과 전략별 확정 보유는 별도 authority다. 시작 IPC·웹·상시 체결 데몬은 broker snapshot 게시와 `OrderManager::sync_strategy_ownership`을 사용한다.

- `PositionTracker`/`OverseasPositionTracker`는 검증된 broker snapshot으로 교체한다. 실제 fill에서 delta 가산/차감을 하지 않고 snapshot version을 무효화하고 해당 시장을 재조회한다. 조회 성공 전 신규 주문 예약을 보류한다.
- `order/ownership.rs`의 원장은 broker/account/profile/투자모드·owner·시장·종목별 수량과 정수 원가를 보관한다. 자동 매도는 exact owner와 계좌 관측 수량 중 작은 값으로 제한하고 예약 직전 다시 확인한다. 수동 매도는 자동 소유와 예산 원장의 보유를 제외한다.
- `StrategyManager::sync_owned_position_for_strategy`는 ID/scope/시장/종목을 일치시킨 대상만 갱신한다. 전체 계좌 `BrokerPositionSnapshot`을 모든 전략에 broadcast하지 않는다. legacy hook은 replay/호환 호출과 구분한다.
- live `on_live_tick`은 관측 지표를 갱신하되 raw 신호로 확정 보유를 바꾸지 않는다. pending/unsettled 상태는 외부 중복 신호만 억제한다. 체결·취소/거부 후 소유 수량과 pending을 다시 전달한다.
- LTH 최고가·보유 관측 수 및 신고가 실패의 진입 기준을 보호 metadata로 보존한다. 부분 체결·설정 변경으로 평균가와 동일한 새 최고가/관측0을 만들지 않는다. replay의 입력 시각/모의 체결 경로는 유지한다.
- manager → order lock 순서로 동기화한다. order mutex 아래에서 전략 manager를 await하지 않는다. live 관측 뒤 확정 보유 metadata를 checkpoint하며 오래된 run/revision은 저장하지 않는다.
- Toss 변환은 `HoldingsUpdate::from_toss` 하나로 IPC/웹 시작·수동·체결 후 refresh를 통일한다. broker/account/시장/통화·중복 종목·유한 범위를 검증한다. 정수 주문 체계에서 소수 수량은 floor하며 양수 최소1 올림을 하지 않는다. KRW 원/USD cents를 구분하고 `< 2^64`를 검사한다.
- 도입 이전/수동 보유를 새 전략에 임의 귀속하지 않는다. 소유 원장과 별도 generation manifest의 누락·손상·충돌, 원가 미상 체결은 신규 주문 차단 사유다. pending/소유 원장을 오래된 backup으로 자동 복원하지 않는다.

검증과 범위: `docs/tasks/strategy-fill-ownership-20261018.md`.

> 마지막 업데이트: 2026-10-18T14:21:43+09:00

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
- 설정창의 `1m` 미리보기는 `LeveragedTrendHoldStrategy::preview_signals_with_driver()`와 공통 `ReplaySession`을 사용한다. 활성 Toss 프로파일의 실제 분봉을 입력 시각 기준으로 replay하고 매 봉 시작의 대기 체결과 종가 신호 처리 뒤 실제 수량/원가를 되먹임한다. `1d`는 장중 엔진을 호출하지 않는 평가 불가 자료 진단이다. 실제 주문은 만들지 않으며 assessment의 기간·시각 한계를 함께 반환한다.

> 마지막 업데이트: 2026-10-05T23:54:16+09:00

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
해외 체결은 USD cents 단위의 owner 원장에 반영한다. 계좌 전체 해외 tracker는 broker 잔고 재조회로 교체하고 원화 통계에는 저장한 체결 환율로 환산한 값만 전달한다. 계좌 snapshot이 이미 포함한 체결을 다시 더하지 않는다.

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

KIS 체결 조회 API로 주문번호를 확인할 때 `tot_ccld_qty`는 주문의 누적 체결 수량으로 취급한다. `src-tauri/src/trading/order/fills.rs`의 `OrderManager::on_fill()`은 누적 수량과 `PendingOrder.filled_quantity`의 차이만 소유 원장, 수수료, 거래 기록에 반영해야 한다. 계좌 전체 tracker는 재조회하며 delta 변경하지 않는다.

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

> 마지막 업데이트: 2026-10-05T23:54:16+09:00

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
| 자동매매 status/lifecycle IPC facade | `src-tauri/src/commands/trading.rs` |
| 자동매매 daemon/tick/시세 source | `src-tauri/src/commands/trading/daemon.rs` |
| broker별 평가액 / 당일 리스크 복원 | `src-tauri/src/commands/trading/{balance,risk_restore}.rs` |
| 자동매매 시작 히스토리/ATR 초기화 | `src-tauri/src/commands/trading/history.rs` |
| 포지션/전략 IPC | `src-tauri/src/commands/strategy.rs` |
| 리스크/pending 주문 IPC | `src-tauri/src/commands/risk.rs` |
| 전략 facade | `src-tauri/src/trading/strategy.rs` |
| 전략 core/manager/state | `src-tauri/src/trading/strategy/{core,manager,state}.rs` |
| 개별 전략 구현 | `src-tauri/src/trading/strategy/{classic,breakout,mean_trend,leveraged_trend_hold,price_condition}.rs` |
| 전략 설정 저장 scope | `src-tauri/src/trading/strategy/core.rs::StrategyConfig` |
| 전략 view builder | `src-tauri/src/trading/views.rs::build_strategy_view()` |
| 시작 전 broker-aware 포지션 복원 | `src-tauri/src/commands/trading/positions.rs::sync_strategy_positions_from_active_broker()` |
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
- 웹 REST `/api/trading/start`도 IPC `start_trading`과 같은 broker gate를 유지해야 한다. `is_trading`만 true로 바꾸지 말고 KIS 설정 검증, 미지원 broker 차단, `OrderManager::set_execution_scope()` 설정, 시작 전 broker 잔고 게시와 exact owner 원장 기반 전략 보유 복원을 수행한다.
- 웹 REST `/api/archive-config` 변경도 IPC `set_trade_archive_config`와 같이 파일 저장 후 `purge_old_trade_files()`를 즉시 예약한다. IPC/REST 관리 경로가 다른 보관 정책을 가지면 장기 운영 중 디스크 사용량 예측이 어긋난다.
- KIS REST client는 주문뿐 아니라 read API도 `RateLimitScheduler`를 거친다. 계좌/잔고는 `kis:account`, 체결 조회는 `kis:execution`, 시세/차트는 `kis:quote`, 주문은 `kis:order` group을 사용하고 응답 rate-limit header를 group별로 반영한다.
- Toss client는 token 발급/401 1회 재시도/accounts/holdings/buying-power/sellable-quantity/commissions 및 주문 생성/조회 API를 제공한다. HTTP timeout과 response body 상한을 두고, `Content-Length`가 없거나 부정확한 응답도 chunk 누적 중 상한 초과 전에 중단한다. 파싱/에러 메시지에는 전체 body를 싣지 말고 snippet만 포함한다. token cache mutex를 잡은 채 네트워크 token 발급을 기다리지 않는다. holdings는 `BrokerHolding`으로 매핑하며 시작 전 계좌 tracker 게시에 사용한다. 전략 보유는 별도의 exact owner 원장에서 복원한다.
- Toss 주문 client surface는 `create_order`, `list_orders`, `get_order`, `modify_order`, `cancel_order`로 둔다. `TossOrderCreateRequest::with_generated_client_order_id()`는 공식 idempotency key 제약을 만족하는 `clientOrderId`를 생성한다. 소액 검증 IPC(`submit_toss_small_buy_verification`)는 실거래 동의, 최종 확인, 최대 허용 주문금액, 직전 preflight, open-order scan 후 1주 시장가 매수만 제출한다. Dashboard는 수동거래 페이지로 안내만 표시하고 소액매매 검증 UI를 두지 않는다. Trading 수동 주문은 `place_order`에서 활성 Toss 프로파일이면 live consent/preflight/local pending/provider open-order scan 후 Toss order API로 분기한다. 자동매매 주문은 `OrderManager::submit_signal_shared()`에서 시작 시점 `BrokerScope`와 일치하는 Toss profile credential을 찾아 제출하고, `confirm_pending_fills_from_broker_shared()`는 Toss `get_order` detail의 식별자·통화·정수 누적 수량과 명시 누적 금액을 검증해 `on_fill_confirmed()`로 반영한다. 평균가만으로 원가를 재구성하지 않는다. Strategy/자동매매 화면에는 소액매매 검증 UI를 두지 않는다.
- Toss 모듈을 분리할 때 내부 DTO/validation/helper는 `pub(super)`로 제한하고, 외부에서 필요한 client/adapter와 공식 응답 타입만 `broker/toss/mod.rs`에서 re-export한다.
- `src-tauri/src/trading/order/fills.rs::confirm_pending_fills_from_broker()`는 pending `OrderRecord.provider` trace로 provider를 판정한다. 자동매매 daemon은 `OrderManager::reconcile_strategy_state_shared()`로 체결 대조·owner 피드백·필요 잔고 재조회를 연결하고 내부 `confirm_pending_fills_from_broker_shared()`는 KIS/Toss 체결 조회 네트워크 await를 `order_manager` mutex 밖에서 수행한다.
- 자동매매 daemon의 전략 신호 주문 제출은 `src-tauri/src/trading/order/submission.rs`의 `OrderManager::submit_signal_shared()`를 사용한다. 이 경로는 `order_manager` mutex를 짧게 잡아 guard/pending/submitting 예약만 수행하고, provider 주문 API와 order store append는 mutex 밖에서 await한다. provider 호출 중 같은 broker scope/symbol의 중복/반대 주문은 `submitting` 예약 맵으로 차단한다.
- 주문 전 금액/수량 판정은 `trading/preflight.rs`의 공통 함수에 모은다. Toss `commissionRate`는 소수 비율(`0.00015` = `0.015%`) 문자열로 해석하고, `BrokerMoney`/`BrokerQuantity` 문자열 precision은 응답 view까지 보존한다.
- 실제 주문 제출 전에는 로컬 pending scan을 먼저 수행해 같은 scope/symbol의 같은 방향 중복 주문과 반대 방향 미체결 주문을 모두 차단한다.
- 수동/자동 및 IPC/REST 주문은 `OrderManager::{submit_manual_order_shared,submit_signal_shared}`가 공유하는 scoped order service를 거친다. provider 응답을 받은 뒤 pending snapshot 저장에 실패하면 신규 매수를 중단한다.
- pending 주문은 `storage::PendingOrderStore`에 broker/account scope, provider/client order ID, 누적 체결 watermark, provider 상태와 함께 저장한다. 자정/stop에서 지우지 않고 앱 시작 및 자동매매 시작 장벽에서 broker 체결/detail과 대조한다.
- 체결 적용은 scope가 포함된 deterministic fill event ID로 OrderStore/TradeStore/StatsStore를 idempotent하게 갱신한다. 명시 누적 체결대금을 이전 watermark와 비교해 delta 원가/체결대금을 구한다. KIS 해외/Toss 금액이 누락된 경우에만 실제 평균가 원문×수량을 총액에서 한번 반올림하며 명시0/오염을 주문가로 대체하지 않는다. 자동매매 시작 장벽은 오늘 TradeStore의 `realized_pnl_krw`를 RiskManager에 재생해 crash 중간 지점과 무관하게 손실 한도를 복원하며, `TradeRecord::matches_scope`와 저장된 KIS TR-ID의 정확한 모드를 확인하고 후보 runtime 저장 후 공개한다. 모드 미상 레코드는 fail-closed이며 같은 계좌의 paper/live 손익을 합산하지 않는다.
- `OrderRecord`/`TradeRecord`는 `with_broker_scope()`로 broker/account를 기록한다. PositionTracker/OverseasPositionTracker는 단일 활성 scope 스냅샷만 보유한다 — 프로파일 전환(비실행 중) 시 `clear()`하고 다음 holdings `replace()` 또는 수동 주문 직전 refresh로 재동기화하며, 실행 중에는 `execution_scope`가 주문 흐름을 시작 시점 scope에 고정한다.
- 리스크 설정과 일별 runtime 상태는 `storage::RiskStore`가 `risk/config.json`(설정)과 `risk/runtime.json`(날짜·손익·비상정지·주문 횟수·연속 손실 차단)으로 분리 저장한다. 주문 접수·체결 반영·비상정지 변경·일별 초기화 시점마다 스냅샷을 저장하고, OrderManager 경로의 저장 실패는 fail-closed로 신규 주문을 차단한다. 시작 시 복원은 스냅샷 날짜가 오늘일 때만 카운터를 적용하고 비상정지는 날짜와 무관하게 유지한다(수동 해제 대상). runtime 파일을 읽지 못하면 비상정지 상태로 시작한다.
- 계좌 잔고·holdings 조회는 fail-closed다. 시작 전 하나라도 실패하거나 총평가액이 0 이하이면 시작하지 않고, 운용 중 갱신 실패는 `buy_suspended_reason`과 Discord에 노출한 뒤 신규 매수를 막는다.
- Toss 연결 진단은 `check_toss_profile_connection` IPC에서 프로파일 lock을 짧게 잡아 clone한 뒤 실행한다. 진단 단계는 OpenAPI spec, token, accounts, holdings, order preflight read-only 순서이며 토큰 문자열은 응답에 포함하지 않는다.
- broker rate limiter는 `broker/rate_limit.rs`의 `shared_scheduler(scope, init)`로 credential scope별 process-wide 공유한다 (Toss `toss|base_url|client_id`, KIS `kis|base_url|app_key|paper=...`). 짧게 생성되는 client나 프로파일 전환에도 pacing/429 pause/운영 상태가 유지된다. 요청 결과는 `record_outcome`으로 기록하고, `get_broker_rate_limit_status` IPC와 Settings "Broker 요청 상태" 섹션이 scope/그룹별 pause 잔여·rate limit 누적·마지막 성공/실패·연속 실패를 노출한다(scope 레이블의 credential은 마스킹).
- 모든 broker HTTP 경로에 timeout과 응답 크기 상한을 둔다: KIS connect 10s/전체 30s/8MB(`read_kis_response_text`), Toss 15s/4MB, KIS token 15s, detect/exchange 10s. 새 HTTP 호출을 추가할 때 `Client::new()`를 그대로 쓰지 않는다.
- Tauri IPC와 axum 웹 REST 응답 필드는 같이 갱신한다. 웹 핸들러에서 내부 struct를 그대로 직렬화하지 말고 `serde_json::json!`으로 camel/snake 응답 키를 명시한다.

## 21. 전략 deterministic replay / 실행 피드백 패턴

- v8 replay ATR은 `simulation/atr.rs`의 bounded TR14 window다. 이전 close reference 1봉과 TR14개가 필요하므로 같은 interval 관측15봉부터 준비한다. `run_backtest_with_history`의 history는 caller가 검증한 엄격한 이전 같은 간격 자료만 전달하고 주문·equity에는 넣지 않는다. 갭은 인접 실제 관측의 이전 close로 반영하며 누락봉을 합성하지 않는다. u128 합계로 overflow를 막고 엄격한 준비 상태를 `atrPreparation`으로 전략 준비와 구분한다. live ATR의 부분-window/rounding 및 LTH 분봉 지표와 의미가 달라 helper를 무리하게 통합하지 않는다.
- 연구 `volatilitySizingEnabled`는 기본 false다. 켰을 때 ATR 미준비/0범위 및 위험0bps의 신규 매수는 replay 경계에서 명시 차단한다. RiskManager의 live용 fallback에 맡기면 요청수량 매수로 우회된다. 보유 매도는 이 제한으로 막지 않는다.
- LTH 전용 v4는 `preview_signals_with_position`과 `report_position_snapshot`으로 실제 잔여수량/평균체결가를 되먹임한다. 매도는 실제 보유 전량을 요청하고 부분매도/차단이면 high-water·보유관측 보호 상태를 보존한다. bool `preview_signals_with_execution`은 기존 raw 연구의 요청수량/신호가 체결 가정 호환용이다. 실제 실행수량을 bool 하나로 축약하면 ATR 확대 뒤 잔여 보유가 flat으로 유실된다.
- 52주 신고가 완료 고가는 bounded ring으로 보관하고 만료 시 최고가를 다시 계산한다. tick 수로 거래일을 세지 않으며 day-start 뒤 완료 이벤트만 한 번 소비한다. generic 일봉과 live 완료 일봉 page를 각각 연결했다. live는 manager targeted completed-only 경계이며 startup warmup의 마지막1봉 제외를 중복 적용하지 않는다. 자료 부족 시에도 실제 보유 손절은 유지한다.

- 전수 연구는 `docs/research/strategy-review-20261018/{prepare_data.py,runner.rs,analyze.py}`처럼 입력/창 선언·생산 Rust 실행·집계를 분리한다. 별도 Python 전략으로 결과를 대체하지 않는다. `stock_analyst`가 비용·표본·benchmark·실현/잔여 평가손익을 독립 감사하고 기존 TODO를 갱신한다. 상세 재현 절차와 해석 경계는 해당 `report.md`를 참조한다.
- 각 창의 수량은 평가 전일 자료로 미리 고정하며 비용 스트레스에서도 같게 유지한다. 분기 리셋/겹침 창/부분 창을 구분하고, 비용으로 체결가·손절·guard·후속 경로가 달라지는 결과를 순수 수수료 차감으로 설명하지 않는다. 0신호·준비 부족·미지원·분봉 진단을 정상 성과0%와 구분한다.
- `src-tauri/src/trading/simulation.rs`의 replay 엔진은 raw signal과 `filled`/`blocked` 실행 결과를 분리한다. 성과 지표에는 TradeGuard, RiskManager, 현금·보유 수량·포지션 비중과 수수료/세금/슬리피지/환율을 통과한 체결 가정만 반영한다.
- 과거 시각 replay는 `TradeGuard::{evaluate_for_scope_at,record_submitted_for_scope_at}`을 사용한다. 실시간 wrapper 내부의 `Local::now()`를 test clock처럼 바꾸지 않는다.
- live 시작과 preview warmup은 `strategy::initialize_strategy_warmup()`을 공유한다. 일봉과 장중 봉을 별도 인자로 전달하고 replay 시작 이후 봉을 warmup에 넣지 않는다.
- 일봉 warmup은 `Strategy::initialize_ohlc()`를 한 번 호출한다. 기본 훅은 **종가**를 `initialize_historical()`에 전달해 평균회귀 밴드·추세 필터 MA를 초기화한다. 52주 신고가는 OHLC 훅을 재정의해 **고가**를 선택한다. 공통 경계에서 historical 훅을 별도로 호출하면 고가/종가 혼용 또는 중복 초기화가 재발한다. LTH의 전용 OHLC 훅과 일봉/분봉 분리는 유지한다. high≠close fixture로 지표·첫 신호·호출 횟수·분봉 혼입 방지를 검증한다.
- `StrategyManager::initialize_warmup`은 live provider의 신고가 일봉에서 완료 여부가 없는 최신 1봉을 보수적으로 제외한다(기존 live 정책 유지). 252봉 요구량을 251로 줄이지 않는다. Generic preview는 엄격한 이전 prefix를 직접 공통 훅에 전달해 완료된 마지막 봉까지 포함한다. Provider 완료 플래그/페이지 모델 확정 전 두 경계를 혼동하지 않는다. 재초기화로 신고가 지표가 부족해져도 실제 보유 포지션의 손절은 고가 준비 검사보다 먼저 실행한다. 자료 부족은 신규 진입만 막으며 보유 청산을 막지 않는다.
- Generic preview v4 이후는 평가 `candles`와 이전 `historyCandles`를 따로 정규화한다. 생략한 `warmupCount`는 0이며 명시한 legacy prefix만 호환한다. 합계 500봉 상한, OHLC/시각 유효성·중복·엄격한 prefix 경계를 검사하고 잘못된 history를 걸러 평가봉으로 보충하지 않는다. 분봉 14자리와 일/주/월봉 날짜 8자리 혼합은 거부한다.
- MA/RSI/모멘텀/이격도/연속/돌파 실패 초기화는 `on_tick`을 호출하지 않고 버퍼를 직접 seed한다. MA/RSI는 이전 지표를 복원하며 보유 상태를 유지한다. `history_readiness`는 실제 상태를 읽고 미지원은 None이다. `can_evaluate_next_tick`은 호출 전 조건 평가 가능 여부를 읽어 MA/RSI/돌파 seed-only 봉을 조건 불충족으로 오판하지 않는다.
- `preparation`은 최초/최종 지표 상태, 처리 후 최초 준비 시각·미준비 봉 수, 실제 조건 평가 봉 수를 분리한다. 미지원 값은 null, 조건 평가 0봉은 `notEvaluable`, 준비 후 신호 없음은 `conditionsNotMet`, 신호 있으나 미체결은 `noTrades`다. 기본 200/252봉과 일봉/분봉 분리를 유지한다. 제공봉 수를 실제 버퍼 준비 여부로 대신하지 않는다.
- Generic 일봉 replay는 정보 공개 시점을 지킨다. 날짜 시작 event는 시가-only, 종가 평가에만 완성 OHLC를 사용한다. 입력은 최대 500봉으로 제한하고 결과에는 engine/strategy version, source/interval/range, warmup count, input hash를 남긴다. 전용 LTH 일봉은 이 장중 평가 경로와 구분하는 자료 진단이다.
- Generic preview v5의 `D`/`1d`는 `on_trading_day_start(symbol, open)` → `on_daily_close_tick(symbol, completed_ohlc, volume)` → backtest 체결/차단 및 `sync_position` → `on_completed_candle` 순서다. 시작에는 시가만 전달하고 완료 OHLC는 종가 평가 때 공개한다. 완료봉 적재 전에 실제 포지션을 되먹임해야 차단된 강한 종가 매수가 다음 날 pending을 잃지 않는다.
- 강한 종가는 매 완료봉으로 다음 평가일 조건을 준비한다. 변동성 확장은 날짜 시작 시 당일 시가·고저가만 초기화하고 보유 상태와 이전 N일 범위를 유지한다. 조건 평가 시 이전 N일 평균에 당일 범위를 섞지 않고, 완료 후 bounded ring에 적재하며 flat-day 0범위도 일수에 포함한다. 날짜 전환에 `reset()`을 사용하지 않는다.
- 강한 종가·변동성 확장의 generic replay는 일봉만 허용하며 다른 간격은 `UNSUPPORTED_REPLAY_INTERVAL`이다. cadence는 `dailyCloseWithDayBoundary`. 강한 종가의 다음 평가일 신호도 완료 종가에서 발생하며 v10의 sameClose/nextOpen 실행 모델에 따라 체결한다. live 현재가 polling에는 완료 일봉 이벤트를 연결하지 않는다. 기본 훅은 no-op/기존 tick 위임이며 다른 전략·전용 LTH 동작을 보존한다.
- Generic preview v6은 factory와 같은 `starts_with("leveraged_trend_hold")` 기준으로 기본/접미 ID를 `UNSUPPORTED_GENERIC_REPLAY`로 명시 거부한다. LTH의 실시간 분/세션/blackout clock을 과거 재생에 호출하지 않으며 `deterministic=false`·준비 미지원 표시에 그치지 않는다. broker 조회 전용 커맨드로 generic 제공봉을 자동 우회하지 않는다.
- LTH `preview_signals_with_execution`은 실제 분봉의 `timed.time`으로 세션·blackout·장마감을 판단한다. 응답 `generatedAt`만 생성 시각이고 신호/체결/hash의 입력이 아니다. `cfg(test)` live clock 대역으로 production 재현성을 주장하지 않고 별도 integration fixture 및 production runner의 서로 다른 분/시간대 재실행을 검증한다. 고정 KST 세션·DST 한계는 입력 재현성과 별개이며 `lookAheadSafe=false`다.
- LTH 일봉 `initialize_ohlc`는 context만 교체하고 기존 분봉 지표와 보유 포지션을 보존한다. `initialize_intraday_ohlc`는 분봉 OHLC·반동 가격·볼린저 snapshot을 교체하며 누적 append로 과거 관측을 중복하지 않는다. `initialize_intraday_prices`는 반동 가격 snapshot만 교체하고 OHLC·볼린저를 초기화하지 않는다. 분봉 EMA/RSI/ADX·반동·볼린저는 분봉만 소비하고 각 버퍼 상한을 독립 유지한다. 빈 일봉 재초기화도 실제 보유 손절을 막지 않는다.
- 전용 LTH v8/LTH v4의 엔진 버전은 공통 `REPLAY_ENGINE_VERSION`을 응답과 hash에 함께 사용한다. `ReplayMetadata.assessment`는 선택 필드이며 generic에는 생략한다. `1d`는 `dailyDiagnostic/notEvaluable`, `1m`은 `intradaySample/sampleOnly`(유효 관측 0개면 `notEvaluable`)다. `dailyContextBars`와 `intradayBars`를 분리하고 `warmupCount=0`으로 일봉이 분봉 지표를 준비했다고 주장하지 않는다. `timestampStatus/sessionStatus=unverified`와 한국어 limitations를 남긴다. 일봉 호환 빈 backtest를 0% 성과로 해석하지 않는다. 실제 2026-07-06~10-02 SOXQ 분봉·시간대·DST·세션 coverage 확보/검증은 P1-05 잔여 조건이다.
- 전략이 raw signal 생성 시 내부 `in_position`을 선반영했는데 주문이 skip되면 `SubmissionOutcome::Skipped`의 실제 `held_quantity`/`avg_price`를 `StrategyManager::sync_position()`에 되먹임한다. 수량 0 snapshot도 flat 상태로 반영해야 한다.
- 자동매매 시작 플래그 lock은 broker reconcile/risk restore/warmup이 완료될 때까지 daemon 첫 tick을 막아야 한다.

> 마지막 업데이트: 2026-10-18T10:22:52+09:00
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
- `database/connection.rs`는 pool 최초 생성을 별도 mutex로 직렬화하고 게시 직전 설정 fingerprint를 재검사한다. 상태 view는 같은 설정·pool snapshot을 사용한다. PostgreSQL `Require=VerifyFull`와 MariaDB `Require=VerifyIdentity`는 대상 DB 자동 생성의 관리자 연결에도 유지한다. 서버 복구는 SQLx의 연결 취득 검증에 맡기며 결과가 불명확한 쓰기를 자동 재실행하지 않는다.
- `database/configuration.rs`는 설정 파일과 keychain을 함께 저장/복구한다. 저장 전 비밀 없는 `.recovery` 표시를 내구성 있게 생성하고 메모리 게시까지 유지한다. 쓰기 오류는 rename 이후 오류일 수 있으므로 원래 파일 내용과 keychain을 둘 다 복원·검증한다. 취소/불확실 복구는 load/save/pool을 차단한다. primary 원복 삭제와 marker 삭제도 내구성 있게 반영하고, marker 삭제 후 sync 실패 시 표시를 재생성한다. 실행 중 recovery latch는 표시 재생성 실패에도 연결/저장을 차단하며 검증된 원복 또는 메모리 게시 완료 후에만 해제한다. 표시만 삭제하는 자동 우회나 JSON fallback을 추가하지 않는다.
- `npm run test:database-tls-contract`는 고유 라벨의 로컬 Docker PostgreSQL16과 임시 CA·hostname 인증서를 사용한다. 일반 계약3개와 TLS/서버 재시작2개를 실제 실행하고 운영 설정/pgpass/OS keychain은 사용하지 않는다. TLS2개는 `--exact` 독립 child process에서 실행하여 global manager와 신뢰 CA 환경이 다른 단위 테스트에 유출되지 않게 한다. 상세 환경·정리·한계는 `docs/HARNESS_MAP.md`를 따른다.
- v1 schema는 기존 JSON 복원성을 위한 document store다. 주문/체결 복구·검색·retention을 위한 정규화 schema는 명시적인 schema version migration과 PostgreSQL/MariaDB contract test를 함께 추가한다.
- JSON 파일 저장은 경로별 lock과 store별 read-modify-write lock을 사용한다. 같은 디렉터리에 temp를 생성해 내용 fsync 후 handle을 닫고, 기존 정상본을 `.bak`에 복사·동기화한 뒤 교체한다. Unix는 읽기 전용 백업 fsync와 rename → parent directory fsync를 유지한다. Windows는 쓰기 가능한 백업 handle과 `MoveFileExW(MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)`를 사용하며 copy/delete fallback을 허용하지 않는다. Windows 경로는 기존 parent만 canonicalize해 Unicode/long-path UTF-16 경로로 전달한다. 교체 API 자체 실패 시 기존 정상본을 유지하고 temp를 정리한다. Unix rename 이후 parent fsync 실패는 오류를 반환하지만 새 정상본은 이미 반영될 수 있다. 역직렬화 실패 시 손상본을 격리하고 마지막 정상 백업으로 복구한다. 이 절차의 fixture 테스트는 실제 전원 차단 시 내구성을 증명하지 않는다.

> 마지막 업데이트: 2026-10-06T04:01:20+00:00


## 자동매매 전용 예산 / 볼린저 보강 불변식 (2026-09-11)

- 자동 주문은 `submit_signal_shared`의 scope별 durable 예산 예약을 반드시 통과한다. 기본 예산0, 수동 주문/보유 구분, 통화 분리, 비용 여유, 부분체결 멱등 처리를 유지한다. 리스크 비활성화가 예산을 해제하지 않는다.
- 예산 terminal 표식은 pending 제거 전 저장한다. 실패나 불명확 응답에 예약을 조기 해제하지 않는다. 원장에 일반 JSON backup 자동 복구를 적용하면 과거 현금으로 되돌아갈 수 있으므로 엄격히 읽는다.
- 자동매수만 지정가 상한을 강제한다. 매도 유형을 함께 바꿔 손절 주문 체결 가능성을 떨어뜨리지 않는다. 예산 주문 정정은 예약 재계산 없이 허용하지 않는다.
- LTH `bollinger`는 기본 비활성 선택 필터다. 추세·장중반동·급반등 세 진입 경로 모두에 적용하고, 보유 중 손절 경로를 막지 않는다. 일봉과 분봉 혼합 금지, 512봉 상한, 마지막 미확정봉 제외, live/replay 공통 함수·회귀 테스트를 유지한다.

마지막 업데이트: 2026-09-11

> 마지막 업데이트: 2026-10-05T23:54:16+09:00

## Rust 1.99 Clippy 호환 패턴 (2026-10-06)

- async-trait 0.1.89는 trait async 메서드에 중복 must_use를 생성해 Rust 1.99의 double_must_use가 발생한다. 앱 trait의 Send/lifetime 계약을 직접 재작성하거나 lint allow/expect를 추가하지 않고 공식 수정판 0.1.92로 갱신한다. 추가 syn 3.0.6의 최소 Rust 버전도 1.71이라 프로젝트 1.90 기준을 유지한다.
- server/market.rs의 private validate_toss_web_order는 큰 HTTP Response를 오류에 직접 넣는 대신 Box<Response>에 보관한다. 네 오류 경로의 응답을 그대로 감싸고 국내·해외 caller에서 *response를 이동하여 상태·헤더·본문·검증 순서를 보존한다. 공개 IPC와 REST 계약은 변경하지 않는다.
- 이 helper의 정상/provider/pending 경로는 실제 Toss endpoint를 사용하므로 API 미호출 검증에서는 합성 프로파일의 비 Toss 및 수량 0 조기 오류만 실행한다. 네트워크 이후 분기를 실행했다고 주장하지 않는다.

## 보안 의존성 및 Mac 검증 기준 (2026-10-06)

- Tauri 2.12.1의 최소 Rust 버전 1.90을 manifest와 harness에 유지한다. Windows Rust 1.92는 이 기준을 충족하며 Mac은 1.95에서 검사했다.
- Windows 저장 패치의 Unix read-only 교체와 private 저장 0600 fixture, backup 실패 시 정상본 보존, 손상 JSON 복구 및 동시 쓰기를 Mac lib 테스트로 확인했다. 실제 전원 차단 내구성과 사용자 데이터 복구를 증명하지 않는다.
- quick-xml 예외 두 개는 plist 1.10.1 업데이트로 해제했다. RSA 예외 하나와 Linux GLib/proc-macro-error 경고는 docs/release-security.md의 실제 경로·해제 조건을 확인하며 테스트 약화나 추가 ignore로 해결하지 않는다.
- PostgreSQL contract 테스트는 DB 환경 변수가 없으면 즉시 반환하므로 전체 통과 개수에서 실서버 검증으로 계산하지 않는다. 이번 Mac 검증에서도 DB 환경 변수를 제거했고 실거래·모의 주문 API를 호출하지 않았다.

> 마지막 업데이트: 2026-10-18T11:00:43+09:00

## 완료봉 페이지와 분봉 actual instant (2026-10-18)

공통 page DTO·scope·cursor·완료 정책은 `docs/ipc-commands.md`의 공통 캔들 페이지 계약을 정본으로 사용한다. 기존 Vec 차트 메서드는 유지하며 broker별 typed cursor를 바꾸어 넣지 않는다. 커서가 고정한 profile/account/투자모드·입력 범위·asOf를 매 페이지 검증한다. 시크릿은 cursor/응답에 넣지 않는다. quote/stop polling을 페이지 수집 network await로 막지 않는다. live 갱신은 별도 generation·config identity·bounded backoff가 필요하다.

❌ RFC3339에서 숫자만 뽑아 compact14로 만들면 offset을 잃거나 길이 오류가 난다. ✅ generic minute replay는 RFC3339를 실제 순간으로 parse해 고정 정밀도 UTC로 정규화하고, 중복·정렬·history overlap·입력 hash에 같은 기준을 사용한다. legacy compact12/14는 KST 차트 계약으로 해석한다. simulation guard parser와 UI 차트도 RFC3339 offset을 우선 해석한다. RFC3339 replay의 일일 리스크/손절 당일 기준은 KR Seoul·US New York으로 고정하고 host Local 날짜를 사용하지 않는다. guard의 지정시각 경계는 generic timezone을 받아 내부 FixedOffset으로 저장해 실제 경과와 시장 날짜를 함께 보존한다. 실운용 Local wrapper 정책은 유지한다. 분봉 시작/종료 의미는 provider별로 확인하며 임의 시각이나 누락봉을 합성하지 않는다.

> 마지막 업데이트: 2026-10-18T11:38:53+09:00

## Live 완료 일봉의 세대·준비 상태 경계 (2026-10-18)

- `commands/trading/daily_history`는 quote polling과 분리한 동시 2개 background task다. 각 8회/512봉, 같은 scope/날짜/종목의 설정은 최대 요구량 공유 조회, 대상 511개+초과 진단 1행, 30~900초 retry로 한정한다. 완료된 handle만 await하고 삭제/정지/교체 task를 abort한다.
- manager ticket은 run generation/config fingerprint/profile/broker/account/symbol/시장 날짜를 포함한다. start/stop/close/설정 변경·rollback은 무효화한다. apply 직전 bool→manager lock 순서로 확인하며 네트워크 await는 lock 밖이다. quote await 중 날짜 전환을 막으려 on_tick 직전에도 `invalidate_stale_daily_history_for_symbol`을 호출한다.
- 완료 일봉만 `apply_daily_history`로 적용하고 마지막 봉을 다시 버리지 않는다. 당일/미래/무효/중복 제외, 기본252/최대512 요구, 마지막 완료일1~10일 과거를 확인한다. 이 freshness 상한은 실제 휴장 달력 검증이 아니다. 부족/실패/한도 초과도 신규 매수만 막고 buy_price 손절 보존, 무관한 LTH 분봉 initializer 미호출을 검증한다.
- `Strategy::reconfigure_preserving_position`은 기본 false이며 현재 13개 전략이 동일 ID/broker/account에서 opt-in한다. 새 파라미터로 유효 관측을 재계산하되 잔존 대상의 보유 기준을 유지한다. LTH의 entry/high-water/held-observations를 평균가 snapshot으로 다시 만들지 않는다. 시작 시 옛 snapshot cache를 새 보유 복원 원본으로 만들지 않는다.
- `TradingStatus.dailyHistory`의 IPC/웹/TS/UI를 같이 갱신한다. 실제 확보/요구·상태·출처·마지막 완료일·실패/재시도를 표시하며 legacy 누락을 ready로 추정하지 않는다. 상세 계약은 docs/ipc-commands.md, 검증은 docs/tasks/live-completed-daily-history-20261018.md다.
- 일봉 ticket과 주문 run은 서로 다른 소비 경계를 가진다. 완료 일봉 준비도 bool→manager→order에서 동일 run snapshot으로 대상을 만든다. 잘못된 scope/profile 조회는 새 실행 generation을 변경하지 않는다. 설정 저장의 보유 보존은 아래 transaction 경계를 따른다.

> 마지막 업데이트: 2026-10-18T11:57:42+09:00

## 자동 주문 실행 context와 예약 경계 (2026-10-18)

- `order/execution_run.rs`는 epoch·configuration revision·BrokerScope·profile ID와 pinned client/profile을 관리한다. 민감 snapshot을 Debug/serde로 노출하지 않는다. begin은 profile/broker/account/credential/투자 모드를 검증하고, end·scope setter는 epoch를 무효화한다.
- `submit_signal_shared`는 mandatory expected context를 받는다. 자동 요청의 context 누락·변경은 `StaleRun`이다. OrderManager mutex 안에서 context→guard→submitting→예산 순으로 검증하며 예약 후 client를 다시 읽지 않는다. 수동 요청은 자동 run 없이 요청 scope를 검증한다.
- daemon은 quote/name/position/preflight await 후 현재 실행을 확인하고, on_tick/rollback은 bool→strategy→order fence 내부에서만 실행한다. stale snapshot으로 새 전략을 sync하지 않는다. provider/network await에는 bool/strategy guard를 유지하지 않는다.
- start는 profile/config publication과 `strategy_update_lock`으로 직렬화하고 snapshot을 고정한다. IPC/웹 설정은 저장 성공 후 manager→order에서 새 revision을 공개하며 저장 실패는 기존 revision을 유지한다. A→B→A도 이전 신호를 복구하지 않는다. revision 역행은 실행 무효화로 처리한다.
- 프로파일 publication은 manager→order gate 아래 scope/client/config/양쪽 tracker/전략 설정을 함께 바꾼다. 수동 holdings 게시·예약도 order gate에서 exact active profile/scope/client를 확인한다. no-profile KIS 설정 계좌의 기존 수동 경로는 유지한다.
- reconciliation은 실제 client/profile을 고정하고 응답 후 scope/credentials/mode를 다시 검사해 fill/terminal/health를 적용한다. 같은 scope의 새 epoch는 실제 체결을 막지 않는다. KIS pending의 저장 TR-ID로 모드를 확인하며 반대 모드·미상 출처는 pending/예산을 보류하고 기존 health에 진단한다. 이전 계좌·모드의 잔고 부족 응답으로 현재 실행의 매수를 정지하지 않는다.
- end·자정 reset은 이미 예약한 submitting/pending/예산을 지우지 않는다. 예약이 먼저 완료된 주문은 최초 scope/client로 완료·기록·대조한다. 정지는 provider 취소가 아니다. 비활성 프로파일 삭제는 runtime을 재생성하지 않는다.
- 회귀는 실제 shared submission coordinator와 localhost 주문 서버를 사용한다. synthetic TokenManager cache는 `cfg(test)`로만 주입하며 실제 token/order URL로 fixture를 보내지 않는다. 검증 기록은 `docs/tasks/execution-run-reservation-20261018.md`를 따른다.

> 마지막 업데이트: 2026-10-18T12:29:40+09:00


## 전략 설정 transaction과 보유 상태 (2026-10-18)

- 전략별 재설정의 ID/broker/account 검사는 `StrategyConfig::same_identity`를 사용한다. 기존 `matches_scope`의 candidate account 빈 문자열 처리 의미를 유지하고, 상태/관측 복원 로직은 각 전략에 둔다.
- IPC/Web 수정은 `trading/configuration.rs`의 공통 coordinator를 사용한다. `strategy_update_lock`은 owned task가 소유하고 저장 전/후 manager→order에서 profile/client/범위와 주문 충돌을 검사한다. manager/order guard는 파일·DB await 동안 해제한다. 대기자 취소가 durable save와 runtime 공개 사이를 중단하지 않게 한다.
- save 전 인스턴스/revision을 바꾸지 않는다. 성공 후 기존 인스턴스를 재설정하고 새 revision을 공개한다. 저장 실패는 기존 보유/설정을 유지한다. 첫 저장 오류도 교체 후 fsync/commit 실패일 수 있으므로 저장 후 identity 검증 실패와 함께 원본을 다시 저장한다. 복구 실패는 persistence block이다.
- manager/order가 Weak lease만 보유한다. transaction 종료 시 자동 해제되며 같은 scope의 새 신호와 영향 종목 주문 예약·전체 수동 holdings 게시를 잠시 보류한다. 이 보류를 시장 폐장으로 분류해 5분 대기하지 않는다.
- raw 신호는 current generation별 미처리 표시를 남긴다. 제출/차단/오류에서 exact 전략만 완료 처리하고 폐장 조기 종료 시 나머지 신호도 현재 실행 gate에서 정리한다. 이미 제출한 다른 전략의 낙관적 상태를 덮지 않는다.
- private held 또는 실제 tracker 양쪽 중 하나에 보유가 있으면 대상 제거/시장 변경을 거부한다. PC params.symbols는 빈 배열까지 대상의 정본이고 LTH 거래 대상도 production constructor로 검증한다. 두 전략 모두 실제 on_tick처럼 첫 중복 entry의 시장 구분을 사용하며 LTH의 legacy inverse 필드로 다른 대상을 덮지 않는다. 공유 tracker의 0수량이나 startup cache로 private 보호를 지우지 않는다. 일반 잔고 게시 fencing은 아래 공통 gate를 사용하며 정상 체결은 §12의 exact owner 피드백을 사용한다.
- ❌ 저장 전에 전략을 재생성하고 실패 시 저장한 config로 다시 재생성하면 보유·trailing 상태를 잃는다. ✅ 먼저 저장한 뒤 같은 ID/scope에서 보유를 유지하며 공개한다. 실제 수량의 소유권을 모르면 새 대상에 다른 전략/수동 보유를 배정하지 않는다.

> 마지막 업데이트: 2026-10-18T12:57:00+09:00

## 일반 잔고 응답 게시 경계 (2026-10-18)

- `order/holdings.rs`의 조회 ticket은 profile/client/scope/투자 모드, 실행 epoch, 설정 Weak lease identity와 시장별 tracker version을 고정한다. 민감 profile snapshot은 Debug/serde로 노출하지 않는다.
- `begin_holdings_query` 후 네트워크 await에는 전역 lock을 해제한다. 게시 시 동일 OrderManager mutex에서 현재 identity와 version을 검증한 뒤 `publish_holdings`만 tracker를 replace한다. IPC/Web 시작 복원은 manager→order를 유지해 게시와 전략 복원 사이 fill을 막는다.
- tracker는 quantity mutation과 최신 조회 UUID를 따로 보관한다. buy/sell/clear/replace·tracker 재생성·후속 요청이 먼저 발생하면 이전 ticket을 폐기한다. update_price는 수량을 바꾸지 않는다. 국내/해외 Both는 양쪽 version을 모두 검증한 뒤 함께 게시한다. 실제 빈 응답과 0수량은 정상 삭제이며 오래된 응답과 구분한다.
- IPC·daemon·시작 복원·수동 주문 갱신에 공통 gate를 적용한다. tracker를 쓰지 않는 Web/일반 broker 조회도 ReadOnly ticket으로 응답 반환 전 identity를 확인하며 새 quantity query를 무효화하지 않는다. no-profile 빈 결과도 검증한다.
- KIS 공통 refresh는 엄격한 수량/가격 변환과 종목명 저장 await를 완료한 뒤 최종 gate를 실행한다. 실패한 값은 0으로 대체하지 않는다. 잔고 이벤트 callback은 성공한 gate 내부에서 동기 실행한다. frontend는 이벤트를 현재 scope 재조회 힌트로 사용한다.
- profile fingerprint는 execution_run의 공통 순수 helper를 사용하되 broker/scope/client 비교를 별도로 유지한다. 해외 거래소 trim/대문자 정규화는 holdings helper를 사용하며 다른 의미의 order_exchange_code와 합치지 않는다.
- 이 gate는 조회 시작 이후의 로컬 변경과 계좌 경합을 막는다. provider snapshot에 이미 포함된 체결의 중복은 계좌 delta 변경을 없애고 소유 원장과 재조회로 분리해 방지한다. P1-FILL-01의 lifecycle 검증은 `docs/tasks/strategy-fill-ownership-20261018.md`를 따른다.

> 마지막 업데이트: 2026-10-18T13:35:00+09:00


### 저장 intent와 정밀 체결 후속 계약 (2026-10-18)

`confirmed_filled_notional`은 intent의 수량·날짜·US 환율과 함께 고정한다. 재시도 중 더 큰 누적값이 와도 원래 intent부터 완료한다. 0수량 terminal도 저장 intent 복구를 건너뛰지 않고, 기확정 수량보다 작은 terminal은 pending을 제거하지 않는다. owner 원장 영수증은 terminal side effect와 pending 제거가 durable한 뒤에만 정리한다.

소액 Toss 즉시 반영은 원 주문 ID·symbol·BUY·통화·정수1주·명시금액을 검증한다. 금액 누락은 즉시 반영을 보류하고 pending을 유지한다. 계좌·record 일치 eligibility를 실제 반영 lock 안에서 다시 검사한다. 수량/금액/날짜는 collector의 같은 checked parser를 재사용한다.

> 마지막 업데이트: 2026-10-18T15:07:40+09:00

## v10 replay 체결 모델과 정보 공개 경계

- `simulation/session.rs::ReplaySession`은 batch wrapper·generic·LTH 분봉이 공유하는 단일 회계다. begin_bar에서 이전 신호를 관측 open과 이전 완료 ATR로 처리하고 optional 보호 시나리오를 실행한 뒤 실제 snapshot을 전략에 반영한다. process_close에서만 현재 OHLC ATR과 종가 신호를 공개하고 equity를 기록한다. prefix 전체를 재실행하거나 사전 계산한 raw signals를 다른 체결 모델에 재사용하지 않는다.
- 기본 sameClose 수치를 유지하고 nextOpen은 다음 관측봉(누락 보간 없음) 시가를 사용한다. 마지막 대기 신호는 unfilled이며 거절/완료가 아니다. signalAvailableAt/fillAt는 봉 label+phase이며 정확한 거래소 timestamp를 합성하지 않는다. sameClose의 lookAheadSafe=false와 경고, LTH의 자료 진단/표본 경계를 보존한다.
- 연구 stopLossPct/takeProfitPct는 기본0으로 live/전략 파라미터와 분리한다. 실제 평균 체결가·갭 시가·부분 원가를 사용한다. OHLC 양방향 접촉은 순서 미상이며 stopFirst에만 보수적 가정 체결을 허용한다. sameClose 신규 진입에 같은 봉의 이미 지난 고저가를 적용하지 않는다.
- LTH의 preview_signals_with_driver(Open/Signal/Close)는 매 봉 callback을 호출한다. 신규 보유 high-water는 open 단계에 close를 미리 넣지 않고, close 관측을 정확히 한 번만 반영한다. 실제 수량·원가와 부분 청산 보호를 유지하며 기존 bool/position callback은 호환 wrapper다.
- 기본 호환은 바뀐 전략이 섞인 옛 연구 결과만으로 판단하지 않는다. 직전 커밋의 생산 라이브러리에서 같은 입력을 재실행하여 신규 metadata를 제외한 전체 회계 report와 signals를 비교한다. QQQ2026Q3 비용·next-open 청산 경로 fixture와 실제 연구/합성 OHLC 계약을 분리한다.

> 마지막 업데이트: 2026-10-18T15:21:57+09:00

## replay v11 원가·현금흐름 불변식

- native 평균 체결가와 역사적 KRW 원가·잔여 매수 fee를 따로 보존한다. 현재 FX로 KRW 원가를 역환산해 전략 진입가를 바꾸지 않는다. 부분청산 fee는 실제 수량 비례로 배분한다.
- 자산 증감은 실현 순손익+잔여 순평가손익+세후 배당과 일치해야 한다. 가격 변화는 진입 FX, 환율 변화는 최종 native 금액으로 귀속한다. 유휴 현금은 KRW인 연구 장부다. 기말 청산은 보고만 작성하고 거래·리스크·승률을 변경하지 않는다.
- 방향별 fee는 공통 bps를 fallback하며 주문당 최소 fee는 국내 KRW/해외 USD×해당 봉 FX다. 최소 fee 사용 시 일반 이익매도는 실제 배분 매수 비용과 매도 비용으로 기대 순익을 확인한다. 손절·보호·native 손실 청산은 수익 gate로 막지 않고 기본 minimum=0 수치를 보존한다. `project_sale`을 guard와 실제 정산이 공유한다.
- 시나리오는 각각64개/시각64바이트/유한수 범위를 검증하고 history 제외 평가봉에 정확히 정렬한 뒤 hash에 포함한다. 배당 조정 가격+현금 배당은 거절하고 미확인은 경고한다. 배당은 봉 직전 보유분에 봉 시작 즉시 세후 KRW로 지급하는 가정이다.
- 비교는 같은 요청 수량과 실제 체결량·노출 차이를 명시하며 같은 엔진을 재사용한다. 실제 경로 비용 환급을 무비용 전략 재실행으로 표현하지 않는다. 소표본·부분 분기·겹침창과 실제 자료/합성 가정을 구분한다. 상세 정본은 `docs/ipc-commands.md`와 P2-15 연구 보고서다.

## replay v12 연속 장부 불변식

- 기존 flat 입력의 history+평가500봉 상한을 유지한다. 새 continuous는1~16청크·각1~500봉·평가총4096·history512이며 unknown field/legacy warmup 혼용을 거부한다. 모든 입력을 먼저 정규화·검증하고 history 내부/청크 내부/경계의 엄격한 시간순을 보장한다. 전역 sort로 잘못된 공급순서를 고치지 않는다.
- `strategy_preview/driver.rs`가 flat/continuous의 단일 생산 실행 경로다. factory·최초 warmup·ReplaySession·최종finish는 각각1회다. 청크마다 reset/재초기화/finish하면 pending nextOpen·원가·ATR·쿨다운·리스크를 잃는다. 경계 ledger는 수량/pending 관측만 하며 live 계좌/장기 서버세션을 만들지 않는다.
- 계산모드는 inputHash에 포함하고 청크 배치만 바뀐 동일 경제 입력은 같은 hash/report/signals를 가져야 한다. 단일/분할의 실제 전략과 sameClose/nextOpen 미래 prefix 불변을 검증한다. 회계 peak 캐시는 이전 반올림 equity 최댓값이라는 기존 순서를 보존한다.
- 연속70/30은 같은 상태를 이어받는 앞/뒤 구간이며 독립학습/검증이 아니다. 분기 초기화는 별도 현금/포지션 계좌이므로 수익률을 합산/복리하지 않는다. 연구 수량·PC기준가/내부수량은 최초 결정값을 고정하고 차단·실제 노출 차이를 남긴다.
- fixedFX/no배당/nextOpen/nooverlay에서는 실제 전봉 보유수량의 close→open 갭과 accepted open 체결 후 수량의 open→close 시장손익을 나눠 기존 비용환급 참조와 대사할 수 있다. 동일 수량/보유구간이지 동일%자산비중·독립 알파 benchmark·무비용 전략 재실행이 아니다.
- 새 IPC/REST/TS/hooks/transport와 source/시나리오 상한·UI 준비/취소/legacy 모드를 함께 갱신한다. 정본은 `docs/ipc-commands.md`, 검증·한계는 `docs/research/continuous-replay-20261018/report.md`를 따른다.

> 마지막 업데이트: 2026-10-18T16:14:29+09:00

## Consecutive v13 진입 확인과 완료 일봉 분리

- 기존 buy_days/sell_days는 가격 관측 횟수다. 선택 3필드는 각각 serde default0이며 신규 필드 오류가 기존 custom 매도 기간을 전체 기본값으로 덮거나 필터를 조용히 끄지 않게 한다. 저장/replay는 엄격히 거부하고 손상된 runtime params는 신규 매수만 차단한다.
- 누적 변화는 native 관측 차이, ATR14는 직전 완료 실제 OHLC15의 산술 TR, MA는 직전 완료 종가 N으로 계산한다. 현재 후보 OHLC는 판단/체결 피드백 후에만 게시한다. ATR/MA 활성 generic은 일봉만 지원한다.
- 완료 일봉 coordinator의 opt-in 훅은 context만 갱신·무효화한다. 기존 tick 관측열·확정 보유·청산을 보존하며 legacy warmup을 검증된 완료 일봉으로 승격하지 않는다. 0 ATR/준비 부족은 신규 진입만 보류한다.
- 후보/준비 부족/각 필터 거절/통과 진단과 후속 guard/risk 차단을 구분한다. 필터 통과는 체결이 아니며 optional 진단 부재는 필터 활성 시 legacy unknown이며, 유효 숫자0/누락으로 모두 비활성이 확인되면 UI는 중립 표시한다. 기본0 전체 report/signals 호환과 실제 비용별 경로를 검증하고 guard를 완화하지 않는다.
- ❌ quote 관측열을 일봉 ATR/MA로 계산하거나 일봉 재조회로 streak/보유를 초기화하면 시간 간격과 청산 계약이 깨진다. ✅ 별도 bounded 완료 일봉 context와 기존 방향 상태를 유지한다. DTO 정본은 docs/ipc-commands.md, 연구는 docs/research/consecutive-filters-20261018/report.md다.

> 마지막 업데이트: 2026-10-18T16:31:45+09:00


## Momentum v14 소유 피드백과 진입 무장

- Momentum raw 관측은 확정 보유를 변경하지 않는다. owned_quantity는 exact owner sync로만 갱신하며 flat Buy/held Sell을 적용한다. 이전 last_sell_price를 같은0피드백마다 지우는 억제 anchor에 보유 여부를 맡기지 않는다.
- 기본 inclusive >=T/<=−T를 유지하고 entry-only crossing(previous<T,current>=T)/간격%p를 별도 선택으로 둔다. 후보는 condition<T에서 취소, 재무장은 condition<=T−band에서만 한다. 1pp deadband의 중간 구간을 재무장으로 처리하면 안 된다.
- nextOpen pending/동일0qty를 체결로 추측하지 않는다. 무체결 후보는 조건 유지 중 재시도하고 실제positiveownedqty로 소모한다. heldnegativelevel은 provider/guard거절·부분매도 뒤에도 평가한다. 요청q와 소유q의 최종min 제한은 공통 submission에 유지한다.
- strict warmup/reconfigure는 가상 신호·보유를 만들지 않고 동일설정은 이전m/무장/후보를 보존하고 설정이 바뀌면 실제소유와 잔존가격을 유지하면서 관측으로 이전m/무장을 재seed하고 후보를 해제한다. 새optional 설정은default를두고 invalidruntime은매수만failclosed, save/preview는거부한다. 비모멘텀1122회실행호환과Momentum raw/filled/현금 차이를 분리해검증하며 raw감소를수익개선으로해석하지않는다. 정본은 docs/ipc-commands.md, 연구는 docs/research/momentum-feedback-20261020/report.md다.

> 마지막 업데이트: 2026-10-20T19:49:11+09:00


## Deviation v15 실제 소유·관측 청산

- 기본 현재 포함 MA·inclusive 매수/매도 임계값을 유지하되 실제 소유량으로 flat Buy/held Sell을 제한한다. 신규 optional 필드는 개별 검증하며 손상된 필드로 custom legacy exit를 덮지 않는다. 저장/두 preview는 엄격 거부, runtime 신규 진입은 fail closed하고 유효 보유 보호는 평가한다.
- crossing은 이전d>buyT→현재d<=buyT의 하향 진입이다. 후보는 <=T 중 동일0/pending/거절에도 유지하고 >T면 취소하며 양수 실제 체결로 소모한다. strict history는 가상 후보/나이를 만들지 않는다.
- 장기 추세는 현재 제외 직전N관측 평균과 엄격 current>mean이다. live 첫 관측에서 별도 context를 비우고 실제 live 관측으로 준비한다. 공통 daily/1분 warmup을10초 관측이라고 간주하지 않는다. legacy MA warmup은 보존한다.
- 보유 나이는 실제 체결 뒤 관측이며 off 중에도 기록하고 같은qty/partial/warmup/설정 변경으로 리셋하지 않는다. exact owner metadata가 없고 진입 후보도 없는 positive 복원은 unknown이다. maxHold 활성 시 다음 관측에 보수적 매도 후보를 내며 날짜/체결 한도로 표현하지 않는다.
- 손절→관측한도→고이격 level 순서다. 보호 후보는 실제 잔량 전량, legacy level은 config수량을 유지하고 공통 clamp/guard/risk를 우회하지 않는다. native 평균가·관측 손절은 장중 손실 상한이 아니다. 새 DTO 없이 UI·저장·두 preview·A/B를 동기화한다. 정본은 docs/ipc-commands.md, 연구는 docs/research/deviation-policy-20261020/report.md다.

> 마지막 업데이트: 2026-10-20T21:51:20+09:00


## MeanReversion v16 실제 소유·선택 청산·공통 관측

- 현재 포함 period20 모집단 분산/std2, strict price<lower/price>upper와 기존 native 손절 계산식/기본5를 보존한다. 손절0은 off가 아니고 원가 이하 손절이며 Deviation의0off와 다르다. 선택 middleBand는 같은 current-inclusive mean을 strict 상회한다.
- q/nativeavg는 exact owner sync에서만 갱신하고 raw signal로 선반영하지 않는다. stop/band는 configuredq, maxHold은 소유 잔량 전량 후보다. stop→maxHold→band 순서·거절/partial/pending·지표 준비 전 보유 보호를 유지한다.
- longtrend는 이전N관측/current제외이고 live 첫 관측에만 seed를 비운다. Mean의 legacy 분봉 warmup no-op과 reset 가격 유지, Deviation의 분봉 seed/resetclear를 합치지 않는다. 미확인 분봉 history를 충분한 준비로 표시하지 않는다.
- 실제 체결 뒤 나이는 off 중에도 세고 sameqty/partial/warmup/params 변경으로 초기화하지 않는다. candidate/pending 또는 kind가 맞는 known checkpoint로 복원하며 나이 불명에 한도 활성은 보수적 매도 후보다. 날짜/장중손실 한도로 주장하지 않는다.
- `strategy/observation.rs`는 bounded_optional_integer, advance/sync_owned_age, encode/decode_owned_observation, trend_allows/observe_trend/begin_live_trend의8개 pub(super) 함수다. 두 kind를 엄격 분리하고 decode 성공 뒤 caller만 상태를 게시한다. q/avg/symbol/scope·각 전략 판정은 포함하지 않는다. Graphify 후보와 Serena 실제 refs·기계적 동등성 검증 후 승격하며 원래 API/원장 JSON을 보존한다.
- 저장/두 preview strict validation과 UI raw/A-B·단위 안내를 동기화한다. 기존 frozen QQQ1주·비용/체결4경로는 그대로 유지하고 새3주후보/OOS에 합치지 않는다. 계약은 docs/ipc-commands.md, 연구는 docs/research/mean-reversion-policy-20261020/report.md다.

> 마지막 업데이트: 2026-10-20T22:32:26+09:00


## MA cross v17 완료 일봉 진입 필터

- 기본5/20 입력관측 교차/Sell/prevMA 갱신을 유지한다. 선택 추세·기울기·ATR%·밴드폭%는 이전 완료 DAILY OHLC로 Buy 후보만 확인하며0은 비활성이다. 거절한 교차를 뒤늦게 되살리지 않는다.
- 맥락은 최대512봉, apply/invalidate 완료자료는 기존 cross열과 독립이다. 현재일봉은 판단 뒤에만 적재하며 live10초/분봉 OHLC로 일봉을 합성하지 않는다. 활성옵션 비일봉 preview는 명시 거부한다.
- strict params/save/two preview·실제raw UI를 동기화한다. 손상optional runtime은 기존 custom periods/Sell을 보존하고 Buy만 failclosed. slope활성은 long_period1..511, 필요한 이전일봉은N/L+1/15/20이다.
- completed_daily의 OHLC유효성/u128종가평균/양수14TRmean만 공유하고 lifecycle/counter는 각 전략에 남긴다. 충분한ATR0에서 MA는 ready+임계값거부, Consecutive는 기존zeroAtr준비차단을 유지한다. optional slope/bandwidth rejection은MA만직렬화하고Consecutive는생략한다.
- 후보 수는 raw golden cross이며 실제 소유/체결이 아니다. 기본 전체1224호환·분할 불변·공통guard/리스크 유지·독립 비용/노출 감사와 [P2-01 연구](../../../docs/research/ma-cross-filters-20261020/report.md)를 확인한다.

> 마지막 업데이트: 2026-10-20T23:11:46+09:00
