# KISAutoTrade 아키텍처

이 문서는 현재 코드에 구현된 경계와 고위험 데이터 흐름을 설명한다. 전체 파일 목록은 [`docs/project-map.md`](docs/project-map.md), 작업별 진입점은 [`PROJECT_MAP.md`](PROJECT_MAP.md)를 사용한다.

## 1. 시스템 경계

KISAutoTrade는 React 18 UI와 Rust backend가 결합된 Tauri v2 애플리케이션이다. 같은 Rust 상태와 도메인 서비스 일부를 Axum 웹 서버도 노출한다.

```text
┌ React / TypeScript ───────────────────────────────────────┐
│ Router → Page/Widget/Feature → Query Hook → Command       │
│             Zustand state       TanStack Query cache       │
└──────────────────┬─────────────────────────────────────────┘
                   │ shared/api/transport.ts
            Tauri invoke │ authenticated/configured Web REST
┌──────────────────▼─────────────────────────────────────────┐
│ Tauri commands/*                  Axum server/*             │
│                  └──── AppState ─────┘                      │
│ broker/api → trading(strategy/order/risk) → storage        │
└──────────────┬───────────────────────────────┬──────────────┘
               │                               │
       KIS REST/WebSocket                Toss REST
               │                               │
       JSON 또는 PostgreSQL/MariaDB, Discord, Tauri events
```

Frontend는 provider credential이나 핵심 주문 상태의 권위자가 아니다. Rust `AppState`, provider 조회, durable store가 권위 경계다.

## 2. Frontend

`src/router/index.ts`가 dashboard, trading, strategy, history, log, settings route를 조립하고 `widgets/app-shell`이 공통 셸을 제공한다. 화면은 FSD 계층(`pages`, `widgets`, `features`, `entities`, `shared`)을 사용한다.

서버 데이터는 `src/api/hooks.ts`와 `databaseHooks.ts`의 TanStack Query로 읽고 변경한다. query key의 단일 진입점은 `src/api/queryKeys.ts`다. account/settings/trading UI 상태는 `entities/*/model`의 Zustand store가 담당한다.

`src/shared/api/transport.ts`는 실행 환경을 감지한다.

- Tauri: command name과 args를 `invoke`한다.
- Web: 정의된 REST route로 변환한다.
- DB credential/파괴 작업 같은 Tauri-only 기능은 웹 REST에 확장하지 않는다.

Backend push는 `src/api/backendEvents.ts`가 `exchange-rate-updated`, `exchange-rate-status-updated`, `balance-updated`, `overseas-balance-updated` 등을 구독하여 Query cache를 갱신한다.

## 3. IPC 계약

Rust command는 `src-tauri/src/commands/*.rs`에 도메인별로 있고 `commands.rs`가 `AppState`, 공통 타입과 facade를 제공한다. 실제 노출 목록은 `src-tauri/src/lib.rs`의 `tauri::generate_handler!`가 결정한다.

계약 변경의 원자적 단위:

1. Rust input/view와 serde 이름·optional 규칙
2. command 구현과 오류 코드
3. `lib.rs` handler 등록
4. `src/shared/api/types.ts` 및 관련 database/research 타입
5. `src/api/commands.ts` 또는 shared command wrapper
6. query hook/key/cache invalidation
7. Web 지원 시 transport route와 `server/*`
8. `docs/ipc-commands.md`

Rust `snake_case` 필드는 `#[serde(rename_all = "camelCase")]`를 통해 TS `camelCase`와 일치해야 한다. 신규 필드가 기존 persisted JSON과 호환되어야 하면 serde default/optional 정책을 명시한다.

## 4. AppState와 동시성

`lib.rs` setup은 설정·프로파일·DB manager를 읽고 `commands.rs::AppState`를 생성한다. 공유 객체는 `Arc<RwLock<_>>`, `Arc<Mutex<_>>`, watch channel로 전달된다.

핵심 원칙:

- lock 안에서는 snapshot/예약/상태 반영만 하고 provider network와 파일/DB await 동안 장기 보유하지 않는다.
- `OrderManager::submit_signal_shared`는 의존성 snapshot → preflight/prepare → 짧은 submission 예약 → provider 호출 → 성공/실패 반영 순서다.
- 여러 lock이 필요하면 기존 획득 순서를 확인한다. provider 호출을 lock 안으로 되돌리지 않는다.
- `data/.kisautotrade.lock` exclusive lock으로 같은 데이터 디렉터리의 중복 프로세스를 막는다.

## 5. Broker와 provider

`broker/domain.rs`가 `BrokerId`, `BrokerScope`, money/quantity/order 타입 등 공통 언어를 정의하고 `broker/adapter.rs`가 adapter 경계를 제공한다. KIS는 `api/rest.rs`, token/WebSocket과 `broker/kis.rs`를 사용하며 Toss는 `broker/toss/*`의 OAuth/HTTP/order 구현을 사용한다. `broker/rate_limit.rs`가 credential scope 기준 rate limiting을 담당한다.

