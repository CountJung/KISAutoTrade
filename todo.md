# KISAutoTrade — Todo

> 한국투자증권(KIS) 전용 자동매매 앱을 다중 증권사 구조로 확장하기 위한 백로그입니다.  
> 완료 이력은 `git log --oneline`과 PR/커밋 메시지에 맡기고, 이 문서는 앞으로 할 일을 우선순위 중심으로 유지합니다.

---

## 공식 문서 참조

토스증권 Open API는 공식 웹에서 계속 업데이트되므로, 구현 시 아래 문서를 먼저 확인한다.

| 용도 | 공식 경로 |
|------|----------|
| 브라우저 문서 | https://developers.tossinvest.com/docs |
| AI/비브라우저 진입점 | https://developers.tossinvest.com/llms.txt |
| 개요 Markdown | https://openapi.tossinvest.com/openapi-docs/overview.md |
| OpenAPI Markdown | https://openapi.tossinvest.com/openapi-docs/latest/api-reference/README.md |
| OpenAPI JSON 원본 | https://openapi.tossinvest.com/openapi-docs/latest/openapi.json |
| 서비스 소개 | https://corp.tossinvest.com/ko/open-api |

- 2026-07-03 확인 기준: OpenAPI title `토스증권 Open API`, version `1.1.5`, base URL `https://openapi.tossinvest.com`, paths 20개.
- 엔드포인트, 요청/응답 스키마, 인증, 에러, rate limit은 항상 OpenAPI JSON을 source of truth로 본다.
- 계좌·자산·주문 API는 `Authorization: Bearer {access_token}` 외에 `X-Tossinvest-Account` 헤더가 필요하다.
- 현재 공식 개요 기준 토스증권 Open API는 REST API만 제공한다. WebSocket 지원 여부는 작업 시점의 공식 문서로 재확인한다.
- 구현 중 새로 검증한 토스증권 API 동작·제한·에러는 새 `toss-api` 스킬 또는 관련 문서에 즉시 남긴다.

---

## 유지할 기존 완료 전제

- 공통 `TradeGuard`와 `RiskManager`는 증권사와 무관한 주문 전 방어 계층으로 유지한다.
- 주문번호 기반 체결 확인, 부분체결/미체결/거부 상태 분리, 실패 주문 이력 저장 패턴은 토스증권 연동에도 동일하게 적용한다.
- 국내/해외 포지션 분리, 수수료·환율·슬리피지 기록, ATR 기반 주문 수량 산정은 broker adapter 아래 공통 도메인 모델로 이전한다.
- React FSD 구조와 `shared/api` IPC wrapper, `entities`, `features`, `widgets`, `pages` 경계는 새 증권사 UI 추가 시에도 유지한다.
- 민감 정보 파일(`.env`, `secure_config.json`, `profiles.json`)은 읽지 않고, 새 토스증권 키도 동일한 보안 원칙으로 저장한다.

---

## P0 — 운영 안정성 / 대형 파일 분리

- [x] 1000라인 초과 소스 파일을 책임 단위로 분리한다.
  - Rust: `commands.rs`의 프로파일/잔고/Toss read-only/preflight, 자동매매 runtime, 전략/리스크, 설정/시세/주문/로그/보관 surface는 `src-tauri/src/commands/{accounts,toss,toss_market,trading,trading/history,strategy,risk,settings,market,orders,records,archive}.rs`로 분리했다. `server/mod.rs`의 market/records/profiles/Toss/trading REST surface는 `src-tauri/src/server/{market,records,profiles,toss,trading}.rs`로 분리했다. `trading/strategy.rs`는 facade로 낮추고 `trading/strategy/{core,manager,state,classic,breakout,mean_trend,leveraged_trend_hold,price_condition,tests}.rs`로 분리했다. `broker/toss.rs`는 `broker/toss/{mod,adapter,client,http,error,support,types,orders,tests}.rs`로 분리했다.
  - 현재 점검 기준 source 파일은 모두 1000라인 아래다. 이후 신규/변경 파일도 가능한 한 1000라인 아래로 유지하고, 불가피한 예외는 분리 축과 후속 작업을 문서화한다.
