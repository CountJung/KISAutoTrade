# Toss Open API 조사 노트

> Source of truth: `https://openapi.tossinvest.com/openapi-docs/latest/openapi.json`

마지막 확인: 2026-07-07

## 공식 스펙 스냅샷

| 항목 | 값 |
|------|----|
| OpenAPI title | `토스증권 Open API` |
| version | `1.2.2` |
| base URL | `https://openapi.tossinvest.com` |
| paths | 27 |
| 인증 | OAuth2 Client Credentials Grant |

## 구현 전 검증

```powershell
npm run verify:toss-openapi
```

검증 스크립트는 공식 OpenAPI JSON을 내려받아 `info.title`, `info.version`, base URL, endpoint inventory, `X-Tossinvest-Account` 헤더 참조, rate-limit 헤더 존재 여부를 확인한다. 스펙이 바뀌면 코드 생성·수동 adapter 작업 전에 이 문서를 먼저 갱신한다.

실제 주문 또는 자동매매 연결 전에는 `docs/toss-readonly-small-order-checklist.md`의 read-only/소액 검증 절차를 먼저 통과한다. Dashboard 소액매매 검증은 자동매매 unlock으로 해석하지 않는다. 공식 스펙 v1.1.5 기준 접수 주문 정정은 `POST /api/v1/orders/{orderId}/modify`로 제공되며, 앱은 수동 주문창에서 `GET /api/v1/orders?status=OPEN` 결과를 표시하고 접수 주문에 한해 정정 요청을 보낸다.

비공식 참고 구현으로 `JungHoonGhae/tossinvest-cli`를 볼 수 있다. 이 저장소는 공식 OpenAPI 범위와 WTS 기반 비공식 기능을 함께 다루므로, 공식 스펙에 없는 기능 후보를 조사할 때만 보조 자료로 사용한다. 공식 OpenAPI로 가능한 기능은 항상 공식 JSON을 source of truth로 삼고, WTS/web session/internal API 기반 기능은 사용자에게 위험을 명시적으로 알린 뒤 별도 요청이 있을 때만 검토한다.

## Endpoint Inventory

| 그룹 | Method | Path | 메모 |
|------|--------|------|------|
| Auth | POST | `/oauth2/token` | `application/x-www-form-urlencoded`, refresh token 없음 |
| Market Data | GET | `/api/v1/orderbook` | 호가 |
| Market Data | GET | `/api/v1/prices` | 현재가, 최대 200개 symbols |
| Market Data | GET | `/api/v1/trades` | 최근 체결 |
| Market Data | GET | `/api/v1/price-limits` | 상하한가 |
| Market Data | GET | `/api/v1/candles` | 캔들, 별도 rate-limit group |
| Stock Info | GET | `/api/v1/stocks` | 종목 기본 정보 |
| Stock Info | GET | `/api/v1/stocks/{symbol}/warnings` | 투자경고/거래 제한 후보 |
| Market Info | GET | `/api/v1/exchange-rate` | KRW/USD 참고 환율 |
| Market Info | GET | `/api/v1/market-calendar/KR` | 국내 시장 캘린더 |
| Market Info | GET | `/api/v1/market-calendar/US` | 미국 시장 캘린더 |
| Market Data | GET | `/api/v1/rankings` | 랭킹 |
| Market Indicator | GET | `/api/v1/market-indicators/prices` | 시장 지표 현재가 |
| Market Indicator | GET | `/api/v1/market-indicators/{symbol}/candles` | 시장 지표 캔들 |
| Market Indicator | GET | `/api/v1/market-indicators/{symbol}/investor-trading` | 시장 지표 투자자 매매 |
| Account | GET | `/api/v1/accounts` | accountSeq 조회 진입점 |
| Asset | GET | `/api/v1/holdings` | `X-Tossinvest-Account` 필요 |
| Order | GET, POST | `/api/v1/orders` | 주문 목록/생성, `clientOrderId` 지원 |
| Order History | GET | `/api/v1/orders/{orderId}` | 주문 상세 |
| Order | POST | `/api/v1/orders/{orderId}/modify` | 정정 |
| Order | POST | `/api/v1/orders/{orderId}/cancel` | 취소 |
| Conditional Order | POST, GET | `/api/v1/conditional-orders` | 조건 주문 |
| Conditional Order | GET, DELETE | `/api/v1/conditional-orders/{conditionalOrderId}` | 조건 주문 상세/삭제 |
| Conditional Order | POST | `/api/v1/conditional-orders/{conditionalOrderId}/modify` | 조건 주문 수정 |
| Order Info | GET | `/api/v1/buying-power` | 주문 전 매수 가능 금액 |
| Order Info | GET | `/api/v1/sellable-quantity` | 주문 전 매도 가능 수량 |
| Order Info | GET | `/api/v1/commissions` | 시장별 수수료 |

