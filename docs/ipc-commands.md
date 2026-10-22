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
| `get_balance` | 국내 잔고 조회 (BalanceSummary + items). 고정 계좌/실행·설정 lease·시장별 요청/포지션 버전을 검증한 응답만 tracker에 게시 |
| `get_overseas_balance` | 해외 잔고 조회 (OverseasBalanceItem[] + summary). 같은 검증 후 해외 tracker만 게시, USD cents·거래소 정규화 유지 |
| `get_broker_holdings` | 활성 broker 보유 종목 조회 (`BrokerHoldingView[]`, Toss/KIS 공통 decimal 문자열 보존) |
| `get_price` | 종목 현재가 조회 |
| `get_toss_market_snapshot` | 활성 Toss 프로파일로 현재가/호가/최근 체결/상하한가 read-only snapshot 조회 |
| `get_toss_stock_safety` | 활성 Toss 프로파일로 종목 기본 정보와 매수 유의사항 조회 (`buyBlocked`, `buyBlockReason`) |
| `check_toss_order_preflight` | 활성 Toss 프로파일로 주문 전 검증 (`buyingPower`, `sellableQuantity`, `commissionRate`, `canSubmit`). commissionRate는 소수 비율 원문이며 필요 현금에는 비율을 그대로 적용한 수수료 포함 |
| `list_toss_open_orders` | 활성 Toss 프로파일의 접수/미체결 주문 목록 조회 (`status=OPEN`, 수동 주문창 표시용) |
| `modify_toss_order` | 활성 Toss 프로파일의 접수 주문 정정 요청 (`orderId`, `orderType`, `quantity`, `price`) |
| `submit_toss_small_buy_verification` | 호환용 Toss 1주 소액매매 검증 endpoint. 동의/최종 확인/최대 금액/accountSeq/사전검증/미체결 scan 후 제출. 즉시 체결은 원 주문 ID·종목·BUY·통화·정수1주·명시 체결대금 검증; 금액 누락은 pending 유지. 현재 UI는 일반 수동주문 경로를 사용 |
| `get_toss_market_calendar` | 활성 Toss 프로파일로 KR/US 정규장 캘린더 조회 (`regularSession`, `isRegularOpen`) |
| `get_candle_page` | KIS/Toss 공통 완료봉 페이지 조회. 활성 프로파일·계좌 scope와 기준 시각을 고정한 typed cursor, 완료/무효 제외 수·출처·종료 사유 반환. 웹 GET `/api/candles/page`와 동일 camelCase DTO |
| `get_toss_chart_data` | 활성 Toss 프로파일로 캔들 조회 (`1d`/`1m`, count 1~200, `ChartCandle[]`) |
| `preview_leveraged_trend_hold` | 활성 Toss profile/account scope와 `1m`/`1d` count 20~200을 검증. 일봉은 성과 평가 불가 자료 진단, 분봉은 실제 장중 관측의 모의 표본 결과를 반환. 일봉 context와 분봉 지표를 분리하고 `replay.assessment`에 관측 수·평가 가능 여부·시간/세션 미검증을 기록 |
| `preview_strategy` | `candles` 평가 구간 + 선택 `historyCandles` 사전자료를 합계 최대 500봉으로 종가 신호·선택 체결 모델 replay. `warmupCount` 생략은 0이며 명시 prefix와 별도 history 동시 지정은 거부. 결과 candles는 평가 구간만 포함. 실제 지표 준비 상태·조건 평가 봉 수(`preparation`), raw 신호·체결/차단·비용/리스크 성과와 OHLCV hash를 반환. generic LTH는 `UNSUPPORTED_GENERIC_REPLAY`로 명시 거부. live 10초 tick·intrabar/provider latency는 재현하지 않음 |
| `preview_strategy_continuous` | 한 요청의 엄격한 이전 사전자료와 시간순 분할 평가봉을 공통 v12 세션에서 연속 재생. history 최대512봉·1~16묶음·묶음당1~500봉·전체 평가4096봉. 상태 보존과 공급 경계 원장을 반환하며 generic LTH는 거부. 웹 POST `/api/strategy/preview/continuous`와 동일 입력 |
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

`preview_strategy` v5의 `D`/`1d` cadence는 `dailyCloseWithDayBoundary`다. 시가-only 날짜 시작, 완성 OHLC 종가 평가, 체결/차단 포지션 피드백, 완료봉 적재 순서로 처리한다. 강한 종가는 전일 조건을 다음 평가일에 사용하고 변동성 확장은 이전 N일 평균과 당일 범위를 비교한다. 두 전략의 다른 interval은 `UNSUPPORTED_REPLAY_INTERVAL`로 거부한다. v5의 해당 신호 봉 종가 근사는 v10에서 호환 기본값 `sameClose`로 유지한다. 아래 v10의 `nextOpen`·장중 연구 가정 및 v11 회계 계약을 함께 확인한다. 기존 입력·`preparation` 계약과 전용 LTH 경로는 유지한다.

`preview_strategy` v6은 `leveraged_trend_hold` 및 접미 ID를 `UNSUPPORTED_GENERIC_REPLAY`로 거부한다. 실시간 LTH factory/tick을 실행하거나 비결정적 backtest를 정상 결과로 돌려주지 않는다. 입력 시각 기반 전용 `preview_leveraged_trend_hold`의 Toss profile/account/interval 계약은 유지하며 generic 입력을 provider 조회로 자동 우회하지 않는다. 나머지 12개 전략의 입력·결과 형식은 같다.

현재 엔진은 `strategy-replay-v18`, 전용 LTH는 `leveraged-trend-hold-v4`이며 공통 엔진 버전을 응답과 입력 hash에 함께 넣는다. `ReplayMetadata`의 선택 `assessment`는 `{model, performanceStatus, dailyContextBars, intradayBars, timestampStatus, sessionStatus, limitations, minuteTiming?}`다. generic 응답에는 생략한다. 전용 LTH `1d`는 cadence/model=`dailyDiagnostic`, status=`notEvaluable`, `intradayBars=0`이며 장중 가격을 합성하거나 전략·체결을 실행하지 않는다. 호환 `backtest`의 빈 결과를 0% 투자 성과로 해석하지 않으며 UI는 성과·거래표·A/B 저장을 숨긴다. assessment 없는 기존 LTH 응답도 평가 불가로 처리한다.