- [x] `src-tauri/src/api/rest.rs`에서 KIS REST 타입과 공개 환율 fetcher를 `api/rest/types.rs`, `api/rest/exchange.rs`로 분리해 facade 파일을 1000라인 아래로 유지한다.
- [x] `src-tauri/src/trading/order.rs`에서 체결 처리, pending 충돌 helper, lock-short 주문 제출을 `trading/order/fills.rs`, `trading/order/conflicts.rs`, `trading/order/submission.rs`로 분리해 OrderManager facade를 1000라인 아래로 유지한다.
- [x] `src-tauri/src/commands.rs`에서 프로파일/잔고/Toss read-only/preflight IPC surface를 `commands/accounts.rs`, `commands/toss.rs`, `commands/toss_market.rs`로 분리하고 새 파일을 1000라인 아래로 유지한다.
- [x] `src-tauri/src/commands.rs`에서 자동매매 status/start/stop/daemon/sync를 `commands/trading.rs`, 히스토리/ATR 초기화를 `commands/trading/history.rs`, 포지션/전략 IPC를 `commands/strategy.rs`, 리스크/pending IPC를 `commands/risk.rs`로 분리하고 새 파일을 1000라인 아래로 유지한다.
- [x] `src-tauri/src/commands.rs`에서 app 설정/refresh/log/web/환율, 시세/차트/종목검색/해외 주문 사전검증, 수동 주문, 체결/통계/로그, trade archive IPC를 `commands/{settings,market,orders,records,archive}.rs`로 분리하고 facade와 새 파일을 1000라인 아래로 유지한다.
- [x] `src-tauri/src/server/mod.rs`에서 market/records/profiles/Toss/trading REST surface를 `server/{market,records,profiles,toss,trading}.rs`로 분리해 server facade와 새 파일을 모두 1000라인 아래로 유지한다.
- [x] `src-tauri/src/trading/strategy.rs`에서 Strategy core/manager/state/helper와 전략군을 `trading/strategy/*` 하위 모듈로 분리해 facade와 새 파일을 모두 1000라인 아래로 유지한다.
- [ ] Rust 중복 view/helper와 장기 운영 리스크를 정리한다.
  - [x] `commands.rs`/`server/mod.rs`의 strategy view를 `src-tauri/src/trading/views.rs` 공용 builder로 승격한다.
  - [x] risk view, pending order view, archive stats builder 중복을 공유 builder/service로 승격한다.
  - [x] IPC/REST 프로파일 view/masking은 `commands/accounts.rs::profile_to_view()`로 공용화한다.
  - [x] `get_strategies`는 `strategy_manager` lock을 잡은 채 stock name lookup을 await하지 않도록 clone-then-await 패턴으로 바꾼다.
  - [x] IPC/REST recent logs count는 reader clamp와 별도로 handler 레벨에서도 상한을 둔다.
  - [x] Toss HTTP client에 명시 timeout과 응답 크기 guard를 두고, token refresh가 token mutex를 잡은 채 네트워크 요청을 기다리지 않도록 분리한다. 응답 body는 `Content-Length` 사전 검사뿐 아니라 chunk 누적 중에도 상한 초과 전에 중단하고, 파싱/에러 메시지에는 전체 body 대신 snippet만 포함한다.
  - [x] KIS token client에 명시 timeout을 두고, token refresh가 token mutex를 잡은 채 네트워크 요청을 기다리지 않도록 분리한다.
  - [x] 자동매매 daemon의 환율 조회가 `order_manager` mutex를 잡고 내부 RwLock await를 다시 호출하지 않도록 `exchange_rate_krw`를 직접 참조한다.
  - [x] 웹 REST `/api/trading/start`가 Toss 등 미지원 broker에서 `is_trading`만 켜지 않도록 KIS-only gate, KIS 설정 검증, `OrderManager` 실행 scope 설정, 시작 전 KIS 잔고 기반 전략 포지션 복원을 수행한다.
  - [x] 웹 REST `/api/archive-config` 변경도 IPC `set_trade_archive_config`처럼 저장 후 old trade purge를 즉시 예약한다.
  - [x] KIS read API에 `kis:account`, `kis:execution`, `kis:quote` rate-limit group을 적용하고 응답 rate-limit header를 scheduler에 반영한다.
  - [x] 전략 설정 업데이트와 프로파일 scope 적용 시 전략 인스턴스를 재빌드해 파라미터/대상 종목 변경이 즉시 반영되고 이전 종목 state가 잔류하지 않게 한다.
  - [x] 전략 대상 종목 판정은 `StrategyConfig::targets_symbol()`로 공용화해 tick마다 `String`을 할당하지 않고, user-param 기반 `VecDeque`는 `trading/strategy/state.rs` bounded helper로 상한을 둔다.
  - [x] KIS/Toss 주문 daemon의 `order_manager` lock-held-await를 분리한다.
    - [x] 주문번호 기반 KIS 체결 조회 네트워크 호출은 `OrderManager::confirm_pending_fills_from_broker_shared()`로 `order_manager` mutex 밖에서 수행한다.
    - [x] 전략 신호 주문 제출 경로는 `OrderManager::submit_signal_shared()`에서 lock-short guard/예약, lock-free provider 주문 API/저장, lock-short 상태 반영으로 분리하고, provider 호출 중 중복/반대 주문은 `submitting` 예약 맵으로 차단한다.
