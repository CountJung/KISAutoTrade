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
| `get_web_config` | 웹 서버 포트, LAN 공개, API token 설정 여부 조회 (token 원문 미반환) |
| `save_web_config` | `WEB_PORT`, `WEB_ALLOW_LAN`, 32자 이상 `WEB_API_TOKEN` 저장. LAN 공개는 token 필수 |
| `detect_trading_type` | 실전/모의투자 자동 감지 |
| `detect_profile_trading_type` | 특정 프로파일 실전/모의 감지. 활성 프로파일은 실행 중 `TRADING_ACTIVE`로 거절 |
| `get_broker_rate_limit_status` | broker rate limiter 운영 상태 (scope/그룹별 pause 잔여, rate limit 누적, 마지막 성공/실패, 연속 실패; credential 마스킹) |
| `list_toss_accounts` | 입력한 토스증권 Client ID/Secret으로 `accountSeq` 후보 조회 |
| `list_toss_profile_accounts` | 저장된 토스증권 프로파일 키로 `accountSeq` 후보 조회 |
| `check_toss_profile_connection` | 토스증권 프로파일 연결 진단 (OpenAPI spec, token, accounts, holdings 단계 결과) |

## 자동매매 전용 예산

| Command | 입력 / 반환 |
|---|---|
| `get_auto_trading_budget` | `input: { brokerId, brokerAccountId }` → `AutoTradingBudgetView` |
| `update_auto_trading_budget` | 위 scope + `krwAmount`, `usdAmount` → 저장 후 view |

- REST: `GET /api/auto-trading-budget?brokerId=...&brokerAccountId=...`, `POST /api/auto-trading-budget` (동일 input body). 기존 웹 인증 경계 적용.
- KRW는 원, USD는 **센트** 정수이며 최대 `9007199254740991`. UI는 달러를 센트로 변환한다.
- 반환: `scope: { brokerId, accountId }`, `krw`/`usd` 각각 `allocatedAmount`, `cashAmount`, `reservedAmount`, `availableAmount`, `ownedPositionCount`, `blockedReason`, `feeBufferBps`.
- 활성 계좌와 요청 scope 불일치는 `BUDGET_SCOPE_CHANGED`, 계좌 미선택은 `BUDGET_SCOPE_REQUIRED`, 실행 중 변경은 `TRADING_RUNNING`, 원장/예산 거부는 `AUTO_TRADING_BUDGET_ERROR`.
- 원장 기본 배정액 0: 자동매수 차단. 리스크 사용 토글과 독립. Settings의 자동매매 전용 예산에서 조회/변경한다.

## 데이터베이스 관리 (Tauri 데스크톱 전용)

> DB 자격증명과 파괴적 작업은 인증되지 않은 axum/LAN REST에 노출하지 않는다.