전용 `1m`은 `minuteClose`, model=`intradaySample`, status=`sampleOnly`로 최대 200개 실제 분봉 표본을 반환한다. 유효 분봉이 없으면 `notEvaluable`이다. 이전 완료 일봉은 별도 context이며 분봉 EMA/RSI/ADX·반동·볼린저 warmup에 넣지 않는다. `warmupCount=0`이고 context 수는 `dailyContextBars`로 구분한다. 두 모델은 `timestampStatus=sessionStatus=unverified`, `lookAheadSafe=false`다. 입력 시각으로 결정적 계산은 가능하지만 현재 고정 KST 세션 근사, 원본 시간대·DST·기간 coverage가 미검증이므로 3개월 성과로 일반화하지 않는다. 기존 profile/account guard와 웹/Tauri 커맨드 이름은 유지한다.

v8 `BacktestReport.atrPreparation`은 `{period:14, historyBars, readyAtStart, readyAtEnd, firstReadyTime, unreadyEvaluationBars, rangePolicy:"observedBarsWithPreviousCloseNoInterpolation"}`다. 전략의 `preparation`과 별도이며 이전 close를 포함한 최근 TR14개를 사용한다. generic의 검증된 같은 간격 history만 지표를 준비하고 주문·손익에는 넣지 않는다. 분봉 LTH는 분봉 관측에서만 ATR을 준비한다. `volatilitySizingEnabled`의 기본 false는 유지하며, 활성화 시 미준비·0범위·0bps 위험예산 매수는 명시 차단한다. `atrPreparation`이 없는 보관/legacy 응답은 UI에서 준비 여부 미상으로 표시한다.

전용 LTH v4의 실행 콜백은 실제 잔여 수량·평균 체결가 snapshot을 반환한다. ATR 수량 확대/축소와 슬리피지를 다음 청산 판단에 반영하고 부분매도·차단 뒤 잔여 포지션의 보호 상태를 유지한다. 기존 bool callback API는 요청수량·신호가격의 raw-signal 연구용 호환 wrapper로 유지한다. 일봉의 실행 callback 미호출과 평가 불가 계약은 동일하다.

v10 공통 `assumptions`에 `executionModel: sameClose | nextOpen`(기본sameClose), `intrabarExitPolicy: unverified | stopFirst`(기본unverified), `stopLossPct`(기본0,0..99.9), `takeProfitPct`(기본0,0..1000)를 추가했다. 0은 보호 비활성이며 live 설정을 바꾸지 않는다. 다음 관측봉 시가 모델은 이전 종가 신호를 보류하고, 다음 봉 시작에 이전 완료봉 ATR·해당 시가로 guard/risk/현금을 검사한 뒤 실제 수량·원가를 전략에 반영한다. 마지막 대기 신호는 `unfilled`로 남기며 체결·차단으로 세지 않는다.

`BacktestTrade`의 선택 `signalAvailableAt`/`fillAt`는 `{barTime,phase:open|close|intrabar}`이며 거래소의 정확한 timestamp가 아닌 봉 label과 정보 공개 단계다. `time`은 처리/체결 봉 label(마지막 미체결은 신호 봉), `intrabarPathStatus`는 `gapAtOpen|ambiguousStopFirst|ohlcUnverified`다. summary에 `unfilledOrderCount`·`ambiguousBarCount`, report에 `executionWarnings`를 추가한다. 신규 `nextOpen` generic은 기술적 미래봉 불변 검증으로 `lookAheadSafe=true`, 같은 종가 체결 가정은 false다. 어느 쪽도 실제 체결 가능성 인증이 아니다. LTH의 false/assessment 제한은 유지한다.

연구 보호는 실제 체결 평균가 기준의 별도 시나리오다. 갭이 보호 가격을 넘으면 관측 시가에서 처리한다. 장중 고저가 접촉은 unverified에서 체결하지 않으며 stopFirst에서만 가정 체결한다. 양방향 접촉은 순서 불명 집계와 손절 우선 표시를 남긴다. 같은 종가에 새로 진입한 포지션에 이미 지난 그 봉 고저가를 적용하지 않는다. 강제 보호 주문은 수익/전략 재진입 guard로 손절을 억제하지 않으며 비용·손익은 공통 회계에 반영한다.

`ReplaySession`의 `begin_bar → position_snapshot → 전략 종가 평가 → process_close → position_snapshot`을 generic과 전용 LTH 분봉에 공통 적용한다. LTH의 새 `preview_signals_with_driver`는 매 관측 `Open/Signal/Close`를 전달하여 신호 없는 봉의 대기 체결도 먼저 반영한다. 기존 callback API는 호환 wrapper다. Tauri 등록·웹 route·query key는 그대로이며 A/B 보관 응답의 새 필드가 없으면 UI는 이전 결과의 체결 정보 부족을 표시한다.

### Replay v11 회계·비교 계약

`preview_strategy`와 전용 LTH `1m`은 같은 v11 회계를 사용한다. 입력·응답 타입 정본은 [`strategyResearchTypes.ts`](../src/shared/api/strategyResearchTypes.ts), 회계는 `trading/simulation/{accounting.rs,session/accounting.rs}`, 비교는 `simulation/comparison.rs`다. 기존 커맨드·웹 route·query key를 유지하며 이 필드는 실운영 주문·브로커 비용 조회를 수행하지 않는다.

| 추가 `assumptions` | 기본값·검증 |
|---|---|
| `costSource` | `estimated` 또는 `brokerVerifiedManual`; 기본 `estimated`. 후자는 사용자의 수동 확인 표시이며 앱이 브로커 API로 검증했다는 뜻이 아님 |
| `costNote` | 기본 빈 문자열. 제어문자 없는 최대 120문자이며 수동 확인에는 공백 이외 메모 필수 |
| `buyFeeBps`, `sellFeeBps` | 생략/null이면 기존 `feeBps`를 방향별로 사용. 지정 값은 유한수 0..1000bps |
| `minimumFeeNative` | 기본 0, 유한수 0..100000000. 국내 원·해외 USD의 체결당 최소 수수료; 비율 수수료와 비교해 큰 값을 적용하고 해당 시점 환율로 원화 환산 |
| `fxScenario` | 기본 `[]`. 각각 `{barTime,rateKrw}`이며 `rateKrw`는 유한수 100..10000. 국내 종목의 비어 있지 않은 시나리오는 거부 |
| `dividendScenario` | 기본 `[]`. 각각 `{barTime,amountPerShare,withholdingTaxBps}`. 주당 금액은 국내 원·해외 USD의 유한수 0..100000000, 원천징수는 유한수 0..10000bps |
| `dividendPriceBasis` | 기본 `unverified`; `unadjusted`/`dividendAdjusted`는 사용자의 가격 자료 선언. `dividendAdjusted`와 비어 있지 않은 현금 배당을 함께 입력하면 중복 계상으로 거부. 미확인은 경고하며 자동 검증으로 표시하지 않음 |

