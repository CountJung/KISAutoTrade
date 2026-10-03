# KISAutoTrade — Todo

> 완료 이력은 `git log`와 릴리스 노트에서 관리한다. 이 문서에는 아직 끝나지 않은, 검증 가능한 작업만 둔다.
> 우선순위는 `P1 정확성/신뢰성 → P2 전략 연구 UX → P3 유지보수` 순이다.

## *마지막 비판적 점검: 2026-10-03*

## P1 — 진행 가능한 잔여

SOXQ 연구에서 확인한 시뮬레이션 정확성 과제는 아래에 기록한다.


## SOXQ 3개월 연구에서 확인한 P1 과제 (2026-10-03)

근거: [평가 보고서](docs/research/soxq-3m-20261003/report.md), [차트](docs/research/soxq-3m-20261003/charts.html), [원시 결과](docs/research/soxq-3m-20261003/results.json).
평가 구간은 2026-07-06~10-02, 64거래일이며 사전자료 252봉을 별도 제공했다. 런타임 수정 없이 현재 엔진을 실행한 연구다.

후속 완료: **SOXQ-P1-02 고가/종가 warmup 분리** — 기본 OHLC 훅은 종가, 52주 신고가는 고가를 선택하도록 수정했다. 기본 200/252봉, 호출 1회, 첫 신호/미래 봉/분봉 혼입·보유 상태 회귀를 검증했다. [검증·동일 입력 재평가](docs/research/soxq-3m-20261003/p1-02-validation.md): 평균회귀 +1.4402%→+2.2083%, 완료 거래 2→1. 나머지 정확성 과제는 아래에 유지한다.

후속 완료: **SOXQ-P1-01 평가 구간·사전자료·준비 상태 분리** — 모든 전략이 같은 평가 기간을 유지한다. 6개 누락 initializer와 이전 MA/RSI 기준선을 복원하고, 200/252봉 기준을 유지하며 실제 준비 상태와 조건 평가 봉 수를 반환한다. live 신고가 최신 봉 제외와 자료 부족 재초기화 뒤 보유 손절을 검증했다. 단일 provider 응답의 이전 봉만 사전자료로 사용하고 자료 부족·준비 중·조건 불충족·신호 무체결을 구분한다. [동일 입력 재평가·검증](docs/research/soxq-3m-20261003/p1-01-validation.md).

후속 완료: **SOXQ-P1-03 일봉 완료봉·날짜 전환 이벤트** — 시가-only 시작 → 완성 OHLC 종가 평가 → 실제 체결/차단 포지션 피드백 → 완료봉 적재 순서를 도입했다. 강한 종가는 매일 전일 조건을 다음 평가일에 사용하고, 변동성 확장은 당일 OHLC와 이전 N일 범위를 비교한다. 0변동폭도 일수에 포함하며 날짜 전환 시 보유 상태를 유지한다. 두 전략은 일봉만 허용한다. 동일 종가 체결 근사이며 익일 시가·장중 체결은 P2-14로 유지한다. [v5 재평가·검증](docs/research/soxq-3m-20261003/p1-03-validation.md).

- [ ] **SOXQ-P1-04 — LTH replay를 입력 시각에만 의존하게 한다.**
  - generic LTH는 현재 시스템 분·장 세션을 읽는다. P1-01에서 deterministic=false 및 준비 미지원/평가 불가로 표시했으나 역사 시각 경로 통일은 남아 있다.
  - 완료 조건: 역사 timestamp를 사용하는 전용 경로로 통일 또는 generic 경로 거부; 실행 시각·분 경계를 달리해도 같은 신호/체결/hash.
  - 대상: trading/strategy/leveraged_trend_hold.rs:390, commands/strategy_preview.rs.
- [ ] **SOXQ-P1-05 — LTH 일봉 근사와 실제 장중 평가를 구분한다.**
  - 기존 전용 일봉 경로는 시가를 15분 뒤 가격으로 간주하고 시가/종가를 한 지표 버퍼에 적재한다. SOXQ 일봉 전용 실행은 신호 0건이며 장중 성과 평가는 불충분하다.
  - 완료 조건: 일봉 결과에 근사/평가 불충분 표시; 거래소 시간대·DST·세션이 일치하는 3개월 분봉 확보 후 재실행; 일봉 context와 분봉 EMA/RSI/ADX 버퍼 분리.
  - 대상: commands/strategy_preview.rs, trading/strategy/leveraged_trend_hold.rs:1110.

## P1 — 보류 사항

