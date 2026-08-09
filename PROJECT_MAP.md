# KISAutoTrade 탐색 지도

> 목적: 작업 유형에서 실제 수정 지점으로 빠르게 이동하는 수동 큐레이션 지도다. 전체 파일 트리와 장문의 모듈 책임표는 자동 생성 기준인 [`docs/project-map.md`](docs/project-map.md)를 사용한다.

## 1. 런타임 한눈에 보기

```text
React 18 + TypeScript + MUI
  pages/features/widgets/entities/shared
       ↓ TanStack Query hooks
  src/api/commands.ts
       ↓ src/shared/api/transport.ts (Tauri invoke 또는 Web REST)
Tauri v2 command / Axum route
       ↓ AppState + broker adapter
strategy → guard/preflight/risk → OrderManager → KIS/Toss
       ↓
JSON/DB storage + pending/order/trade/risk + Discord/event
```

상세 계층과 주문 시퀀스는 [`ARCHITECTURE.md`](ARCHITECTURE.md)를 본다.

## 2. 작업별 첫 진입점

| 작업 | 먼저 읽을 곳 | 같이 확인할 곳 |
|---|---|---|
| 앱 라우트/화면 | `src/router/index.ts`, `src/pages/*/ui/Page.tsx` | `src/widgets/`, `src/features/`, `src/shared/ui/` |
| 전역 UI 상태 | `src/entities/*/model/*Store.ts` | 하위 호환 `src/store/*.ts` re-export |
| 서버 상태 조회/변경 | `src/api/hooks.ts`, `src/api/queryKeys.ts` | `src/api/commands.ts`, `src/shared/api/transport.ts` |
| 공용 IPC 타입 | `src/shared/api/types.ts` | `src/api/types.ts` re-export/legacy surface, Rust serde view |
| DB IPC | `src/api/databaseHooks.ts` | `src/shared/api/databaseCommands.ts`, `commands/database.rs` |
| Tauri command 추가 | `src-tauri/src/commands/<domain>.rs` | `commands.rs`, `lib.rs::generate_handler!`, `docs/ipc-commands.md` |
| Web REST 대응 | `src-tauri/src/server/mod.rs` | `server/{market,profiles,records,toss,trading}.rs`, transport route map |
| KIS 연동 | `src-tauri/src/api/rest.rs`, `api/rest/*` | `api/token.rs`, `api/websocket.rs`, `broker/kis.rs` |
| Toss 연동 | `src-tauri/src/broker/toss/` | `commands/toss*.rs`, `server/toss.rs` |
| broker 공통 계약 | `src-tauri/src/broker/domain.rs`, `adapter.rs` | `kis.rs`, `toss/adapter.rs`, `rate_limit.rs` |
| 자동매매 시작/데몬 | `src-tauri/src/commands/trading.rs` | `lib.rs`, `trading/mod.rs`, `trading/views.rs` |
| 전략 | `src-tauri/src/trading/strategy/` | `commands/strategy.rs`, `commands/strategy_preview.rs`, Strategy UI |
| 수동·자동 주문 | `src-tauri/src/trading/order/submission.rs` | `order/{conflicts,fills}.rs`, `commands/orders.rs` |
| 주문 전 검증 | `src-tauri/src/trading/preflight.rs`, `trading/guard.rs` | broker별 preflight command, `trading/risk.rs` |
| 리스크 | `src-tauri/src/trading/risk.rs` | `commands/risk.rs`, `storage/risk_store.rs`, restore tests |
| pending/체결 | `trading/order/fills.rs` | `storage/pending_order_store.rs`, `order_store.rs`, `trade_store.rs` |
| 저장 backend | `src-tauri/src/storage/mod.rs`, `database.rs` | 각 `*_store.rs`, DB contract tests |
| 데몬/앱 초기화 | `src-tauri/src/lib.rs` | `commands.rs::AppState`, `server/mod.rs` |
| 릴리스/CI | `package.json`, root `Cargo.toml` | `.github/workflows/*.yml`, `docs/release-security.md` |

## 3. Frontend 경계

- `pages`: route 조립과 화면별 UI.
- `widgets`: 앱 셸·사이드바·차트처럼 페이지를 조합하는 큰 블록.
- `features`: 사용자 행동 단위 공개 진입점.
- `entities`: account/settings/trading 도메인 상태.
- `shared`: 재사용 API 계약, transport, 설정, UI, 순수 helper.
- `src/api/*`: 기존 import를 유지하는 애플리케이션 API/hook 계층. 새 중복 구현을 만들지 말고 `shared/api` 공개 surface와 관계를 먼저 확인한다.

현재 route는 dashboard(`/`), trading, strategy, history, log, settings다. import 방향은 `npm run check:fsd`가 검사한다.

## 4. Backend 경계

- `commands/`: Tauri IPC 입력 검증과 facade. 도메인 로직을 command에 복제하지 않는다.
- `server/`: 웹 REST facade. 민감 DB 관리 IPC는 인증 없는 REST에 노출하지 않는다.
- `broker/`와 `api/`: provider 계약·HTTP/WebSocket·rate limit.
- `trading/`: 전략, 주문, 포지션, preflight, guard, risk, simulation의 핵심 도메인.
- `storage/`: JSON/DB 공통 저장 경계와 projection. store에서 backend를 우회한 직접 파일 저장을 추가하지 않는다.
- `lib.rs`: AppState 조립, 6개 백그라운드 작업, command 등록.

## 5. 중요한 동기화 묶음

### IPC 계약 변경

```text
Rust input/view + serde
→ command facade
→ lib.rs generate_handler!
→ TS shared type
→ command wrapper / transport REST map
→ query hook/key/invalidation
→ UI
→ docs/ipc-commands.md
```

### 주문·리스크 변경

```text
BrokerScope + profile/account
→ strategy/manual request
→ provider preflight
→ RiskManager + TradeGuard
→ submission reservation/conflict
→ provider adapter
→ pending/order persistence
→ fill reconciliation
→ trade/stats/risk persistence
```

공용 타입·IPC·주문·리스크의 다중 참조 추적에는 필요할 때 Serena를 선택한다. Graphify는 위 구조 자체를 이동·통합하는 리팩터링 때만 사용한다.

## 6. 데이터와 민감 영역

| 데이터 | 코드상 위치/경계 |
|---|---|
| 프로파일 | Tauri app data의 `profiles.json`; `lib.rs`/`config` 관리 |
| 거래 데이터 | 실행 CWD의 `data/`; `storage::*Store`와 JSON/DB document 경계 |
| 로그 | 실행 CWD의 `logs/`; `logging/` |
| DB 설정 | app data의 `database_config.json`; password는 IPC view에서 반환하지 않음 |
| DB export | app data의 `database_exports/`; manifest 포함 |

실제 `.env`, `secure_config.json`, `profiles.json`은 탐색 대상이 아니다. 위치·형식의 상세 기준은 `docs/project-map.md`와 `docs/coding-guide.md`를 따른다.

## 7. 지도 유지 원칙

- 이 문서는 작업 라우팅이 달라질 때만 갱신한다.
- 전체 파일 추가·이동·삭제는 `npm run project-map:update`로 `docs/project-map.md` 생성 블록을 갱신하고 `npm run check:project-map`으로 확인한다.
- IPC 이름은 이 문서에 복제하지 않고 `docs/ipc-commands.md`에만 유지한다.
- 검증 명령은 이 문서에 복제하지 않고 [`HARNESS_MAP.md`](HARNESS_MAP.md)에 유지한다.