두 시나리오는 **각각 최대 64개의 고유 평가 봉**만 허용한다. `barTime`은 서버에서 최대 64바이트이며 일봉 ISO 날짜/compact 날짜, 분봉 RFC3339/legacy compact 시각을 기존 캔들 규칙으로 정규화한다. 실제 평가 봉 하나와 정확히 일치해야 하며 alias 정규화 후 중복도 거부한다. 사전자료·미래·누락 봉으로 옮기거나 가까운 봉에 붙이지 않는다. 검증 오류는 `INVALID_ACCOUNTING_SCENARIO`다. LTH `1d`는 자료 진단이므로 비어 있지 않은 FX/배당 시나리오를 `SCENARIO_NOT_EVALUABLE`로 거부하고 성과·새 회계·비교를 표시하지 않는다. UI의 JSON 입력은 각각 24,000자, 봉 라벨 40자로 더 좁게 제한한다. 잘못된 draft는 기존 결과와 진행 응답을 무효화하고 계산을 막으며 `[]`로 대체하지 않는다.

환율은 지정 평가 봉의 시작에 알려진 시나리오 값으로 적용하고 다음 지정점까지 유지한다. 첫 지정점 전에는 `exchangeRateKrw`를 사용하며 미래 환율을 소급하지 않는다. 배당은 해당 봉 시작 **직전** 보유 수량에만 귀속되어 같은 봉 대기 매수는 제외한다. 원천징수 후 해당 봉에서 원화 현금으로 지급하는 연구 가정이며 실제 배당락·지급일·기업행동·통화를 검증하지 않는다. 유휴 현금은 KRW로 유지하므로 USD 현금 계좌의 환율 재평가가 아니다. 거래·배당 시점의 원화 환산과 별개로 native 평균 체결가는 보존하여 FX 변화가 전략의 원가·손절 기준을 바꾸지 않게 한다.

최소 수수료가 양수인 일반 이익 매도는 실제 청산 수량·잔여 매수 비용의 비례 배분·현재 환율·수수료·세금·슬리피지를 반영한 세후 기대 순익이 양수여야 한다. native 가격 손실 매도·명시 손절·연구 보호 청산은 이 추가 순익 제한에서 제외하며 기존 시간·리스크 검사는 유지한다. 같은 전망 계산을 실제 청산 장부에도 사용한다. 차단·미체결에는 체결 수수료를 부과하지 않는다.

`BacktestReport.accounting`은 다음 필드를 반환한다. 가격/환율 기여는 수수료 배분 전의 분해이며, 순손익은 해당 비용을 포함한다.

| 회계 필드 | 의미 |
|---|---|
| `realizedPnlKrw`, `residualUnrealizedPnlKrw` | 완료 청산 순손익과 아직 매도하지 않은 잔여 평가 순손익. 표시 반올림 전 `실현 + 잔여 평가 + dividendNetKrw = finalEquityKrw − initialCapitalKrw` |
| `remainingQuantity`, `remainingCostBasisKrw`, `remainingBuyFeeKrw` | 잔여 수량, 과거 환율로 기록한 매수 원가, 아직 잔여분에 배분된 매수 수수료 |
| `realizedPricePnlKrw`, `realizedFxPnlKrw`, `residualPricePnlKrw`, `residualFxPnlKrw` | 가격 효과 `(종료 native 가격 − 진입 native 가격) × 진입 환율 × 수량`; FX 효과 `종료 native 가격 × (종료 환율 − 진입 환율) × 수량`. 슬리피지는 체결 가격에 포함 |
| `totalFeeKrw`, `totalTaxKrw`, `totalSlippageKrw` | 실제 모의 체결의 수수료·거래세·가격에 포함된 슬리피지 영향. 슬리피지를 현금 비용으로 다시 차감하지 않음 |
| `dividendGrossKrw`, `dividendTaxKrw`, `dividendNetKrw` | 시나리오 배당 총액·원천징수·순현금. 시나리오가 없으면 가격 기반 결과이며 배당 포함 총수익이 아님 |
| `terminalLiquidationEquityKrw`, `terminalLiquidationReturnPct`, `terminalExitFeeKrw`, `terminalExitTaxKrw`, `terminalExitSlippageKrw` | 마지막 종가에 잔여분을 청산한다고 가정한 보고 전용 값. 실제 cash/보유·체결 수·승률·리스크 상태를 바꾸지 않음 |
| `averageInvestedNotionalKrw`, `averageInvestedCapitalPct`, `averageInvestedEquityPct`, `peakInvestedCapitalPct`, `peakInvestedEquityPct` | 각 평가 봉말 보유 **평가액**의 평균과 초기자본/당시 순자산 대비 관측 평균·최대 비중. 진입 원금·시간 가중 노출·장중 최대 비중이 아니며 `summary.exposurePct`의 보유 관측 봉 비율과도 구분 |
| `realizedEquitySignMismatch`, `warnings` | 실현손익과 전체 자산 증감 부호 차이 및 비용 출처·현금 장부·시나리오·자료 한계 경고. 청산 표본 20건 미만을 경고하며 부분 청산도 각각 1건으로 집계 |

`BacktestTrade`에는 `executionReferencePrice`(슬리피지 적용 전 native 기준가), `exchangeRateKrw`, `feeKrw`, `taxKrw`, `slippageKrw`를 추가한다. 기존 `costKrw`는 호환용 진단 합계인 수수료+거래세+슬리피지 영향이다. 슬리피지 영향은 이미 체결 가격에 포함되므로 이 진단 합계를 현금에서 다시 차감하지 않는다.

`BacktestReport.comparison`은 `sameInputQuantityBuyHold`, `sameExecutionPathCostReference`, `capitalExposureDifferencePct`, `holdingBarDifferencePct`, `limitations`를 반환한다.

- `sameInputQuantityBuyHold`: 같은 평가 구간·자본·요청 수량·체결/비용/FX/배당·ATR/리스크 조건에서 첫 평가 종가에 매수를 지시한 단순 보유다. 연구 손절·익절과 전략 청산은 끈다. `requestedQuantity`, `actualEntryQuantity`, `remainingQuantity`, `finalEquityKrw`, `cumulativeReturnPct`, `terminalLiquidationReturnPct`, `realizedPnlKrw`, `residualUnrealizedPnlKrw`, `dividendNetKrw`, `mddPct`, `averageInvestedCapitalPct`, `averageInvestedEquityPct`, `peakInvestedCapitalPct`, `peakInvestedEquityPct`, `holdingBarPct`, `filledOrderCount`, `blockedOrderCount`, `unfilledOrderCount`로 실제 수량·노출 차이를 공개한다. 입력 수량이 같아도 실제 체결 수량·기간·비중이 같다고 주장하지 않는다.
- `sameExecutionPathCostReference`: 전략의 실제 모의 체결 수량·시점·노출을 고정해 발생 당시 기록한 원화 수수료·거래세·슬리피지 영향·배당 원천징수만 환급한 사후 산술 참고값이다. `restoredCostKrw`, `finalEquityKrw`, `cumulativeReturnPct`를 반환하며 원가/FX 귀속 재산정 또는 비용을 바꾼 전략 재실행 성과가 아니다.
- 두 difference는 전략에서 단순 보유의 평균 초기자본 투자 비중/보유 관측 봉 비율을 뺀 %p다. 관측 봉이 없는 구간은 비교를 만들지 않는다. 같은/겹친 기간의 A/B·모델 비교는 독립 검증이 아니며 부분기간을 전체 분기·연속 운용 성과로 합산하지 않는다.

