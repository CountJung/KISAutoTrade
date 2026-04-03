# AutoConditionTrade — Todo List

> MasterPlan.md의 Phase별 진행 상황을 추출한 실시간 태스크 목록입니다.  
> 이 파일은 작업 진행 시 업데이트됩니다.

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
- [x] IPC Command 연결 12종 (`src-tauri/src/commands.rs`)

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
  - 실전/모의 별도 키 지원, `is_paper_trading` 플래그 (기본값: false = 실전투자)
- [x] `check_config` IPC 커맨드 — 진단 모드
  - API 설정 상태, 각 키 설정 여부, Discord 연결 상태
- [x] Settings 화면 개선 — 진단 정보 표시 (실전/모의 키 상태 Chip, 문제 목록)
- [x] `secure_config.example.json` 템플릿 파일 생성
- [x] 에러 핸들링 강화 — Dashboard 설정 미비 경고 배너, start_trading 미설정 시 에러 반환
- [x] README.md 작성, MIT 라이선스 적용, .gitignore 수정 (src-tauri/target 제거)
- [ ] Tauri 빌드 및 배포 (`cargo tauri build`)
- [ ] `agent.md` 최종 정리

---

## 다음 작업 후보 (Phase 8+)

> MasterPlan에 없지만 품질 향상을 위해 고려할 사항

- [x] 자동 매매 start/stop IPC 커맨드 구현 (`get_trading_status`, `start_trading`, `stop_trading`)
- [x] 포지션 정보 UI 표시 (`get_positions` IPC + Dashboard 포지션 테이블)
- [x] 전략 IPC 연결 (`get_strategies`, `update_strategy`) + Strategy 페이지 실제 연동
- [x] Dashboard 설정 미비 경고 배너 + 자동매매 start/stop 버튼
- [ ] WebSocket 연결 상태 Tauri Event emit → Dashboard 실시간 반영
- [ ] `trading/order.rs` OrderManager 구현 (현재 stubbed)
- [ ] 추가 전략 구현 (모멘텀, RSI, 이격도)
- [ ] 차트 컴포넌트 (History 페이지 일별 PnL 차트)

---

*마지막 업데이트: 2026-04-02*
