# AutoConditionTrade — Agent Harness Map

> ⚠️ **에이전트 필독 규칙**
> - 모든 작업 시작 전 이 파일을 반드시 읽는다.
> - 파일 추가/삭제/이동, 구조 변경, 모듈 추가 후 이 파일을 즉시 업데이트한다.
> - 이 파일이 최신 상태가 아니라면 작업 전에 먼저 갱신한다.
> - 추측이 아닌 이 맵을 기반으로 작업한다.

**마지막 업데이트**: 2026-04-10T12:00:00  
**프로젝트 상태**: Phase 1~7 완료 / Phase 8+ 진행 중 (WebSocket Dashboard 연동 ✅, 추가 전략 RSI·모멘텀·이격도·52주신고가·연속상승·돌파실패·강한종가·변동성확장·평균회귀·추세필터·**가격조건** ✅, 전략 설정 프로파일별 영구 저장 ✅, GitHub Actions 자동 빌드 ✅, 체결 기록 보관 설정+대시보드 조회 ✅, 네비게이션 단일화(Sidebar only) ✅, 모바일 완전 동일 기능 REST API ✅, 비상정지 수동 발동/해제 버튼 ✅, 해외잔고 USD/KRW 토글 ✅, 실시간 환율 + REFRESH_INTERVAL_SEC ✅, 체결사유 필수 기록 ✅, 해외잔고 항상 표시 + 가격조건매매 전략 ✅, **해외주식 USD 가격 스케일 수정** ✅, **리스크 관리 enabled on/off + 순손실 계산** ✅, **잔고부족 매수정지(buy_suspended)** ✅)

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
│       ├── Dashboard.tsx             ← 잔고/수익 카드, 포지션 테이블, 미체결/체결 주문, 리스크 관리(접기/펼치기) ✅
│       ├── Trading.tsx               ← 수동 매수/매도 폼 + 종목 검색 + 체결 내역 ✅
│       ├── Strategy.tsx              ← 전략 ON/OFF, 파라미터 설정 (11개 전략 범용 UI, STRATEGY_PARAM_META) ✅
│       ├── History.tsx               ← 날짜 범위 조회, 자동매매 체결 기록, 통계 요약 ✅
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
│       │   ├── rest.rs               ← KisRestClient — get_balance, get_overseas_balance, place_order, place_overseas_order, get_today_executed_orders, get_price, get_chart_data, get_overseas_price, get_overseas_chart_data ✅
│       │   ├── token.rs              ← TokenManager — issue_token, get_token, is_expired (auto-refresh) ✅
│       │   └── websocket.rs          ← KisWebSocketClient — subscribe (WsStatusEvent emit, ws-status Tauri 이벤트) ✅
│       ├── market/
│       │   └── mod.rs                ← KRX 종목 목록 StockList (CSV 파싱, 캐시, 이름/코드 검색) ✅
│       ├── market_hours.rs           ← 시장 개장 여부 판단 (KRX 09:00-15:30 KST / US 22:00-07:00 KST), is_domestic_symbol, is_market_open_for, open_markets_summary ✅
│       ├── server/
│       │   └── mod.rs                ← axum 웹 서버 (ServeDir, REST proxy — WEB_PORT 기본 7474) ✅
│       ├── updater/
│       │   └── mod.rs                ← GitHub Releases API 버전 확인 (check_for_update IPC) ✅
│       ├── trading/
│       │   ├── mod.rs                ← 장 시간 감지, 전략 루프 실행 ✅
│       │   ├── strategy.rs       ← Strategy trait, 10개 전략 (MA Cross·RSI·모멘텀·이격도·52주신고가·연속상승하락·돌파실패·강한종가·변동성확장·평균회귀·추세필터), StrategyManager ✅
│       │   ├── order.rs              ← OrderManager: submit_signal(exchange, tick_price) → place_order(국내)/place_overseas_order(해외) 자동 분기, on_fill → TradeStore+OrderStore 저장, confirm_fill_by_symbol (시장가 자동 확인), EGW00201 재시도, 빈 ondo UUID fallback, **buy_suspended(잔고부족 매수정지 플래그)** ✅
│       │   ├── position.rs           ← PositionTracker (on_buy/on_sell/load_if_empty — 앱 재시작 시 잔고 API로 복원) ✅
│       │   └── risk.rs               ← RiskManager (**enabled on/off**, emergency_stop, **record_pnl(순손실=총손실-수익)**, check_position_size, trigger/clear_emergency_stop, reset_if_new_day) ✅
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
| `market_hours.rs` | 시장 개장 여부 판단 (KRX·US), 폴링 루프에서 폐장 시 API 호출 자동 건너뜀 |
| `commands.rs::run_trading_daemon` | **자동매매 폴링 데스크💤 (lib.rs 시작 시 spawn, is_trading=false일 때 5수 슬립, true일 때 자동 재개)** |
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
| `get_balance` | 국내 잔고 조회 (BalanceSummary + items, position_tracker 동기화 포함) |
| `get_overseas_balance` | 해외 잔고 조회 (OverseasBalanceItem[] + summary, TR TTTS3012R) |
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
| `get_trading_status` | 자동 매매 실행 상태 조회 (wsConnected, buySuspended 포함) |
| `start_trading` | 자동 매매 시작 (is_trading=true 설정) + WebSocket 연결 — 폴링은 `run_trading_daemon` 영구 데몬이 담당 (`ws-status` 이벤트 emit) |
| `stop_trading` | 자동 매매 정지 |
| `clear_buy_suspension` | 잔고 부족 매수 정지 수동 해제 (입금 후 사용자 요청 시) |
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
| 2026-04-08T14:00:00 | KIS체결내역 제거: Dashboard useTodayExecuted+KIS섹션 삭제, History.tsx KIS탭(Tab0) 제거→로컬 기록 단독뷰; 리스크 관리 이동: Strategy.tsx RiskPanel→Dashboard.tsx 접기/펼치기 Collapse 패널; docs/user-guide.md 상승장 전략 가이드 섹션 추가(EGW00201 분석, 전략 추천/비추천 표) | AI Agent |
| 2026-04-08T16:00:00 | 시장 시간표 자동 제어: market_hours.rs 신규 모듈 (KRX 09:00-15:30 KST / US 22:00-07:00 KST), lib.rs 등록, commands.rs 폴링 루프에 전체 폐장 5분 대기 + 종목별 skip 게이팅 추가, is_domestic_symbol commands.rs→market_hours.rs 이동, user-guide.md 섹션7(시장 시간표 자동 제어) 추가 | AI Agent || 2026-04-08T18:00:00 | UI 4가지 개선: (1) Dashboard 보유 종목 상단 문제 (2) AppShell TopBar + 자동매매 상태 칩(FiberManualRecord pulse) (3) Settings SliderWithInput 컴포넌트(4개 Slider 교체) (4) 모바일 웹 자동매매: run_trading_daemon 폴링데스크 lib.rs에서 영구 spawn, server/mod.rs /api/trading/* REST 라우트, transport.ts 매핑, MOBILE_HTML start/stop 버튼 추가 | AI Agent |
| 2026-04-08T20:00:00 | 네비게이션 단일화 + 모바일 완전 동일 기능화: (1) AppShell TopBar 제거 → 모바일 전용 미니 AppBar(햄버거+타이틀만), Sidebar DrawerContent에 자동매매 상태 칩+pulse 이전 — 데스크탑=사이드바 only, 모바일=AppBar+Drawer (2) server/mod.rs ServerState 12개 Arc 필드 추가(config, profiles, trade/stats_store, log_config, trade_archive_config, risk/order_manager, stock/strategy_store 등), 18개 신규 REST 엔드포인트 (/api/app-config, /api/profiles, /api/positions, /api/today-stats, /api/stats, /api/trades, /api/kis-executed, /api/pending-orders, /api/log-config, /api/recent-logs, /api/archive-config, /api/archive-stats, /api/risk-config, /api/risk-config/clear-emergency, /api/web-config, /api/strategies/:id) (3) lib.rs 서버 spawn 18개 Arc 전달 (4) transport.ts resolveRest() 16개 신규 케이스 추가 — 모든 페이지(Dashboard/Trading/Strategy/History/Settings/Log) 웹 모드 완전 지원 | AI Agent |
| 2026-04-09T00:00:00 | 3개 버그 수정: (1) Dashboard 보유 주식 미표시 — PositionTracker.load_if_empty() 추가, get_balance/get_overseas_balance 호출 시 tracker 복원 (2) 해외 자동매수 실패 — fetch_overseas_tick 거래소 반환(NAS/NYS/AMS), submit_signal(exchange, tick_price) 파라미터 추가, OrderManager.process_buy/sell 국내(시장가)/해외(지정가USD) 분기, place_overseas_with_retry 추가, NAS→NASD 등 주문코드 변환 (3) 해외 잔고 미표시 — rest.rs OverseasBalanceItem/Summary/Response+get_overseas_balance(TR TTTS3012R), get_overseas_balance IPC커맨드, lib.rs 등록, types.ts/commands.ts/hooks.ts 연동, Dashboard 해외보유주식 섹션 추가(USD 표시), server/mod.rs /api/overseas-balance 엔드포인트, transport.ts 매핑 | AI Agent |
| 2026-04-09T12:30:00 | 4가지 기능 개선: (1) 비상정지 수동 발동 — risk.rs trigger_emergency_stop(), commands.rs activate_emergency_stop IPC, lib.rs 등록, commands.ts/hooks.ts useActivateEmergencyStop 추가 (2) Dashboard RiskPanel Collapse 제거 → 항상 펼침, 비상정지 발동/해제 버튼 토글 (하락장 대응) (3) 해외 보유주식 USD/KRW 토글 — KRW_RATE=1450 근사 환산, 헤더 버튼 2개 (4) user-guide.md 섹션8 하락장 방어전략(2026-04-09) + 섹션9 FAQ(수수료 0원 이유) 추가 | AI Agent |
| 2026-04-09T15:00:00 | 2가지 기능 추가: (1) 실시간 환율 + 공통 갱신주기 — rest.rs fetch_usd_krw_rate()(open.er-api.com), AppState exchange_rate_krw(Arc<RwLock<f64>>)+refresh_interval_sec(u64), REFRESH_INTERVAL_SEC 환경변수(기본 30초/최소 5초), lib.rs 환율 갱신 데뼼(4번 백그라운드 태스크), get_exchange_rate/get_refresh_interval IPC, useExchangeRate/useRefreshInterval 훅, Dashboard KRW_RATE 상수 제거→동적환율, useBalance/useOverseasBalance/useTodayStats refetchInterval 동적화 (2) 체결사유 필수 기록 — TradeRecord signal_reason 필드(serde default), on_fill() pending.signal_reason 전달, Dashboard FilledOrdersPanel 체결사유 커럼메 | AI Agent |
| 2026-04-09T16:00:00 | 2가지 개선: (1) 해외 보유주식 패널 항상 표시 — Dashboard overseasBalance 조건부 숨김 제거, isLoading/isError 추출, 로딩/에러/빈 상태 메시지 표시 (기존: 해외주식 없으면 섹션 자체 숨겨져 API 오류 디버깅 불가) (2) 가격 조건 매매 전략 — strategy.rs PriceConditionStrategy(buy_trigger_price 이하 매수 / sell_trigger_price 이상 지정가익절 / take_profit_pct% 비율익절 / stop_loss_pct% 손절), commands.rs 등록, Strategy.tsx 파라미터 메타+설명 추가 | AI Agent |
| 2026-04-09T18:00:00 | 4가지 개선: (1) **해외주식 USD 가격 스케일 수정(크리티컬 버그)** — fetch_overseas_tick이 USD×100(cents)을 on_tick에 전달하지만 buy_trigger_price가 USD face value로 저장돼 비교 시 항상 false였던 버그 수정. PriceConditionSymbolConfig에 is_overseas:bool 추가, on_tick에서 is_overseas=true이면 threshold×100으로 스케일 변환 후 비교, reason 문자열도 USD/원 분기 (2) **Strategy UI — market 전달 + USD 표시** — PriceConditionEditorPanel에 market prop 추가, handleAdd에서 is_overseas:market==='US' 자동 설정, 가격 칼럼 헤더 "매수가(원/$)", 입력칸에 원/$endAdornment, 해외 종목 행에 "$" 파란 배지, 가격 칼럼 minWidth 70→100, 테이블 minWidth 500→600, step 국내=100/해외=0.01 동적 (3) **중복 주문 방지** — in_position 플래그가 on_tick 내에서 즉시 true 설정(비동기 제출 전)되므로 이미 방지됨. 추가 변경 없음 (4) **전략 카드 시각 정리** — 카드 하단 verbose 설명 박스를 Tooltip 아이콘("전략 설명 보기")으로 교체, 카드 세로 공간 대폭 축소 | AI Agent || 2026-04-10T10:00:00 | 리스크 관리 개편: (1) **RiskManager enabled on/off** — enabled 필드 추가(#[serde(default="default_true")]), Settings페이지 Switch로 토글, 비활성 시 대시보드 리스크 패널 숨김(RiskPanelWrapper), 비활성 시 자동 비상정지 스킵 (2) **순손실 계산** — daily_profit 필드 추가, net_loss()=총손실-당일수익, record_pnl(수익도 추적), 순손실이 한도 이상일 때만 비상정지 (3) commands.rs build_risk_view() 헬퍼 함수+RiskConfigView 업데이트(enabled/dailyProfit/netLoss) (4) Settings.tsx RiskSection 컴포넌트 추가(Slider 한도 설정+오늘 현황 요약) | AI Agent |
| 2026-04-10T12:00:00 | **잔고부족 매수정지(buy_suspended) 구현**: (1) order.rs OrderManager.buy_suspended/buy_suspended_reason 필드, process_buy에서 is_insufficient_balance_error() 감지(APBK0013/APBK0915/APBK0017+msg 키워드) 시 플래그 세팅+Ok(()) 반환, on_fill Sell 체결 시 자동 해제, reset_day()에 자동 초기화 추가 (2) commands.rs TradingStatus에 buySuspended/buySuspendedReason 필드, clear_buy_suspension 커맨드 신규 (3) lib.rs 커맨드 등록 (4) server/mod.rs 거래상태 JSON에 buySuspended/buySuspendedReason 추가 (5) types.ts/commands.ts/hooks.ts 프론트엔드 연동(useClearBuySuspension 훁) (6) Dashboard/Trading 페이지에 경고 Alert+매수재개 버튼 UI (7) 스킬 파일 업데이트: kis-api/SKILL.md 실제 APBK* 에러코드+buy_suspended/RiskManager 패턴 신규 섹션 14~15 추가 | AI Agent |---

> **에이전트에게**: 이 파일을 읽었으면 작업을 시작하세요.  
> 작업 완료 후 **8. 변경 이력** 섹션과 **2. 전체 디렉토리 맵**을 반드시 업데이트하세요.