TS의 새 응답 필드는 보관된 legacy 결과를 위해 선택 필드다. `accounting`·`comparison`이 없으면 확인 불가로 표시하며 0이나 보관 거래 100건에서 역산한 손익으로 대체하지 않는다. A/B는 서버 집계와 비용·FX·배당·가격 자료 선언을 그대로 보존하고 비교 조건 차이를 표시한다. 상세 검증 명령과 mock UI 47/47 근거는 [Harness 지도](HARNESS_MAP.md)의 P2-15 항목을 따른다.

### Replay v12 연속 재생 계약

`preview_strategy_continuous({input})`와 웹 POST `/api/strategy/preview/continuous`(body는 `input` 객체)는 같은 `ContinuousStrategyPreviewInput`을 받는다. 오프라인 생산 라이브러리는 `preview_strategy_continuous_from_candles`다. 기존 `preview_strategy`의 평가+사전자료 합계500봉·선택 `warmupCount` 계약은 그대로 유지한다. 연속 경로는 서버 session ID·이어받기 캐시 없이 **한 번의 bounded 요청 안에서만** 상태를 이어가는 stateless 계산이다.

| 연속 입력 | 검증·의미 |
|---|---|
| 공통 필드 | 기존 `strategyId`, `strategyName`, `symbol`, `isOverseas`, `orderQuantity`, `params`, 선택 `interval`, `dataSource`, `strategyVersion`, `brokerId`, `brokerAccountId`, 기본값이 있는 `assumptions` 사용 |
| `historyCandles` | 기본 `[]`, 최대512봉. 정규화된 모든 시각이 첫 평가 봉보다 엄격히 이전이어야 하며 최초 지표 준비에만 한 번 사용 |
| `chunks` | 필수1~16개. 각각 `{candles: ChartCandle[]}`이며 묶음당1~500봉, 전체 평가 최대4096봉 |
| 입력 형식 | input과 chunk는 알 수 없는 필드를 거부한다. flat `candles` 또는 `warmupCount`를 섞어 prefix/별도 history 의미를 중복 지정할 수 없음 |
| 간격·전략 | 기본 `1d`; API는 기존 `D`/`1d`/`1m`/`M1`/`W`/`M`을 지원. 강한 종가·변동성 확장은 `D`/`1d`만 허용. generic LTH 및 접미 ID는 `UNSUPPORTED_GENERIC_REPLAY`로 거부하며 전용 LTH로 자동 우회하지 않음 |

각 history·chunk 내부와 chunk 사이의 시각은 기존 일봉/분봉 label 규칙으로 정규화 후 엄격한 시간순이어야 한다. 중복·역전을 전체 정렬로 수리하지 않는다. history/평가·chunk 경계 겹침은 `CONTINUOUS_TIME_OVERLAP`, 묶음/개별/history 상한은 `INVALID_CONTINUOUS_BOUNDS`, 전체 평가 초과는 `TOO_MANY_CONTINUOUS_CANDLES`다. 기존 캔들 유효성·간격 오류도 그대로 거부한다. FX/배당 시나리오는 전체 평가 봉을 기준으로 먼저 검증하며 history·다른 묶음의 가까운 봉으로 이동하지 않는다. v11의 비용·배당 조정 중복 거부·현금 장부·회계 및 비교 계약은 그대로 적용한다.

`strategy_preview/driver.rs`는 입력 전체 검증 뒤 전략 factory·warmup·`ReplaySession`을 각각 한 번 만들고, 모든 묶음을 순서대로 append한 후 마지막에만 finish한다. 지표·현금·보유 수량/native 평균 원가·대기 신호·쿨다운·연속손실·리스크 상태를 공급 묶음이나 분기마다 초기화하지 않는다. 일일 상태 변경은 실제 event 거래일 경계를 따른다. `nextOpen`의 묶음 끝 신호는 다음 묶음 첫 관측 시가까지 대기하며, 전체 마지막에도 남은 신호만 `unfilled`로 확정한다. 종료 시 가상 청산은 여전히 보고 전용이며 연속 장부를 바꾸지 않는다.

응답 `ContinuousStrategyPreviewView`는 기존 `StrategyPreviewView`를 최상위에 그대로 펼치고 `continuity`를 추가한다. `continuity`는 `{mode:"continuous",chunkCount,evaluationBars,historyBars,chunks}`이고 각 원장은 `{index,start,end,bars,startQuantity,endQuantity,pendingAtEnd}`다. index는0부터 시작한다. 수량과 대기 여부는 공급 경계의 상태 진단이며 독립 실험·청산/재진입 지시가 아니다. UI는 원장·평가/history 수가 보낸 입력과 일치하는지 확인한 뒤 결과를 게시한다.

v12 `inputHash`는 `boundedWindow`/`continuous` 계산 모드를 포함하므로 같은 봉이라도 두 모드는 구분한다. 동일 연속 경제 입력의 chunk 분할 위치·개수는 hash에 넣지 않아 분할만 바꾼 요청은 같은 hash다. UI의 A/B snapshot은 실제 호출 모드인 `calculationMode`와 최상위 `continuity`를 서버 집계와 함께 보존한다. 이전 슬롯에 모드가 없으면 **계산 범위 정보 없음**으로 표시하며 현 엔진/단기 모드를 추정해 채우지 않는다. 계산 모드가 다른 A/B도 가정 차이를 표시해 비교할 수 있으며 같은 기간 비교는 독립 검증이 아니다.

연속 보고의 기존 `inSample`/`outOfSample` wire key는 호환을 위해 유지한다. UI는 **앞 구간·뒤 구간의 동일 장부 성과**와 **연속 구간 경계(기본70/30)**로 표시한다. 이 경계에서도 포지션·원가·리스크가 이어지므로 독립 훈련/검증, 재학습 또는 분기별 초기화 성과로 해석하지 않는다. 부분 구간 수익률을 더해 전체 연속 수익률을 만들지 않는다.

