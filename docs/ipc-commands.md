# AutoConditionTrade — IPC 커맨드 목록

> Tauri IPC 커맨드 전체 목록 (35개+). `commands.rs` 의 `#[tauri::command]` 함수들.  
> 신규 커맨드 추가 시 이 파일과 `lib.rs` 의 `generate_handler!` 를 함께 업데이트한다.

---

## 설정 / 프로파일

| Command | 설명 |
|---------|------|
| `get_app_config` | 앱 설정 조회 (키 마스킹, 모드) |
| `check_config` | API 설정 진단 (ConfigDiagnostic 반환) |
| `list_profiles` | 멀티 계좌 프로파일 목록 조회 |
| `add_profile` | 프로파일 추가 |
| `update_profile` | 프로파일 수정 |
| `delete_profile` | 프로파일 삭제 |
| `set_active_profile` | 활성 프로파일 전환 |
| `get_web_config` | 웹 서버 포트 설정 조회 |
| `save_web_config` | 웹 서버 포트 저장 (`.env` WEB_PORT) |
| `detect_trading_type` | 실전/모의투자 자동 감지 |
| `detect_profile_trading_type` | 특정 프로파일 실전/모의 감지 |

## 시세 / 주문

| Command | 설명 |
|---------|------|
| `get_balance` | 국내 잔고 조회 (BalanceSummary + items) |
| `get_overseas_balance` | 해외 잔고 조회 (OverseasBalanceItem[] + summary) |
| `get_price` | 종목 현재가 조회 |
| `get_chart_data` | 국내주식 차트 데이터 (일봉) |
| `get_overseas_price` | 해외주식 현재가 조회 |
| `get_overseas_chart_data` | 해외주식 차트 데이터 (일/주/월봉) |
| `place_order` | 국내 수동 주문 (매수/매도) |
| `place_overseas_order` | 해외 수동 주문 |
| `get_today_executed` | 당일 체결 내역 (KIS API) |
| `get_kis_executed_by_range` | KIS API 날짜 범위 체결 조회 |
| `get_exchange_rate` | USD/KRW 환율 조회 (캐시) |
| `search_stock` | 종목명/코드 검색 (캐시된 KRX 목록) |
| `refresh_stock_list` | KRX 종목 목록 강제 갱신 |
| `get_stock_list_stats` | 종목 목록 통계 |
| `set_stock_update_interval` | 종목 목록 갱신 주기 설정 |

## 거래 기록 / 통계

| Command | 설명 |
|---------|------|
| `get_today_trades` | 당일 저장된 거래 기록 조회 |
| `get_trades_by_range` | 날짜 범위 거래 기록 조회 (JSON 파일) |
| `get_today_stats` | 당일 통계 조회 |
| `get_stats_by_range` | 날짜 범위 통계 조회 |
| `save_trade` | 체결 기록 JSON 저장 |
| `upsert_daily_stats` | 일별 통계 저장/갱신 |
| `get_trade_archive_config` | 체결 기록 보관 설정 조회 |
| `set_trade_archive_config` | 체결 기록 보관 설정 저장 + 즉시 정리 |
| `get_trade_archive_stats` | 체결 기록 저장 통계 |

## 자동 매매

| Command | 설명 |
|---------|------|
| `get_trading_status` | 자동 매매 상태 조회 (wsConnected, buySuspended 포함) |
| `start_trading` | 자동 매매 시작 (is_trading=true + WebSocket 연결) |
| `stop_trading` | 자동 매매 정지 |
| `clear_buy_suspension` | 잔고 부족 매수 정지 수동 해제 |
| `get_positions` | 포지션 목록 조회 |
| `get_pending_orders` | 미체결 주문 조회 |
| `get_strategies` | 전략 목록 조회 |
| `update_strategy` | 전략 파라미터 업데이트 |
| `get_risk_config` | 리스크 설정 조회 |
| `update_risk_config` | 리스크 설정 변경 |
| `clear_emergency_stop` | 비상정지 수동 해제 |
| `activate_emergency_stop` | 비상정지 수동 발동 |

## 로그

| Command | 설명 |
|---------|------|
| `get_log_config` | 로그 설정 조회 (보관 기간, 최대 용량, api_debug) |
| `set_log_config` | 로그 설정 저장 |
| `write_frontend_log` | 프론트엔드 로그 → 백엔드 파일 기록 |
| `get_recent_logs` | 최근 로그 라인 조회 |

## 데이터 갱신 주기

| Command | 설명 |
|---------|------|
| `get_refresh_interval` | 갱신 주기(초) 단순 조회 |
| `get_refresh_config` | 갱신 주기 설정 전체 조회 |
| `set_refresh_config` | 갱신 주기 변경 (`.env` REFRESH_INTERVAL_SEC 저장 + 데몬 즉시 적용) |

## 알림 / 업데이트

| Command | 설명 |
|---------|------|
| `send_test_discord` | Discord 테스트 알림 전송 |
| `check_for_update` | GitHub Releases API 버전 확인 |

---

## Tauri 이벤트 (Backend → Frontend Push)

| 이벤트명 | 페이로드 | 발행 주체 |
|---------|--------|---------|
| `exchange-rate-updated` | `f64` (USD/KRW 환율) | 데몬 4 (환율 갱신) |
| `balance-updated` | `{ items, summary }` | 데몬 6 (잔고 갱신) |
| `overseas-balance-updated` | `{ items, summary }` | 데몬 6 (잔고 갱신) |
| `ws-status` | `WsStatusEvent` | `api/websocket.rs` |

> 프론트엔드 구독: `AppShell.tsx` → `useBackendEvents()` (`hooks.ts`)
