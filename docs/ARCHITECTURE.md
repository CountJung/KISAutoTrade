# KISAutoTrade 아키텍처

이 문서는 현재 코드에 구현된 경계와 고위험 데이터 흐름을 설명한다. 전체 파일 목록은 [`docs/project-map.md`](project-map.md), 작업별 진입점은 [`docs/PROJECT_MAP.md`](PROJECT_MAP.md)를 사용한다.

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

### 캔들 페이지 경계

`broker/candles.rs`는 KIS 날짜 anchor와 Toss inclusive timestamp cursor를
구분한다. `commands/candles.rs`의 읽기 전용 서비스는 Tauri와 Web GET이 같은
입력·camelCase view를 사용하도록 하며 활성 profile/broker/account와 커서의
scope/asOf를 검증한다. KIS는 공유 REST client의 계정·실전/모의 일치도 확인한다.
일·주·월봉은 서울/뉴욕 현재 시장 기간을 제외하고 분봉은 고정 asOf 이하 종료
시각의 실제 관측만 반환한다. OHLCV·통화 오류와 미완료 제외 수, 완료 정책을
응답에 구분한다. 이 기준은 거래소 달력·조기폐장 검증을 대신하지 않는다.

일반 preview의 `candlePageCollector.ts`는 최대 8회 요청·500봉 유지 범위에서
커서를 순차 진행하고 중복·무진전·종료·일부 오류를 처리한다. 요청 평가봉을
먼저 확보한 뒤 그 이전 최대 252봉을 사전자료로 나누며 자료가 부족하면 실제
범위를 표시한다. 첫 성공 페이지 이후 다른 거래소로 자료를 섞지 않고,
profile/scope/asOf/source가 바뀌면 결과를 폐기한다. 기존 Vec 차트 API와 전용
LTH 조회는 유지하며 live 완료 일봉 갱신은 이 경로의 책임이 아니다.

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

`trading/strategy/core.rs`가 `Signal`, config, trait와 warmup을 정의하고 `manager.rs`가 전략 생성을 조정한다. 실제 전략은 `strategy/*` 하위 모듈에 있다. live loop는 `trading/mod.rs`와 `commands/trading/daemon.rs`가 시세 tick을 전략에 전달한다. `commands/trading.rs`는 상태/시작·정지 IPC facade이며 positions/balance/risk_restore/history 하위 모듈이 포지션·평가액·당일 복원·legacy 사전자료를 맡고 daily_history가 52주 완료 일봉의 비동기 수집을 맡는다.

`commands/strategy_preview/generic.rs`와 `trading/simulation.rs`는 평가 구간과 이전 사전자료를 분리한 candle-close replay, 비용·환율·리스크 가정과 지표를 제공한다. `Strategy::history_readiness`는 실제 버퍼 상태, `can_evaluate_next_tick`은 해당 봉의 조건 평가 가능 여부를 보고한다. 과거 초기화는 직접 버퍼와 이전 지표를 복원하고 주문/포지션을 만들지 않는다. 기본 추세 200봉·신고가 252봉을 유지하며, generic LTH는 factory 실행 전에 `UNSUPPORTED_GENERIC_REPLAY`로 거부한다. 전용 LTH 경로는 `strategy_preview.rs`에 남아 있다. preview는 live 10초 tick, intrabar, provider latency, 실제 체결을 재현하지 않으므로 실거래 승인 근거로 단독 사용하지 않는다.

Generic 일봉 event 경로는 `Strategy`의 시가-only 날짜 시작 → 완성 OHLC 종가 tick → 실제 실행 결과의 포지션 동기화 → 완료봉 훅으로 흐른다. 강한 종가는 완료일 조건을 다음 평가일에 사용하고, 변동성 확장은 보유 상태를 유지한 채 당일 고저가를 초기화하고 이전 N개 완료 범위(0 포함)를 bounded ring으로 갱신한다. 두 전략은 `D`/`1d`만 지원한다. live quote polling은 완료 일봉 이벤트를 공급하지 않으며 기존 tick 경계를 유지한다. 신호와 체결은 여전히 동일 평가 봉 종가 근사다.

전용 LTH preview와 전략 v4는 일봉 context를 분봉 EMA/RSI/ADX·반동·볼린저 관측과 별도 bounded 버퍼에 보관한다. 분봉 OHLC initializer는 OHLC·반동·볼린저 snapshot을 교체하고 가격 initializer는 반동 가격 snapshot만 교체한다. 일봉 재초기화는 기존 분봉과 실제 보유 상태를 보존한다. `1d`는 전략·체결 경로를 실행하지 않는 `dailyDiagnostic/notEvaluable` 자료 진단이며 시가를 장중 진입 가격으로 합성하지 않는다. `1m`은 실제 관측을 입력 시각에 재생하는 최대 200봉 `intradaySample/sampleOnly`이고 유효 관측이 없으면 `notEvaluable`이다. `ReplayMetadata.assessment`는 일봉 context/분봉 수와 시간대·DST·세션 미검증을 기록하며, `warmupCount=0`으로 일봉을 분봉 지표 사전자료로 계산하지 않는다. 고정 KST 세션 근사 때문에 LTH `lookAheadSafe=false`이며 deterministic 계산만으로 시장 시각 정확성을 주장하지 않는다. UI는 일봉 및 assessment 없는 기존 LTH 응답의 성과/A-B 저장을 숨긴다. 실제 3개월 분봉·거래소 시간 검증은 P1-05 잔여 조건이다.


