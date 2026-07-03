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

2026-07-03 확인 기준:

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
- holdings, orders, order detail, buying-power, sellable-quantity, commissions는 access token과 `X-Tossinvest-Account`가 모두 필요하다.
- Settings 저장 구조에서는 KIS 계좌번호와 토스 `accountSeq`를 같은 문자열로 섞지 말고 `BrokerAccountId`와 broker id로 분리한다.
- 기존 KIS 프로파일에는 `broker_id = "kis"` 기본값을 적용하고, 토스 프로파일은 별도 `broker_id = "toss"` scope로 저장한다.

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
- 일반 실패 응답은 `ErrorResponse { error: ApiError }` envelope를 기준으로 파싱한다.
- OAuth2 실패 응답은 `OAuth2ErrorResponse` 형태로 별도 파싱한다.
- 주문 전 검증은 official error code에 맞춘다. 특히 고액 주문 확인, 주문 가능 시간, 호가 유형, 시장별 지원 여부, 반대 미체결 주문 관련 오류는 로컬 guard와 함께 처리한다.

## Adapter Implementation Rules

- 공통 타입은 `src-tauri/src/broker/domain.rs`의 `BrokerId`, `BrokerAccountId`, `BrokerMarket`, `BrokerSymbol`, `BrokerMoney`, `BrokerQuantity`, `BrokerOrderId`, `BrokerClientOrderId`를 우선 사용한다.
- 토스 Decimal/string 금액과 수량은 Rust `f64`로 먼저 변환하지 말고 문자열을 보존한 뒤 필요한 곳에서 정밀하게 파싱한다.
- 토스 REST 구현은 KIS TR-ID나 CANO/ACNT_PRDT_CD 분리 로직을 재사용하지 않는다.
- `src-tauri/src/broker/toss.rs`의 read-only client는 token 발급, accounts 조회, holdings 조회를 담당한다. Settings/IPC 진단에 연결할 때 이 client를 재사용한다.
- 같은 read-only client는 주문 전 검증 후보인 `buying-power`, `sellable-quantity`, `commissions`도 문자열 정밀도를 유지해 조회한다.
- access token은 만료 5분 전 갱신 대상으로 보고, 401 응답 시 캐시를 지운 뒤 1회 재발급/재시도한다.
- holdings를 공통 `BrokerHolding`으로 매핑할 때 `marketCountry`는 `KR`/`US`, `currency`는 `KRW`/`USD`만 허용한다. unknown enum은 조용히 기본값으로 바꾸지 않는다.
- read-only 진단 UI는 `check_toss_profile_connection` IPC와 `/api/profiles/:id/toss-diagnostic` 웹 REST를 통해 OpenAPI version 확인, token 발급, accounts 조회, holdings 조회, buying-power, commissions, 보유 종목 기반 sellable-quantity 순서로 구현한다.
- Settings 프로파일 카드에서는 KIS 프로파일에 실전/모의 자동 감지 버튼을 유지하고, Toss 프로파일에는 `연결 진단` 버튼만 표시한다.
- Settings Add/Edit 다이얼로그에서 broker가 Toss이면 입력 라벨을 `Client ID`, `Client Secret`, `accountSeq`로 바꾼다. `accountSeq`는 숫자 문자열이어야 한다.
- Toss 실거래 동의 상태는 `AccountProfile.live_trading_consent`로 저장한다. 이 값은 명시 승인 기록이며, 주문/자동매매 연결은 별도 소액 검증 gate와 adapter 구현이 끝나기 전까지 계속 차단한다.
- 실제 주문 생성은 별도 사용자 승인과 소액 검증 절차가 문서화되기 전까지 자동매매 경로에 연결하지 않는다.
- 주문 구현을 시작하더라도 `docs/toss-readonly-small-order-checklist.md`의 명시 승인 gate를 통과하기 전에는 Trading/Strategy/Dashboard/자동매매 흐름에서 호출 가능하게 만들지 않는다.
- 자동매매 실행 경로는 Toss 주문/체결 adapter가 구현되기 전까지 `BROKER_NOT_SUPPORTED`로 차단한다. Settings/Sidebar에는 활성 broker/account와 실행 중 broker/account 스냅샷을 표시한다.

> 마지막 업데이트: 2026-07-03T13:27:56
