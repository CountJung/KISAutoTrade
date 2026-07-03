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
- 공식 태그 설명 기준 WebSocket은 아직 지원 대상이 아니며 REST 중심으로 설계한다.
- 공통 성공 응답은 `ApiResponse` + `result`, 실패 응답은 `ErrorResponse { error: ApiError }` envelope를 기준으로 처리한다.
- 429 응답은 `Retry-After`, `X-RateLimit-*` 헤더를 읽어 broker 공통 throttler로 넘긴다.
- 주문 생성은 `clientOrderId`를 발급해 중복 주문과 `request-in-progress`류 응답을 추적한다.

## 현재 구현 상태

- `src-tauri/src/broker/toss.rs`에 read-only `TossOpenApiClient`와 `TossBrokerAdapter`가 있다.
- 구현된 범위: `POST /oauth2/token`, `GET /api/v1/accounts`, `GET /api/v1/holdings`.
- access token은 만료 5분 전 갱신 대상으로 보고, 401 응답 시 캐시를 지운 뒤 1회 재발급/재시도한다.
- holdings 응답은 `BrokerHolding`으로 매핑한다. `marketCountry`는 `KR`/`US`, `currency`는 `KRW`/`USD`만 공통 타입으로 변환한다.
- 실패 응답은 `ErrorResponse { error }` envelope와 `X-Request-Id`, `Retry-After` 헤더를 함께 에러 메시지에 보존한다.
- `check_toss_profile_connection` IPC와 `/api/profiles/:id/toss-diagnostic` 웹 REST에서 OpenAPI spec, token 발급, accounts 조회, holdings 조회, `buying-power`, `sellable-quantity`, `commissions`를 단계별로 진단한다.
- Settings 프로파일 카드의 `연결 진단` 버튼은 토스 프로파일에만 표시한다. 진단 결과는 `steps[]`, `issues[]`, OpenAPI version, accounts/holdings count, KRW/USD buying power, commissions count로 요약한다.
- 아직 Dashboard/Position UI에는 토스 holdings를 표시하지 않았다. 실제 주문과 자동매매 경로는 계속 `BROKER_NOT_SUPPORTED`로 차단한다.