UI는 일반 전략에서 **연속 일봉·날짜 범위**만 제공하고 기본 요청은 어제까지 최근3년이다. `candlePageCollector.ts`가 같은 profile/account·자료 출처·기준 시각을 고정해 최대48회 페이지 요청, 보관4608봉·평가4096봉 안에서 수집한다. 요청 시작보다 엄격히 이전인252관측을 한 번 확보해야 하며 평가 봉을 history로 옮겨 부족분을 채우지 않는다. 준비된 평가를 오래된 순서로500봉씩 나누어 연속 API를 한 번 호출한다. API의 history512 상한과 UI의252봉 요구는 서로 다른 제한이다.

범위·이전252봉 부족, provider 부분 오류/조기 종료·scope 변경, 반복 cursor·진행 없음·수집 상한 초과는 수집 진단과 계산 보류로 표시한다. 요청 시작→실제 시작 또는 실제 종료→요청 종료 차이, 평가 관측 사이 최대 간격이 **각7달력일을 초과**해도 계산을 보류한다. 7일은 앱의 보수적 자료 경계이며 검증된 거래소 휴장 달력이 아니므로 긴 휴장도 보류될 수 있다. 준비되어도 `coverageVerified=false`를 유지하고 요청/실제 범위·asOf·제외/중복 수·최대 간격을 공개한다. 없는 봉을 합성하거나 부분기간을3년 성과로 표시하지 않는다. 모드·날짜·가정 변경/취소는 기존 결과와 진행 generation을 무효화한다. 기존 단기50/100/200봉의8회·보관500봉 경로와 전용 LTH 계약은 유지한다.

구현 경계는 [아키텍처](ARCHITECTURE.md), 사용법은 [사용자 가이드](user-guide.md), 합성/mock 검증 근거는 [Harness 지도](HARNESS_MAP.md)의 P2-18 항목을 따른다. 연속 계산 성공은 실제 거래소 coverage·체결 가능성·다년 자료 확보의 인증이 아니다.

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
| `start_trading` | 활성 broker/account 잔고 게시·복원 pending 대조·exact owner 전략 복원·당일 모드별 리스크 복원 후 실행 context 고정. 전체 계좌 수량을 각 전략에 배정하지 않음. context 불일치는 `EXECUTION_CONTEXT_CHANGED`, 리스크 복원 실패는 신규 주문 차단 |
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
| `balance-updated` | `{ items, summary }` | 데몬 6의 검증된 국내 게시. UI는 payload 대입 없이 현재 계좌 query 재조회 |
| `overseas-balance-updated` | `{ items, summary }` | 데몬 6의 검증된 해외 게시. UI는 payload 대입 없이 현재 계좌 query 재조회 |
| `ws-status` | `WsStatusEvent` | `api/websocket.rs` |

> 프론트엔드 구독: `AppShell.tsx` → `useBackendEvents()` (`hooks.ts`)

잔고 payload 형식과 기존 command 입력은 유지한다. 늦은 응답·더 최신 조회·조회 중
체결/삭제·설정 lease·프로파일/모드/실행 변경은 기존 오류 경로로 재조회 사유를
반환하며 tracker를 덮지 않는다. Web 일반 balance와 IPC/Web broker holdings도
응답 전 계좌 identity를 확인하지만 기존 ReadOnly 성격을 유지한다. holdings
쿼리 키는 기존 prefix 뒤 profile/broker/account/mode를 붙여 prefix 무효화와 호환한다.

### Generic preview 준비 메타데이터(v4)

`historyCandles`와 평가봉은 동일 interval의 날짜 형식(일/주/월 8자리, 분봉 14자리 또는 초 00으로 보정되는 12자리)을 사용하며 사전자료는 평가 첫 봉보다 엄격히 이전이어야 한다. 잘못된 OHLC/시각·중복·겹침·모호한 prefix는 거부한다. `preparation`의 요구/제공/실제 버퍼 수, 시작·종료 준비, 처리 후 첫 준비 시각/미준비 봉 수와 호출 전 조건 평가 봉 수(`evaluatedBars`)를 구분한다. 지원 여부를 알 수 없는 값은 null이며 seed-only 구간은 `notEvaluable`이다.


## Live 52주 완료 일봉 상태

`get_trading_status` 및 시작/정지/매수정지 해제 응답의 `dailyHistory`는 `strategyId`, `symbol`, `brokerId`, `profileId`, `marketDate`, `requiredBars`, `availableBars`, `state`, `source`, `lastCompletedDate`, `lastError`, `nextRetryAt` 배열이다. state는 pending/ready/insufficient/failed/capacityExceeded다. 웹 GET `/api/trading/status`도 같은 camelCase 의미를 사용하며 실행 profile/broker/account는 실제 manager run identity를 반환한다. 중지 후 남은 OrderManager scope를 실행중으로 표시하지 않는다. 구버전의 필드 누락은 UI에서 미지원 중립 상태다.

최대511개 수집 대상과 초과 진단1행, 동시2개 job, 종목/계좌/날짜 공유 수집,8회 요청/512봉 보관,30~900초 재시도를 적용한다. 당일 제외·최신 완료일1~10일 이내와 실제 요구량 충족을 확인한다. quote 직전 새 시장 날짜를 다시 확인하며 부족 중 신규 매수만 막고 보유 손절은 유지한다. run/config/scope/profile 변경 결과를 버린다. 기존 preview500봉 상한과 다른 live512봉 경계다. [상세 검증·한계](tasks/live-completed-daily-history-20261018.md).


### Replay v13 Consecutive 진입 필터

`consecutive_move` params의 `min_cumulative_move_pct`(유한 0..100), `min_move_atr_multiplier`(유한 0..20), `long_trend_period`(정수 0..512)는 각각 0이 비활성이다. 기존 `buy_days`/`sell_days`는 관측 횟수이며 legacy 사용자 값과 기본 3/3을 보존한다. 신규 필드 누락은 기본 0, 문자열/null/범위 밖은 저장·preview에서 거부한다. 손상된 저장 설정을 읽는 runtime은 신규 진입을 차단하면서 기존 보유 매도 기간을 유지한다.

누적 조건은 현재와 buy_days 관측 이전 가격의 변화율 이상이다. ATR은 같은 native 상승폭이 직전 완료 일봉 15개에서 산술 평균한 TR14×배수 이상이며 현재 후보 OHLC는 제외한다. 장기 MA는 현재 가격이 직전 완료 종가 N개의 평균보다 엄격히 높아야 한다. 복수 활성은 AND다. ATR/MA 활성 generic은 `D`/`1d`만 허용하며 다른 간격은 `UNSUPPORTED_ENTRY_FILTER_INTERVAL`이다. 기존 sell 조건과 공통 guard/risk·체결 피드백은 필터를 통과한 뒤에도 유지한다.

