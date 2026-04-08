# AutoConditionTrade — Agent Harness Map

> ⚠️ **에이전트 필독 규칙**
> - 모든 작업 시작 전 이 파일을 반드시 읽는다.
> - 파일 추가/삭제/이동, 구조 변경, 모듈 추가 후 이 파일을 즉시 업데이트한다.
> - 이 파일이 최신 상태가 아니라면 작업 전에 먼저 갱신한다.
> - 추측이 아닌 이 맵을 기반으로 작업한다.

**마지막 업데이트**: 2026-04-08T00:00:00  
**프로젝트 상태**: Phase 1~7 완료 / Phase 8+ 진행 중 (WebSocket Dashboard 연동 ✅, 추가 전략 RSI·모멘텀·이격도·52주신고가·연속상승·돌파실패·강한종가·변동성확장 ✅, 전략 설정 프로파일별 영구 저장 ✅, GitHub Actions 자동 빌드 ✅, 체결 기록 보관 설정+대시보드 조회 ✅)

---

## 1. 프로젝트 식별 정보

| 항목 | 값 |
|------|-----|
| 프로젝트명 | AutoConditionTrade |
| 목적 | 개인용 자동 주식 매매 시스템 |
| 언어 | Rust (Backend) + TypeScript/React (Frontend) |
| 플랫폼 | Tauri v2 (Desktop) + Web |
| 주요 외부 의존 | 한국투자증권 Open API, Discord Bot API |
| 데이터 저장 | DB 없음 — JSON 파일 (연/월/일 폴더 구조) |

---

## 2. 전체 디렉토리 맵

