# KISAutoTrade — Todo List

> Phase별 진행 현황 및 미완성 작업 목록입니다.  
> 작업 완료 시 즉시 업데이트합니다.

---

## 진행 현황 요약

| Phase | 내용 | 상태 |
|-------|------|------|
| Phase 1 | 기반 구성 | ✅ 완료 |
| Phase 2 | KIS API 연동 | ✅ 완료 |
| Phase 3 | 매매 기능 | ✅ 완료 |
| Phase 4 | 자동 매매 전략 | ✅ 완료 |
| Phase 5 | 알림 시스템 | ✅ 완료 |
| Phase 6 | 로그 및 통계 | ✅ 완료 |
| Phase 7 | 최적화 및 배포 | 🔄 진행 중 |
| Phase 8+ | 고도화 | 🔄 진행 중 |

---

## Phase 1 — 기반 구성 ✅

- [x] Tauri v2 + React 프로젝트 초기화
- [x] Theme 시스템 구현 (light/dark/system)
- [x] TanStack Router 설정 및 기본 라우팅
- [x] MUI 레이아웃 기본 틀 (AppShell, Sidebar)
- [x] `agent.md` 최초 생성

---

## Phase 2 — API 연동 ✅

- [x] KIS REST API 클라이언트 (`src-tauri/src/api/rest.rs`)
  - get_balance, place_order, get_today_executed_orders, get_price
- [x] 토큰 자동 갱신 로직 (`src-tauri/src/api/token.rs`)
  - POST /oauth2/tokenP, 만료 5분 전 자동 갱신
- [x] 계좌 조회 (잔고, 보유종목)
- [x] IPC Command 연결 (Tauri ↔ React)

---

## Phase 3 — 매매 기능 ✅

- [x] 수동 주문 UI (`src/pages/Trading.tsx`)
- [x] WebSocket 실시간 시세 수신 (`src-tauri/src/api/websocket.rs`)
  - H0STCNT0 구독, 파이프 구분 파싱
- [x] 체결 내역 JSON 저장 — `save_trade` IPC 커맨드
- [x] 일별 잔고 스냅샷 — `upsert_daily_stats` IPC 커맨드

---

## Phase 4 — 자동 매매 전략 ✅

- [x] Strategy trait + StrategyManager (`src-tauri/src/trading/strategy.rs`)
- [x] 이동평균 골든/데스 크로스 전략 (MovingAverageCrossStrategy)
- [x] 리스크 관리 (`src-tauri/src/trading/risk.rs`)
  - 일일 손실 한도, 긴급 정지, 포지션 비율 관리
- [x] 전략 ON/OFF UI (`src/pages/Strategy.tsx`)

---

## Phase 5 — 알림 시스템 ✅

- [x] Discord 봇 클라이언트 (`src-tauri/src/notifications/discord.rs`)
- [x] 알림 레벨별 메시지 포맷 (`src-tauri/src/notifications/types.rs`)
- [x] Settings UI에서 Discord 연결 상태 표시 + 테스트 전송
- [x] `docs/discord-setup-guide.md` 작성

---

## Phase 6 — 로그 및 통계 ✅

- [x] Tracing 로그 시스템 (`src-tauri/src/logging/mod.rs`)
- [x] 일별/월별 통계 JSON 저장 (`src-tauri/src/storage/stats_store.rs`)
- [x] History 화면 — 날짜 범위 조회, 통계 요약, 거래 테이블
- [x] Log 화면 — 레벨 필터(ALL/DEBUG/INFO/WARN/ERROR), 검색, 색상 구분

---

## Phase 7 — 최적화 및 배포 🔄

- [x] Vite 청크 분리 (`vite.config.ts` — vendor/mui/tanstack 별도 청크)
- [x] 실전/모의투자 API 키 통합 설정 (`secure_config.json` 기반)
- [x] `check_config` IPC 커맨드 — 진단 모드
- [x] Settings 화면 개선 — 진단 정보 표시
- [x] `secure_config.example.json` 템플릿 파일 생성
- [x] 에러 핸들링 강화 — Dashboard 설정 미비 경고 배너
- [x] README.md 작성, MIT 라이선스 적용
- [x] GitHub Releases API 버전 확인 기능 (updater 모듈)
- [x] 앱 빌드 모드 세팅 (`pnpm build:app`, NSIS Windows 번들)
- [x] 앱 이름 KISAutoTrade로 변경
- [x] pnpm build → 전체 앱 빌드 (`tauri build`)
- [x] 웹 모드 단독 동작 지원 (axum ServeDir + transport layer)
- [x] Settings 웹 접속 포트 설정 + .env 저장
- [x] Trading 종목 검색 UI 개선 (돋보기 버튼, 로딩 인디케이터)
- [x] Trading KR 종목 검색 결과 테이블 표시 ✅ 2026-04-06 (TextField + Paper/Table 드롭다운 방식으로 교체)
- [x] KRX 종목 목록 로드/캐시 (`market/mod.rs`, `search_stock` IPC)
- [x] 로그 설정 UI (`get_log_config` / `set_log_config` IPC — 보관 기간·최대 용량)
- [x] 프론트엔드 로그 백엔드 전달 (`write_frontend_log` IPC)
- [x] `agent.md` 최종 정리 ✅ 2026-04-04

