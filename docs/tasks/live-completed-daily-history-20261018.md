# Live 완료 일봉 갱신 — P2-05 / P2-17

2026-10-18 KST. 52주 신고가의 준비 자료를 실제 provider 완료 일봉 페이지로 갱신한다. 시세·손절 경로와 수집 작업을 분리하며 실제 계좌 조회·주문·자동매매 시작 없이 합성 fixture로 검증한다.

## 동작과 경계

- Manager의 run generation, profile/broker/account, 전체 전략 설정 fingerprint, symbol, KR Seoul/US New York 시장 날짜를 ticket에 고정한다. 성공 시작·정지·window close와 설정 변경/복원에서 이전 결과를 무효화한다. 같은 계좌의 stop→start 및 config A→B→A도 구분한다.
- daemon은 시장 폐장 대기 전에도 작업을 관리한다. provider future는 background에서 실행하고 완료된 handle만 수거한다. 최대2개 동시 작업, 각8회 요청, 실제512봉 보관이며 같은 scope/날짜/종목의 여러52주 설정은 최대 요구량을 한 번 조회해 나눠 적용한다.
- 요구량 기본252와 전략 상한512를 유지한다. 완료 페이지의 마지막 봉을 다시 제외하지 않는다. 당일/미래/무효 OHLCV·중복은 사용하지 않으며 빈 필터 페이지도 커서가 전진하면 계속 조회한다. KIS 거래소 후보 변경은 첫 provider 오류에서만 허용한다. scope/기준시각 변경은 부분 성공으로 삼키지 않는다.
- 상태/수집 대상은 실제511개와 초과 진단1행까지 유지한다. 초과한52주 대상도 신규 매수를 막는다. 제거·비활성 대상의 cache/job은 정리한다. 실패/자료 부족 재시도는30·60·120·240·480·900초 상한이다.
- 새 거래일에는 오래된 고가와 신규 진입 준비를 즉시 비운다. quote 요청 중 날짜가 바뀌어도 on_tick 직전 다시 검사한다. 자료가 부족하거나 실패해도 실제 보유의 buy_price·손절은 보존한다. sampled tick으로 완료 일봉 고가를 만들지 않는다.
- 마지막 완료 일자가 시장 날짜보다1~10일 과거인 경우만 준비 완료가 가능하다. 10일을 초과한 자료·미확인/당일/미래 날짜는 신규 진입 보류다. 이는 휴장 달력을 확정한 증거가 아닌 보수적 freshness 정책이며 긴 휴장도 보류한다.
- 기존 LTH 분봉·일봉 초기화는 호출하지 않는다. 52주만 사용하는 종목은 시작 시 동기 legacy seed에서도 제외한다. 다른 전략과 같은 종목은 기존 seed를 받은 뒤52주만 새 실행의 완료 자료를 기다린다.

## 보유 상태와 설정

52주 전략은 같은 ID/broker/account의 파라미터 변경·동일scope 저장 rollback에서 인스턴스의 보유 상태를 유지하고 새 설정과 관측 준비만 바꾼다. 다른 계정으로 보유 상태를 전달하지 않고, 사용자가 제거한 대상을 자동 복구하지 않는다. 실제 체결 뒤 갱신되지 않는 과거 잔고 cache를 새로 만들지 않는다. 다른 전략의 기존 rebuild/rollback 위험은 별도 P1 자동 과제로 유지한다.

## 상태와 UI

기존 TradingStatus에 optional 호환 `dailyHistory`를 추가한다. 각 항목은 strategyId/symbol/brokerId/profileId/marketDate/requiredBars/availableBars/state/source/lastCompletedDate/lastError/nextRetryAt이다. IPC와 웹 GET가 같은 camelCase 의미를 사용하며 웹 실행 profile/broker/account도 실제 run identity로 표시한다.

Dashboard와 Strategy는 기존10초 상태 query를 공유한다. pending/ready/insufficient/failed/capacityExceeded와 실제 확보량·시장 날짜·출처·실패/재시도 사유를 보여 준다. 화면은3행씩 표시하며 서로 다른 프로파일·계좌 상태를 섞지 않는다. legacy 필드 누락은 준비 완료로 추정하지 않는다. 프로파일 없는 기존KIS 실행의 실패 진단은 실행broker/account가 일치할 때만 표시한다. 상태/준비 완료가 매수 조건 충족이나 실제 주문 안전성·수익성 증명은 아니다.

## 검증 기록

- Manager focused19개:252번째high≠close·고가 만료·같은날 반복·다음날/DST·부족/실패 중 보유 손절·run/config/scope ABA·512봉/대상 cap·무관 LTH 미호출·설정/rollback 보유 보존.
- collector/scheduler focused: 순차252/512봉·겹침/미래/무효·빈 페이지·상한/커서/실패·scope drift, background 대기 중 손절·stop 취소·공유 수집·동시2개·재시도·같은scope 재시작 결과 폐기. 52주 전용 종목의 legacy seed 제외도 검증한다.
- 최종 Rust lib275개 + production LTH integration3개 + doc3개, 합계281개 통과. check all-targets와 Clippy all-targets/all-features 경고0. TypeScript/FSD 및 Vite production build 통과·코드 경고0. 신규 live 상태13개와 기존 Strategy22개, Playwright35개 모두 통과했다. 최종 project-map/Graphify/TODO freshness·rustfmt·diff 검사와 로컬 문서 링크133개도 통과했다.
- 실제 broker 주문·자동매매 시작·민감 계정 파일 접근은 수행하지 않는다. 기존 DB fixture의 내부 skip을 실제 DB 서버 검증으로 해석하지 않는다.

## 독립 검토와 다음 안전 과제

새 일봉 수집/적용·scope/날짜·bounded job·UI 상태의 독립 리뷰에서 확정 결함은 없었다. 다음 기존 경로는 이번 일봉 ticket 검증과 분리해 가장 높은 자동 우선순위로 추적한다.

1. quote await 이후 자동 주문이 현재 execution scope로 재해석될 수 있다. 신호 전 context 재검사, 예약의 원자 gate에서 active/generation/scope/profile 확인, stale 결과의 포지션 복원 금지와 예약 뒤 KIS client/retry pinning을 검증해야 한다. 단순 quote 뒤 bool 확인만으로 해결하지 않는다.
2. 일반 전략의 config rebuild/저장 실패 rollback은 보유 상태를 지울 수 있다. 최신 실제 tracker의 scope 검증 또는 commit 전 상태 보존 구조가 필요하며 시작 시 오래된 잔고 cache로 복원하지 않는다.

위 과제는 사용자 자료·판단 대기가 아닌 자동 구현이다. 일봉 갱신 완료를 전체 주문 lifecycle 경합 안전성 검증으로 확대하지 않는다.


공통 날짜 helper는 Graphify 후보 질의와 Serena 정의·참조(기존3·collector1·manager3곳)를 확인한 뒤 `broker/candles.rs::candle_market_date`로 승격했다. 시장 판별은 caller에 남겼고 가격·통화·OHLC 검증 의미가 다른 helper는 합치지 않았다. 상태 ready 확인도 전체512행 clone 대신 ticket lookup으로 처리해 manager lock 안의 불필요한 반복 할당을 줄였다. Graphify의4개 설정 JSON0-node 안내는 기존 추출 한계이며 코드 빌드 경고와 구분한다.