## 구현 메모

- 토큰 엔드포인트는 BFF 공통 envelope이 아니라 OAuth2 응답 형식을 따른다.
- 공식 스펙 v1.1.5의 `/oauth2/token` 설명 기준 refresh token은 없고, 응답 `expires_in` 예시는 `86400`초다. 만료 시 같은 endpoint로 재발급하되, client당 유효한 access token은 1개이며 재발급 시 이전 token은 즉시 무효화된다.
- 앱은 Toss token을 `base_url + client_id` 단위 프로세스 공유 캐시에 저장한다. 새 `TossBrokerAdapter`를 만들더라도 같은 client는 기존 token을 재사용하고, 만료 5분 전 또는 401 응답에서만 1회 재발급한다. 요청별 adapter 생성이 요청별 토큰 발급으로 이어지면 병렬 호출이 서로의 token을 무효화할 수 있으므로 금지한다.
- 계좌·자산·주문·주문 정보 API는 `Authorization: Bearer {access_token}` 외에 `X-Tossinvest-Account` 헤더가 필요하다.
- `accountSeq`는 임의 순번 입력값이 아니라 `GET /api/v1/accounts` 응답에서 받은 계좌 식별자다. Settings는 KIS/Toss 계좌 프로파일 섹션을 분리하고, Toss 섹션의 Add/Edit 다이얼로그에서 `list_toss_accounts`/`list_toss_profile_accounts`로 계좌 목록을 조회해 마스킹된 드롭다운에서 `accountSeq`를 선택하게 한다.
- 공식 태그 설명 기준 WebSocket은 아직 지원 대상이 아니며 REST 중심으로 설계한다.
- 공통 성공 응답은 `ApiResponse` + `result`, 실패 응답은 `ErrorResponse { error: ApiError }` envelope를 기준으로 처리한다.
- 429 응답은 `Retry-After`, `X-RateLimit-*` 헤더를 읽어 broker 공통 throttler로 넘긴다.
- 주문 생성은 `clientOrderId`를 발급해 중복 주문과 `request-in-progress`류 응답을 추적한다.
- 미국 시장 캘린더는 공식 스펙 v1.2.2 기준 `dayMarket`, `preMarket`, `regularMarket`, `afterMarket` 4개 세션을 제공한다. 다만 주문 생성 request에는 별도 세션 선택 필드가 없고 `timeInForce`만 `DAY`/`CLS`를 허용한다. Trading 수동 주문창은 Toss 미국 종목에서 `자동`/`데이`/`프리`/`정규`/`애프터` 세션 선택을 제공하고, 명시 세션 선택 시 현재 시간이 해당 세션 안인지 local gate로 검증한 뒤 기존처럼 `timeInForce=DAY` 주문을 제출한다. Strategy 자동매매 화면도 Toss 미국 대상 전략에 같은 세션 선택을 제공하며 `params.toss_us_session`에 저장한다. 데몬은 Toss 실행 scope에서 전략별 세션 gate를 적용하고, 레버리지 전략 내부의 진입/마감 전 청산 시간 계산도 같은 세션 정책을 사용한다. `auto`는 US 4개 세션 중 하나라도 열려 있으면 틱 처리와 주문 제출을 허용한다. 저장값이 없으면 `auto`를 기본값으로 사용한다.

## 현재 구현 상태

