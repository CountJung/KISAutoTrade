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
- [x] 날짜 범위 KIS API 체결 조회 (`get_kis_executed_by_range` IPC)
- [x] 최근 로그 조회 (`get_recent_logs` IPC)
- [x] 차트 데이터 조회 (`get_chart_data` IPC — lightweight-charts v5)
- [ ] WebSocket 연결 상태 Tauri Event emit → Dashboard 실시간 반영
- [ ] `trading/order.rs` OrderManager 구현 (현재 stubbed)
- [ ] 추가 전략 구현 (모멘텀, RSI, 이격도)
- [ ] 차트 컴포넌트 고도화 (History 페이지 일별 PnL 차트 + candlestick 연동)
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

*마지막 업데이트: 2026-04-04*