기존 short/continuous 응답 최상위에 선택적 `entryFilterPreparation`을 반환한다. 전 필터 비활성이면 생략하며 legacy 응답의 부재를 준비 완료로 추정하지 않는다. UI는 요청/보관 params가 모두 유효 숫자0 또는 누락이면 알려진 비활성을 중립 표시하고, 활성 필터인데 진단이 없을 때만 준비 미확인 경고를 표시한다. `backtest.atrPreparation`은 수량 조절용으로 별도 계약이다.

| 필드 | 의미 |
|---|---|
| enabled / ready / status | 활성·현재 진입 준비와 ready/insufficientHistory/invalidParams/unsupportedObservation/zeroAtr |
| cadence | priceObservations 또는 priorCompletedDailyOhlc |
| requiredCompletedBars / availableCompletedBars / trueRangeSamples | 완료 OHLC 맥락과 TR 준비 수 |
| candidates / unreadyCandidates | 미보유+기존 연속상승 조건의 진입후보 및 준비 부족 보류 수 |
| minMoveRejections / atrRejections / trendRejections | 유효 후보에서 각 조건 거절 수; 복수 필터에서는 중복 집계 가능 |
| passedCandidates | 필터 통과 raw 매수 수; 실제 주문·체결 수와 다름 |

live는 약10초 quote 관측열과 별도로 범위·세대·시장 날짜가 확인된 완료 일봉 context를 사용한다. context 갱신·실패·시장 날짜 전환은 관측열과 확정 보유를 보존하며 준비 부족과 0 ATR은 신규 매수만 보류한다. replay는 현재 후보 종가 평가와 체결 피드백 후 해당 OHLC를 다음 판단의 완료 자료로 게시한다. A/B는 요청 params와 이 선택 진단을 함께 보존한다. 단위 문구 두 개 외 기본 전체 report/signals 호환과 한계는 [P2-06 연구](research/consecutive-filters-20261018/report.md)에 기록한다.


### Replay v14 Momentum 소유 피드백과 선택 진입 교차

`momentum` params에 `entry_mode`=`level`(기본)/`crossing`, `entry_hysteresis_pct`=유한0..100(기본0, %p)를 추가했다. 누락은 기본값으로 읽고 기존 custom `lookback_period`/`threshold_pct`를 보존한다. 새 필드의 잘못된 enum/null/문자열/범위는 기존 설정 저장·short/continuous preview에서 `INVALID_STRATEGY_PARAMS`로 거부한다. crossing에서는 간격이 기존 threshold_pct 이하여야 한다. level에서는 유효한 간격 값을 보관하지만 사용하지 않으며 UI에 미사용을 표시한다. 저장된 optional 필드가 손상되면 runtime 신규 매수는 보류하되 legacy 기간/임계값과 보유 매도 평가를 보존한다.

모멘텀은 N관측 전 가격 대비 변화율이며 live는 약10초 quote 관측, replay는 선택 봉 종가다. 기존 경계인 매수 `m>=T`, 매도 `m<=−T`를 유지한다. 새 기본 level은 실제 소유량0일 때 매수, 양수일 때 매도만 만든다. raw 신호로 확정 소유량을 바꾸지 않으며 기존 guard/risk와 실제 체결·소유 피드백을 사용한다.

crossing은 이전m<T→현재m>=T에서 신규 진입 후보를 만든다. m<=T−간격에서 무장하고, 후보가 활성화된 뒤 m<T이면 취소한다. T5/간격1%p이면 m<=4에서만 재무장하며 4<m<5는 후보 취소만 가능하다. 엄격한 사전자료는 이전m/무장 상태를 seed하되 가상 후보·체결을 만들지 않는다. 동일0수량 게시와 nextOpen 대기는 실제 체결이 아니며, 유효 후보는 m>=T 동안 미체결 재시도가 가능하다. 실제 양수 소유량 확인으로 후보를 소모한다. 두 모드 모두 보유 negative level 청산을 유지해 거절·실패·부분매도 후 잔여 양수 수량의 청산을 계속 평가한다. 실제 provider 매도 수량은 기존 공통 경계에서 요청량과 현재 소유량의 최솟값으로 제한한다.

새 endpoint/응답 DTO는 없다. 기존 params 전송·preview key 무효화·A/B 저장에 두 필드를 보존하고 화면에 모드/재무장 %p와 차이를 표시한다. 단기/연속 입력 상한·LTH 평가 불가·기존 Consecutive 진입 진단 계약은 유지한다. 비모멘텀1122회 exact호환, Momentum102창과 고정3년/별도분기의 raw 억제·체결/원가 대조 및 한계는 [P2-03 연구](research/momentum-feedback-20261020/report.md)에 기록한다.


### Replay v15 Deviation 소유·진입·보유 보호

새 IPC/응답 DTO 없이 기존 `deviation*` params에 다음 선택 필드를 추가한다. 누락은 기본값이며 기존 custom MA·매수/매도 임계값을 보존한다. enum/null/string/비유한수·범위·소수 정수 오류는 저장과 단기/연속 preview에서 `INVALID_STRATEGY_PARAMS`로 거부한다. 손상된 runtime optional 설정은 신규 매수를 차단하며 기존 보유의 유효 청산 조건은 유지한다.

| 필드 | 기본 / 범위 | 의미 |
|---|---|---|
| `entry_mode` | `level` / `level`, `crossing` | 기본 상태 진입 또는 하향 교차 진입 |
| `long_trend_period` | 0 / 정수0..512 | 현재 제외 직전 N관측 평균보다 현재 가격이 높을 때만 신규 진입. 0 비활성 |
| `max_hold_observations` | 0 / 정수0..1000000 | 실제 체결 뒤 보유 관측 수의 매도 후보 기준. 0 비활성 |
| `stop_loss_pct` | 0 / 유한0..100 | 실제 native 평균체결가 대비 관측 가격 하락률의 손절 후보. 0 비활성 |

기존 이격도는 현재 관측을 포함한 MA(기본20)에 대한 변화율이며 매수 `d<=−5`, 매도 `d>=+5`의 inclusive 경계를 유지한다. 확정 소유량은 exact owner 피드백에서만 바꾸며 flat Buy/held Sell로 부적합 후보를 제거한다. crossing은 이전d>매수임계값→현재d<=매수임계값에서 후보를 만든다. 현재d가 임계값 이하인 동안 동일0수량·guard 거절·nextOpen 대기에도 재시도하며 상회하면 취소한다. 양수 실제 체결은 후보를 소모하고 부분 매도 뒤 잔량은 계속 보유다. strict 준비 자료는 가상 체결·후보·보유 나이를 만들지 않는다.

