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

### Unofficial Reference

공식 OpenAPI에 없는 기능이나 Toss 웹/WTS 동작을 조사할 때는 아래 비공식 구현을 참고할 수 있다.

| 용도 | 경로 |
|------|------|
| 비공식 CLI 구현 | `https://github.com/JungHoonGhae/tossinvest-cli` |

- 이 저장소는 공식 제품이 아니며, README도 WTS 내부 API 사용이 토스증권 이용약관 위반 또는 예고 없는 변경 위험이 있음을 명시한다.
- 공식 OpenAPI로 구현 가능한 기능은 항상 공식 OpenAPI JSON과 토스 공식 문서를 source of truth로 삼는다. `tossinvest-cli`는 endpoint 추정, UX/명령 흐름, dry-run preview, session/routing 개념, 누락 기능 후보를 찾는 참고 자료로만 사용한다.
- WTS/web session, cookie, QR/browser login, 내부 API, 공식 문서에 없는 주문·소수점·실시간 push 기능을 이 앱에 연결하려면 사용자에게 비공식 경로 사용 위험을 먼저 알리고, 별도 명시 요청이 있을 때만 설계한다.
- 비공식 구현에서 발견한 필드·에러·제한은 그대로 신뢰하지 말고 공식 OpenAPI/실계좌 read-only 진단/소액 검증으로 재확인한 뒤 `docs/toss-openapi.md`와 이 스킬에 기록한다.

작업 시작 시 아래 명령으로 최신 스펙을 확인한다.

```powershell
npm run verify:toss-openapi
```

2026-10-29 공식 공개 스펙 확인 기준:

| 항목 | 값 |
|------|----|
| title | `토스증권 Open API` |
| version | `1.2.28` |
| base URL | `https://openapi.tossinvest.com` |
| paths | 39 |

## Authentication

- OAuth2 Client Credentials Grant를 사용한다.
- `POST /oauth2/token` 요청은 `application/x-www-form-urlencoded`로 보낸다.
- refresh token은 제공되지 않는다. 공식 스펙 v1.1.5의 `expires_in` 예시는 `86400`초이며, 만료 또는 401/`expired-token`이면 1회 재발급 후 재시도한다.
- client 당 유효한 access token은 1개다. 재발급 시 이전 토큰이 즉시 무효화되므로 토큰 매니저는 `base_url + client_id` 단위 공유 캐시를 사용하고 중복 발급을 피한다. 새 `TossBrokerAdapter`를 만들 때마다 독립 token cache를 만들면 화면 병렬 쿼리끼리 서로 token을 무효화하므로 금지한다.
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
| Market Info | `GET /api/v1/exchange-rate`, `GET /api/v1/market-calendar/KR`, `GET /api/v1/market-calendar/US`, rankings, market-indicators |
| Account/Asset | `GET /api/v1/accounts`, `GET /api/v1/holdings` |
| Order | `GET/POST /api/v1/orders`, order detail, modify, cancel, conditional-orders |
| Order Info | `GET /api/v1/buying-power`, `sellable-quantity`, `commissions` |

자세한 inventory는 `docs/toss-openapi.md`를 먼저 읽는다.
실제 주문 또는 자동매매 연결 전에는 `docs/toss-readonly-small-order-checklist.md`를 반드시 따른다.

## Rate Limit And Errors

