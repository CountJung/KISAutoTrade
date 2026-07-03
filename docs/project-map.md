# AutoConditionTrade — 프로젝트 맵

> 이 문서는 `AGENTS.md` 에서 분리된 상세 디렉토리 맵 및 아키텍처 참조 문서입니다.

---

## 1. 전체 디렉토리 맵

```
AutoConditionTrade/                   ← 루트
│
├── AGENTS.md                         ← Codex 에이전트 네비게이션 가이드 (핵심만)
├── todo.md                           ← 개선 백로그 및 다음 작업 목록
├── Cargo.toml                        ← Rust workspace 루트 (resolver="2")
├── package.json                      ← npm 패키지 설정 (engines: node>=20)
├── vite.config.ts                    ← Vite 빌드 설정 (port:1420, 청크 분리)
├── tsconfig.json / tsconfig.node.json
├── index.html                        ← HTML 진입점 (테마 Hydration 스크립트 포함)
├── .gitignore                        ← 민감 파일/데이터/빌드/.cargo/config.toml 제외
├── .cargo/config.toml                ← Cargo 로컬 설정 (gitignore, 머신별 target-dir)
├── secure_config.example.json        ← 민감 설정 템플릿 ✅
│
├── .github/
│   ├── codex-instructions.md         ← Codex 프로젝트 지침 (살아있는 문서)
│   ├── copilot-instructions.md       ← GitHub Copilot/Codex 호환용 shim
│   ├── skills/                       ← 도메인 스킬 파일 6종 (KIS API, Toss API, Rust, React, FSD, UI)
│   └── workflows/release.yml         ← GitHub Actions 자동 빌드/릴리즈
│
├── .codex/
│   ├── README.md                     ← 프로젝트 소유 Codex 브리지 스킬 안내
│   └── skills/kisautotrade-*/        ← 계정 홈에 의존하지 않는 Codex 브리지 스킬
│
├── docs/
│   ├── project-map.md                ← [이 파일] 디렉토리 맵 + 아키텍처
│   ├── ipc-commands.md               ← IPC 커맨드 전체 목록 (35개)
│   ├── coding-guide.md               ← 설정 추가·AppState·IPC·데몬·제어흐름 실전 가이드
│   ├── toss-openapi.md               ← 토스증권 OpenAPI endpoint inventory + 검증 절차
│   ├── toss-readonly-small-order-checklist.md ← 토스 read-only/소액 실거래 검증 안전 절차
│   ├── mock-trading-e2e-checklist.md ← 모의투자 국내/해외/E2E 검증 체크리스트
│   ├── MasterPlan.md                 ← 전체 설계 문서 (아카이브, 읽기 전용)
│   ├── discord-setup-guide.md        ← Discord 봇 설정 가이드
│   └── user-guide.md                 ← 사용 가이드 (개요·전략 세팅)
│
├── scripts/check-fsd-imports.mjs     ← FSD 레이어 역방향 import 검증
├── scripts/verify-toss-openapi.mjs   ← 공식 토스증권 OpenAPI JSON 버전/경로/헤더 검증
├── scripts/sync-codex-skills.ps1     ← 프로젝트 Codex 브리지 스킬을 계정 Codex 홈으로 동기화 (`npm run sync:codex-skills`)
│
├── src/                              ← React Frontend (TypeScript, FSD 점진 구조)
│   ├── main.tsx                      ← React 진입점 (QueryClient, RouterProvider)
│   ├── router/index.ts               ← TanStack Router 코드 기반 라우팅 ✅
│   ├── shared/
│   │   ├── api/                      ← Tauri IPC/Web REST 공통 wrapper + Rust 타입 미러
│   │   ├── config/theme/             ← createAppTheme, getResolvedMode
│   │   ├── config/scheduler/         ← 전역 폴링 주기 상수
│   │   ├── lib/                      ← localStorage 기반 레이아웃 상태 헬퍼
│   │   └── ui/LayoutResizer.tsx      ← 범용 리사이저 UI
│   ├── entities/
│   │   ├── account/model/            ← 계좌 상태 store
│   │   ├── settings/model/           ← 테마/로그/Discord 설정 store
│   │   └── trading/model/            ← 자동매매 실행 상태 store
│   ├── features/                     ← manual-order, symbol-search 등 행동 단위 slice scaffold
│   ├── widgets/
│   │   ├── app-shell/                ← 전체 레이아웃 + ThemeProvider + Outlet
│   │   ├── sidebar/                  ← MUI permanent/temporary Drawer
│   │   └── stock-chart/              ← 국내/해외 lightweight-charts v5 캔들
│   ├── pages/
│   │   ├── dashboard/ui/Page.tsx     ← 잔고/수익 카드, 포지션, 미체결/체결, 리스크
│   │   ├── trading/ui/Page.tsx       ← 수동 매수/매도 + 종목 검색 + 체결 내역
│   │   ├── strategy/ui/Page.tsx      ← 전략 ON/OFF + 파라미터 설정 (12개 전략)
│   │   ├── history/ui/Page.tsx       ← 날짜 범위 조회, 자동매매 체결 기록
│   │   ├── log/ui/Page.tsx           ← 레벨 필터, 검색, 색상 구분 로그 뷰어
│   │   └── settings/ui/Page.tsx      ← API 키, 테마, 멀티 계좌, 웹 포트, 리스크 설정
│   ├── api/                          ← shared/api 호환 re-export + TanStack Query hooks
│   ├── components/                   ← widgets/shared 호환 re-export
│   ├── store/                        ← entities 호환 re-export
│   ├── theme/                        ← shared/config/theme 호환 re-export
│   └── scheduler/                    ← shared/config/scheduler 호환 re-export
│
└── src-tauri/                        ← Rust Backend
    ├── Cargo.toml                    ← Tauri v2 + reqwest + tokio + tracing
    ├── build.rs                      ← tauri_build::build()
    ├── tauri.conf.json               ← 앱 설정 (1400x900, window-state 복원용 visible:false, bundle icons)
    └── src/
        ├── main.rs                   ← Tauri 진입점
        ├── lib.rs                    ← Builder 설정 + 백그라운드 데몬 6개 spawn
        ├── commands.rs               ← AppState + IPC 커맨드 핸들러 전체
        ├── market_hours.rs           ← 시장 개장 여부 판단 (KRX / US)
        ├── api/
        │   ├── detect.rs             ← KIS 실전/모의 앱키 자동 감지
        │   ├── rest.rs               ← KisRestClient — 잔고/주문/차트/환율
        │   ├── token.rs              ← TokenManager — 자동 갱신
        │   └── websocket.rs          ← KisWebSocketClient — 실시간 시세
        ├── broker/                   ← BrokerId/domain 타입 + BrokerAdapter + KIS/Toss adapter 경계 + rate_limit scheduler
        ├── market/mod.rs             ← KRX 종목 목록 (CSV 파싱, 캐시, 검색)
        ├── server/mod.rs             ← axum 웹 서버 (ServeDir + REST proxy)
        ├── updater/mod.rs            ← GitHub Releases API 버전 확인
        ├── trading/
        │   ├── mod.rs                ← 장 시간 감지, 전략 루프 실행
        │   ├── strategy.rs           ← Strategy trait + 12개 전략 + StrategyManager
        │   ├── order.rs              ← OrderManager: 주문 → 체결 → 저장
        │   ├── preflight.rs          ← 주문 전 read-only 금액/수량/수수료 판정
        │   ├── position.rs           ← PositionTracker (잔고 API 복원 지원)
        │   └── risk.rs               ← RiskManager (enabled on/off, 비상정지, 순손실, broker/account scope별 주문/손실 제한)
        ├── storage/
        │   ├── mod.rs                ← build_daily_path, read_json_or_default, write_json
        │   ├── trade_store.rs        ← TradeRecord, TradeStore
        │   ├── order_store.rs        ← OrderRecord, OrderStore
        │   ├── stats_store.rs        ← DailyStats, StatsStore
        │   ├── balance_store.rs      ← BalanceSnapshot, BalanceStore
        │   ├── stock_store.rs        ← StockStore (종목코드↔이름 캐시)
        │   └── strategy_store.rs     ← 전략 설정 JSON 영구 저장
        ├── notifications/
        │   ├── discord.rs            ← DiscordNotifier (HTTP POST)
        │   ├── types.rs              ← NotificationLevel/Event, to_discord_message()
        │   └── mod.rs
        ├── logging/mod.rs            ← tracing-appender (app.log, error.log), LogConfig
        └── config/mod.rs             ← AccountProfile, ProfilesConfig, AppConfig, DiscordConfig
```