| Command | 설명 |
|---------|------|
| `get_database_config` | PostgreSQL/MariaDB 연결 설정 조회 (password 미반환, 설정 여부만 반환) |
| `save_database_config` | 연결 설정 저장. 대상 변경 시 backend를 JSON으로 되돌림 |
| `test_database_connection` | 연결, 서버 버전, latency, schema/table 상태 조회 |
| `create_database_tables` | 문서 호환 계층과 scope별 주문·체결·포지션·risk projection 테이블을 schema v2로 migration |
| `clear_database_tables` | 앱 문서 데이터 전체 삭제 (자동매매 정지 + 확인 문구 필요) |
| `drop_database_tables` | KISAutoTrade 관리 테이블만 삭제 (자동매매 정지 + 확인 문구 필요) |
| `inspect_json_storage` | `data/` JSON 파일 수·크기·category 조회 |
| `import_json_to_database` | JSON 문서와 정규화 projection을 한 transaction으로 backfill/upsert |
| `export_database_to_json` | DB 문서를 timestamped export 디렉토리에 원래 상대경로로 반출하고 SHA-256 manifest 생성 |
| `set_storage_backend` | JSON/DB 저장 backend 전환. DB 전환은 현재 JSON key가 모두 import된 경우만 허용 |

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
| `submit_toss_small_buy_verification` | 호환용 Toss 1주 소액매매 검증 endpoint. 실거래 동의/최종 확인/최대 허용금액/accountSeq 일치/사전검증/미체결 scan 후 시장가 매수를 제출하고 주문·체결 기록을 저장. 현재 UI는 일반 수동주문 경로를 사용 |
| `get_toss_market_calendar` | 활성 Toss 프로파일로 KR/US 정규장 캘린더 조회 (`regularSession`, `isRegularOpen`) |
| `get_candle_page` | KIS/Toss 공통 완료봉 페이지 조회. 활성 프로파일·계좌 scope와 기준 시각을 고정한 typed cursor, 완료/무효 제외 수·출처·종료 사유 반환. 웹 GET `/api/candles/page`와 동일 camelCase DTO |
| `get_toss_chart_data` | 활성 Toss 프로파일로 캔들 조회 (`1d`/`1m`, count 1~200, `ChartCandle[]`) |
| `preview_leveraged_trend_hold` | 활성 Toss profile/account scope와 `1m`/`1d` count 20~200을 검증. 일봉은 성과 평가 불가 자료 진단, 분봉은 실제 장중 관측의 모의 표본 결과를 반환. 일봉 context와 분봉 지표를 분리하고 `replay.assessment`에 관측 수·평가 가능 여부·시간/세션 미검증을 기록 |
| `preview_strategy` | `candles` 평가 구간 + 선택 `historyCandles` 사전자료를 합계 최대 500봉으로 candle-close replay. `warmupCount` 생략은 0이며 명시 prefix와 별도 history 동시 지정은 거부. 결과 candles는 평가 구간만 포함. 실제 지표 준비 상태·조건 평가 봉 수(`preparation`), raw 신호·체결/차단·비용/리스크 성과와 OHLCV hash를 반환. generic LTH는 `UNSUPPORTED_GENERIC_REPLAY`로 명시 거부. live 10초 tick·intrabar/provider latency는 재현하지 않음 |
| `get_chart_data` | 국내주식 차트 데이터 (일/주/월봉, 날짜 범위와 선택적 count) |
| `get_overseas_price` | 해외주식 현재가 조회 |
| `get_overseas_chart_data` | 해외주식 최신 차트 데이터 (일/주/월봉, 선택적 count 상한) |
| `place_order` | 국내 수동 주문. 자동주문과 같은 scoped preflight/risk/pending/영속화 서비스 사용 |
| `place_overseas_order` | 해외 수동 지정가 주문. 자동주문과 같은 scoped order service 사용 |
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

`preview_strategy` v5의 `D`/`1d` cadence는 `dailyCloseWithDayBoundary`다. 시가-only 날짜 시작, 완성 OHLC 종가 평가, 체결/차단 포지션 피드백, 완료봉 적재 순서로 처리한다. 강한 종가는 전일 조건을 다음 평가일에 사용하고 변동성 확장은 이전 N일 평균과 당일 범위를 비교한다. 두 전략의 다른 interval은 `UNSUPPORTED_REPLAY_INTERVAL`로 거부한다. 모든 generic 체결은 해당 신호 봉 종가 근사이며 next-open/장중 경로는 미지원이다. 기존 입력·`preparation` 계약과 전용 LTH 경로는 유지한다.

`preview_strategy` v6은 `leveraged_trend_hold` 및 접미 ID를 `UNSUPPORTED_GENERIC_REPLAY`로 거부한다. 실시간 LTH factory/tick을 실행하거나 비결정적 backtest를 정상 결과로 돌려주지 않는다. 입력 시각 기반 전용 `preview_leveraged_trend_hold`의 Toss profile/account/interval 계약은 유지하며 generic 입력을 provider 조회로 자동 우회하지 않는다. 나머지 12개 전략의 입력·결과 형식은 같다.

현재 엔진은 `strategy-replay-v9`, 전용 LTH는 `leveraged-trend-hold-v4`이며 공통 엔진 버전을 응답과 입력 hash에 함께 넣는다. `ReplayMetadata`의 선택 `assessment`는 `{model, performanceStatus, dailyContextBars, intradayBars, timestampStatus, sessionStatus, limitations}`다. generic 응답에는 생략한다. 전용 LTH `1d`는 cadence/model=`dailyDiagnostic`, status=`notEvaluable`, `intradayBars=0`이며 장중 가격을 합성하거나 전략·체결을 실행하지 않는다. 호환 `backtest`의 빈 결과를 0% 투자 성과로 해석하지 않으며 UI는 성과·거래표·A/B 저장을 숨긴다. assessment 없는 기존 LTH 응답도 평가 불가로 처리한다.

