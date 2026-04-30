# AutoConditionTrade — 프로젝트 맵

> 이 문서는 `agents.md` 에서 분리된 상세 디렉토리 맵 및 아키텍처 참조 문서입니다.

---

## 1. 전체 디렉토리 맵

```
AutoConditionTrade/                   ← 루트
│
├── agents.md                         ← 에이전트 네비게이션 가이드 (핵심만)
├── TodoList.md                       ← Phase별 할 일 목록
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
│   ├── copilot-instructions.md       ← AI 에이전트 프로젝트 지침 (살아있는 문서)
│   ├── skills/                       ← 도메인 스킬 파일 4종 (KIS API, Rust, React, UI)
│   └── workflows/release.yml         ← GitHub Actions 자동 빌드/릴리즈
│
├── docs/
│   ├── project-map.md                ← [이 파일] 디렉토리 맵 + 아키텍처
│   ├── ipc-commands.md               ← IPC 커맨드 전체 목록 (35개)
│   ├── coding-guide.md               ← 설정 추가·AppState·IPC·데몬·제어흐름 실전 가이드
│   ├── MasterPlan.md                 ← 전체 설계 문서 (아카이브, 읽기 전용)
│   ├── discord-setup-guide.md        ← Discord 봇 설정 가이드
│   └── user-guide.md                 ← 사용 가이드 (개요·전략 세팅)
│
├── src/                              ← React Frontend (TypeScript)
│   ├── main.tsx                      ← React 진입점 (QueryClient, RouterProvider)
│   ├── router/index.ts               ← TanStack Router 코드 기반 라우팅 ✅
│   ├── api/
│   │   ├── types.ts                  ← Rust 타입 미러 (TypeScript)
│   │   ├── commands.ts               ← invoke() 래퍼 함수 37종
│   │   ├── hooks.ts                  ← TanStack Query 훅 모음 (KEYS 상수)
│   │   └── transport.ts              ← Tauri IPC / Web REST 듀얼 모드
│   ├── theme/index.ts                ← createAppTheme, getResolvedMode
│   ├── store/
│   │   ├── settingsStore.ts          ← 테마/로그/Discord 설정 (zustand+persist)
│   │   ├── accountStore.ts           ← 계좌 잔고 상태
│   │   └── tradingStore.ts           ← 자동매매 실행 상태
│   ├── components/
│   │   ├── LayoutResizer.tsx         ← 사이드바 드래그 리사이즈
│   │   ├── chart/StockChart.tsx      ← lightweight-charts v5 국내주식 캔들
│   │   ├── chart/OverseasStockChart.tsx ← lightweight-charts v5 해외주식 캔들
│   │   ├── layout/AppShell.tsx       ← 전체 레이아웃 + ThemeProvider + Outlet
│   │   └── layout/Sidebar.tsx        ← MUI permanent/temporary Drawer
│   └── pages/
│       ├── Dashboard.tsx             ← 잔고/수익 카드, 포지션, 미체결/체결, 리스크
│       ├── Trading.tsx               ← 수동 매수/매도 + 종목 검색 + 체결 내역
│       ├── Strategy.tsx              ← 전략 ON/OFF + 파라미터 설정 (11개 전략)
│       ├── History.tsx               ← 날짜 범위 조회, 자동매매 체결 기록
│       ├── Log.tsx                   ← 레벨 필터, 검색, 색상 구분 로그 뷰어
│       └── Settings.tsx              ← API 키, 테마, 멀티 계좌, 웹 포트, 갱신 주기
│
└── src-tauri/                        ← Rust Backend
    ├── Cargo.toml                    ← Tauri v2 + reqwest + tokio + tracing
    ├── build.rs                      ← tauri_build::build()
    ├── tauri.conf.json               ← 앱 설정 (1400x900, bundle icons)
    └── src/
        ├── main.rs                   ← Tauri 진입점
        ├── lib.rs                    ← Builder 설정 + 백그라운드 데몬 6개 spawn
        ├── commands.rs               ← AppState + IPC 커맨드 핸들러 전체
        ├── market_hours.rs           ← 시장 개장 여부 판단 (KRX / US)
        ├── api/
        │   ├── rest.rs               ← KisRestClient — 잔고/주문/차트/환율
        │   ├── token.rs              ← TokenManager — 자동 갱신
        │   └── websocket.rs          ← KisWebSocketClient — 실시간 시세
        ├── market/mod.rs             ← KRX 종목 목록 (CSV 파싱, 캐시, 검색)
        ├── server/mod.rs             ← axum 웹 서버 (ServeDir + REST proxy)
        ├── updater/mod.rs            ← GitHub Releases API 버전 확인
        ├── trading/
        │   ├── mod.rs                ← 장 시간 감지, 전략 루프 실행
        │   ├── strategy.rs           ← Strategy trait + 11개 전략 + StrategyManager
        │   ├── order.rs              ← OrderManager: 주문 → 체결 → 저장
        │   ├── position.rs           ← PositionTracker (잔고 API 복원 지원)
        │   └── risk.rs               ← RiskManager (enabled on/off, 비상정지, 순손실)
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
| `store/` | Zustand 전역 상태 (계좌, 매매, 설정) |
| `api/hooks.ts` | TanStack Query 훅 + `useBackendEvents()` (Tauri 이벤트 → 캐시 갱신) |
| `api/commands.ts` | invoke() 래퍼 (37종) |
| `pages/Settings.tsx` | 데이터 갱신 주기 슬라이더, 웹 포트, 계좌 프로파일, 로그 설정 |
| `pages/Dashboard.tsx` | 잔고/수익 카드, 포지션, 미체결/체결, 리스크 |

### Backend (Rust)

| 모듈 | 책임 |
|------|------|
| `lib.rs` | Tauri Builder + 6개 백그라운드 데몬 spawn + `on_window_event` (종료 안전 처리) |
| `commands.rs` | AppState + 모든 IPC 커맨드 핸들러 |
| `api/token.rs` | KIS Access Token 자동 갱신 |
| `api/websocket.rs` | 실시간 시세 수신, 체결 콜백 |
| `trading/mod.rs` | 전략 루프 실행, 장 시간 감지 |
| `trading/order.rs` | submit_signal → 주문 → on_fill → 저장, `buy_suspended` 플래그 |
| `trading/risk.rs` | 일일 손실 한도, 비상 정지, `record_pnl` |
| `market_hours.rs` | 시장 개장 여부 (KRX 09:00-15:30 / US 22:00-07:00 KST) |
| `server/mod.rs` | axum 웹 서버 (21개 REST 핸들러, ServeDir) |
| `storage/trade_store.rs` | `data/trades/YYYY/MM/DD/trades.json` |
| `storage/stats_store.rs` | `data/stats/YYYY/MM/daily_stats.json` |
| `storage/strategy_store.rs` | `data/strategies/{profile_id}/strategies.json` |
| `notifications/discord.rs` | Discord Bot 알림 |
| `config/mod.rs` | `secure_config.json` + `.env` 로드 |

---

## 4. 백그라운드 데몬 목록 (lib.rs spawn 순서)

| 번호 | 역할 | 제어 방식 |
|------|------|----------|
| 1 | KRX 종목 목록 로드 | 1회성 |
| 2 | 자동매매 폴링 (`run_trading_daemon`) | `is_trading: Arc<Mutex<bool>>` |
| 3 | axum 웹 서버 | 영구 실행 |
| 4 | 환율 갱신 (USD/KRW) | `watch::Receiver` — interval 변경 즉시 반영 |
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
storage/trade_store.rs — JSON 저장 (data/trades/YYYY/MM/DD/)
    ↓
storage/stats_store.rs — 통계 집계 갱신
    ↓
notifications/discord.rs — TRADE 레벨 알림 전송
    ↓
Tauri Event emit → Frontend (실시간 UI 갱신)
```

### 실시간 데이터 Push (백그라운드 데몬 → 프론트)

```
lib.rs daemon 4/6 → app_handle.emit("exchange-rate-updated" / "balance-updated" / "overseas-balance-updated")
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
| Discord Bot API | 알림 전송 | `docs/discord-setup-guide.md` |