---

## 2. 파일 저장 위치 요약

| 데이터 종류 | 위치 |
|-----------|------|
| `profiles.json` | `~/Library/Application Support/com.countjung.kisautotrade/` (macOS) |
| `data/` (거래기록 등) | CWD 기준 `./data/` (레거시: app_data_dir, 자동 이전) |
| `logs/` | CWD 기준 `./logs/` |
| `secure_config.json` | 프로젝트 루트 (CWD) |
| `.env` | 프로젝트 루트 (CWD) |

---

## 3. 핵심 모듈 책임 요약

### Frontend

| 모듈 | 책임 |
|------|------|
| `router/` | TanStack Router 기반 라우팅 |
| `shared/api/` | Tauri IPC/Web REST wrapper, Rust 타입 미러 (`BrokerHoldingView` 포함) |
| `shared/ui/` | 공통 UI (`LayoutResizer`, `BrokerScopeIndicator` broker/profile/account scope 표시, `ProviderTraceChips` 원본 요청 trace 표시) |
| `shared/lib/` | 공통 유틸 (`persistentLayout` localStorage 숫자 상태 저장/복원) |
| `shared/config/theme/` | 앱 테마 생성과 theme mode 타입 |
| `shared/config/scheduler/` | TanStack Query 공통 폴링 주기 |
| `entities/*/model/` | Zustand 전역 상태 (계좌, 매매, 설정) |
| `api/hooks.ts` | TanStack Query 훅 + `useBackendEvents()` (Tauri 이벤트 → 환율/잔고 캐시 갱신, 점진 이동 전 legacy entry) |
| `widgets/app-shell/` | 전체 앱 레이아웃, ThemeProvider, responsive navigation |
| `widgets/stock-chart/` | 국내/해외/Toss 캔들 차트 |
| `pages/settings/ui/Page.tsx` | 데이터 갱신 주기 슬라이더, 웹 포트, broker-aware 계좌 프로파일, Toss 연결 진단, 로그/리스크 설정 |
| `pages/dashboard/ui/Page.tsx` | 활성 broker scope, KIS 국내/해외 잔고, 활성 Toss broker 보유 종목, USD/KRW 환율 출처 chip, 수익 카드, 미체결/체결, 리스크 |
| `pages/trading/ui/Page.tsx` | 활성 broker scope, KIS 국내/해외 수동 주문과 차트, 활성 Toss 프로파일의 read-only 시세 snapshot/차트/종목 유의사항/장 운영 상태 표시 |
| `pages/strategy/ui/Page.tsx` | 활성 broker scope, 전략별 저장 broker/account scope 표시, 전략 활성화/파라미터/대상 종목 관리 |
| `pages/history/ui/Page.tsx` | 활성 broker scope, 자동매매 체결 기록과 기간별 통계 조회, provider 원본 trace 표시 |
| `pages/log/ui/Page.tsx` | 로그 레벨/검색 필터, provider trace 토큰 chip 표시 |

