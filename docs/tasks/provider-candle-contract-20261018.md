# 사전자료 페이지 구현을 위한 공개 계약 확인

2026-10-18 KST. P2-17과 live P2-05의 선행 조사다. 공개 문서만 조회했고 token/계좌/주문 요청은 하지 않았다. 공개 계약과 구현·fixture 검증, 실제 provider 검증을 구분한다. live 완료 연결은 [후속 구현](live-completed-daily-history-20261018.md)에서 검증했다.

## Toss

[공식 OpenAPI v1.2.24](https://openapi.tossinvest.com/openapi-docs/latest/openapi.json)에서 확인했다.

- `/api/v1/candles`는 최신순, 최대200봉, `MARKET_DATA_CHART` group이다.
- `before`는 ISO8601 inclusive 상한이다. 응답 `nextBefore`를 그대로 보내고 timezone `+`를 `%2B`로 인코딩한다. 겹침 제거와 cursor 무진전 종료가 필요하다.
- `1m` timestamp는 **봉 종료 시각**, `1d`는 현지 거래일 자정이다. 다른 자료원의 봉 시작 표기에 같은 정책을 적용하지 않는다.
- 공통 page adapter가 `nextBefore`를 보존하고 count보다 큰 응답을 거부한다. 기존 Vec 커맨드는 그대로 유지한다. generic UI와 후속 live52주 갱신은 새 page 경로를 사용한다.

## KIS

[국내 공식 샘플](https://github.com/koreainvestment/open-trading-api/blob/main/examples_llm/domestic_stock/inquire_daily_itemchartprice/inquire_daily_itemchartprice.py)은 `FHKST03010100`, 날짜1/2, 호출당 최대100건이다. 날짜를 후퇴시키는 bounded 페이지와 중복/미래/무진전 검증이 필요하다.

[해외 공식 샘플](https://github.com/koreainvestment/open-trading-api/blob/main/examples_llm/overseas_stock/dailyprice/dailyprice.py)과 [user 샘플](https://github.com/koreainvestment/open-trading-api/blob/main/examples_user/overseas_stock/overseas_stock_functions.py)은 `dailyprice`에 **HHDFS76240000**을 사용한다. `api/rest/chart.rs`의 차트 요청을 이 TR-ID로 수정했다. 해외 현재가 TR-ID는 바꾸지 않는다. API_URL·BYMD·GUBN·MODP를 생산 요청 빌더 fixture로 검증하며 새 페이지는 BYMD 날짜 anchor를 후퇴시킨다. 샘플의 tr_cont 재귀 방식은 자동 도입하지 않는다. 샘플의 재귀 방식만으로 중복 없는 자료 확보를 보장하지 않는다.

[공식 응답 컬럼 샘플](https://github.com/koreainvestment/open-trading-api/blob/main/examples_llm/overseas_stock/dailyprice/chk_dailyprice.py)의 날짜는 `xymd`다. 기존 `bass_dt` 파서와 이를 복제한 초기 fixture를 함께 수정해 실제 컬럼의 날짜·OHLCV가 보존되는지 확인한다.

## 구현 범위와 잔여

1. `api/rest.rs`가1,000행을 넘으면 차트/시세 모듈을 먼저 분리하고 공개 메서드 경로를 유지한다.
2. 공식 요청/TR-ID·cursor·시간 표기 회귀, 동일 scope·공유 rate-limit·오류/응답 상한을 보존한다.
3. 공통 page/cursor 계약과 IPC/웹/TS/UI를 연결한다. 합계500봉 replay 상한은 유지하며200/252봉 확보 실패 이유를 표시한다.
4. exchange 날짜와 완료 증거를 갖춘 live 일봉 refresh를 scope/symbol별 bounded cache·backoff로 연결하고, 자료 부족은 신규 진입만 막는다. sampled tick을 완료 고가로 합성하지 않는다.

공개 스펙 inventory는 섹터 GET5개 추가로33→38경로를 확인하고 검사·문서를 갱신해 통과했다. page/TR-ID/UI는 아래 fixture로 검증하며, live refresh는 후속 구현 기록을 따르며 실제 provider 응답·보관 범위 검증과 구분한다.


## 공통 페이지와 UI

- `broker/candles.rs`의 provider별 cursor를 공통 trait에 추가했다. IPC `get_candle_page`와 GET `/api/candles/page`는 같은 camelCase 입력·응답을 사용한다. 기존 chart Vec command와 다른 차트 소비자는 유지한다.
- cursor는 profile/broker/account/실전·모의·종목·시장·간격·거래소·count scope hash와 최초 `asOf`를 포함한다. 입력/커서 오류는 provider 실패와 분리한다. KIS는 공유 client 자격증명4필드 일치, Toss 프로파일 누락은 KIS 폴백 금지를 적용한다. 응답/cursor는 비밀키를 포함하지 않는다.
- 일봉은 Seoul/New York 당일, 주봉은 현재 주, 월봉은 현재 달을 보수적으로 제외한다. 이는 provider 확정 flag 검증과 다르다. Toss 1m은 종료시각이 asOf 이하인 봉만 사용하며 잘못된 OHLCV와 미래봉을 제외하고 gap을 합성하지 않는다.
- UI는 평가봉을 우선 보존하며 과거252봉을 별도로 수집한다. 요청 최대8회(거래소 실패 포함), 유지500봉, 페이지 최대200봉, cursor 반복·추가 자료 없음·provider 종료·상한·부분 오류를 표시한다. 빈 필터 결과라도 커서가 과거로 전진하면 다음 페이지를 조회한다.
- KIS 미국 거래소 fallback은 첫 provider 오류 때만 NAS→NYS→AMS로 시도한다. 성공한 페이지 이후 거래소를 바꾸지 않는다. scope/커서/설정 오류는 부분 성공으로 삼키지 않는다. 취소와 입력 변경 결과는 replay에 게시하지 않는다.

## 분봉 시간 경계

v9는 RFC3339 offset·소수초를 실제 순간으로 해석하고 UTC 고정 정밀도로 정규화해 중복·정렬·history overlap·hash·신호에 일관되게 사용한다. legacy compact12/14는 기존 차트와 같은 KST 입력이다. UI 차트도 RFC3339 offset을 우선 해석한다. 일봉 계산과 frozen 연구 파일을 재생성하지 않는다. actual instant와 시장 날짜를 구분해 replay 리스크·당일 재진입 규칙의 host timezone 의존을 제거한다. RFC3339 replay는 한국 Seoul/미국 New York 시장 날짜로 처리하고, 공통 guard의 지정시각 경계는 FixedOffset을 보존한다. 실운용 Local wrapper는 유지한다. 전용 LTH의 세션/DST/3개월 coverage가 검증됐다는 뜻은 아니다.

## 검증

- Rust check/all-targets, Clippy all-targets/all-features 경고0.
- 전체 Rust lib243개 + production LTH integration3개 + doc3개, 합계249개 통과.
- 페이지 순서·기간 제외·무효 OHLCV·cursor scope/유형/미래시각·camelCase Query/응답을 검증한다. 생산 page 결과의 RFC3339 분봉을 실제 generic replay에 넣어 실행했다.
- Toss localhost HTTP fixture에서 synthetic token과 실제 client/adapter로 inclusive cursor·plus URL 인코딩·정체 종료를 검증했다. KIS 생산 요청 빌더와 parser, Content-Length와 실제 bytes 8MiB 상한을 검증했다. 실제 계좌·주문 API는 호출하지 않았다.
- Playwright54개 통과: 기존 전략·연구 화면과200+252봉 수집, 겹침·빈 페이지·상한/부분 오류·scope drift/취소·분봉 offset·거래소 fallback. tsc/FSD/Vite build 통과.
- 독립 리뷰가 발견한 해외 날짜 컬럼·RFC3339 replay 입력·페이지 경계 instant 중복을 수정했다. 후속 리뷰의 host Local 날짜 의존도 수정했다. UTC/Seoul/New_York 독립 프로세스에서 KR/US 시장 자정 손절·일일 손실 제한 초기화·재진입의 전체 결과/hash/trades가 같음을 검증했다. NY DST 중복01:30의 실제60분 경과와 실운용 Local wrapper 보존 fixture도 통과했다. 최종 독립 재리뷰에서 추가 확정 버그는 없었다.
- 실제 DB 서버 contract는 이번 범위가 아니며 기존 lib fixture 통과를 실제 서버 검증으로 해석하지 않는다. Graphify의4개 설정 JSON0-node 안내는 기존 추출 한계이며 코드 빌드 경고와 구분한다.

## 후속 live 구현 기록

live52w: `{run generation, broker/account/profile, symbol, config fingerprint, market date}`를 고정하고 수집 중 상태가 바뀌면 폐기한다. 현재 stop/start는 bool만 있어서 같은scope의 빠른 재시작을 구분할 공유 generation이 필요하다. bounded background job·retry backoff로 quote/손절을 막지 않으며 부족 시 신규 진입만 차단하고 실제 보유 손절은 보존한다. 완료252봉은 최신봉 추가 제외 없는 targeted manager 경계로 적용한다. 상태를 IPC/웹/TS와 UI에 노출하고 같은날 반복 tick·날짜 변경·재시작·scope/config race fixture를 검증한다.

위 live 설계는 [후속 완료 일봉 구현·검증](live-completed-daily-history-20261018.md)에 반영했다. 실제 provider 운용·전체 주문 lifecycle 검증을 완료한 것으로 해석하지 않는다.
