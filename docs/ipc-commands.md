# AutoConditionTrade — IPC 커맨드 목록

> Tauri IPC 커맨드 전체 목록 (35개+). `commands.rs` 의 `#[tauri::command]` 함수들.  
> 신규 커맨드 추가 시 이 파일과 `lib.rs` 의 `generate_handler!` 를 함께 업데이트한다.

---

## 설정 / 프로파일

| Command | 설명 |
|---------|------|
| `get_app_config` | 앱 설정 조회 (키 마스킹, 활성 broker/profile/account, 모드, `active_broker_configured`) |
| `check_config` | API 설정 진단 (ConfigDiagnostic 반환) |
| `list_profiles` | 멀티 계좌 프로파일 목록 조회 (`broker_id`, `broker_account_id`, `live_trading_consent` 포함) |
| `add_profile` | 프로파일 추가 (`live_trading_consent`는 토스 실거래 명시 동의 저장 상태) |
| `update_profile` | 프로파일 수정 (`live_trading_consent` 갱신 가능) |
| `delete_profile` | 프로파일 삭제 |
| `set_active_profile` | 활성 프로파일 전환 |
| `get_web_config` | 웹 서버 포트 설정 조회 |
| `save_web_config` | 웹 서버 포트 저장 (`.env` WEB_PORT) |
| `detect_trading_type` | 실전/모의투자 자동 감지 |
| `detect_profile_trading_type` | 특정 프로파일 실전/모의 감지 |
| `list_toss_accounts` | 입력한 토스증권 Client ID/Secret으로 `accountSeq` 후보 조회 |
| `list_toss_profile_accounts` | 저장된 토스증권 프로파일 키로 `accountSeq` 후보 조회 |
| `check_toss_profile_connection` | 토스증권 프로파일 연결 진단 (OpenAPI spec, token, accounts, holdings 단계 결과) |

## 시세 / 주문

| Command | 설명 |
|---------|------|
| `get_balance` | 국내 잔고 조회 (BalanceSummary + items) |
| `get_overseas_balance` | 해외 잔고 조회 (OverseasBalanceItem[] + summary) |
| `get_broker_holdings` | 활성 broker 보유 종목 조회 (`BrokerHoldingView[]`, Toss/KIS 공통 decimal 문자열 보존) |
| `get_price` | 종목 현재가 조회 |
| `get_toss_market_snapshot` | 활성 Toss 프로파일로 현재가/호가/최근 체결/상하한가 read-only snapshot 조회 |
| `get_toss_stock_safety` | 활성 Toss 프로파일로 종목 기본 정보와 매수 유의사항 조회 (`buyBlocked`, `buyBlockReason`) |
| `check_toss_order_preflight` | 활성 Toss 프로파일로 주문 전 검증 (`buyingPower`, `sellableQuantity`, `commissionRate`, `canSubmit`) |
| `list_toss_open_orders` | 활성 Toss 프로파일의 접수/미체결 주문 목록 조회 (`status=OPEN`, 수동 주문창 표시용) |
| `modify_toss_order` | 활성 Toss 프로파일의 접수 주문 정정 요청 (`orderId`, `orderType`, `quantity`, `price`) |
| `submit_toss_small_buy_verification` | Dashboard 전용 Toss 소액매매 검증. 실거래 동의/최종 확인/최대 허용금액/accountSeq 일치/사전검증/미체결 scan 후 검색 종목 1주 시장가 매수를 제출하고 주문·체결 기록을 저장 |
| `get_toss_market_calendar` | 활성 Toss 프로파일로 KR/US 정규장 캘린더 조회 (`regularSession`, `isRegularOpen`) |
| `get_toss_chart_data` | 활성 Toss 프로파일로 캔들 조회 (`1d`/`1m`, count 1~200, `ChartCandle[]`) |
| `get_chart_data` | 국내주식 차트 데이터 (일봉) |
| `get_overseas_price` | 해외주식 현재가 조회 |
| `get_overseas_chart_data` | 해외주식 차트 데이터 (일/주/월봉) |
| `place_order` | 국내 수동 주문 (매수/매도) |
| `place_overseas_order` | 해외 수동 주문 |
| `get_today_executed` | 당일 체결 내역 (KIS API) |
| `get_today_overseas_executed` | 당일 해외주식 주문체결 내역 (KIS API) |
| `get_kis_executed_by_range` | KIS API 날짜 범위 체결 조회 |
| `get_overseas_executed_by_range` | KIS API 날짜 범위 해외주식 주문체결 조회 |
| `get_exchange_rate` | USD/KRW 환율 조회 (숫자 캐시, 기존 UI 호환) |
| `get_exchange_rate_status` | USD/KRW 환율 조회 정책/출처 조회 (`source`, `fallbackUsed`, `validFrom`, `validUntil`) |
| `search_stock` | 종목명/코드 검색 (캐시된 KRX 목록) |
| `refresh_stock_list` | KRX 종목 목록 강제 갱신 |
| `get_stock_list_stats` | 종목 목록 통계 |
| `set_stock_update_interval` | 종목 목록 갱신 주기 설정 |