```
AutoConditionTrade/                   ← 루트
│
├── agent.md                          ← [이 파일] 하네스 구조 맵 (항상 최신)
├── TodoList.md                       ← Phase별 할 일 목록 (항상 최신)
├── Cargo.toml                        ← Rust workspace 루트 (resolver="2", profile 설정)
├── package.json                      ← npm 패키지 설정 (engines: node>=20)
├── vite.config.ts                    ← Vite 빌드 설정 (port:1420, 청크 분리)
├── tsconfig.json / tsconfig.node.json
├── index.html                        ← HTML 진입점 (테마 Hydration 스크립트 포함)
├── .nvmrc                            ← Node.js 버전 고정 (25.9.0) ✅
├── .gitignore                        ← 민감 파일/데이터/빌드/.cargo/config.toml 제외
├── .cargo/
│   └── config.toml                   ← Cargo 로컬 설정 (gitignore, 머신별 target-dir)
├── .env.example                      ← 환경변수 예시
├── secure_config.example.json        ← 민감 설정 템플릿 (실전/모의 키 포함) ✅
├── profiles.json                     ← 멀티 계좌 프로파일 (git ignore, 로컬 전용)
│
├── .github/
│   ├── copilot-instructions.md       ← AI 에이전트 프로젝트 지침 (살아있는 문서)
│   ├── skills/                       ← 도메인 스킬 파일 4종 (KIS API, Rust, React, UI)
│   └── workflows/
│       └── release.yml               ← GitHub Actions 자동 빌드/릴리즈 (Windows + macOS Universal) ✅
├── .vscode/
│   ├── settings.json                 ← rust-analyzer.linkedProjects, TypeScript SDK 지정 (Go-to-Definition) ✅
│   ├── launch.json                   ← cppvsdbg + CodeLLDB + Chrome 디버거 설정
│   ├── extensions.json               ← 권장 확장 목록
│   └── tasks.json                    ← cargo check, tsc --noEmit, Vite dev, Tauri dev 태스크
│
│   └── setup-local.sh                ← 로컬 환경 초기 설정 스크립트 (macOS exFAT 대응) ✅
│
├── docs/
│   ├── MasterPlan.md                 ← 전체 설계 문서 (아카이브, 읽기 전용) ✅
│   ├── discord-setup-guide.md        ← Discord 봇 연계 상세 가이드 ✅
│   ├── user-guide.md                 ← 사용 가이드 (개요·GitHub Actions·전략 세팅) ✅
│   └── api-reference.md              ← KIS API 참조 (추후 작성)
│
├── src/                              ← React Frontend (TypeScript)
│   ├── main.tsx                      ← React 진입점 (QueryClient, RouterProvider)
│   ├── router/
│   │   └── index.ts                  ← TanStack Router 코드 기반 라우팅 ✅
│   ├── api/
│   │   ├── types.ts                  ← Rust 타입 미러 (TypeScript, 모든 IPC 응답 타입) ✅
│   │   ├── commands.ts               ← invoke() 래퍼 함수 37종 ✅
│   │   ├── hooks.ts                  ← TanStack Query 훅 모음 (KEYS 상수 관리) ✅
│   │   └── transport.ts              ← Tauri IPC / Web REST 듀얼 모드 invoke 래퍼 ✅
│   ├── theme/
│   │   └── index.ts                  ← createAppTheme, getResolvedMode, Hydration ✅
│   ├── store/
│   │   ├── settingsStore.ts          ← 테마/로그/Discord 설정 (zustand+persist) ✅
│   │   ├── accountStore.ts           ← 계좌 잔고 상태 ✅
│   │   └── tradingStore.ts           ← 자동매매 실행 상태 ✅
│   ├── components/
│   │   ├── LayoutResizer.tsx         ← 사이드바 드래그 리사이즈 ✅
│   │   ├── chart/
│   │   │   ├── StockChart.tsx        ← lightweight-charts v5 국내주식 캔들 차트 ✅
│   │   │   └── OverseasStockChart.tsx← lightweight-charts v5 해외주식 캔들 차트 ✅
│   │   └── layout/
│   │       ├── AppShell.tsx          ← 전체 레이아웃 + ThemeProvider + Outlet ✅
│   │       └── Sidebar.tsx           ← MUI permanent/temporary Drawer ✅
│   └── pages/
│       ├── Dashboard.tsx             ← 잔고/수익 카드, 당일 거래 목록, 포지션 테이블, WS연결상태 Chip ✅
│       ├── Trading.tsx               ← 수동 매수/매도 폼 + 종목 검색 + 체결 내역 ✅
│       ├── Strategy.tsx              ← 전략 ON/OFF, 파라미터 설정 (10개 전략 범용 UI, STRATEGY_PARAM_META) ✅
│       ├── History.tsx               ← 날짜 범위 조회, 통계 요약, 거래 테이블 ✅
│       ├── Log.tsx                   ← 레벨 필터, 검색, 색상 구분 로그 뷰어 ✅
│       ├── Settings.tsx              ← Discord 테스트, API 키 표시, 테마 설정, 멀티 계좌 프로파일, 웹 포트 ✅
│
├── src-tauri/                        ← Rust Backend
│   ├── Cargo.toml                    ← Tauri v2 + reqwest + tokio + tracing 등
│   ├── build.rs                      ← tauri_build::build()
│   ├── tauri.conf.json               ← 앱 설정 (1400x900, bundle icons)
│   ├── icons/                        ← 앱 아이콘 (icon.ico, icon.icns, PNG 등) ✅
│   └── src/
│       ├── main.rs                   ← Tauri 진입점
│       ├── lib.rs                    ← Builder 설정 + logging 초기화 ✅
│       ├── api/
│       │   ├── mod.rs                ← KisRestClient, KisWebSocketClient 재공개
│       │   ├── rest.rs               ← KisRestClient — get_balance, place_order, get_today_executed_orders, get_price, get_chart_data, get_overseas_price, get_overseas_chart_data ✅
│       │   ├── token.rs              ← TokenManager — issue_token, get_token, is_expired (auto-refresh) ✅
│       │   └── websocket.rs          ← KisWebSocketClient — subscribe (WsStatusEvent emit, ws-status Tauri 이벤트) ✅
│       ├── market/
│       │   └── mod.rs                ← KRX 종목 목록 StockList (CSV 파싱, 캐시, 이름/코드 검색) ✅
│       ├── server/
│       │   └── mod.rs                ← axum 웹 서버 (ServeDir, REST proxy — WEB_PORT 기본 7474) ✅
│       ├── updater/
│       │   └── mod.rs                ← GitHub Releases API 버전 확인 (check_for_update IPC) ✅
│       ├── trading/
│       │   ├── mod.rs                ← 장 시간 감지, 전략 루프 실행 ✅
│       │   ├── strategy.rs       ← Strategy trait, 10개 전략 (MA Cross·RSI·모멘텀·이격도·52주신고가·연속상승하락·돌파실패·강한종가·변동성확장·평균회귀·추세필터), StrategyManager ✅
│       │   ├── order.rs              ← OrderManager: submit_signal → place_order, on_fill → TradeStore+OrderStore 저장, confirm_fill_by_symbol (시장가 자동 확인), EGW00201 재시도, 빈 ondo UUID fallback ✅
│       │   ├── position.rs           ← PositionTracker (add_buy/reduce/unrealized_pnl) ✅
│       │   └── risk.rs               ← RiskManager (emergency_stop, record_pnl, check_position_size) ✅
│       ├── storage/                  ← 연/월/일 JSON 파일 I/O ✅
│       │   ├── mod.rs                ← build_daily_path, read_json_or_default, write_json
│       │   ├── trade_store.rs        ← TradeRecord, TradeStore
│       │   ├── order_store.rs        ← OrderRecord, OrderStore
│       │   ├── stats_store.rs        ← DailyStats, StatsStore
│       │   └── balance_store.rs      ← BalanceSnapshot, BalanceStore
│       ├── notifications/            ← Discord Bot 알림 ✅
│       │   ├── mod.rs                ← NotificationService trait (async-trait)
│       │   ├── discord.rs            ← DiscordNotifier (HTTP POST to Discord API)
│       │   └── types.rs              ← NotificationLevel/Event, to_discord_message()
│       ├── logging/
│       │   └── mod.rs                ← tracing-appender (app.log, error.log), LogConfig, read_recent_entries (날짜 폴백 포함) ✅
│       ├── commands.rs               ← AppState(ws_connected:Arc<AtomicBool> 포함) + IPC 커맨드 핸들러 ✅
│       └── config/
│           └── mod.rs                ← AccountProfile, ProfilesConfig, AppConfig(from_profile), DiscordConfig ✅
│       ├── updater/
│           └── mod.rs                ← GitHub Releases API 버전 확인 (check_for_update IPC) ✅
│
├── data/                             ← JSON 데이터 (git ignore)
├── log/                              ← 로그 파일 (git ignore, CWD 기준 logs/)
├── target/                           ← Cargo 빌드 산출물 (git ignore)
├── node_modules/                     ← npm 패키지 (git ignore)
├── dist/                             ← Vite 빌드 산출물 (git ignore)
├── secure_config.json                ← Discord 봇 토큰 (git ignore, 프로젝트 루트)
├── secure_config.example.json        ← 설정 템플릿 (git 추적)
└── .env                              ← 환경변수 (git ignore, 프로젝트 루트)
```