- 429는 `Retry-After`, `X-RateLimit-*` 헤더를 읽어 broker 공통 throttler/backoff로 전달한다.
- 현재 구현은 `src-tauri/src/broker/rate_limit.rs`의 `RateLimitScheduler`를 사용한다. Toss group은 `toss:auth`, `toss:account`, `toss:market`으로 분리하고, 공식 응답 헤더의 남은 횟수/재시도 시각을 pause로 반영한다.
- Toss HTTP 응답 body는 `Content-Length` 사전 검사와 실제 chunk 누적 상한 검사를 모두 수행한다. `Content-Length`가 없거나 부정확해도 `TOSS_MAX_RESPONSE_BYTES`를 넘기기 전에 읽기를 중단해야 한다.
- 파싱 실패와 provider error 메시지는 전체 body를 `anyhow`/IPC/로그로 전달하지 말고 snippet만 포함한다. 보존해야 하는 값은 HTTP status, Toss error code/message snippet, provider request id, `X-Request-Id`, `Retry-After`다. GET market-data 전송 단계에서 실패하면 1회 짧게 재시도하고, 그래도 실패하면 `OpenAPI 요청 실패`만 남기지 말고 path와 transport error chain을 함께 남긴다.
- 일반 실패 응답은 `ErrorResponse { error: ApiError }` envelope를 기준으로 파싱한다.
- OAuth2 실패 응답은 `OAuth2ErrorResponse` 형태로 별도 파싱한다.
- 주문 전 검증은 official error code에 맞춘다. 특히 고액 주문 확인, 주문 가능 시간, 호가 유형, 시장별 지원 여부, 반대 미체결 주문 관련 오류는 로컬 guard와 함께 처리한다.


### 응답 content-coding과 캔들 파싱 진단 (2026-10-31)