- [x] `src/pages/settings/ui/Page.tsx`에서 계좌 프로파일 섬을 `accountProfiles.tsx`, `profileDialogs.tsx`, `profileUtils.ts`, `section.tsx`로 분리해 Settings route와 새 파일을 모두 1000라인 아래로 유지한다.
- [x] `src/pages/strategy/ui/Page.tsx`에서 레버리지 추세 보유 편집 패널을 `leveragedTrendHoldEditorPanel.tsx`로 분리해 Strategy route 파일을 1000라인 아래로 유지한다.
- [x] `src/pages/dashboard/ui/Page.tsx`에서 미체결/체결 주문 패널을 `orderPanels.tsx`로 분리해 Dashboard route 파일을 1000라인 아래로 유지한다.
- [x] `src/api/hooks.ts`에서 `queryKeys.ts`와 `backendEvents.ts`를 분리해 query key/event bridge 책임을 낮추고 파일을 1000라인 아래로 유지한다.
- [x] Log 페이지 최근 로그 조회를 bounded tail reader + count별 query key로 변경해 큰 로그 파일에서 OOM/jank 위험을 낮춘다.
- [x] 백그라운드 KIS 잔고 갱신 daemon은 활성 broker가 KIS일 때만 KIS API를 호출한다.

---

## P0 — 토스증권 공식 API 조사/스킬화

- [x] 토스증권 OpenAPI JSON을 기준으로 endpoint inventory 작성
  - 인증: `POST /oauth2/token`
  - 시세: orderbook, prices, trades, price-limits, candles
  - 종목/시장: stocks, warnings, exchange-rate, KR/US market-calendar
  - 계좌/자산: accounts, holdings
  - 주문: create, modify, cancel, list, detail, buying-power, sellable-quantity, commissions
- [x] KIS API 스킬과 분리된 `toss-api` 문서/스킬 초안 작성
  - OAuth2 Client Credentials Grant
  - `X-Tossinvest-Account` 계좌 헤더
  - 공식 rate limit 그룹과 `Retry-After`, `X-RateLimit-*` 헤더 처리
  - 공식 error envelope와 주요 에러 코드
- [x] `scripts/` 또는 문서에 공식 OpenAPI 최신 버전 확인 절차 추가
  - 작업 시작 시 `openapi.json`의 `info.version`, `servers`, `paths` 개수를 확인한다.
  - 스냅샷을 저장할 경우 민감 정보가 없는 공식 spec만 저장하고, generated code와 수동 수정 코드를 분리한다.

## P0 — 다중 증권사 아키텍처 설계

- [x] `KisRestClient` 중심 구조를 `BrokerClient`/`BrokerAdapter` 구조로 분리
  - 공통 trait 후보: auth, price, candle, balance/holdings, order, order status, market calendar, commission.
  - KIS 전용 TR-ID/계좌번호 분리 로직은 `KisBrokerAdapter` 내부에 격리한다.
  - 토스 전용 OAuth2, account header, 통합 국내/미국 symbol 모델은 `TossBrokerAdapter` 내부에 격리한다.
