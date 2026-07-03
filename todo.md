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

- [ ] 토스 주문 생성 구현
  - `clientOrderId`를 발급해 중복 주문과 `request-in-progress`를 추적한다.
  - 고액 주문 확인, 주문 가능 시간, 호가 유형 제한, 시장별 지원 여부를 공식 에러 코드 기준으로 처리한다.
- [ ] 정정/취소/주문 목록/주문 상세 조회 구현
  - pending order 상태를 토스 주문 상세 기준으로 갱신한다.
  - 이미 체결/취소/거부된 주문에 대한 409 계열 에러를 사용자 로그와 주문 이력에 분리 저장한다.
- [x] buying-power/sellable-quantity/commissions를 주문 전 검증에 연결
  - 기존 잔고 부족 반복 주문 방지와 수수료 추정 로직을 토스 공식 수수료 조회로 보완한다.
  - `trading/preflight.rs`에 브로커 공통 주문 전 판정 함수를 추가하고, `check_toss_order_preflight` IPC와 `/api/toss-order-preflight` REST에서 Toss 현재가/종목 유의사항/`buying-power`/`sellable-quantity`/`commissions`를 모아 검증한다.
  - Trading 화면은 활성 Toss 프로파일에서 주문 버튼은 계속 차단하되, 수량 입력 시 주문 전 검증 패널에 주문금액, 필요 현금, 매수가능금액/매도가능수량, 수수료 추정, 차단 사유를 표시한다.
  - Toss 주문 생성 adapter는 아직 `orderAdapterSupported=false`로 내려가며, 실제 주문/자동매매 실행은 소액 검증 gate 전까지 계속 차단된다.
- [ ] 체결 확인 루프를 broker별 adapter로 분리
  - KIS 주문번호 기반 조회와 토스 order detail/list 조회를 같은 `confirm_pending_fills_from_broker()` 흐름에서 사용한다.

## P2 — 자동매매 안전장치 확장

- [x] `TradeGuard`와 `RiskManager`에 broker/account scope 추가
  - 전략/종목/방향/날짜뿐 아니라 broker/account 단위 주문 횟수와 손실 제한을 분리 집계한다.
  - `BrokerScope`가 공통 broker 도메인 타입으로 추가되었고, 자동매매 시작 시 `OrderManager` 실행 scope가 활성 broker/account 스냅샷으로 고정된다.
  - `TradeGuard` 쿨다운/손절 재진입 차단과 `RiskManager` 일일 주문 제한/연속 손실 차단은 broker/account scope별로 격리된다.
- [ ] 반대 미체결 주문 차단 구현
  - 토스 공식 `opposite-pending-order-exists` 에러와 로컬 pending 상태를 모두 고려한다.
- [ ] rate limit-aware scheduler 도입
  - 토스 rate limit 그룹별 TPS와 응답 헤더 기반 backoff를 공통 API client 계층에 반영한다.
  - KIS 실전/모의 rate limit도 같은 throttler로 점진 이전한다.

## P3 — UI/UX와 문서 정리

- [ ] broker 선택 UI 추가
  - Settings, Dashboard, Trading, Strategy, History에서 현재 broker/profile/account가 명확히 보이게 한다.
  - 현재 Settings와 Sidebar에는 활성 broker/account와 자동매매 실행 broker/account 스냅샷을 표시한다. 전 화면 선택 UI는 별도 진행한다.
- [ ] Strategy 설정에 broker/account scope 추가
  - 같은 전략이 KIS와 토스 계좌를 동시에 대상으로 삼지 않도록 저장 구조를 명확히 한다.
- [ ] History/Log에 provider 원본 요청 추적 정보 추가
  - 토스 `requestId`, KIS TR-ID/odno 등 문의와 디버깅에 필요한 값을 안전하게 표시한다.
- [ ] `docs/project-map.md`, `docs/ipc-commands.md`, `docs/coding-guide.md` 갱신
  - 새 broker adapter, IPC, 설정, 검증 절차를 문서화한다.

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

*마지막 업데이트: 2026-07-03T15:25:46*