- [ ] MariaDB contract fixture와 지원 버전 matrix 검증.
  - adapter·DDL은 유지하지만 사용자 결정에 따라 실제 서버 fixture 검증은 보류한다.
- [ ] 여러 broker/account 포지션과 pending을 동시에 상주·대조하는 multi-scope runtime.
  - 현재 제품은 단일 활성 scope 불변식을 사용한다. 동시 다계좌 운용을 제품 범위로 채택할 때 `(broker, account, market, symbol)` tracker와 scope별 credential reconciliation으로 전환한다.

## P1 — 외부 검증 환경 필요

- [ ] TLS verify-full 인증서, 서버 강제 단절·재접속 contract.
  - CA/hostname 인증서 fixture와 서버 kill/restart 제어가 가능한 별도 통합 테스트 환경에서 검증한다.
- [ ] KIS/Toss 실제 정정·취소 응답의 최종 provider mapping 확정.
  - 공통 상태와 정정/reconciliation 직렬화는 구현했으나 old/new order ID 및 거부·만료 필드 의미는 provider fixture/소액 검증 후 고정한다.

## P2 — 전략 시뮬레이션과 연구 워크플로

아래 수치는 명시된 초기 연구/v3 시점의 관측이다. P1-01(v4) 초기화 결과는 [준비 상태 검증](docs/research/soxq-3m-20261003/p1-01-validation.md), 강한 종가·변동성 확장의 최신 결과는 [P1-03(v5) 검증](docs/research/soxq-3m-20261003/p1-03-validation.md)을 먼저 확인한다. 파라미터 비교는 남은 시각·체결 모델 과제 해결 후 진행한다.