---

## Phase 8+ — 고도화 🔄

- [x] 자동 매매 start/stop IPC 커맨드 구현
- [x] 포지션 정보 UI 표시 (`get_positions` IPC + Dashboard 포지션 테이블)
- [x] 전략 IPC 연결 (`get_strategies`, `update_strategy`) + Strategy 페이지 실제 연동
- [x] Dashboard 설정 미비 경고 배너 + 자동매매 start/stop 버튼
- [x] 멀티 계좌 프로파일 관리 (Settings 화면) — add/update/delete/set_active_profile
- [x] 멀티프로필 동작 기능 강화 ✅ 2026-04-07
  - `AppState.trading_profile_id`: 자동매매 시작 시점 프로파일 ID 스냅샷 저장
  - `TradingStatus.trading_profile_id`: 현재 동작 중인 프로파일 ID 반환
  - `set_active_profile`: 자동매매 실행 중 프로파일 전환 시 REST 클라이언트 교체 방지 (UI active_id만 변경)
  - `Settings.tsx`: 동작 중인 프로파일 카드에 "동작중" Chip 표시, 실행 중 경고 배너 추가
- [x] 날짜 범위 KIS API 체결 조회 (`get_kis_executed_by_range` IPC)
- [x] 최근 로그 조회 (`get_recent_logs` IPC)
- [x] 차트 데이터 조회 (`get_chart_data` IPC — lightweight-charts v5)
- [x] 해외(미국) 주식 현재가/주문 IPC 커맨드 (`get_overseas_price`, `place_overseas_order`)
- [x] Transport layer (Tauri IPC ↔ Web REST 듀얼 모드 `transport.ts`)
- [x] 해외 주식 차트 (`OverseasStockChart.tsx`, `get_overseas_chart_data` IPC, `/api/overseas-chart` 웹 엔드포인트)
- [x] README OS별 프로필/AppData 경로 안내 상세화
- [x] copilot-instructions.md 경고 해소 의무 지침 강화
- [x] WebSocket 연결 상태 Tauri Event emit → Dashboard 실시간 반영 ✅ 2026-04-06
  - [x] `WsStatusEvent { connected, message }` Rust 구조체 + Tauri `ws-status` emit
  - [x] `AppState.ws_connected: Arc<AtomicBool>` 관리
  - [x] `start_trading` 시 `KisWebSocketClient` 생성 및 subscribe 연동
  - [x] `TradingStatus.wsConnected` 필드 추가
  - [x] Dashboard: `listen('ws-status')` 훅 + WS Chip 표시
  - [x] `StrategyManager.active_symbols()` 메서드 추가
- [x] `trading/order.rs` **OrderManager 구현** ✅ 2026-04-05
  - [x] ① 전략 신호 실행: `Signal::Buy/Sell` → KIS API `place_order()` 호출 (전략 루프에서 `StrategyManager`가 OrderManager 에 위임)
  - [x] ② 미체결 주문 풀: `HashMap<odno, PendingOrder>` — 주문 접수 후 체결/취소 확인 전까지 관리
  - [x] ③ 중복 주문 방지: 동일 종목에 Pending 상태 주문이 있으면 신규 주문 차단
  - [x] ④ 체결 이벤트 처리: `on_fill(odno, filled_qty, avg_price)` — WebSocket H0STCNI0 또는 폴링에서 호출
  - [x] ⑤ 포지션 연동: 체결 확인 시 `PositionTracker.on_buy/on_sell()` 호출
  - [x] ⑥ 주문 저장: 체결·취소 완료 시 `OrderStore.append()` 로 JSON 기록
  - [x] ⑦ 통계 연동: 매도 체결 손익 계산 후 `StatsStore.upsert()` 및 `RiskManager.record_pnl()` 호출
  - [x] ⑧ 리스크 검증: 주문 전 `RiskManager.can_trade()` + `check_position_size()` 통과 여부 확인
  - [x] ⑨ Discord 알림: 체결 완료 시 `NotificationEvent::trade()` 전송
  - [x] ⑩ 주문 재시도: KIS API rate-limit(EGW00133) 오류 시 1초 대기 후 최대 3회 재시도