전용 `1m`은 `minuteClose`, model=`intradaySample`, status=`sampleOnly`로 최대 200개 실제 분봉 표본을 반환한다. 유효 분봉이 없으면 `notEvaluable`이다. 이전 완료 일봉은 별도 context이며 분봉 EMA/RSI/ADX·반동·볼린저 warmup에 넣지 않는다. `warmupCount=0`이고 context 수는 `dailyContextBars`로 구분한다. 두 모델은 `timestampStatus=sessionStatus=unverified`, `lookAheadSafe=false`다. 입력 시각으로 결정적 계산은 가능하지만 현재 고정 KST 세션 근사, 원본 시간대·DST·기간 coverage가 미검증이므로 3개월 성과로 일반화하지 않는다. 기존 profile/account guard와 웹/Tauri 커맨드 이름은 유지한다.

v8 `BacktestReport.atrPreparation`은 `{period:14, historyBars, readyAtStart, readyAtEnd, firstReadyTime, unreadyEvaluationBars, rangePolicy:"observedBarsWithPreviousCloseNoInterpolation"}`다. 전략의 `preparation`과 별도이며 이전 close를 포함한 최근 TR14개를 사용한다. generic의 검증된 같은 간격 history만 지표를 준비하고 주문·손익에는 넣지 않는다. 분봉 LTH는 분봉 관측에서만 ATR을 준비한다. `volatilitySizingEnabled`의 기본 false는 유지하며, 활성화 시 미준비·0범위·0bps 위험예산 매수는 명시 차단한다. `atrPreparation`이 없는 보관/legacy 응답은 UI에서 준비 여부 미상으로 표시한다.

전용 LTH v4의 실행 콜백은 실제 잔여 수량·평균 체결가 snapshot을 반환한다. ATR 수량 확대/축소와 슬리피지를 다음 청산 판단에 반영하고 부분매도·차단 뒤 잔여 포지션의 보호 상태를 유지한다. 기존 bool callback API는 요청수량·신호가격의 raw-signal 연구용 호환 wrapper로 유지한다. 일봉의 실행 callback 미호출과 평가 불가 계약은 동일하다.

### 공통 캔들 페이지 계약

`get_candle_page({input})`와 GET `/api/candles/page`는 같은 `CandlePageInput`을 사용한다. 필수 값은 `symbol`, `brokerId`, `isOverseas`, `interval`, `expectedProfileId`; 선택 값은 `expectedBrokerAccountId`, `count`(1~200, 기본200), `exchange`, `cursor`다. KIS는 D/W/M(한 페이지 최대100), Toss는 D/1m(최대200)을 지원하며 KIS 미국 조회는 NAS/NYS/AMS 거래소를 지정한다. 기존 Vec 반환 차트 커맨드는 유지한다.

응답 `CandlePageView`는 `candles`, `nextCursor`, `endReason`, `brokerId`, `profileId`, `scopeKey`, `symbol`, `interval`, `exchange`, `source`, `timestampSemantics`, `completionPolicy`, `asOf`, `excludedUnfinishedBars`, `excludedInvalidBars`다. cursor는 클라이언트가 해석·수정하지 않고 그대로 전달한다. profile/broker/account/투자모드·종목·간격·거래소·count가 바뀌면 재사용하지 않는다. KIS는 최저 거래일 이전 날짜로 후퇴하고 Toss inclusive `nextBefore`는 변경 없이 URL 인코딩한다.

`asOf`는 첫 페이지에서 고정된다. 일봉은 거래소 현지의 당일, 주봉은 이번 주, 월봉은 이번 달을 보수적으로 제외한다(`strictlyBeforeCurrentMarketPeriod`). 이는 provider의 확정 플래그가 아닌 기간 경계 정책이다. Toss 분봉은 공식 종료 시각이 `asOf` 이하인 관측만 반환한다(`barCloseAtOrBeforeAsOf`). 미래·무효 OHLCV는 제외하며 누락봉을 합성하지 않는다. 실패는 `{code,message}`이며 scope/커서/입력 오류를 일반 provider 실패(`CANDLE_PAGE_FAILED`)와 구분한다.

일반 미리보기는 최대8회 요청·보관500봉으로 요청 평가봉50/100/200개를 우선 보존하고 이전252봉을 별도 수집한다. 부족·provider 종료·무진전·커서 반복·요청 상한·부분 오류와 제외 수를 UI에 표시한다. 범위 변경·취소 결과는 폐기하며 첫 페이지 provider 오류 때만 KIS 미국 거래소 후보를 변경한다. live52주 완료 일봉 갱신은 아래 상태 계약과 연결했다. 전용 LTH의 실제 장중 자료 검증은 별도 미완료 과제다.