- [ ] **SOXQ-P2-01 — 이동평균 교차:** 이번 -1.1197%, 완료 거래 2회 모두 손실. 장기 추세/MA 기울기·ATR 또는 밴드폭 필터를 각각 A/B 비교한다. 완료 조건: P1 초기화·체결 수정 후 동일 기간과 별도 하락/횡보 구간에서 순수익·MDD·거래 수 비교.
- [ ] **SOXQ-P2-02 — RSI:** +2.0224%, 완료 거래 2회. 하락 추세에서 조기 매수 위험을 확인하고 장기 추세 필터/손절을 비교한다. 완료 조건: 최소 표본 기준을 정해 별도 구간 반복 평가; 2회 승리로 우수 전략 확정 금지.
- [ ] **SOXQ-P2-03 — 모멘텀:** -0.4868%, 무보유 매도 차단 9건. 보유 상태와 임계값 교차를 이용해 중복 신호를 줄이고 진입/청산 임계값의 hysteresis를 비교한다. 완료 조건: 차단 사유별 수치와 순수익·MDD·신호 감소 확인; 공통 guard 유지.
- [ ] **SOXQ-P2-04 — 이격도:** +2.4987%, 완료 거래 2회, 무보유 매도 차단 10건. 보유 상태 동기화와 장기 하락 추세/최대 보유 기간/손절을 검토한다. 완료 조건: 기본값 대비 비용 포함 A/B와 급락 구간에서 손실 제한 검증.
- [ ] **SOXQ-P2-05 — 52주 신고가:** 이번 무거래. 롤링 252일 고가의 날짜별 만료·갱신과 충분한 히스토리 상태를 검증한다. 완료 조건: 실제 신고가 없음과 데이터 부족 구분; 기간을 임의 축소하지 않고 돌파/손절/오래된 고가 만료 fixture 및 별도 구간 실행.
- [ ] **SOXQ-P2-06 — 연속 상승/하락:** -0.6445%, MDD 2.9268%, 보유 관측 비율 81.25%. 작은 등락을 세는 진입에 최소 누적 변화율/ATR·장기 추세 필터를 비교한다. 완료 조건: 동일 예산·비용으로 MDD와 잦은 방향 전환 감소 여부 평가.
- [ ] **SOXQ-P2-07 — 돌파 실패:** -0.0005%, 완료 거래 1회와 미청산 보유 10주. 종가 단일 돌파와 실제 고가 돌파를 구분하고 거래량/ATR 버퍼·수익 보호 청산을 비교한다. 완료 조건: false breakout fixture 및 별도 구간에서 완료 손익과 미실현 손익 분리.
- [ ] **SOXQ-P2-08 — 강한 종가:** P1-03(v5) 날짜 이벤트 수정 후 +1.7179%(각20bp +1.5871%), 완료 거래 2회 모두 손실·기말 미청산 10주. 전일 조건→다음 평가일 종가 신호와 포지션 유지를 검증했다. 완료 조건: P2-14 next-open 모델로 익일 갭·체결 시점을 구분하고 별도 구간의 완료/미실현 손익과 조건 파라미터를 비교.
- [ ] **SOXQ-P2-09 — 변동성 확장:** P1-03(v5) 당일 OHLC·이전 N일 롤링 범위 수정 후 +0.7807%(각20bp +0.7523%), 9/21 매수 1회·완료 거래 0회. 0범위 포함 일수와 날짜별 고저가 초기화를 검증했다. 완료 조건: 실제 장중 경로/체결 가능성과 청산이 있는 별도 구간에서 범위·배수 조건을 비교; 미청산 수익만으로 전략 우수성 판정 금지.
- [ ] **SOXQ-P2-10 — 평균회귀:** P1-02 종가 초기화 수정 후 +2.2083%, 완료 1회(각20bp 비용 +2.1561%). 수정 전 +1.4402%와 신호 차이는 [재평가](docs/research/soxq-3m-20261003/p1-02-validation.md)에 기록했다. 하락 추세 회피/최대 보유 기간·중심선 청산을 비교한다. 완료 조건: 표본 확장과 추세 지속 하락 구간의 손실 확인.
- [ ] **SOXQ-P2-11 — 추세 필터:** 종가 warmup 수정 후에도 사전자료 실행 +0.8722%, 완료 거래 0회/미청산 보유 10주; 첫 매수 날짜는 같고 장기MA는 수정됐다. 사전자료 없는 64봉 실행은 200봉 부족으로 평가 불가 표시한다. P1-01에서 준비 상태를 분리했고, 후속 연구에서는 200일 MA 값과 실제 청산이 있는 별도 구간을 평가한다.
- [ ] **SOXQ-P2-12 — 레버리지 추세 보유(LTH):** generic/일봉 전용 모두 무신호. SOXQ를 넣은 엔진 동작 진단이며 레버리지 상품 성과로 일반화할 수 없다. P1 시간·분봉 모델을 해결한 후 기본값과 선택형 볼린저/반동 옵션을 각각 비교한다. 완료 조건: 실제 진입·장마감 청산 창을 포함하는 3개월 분봉, 시간대·DST 검증, 원본 분봉 보관.
- [ ] **SOXQ-P2-13 — 가격조건:** 평가 전일 종가 $99.29를 매수가로 고정, 익절 5%/손절 3%; +2.2408%, 완료 12회, 회전율 325.2350%. 익절/손절 직후 동일 기준 재진입을 공통 쿨다운/손절 후 재진입 guard와 함께 비교한다. 완료 조건: 조건을 평가 전에 기록하고 비용 스트레스·재진입 횟수·순수익을 비교; 임의 사후 최적화 금지.
- [ ] **SOXQ-P2-17 — provider별 사전자료 페이지 조회를 연결한다.**
  - 현재 generic UI는 KIS 단일 응답, Toss 커맨드 최대 200봉 안에서 평가봉을 우선 보존한다. 252봉 이전 자료가 부족하면 기간을 줄이지 않고 부족 상태를 표시한다.
  - 완료 조건: 공통 candle page/cursor 계약과 Tauri·웹 경로 동기화, 중복/겹침/미래 자료 제거, provider scope·rate limit 유지, 완료 일봉 확인과 live 신고가 최신 봉 제외 경계 유지, 200/252봉 확보 또는 명확한 부족 사유.
- [ ] **SOXQ-P2-14 — 체결 가능성과 장중 손절을 별도로 모델링한다.**
  - 현재 종가 신호가 동일 종가+슬리피지에 체결된다. next-bar-open 체결 모드, 신호/체결 시각 구분, 장중 경로 미확인 상태를 추가한다.
  - 완료 조건: 익일 갭 fixture; 같은 봉에서 손절·익절이 함께 닿을 때 순서 불명/보수적 처리; close-only와 next-open 결과 비교.
- [ ] **SOXQ-P2-15 — 비용·배당·환율·청산 가정과 비교 기준을 명시한다.**
  - 이번 가정은 1천만원/고정 10주/환율 1450원, 수수료·슬리피지 각 10bp, 거래세 0; 20bp 스트레스도 실행했다. 배당·환율 변화·기말 강제 매도는 제외했다.
  - 완료 조건: 같은 10주 보유와 동일 투자 비중 기준 비교, 열린 포지션/기말 청산 비용 표시, broker 실제 비용 입력과 배당/환율 시나리오 지원.