## 거래 기록 / 통계

| Command | 설명 |
|---------|------|
| `get_today_trades` | 당일 저장된 거래 기록 조회 |
| `get_trades_by_range` | 날짜 범위 거래 기록 조회 (JSON 파일, provider trace 포함) |
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
| `get_trading_status` | 자동 매매 상태 조회 (wsConnected, buySuspended, 실행 broker/profile/account 스냅샷 포함) |
| `start_trading` | 자동 매매 시작 (is_trading=true + WebSocket 연결) |
| `stop_trading` | 자동 매매 정지 |
| `clear_buy_suspension` | 잔고 부족 매수 정지 수동 해제 |
| `get_positions` | 포지션 목록 조회 |
| `get_pending_orders` | 미체결 주문 조회 (`status`, `filledQuantity`, `remainingQuantity`, provider trace 포함) |
| `get_strategies` | 전략 목록 조회 (`brokerId`, `brokerAccountId`, 대상 종목, params 포함) |
| `update_strategy` | 전략 파라미터 업데이트 + 현재 활성 broker/account scope로 저장 |
| `get_risk_config` | 리스크 설정 조회 (손실/비중/일일 주문 제한/연속 손실/ATR 수량 산정) |
| `update_risk_config` | 리스크 설정 변경 (손실/비중/일일 주문 제한/연속 손실/ATR 수량 산정) |
| `clear_emergency_stop` | 비상정지 수동 해제 |
| `activate_emergency_stop` | 비상정지 수동 발동 |

## 로그

| Command | 설명 |
|---------|------|
| `get_log_config` | 로그 설정 조회 (보관 기간, 최대 용량, api_debug) |
| `set_log_config` | 로그 설정 저장 |
| `write_frontend_log` | 프론트엔드 로그 → 백엔드 파일 기록 |
| `get_recent_logs` | 최근 로그 라인 조회 (`provider=`, `tr_id=`, `odno=`, `requestId=` 토큰은 Log UI trace chip으로 표시) |

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
| `exchange-rate-updated` | `f64` (USD/KRW 환율) | 데몬 4 (환율 갱신, 숫자 캐시 호환) |
| `exchange-rate-status-updated` | `ExchangeRateView` | 데몬 4 (환율 출처/유효시간 갱신) |
| `balance-updated` | `{ items, summary }` | 데몬 6 (잔고 갱신) |
| `overseas-balance-updated` | `{ items, summary }` | 데몬 6 (잔고 갱신) |
| `ws-status` | `WsStatusEvent` | `api/websocket.rs` |

> 프론트엔드 구독: `AppShell.tsx` → `useBackendEvents()` (`hooks.ts`)
