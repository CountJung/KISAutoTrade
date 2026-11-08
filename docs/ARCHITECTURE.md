# KISAutoTrade 아키텍처

이 문서는 현재 코드에 구현된 경계와 고위험 데이터 흐름을 설명한다. 전체 파일 목록은 [`docs/project-map.md`](project-map.md), 작업별 진입점은 [`docs/PROJECT_MAP.md`](PROJECT_MAP.md)를 사용한다.

## 1. 시스템 경계

KISAutoTrade는 React 19 UI와 Rust backend가 결합된 Tauri v2 애플리케이션이다. 같은 Rust 상태와 도메인 서비스 일부를 Axum 웹 서버도 노출한다.

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

Settings의 목차·목차 복귀는 `settingsNavigation.tsx::focusSettingsTarget`에서 경로별 복원 취소 알림과 `preventScroll` focus를 유지하고, 가장 가까운 `app-main-scroll`만 상대좌표·border·기존 scroll margin으로 이동한다. main의 끝에서 자연스럽게 clamp되며 상위 `overflow:hidden` 셸까지 이동해 상단 정렬을 강요하지 않는다. main 밖의 독립 렌더는 기존 `scrollIntoView` fallback을 유지한다. 명시적 목적지가 포커스를 유지하는 동안 DOM·크기 변경 관측을 예약 프레임 하나로 합쳐 main 안의 가시성만 재검사한다. wheel/touch/pointer·스크롤 키·새 focus/navigation·페이지 이탈은 이 추적을 취소하며 observer/listener/예약 프레임을 정리한다. 보정은 focus를 다시 강요하지 않고 연결 상태·현재 focus·경로를 재검사한다. AppShell의 같은 경로 복원 취소·다른 경로 보호·사용자 의도 경계는 바꾸지 않는다.

서버 데이터는 `src/api/hooks.ts`와 `databaseHooks.ts`의 TanStack Query로 읽고 변경한다. query key의 단일 진입점은 `src/api/queryKeys.ts`다. account/settings/trading UI 상태는 `entities/*/model`의 Zustand store가 담당한다.

`src/shared/api/transport.ts`는 실행 환경을 감지한다.

- Tauri: command name과 args를 `invoke`한다.
- Web: 정의된 REST route로 변환한다.
- DB credential/파괴 작업 같은 Tauri-only 기능은 웹 REST에 확장하지 않는다.

Backend push는 `src/api/backendEvents.ts`가 구독한다. 환율은 전역 캐시를 갱신하고, 잔고 이벤트는 현재 계좌 쿼리를 무효화해 재조회한다. 잔고 payload를 캐시에 직접 넣지 않는다. 국내·해외·broker holdings는 profile/broker/account/투자 모드별 키를 사용한다.

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