- [x] 공통 도메인 타입 정리
  - `BrokerId`, `BrokerAccountId`, `Market`, `Symbol`, `Currency`, `Money`, `Quantity`, `OrderId`, `ClientOrderId`.
  - KIS KRW/해외 USD cents 처리와 토스 Decimal/string 금액 처리를 안전하게 매핑한다.
- [x] AppState와 프로파일 설정을 다중 broker aware 구조로 확장
  - 현재 활성 broker/profile/account를 명시한다.
  - 자동매매 시작 시 전략별 대상 broker를 고정해 중간 전환으로 주문이 섞이지 않게 한다.

## P1 — 토스증권 인증/설정 UI

- [x] Settings에 토스증권 Open API 프로파일 추가
  - `client_id`, `client_secret`, 계좌 식별값, 실거래 사용 동의 상태를 KIS 프로파일과 분리한다.
  - 토스증권 WTS 설정 > Open API 메뉴에서 키를 발급받는 흐름을 도움말로 연결한다.
  - Settings Add/Edit 다이얼로그에서 broker 선택과 토스 `Client ID`/`Client Secret`/`accountSeq` 라벨을 제공한다. `live_trading_consent`는 별도 저장 필드이며, 주문/자동매매 unlock이 아니라 향후 소액 검증 gate의 명시 승인 기록으로만 사용한다.
- [x] 토큰 발급/갱신 모듈 추가
  - KIS `TokenManager`와 독립된 `TossTokenManager` 또는 공통 `OAuthTokenManager`를 검토한다.
  - 만료/401/`expired-token` 시 1회 재발급 후 재시도한다.
- [x] 연결 진단 IPC 추가
  - OpenAPI version 확인, token 발급 가능 여부, accounts 조회 가능 여부를 단계별로 표시한다.
  - `check_toss_profile_connection`이 OpenAPI spec, token 발급, accounts 조회, holdings 조회를 단계별로 반환하고 Settings 프로파일 카드에서 실행한다.

## P1 — 시세/종목/캔들 연동

- [x] 토스 현재가/호가/최근 체결/상하한가 조회 구현
  - KIS `get_price` 계열과 동일한 UI 표면을 유지하되 provider별 원본 필드는 보존한다.
  - `TossOpenApiClient`/`TossBrokerAdapter`에는 `prices`, `orderbook`, `trades`, `price-limits` read-only 조회와 `BrokerPriceQuote` 매핑 테스트가 있다.
  - `get_toss_market_snapshot` IPC, `/api/toss-market-snapshot/:symbol` REST, `useTossMarketSnapshot()` 훅을 추가했고, Trading 화면은 활성 broker가 Toss일 때 현재가/호가/최근 체결/상하한가 snapshot을 표시한다.
  - 활성 Toss 프로파일에서는 기존 KIS 가격/차트/수동 주문 호출을 막고 read-only 안내를 표시한다.
- [x] 토스 candles를 기존 lightweight-charts 데이터 구조로 매핑
  - 1분봉/일봉 지원 범위와 query parameter는 공식 OpenAPI JSON으로 작업 시점 재확인.
  - 현재 Toss candles는 `1m`/`1d`, count 1~200, `before`, `adjusted` 쿼리를 지원하는 client 메서드와 `BrokerCandle` 매핑 테스트가 있다.
  - `get_toss_chart_data` IPC, `/api/toss-chart/:symbol` REST, `useTossChartData()` 훅을 추가했고, Trading 화면은 활성 broker가 Toss일 때 기존 `StockChart`를 `source="toss"`로 재사용한다.
  - Toss 차트 preset은 일봉/1분봉으로 제한하고, 일봉은 `YYYYMMDD`, 1분봉은 provider timestamp를 lightweight-charts `Time`으로 변환한다.