> **파일 저장 위치 요약**  
> - `profiles.json` + `data/` → Tauri app_data_dir: `~/Library/Application Support/com.countjung.kisautotrade/` (macOS)  
> - `logs/` → CWD 기준 프로젝트 루트 `logs/`  
> - `secure_config.json` + `.env` → 프로젝트 루트 (CWD)

---

## 3. 핵심 모듈 책임 요약

### Frontend

| 모듈 | 책임 |
|------|------|
| `router/` | TanStack Router 기반 라우팅, Breadcrumb |
| `store/` | Zustand 전역 상태 (계좌, 매매, 설정) |
| `pages/History.tsx` | JSON 거래 기록 날짜 범위 조회 및 차트 |
| `pages/Settings.tsx` | Discord 봇 설정, API 키, 로그 설정 |

### Backend (Rust)

| 모듈 | 책임 |
|------|------|
| `api/token.rs` | KIS Access Token 자동 갱신 |
| `api/websocket.rs` | 실시간 시세 수신, 체결 콜백 |
| `trading/mod.rs` | 전략 루프 실행, 장 시간 감지 |
| `trading/risk.rs` | 일일 손실 한도 감시, 비상 정지 |
| `commands.rs::start_trading` | **폴링 루프** (10초 주기, 국내/해외 현재가 → on_tick → submit_signal → fills_pending → confirm_fill_by_symbol) + 일별 초기화 |
| `trading/order.rs::OrderManager` | submit_signal → KIS 주문, on_fill → TradeStore+OrderStore+StatsStore 저장, confirm_fill_by_symbol → 시장가 자동 체결 확인 |
| `storage/trade_store.rs` | `data/trades/YYYY/MM/DD/trades.json` 읽기/쓰기 |
| `storage/stats_store.rs` | 체결 집계 → `data/stats/YYYY/MM/daily_stats.json` |
| `storage/strategy_store.rs` | 전략 설정 영구 저장 → `data/strategies/{profile_id}/strategies.json` |
| `notifications/discord.rs` | Discord Bot으로 알림 메시지 전송 |
| `config/mod.rs` | `secure_config.json` + `.env` 로드 (실전/모의 듀얼 키, 기본: 실전투자) |