- `src-tauri/src/broker/toss/` 하위에 read-only `TossOpenApiClient`와 `TossBrokerAdapter`가 있다. 공개 surface는 `mod.rs`에서 re-export하고, 구현은 `adapter.rs`, `client.rs`, `http.rs`, `error.rs`, `support.rs`, `types.rs`, `orders.rs`, `tests.rs`로 나눈다.
- 구현된 범위: `POST /oauth2/token`, `GET /api/v1/accounts`, `GET /api/v1/holdings`, `GET /api/v1/prices`, `GET /api/v1/orderbook`, `GET /api/v1/trades`, `GET /api/v1/price-limits`, `GET /api/v1/candles`, `GET /api/v1/stocks`, `GET /api/v1/stocks/{symbol}/warnings`, `GET /api/v1/market-calendar/KR`, `GET /api/v1/market-calendar/US`.
- access token은 `base_url + client_id` 단위 공유 캐시에서 재사용하고, 만료 5분 전 갱신 대상으로 본다. 401 응답 시에는 해당 요청이 사용한 token이 아직 캐시에 남아 있을 때만 캐시를 지우고 1회 재발급/재시도한다. 다른 병렬 요청이 이미 token을 갱신했다면 그 token을 재사용한다.
- holdings 응답은 `BrokerHolding`으로 매핑한다. `marketCountry`는 `KR`/`US`, `currency`는 `KRW`/`USD`만 공통 타입으로 변환한다. Dashboard 표시와 자동매매 시작 전 전략 포지션 복원에 사용하되, 자동매매 주문 수량 산정에는 재사용하지 않는다.
- prices 응답은 `BrokerPriceQuote`로, candles 응답은 `BrokerCandle`로 매핑한다. `prices`는 최대 200개 symbols, `trades`는 count 1~50, `candles`는 interval `1m`/`1d`와 count 1~200 범위를 client에서 선검증한다. 공식 스펙 v1.2.2 기준 `prices` symbols는 영문 대/소문자, 숫자, `.`, `-`를 허용하고 응답 symbol은 canonical casing으로 올 수 있으므로 adapter/자동매매는 응답 symbol을 대소문자 무시로 매칭한다.
- stocks 응답은 `TossStockInfo`, warnings 응답은 `TossStockWarning`으로 보존한다. 공식 스펙이 unknown warning code 허용을 요구하므로 `warningType`은 enum이 아니라 문자열로 유지한다.
- market-calendar 응답은 KR의 `today.integrated.regularMarket`과 US의 `today.regularMarket`을 `MarketCalendarOverride`로 변환해 장 시간 판단에 사용한다. 공식 US calendar의 `dayMarket`/`preMarket`/`afterMarket`도 함께 보존해 Toss 미국 수동 주문과 자동매매 세션 gate에 사용한다. 공식 calendar가 있으면 우선 사용하고, 없거나 조회 실패하면 기존 KST 하드코딩 fallback을 유지한다.
- exchange-rate 응답은 `baseCurrency`, `quoteCurrency`, 문자열 decimal `rate`, `midRate`, `basisPoint`, `rateChangeType`, `validFrom`, `validUntil`을 보존한다. 앱 정책은 활성 Toss 프로파일에서 `USD`→`KRW` Toss 환율을 우선 사용하고, 실패하면 기존 공개 환율 API(open.er-api.com), 그마저 실패하면 마지막 캐시를 유지한다.
- orderbook, trades, price-limits 원본 응답은 토스 문자열 decimal 정밀도를 보존하는 read-only 타입으로 유지한다.
- 실패 응답은 `ErrorResponse { error }` envelope와 `X-Request-Id`, `Retry-After` 헤더를 함께 에러 메시지에 보존한다.
- GET market-data 전송 단계에서 실패하면 1회 짧게 재시도한다. 그래도 실패하면 `OpenAPI 요청 실패`만 남기지 말고 path와 transport error chain을 함께 남겨 DNS/TLS/timeout 같은 원인을 구분할 수 있게 한다.
- `list_toss_accounts`, `list_toss_profile_accounts` IPC와 `/api/toss-accounts`, `/api/profiles/:id/toss-accounts` 웹 REST에서 Settings 저장 전 `accountSeq` 후보를 조회한다. 응답은 `accountSeq`, 마스킹된 계좌번호, 계좌 타입 label만 포함한다.
- `check_toss_profile_connection` IPC와 `/api/profiles/:id/toss-diagnostic` 웹 REST에서 OpenAPI spec, token 발급, accounts 조회, holdings 조회, `buying-power`, `sellable-quantity`, `commissions`를 단계별로 진단한다.
- `get_broker_holdings` IPC와 `/api/broker-holdings` 웹 REST는 활성 프로파일 기준 holdings를 `BrokerHoldingView[]`로 내려준다. Dashboard와 Trading은 활성 broker가 Toss일 때 KIS 국내/해외 잔고 조회를 실행하지 않고 이 view로 Toss 보유종목, 평가금액, 미실현손익, accountSeq를 표시한다.
- `get_toss_market_snapshot` IPC와 `/api/toss-market-snapshot/:symbol` 웹 REST는 활성 Toss 프로파일 기준 현재가, 호가, 최근 체결 10건, 상하한가를 `TossMarketSnapshotView`로 내려준다. Trading 화면은 활성 broker가 Toss일 때 이 snapshot과 Toss chart를 표시하고 KIS 가격/차트 호출이 섞이지 않게 한다.
- `get_toss_stock_safety` IPC와 `/api/toss-stock-safety/:symbol` 웹 REST는 활성 Toss 프로파일 기준 종목 기본 정보와 매수 유의사항을 `TossStockSafetyView`로 내려준다. `buyBlocked`/`buyBlockReason`은 상장 상태와 blocking warning을 주문 전 검증 후보로 표현한다.
- `check_toss_order_preflight` IPC와 `/api/toss-order-preflight` 웹 REST는 활성 Toss 프로파일 기준 현재가 snapshot, 종목 유의사항, `buying-power`, `sellable-quantity`, `commissions`를 모아 `TossOrderPreflightView`로 내려준다. `liquidityOk`/`safetyOk`와 `live_trading_consent`가 모두 통과하면 `orderAdapterSupported=true`, `canSubmit=true`가 되어 Trading 수동 주문 버튼을 열 수 있다. Dashboard는 이 preflight를 검색 종목 1주 시장가 매수 조건으로 재사용한다.
- `submit_toss_small_buy_verification` IPC와 `/api/toss-small-buy-verification` 웹 REST는 Dashboard 전용 소액 실주문 gate다. 활성 Toss 프로파일, `live_trading_consent`, 최종 확인 checkbox, 화면 `accountSeq` 일치, 사용자가 입력한 최대 허용 주문금액, 직전 preflight 재실행, 같은 symbol 미체결 주문 scan을 모두 통과해야 검색 종목 1주 `MARKET` `BUY` 주문을 제출한다. 시장가 주문은 공식 스펙대로 `quantity="1"`만 보내고 `price`/`orderAmount`는 보내지 않는다.
- Dashboard 소액 실주문은 `TossOrderCreateRequest::with_generated_client_order_id()`로 `clientOrderId`를 만들고, `POST /api/v1/orders` 응답의 `orderId`를 받은 뒤 `GET /api/v1/orders/{orderId}`를 짧게 polling한다. 주문 접수 결과는 `OrderStore`에 provider `toss`, `orderId`, `clientOrderId` trace로 저장하고, 즉시 체결 또는 부분체결이 확인되면 `TradeStore`에도 provider trace와 함께 저장한다.
- Trading 수동 주문은 활성 Toss 프로파일의 `live_trading_consent`, 직전 preflight, 로컬 pending scan, provider open-order scan을 통과한 뒤 기존 `place_order` IPC에서 `POST /api/v1/orders`로 제출한다. 접수된 주문은 `OrderManager` pending으로 편입되어 이후 주문번호 기반 체결 확인 루프가 `GET /api/v1/orders/{orderId}`로 체결을 반영한다.
- Trading 수동 주문창은 `list_toss_open_orders` IPC와 `/api/toss-open-orders` 웹 REST로 활성 Toss 프로파일의 `status=OPEN` 주문 목록을 표시한다. 현재 검색 종목 주문을 상단에 정렬하되, 같은 계좌의 다른 접수 주문도 함께 보여 주문 충돌을 확인할 수 있게 한다.
- 접수 주문 정정은 `modify_toss_order` IPC와 `/api/toss-order-modify` 웹 REST가 담당한다. 공식 `POST /api/v1/orders/{orderId}/modify` 스키마에 맞춰 `orderType`, 선택 `quantity`, 선택 `price`, `confirmHighValueOrder`를 전달하고, 성공 후 주문 상세를 재조회해 로컬 pending 주문의 수량/가격/type snapshot을 갱신한다.
- 주문 정정 request는 시장별 제약을 반드시 분기한다. 공식 스펙 v1.1.5 기준 KR 주식 정정은 `quantity`가 필수이고 양의 정수만 허용되며, US 주식 정정은 가격 변경만 지원하므로 `quantity`를 보내면 `400 us-modify-quantity-not-supported`가 반환된다. 같은 수량이라도 US 정정 요청에는 `quantity` 필드를 직렬화하지 않는다.
- Toss 정정 성공 응답의 `orderId`는 원 주문번호가 아니라 새로 발급된 주문 식별자다. 앱은 정정 성공 후 새 `orderId` 상세를 조회하고 로컬 pending key와 provider trace를 새 주문번호로 갱신해 이후 체결 확인이 계속 이어지게 한다.
- 자동매매 주문 제출 전 로컬 pending scan은 같은 scope/symbol의 같은 방향 중복 주문과 반대 방향 미체결 주문을 모두 차단한다. Toss 자동매매 주문도 실행 scope의 `accountSeq`와 일치하는 profile credential을 찾아 `TossOrderCreateRequest`로 제출하고 provider의 `opposite-pending-order-exists` 오류는 같은 pending conflict 계열로 저장/표시한다.
- `get_toss_market_calendar` IPC와 `/api/toss-market-calendar` 웹 REST는 활성 Toss 프로파일 기준 KR/US 정규장 세션과 현재 개장 여부를 `TossMarketCalendarView`로 내려준다. 자동매매 데몬의 시장 폐장 사전 체크는 Toss 활성 프로파일 calendar override를 받을 수 있다.
- `get_toss_chart_data` IPC와 `/api/toss-chart/:symbol` 웹 REST는 활성 Toss 프로파일 기준 `1d`/`1m` candles를 기존 `ChartCandle[]`로 내려준다. Trading 화면은 `StockChart source="toss"`로 lightweight-charts를 재사용한다. Strategy 화면의 `preview_leveraged_trend_hold` IPC와 `/api/strategy/leveraged-trend-hold/preview` 웹 REST는 활성 Toss 프로파일의 `1m` candles를 주문 없이 리플레이해 현재 레버리지 전략 파라미터 기준 매수/청산 신호와 차트 표시용 candles를 반환한다.
- `get_exchange_rate_status` IPC와 `/api/exchange-rate/status` 웹 REST는 환율 source/fallback/유효시간을 `ExchangeRateView`로 내려준다. 기존 `get_exchange_rate`와 `/api/exchange-rate`는 숫자 캐시 호환 경로로 유지한다.
- Settings 프로파일 카드의 `연결 진단` 버튼은 토스 프로파일에만 표시한다. 진단 결과는 `steps[]`, `issues[]`, OpenAPI version, accounts/holdings count, KRW/USD buying power, commissions count로 요약한다. Add 다이얼로그는 열린 섹션의 broker로 고정하고, Edit 다이얼로그는 저장된 `broker_id`를 바꾸지 않는다.
- 자동매매 주문 실행 경로는 Toss 프로파일에서도 활성화된다. `start_trading()`은 Toss holdings 기반 `BrokerPositionSnapshot`으로 전략 내부 포지션 상태를 복원한 뒤, 활성 Toss 프로파일 설정과 `live_trading_consent`를 확인하고 실행 scope를 `BrokerScope { brokerId: Toss, accountSeq }`로 고정한다. 실행 scope가 Toss이면 자동매매 데몬은 KIS 해외 현재가로 폴백하지 않고 Toss `/api/v1/prices`를 사용한다. 전략 히스토리 초기화도 Toss 실행 scope에서는 KIS chart API가 아니라 Toss `/api/v1/candles`를 사용하며, `1d` candles는 일봉 지표에, `1m` candles OHLC는 레버리지 전략 장중 상태와 반동 관측 버퍼에 사용한다. 이후 실시간 현재가 polling은 같은 분의 마지막 장중 캔들을 갱신하고 분이 바뀔 때 새 캔들을 추가한다. Dashboard는 자동매매 시작 버튼을 활성화하고 검색 종목 1주 시장가 소액매매 검증 패널은 별도 최종 점검용으로 유지한다. Strategy/자동매매 화면에는 소액매매 검증 UI를 두지 않는다.

> 마지막 업데이트: 2026-07-08T03:20:00+09:00