- [x] 토스 종목 기본 정보와 warnings를 종목 검색/주문 전 검증에 연결
  - 거래 제한, 투자경고, VI, 정리매매 등은 주문 전 guard에서 차단 또는 확인 요구한다.
  - `TossOpenApiClient`/`TossBrokerAdapter`에 `stocks`, `stocks/{symbol}/warnings` read-only 조회를 추가했다. `warnings`의 unknown code는 문자열로 보존한다.
  - `get_toss_stock_safety` IPC, `/api/toss-stock-safety/:symbol` REST, `useTossStockSafety()` 훅을 추가했고, Trading 화면은 활성 broker가 Toss일 때 종목 기본 정보와 매수 유의사항을 표시한다.
  - `TossStockSafetyView.buyBlocked`/`buyBlockReason`은 상장 상태와 blocking warning을 주문 전 검증 후보로 내려준다. 실제 Toss 주문 생성 경로 연결은 주문 adapter와 소액 검증 gate 이후 진행한다.
- [x] 토스 KR/US market-calendar를 장 시간 판단에 통합
  - 기존 `market_hours.rs`의 하드코딩 보완 후보로 사용한다.
  - `TossOpenApiClient`/`TossBrokerAdapter`에 `market-calendar/KR`, `market-calendar/US` read-only 조회와 serde 매핑 테스트를 추가했다.
  - `market_hours.rs`는 `MarketCalendarOverride`를 받으면 공식 regular session으로 개장 여부를 판단하고, calendar가 없으면 기존 KST 하드코딩 로직으로 fallback한다.
  - `get_toss_market_calendar` IPC, `/api/toss-market-calendar` REST, `useTossMarketCalendar()` 훅을 추가했고, Trading 화면은 활성 broker가 Toss일 때 KR/US 정규장 상태를 표시한다.
  - 자동매매 데몬의 장 시간 사전 체크와 종목별 tick skip 경로는 Toss 활성 프로파일 calendar override를 받을 수 있게 연결했다. 현재 Toss 자동매매 시작은 주문 adapter gate 전까지 계속 차단된다.

## P1 — 계좌/잔고/포지션 통합

- [x] 토스 accounts/holdings 조회 구현
  - KIS 국내/해외 잔고와 동일한 Dashboard/Position UI에 표시한다.
  - 통화별 평가금액, 손익, 수량 precision을 공통 타입으로 정규화한다.
  - `TossOpenApiClient`/`TossBrokerAdapter`의 accounts 조회와 holdings → `BrokerHolding` 매핑을 `get_broker_holdings` IPC, `/api/broker-holdings` REST, `useBrokerHoldings()` 훅으로 연결했다.
  - Dashboard는 활성 broker가 Toss일 때 `BrokerHoldingView` 기반 보유 종목 섹션을 표시하며, 금액/수량은 문자열 precision을 보존한 뒤 화면 표시 시에만 포맷한다.
- [x] 자동매매 시작 시 토스 holdings 기반 전략 포지션 동기화
  - 기존 `Strategy::sync_position()` 흐름에 broker 인자를 추가한다.
  - `BrokerPositionSnapshot { brokerId, market, symbol, quantity, avgPrice }` 기반 `sync_position_for_broker()` 훅을 추가하고 기존 `sync_position()`은 하위 호환 래퍼로 유지했다.
  - `start_trading`은 시작 전 활성 broker 기준으로 KIS 잔고 또는 Toss holdings를 읽어 `PositionTracker`/`OverseasPositionTracker`와 전략 내부 포지션 상태를 동기화한다.
  - Toss holdings는 KRW 종목은 국내 tracker, USD 종목은 해외 tracker에 분리 복원하고 decimal 수량은 포지션 보유 여부가 사라지지 않도록 양수면 최소 1주 단위로 전략 snapshot에 반영한다.
  - 현재 Toss 주문/체결 adapter 전까지 자동매매 실행은 계속 `BROKER_NOT_SUPPORTED`로 차단되며, 차단 전에 holdings 기반 전략 상태 복원만 수행한다.
- [x] 환율 조회 소스 정책 정리
  - 토스 `exchange-rate`와 기존 외부 환율/KIS 환율 중 우선순위와 fallback을 명시한다.
  - 활성 Toss 프로파일에서는 `GET /api/v1/exchange-rate?baseCurrency=USD&quoteCurrency=KRW`를 우선 사용한다.
  - Toss 환율 조회가 실패하면 기존 공개 환율 API(open.er-api.com)로 fallback하고, 공개 환율도 실패하면 마지막 캐시/기본값 1450원을 유지한다.
  - KIS 활성 프로파일은 별도 KIS 환율 endpoint가 연결되기 전까지 기존 공개 환율 캐시를 계속 사용한다.
  - `get_exchange_rate_status` IPC와 `/api/exchange-rate/status` REST는 `source`, `fallbackUsed`, Toss `validFrom`/`validUntil`을 내려주며 Dashboard 해외 보유주식 섹션에 출처 chip을 표시한다.

