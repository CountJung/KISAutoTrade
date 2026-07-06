---
name: toss-api
description: "토스증권 Open API 전용 스킬. Toss OpenAPI JSON, OAuth2 Client Credentials, X-Tossinvest-Account 헤더, REST endpoint inventory, rate-limit/error envelope, broker adapter 구현, 계좌/자산/주문/시세 연동을 다룰 때 사용한다."
---

# Toss Open API Skill

## Source Of Truth

토스증권 API 동작은 추측하지 말고 공식 OpenAPI JSON을 기준으로 확인한다.

| 용도 | 공식 경로 |
|------|----------|
| 브라우저 문서 | `https://developers.tossinvest.com/docs` |
| AI/비브라우저 진입점 | `https://developers.tossinvest.com/llms.txt` |
| OpenAPI JSON | `https://openapi.tossinvest.com/openapi-docs/latest/openapi.json` |
| OpenAPI Markdown | `https://openapi.tossinvest.com/openapi-docs/latest/api-reference/README.md` |

작업 시작 시 아래 명령으로 최신 스펙을 확인한다.

```powershell
npm run verify:toss-openapi
```

2026-07-06 확인 기준:

| 항목 | 값 |
|------|----|
| title | `토스증권 Open API` |
| version | `1.1.5` |
| base URL | `https://openapi.tossinvest.com` |
| paths | 20 |

## Authentication

- OAuth2 Client Credentials Grant를 사용한다.
- `POST /oauth2/token` 요청은 `application/x-www-form-urlencoded`로 보낸다.
- refresh token은 제공되지 않는다. 만료 또는 401/`expired-token`이면 1회 재발급 후 재시도한다.
- client 당 유효한 access token은 1개다. 재발급 시 이전 토큰이 무효화될 수 있으므로 토큰 매니저는 중복 발급을 피한다.
- 토큰 endpoint 응답은 공통 `ApiResponse` envelope가 아니라 OAuth2 표준 응답이다.

## Account Header

- `/api/v1/accounts`에서 받은 `accountSeq`를 계좌 API의 `X-Tossinvest-Account` 헤더 값으로 사용한다.
- `accountSeq`는 사용자가 임의로 고르는 단순 1/2 값이 아니라 `/api/v1/accounts` 응답에서 선택해야 하는 계좌 식별자다. 예시가 `1`이어도 저장 전 계좌 목록 조회 결과와 맞는지 확인한다.
- holdings, orders, order detail, buying-power, sellable-quantity, commissions는 access token과 `X-Tossinvest-Account`가 모두 필요하다.
- Settings 저장 구조에서는 KIS 계좌번호와 토스 `accountSeq`를 같은 문자열로 섞지 말고 `BrokerAccountId`와 broker id로 분리한다.
- 기존 KIS 프로파일에는 `broker_id = "kis"` 기본값을 적용하고, 토스 프로파일은 별도 `broker_id = "toss"` scope로 저장한다.
- Settings UI는 KIS/Toss 계좌 프로파일 섹션을 분리한다. Add 다이얼로그는 열린 섹션의 broker로 고정하고, Edit 다이얼로그는 기존 프로파일의 `broker_id`를 변경하지 않는다.

## Endpoint Groups

| 그룹 | 경로 |
|------|------|
| Auth | `POST /oauth2/token` |
| Market Data | `GET /api/v1/orderbook`, `prices`, `trades`, `price-limits`, `candles` |
| Stock Info | `GET /api/v1/stocks`, `GET /api/v1/stocks/{symbol}/warnings` |
| Market Info | `GET /api/v1/exchange-rate`, `GET /api/v1/market-calendar/KR`, `GET /api/v1/market-calendar/US` |
| Account/Asset | `GET /api/v1/accounts`, `GET /api/v1/holdings` |
| Order | `GET/POST /api/v1/orders`, order detail, modify, cancel |
| Order Info | `GET /api/v1/buying-power`, `sellable-quantity`, `commissions` |

자세한 inventory는 `docs/toss-openapi.md`를 먼저 읽는다.
실제 주문 또는 자동매매 연결 전에는 `docs/toss-readonly-small-order-checklist.md`를 반드시 따른다.

## Rate Limit And Errors