### Backend (Rust)

| 모듈 | 책임 |
|------|------|
| `lib.rs` | Tauri Builder + window-state 플러그인 + 6개 백그라운드 데몬 spawn + `on_window_event` (종료 안전 처리) |
| `commands.rs` | AppState + 모든 IPC 커맨드 핸들러 (`get_broker_holdings`, Toss read-only views, `get_exchange_rate_status` 환율 정책 view, 자동매매 시작 전 broker-aware 포지션 복원 포함) |
| `api/detect.rs` | KIS 토큰 응답 기반 실전/모의 앱키 자동 감지 |
| `broker/` | 다중 증권사 공통 타입(`BrokerScope` 포함), adapter trait, `RateLimitScheduler`. KIS 기존 REST 호출을 점진 래핑하고 Toss token/accounts/holdings/market-data/market-info/order client를 수용 |
| `api/token.rs` | KIS Access Token 자동 갱신 |
| `api/websocket.rs` | 실시간 시세 수신, 체결 콜백 |
| `trading/mod.rs` | 전략 루프 실행, 장 시간 감지 |
| `trading/order.rs` | submit_signal → 주문 → provider별 체결 확인 → on_fill → 저장, `buy_suspended` 플래그, provider trace 전파 |
| `trading/risk.rs` | 일일 손실 한도, 비상 정지, `record_pnl` |
| `market_hours.rs` | 시장 개장 여부 (KRX 09:00-15:30 / US 22:00-07:00 KST) |
| `server/mod.rs` | axum 웹 서버 (`/api/broker-holdings`, Toss read-only REST, `/api/exchange-rate/status` 포함 REST 핸들러, ServeDir) |
| `storage/trade_store.rs` | `data/trades/YYYY/MM/DD/trades.json` (`provider_*` 원본 요청 trace 포함) |
| `storage/order_store.rs` | `data/orders/YYYY/MM/DD/orders.json` (`provider_*` 원본 주문 trace 포함) |
| `storage/stats_store.rs` | `data/stats/YYYY/MM/daily_stats.json` |
| `storage/strategy_store.rs` | `data/strategies/{profile_id}/strategies.json` (`StrategyConfig`에 broker/account scope 저장) |
| `notifications/discord.rs` | Discord Bot 알림 |
| `config/mod.rs` | `secure_config.json` + `.env` 로드 |

---

## 4. 백그라운드 데몬 목록 (lib.rs spawn 순서)

