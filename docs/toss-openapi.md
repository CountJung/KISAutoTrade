# Toss Open API 조사 노트

> Source of truth: `https://openapi.tossinvest.com/openapi-docs/latest/openapi.json`

마지막 확인: 2026-07-03

## 공식 스펙 스냅샷

| 항목 | 값 |
|------|----|
| OpenAPI title | `토스증권 Open API` |
| version | `1.1.5` |
| base URL | `https://openapi.tossinvest.com` |
| paths | 20 |
| 인증 | OAuth2 Client Credentials Grant |

## 구현 전 검증

```powershell
npm run verify:toss-openapi
```

검증 스크립트는 공식 OpenAPI JSON을 내려받아 `info.title`, `info.version`, base URL, endpoint inventory, `X-Tossinvest-Account` 헤더 참조, rate-limit 헤더 존재 여부를 확인한다. 스펙이 바뀌면 코드 생성·수동 adapter 작업 전에 이 문서를 먼저 갱신한다.

실제 주문 또는 자동매매 연결 전에는 `docs/toss-readonly-small-order-checklist.md`의 read-only/소액 검증 gate를 먼저 통과한다.

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
| Account | GET | `/api/v1/accounts` | accountSeq 조회 진입점 |
| Asset | GET | `/api/v1/holdings` | `X-Tossinvest-Account` 필요 |
| Order | GET, POST | `/api/v1/orders` | 주문 목록/생성, `clientOrderId` 지원 |
| Order History | GET | `/api/v1/orders/{orderId}` | 주문 상세 |
| Order | POST | `/api/v1/orders/{orderId}/modify` | 정정 |
| Order | POST | `/api/v1/orders/{orderId}/cancel` | 취소 |
| Order Info | GET | `/api/v1/buying-power` | 주문 전 매수 가능 금액 |
| Order Info | GET | `/api/v1/sellable-quantity` | 주문 전 매도 가능 수량 |
| Order Info | GET | `/api/v1/commissions` | 시장별 수수료 |

## 구현 메모

- 토큰 엔드포인트는 BFF 공통 envelope이 아니라 OAuth2 응답 형식을 따른다.
- 계좌·자산·주문·주문 정보 API는 `Authorization: Bearer {access_token}` 외에 `X-Tossinvest-Account` 헤더가 필요하다.
- `accountSeq`는 임의 순번 입력값이 아니라 `GET /api/v1/accounts` 응답에서 받은 계좌 식별자다. Settings는 KIS/Toss 계좌 프로파일 섹션을 분리하고, Toss 섹션의 Add/Edit 다이얼로그에서 `list_toss_accounts`/`list_toss_profile_accounts`로 계좌 목록을 조회해 마스킹된 드롭다운에서 `accountSeq`를 선택하게 한다.
- 공식 태그 설명 기준 WebSocket은 아직 지원 대상이 아니며 REST 중심으로 설계한다.
- 공통 성공 응답은 `ApiResponse` + `result`, 실패 응답은 `ErrorResponse { error: ApiError }` envelope를 기준으로 처리한다.
- 429 응답은 `Retry-After`, `X-RateLimit-*` 헤더를 읽어 broker 공통 throttler로 넘긴다.
- 주문 생성은 `clientOrderId`를 발급해 중복 주문과 `request-in-progress`류 응답을 추적한다.

## 현재 구현 상태