- 429는 `Retry-After`, `X-RateLimit-*` 헤더를 읽어 broker 공통 throttler/backoff로 전달한다.
- 현재 구현은 `src-tauri/src/broker/rate_limit.rs`의 `RateLimitScheduler`를 사용한다. Toss group은 `toss:auth`, `toss:account`, `toss:market`으로 분리하고, 공식 응답 헤더의 남은 횟수/재시도 시각을 pause로 반영한다.
- Toss HTTP 응답 body는 `Content-Length` 사전 검사와 실제 chunk 누적 상한 검사를 모두 수행한다. `Content-Length`가 없거나 부정확해도 `TOSS_MAX_RESPONSE_BYTES`를 넘기기 전에 읽기를 중단해야 한다.
- 파싱 실패와 provider error 메시지는 전체 body를 `anyhow`/IPC/로그로 전달하지 말고 snippet만 포함한다. 보존해야 하는 값은 HTTP status, Toss error code/message snippet, provider request id, `X-Request-Id`, `Retry-After`다.
- 일반 실패 응답은 `ErrorResponse { error: ApiError }` envelope를 기준으로 파싱한다.
- OAuth2 실패 응답은 `OAuth2ErrorResponse` 형태로 별도 파싱한다.
- 주문 전 검증은 official error code에 맞춘다. 특히 고액 주문 확인, 주문 가능 시간, 호가 유형, 시장별 지원 여부, 반대 미체결 주문 관련 오류는 로컬 guard와 함께 처리한다.

## Adapter Implementation Rules