v9 generic 분봉은 offset 포함 RFC3339 또는 KST compact 시각을 고정 정밀도 UTC instant로 정규화해 정렬·중복·사전자료 겹침을 검사한다. `simulation.rs`와 공용 preview 차트도 RFC3339의 실제 instant를 우선 해석하며 관측 갭을 보간하지 않는다. 이 변경은 전용 LTH의 고정 KST 세션/DST 미검증 상태를 해소하지 않는다.

현재 replay의 `simulation/atr.rs`는 같은 간격의 이전 close를 포함한 TR14를 보관한다. `run_backtest_with_history`의 검증된 이전 자료는 ATR 지표만 준비하며 주문·equity에는 포함하지 않는다. 15관측/14TR 준비 상태를 `backtest.atrPreparation`으로 전략 준비와 구분한다. 연구 수량 산정 활성 시 부족/0변동폭/0bps는 신규 매수만 차단한다. 전용 LTH v4의 callback은 실제 잔여 수량·평균 체결가를 반환하여 ATR 수량 변경과 부분 청산 뒤에도 보호 상태를 유지한다. 기존 bool API는 raw 신호 연구용 호환 wrapper다.

52주 신고가의 완료 고가는 bounded ring으로 만료하며 generic 일봉 완료 이벤트 또는 live 완료 일봉 페이지에서 갱신한다. sampled 가격 tick을 완료 고가로 적재하지 않는다.

### Live 52주 사전자료와 실행 경계

`commands/trading/daily_history.rs`는 공통 완료 페이지로 최대 8회 요청·512봉을
수집한다. `daily_history/jobs.rs`는 같은 실행 profile/종목의 여러 52주 설정을
최대 요구량 1회 수집으로 공유하고 동시 2개 background job만 실행한다.
시세 loop는 provider future를 기다리지 않고 완료 handle만 수거한다.
실패/부족은 30~900초 backoff를 사용하고, 대상 제거·generation/config/scope/
시장 날짜 변경·정지 시 과거 작업을 취소하거나 결과 적용을 거부한다.

`strategy/manager/live_history.rs`는 실제 511대상과 초과 진단 1행을 유지한다.
실행 generation·전체 config fingerprint·profile/broker/account·종목·서울/뉴욕
시장 날짜가 일치하는 ticket에만 완료 OHLC를 적용하며 마지막 완료봉을 다시
제외하지 않는다. 최신 완료봉은 현재 시장 날짜보다 1~10일 이전이어야 하며,
긴 휴장도 보류하는 보수적 freshness 기준이다. 새 거래일에 신규 진입 준비를
비우며 quote 중 자정을 넘어도 tick과 같은 manager lock에서 재확인한다.
준비 부족·실패 중 보유 손절을 유지하고 LTH 분봉 상태는 건드리지 않는다.

IPC/Web 시작 준비 완료와 정지, window close는 일봉 generation·실행 identity를
갱신한다. 웹 상태도 이 identity와 동일한 camelCase `dailyHistory`를 반환한다.
Dashboard/Strategy의 `LiveDailyHistoryPanel`은 기존 상태 polling으로 확보량·
출처·날짜·실패/재시도 사유를 표시하고 최대 512항목 중 3행씩 렌더링한다.
활성 계좌와 실행 계좌가 다르면 상태를 섞지 않는다.

같은 ID/broker/account의 52주 설정 변경·동일 scope 저장 rollback은
`reconfigure_preserving_position` opt-in으로 잔존 대상의 보유 기준을 유지하고
관측 준비만 무효화한다. 다른 계정/제거 대상으로 보유 상태를 이전하지 않는다.
이 일봉 경계가 일반 전략 설정 재빌드/rollback이나 quote 이후 주문 예약의
전체 경합 안전성을 보장하지는 않는다. 그 기존 위험은 별도 P1 자동 과제다.

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

공용 타입·IPC·주문·리스크처럼 영향 범위가 넓을 때 정의/참조 누락 방지를 위해 Serena를 선택 사용할 수 있다. Graphify는 모듈 이동, helper 공용화, 호출 계층 재편 같은 구조 리팩터링 때만 사용한다. 검증 선택표는 [`docs/HARNESS_MAP.md`](HARNESS_MAP.md)를 따른다.


## 자동매매 예산 경계

`trading/order/budget.rs`는 broker/account와 KRW/USD별 배정·현금·소유수량·예약 원장을 담당한다. `orders/auto_trading_budget.json`을 공통 managed storage 경계로 읽고 쓴다. 과거 백업으로 자금이 되돌아가지 않도록 원장 JSON 파싱 실패 시 자동 복구를 하지 않는다.

`submit_order_shared`는 manager 잠금 내에서 예산 intent를 영속화한 뒤 provider에 제출한다. 부분체결은 누적 수량/금액으로 멱등 적용하고, terminal 표식을 pending 제거 이전에 저장해 중간 종료를 복구한다. 확정 주문 거절만 예약을 해제하며 transport/파싱 실패와 애매한 HTTP 결과는 예약을 유지한다. 매수만 지정가로 제한하고 기존 자동매도 주문 유형을 보존한다. 예산은 위험 관리 비활성화로 우회되지 않는다.

`leveraged_trend_hold/bollinger.rs`는 기존 전략 내부 필터다. 별도의 분봉 버퍼로 일봉 warmup 혼합을 막고, 세 진입 경로와 기존 청산 처리에 연결된다. 새로운 strategy ID는 없다.