- `src-tauri/src/broker/toss.rs`에 read-only `TossOpenApiClient`와 `TossBrokerAdapter`가 있다.
- 구현된 범위: `POST /oauth2/token`, `GET /api/v1/accounts`, `GET /api/v1/holdings`, `GET /api/v1/prices`, `GET /api/v1/orderbook`, `GET /api/v1/trades`, `GET /api/v1/price-limits`, `GET /api/v1/candles`, `GET /api/v1/stocks`, `GET /api/v1/stocks/{symbol}/warnings`, `GET /api/v1/market-calendar/KR`, `GET /api/v1/market-calendar/US`.
- access token은 만료 5분 전 갱신 대상으로 보고, 401 응답 시 캐시를 지운 뒤 1회 재발급/재시도한다.
- holdings 응답은 `BrokerHolding`으로 매핑한다. `marketCountry`는 `KR`/`US`, `currency`는 `KRW`/`USD`만 공통 타입으로 변환한다. Dashboard 표시와 자동매매 시작 전 전략 포지션 복원에 사용하되, 실제 주문 실행에는 연결하지 않는다.
- prices 응답은 `BrokerPriceQuote`로, candles 응답은 `BrokerCandle`로 매핑한다. `prices`는 최대 200개 symbols, `trades`는 count 1~50, `candles`는 interval `1m`/`1d`와 count 1~200 범위를 client에서 선검증한다.
- stocks 응답은 `TossStockInfo`, warnings 응답은 `TossStockWarning`으로 보존한다. 공식 스펙이 unknown warning code 허용을 요구하므로 `warningType`은 enum이 아니라 문자열로 유지한다.
- market-calendar 응답은 KR의 `today.integrated.regularMarket`과 US의 `today.regularMarket`을 `MarketCalendarOverride`로 변환해 장 시간 판단에 사용한다. 공식 calendar가 있으면 우선 사용하고, 없거나 조회 실패하면 기존 KST 하드코딩 fallback을 유지한다.
- exchange-rate 응답은 `baseCurrency`, `quoteCurrency`, 문자열 decimal `rate`, `midRate`, `basisPoint`, `rateChangeType`, `validFrom`, `validUntil`을 보존한다. 앱 정책은 활성 Toss 프로파일에서 `USD`→`KRW` Toss 환율을 우선 사용하고, 실패하면 기존 공개 환율 API(open.er-api.com), 그마저 실패하면 마지막 캐시를 유지한다.
- orderbook, trades, price-limits 원본 응답은 토스 문자열 decimal 정밀도를 보존하는 read-only 타입으로 유지한다.
- 실패 응답은 `ErrorResponse { error }` envelope와 `X-Request-Id`, `Retry-After` 헤더를 함께 에러 메시지에 보존한다.
- `list_toss_accounts`, `list_toss_profile_accounts` IPC와 `/api/toss-accounts`, `/api/profiles/:id/toss-accounts` 웹 REST에서 Settings 저장 전 `accountSeq` 후보를 조회한다. 응답은 `accountSeq`, 마스킹된 계좌번호, 계좌 타입 label만 포함한다.
- `check_toss_profile_connection` IPC와 `/api/profiles/:id/toss-diagnostic` 웹 REST에서 OpenAPI spec, token 발급, accounts 조회, holdings 조회, `buying-power`, `sellable-quantity`, `commissions`를 단계별로 진단한다.
- `get_broker_holdings` IPC와 `/api/broker-holdings` 웹 REST는 활성 프로파일 기준 holdings를 `BrokerHoldingView[]`로 내려준다. Dashboard와 Trading은 활성 broker가 Toss일 때 KIS 국내/해외 잔고 조회를 실행하지 않고 이 view로 Toss 보유종목, 평가금액, 미실현손익, accountSeq를 표시한다.
- `get_toss_market_snapshot` IPC와 `/api/toss-market-snapshot/:symbol` 웹 REST는 활성 Toss 프로파일 기준 현재가, 호가, 최근 체결 10건, 상하한가를 `TossMarketSnapshotView`로 내려준다. Trading 화면은 활성 broker가 Toss일 때 이 snapshot과 Toss chart를 표시하고 KIS 가격/차트/수동 주문 호출을 막는다.
- `get_toss_stock_safety` IPC와 `/api/toss-stock-safety/:symbol` 웹 REST는 활성 Toss 프로파일 기준 종목 기본 정보와 매수 유의사항을 `TossStockSafetyView`로 내려준다. `buyBlocked`/`buyBlockReason`은 상장 상태와 blocking warning을 주문 전 검증 후보로 표현한다.
- `check_toss_order_preflight` IPC와 `/api/toss-order-preflight` 웹 REST는 활성 Toss 프로파일 기준 현재가 snapshot, 종목 유의사항, `buying-power`, `sellable-quantity`, `commissions`를 모아 `TossOrderPreflightView`로 내려준다. `liquidityOk`/`safetyOk`는 read-only 검증 결과이고, `orderAdapterSupported=false`와 `canSubmit=false`를 유지해 실제 주문 제출은 차단한다.
- 자동매매 주문 제출 전 로컬 pending scan은 같은 scope/symbol의 같은 방향 중복 주문과 반대 방향 미체결 주문을 모두 차단한다. 향후 Toss 주문 adapter가 provider의 `opposite-pending-order-exists` 오류를 받으면 같은 pending conflict 계열로 저장/표시한다.
- `get_toss_market_calendar` IPC와 `/api/toss-market-calendar` 웹 REST는 활성 Toss 프로파일 기준 KR/US 정규장 세션과 현재 개장 여부를 `TossMarketCalendarView`로 내려준다. 자동매매 데몬의 시장 폐장 사전 체크는 Toss 활성 프로파일 calendar override를 받을 수 있다.
- `get_toss_chart_data` IPC와 `/api/toss-chart/:symbol` 웹 REST는 활성 Toss 프로파일 기준 `1d`/`1m` candles를 기존 `ChartCandle[]`로 내려준다. Trading 화면은 `StockChart source="toss"`로 lightweight-charts를 재사용한다.
- `get_exchange_rate_status` IPC와 `/api/exchange-rate/status` 웹 REST는 환율 source/fallback/유효시간을 `ExchangeRateView`로 내려준다. 기존 `get_exchange_rate`와 `/api/exchange-rate`는 숫자 캐시 호환 경로로 유지한다.
- Settings 프로파일 카드의 `연결 진단` 버튼은 토스 프로파일에만 표시한다. 진단 결과는 `steps[]`, `issues[]`, OpenAPI version, accounts/holdings count, KRW/USD buying power, commissions count로 요약한다. Add 다이얼로그는 열린 섹션의 broker로 고정하고, Edit 다이얼로그는 저장된 `broker_id`를 바꾸지 않는다.
- 실제 주문과 자동매매 주문 실행 경로는 계속 `BROKER_NOT_SUPPORTED`로 차단한다. `start_trading()`은 차단 전에 Toss holdings 기반 `BrokerPositionSnapshot`으로 전략 내부 포지션 상태를 복원할 수 있다. Dashboard는 Toss 활성 시 자동매매 시작 버튼을 비활성화하고, Strategy는 Toss read-only 자동매매 차단 안내를 표시한다.

> 마지막 업데이트: 2026-07-04T11:45:53+09:00