## P2 — 주문/체결/수수료 연동

- [x] 토스 주문 생성 구현
  - `clientOrderId`를 발급해 중복 주문과 `request-in-progress`를 추적한다.
  - 고액 주문 확인, 주문 가능 시간, 호가 유형 제한, 시장별 지원 여부를 공식 에러 코드 기준으로 처리한다.
  - `TossOpenApiClient::create_order()`와 `TossOrderCreateRequest::with_generated_client_order_id()`를 추가했다. 공식 idempotency key 제약(36자, 영숫자/`-`/`_`)은 client-side에서 선검증한다.
  - 실제 Trading/자동매매 경로 호출은 소액 검증 gate 전까지 계속 차단한다.
- [x] 정정/취소/주문 목록/주문 상세 조회 구현
  - pending order 상태를 토스 주문 상세 기준으로 갱신한다.
  - 이미 체결/취소/거부된 주문에 대한 409 계열 에러를 사용자 로그와 주문 이력에 분리 저장한다.
  - `list_orders`, `get_order`, `modify_order`, `cancel_order` client/adapter 메서드와 serde 테스트를 추가했다.
  - 주문 목록/상세는 `ORDER_HISTORY`, 생성/정정/취소는 `ORDER` rate group으로 분리한다.
- [x] buying-power/sellable-quantity/commissions를 주문 전 검증에 연결
  - 기존 잔고 부족 반복 주문 방지와 수수료 추정 로직을 토스 공식 수수료 조회로 보완한다.
  - `trading/preflight.rs`에 브로커 공통 주문 전 판정 함수를 추가하고, `check_toss_order_preflight` IPC와 `/api/toss-order-preflight` REST에서 Toss 현재가/종목 유의사항/`buying-power`/`sellable-quantity`/`commissions`를 모아 검증한다.
  - Trading 화면은 활성 Toss 프로파일에서 주문 버튼은 계속 차단하되, 수량 입력 시 주문 전 검증 패널에 주문금액, 필요 현금, 매수가능금액/매도가능수량, 수수료 추정, 차단 사유를 표시한다.
  - Toss 주문 생성 adapter는 아직 `orderAdapterSupported=false`로 내려가며, 실제 주문/자동매매 실행은 소액 검증 gate 전까지 계속 차단된다.
- [x] 체결 확인 루프를 broker별 adapter로 분리
  - KIS 주문번호 기반 조회와 토스 order detail/list 조회를 같은 `confirm_pending_fills_from_broker()` 흐름에서 사용한다.
  - `OrderRecord.provider` trace로 pending provider를 판정하고, KIS 국내/해외 체결 조회는 `confirm_kis_pending_fills()`로 분리했다.
  - Toss pending은 주문 상세/목록 adapter 연결 지점에서 명시적으로 skip 로그를 남긴다. 실제 자동매매 연결은 소액 검증 gate 전까지 계속 차단된다.

## P2 — 자동매매 안전장치 확장