- [x] 추가 전략 구현 (모멘텀, RSI, 이격도) ✅ 2026-04-06
  - [x] `RsiStrategy` — RSI 과매도(기본 30) 돌파 시 매수 / 과매수(기본 70) 하향 시 매도
  - [x] `MomentumStrategy` — N기간 전 대비 변화율 임계값 돌파 시 매매
  - [x] `DeviationStrategy` — 이격도(현재가/MA-1)*100 임계값 매매
  - [x] `commands.rs` — rsi_default / momentum_default / deviation_default 등록
  - [x] `Strategy.tsx` — 파라미터 UI 범용화 (STRATEGY_PARAM_META 드리프트 렌더링)
- [x] 국내 주식 검색 STOCK_LIST_EMPTY 진단 및 복구 UI ✅ 2026-04-07
  - [x] `search_stock` IPC: 목록 비어있을 때 `STOCK_LIST_EMPTY` 에러코드 반환 + 로그
  - [x] `market/mod.rs load_or_fetch`: 캐시/KRX 다운로드 상세 로그 추가
  - [x] `useStockSearch`: STOCK_LIST_EMPTY 오류 시 재시도 중단, 에러 타입 `CmdError`
  - [x] `useRefreshStockList`: 강제 새로고침 mutation — 성공 시 stockSearch 캐시 무효화
  - [x] Trading.tsx: 종목 목록 없을 때 Warning Alert + 새로고침 버튼 표시
- [x] Strategy.tsx 종목 검색 패널 + 테이블 UI ✅ 2026-04-07
  - [x] 상단 종목 선택 패널: 검색/드롭다운/선택 칩 (검색어 디바운스 350ms)
  - [x] 각 전략 카드: 종목 코드+이름 테이블 (삭제 버튼), "선택한 종목 추가" 버튼
  - [x] `symbolNames` 캐시로 코드→이름 표시 유지
  - [x] `EditState.symbols: string[]` — 직접 배열로 관리 (쉼표 구분 문자열 제거)
- [x] StockStore 영구 캐시 + Settings 종목 목록 관리 섹션 ✅ 2026-04-06
  - [x] `storage/stock_store.rs` — `StockEntry { name, updated_at }` + `StockStore` 영구 JSON 캐시 (`stocklist/stocklist.json`)
  - [x] `storage/mod.rs` — `pub mod stock_store` + `pub use StockStore` 등록
  - [x] `AppState.stock_store: Arc<StockStore>` — `AppState::new()` 초기화 연결
  - [x] `get_balance` IPC — 응답 `prdt_name` 에서 자동 upsert (incremental 수집)
  - [x] `get_price` IPC — 응답 `hts_kor_isnm` 에서 자동 upsert
  - [x] `search_stock` 4단계: StockStore → KRX 레거시 캐시 → NAVER 실시간(+upsert) → STOCK_LIST_EMPTY
  - [x] `get_stock_list_stats` / `set_stock_update_interval` IPC 추가
  - [x] `lib.rs` — KRX 백그라운드 로드 시 `stock_store.upsert_many` 동기화
  - [x] `types.ts` — `StockListStats` 인터페이스 추가
  - [x] `commands.ts` — `getStockListStats`, `setStockUpdateInterval` 래퍼 추가
  - [x] `hooks.ts` — `useStockListStats`, `useSetStockUpdateInterval` 훅 추가 / `useRefreshStockList` invalidation 갱신
  - [x] `Settings.tsx` — "종목 목록 관리" 섹션 추가 (통계 Chip, 자동 갱신 간격, KRX 다운로드 버튼, 경로 표시)
