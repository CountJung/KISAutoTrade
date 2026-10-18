# 사전자료 페이지 구현을 위한 공개 계약 확인

2026-10-18 KST. P2-17과 live P2-05의 선행 조사다. 공개 문서만 조회했고 token/계좌/주문 요청은 하지 않았다. 아래 구현 잔여는 자동 구현 과제로 유지한다.

## Toss

[공식 OpenAPI v1.2.24](https://openapi.tossinvest.com/openapi-docs/latest/openapi.json)에서 확인했다.

- `/api/v1/candles`는 최신순, 최대200봉, `MARKET_DATA_CHART` group이다.
- `before`는 ISO8601 inclusive 상한이다. 응답 `nextBefore`를 그대로 보내고 timezone `+`를 `%2B`로 인코딩한다. 겹침 제거와 cursor 무진전 종료가 필요하다.
- `1m` timestamp는 **봉 종료 시각**, `1d`는 현지 거래일 자정이다. 다른 자료원의 봉 시작 표기에 같은 정책을 적용하지 않는다.
- 현재 client는 before를 지원하지만 adapter는 최신200봉만 가져오고 cursor를 버린다. 공통 page 계약과 UI/실시간 준비 연결을 구현해야 한다.

## KIS

[국내 공식 샘플](https://github.com/koreainvestment/open-trading-api/blob/main/examples_llm/domestic_stock/inquire_daily_itemchartprice/inquire_daily_itemchartprice.py)은 `FHKST03010100`, 날짜1/2, 호출당 최대100건이다. 날짜를 후퇴시키는 bounded 페이지와 중복/미래/무진전 검증이 필요하다.

[해외 공식 샘플](https://github.com/koreainvestment/open-trading-api/blob/main/examples_llm/overseas_stock/dailyprice/dailyprice.py)과 [user 샘플](https://github.com/koreainvestment/open-trading-api/blob/main/examples_user/overseas_stock/overseas_stock_functions.py)은 `dailyprice`에 **HHDFS76240000**을 사용한다. 현재 `api/rest.rs`의 HHDFS76200200과 불일치한다. API_URL·BYMD·GUBN·MODP 및 `tr_cont` 전송 계약을 공식 기준으로 맞추는 요청 fixture를 우선 구현한다. 샘플의 재귀 방식만으로 중복 없는 자료 확보를 보장하지 않는다.

## 구현 순서

1. `api/rest.rs`가1,000행을 넘으면 차트/시세 모듈을 먼저 분리하고 공개 메서드 경로를 유지한다.
2. 공식 요청/TR-ID·cursor·시간 표기 회귀, 동일 scope·공유 rate-limit·오류/응답 상한을 보존한다.
3. 공통 page/cursor 계약과 IPC/웹/TS/UI를 연결한다. 합계500봉 replay 상한은 유지하며200/252봉 확보 실패 이유를 표시한다.
4. exchange 날짜와 완료 증거를 갖춘 live 일봉 refresh를 scope/symbol별 bounded cache·backoff로 연결하고, 자료 부족은 신규 진입만 막는다. sampled tick을 완료 고가로 합성하지 않는다.

공개 스펙 inventory는 섹터 GET5개 추가로33→38경로를 확인하고 검사·문서를 갱신해 통과했다. 위 page/TR-ID/live refresh 구현과 실제 provider 응답 검증은 아직 완료하지 않았다.