자동매매는 OrderManager에 epoch·configuration revision·broker/account·profile context와 검증한 client/profile snapshot을 저장한다. daemon의 시세·평가액은 같은 snapshot을 사용하고, 전략 변경은 bool→strategy→order 검증 구간에서만 수행한다. 자동 예약은 같은 OrderManager mutex에서 context를 검사한 다음 guard·submitting·예산을 변경한다. 설정 저장 성공 후 manager→order 임계구역에서 새 revision을 공개하고, 저장 실패는 기존 revision을 유지한다. provider await에는 bool/strategy lock을 유지하지 않는다. 예약 후에는 최초 client/scope로 응답·재시도·pending을 끝까지 추적하며 정지나 자정 초기화로 submitting을 지우지 않는다. 세부 검증은 [P1-RUN-01 기록](completed-history.md#task-execution-run-reservation-20261018)을 참조한다.

```text
UI/전략 Signal
 → 실행 BrokerScope와 현재 profile 검증
 → 수동 주문이면 holdings 재조회
 → 계좌 관측 snapshot + exact owner 수량·원가
 → provider preflight / 입력·통화·유동성 검증
 → RiskManager + TradeGuard
 → 같은 scope/symbol의 submitting·pending 충돌 검사
 → submission 예약
 → KIS/Toss provider 호출 (manager mutex 밖)
 → PendingOrder 및 OrderRecord 저장
 → scope별 risk 주문 카운트 저장
 → fills daemon/WebSocket reconciliation
 → 소유 원장/TradeRecord/Stats/Risk 갱신
 → 소유 전략 피드백 + 해당 시장 계좌 snapshot 재조회
```

`trading/preflight.rs`는 수량·가격 양수 여부, 수수료를 포함한 매수가능금액, 매도가능수량, 통화 일치를 검사한다. provider별 command가 얻은 constraints를 공통 판정에 연결한다.

`trading/order/conflicts.rs`와 submission 예약은 같은 scope/symbol의 중복·반대 방향 주문을 막는다. provider 접수 성공 뒤 pending/order/risk 영속화가 실패하면 `persistence_blocked`/`buy_suspended`를 설정해 신규 주문을 차단한다. 이 fail-closed 동작은 완화하지 않는다.

`trading/order/fills.rs`는 KIS/Toss pending 체결을 확인하고 체결, 수수료, 통계, trade store와 risk state를 갱신한다. provider order/request/TR trace는 credential을 제외한 최소 식별자만 저장한다.

`order/ownership.rs`는 broker/account/profile/투자모드·전략·시장·종목별 확정 수량과 정수 원가의 권위자다. `position_ownership.json`과 별도 generation manifest를 검증하며 pending/order record와 연결한다. 자동매도는 owner 수량과 계좌 관측 수량의 교집합을 예약 직전에 다시 확인한다. 계좌 tracker는 broker snapshot의 권위만 유지하고 fill delta로 변경하지 않는다. 체결 뒤 해당 시장 재조회가 성공하기 전 신규 예약을 보류한다.

`order/feedback.rs`가 manager → order lock 순서로 정확한 전략에만 수량·pending·보호 metadata를 전달한다. live 관측은 지표·LTH 최고가/보유 관측 수를 유지하지만 raw 신호만으로 확정 보유를 바꾸지 않는다. 전체 계좌 보유를 모든 전략에 broadcast하지 않는다. 원가·세대·모드 미상 기록과 저장 실패는 신규 주문을 차단한다. 리스크 재복원은 동일 당일 체결 이벤트·계좌·투자모드를 검증하고 후보를 저장한 뒤 공개한다. 상세 검증은 [P1-FILL-01 기록](completed-history.md#task-strategy-fill-ownership-20261018)을 참조한다.

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

<a id="p2-ux-07-action-matrix"></a>
### 위험 행동의 현행 영향표 — P2-UX-07

`features/financial-confirmation`의 단일 Provider가 Sidebar/Dashboard와 Trading의 확인·전송 잠금을 공유하며 기존 mutation 훅을 호출한다. 아래 표는 UI 확인과 기존 backend 경계의 역할을 설명한다. backend 설명은 코드 추론이며 실제 Rust IPC/native 실행 증거가 아니다. 변경 경과·실제 검사·미검증 범위의 정본은 [P2-UX-07 이력](completed-history.md#p2-ux-07)이다.

| 행동·진입점 | 현재 UI 확인·pending | backend 경계·영향 |
|---|---|---|
| 자동매매 시작: [Sidebar L66](../src/widgets/sidebar/ui/Sidebar.tsx#L66), [Dashboard L491](../src/pages/dashboard/ui/Page.tsx#L491) → `start_trading` | 실전(KIS 실전·Toss)은 scope/영향 최종 확인. KIS 모의 시작은 request 시 캡처한 신뢰 가능한 config가 KIS 모의일 때만 즉시 요청. 단일 동기 잠금으로 Sidebar/Dashboard 중복 요청을 차단한다. | [trading.rs L100](../src-tauri/src/commands/trading.rs#L100): 설정·저장소/Toss 저장 동의, 보유/평가액 동기화·pending 대조·risk 복원, profile/client snapshot과 execution scope 생성 후 daemon 실행 상태를 켠다. 이후 주문 가능한 상태이며 비상정지 해제와 별개다. |
| 자동매매 정지: 같은 Sidebar, [Dashboard L480](../src/pages/dashboard/ui/Page.tsx#L480) → `stop_trading` | 추가 확인 없이 기존 정지 요청. 실행 중 Sidebar는 정지 pending만 차단하며 확인창 안에도 즉시 정지 진입점이 있다. | [trading.rs L420](../src-tauri/src/commands/trading.rs#L420): 실행 flag·history generation·execution run을 끝내고 실행 프로필 식별자를 비운다. 이 함수는 기접수 주문 취소·보유 청산을 호출하지 않으며 이미 예약한 주문의 원래 scope 추적은 유지한다. |
| 비상정지 발동: [Dashboard RiskPanel L254](../src/pages/dashboard/ui/Page.tsx#L254) → `activate_emergency_stop` | 추가 확인 없이 기존 비상정지 요청. 확인창 안에서도 즉시 발동하며 미전송 승인은 무효화한다. | [risk.rs L209](../src-tauri/src/commands/risk.rs#L209): emergency flag와 runtime 저장. [주문 prepare](../src-tauri/src/trading/order/submission/prepare.rs#L65)에서 신규 매수·매도를 차단하며 daemon 실행 flag 변경이나 기접수 주문 취소와 다르다. |
| 비상정지 해제: [Dashboard L122](../src/pages/dashboard/ui/Page.tsx#L122) → `clear_emergency_stop` | 현재 broker/계좌/프로필/모드와 해제 영향을 확인한다. daemon 시작·손실 카운터 초기화가 아님을 표시한다. | [risk.rs L200](../src-tauri/src/commands/risk.rs#L200): flag=false와 저장. [RiskManager L544](../src-tauri/src/trading/risk.rs#L544)는 손실 카운터를 초기화하지 않아 한도 초과 시 can_trade=false가 유지될 수 있다. daemon 시작·매수 정지 해제는 하지 않는다. 저장 실패 IPC 오류와 이미 변경한 메모리 flag를 구분한다. |
| 매수 재개: [Dashboard L520](../src/pages/dashboard/ui/Page.tsx#L520), [Trading L205](../src/pages/trading/ui/tradingOrderForm.tsx#L205) → `clear_buy_suspension` | Dashboard/Trading이 공용 확인·잠금을 사용한다. 입금/잔고 검증·비상정지 해제·daemon 시작이 아님을 표시한다. | [trading.rs L482](../src-tauri/src/commands/trading.rs#L482) → [health.rs L97](../src-tauri/src/trading/order/health.rs#L97): persistenceBlocked이면 해제하지 않는다. 그 외 매수 정지와 사유만 지운다. 입금·잔고 재조회·비상정지 해제·daemon 시작은 하지 않는다. |
| KIS 국내 수동 매수/매도: [Trading L222](../src/pages/trading/ui/tradingOrderForm.tsx#L222) → `place_order` | 종목·매수매도·수량·실제 유형/가격(KRW)을 최종 확인한다. 기존 입력 검사와 backend guard는 보존한다. | [orders.rs L20](../src-tauri/src/commands/orders.rs#L20): 방향/유형·현재가/평가액·KIS account scope 확인 후 공통 제출. 성공 응답은 provider 접수이며 체결 완료 인증이 아니다. |
| KIS 미국 수동 매수/매도: 같은 Trading → `place_overseas_order` | 종목·매수매도·수량·지정가 USD/거래소를 최종 확인하고 기존 해외 모의 제한을 재검사한다. | [market.rs L351](../src-tauri/src/commands/market.rs#L351): 현재가·계좌평가액·KIS scope와 Limit 주문으로 공통 제출. 모의 제한 UI 확인을 provider 성공·정규장 허용 인증으로 확대하지 않는다. |
| Toss 국내/미국 수동 매수/매도: 같은 Trading → `place_order` Toss 분기 | 국내/미국 종목·방향·수량·유형/가격·미국 세션을 최종 확인한다. 기존 동의/preflight/세션 검사를 최종 실행 때 다시 통과한다. | [orders.rs L130](../src-tauri/src/commands/orders.rs#L130): 동의/계좌 설정·제출 직전 preflight·scope별 local pending·선택 세션/provider open orders·계좌평가액 확인 후 공통 제출. 운영 기능 추적이며 정규장만의 모의 연구 설정 변경이 아니다. |

수동주문 공통 [submission.rs L294](../src-tauri/src/trading/order/submission.rs#L294)는 설정 저장 gate·실행 scope/profile/client snapshot과 준비 후 snapshot을 대조한다. 최종 예약은 persistence 장애·보유 재확인·매수 정지·TradeGuard·scope별 pending/submitting 충돌을 검사한다(L750–814). 매수는 일일손실/비상정지·최대비중, 매도는 확인된 보유·비상정지를 검사한다. 자동매매 예산 예약은 `!is_manual` 분기이므로 모든 수동주문에 자동예산 예약이 적용된다고 설명하지 않는다.

확인 토큰은 AppConfig `dataUpdateCount`와 broker/account/profile/name/모드, router `onBeforeNavigate` 세대를 고정한다. 브라우저 back/forward로 원래 경로에 돌아와도 이전 승인은 되살아나지 않는다. 모의 시작 생략 여부도 같은 request에서 캡처한 신뢰 config로만 결정한다. config fetch/error/invalidated·프로필 publication pending·공개세대 변경 또는 주문 초안 generation 변경은 이전 확인을 영구 무효화한다. 최종 확인은 동기 소비 후 기존 mutation을 정확히 한 번 호출하며 오류 재시도는 닫기→새 확인이 필요하다. Cancel/Escape/배경 닫기는 전송하지 않고 일반 입력/IME·반복·modifier 금융키를 자동 제출로 쓰지 않는다. 순수 Simulation 계산은 이 흐름을 호출하지 않는다. 이 frontend 공개세대는 P1의 미해결 server ABA/원장 모드 이관을 해결하지 않는다.

확인창은 busy로 opener가 비활성화되기 전에 해당 요소를 캡처하고, 취소에 초기 focus를 준 뒤 transition 종료 후 같은 opener로 복귀한다. 처리 중에도 즉시 정지/비상정지 요청은 가능하지만 이미 전송한 요청을 취소하지 않는다. focused mock 계약은 각 행동 전 exact origin/method/path/noquery/body(무본문 포함) 요청열을 선언하고 최종 전체 배열을 비교한다. broker/profile/account/paper는 요청 시점의 **브라우저 캐시 관측 메타데이터**이며 Dialog의 broker·프로필명·계좌·모드와 대조한다. start/정지/위험 해제 API는 무본문이므로 서버로 identity를 전송한 증거가 아니다. 잘못된 account/path/method/body/query/origin은 HTTP403과 거부 기록으로 검사한다. 해제의 비대상 상태 보존은 HTTP fake를 사용한 frontend 검증이고 실제 backend 위험정책 검증은 아니다. 확장 회귀·375px/dark는 mock 관측을 마쳤으며 native/실계좌 검증은 남았다. [경과·부모 명령](completed-history.md#p2-ux-07).

## 8. 전략과 preview

`trading/strategy/core.rs`가 `Signal`, config, trait와 warmup을 정의하고 `manager.rs`가 전략 생성을 조정한다. 실제 전략은 `strategy/*` 하위 모듈에 있다. live loop는 `trading/mod.rs`와 `commands/trading/daemon.rs`가 시세 tick을 전략에 전달한다. `commands/trading.rs`는 상태/시작·정지 IPC facade이며 positions/balance/risk_restore/history 하위 모듈이 포지션·평가액·당일 복원·legacy 사전자료를 맡고 daily_history가 52주 완료 일봉의 비동기 수집을 맡는다.

`commands/strategy_preview/generic.rs`와 `trading/simulation.rs`는 평가 구간과 이전 사전자료를 분리한 candle-close replay, 비용·환율·리스크 가정과 지표를 제공한다. `Strategy::history_readiness`는 실제 버퍼 상태, `can_evaluate_next_tick`은 해당 봉의 조건 평가 가능 여부를 보고한다. 과거 초기화는 직접 버퍼와 이전 지표를 복원하고 주문/포지션을 만들지 않는다. 기본 추세 200봉·신고가 252봉을 유지하며, generic LTH는 factory 실행 전에 `UNSUPPORTED_GENERIC_REPLAY`로 거부한다. 전용 LTH 공개 facade는 `strategy_preview.rs`이며 자료 연결은 `strategy_preview/lth_fetch.rs`가 소유한다. preview는 live 10초 tick, intrabar, provider latency, 실제 체결을 재현하지 않으므로 실거래 승인 근거로 단독 사용하지 않는다.

v12 generic의 단기 `preview_strategy`와 새 `preview_strategy_continuous`는 `strategy_preview/driver.rs`의 공통 실행기를 사용한다. `continuous.rs`가 history·분할 평가의 전체 시각/상한·시나리오를 먼저 검증한 뒤 전략 factory·warmup·세션을 한 번 만들고 전체 append 후 finish한다. 서버 session/cache를 만들지 않는 단일 요청이며 공급 묶음·분기 경계에서 포지션·원가·pending·리스크를 초기화하지 않는다. 일일 경계는 event 날짜를 따르며 마지막 미체결 확정과 보고 전용 가상 청산은 전체 종료에서만 수행한다. 기존 flat 합계500봉은 유지하고 연속 입력은 history512·16묶음·묶음당500·전체 평가4096으로 제한한다. LTH는 두 generic 경로 모두 거부한다.

연속 IPC와 REST `/api/strategy/preview/continuous`는 같은 입력이며 연속 응답은 기존 preview 최상위에 `continuity` 공급 원장을 추가한다. hash에는 계산 모드를 넣고 동일 경제 입력의 chunk 분할은 제외한다. UI의 `continuousReplayPanel.tsx`·`candlePageCollector.ts`는 일봉 날짜 범위와 strict 이전252봉을 수집하고48회 요청·보관4608/평가4096을 제한한다. 요청 경계 차이·관측 간격7달력일 보수 gate는 거래소 달력 검증이 아니며 `coverageVerified=false`가 유지된다. A/B 모드·원장 보존과 legacy unknown, 같은 장부 앞/뒤 구간 표시를 포함한 상세 계약은 [IPC 정본](ipc-commands.md)의 v12 항목을 따른다.

Generic 일봉 event 경로는 `Strategy`의 시가-only 날짜 시작 → 완성 OHLC 종가 tick → 실제 실행 결과의 포지션 동기화 → 완료봉 훅으로 흐른다. 강한 종가는 완료일 조건을 다음 평가일에 사용하고, 변동성 확장은 보유 상태를 유지한 채 당일 고저가를 초기화하고 이전 N개 완료 범위(0 포함)를 bounded ring으로 갱신한다. 두 전략은 `D`/`1d`만 지원한다. live quote polling은 완료 일봉 이벤트를 공급하지 않으며 기존 tick 경계를 유지한다. 신호는 완료 종가에서 판단하고 체결은 공통 세션의 sameClose/nextOpen 가정에 따른다.

US1m의 자료 흐름은 `strategy_preview.rs` → `lth_fetch.rs` → `lth_collection.rs` → `lth_regular.rs`이며 KR/일봉의 기존 tuple 조회 호환 경로는 유지한다. `lth_research.rs`·`lth_runtime.rs`는 요청 소유권·예약/단일 absolute deadline·취소 수명을, `lth_collection.rs`는 bounded 수집·출처·통계와 단일 `Summary` 타입을 소유한다. 성공 summary만 같은 preview View를 통해 IPC/REST·TS에 전달한다. `useLthResearchJob.ts`는 UI 작업/입력 세대와 늦은 응답 차단을, `lthCollectionSummaryPanel.tsx`는 받은 summary의 구조 검증·표시를 소유하며 통계를 재계산하지 않는다. 실패 시 내부 partial Summary는 공개하지 않고 현행 오류 계약을 유지한다. 정확한 wire 필드와 raw/typed/engine hash의 구분은 [IPC 정본](ipc-commands.md)을 따른다. 수집 metadata는 실제 provider 발행/PIT·조정·기간 coverage 인증이 아니다.

전용 LTH preview는 일봉 context를 분봉 EMA/RSI/ADX·반동·볼린저 관측과 별도 bounded 버퍼에 보관한다. 분봉 OHLC initializer는 OHLC·반동·볼린저 snapshot을 교체하고 가격 initializer는 반동 가격 snapshot만 교체한다. 일봉 재초기화는 기존 분봉과 실제 보유 상태를 보존한다. `1d`는 전략·체결 경로를 실행하지 않는 `dailyDiagnostic/notEvaluable` 자료 진단이며 시가를 장중 진입 가격으로 합성하지 않는다. `1m` 평가는 최대 200봉 `intradaySample/sampleOnly`이며 US1m의 다중 페이지 수집 상한과 평가 채택 수를 구분하고 유효 관측이 없으면 `notEvaluable`이다. 미국 연구 v5는 `lth_regular::prepare`에서 실제 instant 정렬·중복 전체 제외·단일 cutoff 및 NY 계획 정규장 안의 종료 표기 봉 전체 구간을 먼저 검증한다. 같은 채택 순서/인덱스가 Open→분봉 지표/LTH→ReplaySession과 원본 라벨/OHLCV 차트에 흐르며, 이전 NY 거래일의 일봉만 별도 context로 사용한다. `research_session`이 NY/DST·2023–2028 계획 달력과 알려진 Carter 예외를 소유하고 horizon 밖은 unknown으로 제외한다. blackout 설정은 legacy Asia/Seoul 분이며 KR/일봉 v4·live 세션 설정은 유지한다. `ReplayMetadata.assessment`의 시각/세션 unverified·`warmupCount=0`·`lookAheadSafe=false`는 유지한다. 계획 달력 적용은 실제 provider 원본/offset·발행·정규장 전용 일봉 구성·모든 특별휴장/기간 coverage 인증이 아니다. UI는 일봉 및 assessment 없는 기존 LTH 응답의 성과/A-B 저장을 숨긴다. 실제 3개월 검증은 P1-05 잔여 조건이다.


v9 generic 분봉은 offset 포함 RFC3339 또는 KST compact 시각을 고정 정밀도 UTC instant로 정규화해 정렬·중복·사전자료 겹침을 검사한다. `simulation.rs`와 공용 preview 차트도 RFC3339의 실제 instant를 우선 해석하며 관측 갭을 보간하지 않는다. 구문 정규화와 위 US LTH 연구의 세션 채택 정책은 별도 책임이며 실제 provider 시각/coverage 미검증을 해소하지 않는다.

현재 replay의 `simulation/atr.rs`는 같은 간격의 이전 close를 포함한 TR14를 보관한다. `run_backtest_with_history`의 검증된 이전 자료는 ATR 지표만 준비하며 주문·equity에는 포함하지 않는다. 15관측/14TR 준비 상태를 `backtest.atrPreparation`으로 전략 준비와 구분한다. 연구 수량 산정 활성 시 부족/0변동폭/0bps는 신규 매수만 차단한다. 전용 LTH v4의 callback은 실제 잔여 수량·평균 체결가를 반환하여 ATR 수량 변경과 부분 청산 뒤에도 보호 상태를 유지한다. 기존 bool API는 raw 신호 연구용 호환 wrapper다.

v11 공통 실행 회계는 `simulation/accounting.rs`의 입력 검증, `simulation/session/accounting.rs`의 청산 전망·순손익·기여 분해와 `simulation/comparison.rs`의 비교 보고로 분리한다. `simulation/session.rs`는 봉 시작의 FX 변경·직전 보유 배당과 봉말 보유 평가액을 처리하고 native 평균 체결가는 환율 변화와 분리해 유지한다. 현금 장부는 KRW이며 배당 시나리오는 실제 브로커 기업행동·USD 유휴 현금 장부를 구현하지 않는다. 최소 수수료가 있는 일반 이익 청산은 실제 청산 전망과 같은 수량·과거 매수 비용 배분·현재 FX의 순익을 검사하며 손절/위험 축소 예외를 유지한다.

보고는 실현·잔여 평가·순배당으로 자산 증감을 설명하고 종료 시 가상 청산을 상태 변경과 분리한다. 평균/최대 투자 비중은 관측 봉말의 평가액이며 보유 봉 비율·시간 가중 노출과 다르다. 같은 입력 수량의 단순 보유는 실제 체결 수량·노출 차이를 공개하고, 같은 체결 경로의 비용 제거 값은 사후 원화 환급 산술로만 제공한다. 가격 자료의 배당 조정 선언·수동 비용 확인·정규화 후 정확한 평가 봉 일치 및 legacy 보관 집계의 계약은 [IPC 정본](ipc-commands.md)의 v11 항목을 따른다. LTH 일봉 자료 진단은 이 성과·비교 경로를 실행하지 않는다.

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

13개 전략은 같은 ID/broker/account의 `reconfigure_preserving_position`으로
잔존 대상의 보유 기준과 필요한 관측을 유지하며 새 파라미터를 적용한다.
LTH는 진입가·최고가·보유 관측 수를 그대로 유지하고, 평균회귀는 새 기간의
밴드 준비 전에도 기존 보유 손절을 평가한다. 프로파일 publication은 같은
계좌 문자열이라도 인스턴스를 새로 만들어 이전 프로파일 상태를 넘기지 않는다.

`trading/configuration.rs`는 IPC와 Web이 공유하는 저장 transaction이다.
`strategy_update_lock`으로 설정/시작/프로파일 변경을 직렬화하고,
manager→order 아래 private 보유·실제 tracker·submitting/pending/modifying·
profile/client identity를 검사한 뒤 Weak lease를 공개한다. lease 동안 같은
scope의 새 전략 신호와 영향 종목 예약·수동 전체 잔고 게시를 보류한다.
파일/DB 저장 await에는 manager/order lock을 유지하지 않는다. 저장 성공 후
identity를 다시 검사하고 기존 인스턴스와 새 revision을 함께 공개한다.
저장 실패는 runtime/revision을 바꾸지 않으며, 첫 저장 오류도 디스크 교체 후
오류일 수 있으므로 저장 후 검증 실패와 함께 원본 설정을 다시 저장한다. 복구 저장도 실패하면 persistence block으로 신규 주문을 막는다.
IPC/HTTP waiter 취소는 별도의 owned transaction task를 중단하지 않는다.

공용 tracker의 0수량만으로 전략의 private 보유 보호를 지우거나 다른 전략의
보유를 할당하지 않는다. private 보호 또는 실제 tracker에 보유가 있으면 대상
제거·국내/해외 전환을 거부한다. 정상 체결의 전략별 소유권 피드백은 위 주문
파이프라인의 별도 원장·feedback 경계를 사용한다.

### 잔고 응답과 포지션 게시

`order/holdings.rs`의 `HoldingsQuery`는 조회 시작 시 broker/account·profile·
credential/mode·client Arc·실행 epoch·설정 lease identity를 고정한다.
국내/해외 tracker는 고정 크기 UUID로 마지막 발급 조회와 수량 변경을 기록한다.
매수·매도·전체 교체·clear·tracker 재생성은 이전 응답을 무효화하고 현재가만
바뀐 경우는 수량 조회를 버리지 않는다. 조회·파싱·종목명 저장은 잠금 밖이다.
게시 직전 OM gate에서 identity와 두 시장의 버전을 모두 확인한 뒤 교체한다.
Both 응답은 한 시장이 무효이면 어느 쪽도 반영하지 않는다. 유효한 빈 목록과
0수량은 실제 청산/삭제로 처리하며 반대 시장을 지우지 않는다.

일반 IPC, KIS background, 수동 주문 준비, IPC/Web 시작 동기화가 같은 게시
경계를 사용한다. 시작은 manager→OM을 유지한 채 게시와 전략 복원을 마친다.
Web 일반 잔고와 IPC/Web broker holdings는 ReadOnly ticket으로 계좌 연속성만
확인하며 tracker에 부작용을 추가하지 않는다. background 성공 이벤트도 게시와
같은 OM gate에서 발행하고 거절된 응답은 이벤트를 발행하지 않는다.

프런트엔드는 이벤트를 현재 계좌 재조회 힌트로 취급한다. 계좌별 쿼리는 이전
scope의 payload/placeholder를 넘기지 않고 요청 시작·완료 때 config를 확인한다.
프로파일 전환은 이전 config 요청과 보유 쿼리를 취소하고 이전 범위 캐시를 제거한다.
새 config를 확인하지 못한 상태에서 이전 계좌 잔고를 현재 자료로 표시하지 않는다.
예산·복구 조회도 같은 설정 신뢰·취소/캐시 제거 경계를 사용한다. 기존 key 뒤에 AppConfig의 성공 공개 횟수와 profile/broker/account/모드를 붙이며, 같은 scope의 설정 복구도 새 공개 후 조회가 성공해야 금액을 표시한다. Settings의 설정 재점검은 AppConfig와 진단을 함께 재조회한다. 예산 저장은 요청 시 공개 횟수를 캡처하며, 같은 공개의 후속 GET 성공 뒤에만 성공/초안 초기화를 허용한다. 재조회 실패는 저장 반영 가능성을 알리고 초안을 유지하며, 이전 공개의 응답은 현재 금액을 무효화하지 않는다. 복구 조회의 무폴링·명시 검토 정책은 유지한다. Settings의 계좌 선택은 전환 pending·실행 중·거래/설정 미확인에서 차단한다.
증권사 잔고의 체결 반영 지연은 남아 있다. 정상 체결 소유권은 위 별도 원장과
feedback으로 다루며 계좌 게시 gate만으로 이를 대신하지 않는다.

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

예산 저장 입력의 `expectedProfileId`·`expectedIsPaper`는 필수다. IPC/REST 모두 storage→selection lock 획득 후 선택 broker/account/profile/mode를 검증하고 원장을 변경한다. 누락 payload는 역직렬화 단계에서 거부한다. 이 검증은 처리 전에 지연된 paper 요청이 live 선택에 적용되는 것을 막지만, 원장 자체의 모드 귀속을 제공하지 않는다. 기존 원장은 broker/account 단위다. 새 `is_paper` 영속화와 legacy 유자금 차단은 수동 매도 회귀 때문에 철회했다. 모드 없는 자금의 소유/모드를 입증하는 이관과 모드별 원장 격리는 미해결이며 현재 선택으로 추정·삭제·자동 이관하지 않는다.

`submit_order_shared`는 manager 잠금 내에서 예산 intent를 영속화한 뒤 provider에 제출한다. 부분체결은 누적 수량/금액으로 멱등 적용하고, terminal 표식을 pending 제거 이전에 저장해 중간 종료를 복구한다. 확정 주문 거절만 예약을 해제하며 transport/파싱 실패와 애매한 HTTP 결과는 예약을 유지한다. 매수만 지정가로 제한하고 기존 자동매도 주문 유형을 보존한다. 예산은 위험 관리 비활성화로 우회되지 않는다.

`leveraged_trend_hold/bollinger.rs`는 기존 전략 내부 필터다. 별도의 분봉 버퍼로 일봉 warmup 혼합을 막고, 세 진입 경로와 기존 청산 처리에 연결된다. 새로운 strategy ID는 없다.


v13 Consecutive는 가격 관측열/확정 보유와 `sequence/consecutive/context.rs`의 완료 일봉 맥락을 분리한다. coordinator는 `required_completed_daily_bars` opt-in 및 apply/invalidate/readiness 훅으로 기존 52주 고가와 선택 ATR/MA 진입 확인을 공급한다. 일봉 갱신·실패·날짜 전환으로 가격 관측열과 보유 청산을 지우지 않는다. legacy warmup은 scoped 완료 일봉 인증을 대신하지 않는다. generic driver는 현재 후보 평가/피드백 뒤에만 OHLC를 완료 맥락으로 게시한다. 선택 `entryFilterPreparation`과 수량 조절 ATR은 별도이며 전 필터0이면 기존 경로를 유지한다. 계약은 [IPC v13](ipc-commands.md), 인과/기본 호환·후보 기각은 [P2-06 연구](research/consecutive-filters-20261018/report.md)에 기록한다.


v14 Momentum은 classic/momentum.rs의 owned_quantity를 가격 관측/previous momentum/무장/미체결 후보와 분리한다. raw tick으로 소유량을 선반영하지 않으며 exact owner 피드백에서만 갱신한다. 기본 level은 flat Buy/held Sell, crossing/간격%p는 entry-only이고 보유 negative level은 유지한다. 같은0수량·nextOpen pending과 부분체결에서 후보/청산을 잃지 않으며 설정이 바뀌면 보유/잔존가격을 유지하고 관측 상태를 재seed한다. 기존 save/short/continuous validator와 UI params/AB를 사용해 새endpoint/DTO를 만들지 않는다. 숫자 lexical draft는 shared/lib 공개 parser를 사용하고 전략 범위/mode 관계는 controls에 남긴다. 계약은 [IPC v14](ipc-commands.md), 생산 대조는 [P2-03 연구](research/momentum-feedback-20261020/report.md)다.


v15 Deviation은 classic/deviation.rs와 params/state 모듈에서 관측·하향 교차 후보·live 전용 추세 버퍼를 exact owner 수량/평균가/보유 나이와 분리한다. 기존 guard/risk·owner clamp 경계를 유지하고 metadata를 통해 보호 관측을 복원한다. generic warmup과 live 추세 cadence를 혼합하지 않으며 진입 준비 부족 중에도 held stop/maxHold을 평가한다. 기존 params 저장/두 preview/UI/A-B를 사용하고 새 endpoint/DTO는 없다. [계약](ipc-commands.md), [연구](research/deviation-policy-20261020/report.md), [작업 기록](completed-history.md#task-deviation-policy-20261020)을 참조한다.


v16 MeanReversion은 mean_trend facade 아래 독립 모듈로 분리하고 actual q/native avg·기록 나이와 현재 포함 band/선택 청산·live전용 trend를 구분한다. 당시 TrendFilter 원형을 유지하며 observation.rs가 두 전략의 동등한8개 기계 동작만 제공한다. 이후 TrendFilter의 실제 소유·선택 보호 경계는 아래 절을 따른다. kind/scope/대상·stop0/strict/inclusive·reset/warmup 차이는 전략에 남는다. 기존 owner ledger/관측후 checkpoint·pending복원/저장실패 차단과 save/two preview/UI-A-B를 사용하며 새IPC/DTO/schema는 없다. [계약](ipc-commands.md), [작업](completed-history.md#task-mean-reversion-policy-20261020).


v17 MA cross는 기존 관측 교차를 보존하고 별도 bounded512 완료 일봉 context로 Buy 후보만 확인한다. 기존 scoped daily coordinator와 apply/invalidate hook을 재사용하며 현재봉 after-decision, 일봉 재조회와 교차열 분리를 유지한다. 순수 일봉 지표3개만 completed_daily.rs에서 Consecutive와 공유한다. EntryFilterPreparation의 MA 전용 선택 slope/bandwidth 거부 수는 Consecutive에서 생략한다. 실제 소유·주문·provider/리스크 경계는 기존 그대로다. [계약](ipc-commands.md), [작업](completed-history.md#task-ma-cross-filters-20261020).

## LTH minute timing 진단 경계

`strategy_preview → lth_preview_assessment → simulation/minute_timing`의 pure 진단은 준비된 분봉 label을 읽어 실제 순간의 거래소 timezone·봉 시작/종가 가능 시각·일반 평일창 내부 미관측 슬롯을 bounded metadata로 만든다. 진단 자체는 관측·신호·회계·hash를 바꾸지 않는다. US 연구의 실제 채택/제외·계획 달력 근거는 같은 v1 객체의 선택 `executionClock`에 별도로 투영한다. 일반 평일창 슬롯과 적용된 계획 달력의 날짜/마감을 혼동하지 않는다. 구문 파서만 `simulation/minute_time`에서 generic 정규화와 공유한다. UI와 저장 A/B는 실행 정책·채택 identity 차이와 legacy/unknown을 표시하되 같은 정책/채택봉의 asOf 밀리초 차이는 다른 모델로 취급하지 않는다. 달력/실제 발행/coverage flag는 false이고 성과 적합성도 인증하지 않는다. [IPC 계약](ipc-commands.md#lth-분봉-시각-진단-v1-2026-10-22), [현재 경과](completed-history.md#p1-session-01), [과거 진단 기록](completed-history.md#task-lth-minute-timing-20261022)을 따른다.

## RSI 소유 피드백

RSI의가격/이전RSI 상태에 exact전략소유 수량·원가와pending/known age를결합한다. 기존manager/주문피드백 경계로만소유를갱신하고공유계좌잔고를전략소유로합치지않는다. 기존교차조건의prev기준은매관측갱신하고추세는current제외이전입력가격을사용한다. 선택손절/보유한도는실잔량에후보를계속낼수있지만기존공통guard/risk·nextOpen이실제청산을결정한다. 세부옵션/복원/단위·기본값은[IPC](ipc-commands.md#rsi-v18-실제-소유선택-보호)와[연구](research/rsi-policy-20261022/report.md)를따른다.

## 돌파 실패의 완료 자료·소유 보호

기존 완료 일봉 수집/jobs/manager의 검증 ticket을 통해 atomic CompletedDailyBar(OHLC+Option volume)를 전달한다. 기본 forwarding은 타전략 warmup/완료 처리를 한 번만 호출한다. 선택 고가·ATR·거래량은 현재 봉이나 live 누적량을 미리 사용하지 않는다. 후보/확정 anchor와 실제 소유는 자료 context와 분리하고 실제 flat 전 거절·부분·재설정이 보호를 지우지 않는다. 원래 v1 anchor와 새 v2 age/peak/armed의 근거를 구분한다. [IPC 계약](ipc-commands.md#v19-돌파-실패-완료-자료와-실제-소유-보호)과 [연구](research/failed-breakout-policy-20261022/report.md)를 따른다.

### StrongClose의 완료 시장일

StrongClose의 live 이전 일봉은 기존 coordinator의 target·scope/profile/account·market/date·generation/freshness 검증 후 공급된다. dated 기본 hook은 기존 자료 hook을 한 번만 호출해 타 전략을 보존한다. StrongClose는 새날 seed와 같은 날 context 교체를 나누며, 준비 무효화는 실제 소유 보호·날짜 소비를 지우지 않는다. replay 종가 신호와 live 첫 quote의 시점 차이, 날짜 교정 fixture와 실제 운용 성과를 구분한다. [P2-08](research/strong-close-policy-20261022/report.md).

### 변동성 확장의 완료 범위와 관측 시장일

변동성 전략은 완료N일봉 coordinator에 참여하고 검증된 ticket의 시장일로 새날의 관측 시가/고저를 나눈다. 같은날 history 교체는 해당 관측 범위를 초기화하지 않고 자료 실패·늦은 초기화가 실제 소유 보호를 지우지 않는다. replay는 당일 완성 OHLC→종가 신호→실제 체결 피드백→완료범위 게시 순서를 유지한다. 전용 범위 빈도 optional DTO는 준비/보유·미보유·범위/방향·통과를 구분해 기존 preview와 A/B에 보존한다. 실제 장중 관측과 완료 일봉의 공개 시점·전체 범위·실제 체결 검증은 별도다. [P2-09](research/volatility-expansion-policy-20261022/report.md).

### TrendFilter의 현재 포함 지표와 소유 증거

`mean_trend.rs`는 현재 포함 MA 200/5/20·strict 진입과 세 MA 준비 뒤 장기 이탈을 유지하면서 exact 소유 수량/native 평균·pending·관측 나이/최고가/수익 무장을 분리한다. 장기 이탈 확인·수익 되밀림·보유 한도 순서로 실제 잔량 전량 후보를 내며 선택 수익/보유 보호는 지표 미준비에도 평가한다. warmup은 지표만 준비하고 reset은 가격 버퍼를 보존한다. 이탈 연속값은 재시작·과거 재적재·핵심 MA 기간 변경에서0, 동일 프로세스 청크·동량/부분 매도에서 보존한다.

`trend_filter_params`는 저장/단기·연속 preview의 strict 범위를, `trend_filter_protection`은7필드 v1 checkpoint의 원자적 검증을 소유한다. 양수 보유는 나이 불명이어도 metadata를 반환해 기존 OrderManager caller의 반복 소유 갱신에서 cold restore가 반복되지 않게 한다. `strategyPolicyControls`는 페이지 조립만 맡고 `trendFilterControls`는7필드 raw/stored 초안·현재/A-B·단위·legacy 미확인을 유지한다. 고정 기준81fa00d/v21과 v22의 생산 실행·기록 정산은 [P2-11 연구](research/trend-filter-policy-20261023/report.md)에 두고 주문·계좌·운영 DB를 호출하지 않는다.

## 섹터 ETF 연구의 독립 경계

`trading/sector_portfolio`는 고정 universe의 상대강도 순위와 이전 완료자료의
장전 목표계산을 공개 순수 함수로 제공한다. 별도 USD 다종목 장부가 같은 날짜의
같은 관측시가를 3전략·SPY/현금 비교에 공유하며 단일종목 KRW ReplaySession을
합산하거나 FX=1로 가장하지 않는다. 입력 전체 검증 후에만 모든 계정을 계산한다.

`commands/sector_portfolio`·`server/sector_portfolio`는 AppState 없이 같은 계산을
호출한다. `/simulation`의3탭 작업 공간과 내부 연구 화면은 `widgets/sector-portfolio`의
입력·준비·비교·수동 대조 UI를 공유한다. 기존 Strategy 위치에는 바로가기를 두며
이전 페이지 import는 shim으로 유지한다. 기존 A/B experimentStore의 축약 저장·scope정리 경로는
사용하지 않는다. 이 모듈은 운영 factory/daemon/주문/복원 원장에 등록되지 않으며 향후 실제
연결에는 별도의 소유·예산·예약/부분체결/거절·정지/재시작 검증이 필요하다.
자료·기업행위 지원 제한과 wire 계약은 [IPC 정본](ipc-commands.md#섹터-etf-포트폴리오-연구-v1)을 따른다.

`trading/sector_data`는 고정12종목 raw/adjusted 자료를 모으는 quote-source 경계와
provider를 호출하지 않는 bundle 검증을 분리한다. 일반 native의 명시 scope 확인·동의
수집만 AppState의 KIS snapshot을 사용하며 provider await 전후 ID/client identity를
검사한다. REST는 순수 검증만 제공하고 수집·scope 조회는 등록하지 않는다.

`research-only` native는 별도 identifier·incognito window·순수 자료 검증·검증형/차트근사 계산3개 IPC를
사용하며 운영 AppState/profile/config/store/plugin/server/daemon을 초기화하지 않는다.
`main.tsx`는 marker/query로 운영 router import 전에 연구 화면을 고르고 전용 Vite는
`.env` 없이 loopback/HMR-off를 사용한다. 일반 앱의 query만으로 native 권한이
격리되는 것은 아니다. 수집이 수동 확인 선언을 승인하거나 실제 자료 진위를 인증하지
않으며 기업행위 정산·운영 주문 연결은 미완료다. [상세 계약](ipc-commands.md#섹터-자료-준비와-연구-전용-실행),
[로컬 실행법](user-guide.md#연구-전용-앱과-섹터-자료-준비)을 따른다.

일반 `/simulation`은 운영 AppShell 안의 경로다. Sidebar·계좌/운영 상태 query와 기존
backend daemon을 중단하지 않으며 `SimulationOperatingStatus`가 최근 정상/이전/오류를
표시한다. 메뉴·수집/검증·계산은 주문이나 시작/정지 명령을 보내지 않고 Sidebar의
운영 제어는 별도로 유지한다. `initialCashUsd`는 파일 가정이며 운영 잔고를 가져오지
않는다. 프로필 publication/epoch는 시세 scope/ack·늦은 수집 결과의 유효성에만 사용하고
이미 적용한 비교 입력·계산 결과의 계좌 귀속이나 자동 삭제를 만들지 않는다.
수집 기반 준비 bundle의 확인 선언/준비 보고서는 scope 변경 시 재검증한다. 페이지가 unmount되면
메모리 작업 상태는 별도 영속화를 하지 않는다. 내부 연구 앱의 운영 초기화 제외·자동API0
관측을 일반 Simulation 경로의 보장으로 확대하지 않는다.


`trading/broker_chart`는 명시 동의한 일반 native의 KIS/Toss 비수정 시세 수집을 소유하고,
`sector_portfolio/price_research`는 별도 DTO의 순수 가격근사 검증·모드 hash를 소유한다.
기존 replay/planner를 공유하되 strict preview의 확인 조건을 낮추지 않으며 내부 source
flag도 false다. 알려진 사건은 거부하고 미확인 권리/배당은 정산하지 않는다.
수집에는 현재 profile snapshot과 client identity를 대조하고 순수 계산에는 AppState를
전달하지 않는다. REST/연구 native는 순수 preview만 제공한다. Simulation의 기본
검증형과 근사형은 입력/결과를 각각 보존하며 실제/forward 원장을 갱신하지 않는다.
[별도 wire·수집 한계](ipc-commands.md#브로커-차트-가격수익률-근사),
[실측과 미검증](completed-history.md#p2-sector-01-broker-chart)을 따른다.