`BrokerScope`의 broker/account 식별은 전략 저장, 실행 scope, 주문 충돌, pending, 체결, risk runtime 전 구간에서 유지되어야 한다. 프로파일 전환을 전역 문자열 변경으로 간주하면 안 된다.

## 6. 주문 파이프라인

수동 주문(`commands/orders.rs`)과 전략 신호는 공통 `OrderManager` 제출 서비스를 사용한다.

```text
UI/전략 Signal
 → 실행 BrokerScope와 현재 profile 검증
 → 수동 주문이면 holdings 재조회
 → 현재 position snapshot
 → provider preflight / 입력·통화·유동성 검증
 → RiskManager + TradeGuard
 → 같은 scope/symbol의 submitting·pending 충돌 검사
 → submission 예약
 → KIS/Toss provider 호출 (manager mutex 밖)
 → PendingOrder 및 OrderRecord 저장
 → scope별 risk 주문 카운트 저장
 → fills daemon/WebSocket reconciliation
 → TradeRecord/Stats/Risk/Position 갱신
```

`trading/preflight.rs`는 수량·가격 양수 여부, 수수료를 포함한 매수가능금액, 매도가능수량, 통화 일치를 검사한다. provider별 command가 얻은 constraints를 공통 판정에 연결한다.

`trading/order/conflicts.rs`와 submission 예약은 같은 scope/symbol의 중복·반대 방향 주문을 막는다. provider 접수 성공 뒤 pending/order/risk 영속화가 실패하면 `persistence_blocked`/`buy_suspended`를 설정해 신규 주문을 차단한다. 이 fail-closed 동작은 완화하지 않는다.

`trading/order/fills.rs`는 KIS/Toss pending 체결을 확인하고 체결, 수수료, 통계, trade store와 risk state를 갱신한다. provider order/request/TR trace는 credential을 제외한 최소 식별자만 저장한다.

## 7. 리스크 계층

`trading/risk.rs::RiskManager`는 다음을 구현한다.

- 일일 순손실 한도와 자동 비상정지
- 수동 비상정지(리스크 설정 비활성 상태에서도 유효)
- 종목당 최대 비중
- scope/전략/종목별 매도 주문 횟수
- scope/전략/종목별 연속 손실 신규 진입 차단
- 선택적 ATR 기반 수량 조정과 계좌 위험 한도

설정과 일별 runtime은 `storage/risk_store.rs`를 통해 분리 저장되며 재시작 후 복원된다. durable 체결 ledger로 일일 PnL을 재구축하는 경로도 있다. `commands/trading/risk_restore_tests.rs`와 risk 단위 테스트는 앱 재시작으로 한도를 우회하지 못하는지 확인하는 핵심 harness다.

비상정지 해제나 risk limit 완화는 단순 UI 설정 변경이 아니다. 자동매매 정지 상태, 현재 scope, pending 주문, durable runtime을 확인하고 명시적 운영 승인을 받아야 한다.

## 8. 전략과 preview

`trading/strategy/core.rs`가 `Signal`, config, trait와 warmup을 정의하고 `manager.rs`가 전략 생성을 조정한다. 실제 전략은 `strategy/*` 하위 모듈에 있다. live loop는 `trading/mod.rs`와 `commands/trading.rs`가 시세 tick을 전략에 전달한다.

`commands/strategy_preview.rs`와 `trading/simulation.rs`는 candle-close 기반 deterministic replay, 비용·환율·리스크 가정과 지표를 제공한다. preview는 live 10초 tick, intrabar, provider latency, 실제 체결을 재현하지 않으므로 실거래 승인 근거로 단독 사용하지 않는다.

## 9. 저장 경계

`storage::read_json_or_default`/`write_json` 및 각 store가 JSON과 DB backend의 공통 경계다. 도메인에서 직접 `tokio::fs` 저장을 추가하면 DB 활성 시 데이터가 분리될 수 있다.

- JSON: 실행 CWD의 `data/`, atomic temp/rename 패턴.
- DB: PostgreSQL/MariaDB document compatibility + schema v2 projection.
- backend 오류를 JSON으로 조용히 fallback하지 않는다.
- DB 전환은 import와 key 확인이 끝난 뒤 활성화한다.
- 프로파일, credential, 로그, DB password는 document import/export 대상이 아니다.

## 10. 백그라운드 작업

`lib.rs` setup 순서 기준:

1. KRX 종목 목록 로드
2. `run_trading_daemon`(플래그로 활성화)
3. Axum 웹 서버
4. USD/KRW 환율 갱신과 event
5. 로그/체결 retention 정리
6. KIS 잔고·해외잔고 갱신과 event

데몬 변경은 종료·비활성 전환 가능성, interval 변경, scope, lock, provider timeout/rate limit, 오류 시 fail-closed 상태 노출을 함께 검토한다.

## 11. 변경 도구와 검증

공용 타입·IPC·주문·리스크처럼 영향 범위가 넓을 때 정의/참조 누락 방지를 위해 Serena를 선택 사용할 수 있다. Graphify는 모듈 이동, helper 공용화, 호출 계층 재편 같은 구조 리팩터링 때만 사용한다. 검증 선택표는 [`HARNESS_MAP.md`](HARNESS_MAP.md)를 따른다.