- [ ] **SOXQ-P2-16 — ATR을 true range로 계산하고 표본 부족을 성과와 구분한다.**
  - 현재 replay ATR은 high-low 평균으로 갭을 제외한다. 이번 대다수 전략은 완료 거래 0~2회여서 승률 100%나 PF null을 우수성으로 해석할 수 없다.
  - 완료 조건: 이전 종가 포함 TR·사전 ATR warmup·봉 간격 유지; 표본 수/신뢰 부족 표시; 70/30 구간을 기간 정규화 없이 과최적화 확정에 사용하지 않고 별도 walk-forward 평가.


## 자동매매 예산·볼린저 보강 검증 기록 (2026-09-11)

- [x] 계좌/통화별 전용 예산, 자동 소유수량, durable 예약·체결·완료 복원, 설정 UI와 E2E 추가.
- [x] 기존 LTH에 기본 비활성 볼린저 필터와 중심선/ATR 청산, 공통 live/replay 및 미리보기 설정 연결.
- [ ] 실제 provider 응답 불명 주문의 운영자 대조/복구 UI: 현재는 자동 해제 없이 예산을 보류한다. 주문번호/clientOrderId 대조 및 멱등 승인 흐름으로 확장할 것.
- [ ] LTH 보강의 일봉 미리보기: 분봉과 일봉의 완료 시점 모델을 분리한 후 지원. 현재 활성 옵션은 1분봉만 허용한다.

## P3 — 유지보수·품질 게이트

- [ ] 1,000라인 초과 파일을 책임 단위로 분리한다.
  - `src-tauri/src/trading/strategy/leveraged_trend_hold.rs`: 계산/상태/preview/tests 분리.
  - `src-tauri/src/commands/trading.rs`: lifecycle/reconciliation/price-source/daemon cycle 분리.
  - `src/pages/trading/ui/Page.tsx`: broker별 orchestration과 공통 order form 분리.
  - `src/api/hooks.ts`: account/market/order/strategy/settings query 모듈 분리 후 public API 유지.
  - `src-tauri/src/commands/toss.rs`: diagnostic/preflight/orders surface 분리.
  - `src-tauri/src/trading/order/submission.rs`: provider request builder와 submission persistence/test 분리.
  - `src/pages/strategy/ui/leveragedTrendHoldEditorPanel.tsx`: ticker picker/preview UI 분리.
  - `src-tauri/src/trading/order.rs`: facade를 낮추고 남은 helper/state 책임 분리.
  - 검증: 변경 파일과 신규 파일은 1,000라인 아래, FSD/API public surface와 IPC 이름은 유지한다.

- [x] 핵심 거래 흐름의 자동화 테스트를 release gate로 승격한다.
  - broker mock 기반 제출→부분체결→완전체결→취소/거부→재시작 복구 테스트를 추가한다.
  - balance fail-closed, scope 전환, midnight rollover, manual/auto parity, 인증 없는 REST 거부를 포함한다.
  - `cargo check`, `cargo test`, `npx tsc --noEmit`, `npm run check:fsd`, `npm run test:e2e`, OpenAPI 검증을 CI에서 실행한다.

- [x] 의존성·릴리스 보안 점검을 자동화한다.
  - `cargo audit`와 npm audit 정책, Dependabot/Renovate, lockfile 검증을 CI에 추가한다.
  - release artifact 서명/해시와 updater 경로를 검증하고, 실패 시 사용자가 확인할 수 있게 한다.
  - `release-gate.yml`, `security.yml`, Dependabot, 추적 lockfile 검증, SHA-256/attestation 및 detached signature 준비 검증으로 반영했다. 실제 플랫폼 코드 서명/updater 서명 강제는 배포 키 준비 후 `REQUIRE_RELEASE_SIGNATURES=true`로 전환한다.

- [ ] 사용자 가이드를 broker별 실제 지원 범위와 일치시킨다.
  - KIS 전용 소개를 KIS/Toss 공통 기능, broker별 주문·시세·자동매매 제한 표로 교체한다.
  - 실제 UI의 13개 전략 카드, 프로파일 설정, 세션/캔들 source, 소액 실거래 gate를 기준으로 오래된 절차를 정리한다.

## 완료 기준

- 각 항목은 코드, 실패/복구 테스트, 사용자-visible 상태, 관련 문서가 함께 반영되어야 닫는다.
- 실계좌 검증은 모의/fixture → read-only 계좌 조회 → 명시 승인된 소액 주문 순서를 지킨다.
- 경고를 포함해 `cargo check`와 TypeScript 검증이 깨끗해야 하며, 주요 UI 배치 변경은 Playwright로 확인한다.