- [ ] 레버리지 추세 보유 전략을 롱 전용 기본 모델로 단순화한다.
  - 현재 구현은 `LeveragedTrendHoldEntry`에 롱 레버리지, 선택 숏 레버리지, 기초/유사기초 ETF를 한 세트로 두고, 상승 추세는 롱, 하락 추세는 숏 진입이 가능하다.
  - 재검토 방향: 숏 레버리지는 기초지수의 단순 반대편으로 움직인다고 보기 어렵고 괴리율·추적오차·수급·스프레드가 별도 리스크가 되므로, 기본 자동매매는 단일 롱 레버리지 진입/청산으로 단순화한다.
  - 1단계: 전략 파라미터에 `mode` 후보를 추가한다. 기본값은 `long_only`; 기존 저장값 호환을 위해 숏 ETF 필드는 유지하되 기본 진입 로직에서는 사용하지 않는다.
  - 2단계: 진입 조건은 기초/유사기초 ETF의 상승 추세 확인으로만 둔다. 예: 현재가 > EMA short, EMA short > EMA long, RSI/ADX 기준 통과, 최근 양봉 수 확인, 갭/블랙아웃/진입 시간 필터 유지.
  - 3단계: 청산 조건은 레버리지 자체 고점 대비 trailing stop, 기초 ETF EMA short 하향 이탈, RSI 약화, 장마감 청산, 필요 시 기초-레버리지 괴리/스프레드/거래량 이상 감지로 확장한다.
  - 4단계: 역방향/숏 자동 진입은 `inverse_experimental` 같은 별도 모드와 feature gate 뒤에 둔다. 다시 열 경우 롱과 동시 보유 금지, 독립 손절, 괴리율/추적오차/유동성 필터, backtest/소액 검증을 선행한다.
  - 5단계: Strategy UI는 숏 ETF를 필수가 아닌 고급 옵션으로 접고, 기본 설명을 “롱 레버리지 상승 추세 진입 + 상승여력 훼손 시 청산”으로 바꾼다.
  - 6단계: `trading::strategy::leveraged_trend_hold` 단위 테스트를 추가해 롱 진입, 추세 훼손 청산, 숏 필드가 있어도 `long_only`에서는 숏 매수가 발생하지 않음을 검증한다.

- [x] `TradeGuard`와 `RiskManager`에 broker/account scope 추가
  - 전략/종목/방향/날짜뿐 아니라 broker/account 단위 주문 횟수와 손실 제한을 분리 집계한다.
  - `BrokerScope`가 공통 broker 도메인 타입으로 추가되었고, 자동매매 시작 시 `OrderManager` 실행 scope가 활성 broker/account 스냅샷으로 고정된다.
  - `TradeGuard` 쿨다운/손절 재진입 차단과 `RiskManager` 일일 주문 제한/연속 손실 차단은 broker/account scope별로 격리된다.
- [x] 반대 미체결 주문 차단 구현
  - 토스 공식 `opposite-pending-order-exists` 에러와 로컬 pending 상태를 모두 고려한다.
  - `OrderManager`는 주문 제출 전 실행 `BrokerScope` + symbol 기준 pending을 scan하고, 같은 방향 미체결과 반대 방향 미체결을 모두 차단하되 로그 사유를 분리한다.
  - 반대 방향 pending은 “기존 매수/매도 미체결 주문 존재 — 요청 매도/매수 차단” 형식으로 주문번호와 함께 남긴다.
  - Toss 주문 adapter 구현 시 provider의 `opposite-pending-order-exists` 응답은 같은 pending conflict 계열로 매핑한다.
- [x] rate limit-aware scheduler 도입
  - 토스 rate limit 그룹별 TPS와 응답 헤더 기반 backoff를 공통 API client 계층에 반영한다.
  - KIS 실전/모의 rate limit도 같은 throttler로 점진 이전한다.
  - `broker/rate_limit.rs`의 `RateLimitScheduler`가 group별 최소 간격과 `Retry-After`/`X-RateLimit-Reset` pause를 처리한다.
  - Toss OpenAPI client는 auth/account/market group으로 분류해 `X-RateLimit-*` 응답 헤더를 반영하고, KIS 국내/해외 주문 제출은 `kis:order` group으로 먼저 이전했다.

## P3 — UI/UX와 문서 정리

- [x] broker 선택 UI 추가
  - Settings, Dashboard, Trading, Strategy, History에서 현재 broker/profile/account가 명확히 보이게 한다.
  - Settings와 Sidebar에는 활성 broker/account와 자동매매 실행 broker/account 스냅샷을 표시한다.
  - Dashboard, Trading, Strategy, History 제목 영역에 공통 `BrokerScopeIndicator`를 추가해 활성 broker/profile/account와 KIS 모의/실전 모드를 일관되게 표시한다.
- [x] Strategy 설정에 broker/account scope 추가
  - 같은 전략이 KIS와 토스 계좌를 동시에 대상으로 삼지 않도록 저장 구조를 명확히 한다.
  - `StrategyConfig`에 `broker_id`/`broker_account_id`를 추가하고, 프로파일 전환·전략 저장 시 현재 활성 broker/account scope로 자동 stamp한다.
  - 저장 전략이 없는 프로파일로 전환하면 기존 프로파일 전략을 reset해 종목/활성 상태가 계좌 사이에 잔류하지 않게 했다.
  - Strategy 화면 각 전략 카드에 저장된 broker/account scope chip을 표시하고, 활성 scope와 다르면 warning 색상으로 보여준다.