- 공통 타입은 `src-tauri/src/broker/domain.rs`의 `BrokerId`, `BrokerAccountId`, `BrokerMarket`, `BrokerSymbol`, `BrokerMoney`, `BrokerQuantity`, `BrokerOrderId`, `BrokerClientOrderId`를 우선 사용한다.
- 토스 Decimal/string 금액과 수량은 Rust `f64`로 먼저 변환하지 말고 문자열을 보존한 뒤 필요한 곳에서 정밀하게 파싱한다.
- 토스 REST 구현은 KIS TR-ID나 CANO/ACNT_PRDT_CD 분리 로직을 재사용하지 않는다.
- `src-tauri/src/broker/toss/`의 read-only client는 token 발급, accounts 조회, holdings 조회를 담당한다. 공개 surface는 `mod.rs`에서 re-export하고, 구현은 `adapter.rs`, `client.rs`, `http.rs`, `error.rs`, `support.rs`, `types.rs`, `orders.rs`로 나눈다. Settings/IPC 진단에 연결할 때 이 client를 재사용한다.
- 같은 read-only client는 market data 후보인 `prices`, `orderbook`, `trades`, `price-limits`, `candles`도 담당한다. `prices`는 `BrokerPriceQuote`, `candles`는 `BrokerCandle`로 매핑하고, orderbook/trades/price-limits는 문자열 decimal 정밀도를 보존하는 Toss 원본 타입으로 유지한다.
- 같은 read-only client는 stock info 후보인 `stocks`, `stocks/{symbol}/warnings`도 담당한다. 공식 스펙이 unknown warning code 허용을 요구하므로 `warningType`은 enum으로 닫지 말고 문자열로 보존한다.
- 같은 read-only client는 market info 후보인 `market-calendar/KR`, `market-calendar/US`도 담당한다. KR의 `today.integrated.regularMarket`과 US의 `today.regularMarket`이 있으면 `MarketCalendarOverride`로 변환해 장 시간 판단에 우선 사용하고, 조회 실패 또는 미연결 상태에서는 기존 KST 하드코딩 fallback을 유지한다.
- 같은 read-only client는 market info 후보인 `exchange-rate`도 담당한다. 공식 스펙상 `baseCurrency`, `quoteCurrency`, `rate`, `midRate`, `basisPoint`, `rateChangeType`, `validFrom`, `validUntil`을 반환하며 decimal 값은 문자열로 보존한다.
- USD/KRW 환율 정책은 활성 Toss 프로파일이면 Toss `GET /api/v1/exchange-rate?baseCurrency=USD&quoteCurrency=KRW`를 우선 사용하고, 실패하면 기존 공개 환율 API(open.er-api.com), 그마저 실패하면 마지막 캐시/기본값을 유지한다. KIS 활성 프로파일은 별도 KIS 환율 endpoint가 연결되기 전까지 기존 공개 환율 캐시를 사용한다.
- 공식 스펙 기준 `prices`/`stocks`는 최대 200개 symbols, `trades` count는 1~50, `candles` interval은 `1m`/`1d`, count는 1~200만 허용한다. 네트워크 호출 전 client에서 범위를 선검증한다.
- Trading/Dashboard 등 UI에 Toss 시세를 붙일 때는 `get_toss_market_snapshot`처럼 현재가/호가/최근 체결/상하한가를 read-only view로 묶고, 활성 Toss 프로파일에서는 기존 KIS 가격/차트/주문 호출이 섞이지 않게 한다.
- Toss 종목 유의사항 UI는 `get_toss_stock_safety` IPC, `/api/toss-stock-safety/:symbol`, `useTossStockSafety()`로 연결한다. `buyBlocked`와 `buyBlockReason`은 상장 상태와 blocking warning을 주문 전 검증 후보로 표현하되, 실제 주문 adapter 연결 전까지 read-only 경고로만 사용한다.
- Toss 장 운영 UI는 `get_toss_market_calendar` IPC, `/api/toss-market-calendar`, `useTossMarketCalendar()`로 연결한다. Trading 화면에는 KR/US 정규장 개장 여부와 정규장 시간을 간단한 status chip으로 표시한다.
- 환율 source/fallback/유효시간 UI는 `get_exchange_rate_status` IPC, `/api/exchange-rate/status`, `useExchangeRateStatus()`로 연결한다. 기존 `get_exchange_rate`는 숫자 캐시 호환 경로로 유지한다.
- Toss candles UI는 `get_toss_chart_data` IPC, `/api/toss-chart/:symbol`, `useTossChartData()`를 통해 기존 `ChartCandle[]`와 `StockChart source="toss"` 경로로 연결한다. 일봉은 `YYYYMMDD`, 1분봉은 provider timestamp를 lightweight-charts `Time`으로 변환한다.
- 같은 client는 주문 전 검증 후보인 `buying-power`, `sellable-quantity`, `commissions`도 문자열 정밀도를 유지해 조회한다. `check_toss_order_preflight` IPC, `/api/toss-order-preflight`, `useTossOrderPreflight()`는 현재가 snapshot과 종목 유의사항까지 함께 평가해 `liquidityOk`/`safetyOk`/차단 사유를 내려주고, `live_trading_consent`까지 통과하면 `orderAdapterSupported=true`, `canSubmit=true`를 반환한다.
- Dashboard 화면의 `Toss 소액 수동매매 검증` UI는 활성 Toss `accountSeq`, 검색 종목 1주 시장가 매수 조건, `live_trading_consent`, 최종 확인 checkbox, 최대 허용금액을 표시한다. 별도 `submit_toss_small_buy_verification` IPC/REST gate에서 실거래 동의, 최종 확인, 최대 허용 주문금액, accountSeq 일치, 직전 preflight, 같은 symbol 미체결 scan을 통과한 경우에만 실제 1주 `MARKET` `BUY`를 제출한다. Trading은 일반 수동 주문 UI에서 preflight 통과 시 Toss `place_order` 분기로 주문을 제출한다. Strategy/자동매매 화면에는 소액매매 검증 UI를 두지 않는다.
- 주문 adapter를 연결할 때는 provider 호출 전 로컬 pending scan으로 같은 scope/symbol의 같은 방향 중복 주문과 반대 방향 미체결 주문을 먼저 차단한다. provider가 `opposite-pending-order-exists`를 반환하면 로컬 pending conflict와 같은 계열로 주문 이력/로그에 남긴다.
- 주문 API client surface는 `TossOpenApiClient::{create_order,list_orders,get_order,modify_order,cancel_order}`로 둔다. `TossOrderCreateRequest::with_generated_client_order_id()`는 공식 idempotency key 제약(36자 이하, 영숫자/`-`/`_`)을 만족하는 `clientOrderId`를 만든다. Dashboard 소액 시장가 매수는 공식 스펙대로 `quantity="1"`만 보내고 `price`/`orderAmount`는 보내지 않는다.
- 주문 생성 request는 `quantity` 또는 `orderAmount` 중 정확히 하나만 허용한다. 시장별 세부 제한은 provider error envelope를 보존해 처리한다.
- 자동매매 체결 확인 루프는 pending `OrderRecord.provider` trace로 provider를 판정한다. Dashboard 소액 주문은 `create_order` 뒤 `get_order`를 짧게 polling해 `OrderStore`와, 즉시 체결/부분체결이면 `TradeStore`에 provider trace를 저장한다. Trading/자동매매 Toss pending은 `get_order` detail의 누적 체결수량과 평균체결가를 읽어 `OrderManager::on_fill()`로 반영한다.
- access token은 만료 5분 전 갱신 대상으로 보고, 401 응답 시 캐시를 지운 뒤 1회 재발급/재시도한다.
- holdings를 공통 `BrokerHolding`으로 매핑할 때 `marketCountry`는 `KR`/`US`, `currency`는 `KRW`/`USD`만 허용한다. unknown enum은 조용히 기본값으로 바꾸지 않는다.
- holdings를 Dashboard/REST/IPC에 표시할 때는 원본 `raw`를 노출하지 않는 `BrokerHoldingView` 계열 view 타입을 만들고, `BrokerMoney`/`BrokerQuantity` 문자열 precision은 UI 표시 직전까지 보존한다.
- Dashboard와 Trading은 활성 broker가 Toss이면 KIS 국내/해외 잔고 쿼리를 비활성화하고 `get_broker_holdings` 결과로 보유종목, 평가금액, 미실현손익, accountSeq를 표시한다. KIS 전용 `get_balance` 오류를 Toss 화면에 노출하지 않는다.
- holdings는 자동매매 시작 전 전략 내부 포지션 복원에도 사용할 수 있다. `BrokerPositionSnapshot`은 `brokerId=Toss`, `market`, `symbol`, `quantity`, `avgPrice`를 들고, KRW 평균가는 원 단위, USD 평균가는 cents 단위로 전달한다. Toss decimal 수량은 in-position 복원 목적상 양수면 최소 1 단위로 반영하되, 실제 주문 수량으로 재사용하지 않는다.
- read-only 진단 UI는 `check_toss_profile_connection` IPC와 `/api/profiles/:id/toss-diagnostic` 웹 REST를 통해 OpenAPI version 확인, token 발급, accounts 조회, holdings 조회, buying-power, commissions, 보유 종목 기반 sellable-quantity 순서로 구현한다.
- Settings Toss Add/Edit 다이얼로그는 `list_toss_accounts` 또는 `list_toss_profile_accounts`로 `/api/v1/accounts`를 먼저 호출하고, 계좌번호를 마스킹한 드롭다운에서 `accountSeq`를 선택하게 한다. 전체 `accountNo`는 UI 응답에 노출하지 않는다.
- Settings 프로파일 카드에서는 KIS 프로파일에 실전/모의 자동 감지 버튼을 유지하고, Toss 프로파일에는 `연결 진단` 버튼만 표시한다.
- Settings Add/Edit 다이얼로그에서 broker가 Toss이면 입력 라벨을 `Client ID`, `Client Secret`, `accountSeq`로 바꾼다. `accountSeq`는 숫자 문자열이어야 한다.
- Toss 실거래 동의 상태는 `AccountProfile.live_trading_consent`로 저장한다. 이 값은 Dashboard 소액 실주문 gate, Trading 수동 주문, 자동매매 시작 gate의 필수 조건이다.
- 실제 주문 생성은 Dashboard `submit_toss_small_buy_verification`, Trading `place_order` Toss 분기, 자동매매 `OrderManager::submit_signal_shared()` Toss 분기에 연결한다. 모든 경로는 provider 호출 전 local pending scan과 provider open-order/order detail 확인을 사용해 같은 scope/symbol의 충돌을 줄인다.
- 자동매매 실행 경로는 Toss 주문/체결 adapter가 구현되어 있으므로 `live_trading_consent`가 저장된 Toss 프로파일에서 허용한다. `start_trading()`은 Toss holdings 기반 전략 포지션 복원을 수행하고 실행 scope를 시작 시점 broker/account로 고정한다. Settings/Sidebar에는 활성 broker/account와 실행 중 broker/account 스냅샷을 표시한다.
- Toss 모듈 내부 DTO/validation/helper는 외부 API가 아니면 `pub(super)`로 열고, 앱 외부에서 필요한 타입과 client/adapter만 `mod.rs`에서 re-export한다.

> 마지막 업데이트: 2026-07-06T22:10:00+09:00
