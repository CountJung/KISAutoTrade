# AutoConditionTrade — Agent Harness Map

> ⚠️ **에이전트 필독 규칙**
> - 모든 작업 시작 전 이 파일을 반드시 읽는다.
> - 파일 추가/삭제/이동, 구조 변경, 모듈 추가 후 이 파일을 즉시 업데이트한다.
> - 이 파일이 최신 상태가 아니라면 작업 전에 먼저 갱신한다.
> - 추측이 아닌 이 맵을 기반으로 작업한다.

**마지막 업데이트**: 2026-04-03  
**프로젝트 상태**: Phase 1~6 완료 / Phase 7 진행 중 (업데이트 확인 기능 + 앱 빌드 모드 세팅 완료)

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
├── MasterPlan.md                     ← 전체 설계 문서
├── Cargo.toml                        ← Rust workspace 루트 (resolver="2", profile 설정)
├── package.json                      ← npm 패키지 설정
├── vite.config.ts                    ← Vite 빌드 설정 (port:1420)
├── tsconfig.json / tsconfig.node.json
├── index.html                        ← HTML 진입점 (테마 Hydration 스크립트 포함)
├── .gitignore                        ← 민감 파일/데이터/빌드 제외
├── .env.example                      ← 환경변수 예시
├── secure_config.example.json        ← 민감 설정 템플릿 (실전/모의 키 포함) ✅
├── profiles.json                     ← 멀티 계좌 프로파일 (git ignore, 로컬 전용)
│
├── docs/
│   ├── discord-setup-guide.md        ← Discord 봇 연계 상세 가이드 ✅
│   └── api-reference.md              ← KIS API 참조 (추후 작성)
│
├── src/                              ← React Frontend (TypeScript)
│   ├── main.tsx                      ← React 진입점 (QueryClient, RouterProvider)
│   ├── router/
│   │   └── index.ts                  ← TanStack Router 코드 기반 라우팅 ✅
│   ├── api/
│   │   ├── types.ts                  ← Rust 타입 미러 (TypeScript, ConfigDiagnostic 포함) ✅
│   │   ├── commands.ts               ← invoke() 래퍼 함수 13종 (checkConfig 포함) ✅
│   │   └── hooks.ts                  ← TanStack Query 훅 모음 (useCheckConfig 포함) ✅
│   ├── theme/
│   │   └── index.ts                  ← createAppTheme, getResolvedMode, Hydration ✅
│   ├── store/
│   │   ├── settingsStore.ts          ← 테마/로그/Discord 설정 (zustand+persist) ✅
│   │   ├── accountStore.ts           ← 계좌 잔고 상태 ✅
│   │   └── tradingStore.ts           ← 자동매매 실행 상태 ✅
│   ├── components/
│   │   └── layout/
│   │       ├── AppShell.tsx          ← 전체 레이아웃 + ThemeProvider + Outlet ✅
│   │       └── Sidebar.tsx           ← MUI permanent/temporary Drawer ✅
│   └── pages/
│       ├── Dashboard.tsx             ← 잔고/수익 카드, 당일 거래 목록 (IPC 연결) ✅
│       ├── Trading.tsx               ← 수동 매수/매도 폼 + 체결 내역 ✅
│       ├── Strategy.tsx              ← MA Cross 전략 ON/OFF, 파라미터 설정 ✅
│       ├── History.tsx               ← 날짜 범위 조회, 통계 요약, 거래 테이블 ✅
│       ├── Log.tsx                   ← 레벨 필터, 검색, 색상 구분 로그 뷰어 ✅
│       ├── Settings.tsx              ← Discord 테스트, API 키 표시, 테마 설정, 멀티 계좌 프로파일 관리 ✅
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
│       │   ├── rest.rs               ← KisRestClient — get_balance, place_order, get_today_executed_orders, get_price ✅
│       │   ├── token.rs              ← TokenManager — issue_token, get_token, is_expired (auto-refresh) ✅
│       │   └── websocket.rs          ← KisWebSocketClient — subscribe, parse_realtime_price ✅
│       ├── trading/
│       │   ├── mod.rs
│       │   ├── strategy.rs           ← Strategy trait, MovingAverageCrossStrategy, StrategyManager ✅
│       │   ├── order.rs              ← OrderManager 스텁
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
│       │   └── mod.rs                ← tracing-appender (app.log, error.log) ✅
│       ├── commands.rs               ← AppState + IPC 커맨드 핸들러 (check_for_update 포함) ✅
│       └── config/
│           └── mod.rs                ← AccountProfile, ProfilesConfig, AppConfig(from_profile), DiscordConfig ✅
│       ├── updater/
│           └── mod.rs                ← GitHub Releases API 버전 확인 (check_for_update IPC) ✅
│
├── data/                             ← JSON 데이터 (git ignore)
├── log/                              ← 로그 파일 (git ignore)
├── target/                           ← Cargo 빌드 산출물 (git ignore)
├── node_modules/                     ← npm 패키지 (git ignore)
├── dist/                             ← Vite 빌드 산출물 (git ignore)
├── secure_config.json                ← 민감 설정 (git ignore) — 실전/모의 키 모두 포함 가능
├── secure_config.example.json        ← 설정 템플릿 (git 추적)
└── .env                              ← 환경변수 (git ignore)
```

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
| `storage/trade_store.rs` | `data/trades/YYYY/MM/DD/trades.json` 읽기/쓰기 |
| `storage/stats_store.rs` | 체결 집계 → `data/stats/YYYY/MM/daily_stats.json` |
| `notifications/discord.rs` | Discord Bot으로 알림 메시지 전송 |
| `config/mod.rs` | `secure_config.json` + `.env` 로드 (실전/모의 듀얼 키, 기본: 실전투자) |

---

## 4. IPC Command 목록 (Tauri)

| Command | 방향 | 설명 |
|---------|------|------|
| `get_balance` | Frontend → Backend | 잔고 조회 |
| `place_order` | Frontend → Backend | 수동 주문 |
| `get_orders` | Frontend → Backend | 주문 이력 조회 |
| `get_trades` | Frontend → Backend | 체결 기록 조회 (날짜 범위) |
| `get_daily_stats` | Frontend → Backend | 일별 통계 조회 |
| `start_trading` | Frontend → Backend | 자동 매매 시작 |
| `stop_trading` | Frontend → Backend | 자동 매매 정지 |
| `get_strategies` | Frontend → Backend | 전략 목록 조회 |
| `save_settings` | Frontend → Backend | 설정 저장 |
| `send_test_notification` | Frontend → Backend | Discord 테스트 알림 전송 |
| `get_app_config` | Frontend → Backend | 앱 설정 조회 (키 마스킹, 모드) |
| `check_config` | Frontend → Backend | API 설정 진단 (ConfigDiagnostic 반환) |

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

---

> **에이전트에게**: 이 파일을 읽었으면 작업을 시작하세요.  
> 작업 완료 후 **8. 변경 이력** 섹션과 **2. 전체 디렉토리 맵**을 반드시 업데이트하세요.