longtrend는 같은 평가 cadence의 직전 N가격이다. replay의 strict 과거 봉을 seed할 수 있지만 live 최초 관측은 별도 trend 버퍼를 비우고 실제 live 관측 N개를 모은 뒤 평가한다. 공통 일봉/1분봉 warmup을 약10초 live 관측으로 인증하지 않는다. 기존 MA warmup은 별도 유지하며 준비 부족 중에도 보유 손절·관측 한도 청산은 평가한다. UI 준비 수치는 이 진입 준비와 실제 조건 평가를 구분한다.

보유 관측은 첫 양수 체결 뒤 평가부터 옵션 비활성 중에도 센다. 동일 수량 게시·부분 매도·준비 자료·설정 변경은 기록 나이를 줄이지 않는다. exact owner 보호 metadata는 보유 관측을 복원하고, 실제 진입 후보나 유효 metadata 없이 복원된 보유는 나이 불명이다. 관측 한도가 활성이라면 다음 관측에 보수적인 매도 후보를 만든다. 이는 실제 매수일부터의 거래일·시간 한도나 체결 보장이 아니다.

청산 우선순위는 손절→보유 관측 한도→기존 고이격 level이다. 두 보호 청산은 소유 잔량 전량을 요청하고 기존 level은 설정 수량을 유지한다. 공통 owner/보유 clamp·guard·risk·미체결 억제는 유지하며 거절·부분체결 후 조건을 다시 평가한다. 손절은 현재 관측 가격의 native 손실 조건이며 비용/FX를 포함한 손익이나 장중 손실 상한이 아니다. nextOpen에서는 다음 관측 시가·갭·미체결이 존재한다. 기존 손절 후 당일 재진입 차단도 유지한다.

UI는 네 필드의 raw 초안·유효성·활성 잠금·저장/두 preview 차단과 현재/A-B 파라미터 요약을 함께 제공한다. 고정 기본 교정과 단독 후보의 실제 체결/회계·비용·노출 차이 및 한계는 [P2-04 연구](research/deviation-policy-20261020/report.md)에 기록한다.


### Replay v16 MeanReversion 실제 소유와 선택 청산

새 endpoint/DTO 없이 기존 `mean_reversion*` params에 선택3필드를 추가한다. 기존 `period`/`std_dev`/`stop_loss_pct` custom 값은 optional 손상으로 기본값에 덮지 않는다. enum/null/string/비유한수·범위·소수 정수 오류는 저장과 short/continuous preview에서 `INVALID_STRATEGY_PARAMS`다. runtime 손상은 신규 Buy를 보류하고 기존 유효 보유 청산을 유지한다.

| 필드 | 기본 / 범위 | 의미 |
|---|---|---|
| `exit_mode` | `upperBand` / `upperBand`, `middleBand` | 기존 상단 또는 같은 평균 중심선의 엄격 초과 청산 |
| `long_trend_period` | 0 / 정수0..512 | 현재 제외 이전 N가격 평균보다 현재가가 높을 때 신규 진입. 0 비활성 |
| `max_hold_observations` | 0 / 정수0..1000000 | 실제 체결 뒤 보유 관측 수의 청산 후보. 0 비활성 |

기존 밴드는 현재 포함 최근 period(기본20) 가격의 population variance(ddof0)와 표준편차배수2다. 진입은 price<lower, 기본 청산은 price>upper, middleBand는 price>같은mean이며 등호는 제외한다. 실제 owned quantity/native avg는 exact owner 피드백에서만 바꾸며 raw on_tick/live는 보유·원가를 선반영하지 않는다. 기존 손절은 ((nativeAvg−현재가격)/nativeAvg*100)>=stop_loss_pct 공식과 configured 주문 수량을 유지한다. **stop_loss_pct=0은 비활성이 아니라 원가 이하 손절**이다. 새 두 숫자 옵션의0 비활성과 혼동하지 않는다.

우선순위는 기존 손절→보유 관측 한도→선택 밴드다. 손절/밴드 청산은 설정 수량, 새 관측 한도는 실제 잔량 전량 후보를 만들며 공통 clamp·guard/risk·대기/거절/부분체결 뒤 재평가를 유지한다. 지표 준비가 부족해도 보유 손절·한도는 평가한다. 종가 관측 신호와 다음 관측 시가/갭·비용 때문에 장중 손실 상한 또는 날짜/체결 보장이 아니다.

추세 버퍼는 live 최초 관측에서만 warmup seed를 비우고 실제 live 약10초 가격을 모은다. 현재 후보를 넣기 전 이전 N평균과 strict 현재>평균을 확인한다. 일봉 replay는 strict 이전 종가를 seed한다. MeanReversion의 legacy 분봉 초기화 훅은 no-op이므로1분 preview의 사전분봉을 밴드/추세 준비로 추정하지 않고 평가 관측부터 모은다. reset은 legacy 가격 버퍼 보존과 보유 보호 정리를 유지한다.

보유 나이는 off 중에도 실제 양수 체결 뒤 관측을 기록하고 sameqty/partial/warmup/reconfigure로 줄이지 않는다. exact owner metadata의 kind=mean_reversion을 엄격 검증하며 다른 전략 checkpoint를 거부한다. pending origin은 체결0나이로, 유효 known checkpoint는 기록 나이로 복원한다. 둘 다 없는 positive 복원은 unknown이며 활성 한도에서 다음 관측 보수적 매도 후보다. 관측수는 live/선택봉마다 기간이 다르다.

공통 `strategy/observation.rs`는 두 전략에서 동등성이 확인된8개 내부 함수(정수·age 전이·codec·이전N추세·bounded/live초기화)만 제공한다. q/avg·symbol/scope 검사·전략별 판정/초기화/재설정/손절0 의미는 caller에 남는다. codec은 검증 성공 뒤 caller가 게시하므로 malformed에서 상태를 보존한다. 기존 frozen QQQ의 비용/체결4경로를 유지하고 [P2-10 사전 선언·단독 비교·한계](research/mean-reversion-policy-20261020/report.md)에 기본 호환과 선택 효과를 구분한다. UI는 raw 필드·유효성·활성 잠금·저장/두 preview 차단·현재/A-B params를 함께 표시한다.


### Replay v17 MA 교차의 이전 완료 일봉 진입 확인

`ma_cross*`의 기존 `short_period=5`, `long_period=20` 교차는 입력 가격 관측 기준이며 기본 live 약10초 관측과 일봉/분봉 replay를 구분한다. 선택 params는 모두0 기본이고 Buy 후보에만 적용한다. `long_trend_period`는0..512 정수, `min_long_ma_slope_pct`·`min_atr_pct`는0..100 유한수, `min_bandwidth_pct`는0..1000 유한수다. 기울기 활성시 `long_period`는1..511이어야 한다. null/string/비유한수·범위·분수 정수를 save와 단기/연속 preview에서 `INVALID_STRATEGY_PARAMS`로 거부한다. 손상 optional runtime은 기존 custom 교차 기간·Sell을 유지하며 Buy만 차단한다.