- [x] History/Log에 provider 원본 요청 추적 정보 추가
  - 토스 `requestId`, KIS TR-ID/odno 등 문의와 디버깅에 필요한 값을 안전하게 표시한다.
  - `OrderRecord`/`TradeRecord`에 `provider`, `provider_order_id`, `provider_request_id`, `provider_tr_id`를 추가하고 기존 JSON은 `serde(default)`로 호환한다.
  - History 체결 테이블과 Log 메시지 trace chip은 공통 `ProviderTraceChips`를 재사용한다.
- [x] `docs/project-map.md`, `docs/ipc-commands.md`, `docs/coding-guide.md` 갱신
  - 새 broker adapter, IPC, 설정, 검증 절차를 문서화한다.
  - 기존 IPC 확장 필드와 broker scope/trace 표시 패턴을 함께 반영했다.

## P4 — 검증

- [x] 공식 OpenAPI JSON 기반 contract test 추가
  - 최소한 endpoint 존재, 주요 schema 필드, error envelope, rate limit header 처리 코드를 검증한다.
- [x] broker adapter 단위 테스트 추가
  - KIS mock, Toss mock 응답을 각각 공통 도메인 타입으로 변환하는 테스트를 둔다.
  - Toss accounts/preflight deserialization, holdings → `BrokerHolding` 매핑, KIS `BalanceItem` → `BrokerHolding` 매핑, adapter trait 기본 unsupported 동작 테스트가 있다.
- [x] 토스증권 sandbox/실계좌 소액 검증 체크리스트 작성
  - 실제 주문 전 read-only 조회 검증, 주문 가능 금액 조회, 매도가능수량 조회, 수수료 조회를 분리한다.
  - 실거래 주문은 별도 명시 승인과 소액 테스트 절차를 요구한다.
  - `docs/toss-readonly-small-order-checklist.md`에 read-only 진단, 주문 전 검증 API, 명시 승인 gate, 자동매매 unlock 기준을 분리했다.
- [x] 전체 검증 명령 유지
  - `cd src-tauri; cargo check`
  - `npx tsc --noEmit`
  - `npm run check:fsd`
  - 현재 추가 검증: `cargo test --manifest-path src-tauri\Cargo.toml broker:: --lib`, `cargo test --manifest-path src-tauri\Cargo.toml trading::risk --lib`, `cargo test --manifest-path src-tauri\Cargo.toml trading::guard --lib`, `npm run verify:toss-openapi`, `npm run build:web`, Playwright `/settings` 콘솔 스모크.

---

## 최근 점검 메모

- 토스증권 공식 비브라우저 문서는 `https://developers.tossinvest.com/llms.txt`에서 시작한다.
- 공식 OpenAPI JSON은 `https://openapi.tossinvest.com/openapi-docs/latest/openapi.json`이며, 구현 시점에 매번 최신 version과 paths를 확인한다.
- 토스증권 API는 KIS의 TR-ID 방식이 아니라 OAuth2 + REST endpoint + account header 중심으로 설계해야 한다.
- 기존 KIS 전용 파일명과 타입명은 한 번에 모두 바꾸지 말고, adapter 경계부터 만든 뒤 UI와 저장 구조를 단계적으로 이전한다.
- 주문/체결 trace는 provider 원본 문의에 필요한 최소 식별자만 저장한다. KIS는 TR-ID/odno, Toss는 requestId/order id를 각각 `provider_*` 필드에 매핑한다.
- rate limit은 `RateLimitScheduler` group key로 관리한다. Toss는 auth/account/market, KIS는 주문 제출부터 `kis:order`로 점진 이전한다.
- Toss 주문 API client surface는 준비됐지만 UI/자동매매 연결은 소액 검증 gate와 체결 확인 adapter가 준비되기 전까지 열지 않는다.

*마지막 업데이트: 2026-07-04T14:39:28+09:00*