| 번호 | 역할 | 제어 방식 |
|------|------|----------|
| 1 | KRX 종목 목록 로드 | 1회성 |
| 2 | 자동매매 폴링 (`run_trading_daemon`) | `is_trading: Arc<Mutex<bool>>` |
| 3 | axum 웹 서버 | 영구 실행 |
| 4 | 환율 갱신 (USD/KRW) | `watch::Receiver` — Toss 우선/공개 환율/fallback 캐시 정책 + interval 변경 즉시 반영 |
| 5 | 로그/체결기록 일일 정리 | 24h 주기 |
| 6 | 잔고 갱신 + 이벤트 발행 | `watch::Receiver` — interval 변경 즉시 반영 |

---

## 5. 데이터 흐름

### 체결 발생 시

```
WebSocket 수신 (체결 이벤트)
    ↓
trading/order.rs — 체결 확인
    ↓
storage/trade_store.rs — JSON 저장 (data/trades/YYYY/MM/DD/, provider/order/request/TR trace 포함)
    ↓
storage/stats_store.rs — 통계 집계 갱신
    ↓
notifications/discord.rs — TRADE 레벨 알림 전송
    ↓
Tauri Event emit → Frontend (실시간 UI 갱신)
```

### 실시간 데이터 Push (백그라운드 데몬 → 프론트)

```
lib.rs daemon 4/6 → app_handle.emit("exchange-rate-updated" / "exchange-rate-status-updated" / "balance-updated" / "overseas-balance-updated")
    ↓
AppShell.tsx — useBackendEvents() listen()
    ↓
TanStack Query — setQueryData() (캐시 직접 갱신, 네트워크 요청 없음)
    ↓
관련 컴포넌트 리렌더
```

---

## 6. 설정 파일 레퍼런스

### `.env` (git ignore)

```
KIS_APP_KEY=실전투자_앱키
KIS_APP_SECRET=실전투자_앱시크릿
KIS_ACCOUNT_NO=12345678-01
KIS_IS_PAPER_TRADING=false
WEB_PORT=7474
REFRESH_INTERVAL_SEC=30
```

> `WEB_PORT` / `REFRESH_INTERVAL_SEC` 는 Settings UI에서도 수정 가능 (`.env` 자동 갱신)

### `secure_config.json` (git ignore)

`secure_config.example.json` 참고. Discord 봇 토큰, 모의/실전 듀얼 키 포함.

> **우선순위**: `secure_config.json` > `.env` 환경변수 > 기본값

---

## 7. 외부 의존 서비스

| 서비스 | 용도 | 참고 |
|--------|------|------|
| 한국투자증권 Open API | REST + WebSocket 주식 거래 | [apiportal.koreainvestment.com](https://apiportal.koreainvestment.com) |
| 토스증권 Open API | REST 기반 시세·계좌·주문 확장 후보 | [developers.tossinvest.com](https://developers.tossinvest.com/docs) |
| Discord Bot API | 알림 전송 | `docs/discord-setup-guide.md` |

---

## 8. Codex 프로젝트 브리지 스킬

GitHub Copilot 호환용으로 유지하던 `.github/skills/**/SKILL.md` 원본 스킬은 Codex에서도 자동 트리거될 수 있도록 프로젝트 루트 `.codex/skills/kisautotrade-*`에 얇은 브리지로 연결되어 있다. 브리지는 절대 경로를 저장하지 않고, 현재 작업 저장소에서 `AGENTS.md`와 `.github/skills/**`를 찾아 원본을 읽는다.

| Codex 프로젝트 스킬 | 저장소 원본 |
|-----------------|-------------|
| `.codex/skills/kisautotrade-project` | `AGENTS.md`, `.github/codex-instructions.md` |
| `.codex/skills/kisautotrade-kis-api` | `.github/skills/kis-api/SKILL.md` |
| `.codex/skills/kisautotrade-toss-api` | `.github/skills/toss-api/SKILL.md` |
| `.codex/skills/kisautotrade-rust` | `.github/skills/rust-skills/SKILL.md` |
| `.codex/skills/kisautotrade-react` | `.github/skills/react-best-practices/SKILL.md` |
| `.codex/skills/kisautotrade-frontend-fsd` | `.github/skills/frontend-fsd/SKILL.md` |
| `.codex/skills/kisautotrade-ui` | `.github/skills/ui-conventions/SKILL.md` |

규칙 변경 시 브리지 파일이 아니라 저장소 원본을 수정한다. 프로젝트 위치나 폴더명이 바뀌어도 `AGENTS.md`와 `.github/skills/**` 구조가 유지되면 브리지는 그대로 동작한다. Codex 런타임이 프로젝트 스킬을 직접 읽지 못하는 경우 `scripts/sync-codex-skills.ps1`로 계정 홈에 동기화한 뒤 새 세션을 시작한다.