필터는 strict 이전 완료 DAILY OHLC를 사용한다. 추세는 현재 후보가격>이전N 종가MA, 기울기는 `(직전 long_period일 MA / 한 일봉 전 MA−1)×100`, ATR%는 이전15OHLC의14TR 산술평균/직전완료종가×100, 밴드폭%는 이전20종가의4×모집단표준편차/평균×100이다. 추세는 strict>, 나머지 활성 양수 임계값은 >=다. 필요한 일봉 수는 각각N/L+1/15/20의 최댓값이며512 이하다. 충분한 유효자료의 ATR0/폭0은 준비 완료·양수 임계값 미달이다.

현재 일봉 OHLC는 후보 판단 후에만 게시한다. apply/invalidate 완료 일봉 hook은 기존 가격 관측열·prevMA·Sell을 건드리지 않으며 scoped live daily coordinator를 재사용한다. 활성 옵션의1분 preview는 필요한 일봉 맥락을 증명할 수 없어 명시 거부하고, 옵션0은 기존 분봉 경로를 유지한다. 거절된 골든크로스에도 prevMA를 갱신하므로 이후 필터가 통과했다는 이유로 과거 교차를 되살리지 않는다. 기존 무보유Sell/보유Buy 후보와 공통guard/소유clamp/risk는 유지한다.

기존 선택 `entryFilterPreparation`에 MA 전용 `slopeRejections?: number`, `bandwidthRejections?: number`를 추가한다. Consecutive 응답에서는 생략해 기존 JSON을 보존한다. MA `cadence=priorCompletedDailyOhlc`, status는 기존 ready/invalidParams/unsupportedObservation/insufficientHistory를 사용한다. `minMoveRejections`는0이며 UI는 추세/기울기/ATR/폭 거부를 각각 표시한다. candidates는 소유 검사 전 raw 골든크로스 후보, 거부 사유는 중복 가능하고 passed는 체결 수가 아니다. 설정과 진단을 A/B 슬롯에도 보존한다. 수량 산정 ATR과 별개다. [연구](research/ma-cross-filters-20261020/report.md).

### LTH 분봉 시각 진단 v1 (2026-10-22)

전용 `1m`의 `replay.assessment.minuteTiming`은 선택 객체이며 `diagnosticVersion=lth-minute-timing-v1`이다. `1d`/generic에는 생략한다. 기존 engine v17/LTH v4·입력 hash·시계·신호·정산은 유지한다. 요청 시작의 `referenceAsOf`를 고정하고, 준비된 실행 label을 읽기만 한다. sourceRows/selectedRows/usedRows/priceExcludedRows는 provider 결과/기존 lexical tail 선택/유효 가격 관측/가격 제외를 구분한다. 진단 처리 상한512행·label64바이트·날짜8개·예시6개이며 inputTruncated/omittedDayRows를 반환한다. 전체 합계는 처리한 입력 범위에서 계산하고 날짜 목록만 줄인다.

RFC3339는 실제 순간을 America/New_York 또는 Asia/Seoul로 변환한다. compact12/14는 기존 KST 가정으로 별도 집계한다. `labelConvention=barClose`인 Toss는 label−60초/label을 봉 시작/종가 공개 가능 시각 모델로 제공한다. 내부 barStart fixture는 label/label+60초, unknown은 세션·공개 가능 시각·공백 판정을 하지 않는다. `firstMarketLocal/lastMarketLocal`은 원본 label 순간의 시간순 경계이며, 원본 실행 순서를 바꾸지 않는다. duplicateInstants/outOfOrderRows/nonMinuteAlignedRows/unfinishedRows는 동일 순간/실제 순간 역순/초·소수초/기준시각 뒤 종가를 센다. 진단 때문에 실행 관측을 제외하지 않는다.

일별 days는 일반 평일 정규 시간창 안의 정렬된 고유 분 격자를 산술 비교한다. interiorUnobservedSlots/maxInteriorGapSlots는 같은 관측 날짜 내부이며, 앞뒤 시간창과 무관측 날짜·야간·주말을 실제 누락으로 확정하지 않는다. regularRows는 중복·초단위도 포함한 시간창 분류 수, regularUniqueBars는 분 경계의 고유 격자 수다. 달력·자료 완전성·요청 기간·실제 발행/수신 검증 flag4개는 false다. legacySessionPolicy=legacyFixedKstUnverified, sampleOnly/notEvaluable·timestamp/session unverified·lookAheadSafe=false를 유지한다. UI/A-B는 객체를 보존하고 이전 응답의 객체 부재·미지원/손상 형식을 경고하며 봉 표기·시간대·진단 버전·검증 가정 차이를 따로 표시한다. 실제3개월 검증은 P1-05/P2-12에 남는다.

### RSI v18 실제 소유·선택 보호

RSI는 기존 period14/oversold30/overbought70 단순평균 계산과 교차 기준을 유지하며 실제 owned 수량·native 평균가만 체결 피드백으로 반영한다. flat Buy/held Sell을 여과하고 Sell은 확정 잔량 전량을 요청한다. 기존 교차는 매 관측 prevRSI를 갱신하며 거절 뒤 지속조건으로 재시도하지 않는다.

선택 `long_trend_period`는 정수0..512, `stop_loss_pct`는 유한0..99.9, `max_hold_observations`는 정수0..1000000이다. 누락/0은 비활성이며 null/문자열/bool/소수정수/범위초과는 저장과 공통 bounded/continuous driver에서 거부한다. runtime optional 손상은 custom legacy RSI와 보유 청산을 보존하며 신규 Buy만 차단한다. 추세는 현재제외 이전N 입력가격 평균을 현재가가 엄격히 상회해야 한다. live 첫 관측의 새 추세 사전자료만 비우므로 입력관측N은 일수N이 아니다.

청산은 실제평균가 손실≥손절률 → 알려진보유관측≥한도/미상나이 보수적후보 → 기존 과매수 하향교차 순서다. stop/cap은 차단/부분체결뒤 조건지속·잔량양수에서 다시후보를내며 공통guard/리스크를유지한다. 최초양수체결뒤age0/다음관측1, 같은수량·부분매도·warmup·reconfigure는age를지우지않는다. origin없는복원은unknown, 정확한owner v1 kind=rsi metadata로만알려진age를복원한다. RSI준비/추세부족은기존보유보호를막지않는다. UI/current/A-B는실제params와관측단위·기본0·미확인보관결과를표시한다. [작업](tasks/rsi-policy-20261022.md), [단독 비교](research/rsi-policy-20261022/report.md).