- [ ] 추가 프리셋 전략 구현 (나머지 6개)
  - [x] **03. 52주 신고가** (`FiftyTwoWeekHighStrategy`) ✅ 2026-04-06
    - `strategy.rs`: `FiftyTwoWeekHighParams { lookback_days, stop_loss_pct }` + `FiftyTwoWeekHighStrategy` 구현
    - `Strategy` trait에 `initialize_historical(symbol, prices)` 추가 (기본 no-op)
    - `StrategyManager::initialize_historical(symbol, prices)` — 해당 종목 타겟 전략에 일괄 전달
    - `commands.rs` `start_trading`: 활성 종목별 KIS 일봉 차트 400일치 로드 → `initialize_historical` 호출
    - `commands.rs` AppState: `FiftyTwoWeekHighStrategy` 기본 등록 (`fifty_two_week_high_default`)
    - `Strategy.tsx` `STRATEGY_PARAM_META` + `STRATEGY_DESCRIPTION` + `getStrategyType` 추가
    - **동작**: 자동매매 시작 시 KIS 차트 API로 52주 고가 자동 초기화 → 실시간 틱에서 돌파 감지 → 매수; 매수 후 stop_loss_pct% 하락 시 자동 손절
  - [x] **04. 연속 상승/하락** (`ConsecutiveMoveStrategy`) ✅ 2026-04-06
    - `strategy.rs`: `ConsecutiveMoveParams { buy_days, sell_days }` + `ConsecutiveMoveStrategy` 구현
    - N일 연속 종가 상승(prev < cur 비교) 시 매수, M일 연속 하락 시 매도
    - `in_position` 플래그로 중복 매수 방지; 매도 후 포지션 해제
    - `commands.rs`: `consecutive_move_default` 기본 등록
    - `Strategy.tsx`: `STRATEGY_PARAM_META`, `STRATEGY_DESCRIPTION`, `getStrategyType` 추가
    - Trading/Strategy 검색창: "국내 주식은 6자리 종목코드로만 검색 가능" 안내 추가 및 이름 입력 차단
  - [x] **06. 돌파 실패** (`FailedBreakoutStrategy`) — 전고점 돌파 후 당일 종가가 다시 전고점 아래로 내려오면 손절/매도 ✅ 2026-04-07
    - 파라미터: `lookback_days: usize = 20`, `buffer_pct: f64 = 0.5`
    - 조건: 전고점 × (1 + buffer_pct/100) 이상 → 매수, 현재가 < 전고점 → 이탈 → 매도
    - `commands.rs`: `failed_breakout_default` 기본 등록
    - `Strategy.tsx`: `STRATEGY_PARAM_META`, `STRATEGY_DESCRIPTION`, `getStrategyType` 추가
  - [x] **07. 강한 종가** (`StrongCloseStrategy`) — 종가가 당일 고가 대비 N% 이내(강한 마감)이면 다음날 매수 ✅ 2026-04-07
    - 파라미터: `threshold_pct: f64 = 3.0`, `stop_loss_pct: f64 = 3.0`
    - 자동매매 시작 시 `initialize_candles`로 전일 일봉 (고가, 종가) 전달 → 강한 종가 감지
    - 강한 종가 확인 시 `pending_buy` 설정 → 당일 첫 틱에서 매수
    - 매수 후 `stop_loss_pct%` 하락 시 손절 매도
    - `commands.rs`: `strong_close_default` 기본 등록, `start_trading` `initialize_candles` 호출 추가
    - `Strategy.tsx`: `STRATEGY_PARAM_META`, `STRATEGY_DESCRIPTION`, `getStrategyType` 추가
  - [ ] **08. 변동성 확장** (`VolatilityExpansionStrategy`) — N일 평균 변동성(일봉 고-저 범위) 대비 당일 변동성이 K배 이상이면 방향에 따라 매수/매도
    - 파라미터: `lookback_days: u32 = 10`, `expansion_factor: f64 = 2.0`
    - 조건: 당일 변동폭 > 평균 변동폭 × expansion_factor AND 종가 > 시가 → 매수
  - [ ] **09. 평균회귀** (`MeanReversionStrategy`) — 현재가가 MA에서 N 표준편차 이상 이탈 시 반대 방향으로 매매 (볼린저 밴드 기반)
    - 파라미터: `period: u32 = 20`, `std_dev: f64 = 2.0`
    - 조건: 현재가 < 하단밴드 → 매수 / 현재가 > 상단밴드 → 매도
  - [ ] **10. 추세 필터** (`TrendFilterStrategy`) — 장기 MA(기본 200일) 위에서 단기 상승 신호(5일 MA > 20일 MA) 시에만 매수
    - 파라미터: `long_period: u32 = 200`, `short_period: u32 = 5`, `mid_period: u32 = 20`
    - 조건: 현재가 > long_MA AND short_MA > mid_MA → 매수; 현재가 < long_MA → 청산
  - [ ] 각 전략을 `commands.rs` 의 `*_default()` 헬퍼로 등록 (프리셋 기본값)
  - [ ] `Strategy.tsx` — 신규 전략 파라미터 메타 (`STRATEGY_PARAM_META`) 추가
- [ ] GitHub Actions CI/CD (Windows + macOS 자동 빌드 & 릴리스)
- [ ] 웹 모드 고도화 (주문 REST API 추가, 인증 처리)
- [ ] 실전 매매 검증 (모의투자 완전 통과 후 실전 전환)
- [ ] 다중 종목 동시 전략 실행 (StrategyManager 확장)

---

## 환경 설정 참고

- **Node.js**: `>=20.0.0` (`.nvmrc`: 25.9.0)
- **Rust**: 1.93.1+
- **macOS 외장 드라이브 사용 시**: `./scripts/setup-local.sh` 실행 필수
  - `.cargo/config.toml` 자동 생성 (gitignore, 머신별 target 경로 분리)

---

*마지막 업데이트: 2026-04-06*