---

## 4. IPC Command 목록 (Tauri) — 35개

### 설정 / 프로파일

| Command | 설명 |
|---------|------|
| `get_app_config` | 앱 설정 조회 (키 마스킹, 모드) |
| `check_config` | API 설정 진단 (ConfigDiagnostic 반환) |
| `list_profiles` | 멀티 계좌 프로파일 목록 조회 |
| `add_profile` | 프로파일 추가 |
| `update_profile` | 프로파일 수정 |
| `delete_profile` | 프로파일 삭제 |
| `set_active_profile` | 활성 프로파일 전환 (자동매매 중이면 UI active_id만 변경, REST 클라이언트 보존) |
| `get_web_config` | 웹 서버 포트 설정 조회 |
| `save_web_config` | 웹 서버 포트 저장 (.env) |

### 시세 / 주문

| Command | 설명 |
|---------|------|
| `get_balance` | 잔고 조회 (BalanceSummary + items) |
| `get_price` | 종목 현재가 조회 |
| `get_chart_data` | 종목 차트 데이터 조회 (일봉) |
| `get_overseas_chart_data` | 해외주식 기간별 차트 데이터 조회 (일/주/월봉) |
| `place_order` | 수동 주문 (매수/매도) |
| `get_today_executed` | 당일 체결 내역 (KIS API) |
| `search_stock` | 종목명/코드 검색 (캐시된 KRX 목록) |
| `refresh_stock_list` | KRX 종목 목록 강제 갱신 |
| `get_kis_executed_by_range` | KIS API 날짜 범위 체결 조회 |

### 거래 기록 / 통계

| Command | 설명 |
|---------|------|
| `get_today_trades` | 당일 저장된 거래 기록 조회 |
| `get_trades_by_range` | 날짜 범위 거래 기록 조회 (JSON 파일) |
| `get_today_stats` | 당일 통계 조회 |
| `get_stats_by_range` | 날짜 범위 통계 조회 |
| `save_trade` | 체결 기록 JSON 저장 |
| `upsert_daily_stats` | 일별 통계 저장/갱신 |

### 자동 매매

| Command | 설명 |
|---------|------|
| `get_trading_status` | 자동 매매 실행 상태 조회 (wsConnected 포함) |
| `start_trading` | 자동 매매 시작 + WebSocket 연결 + **폴링 루프** spawn (`ws-status` 이벤트 emit) |
| `stop_trading` | 자동 매매 정지 |
| `get_positions` | 포지션 목록 조회 |
| `get_strategies` | 전략 목록 조회 |
| `update_strategy` | 전략 파라미터 업데이트 |

### 로그

| Command | 설명 |
|---------|------|
| `get_log_config` | 로그 설정 조회 (보관 기간, 최대 용량) |
| `set_log_config` | 로그 설정 저장 |
| `write_frontend_log` | 프론트엔드 로그 → 백엔드 파일 기록 |
| `get_recent_logs` | 최근 로그 라인 조회 |
| `get_trade_archive_config` | 체결 기록 보관 설정 조회 (보관 기간, 최대 용량) |
| `set_trade_archive_config` | 체결 기록 보관 설정 저장 + 즉시 정리 실행 |
| `get_trade_archive_stats` | 체결 기록 저장 통계 (파일 수, 용량, 날짜 범위) |