v9 generic 1m 입력은 RFC3339 offset을 실제 순간으로 해석하고 고정 정밀도 UTC로 정규화한다. 같은 순간의 다른 표기는 중복이며 history/evaluation 경계에도 동일 규칙을 적용한다. legacy compact12/14는 기존 차트의 KST 계약을 유지한다. simulation guard 시간 parser도 RFC3339를 우선 해석하고, replay의 일일 리스크·손절 당일 재진입 기준 날짜는 한국 Seoul/미국 New York으로 고정한다. cooldown 경과는 실제 순간으로 비교한다. 실운용 guard의 Local 시각 정책과 일봉 OHLC/전략 계산은 바꾸지 않는다.

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
| `get_trading_status` | 자동 매매 상태와 건강 정보 조회 (holdings sync/reconciliation/pending/persistence/daemon 실패, 실행 scope 포함) |
| `start_trading` | 활성 broker/account 잔고·holdings 및 복원 pending 대조 성공 후 실행 epoch·설정 revision·profile/client snapshot 고정. 시작 context 불일치는 `EXECUTION_CONTEXT_CHANGED`로 차단 |
| `stop_trading` | 실행 epoch 무효화 후 자동 매매 정지. 이미 예약한 주문의 원래 scope·pending·예산 추적 유지 |
| `clear_buy_suspension` | 잔고 부족 매수 정지 수동 해제 |
| `get_positions` | 포지션 목록 조회 |
| `get_pending_orders` | 미체결 주문 조회 (`status`, `filledQuantity`, `remainingQuantity`, provider trace 포함) |
| `get_strategies` | 전략 목록 조회 (`brokerId`, `brokerAccountId`, 대상 종목, params 포함) |
| `update_strategy` | 전략 파라미터 업데이트. 공통 설정 transaction에서 저장 성공 후 기존 보유 상태를 유지하며 새 revision 공개. 저장 실패는 기존 설정·revision 유지. expected profile/broker/account 불일치는 `SCOPE_MISMATCH`, 신호·영향 종목 주문 처리 중은 `CONFIGURATION_BUSY`, 보유 대상 제거/시장 변경은 `HELD_TARGET_REMOVAL`/`HELD_MARKET_CHANGE`로 거부 |
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

### Generic preview 준비 메타데이터(v4)

`historyCandles`와 평가봉은 동일 interval의 날짜 형식(일/주/월 8자리, 분봉 14자리 또는 초 00으로 보정되는 12자리)을 사용하며 사전자료는 평가 첫 봉보다 엄격히 이전이어야 한다. 잘못된 OHLC/시각·중복·겹침·모호한 prefix는 거부한다. `preparation`의 요구/제공/실제 버퍼 수, 시작·종료 준비, 처리 후 첫 준비 시각/미준비 봉 수와 호출 전 조건 평가 봉 수(`evaluatedBars`)를 구분한다. 지원 여부를 알 수 없는 값은 null이며 seed-only 구간은 `notEvaluable`이다.


## Live 52주 완료 일봉 상태

`get_trading_status` 및 시작/정지/매수정지 해제 응답의 `dailyHistory`는 `strategyId`, `symbol`, `brokerId`, `profileId`, `marketDate`, `requiredBars`, `availableBars`, `state`, `source`, `lastCompletedDate`, `lastError`, `nextRetryAt` 배열이다. state는 pending/ready/insufficient/failed/capacityExceeded다. 웹 GET `/api/trading/status`도 같은 camelCase 의미를 사용하며 실행 profile/broker/account는 실제 manager run identity를 반환한다. 중지 후 남은 OrderManager scope를 실행중으로 표시하지 않는다. 구버전의 필드 누락은 UI에서 미지원 중립 상태다.

최대511개 수집 대상과 초과 진단1행, 동시2개 job, 종목/계좌/날짜 공유 수집,8회 요청/512봉 보관,30~900초 재시도를 적용한다. 당일 제외·최신 완료일1~10일 이내와 실제 요구량 충족을 확인한다. quote 직전 새 시장 날짜를 다시 확인하며 부족 중 신규 매수만 막고 보유 손절은 유지한다. run/config/scope/profile 변경 결과를 버린다. 기존 preview500봉 상한과 다른 live512봉 경계다. [상세 검증·한계](tasks/live-completed-daily-history-20261018.md).