- 현재 Toss 전용 client는 자동 압축 해제 feature가 없으므로 `Accept-Encoding: identity`를 명시한다. 헤더 생략은 서버가 임의 content-coding을 선택할 여지를 남긴다. 근거는 [RFC9110 §12.5.3](https://www.rfc-editor.org/rfc/rfc9110.html#section-12.5.3)과 [reqwest optional gzip](https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html#method.gzip)이며 다른 서비스의 HTTP/TLS·전역 feature를 임의 변경하지 않는다.
- 캔들 GET은 Content-Encoding 없음/단일identity만 읽고 미지원/복수·stacked coding을 명시 거부한다. 압축 wire를 UTF-8로 오인하거나 lossy decode·빈캔들 성공으로 바꾸지 않는다.15초/4MiB 사전·누적 제한을 보존한다.
- 캔들 body_limit/body_read/body_utf8/content_encoding/json 오류는 고정 endpoint·HTTP status·고정분류/숫자 위치만 남긴다. 원문 body/header·토큰·계좌·query/cursor·동적orderID/serde의응답값·rawsource는 새 진단에 넣지 않는다. 비시장/비성공HTTP의 기존 오류 정책까지 전부 redaction됐다고 주장하지 않는다.
- 캔들 정상DTO와401·GET send재시도·POST비재전송·typed 거절/불명 예산 경계를 fixture로 대조한다. 재현된 rawgzip 실패와 최종 미지원coding 조기 거절을 구분하고 실제 provider 응답 원인으로 확대하지 않는다. [수정·검증·한계](../../../docs/tasks/toss-simulation-response-parsing-20261031.md).

## Adapter Implementation Rules

- 공통 타입은 `src-tauri/src/broker/domain.rs`의 `BrokerId`, `BrokerAccountId`, `BrokerMarket`, `BrokerSymbol`, `BrokerMoney`, `BrokerQuantity`, `BrokerOrderId`, `BrokerClientOrderId`를 우선 사용한다.
- 토스 Decimal/string 금액과 수량은 Rust `f64`로 먼저 변환하지 말고 문자열을 보존한 뒤 필요한 곳에서 정밀하게 파싱한다.
- 토스 REST 구현은 KIS TR-ID나 CANO/ACNT_PRDT_CD 분리 로직을 재사용하지 않는다.
- `src-tauri/src/broker/toss/`의 read-only client는 token 발급, accounts 조회, holdings 조회를 담당한다. 공개 surface는 `mod.rs`에서 re-export하고, 구현은 `adapter.rs`, `client.rs`, `http.rs`, `error.rs`, `support.rs`, `types.rs`, `orders.rs`로 나눈다. Settings/IPC 진단에 연결할 때 이 client를 재사용한다.
- 같은 read-only client는 market data 후보인 `prices`, `orderbook`, `trades`, `price-limits`, `candles`도 담당한다. `prices`는 `BrokerPriceQuote`, `candles`는 `BrokerCandle`로 매핑하고, orderbook/trades/price-limits는 문자열 decimal 정밀도를 보존하는 Toss 원본 타입으로 유지한다.
- 같은 read-only client는 stock info 후보인 `stocks`, `stocks/{symbol}/warnings`도 담당한다. 공식 스펙이 unknown warning code 허용을 요구하므로 `warningType`은 enum으로 닫지 말고 문자열로 보존한다.
- 같은 read-only client는 market info 후보인 `market-calendar/KR`, `market-calendar/US`도 담당한다. KR의 `today.integrated.regularMarket`과 US의 `today.regularMarket`이 있으면 `MarketCalendarOverride`로 변환해 장 시간 판단에 우선 사용하고, 조회 실패 또는 미연결 상태에서는 기존 KST 하드코딩 fallback을 유지한다.
- 공식 스펙 v1.2.2 기준 US market-calendar는 `dayMarket`, `preMarket`, `regularMarket`, `afterMarket` 4개 세션을 제공한다. 주문 생성 request에는 별도 세션 선택 필드가 없고 `timeInForce`는 `DAY`/`CLS`만 허용된다. Trading 수동 주문창은 Toss 미국 종목에서 `자동`/`데이`/`프리`/`정규`/`애프터` 세션 선택을 제공하고, 명시 세션 선택 시 현재 시간이 해당 세션 안인지 local gate로 검증한 뒤 기존처럼 `DAY` 주문을 제출한다. Strategy 자동매매 화면도 Toss 미국 대상 전략에 같은 세션 선택을 제공하고 `params.toss_us_session`에 저장한다. 데몬은 Toss 실행 scope에서 전략별 세션 gate를 적용하고, 레버리지 전략 내부의 진입/마감 전 청산 시간 계산도 같은 세션 정책을 사용한다. `auto`는 US 4개 세션 중 하나라도 열려 있으면 틱 처리와 주문 제출을 허용하며, 저장값이 없으면 `auto`를 기본값으로 사용한다. 정규장 외 자동매매 주문에서는 `order-hours-closed`, `amount-order-outside-regular-hours`, `fractional-quantity-outside-regular-hours`, `order-type-not-allowed`를 provider 정책으로 보존한다.
- 같은 read-only client는 market info 후보인 `exchange-rate`도 담당한다. 공식 스펙상 `baseCurrency`, `quoteCurrency`, `rate`, `midRate`, `basisPoint`, `rateChangeType`, `validFrom`, `validUntil`을 반환하며 decimal 값은 문자열로 보존한다.
- USD/KRW 환율 정책은 활성 Toss 프로파일이면 Toss `GET /api/v1/exchange-rate?baseCurrency=USD&quoteCurrency=KRW`를 우선 사용하고, 실패하면 기존 공개 환율 API(open.er-api.com), 그마저 실패하면 마지막 캐시/기본값을 유지한다. KIS 활성 프로파일은 별도 KIS 환율 endpoint가 연결되기 전까지 기존 공개 환율 캐시를 사용한다.
- 공식 스펙 기준 `prices`/`stocks`는 최대 200개 symbols, `trades` count는 1~50, `candles` interval은 `1m`/`1d`, count는 1~200만 허용한다. `prices` symbols는 영문 대/소문자, 숫자, `.`, `-`를 허용하고 응답 symbol은 canonical casing으로 올 수 있으므로 자동매매/adapter의 응답 매칭은 대소문자를 무시한다. 네트워크 호출 전 client에서 범위를 선검증한다.
- Trading/Dashboard 등 UI에 Toss 시세를 붙일 때는 `get_toss_market_snapshot`처럼 현재가/호가/최근 체결/상하한가를 read-only view로 묶고, 활성 Toss 프로파일에서는 기존 KIS 가격/차트/주문 호출이 섞이지 않게 한다.
- Toss 종목 유의사항 UI는 `get_toss_stock_safety` IPC, `/api/toss-stock-safety/:symbol`, `useTossStockSafety()`로 연결한다. `buyBlocked`와 `buyBlockReason`은 상장 상태와 blocking warning을 주문 전 검증 후보로 표현하되, 실제 주문 adapter 연결 전까지 read-only 경고로만 사용한다.
- Toss 장 운영 UI는 `get_toss_market_calendar` IPC, `/api/toss-market-calendar`, `useTossMarketCalendar()`로 연결한다. Trading 화면에는 KR/US 정규장 개장 여부와 정규장 시간을 간단한 status chip으로 표시한다.
- 환율 source/fallback/유효시간 UI는 `get_exchange_rate_status` IPC, `/api/exchange-rate/status`, `useExchangeRateStatus()`로 연결한다. 기존 `get_exchange_rate`는 숫자 캐시 호환 경로로 유지한다.
- Toss candles UI는 `get_toss_chart_data` IPC, `/api/toss-chart/:symbol`, `useTossChartData()`를 통해 기존 `ChartCandle[]`와 `StockChart source="toss"` 경로로 연결한다. 일봉은 `YYYYMMDD`, 1분봉은 provider timestamp를 lightweight-charts `Time`으로 변환한다. Toss 실행 scope에서 자동매매를 시작할 때도 Toss `1d` candles로 일봉 지표를 초기화하고 Toss `1m` candles OHLC를 레버리지 전략 장중 상태에 주입한다. 레버리지 설정창 preview는 활성 profile/account를 검증하고 `1m`/`1d` raw 신호, replay 메타데이터와 비용·리스크 backtest를 반환한다. 이는 candle-close 모델이며 live 10초 tick과 provider latency는 별도 근사다.
- 같은 client는 주문 전 검증 후보인 `buying-power`, `sellable-quantity`, `commissions`도 문자열 정밀도를 유지해 조회한다. `check_toss_order_preflight` IPC, `/api/toss-order-preflight`, `useTossOrderPreflight()`는 현재가 snapshot과 종목 유의사항까지 함께 평가해 `liquidityOk`/`safetyOk`/차단 사유를 내려주고, `live_trading_consent`까지 통과하면 `orderAdapterSupported=true`, `canSubmit=true`를 반환한다.
- Dashboard의 Toss 소액매매 안내 패널은 동의 상태와 Trading 이동 경로만 표시하며 주문을 직접 제출하지 않는다. 일반 Trading 주문에는 호환 endpoint의 1주/최대금액 제한이 자동 적용되지 않는다. 별도 `submit_toss_small_buy_verification` IPC/REST gate에서 실거래 동의, 최종 확인, 최대 허용 주문금액, accountSeq 일치, 직전 preflight, 같은 symbol 미체결 scan을 통과한 경우에만 실제 1주 `MARKET` `BUY`를 제출한다. Trading은 일반 수동 주문 UI에서 preflight 통과 시 Toss `place_order` 분기로 주문을 제출한다. Strategy/자동매매 화면에는 소액매매 검증 UI를 두지 않는다.
- 주문 adapter를 연결할 때는 provider 호출 전 로컬 pending scan으로 같은 scope/symbol의 같은 방향 중복 주문과 반대 방향 미체결 주문을 먼저 차단한다. provider가 `opposite-pending-order-exists`를 반환하면 로컬 pending conflict와 같은 계열로 주문 이력/로그에 남긴다.
- 주문 API client surface는 `TossOpenApiClient::{create_order,list_orders,get_order,modify_order,cancel_order}`로 둔다. `TossOrderCreateRequest::with_generated_client_order_id()`는 공식 idempotency key 제약(36자 이하, 영숫자/`-`/`_`)을 만족하는 `clientOrderId`를 만든다. 호환용 소액매매 endpoint의 시장가 매수는 공식 스펙대로 `quantity="1"`만 보내고 `price`/`orderAmount`는 보내지 않는다.
- 수동 주문창의 Toss 접수 주문 목록은 `TossOrderListQuery::open()`으로 `GET /api/v1/orders?status=OPEN`을 조회해 표시한다. `list_toss_open_orders` IPC, `/api/toss-open-orders`, `useTossOpenOrders()` 경로를 함께 갱신하고, 현재 검색 종목 주문을 먼저 정렬하되 활성 `accountSeq`의 다른 접수 주문도 보여준다.
- 공식 스펙 v1.1.5 기준 접수 주문 정정은 `POST /api/v1/orders/{orderId}/modify`가 제공한다. `modify_toss_order` IPC, `/api/toss-order-modify`, `useModifyTossOrder()`로 연결하고, 성공 후 `get_order`를 다시 호출해 로컬 pending 주문의 `quantity`/`price`/`order_type` snapshot을 갱신한다.
- 주문 정정 request는 시장별 제약을 분기한다. KR 주식 정정은 `quantity` 필수, US 주식 정정은 가격 변경만 지원하며 `quantity` 제공 시 `400 us-modify-quantity-not-supported`가 반환된다. US 주문에서 기존 수량과 같은 값을 UI가 들고 있어도 request body에는 `quantity`를 보내지 않는다.
- 정정 성공 응답의 `orderId`는 원 주문번호와 다르다. 정정 후 새 `orderId` 상세를 조회하고 `OrderManager` pending key/provider trace를 새 주문번호로 갱신해야 주문번호 기반 체결 확인이 이어진다.
- 주문 생성 request는 `quantity` 또는 `orderAmount` 중 정확히 하나만 허용한다. 시장별 세부 제한은 provider error envelope를 보존해 처리한다.
- 자동매매 체결 확인 루프는 pending `OrderRecord.provider` trace로 provider를 판정한다. 호환용 소액매매 endpoint는 `create_order` 뒤 `get_order`를 짧게 polling해 `OrderStore`와, 즉시 체결/부분체결이면 `TradeStore`에 provider trace를 저장한다. Trading/자동매매 Toss pending은 `get_order` detail의 누적 체결수량과 평균체결가를 읽어 `OrderManager::on_fill()`로 반영한다.
- access token은 만료 5분 전 갱신 대상으로 보고, 401 응답 시 1회 재발급/재시도한다. 401 처리에서는 요청에 사용한 token이 아직 공유 캐시에 남아 있을 때만 캐시를 지우고, 다른 병렬 요청이 이미 갱신한 token이 있으면 그 token을 재사용한다.
- holdings를 공통 `BrokerHolding`으로 매핑할 때 `marketCountry`는 `KR`/`US`, `currency`는 `KRW`/`USD`만 허용한다. unknown enum은 조용히 기본값으로 바꾸지 않는다.
- holdings를 Dashboard/REST/IPC에 표시할 때는 원본 `raw`를 노출하지 않는 `BrokerHoldingView` 계열 view 타입을 만들고, `BrokerMoney`/`BrokerQuantity` 문자열 precision은 UI 표시 직전까지 보존한다.
- Dashboard와 Trading은 활성 broker가 Toss이면 KIS 국내/해외 잔고 쿼리를 비활성화하고 `get_broker_holdings` 결과로 보유종목, 평가금액, 미실현손익, accountSeq를 표시한다. KIS 전용 `get_balance` 오류를 Toss 화면에 노출하지 않는다.
- holdings는 `HoldingsUpdate::from_toss`에서 broker/account·시장/통화·중복·수치 범위를 검증해 계좌 tracker에 게시한다. KRW 원과 USD cents를 구분하고 소수 수량은 floor하며 양수 최소1로 올리지 않는다. 전략 보유는 별도 exact owner 원장에서 복원하며 전체 계좌 holdings를 여러 전략에 복사하지 않는다. 공통 계약은 Rust 스킬 §12를 따른다.
- read-only 진단 UI는 `check_toss_profile_connection` IPC와 `/api/profiles/:id/toss-diagnostic` 웹 REST를 통해 OpenAPI version 확인, token 발급, accounts 조회, holdings 조회, buying-power, commissions, 보유 종목 기반 sellable-quantity 순서로 구현한다.
- Settings Toss Add/Edit 다이얼로그는 `list_toss_accounts` 또는 `list_toss_profile_accounts`로 `/api/v1/accounts`를 먼저 호출하고, 계좌번호를 마스킹한 드롭다운에서 `accountSeq`를 선택하게 한다. 전체 `accountNo`는 UI 응답에 노출하지 않는다.
- Settings 프로파일 카드에서는 KIS 프로파일에 실전/모의 자동 감지 버튼을 유지하고, Toss 프로파일에는 `연결 진단` 버튼만 표시한다. KIS 자동감지(`detect_trading_type`, `/api/profiles/:id/detect`)는 KIS `/oauth2/tokenP`를 호출하므로 Toss Client ID/Secret을 넣으면 provider가 `앱키/appkey` 오류로 응답할 수 있다. 백엔드도 Toss 프로파일에 대해 이 경로를 거부하고, Toss 검증은 항상 `check_toss_profile_connection` 또는 accountSeq 조회 경로를 사용한다.
- Settings Add/Edit 다이얼로그에서 broker가 Toss이면 입력 라벨을 `Client ID`, `Client Secret`, `accountSeq`로 바꾼다. `accountSeq`는 숫자 문자열이어야 한다.
- Toss 실거래 동의 상태는 `AccountProfile.live_trading_consent`로 저장한다. 이 값은 호환용 소액 실주문 gate, Trading 수동 주문, 자동매매 시작 gate의 필수 조건이다.
- 실제 주문 생성은 Dashboard `submit_toss_small_buy_verification`, Trading `place_order` Toss 분기, 자동매매 `OrderManager::submit_signal_shared()` Toss 분기에 연결한다. 모든 경로는 provider 호출 전 local pending scan과 provider open-order/order detail 확인을 사용해 같은 scope/symbol의 충돌을 줄인다.
- 일반 수동 주문과 자동 주문은 `OrderManager`의 scoped order service를 공유한다. broker/account scope, 공통 리스크, pending 예약, 주문 스냅샷 영속화를 거친 뒤 adapter 내부에서만 KIS/Toss 요청 형식을 분기한다. 미체결 스냅샷에는 provider/client 주문 ID, 누적 체결수량, 마지막 provider 상태를 보존하고 재시작 시 `get_order` detail과 대조한다.
- 자동매매 실행 경로는 Toss 주문/체결 adapter가 구현되어 있으므로 `live_trading_consent`가 저장된 Toss 프로파일에서 허용한다. `start_trading()`은 Toss 계좌 snapshot 게시와 exact owner 전략 복원을 수행하고 실행 scope를 시작 시점 broker/account로 고정한다. 데몬은 실행 scope가 Toss이면 KIS 해외 현재가로 폴백하지 않고 Toss `/api/v1/prices`를 사용하며, 저장 ticker와 응답 symbol casing 차이로 현재가 조회가 실패하지 않도록 대소문자 무시 매칭을 유지한다. 전략 히스토리 초기화도 Toss 실행 scope에서는 KIS chart/overseas chart가 아니라 Toss `/api/v1/candles`를 사용해야 한다. KIS chart를 호출하면 Toss 키가 정상이어도 KIS tokenP에서 `유효하지 않은 AppKey`가 발생한다. Settings/Sidebar에는 활성 broker/account와 실행 중 broker/account 스냅샷을 표시한다.
- Toss 모듈 내부 DTO/validation/helper는 외부 API가 아니면 `pub(super)`로 열고, 앱 외부에서 필요한 타입과 client/adapter만 `mod.rs`에서 re-export한다.



2026-09-11 공식 스펙 검증: v1.2.15, 33 paths. 기존 27개에 `/api/v1/stocks/all` 및 `/api/v1/stocks/{symbol}/investor-trading`, `program-trades`, `short-selling`, `credit-trades`, `securities-lending` GET 경로가 추가되어 inventory 검증 기준을 갱신했다. 추가 경로의 앱 기능은 이 변경에 포함하지 않는다.

자동매매 예산: Toss HTTP 오류는 상태 코드를 보존하는 내부 타입으로 전달한다. 400/401/403/404/405/422만 확정 거절로 예약을 해제하며 408/409/429/5xx, transport/응답 파싱 실패는 보류한다. 자동 예산 주문은 정정 차단, 자동매수 지정가, 수동/자동 소유수량 분리를 유지한다.


2026-10-18 공개 스펙 v1.2.24는 기존33개에 섹터 GET5개(`/api/v1/sectors`, `/sectors/rankings`, `/sectors/{sectorId}`, 그 하위 `/stocks`, `/etfs`)를 추가했다. 정확한 inventory 검증을38개로 갱신하며 새 섹터 앱 기능은 추가하지 않는다. [공식 OpenAPI](https://openapi.tossinvest.com/openapi-docs/latest/openapi.json).

캔들 page adapter는 inclusive `nextBefore`를 그대로 보존하며 요청한 count(최대200)보다 큰 응답은 거부한다. 다음 커서가 없거나 이전 커서보다 과거로 전진하지 않으면 명시 종료한다. 공통 `get_candle_page`/웹 GET는 동일 scope·기준 시각을 고정한다. `1m`은 봉 종료 시각을 RFC3339 offset 포함 actual instant로 처리하고, `1d` 거래일은 시장 현지 날짜로 변환한다. provider 응답의 timezone `+`를 URL `%2B`로 인코딩하며 임의1초 차감으로 inclusive 계약을 바꾸지 않는다. UI 수집8회/500봉·사전252봉과 정확한 부족 사유, live52주 갱신은 `docs/tasks/live-completed-daily-history-20261018.md`를 따른다. live는실행profile/account고정,동시2개·8회/512봉·30~900초backoff이며프로파일누락을KIS조회로폴백하지않는다.

> 마지막 업데이트: 2026-10-18T11:57:42+09:00


### 체결·수수료의 네이티브 단위 확인 (2026-10-18)

[공식 OpenAPI v1.2.24](https://openapi.tossinvest.com/openapi-docs/latest/openapi.json)의 `OrderExecution.filledAmount`는 총 체결 금액(native currency), `averageFilledPrice`는 평균 가격이다. 정수 평균가에 수량을 다시 곱해 누적 금액을 복원하면 나눗셈 잔액이 사라질 수 있다. nullable 응답·미확인 자료와 실제 0을 구분한다.

`Commission.commissionRate`는 소수 비율이다. `0.00015`는 `0.015%`이며 1,000,000원 주문의 수수료는150원이다. `OrderPreflightConstraints.commission_rate_ratio`는 추가로100을 나누지 않으며, wire `commissionRate` 문자열은 원본 비율을 유지하고 화면에서만 퍼센트로 변환한다. 비정상·음수 rate는 사전검증을 차단한다. 수수료 정책 미확보의 기존 명시적 경고는 유지한다.

❌ 기존 오류: 비율을 percent로 해석해 `/100`하거나 원본 뒤에 `%`를 붙여 수수료/표시를100배 낮추는 것.

✅ 현 계약: 공식 비율을 주문 필요 현금 계산에 사용하고 표시 시에만 퍼센트로 변환한다. 경계 cash10001/필요10002와 실제 패널 표시를 검증한다. 실제 broker 주문 검증을 뜻하지 않는다.

> 마지막 업데이트: 2026-10-18T14:30:00+09:00


앱의 누적금액 정책: `filledAmount` 명시값이 우선이다. 누락에만 실제 `averageFilledPrice` 원문×확정 정수 수량을 총액에서 한번 반올림한다. 명시0/오염·시장 통화 불일치를 주문가나 신호가로 대체하지 않는다. 소액 검증 즉시 fill은 명시금액이 있어야 하며 원 주문 ID·symbol·BUY·통화·정수1주를 검증한다. 누락 금액은 즉시 적용 없이 pending을 유지해 정규 대조로 넘긴다.

> 마지막 업데이트: 2026-10-18T14:40:06+09:00


## 2026-10-29 REST inventory 변경

공식 공개 v1.2.28은 기존38경로를 유지하고 `GET /api/v1/rankings/investor-trading`1개를 추가해39경로/42 operations다. count는 선택1~100/default100, OAuth2 Bearer·RANKING group이며 계좌 헤더는 없다. 검증기의 정확한 목록을 갱신하고 인증/오류/rate-limit 검사를 유지한다. 앱의 신규 API 호출·거래 기능을 추가하지 않는다. REST 명세의 AsyncAPI3 별도 안내와 현재 앱의 Toss REST 구현을 구분하고 실제 WebSocket 지원/연결 성공으로 확대하지 않는다. [정본](../../../docs/toss-openapi.md).