### 알림 / 업데이트

| Command | 설명 |
|---------|------|
| `send_test_discord` | Discord 테스트 알림 전송 |
| `check_for_update` | GitHub Releases API 버전 확인 |

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

### 심각한 오류 발생 시

```
오류 감지 (panic hook / 재시도 초과)
    ↓
notifications/discord.rs — CRITICAL/ERROR 레벨 알림 전송
    ↓
logging/mod.rs — error.log 기록
    ↓
trading/mod.rs — 자동 매매 비상 정지
```

---

## 6. 환경변수 및 설정 파일

### `.env` (git ignore)

```
VITE_API_URL=http://localhost:1420
KIS_APP_KEY=실전투자_앱키
KIS_APP_SECRET=실전투자_앱시크릿
KIS_ACCOUNT_NO=12345678-01
KIS_IS_PAPER_TRADING=false   # 기본값: 실전투자
```

### `secure_config.json` (git ignore) — `secure_config.example.json` 참고

```json
{
  "kis_app_key": "실전투자 APP KEY",
  "kis_app_secret": "실전투자 APP SECRET",
  "kis_account_no": "12345678-01",
  "kis_paper_app_key": "모의투자 APP KEY (선택)",
  "kis_paper_app_secret": "모의투자 APP SECRET (선택)",
  "kis_paper_account_no": "모의투자 계좌 (선택)",
  "is_paper_trading": false,
  "discord_bot_token": "",
  "discord_channel_id": "",
  "notification_levels": ["CRITICAL", "ERROR", "TRADE"]
}
```

> **우선순위**: `secure_config.json` > `.env` 환경변수 > 기본값

---

## 7. 외부 의존 서비스

| 서비스 | 용도 | 문서 |
|--------|------|------|
| 한국투자증권 Open API | REST + WebSocket 주식 거래 | https://apiportal.koreainvestment.com |
| Discord Bot API | 알림 전송 | `docs/discord-setup-guide.md` |

---

## 8. 변경 이력

| 날짜 | 변경 내용 | 작성자 |
|------|----------|--------|
| 2025-07-04 | 최초 생성 (Phase 1~6 완료 상태) | AI Agent |
| 2025-07-16 | Phase 7: 듀얼 키 설정, check_config IPC, Settings UI 진단, secure_config.example.json | AI Agent |
| 2026-04-04 | Phase 7 완료 확인. Phase 8 주요 기능 반영: market/server/updater 모듈, 35개 IPC 커맨드 목록 전면 갱신, scripts/setup-local.sh, .nvmrc, .cargo/config.toml 크로스플랫폼 처리, TodoList 동기화 | AI Agent |
| 2026-04-07T17:48:01 | 타임스탬프 형식 날짜→datetime(YYYY-MM-DDTHH:MM:SS) 전환(agent.md+4개SKILL.md+copilot-instructions.md), .vscode/settings.json 생성(rust-analyzer linkedProjects+TSsdk), strategy.rs apply_saved_configs 프로필 전환 시 전략 기본값 리셋 버그 수정, release.yml releaseName 앱명 수정(AutoConditionTrade→KISAutoTrade), .github/workflows/release.yml+.vscode/ 디렉토리 맵 추가 | AI Agent |
| 2026-04-08 | 체결 기록 보관 기능 추가: TradeArchiveConfig 구조체+commands.rs 3개 커맨드 (get/set/stats), lib.rs 등록, types.ts+commands.ts+hooks.ts 프론트엔드 연동, Dashboard FilledOrdersPanel(날짜 범위 조회), Settings 체결 기록 보관 섹션, discord.rs 미사용 import 제거 | AI Agent |

---

> **에이전트에게**: 이 파일을 읽었으면 작업을 시작하세요.  
> 작업 완료 후 **8. 변경 이력** 섹션과 **2. 전체 디렉토리 맵**을 반드시 업데이트하세요.
