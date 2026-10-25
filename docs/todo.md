# KISAutoTrade — Todo

> 완료 이력은 `git log`와 릴리스 노트에서 관리한다. 이 문서에는 아직 끝나지 않은, 검증 가능한 작업만 둔다.
> 우선순위는 `P1 정확성/신뢰성 → P2 전략 연구 UX → P3 유지보수` 순이다.

## *마지막 비판적 점검: 2026-10-25*

[색상·필터가 있는 할일 보기](todo.html) · 원본은 이 문서다. 수정 후 `npm run todo:render`, 동기화 확인은 `npm run check:todo`를 실행한다.

- **🟢 자동 구현**: 추가 사용자 판단 없이 구현·fixture·연구 검증을 진행한다. 선행 과제가 있으면 순서대로 해결하며 어려움만으로 대기로 바꾸지 않는다.
- **🟠 사용자 판단 대기**: 제품 범위·기존 보류 해제·배포 키 등 명시적 선택이 필요하다. 항목의 필요한 결정을 먼저 확인한다.
- **🟠 사용자 검증·자료 대기**: 실제 자료 제공·계좌 접근·별도 승인된 검증이 필요하다. 로컬 fixture와 자동 구현 부분은 계속 진행한다.

주황 표시는 미완료 상태다. 테스트·문서·독립 검토가 완료된 항목만 체크하며, 실제 검증이 남은 항목을 자동 완료하지 않는다. 대기 항목에는 미완료 범위·진행 불가 사유·필요한 자료/결정·재개 조건·완료 기준을 나누어 기록한다. 자동 구현의 로컬 검증은 추가 사용자 승인 대상으로 바꾸지 않는다.

## P1 — 진행 가능한 잔여

### 현재 전략 전수 연구 후 권장 순서 (2026-10-18)

[전담 분석 보고서](research/strategy-review-20261018/report.md) · [실행별 근거](research/strategy-review-20261018/results.jsonl) · [요약](research/strategy-review-20261018/summary.json). 등록 13종 중 일봉 12종 1,224회, LTH 실제 분봉 표본 4회 진단을 실행했다. 주집계는 SOXQ/SPY/QQQ의 2023Q1~2026Q3 완전 45창이다. 고정된 전일 기준 15% 목표 수량, 수수료·슬리피지 각각 10/20bp, 기존 guard를 유지했다. 모두 기본 파라미터이며 가격조건은 전일 종가·익절5%·손절3% 연구 설정이다. 전략·계좌 설정을 변경한 작업이 아니다.

1. **정확성 선행:** **SOXQ-P2-05** generic 일봉 rolling252 고가와 **P2-16** TR14/준비 진단을 수정·회귀 검증했다. live 완료 일봉 공급은 후속 P2-05·17에서 연결했다. P2-16의 표본/walk-forward는 남아 열린 상태다. [v8 비교 근거](research/autonomous-corrections-20261018/report.md).
2. **비교 기반 완료:** **P2-14·15** 체결 시점과 실현/잔여 평가손익·기말 청산 비용, 가상 FX/배당·실제 투자비중 비교를 분리했다. 강한 종가 15창의 평가이익/실현손실과 QQQ 가정청산 비용의 부호 전환을 검증했다.
3. **불필요한 반복:** **P2-03·04·13**의 상태·중복 신호·손절 후 재진입을 공통 guard 유지하에 검증한다.
4. **자료/표본:** **P1-05·P2-12·16·17**의 실제 분봉·봉 공개 시각·DST·누락 자료·ATR·준비 자료 조회를 검증한다.
5. **연속 기반 완료·전략별 가설:** **P2-18**의3년 상태 보존/분기 초기화 비교를 완료했다. 아래 P2-01~13 후보는 비용 경로·반복 신호 근거를 우선하며 하나씩 사전 고정하고 기각 결과도 보관한다.

P1-RUN-01의 실행 epoch·설정 revision 예약 검증, provider 고정, 수동/체결 대조 계좌·모드 보호를 구현했다. [검증 기록](tasks/execution-run-reservation-20261018.md). P1-CONFIG-01은 저장 후 인스턴스 공개와 13전략 보유 보호 보존으로 해결했다. [검증 기록](tasks/strategy-configuration-preservation-20261018.md). P1-BAL-01은 공통 잔고 게시 gate와 계좌별 캐시·오류 복구로 해결했다. [검증 기록](tasks/holdings-publication-fence-20261018.md). P1-FILL-01은 exact owner 체결·원가·보호 복원으로 해결했다. [검증 기록](tasks/strategy-fill-ownership-20261018.md). 로컬 TLS verify-full·서버 단절/재접속 contract도 완료했다. P2-14 체결 모델과 P2-15 손익·비용·현금흐름 비교도 완료했다. P2-18의 연속 상태 경계·실제3년/별도분기 비교도 완료했다. P2-06의 기본 비용 경로와 단독 진입 필터 비교를 완료했다. P2-03 모멘텀의 실제 소유 피드백·교차 후보 비교도 완료했다. P2-04 이격도 실제 소유·단독 진입/청산 후보 비교도 완료했다. P2-10 평균회귀 소유 피드백·단독 후보 비교도 완료했다. P2-01 이동평균 교차의 이전 완료 일봉 필터와 단독·사후 스트레스 비교도 완료했다. P2-02 RSI 실제 소유·단독 후보 비교도 완료했다. P2-07 돌파 실패의 실제 소유·완료 자료·선택 보호 비교도 완료했다. P2-08 강한 종가의 실제 소유·완료 시장일·선택 청산 비교도 완료했다. P2-09 변동성 확장의 테스트 가능한 구현·일봉 비교도 완료했고 실제 장중 근거는 열린 자료 조건으로 유지한다. 다음 자동 과제는 P2-11 추세 필터의 청산·노출 비교다.

모든 12전략의 동일 수량 보유 대비 초과수익 중앙값이 0 이하이며 우수 전략을 확정하지 않았다. 분기 포트폴리오는 매번 초기화하며, 부분 Q4와 겹치는 최근248봉은 주집계에서 제외했다. 작은 완료 표본·서로 상관된 ETF·same-close 체결·고정 환율/무배당의 한계를 유지한다.

SOXQ 연구에서 확인한 시뮬레이션 정확성 과제는 아래에 기록한다.


## SOXQ 3개월 연구에서 확인한 P1 과제 (2026-10-03)

근거: [평가 보고서](research/soxq-3m-20261003/report.md), [차트](research/soxq-3m-20261003/charts.html), [원시 결과](research/soxq-3m-20261003/results.json).
평가 구간은 2026-07-06~10-02, 64거래일이며 사전자료 252봉을 별도 제공했다. 런타임 수정 없이 현재 엔진을 실행한 연구다.

후속 완료: **SOXQ-P1-02 고가/종가 warmup 분리** — 기본 OHLC 훅은 종가, 52주 신고가는 고가를 선택하도록 수정했다. 기본 200/252봉, 호출 1회, 첫 신호/미래 봉/분봉 혼입·보유 상태 회귀를 검증했다. [검증·동일 입력 재평가](research/soxq-3m-20261003/p1-02-validation.md): 평균회귀 +1.4402%→+2.2083%, 완료 거래 2→1. 나머지 정확성 과제는 아래에 유지한다.

후속 완료: **SOXQ-P1-01 평가 구간·사전자료·준비 상태 분리** — 모든 전략이 같은 평가 기간을 유지한다. 6개 누락 initializer와 이전 MA/RSI 기준선을 복원하고, 200/252봉 기준을 유지하며 실제 준비 상태와 조건 평가 봉 수를 반환한다. live 신고가 최신 봉 제외와 자료 부족 재초기화 뒤 보유 손절을 검증했다. 단일 provider 응답의 이전 봉만 사전자료로 사용하고 자료 부족·준비 중·조건 불충족·신호 무체결을 구분한다. [동일 입력 재평가·검증](research/soxq-3m-20261003/p1-01-validation.md).

후속 완료: **SOXQ-P1-03 일봉 완료봉·날짜 전환 이벤트** — 시가-only 시작 → 완성 OHLC 종가 평가 → 실제 체결/차단 포지션 피드백 → 완료봉 적재 순서를 도입했다. 강한 종가는 매일 전일 조건을 다음 평가일에 사용하고, 변동성 확장은 당일 OHLC와 이전 N일 범위를 비교한다. 0변동폭도 일수에 포함하며 날짜 전환 시 보유 상태를 유지한다. 두 전략은 일봉만 허용한다. 동일 종가 체결 근사이며 익일 시가·장중 체결은 P2-14로 유지한다. [v5 재평가·검증](research/soxq-3m-20261003/p1-03-validation.md).

후속 완료: **SOXQ-P1-04 LTH 과거 실행의 현재시각 의존 차단** — generic LTH는 factory 전 `UNSUPPORTED_GENERIC_REPLAY`로 명시 거부하며 접미 ID·모든 간격에 적용한다. 기존 입력 시각 전용 경로를 유지하고, production-library fixture에서 세션/blackout/장마감·신호/체결·미래 봉 불변을 검증했다. 서로 다른 시스템 분·시간대의 오프라인 재실행에서 지원 24실행과 거부 결과·전용 일봉 진단이 일치한다. 합성 일봉/고정 세션의 시장 정확성은 P1-05에 유지한다. [v6 검증](research/soxq-3m-20261003/p1-04-validation.md).

- [ ] **🟠 사용자 검증·자료 대기** **SOXQ-P1-05 — LTH 일봉 근사와 실제 장중 평가를 구분한다.**
  - [x] v7/LTH v3에서 일봉 context를 bounded 별도 버퍼로 분리했다. 분봉 EMA/RSI/ADX·반동·볼린저에 일봉을 넣지 않으며, 분봉 OHLC 재초기화는 OHLC·반동·볼린저 snapshot, 가격 재초기화는 반동 snapshot만 교체한다. 일봉 재초기화는 분봉·보유 상태를 유지한다.
  - [x] 전용 일봉 경로는 시가를 장중 진입 가격으로 합성하지 않고 `dailyDiagnostic/notEvaluable` 자료 진단만 반환한다. UI는 수익률·거래 성과·A/B 저장을 숨기며 assessment 없는 기존 LTH 응답도 평가 불가로 표시한다.
  - [x] 현재 최대 200봉의 실제 분봉은 `intradaySample/sampleOnly`, 유효 분봉 0개는 `notEvaluable`로 표시한다. 일봉 context 수와 실제 장중 관측 수를 분리하고 시간대·DST·세션 미검증, 고정 KST 근사와 3개월 대표성 부족을 명시한다.
  - **현재 미완료:** 2026-07-06~10-02 SOXQ 실제 분봉 원본, 거래소 시간대·DST·실제 거래일/조기마감·세션·기간 전체 coverage 검증과 생산 replay 재실행이 남아 있다. 2026-10-22의 시간/미관측 슬롯 진단은 합성 fixture·mock 검증이며 실제 공급원·발행/수신 시각·전체 달력을 확인한 것은 아니다.
  - **진행할 수 없는 사유:** 보관한 짧은 실제 분봉 표본과 일봉으로는 요청한 3개월의 장중 경로·누락 원인을 복원할 수 없다. 사용 가능한 해당 기간 원본/이용 권한이 확인되지 않았으며 일봉 보간이나 합성 fixture로 대체하지 않는다.
  - **필요한 비식별 자료:** 원본 파일 경로 또는 사용 가능한 공급원과 자료 이용 범위, feed·가격 조정 여부·봉 시작/종료 표기·요청 기간·수집 시각 metadata를 제공한다. API key·계좌번호·개인 계정 파일은 문서나 에이전트에 전달하지 않는다.
  - **재개 조건:** 원본과 이용 범위가 확인되면 원본 hash·누락/중복·거래소 달력·공개 가능 시각을 먼저 고정하고, 기존 fixed KST 실행 모델과 검증된 시장 시각의 차이를 해결한 뒤 재실행한다. 파싱 진단 통과만으로 `lookAheadSafe`나 성과 상태를 승격하지 않는다.
  - [ ] **객관적 완료 기준:** 요청 기간의 원본·출처·hash·세션별 coverage/제외 사유를 보관하고, DST·휴장/조기마감·gap·진입/마감 창 및 생산 신호/체결·독립 정산을 재현 가능하게 검증한다. 미확인 구간은 평가 불가로 남긴다.
  - **중복 미완료 조건 통합:** 기존 제목 **“LTH 보강의 일봉 성과 미리보기”**의 근거와 잔여 조건을 이 항목에 통합했다. 일봉 context/분봉 지표 분리와 일봉 `notEvaluable`는 이미 구현되었고, 일봉에서 장중 성과를 복원하는 새 기능을 약속하지 않는다. 활성 볼린저 성과는 실제 1분봉·시장 시간 검증 후에만 평가하며 옵션별 비교 결과는 별도 P2-12에 남긴다. 통합은 삭제나 완료 처리가 아니다. [시간 진단의 검증 범위](tasks/lth-minute-timing-20261022.md).
  - 대상: `commands/strategy_preview.rs`, `trading/strategy/leveraged_trend_hold/{live,preview,indicators}.rs`, `trading/simulation/assessment.rs`. [작업 기록](tasks/soxq-p1-05.md), [v7 검증](research/soxq-3m-20261003/p1-05-validation.md).

## P1 — 보류 사항

- [ ] **🟠 사용자 판단 대기** MariaDB contract fixture와 지원 버전 matrix 검증.
  - **현재 미완료:** MariaDB 실제 서버의 contract fixture와 지원 버전별 schema/import/export·transaction·실패/재접속·인증 matrix 검증이다. adapter·DDL은 유지하며 PostgreSQL16 로컬 TLS/재접속 성공을 MariaDB 성공으로 확대하지 않는다.
  - **진행할 수 없는 사유:** 실제 서버 fixture는 기존 사용자 결정으로 보류되었고 지원할 MariaDB 버전·인증 범위가 확정되지 않았다. 임의로 보류를 해제하거나 비TLS 인증 지원을 제거하지 않는다.
  - **필요한 사용자 판단:** 보류 해제 여부, 지원 버전과 인증/TLS 범위를 결정한다. 운영 계정/DB 자료 대신 격리된 합성 fixture를 사용한다.
  - **재개 조건:** 범위가 정해지면 운영 서비스와 분리한 일회용 서버·합성 접속정보로 테스트를 실제 실행한다. 환경 미설정으로 즉시 반환한 테스트는 통과로 세지 않는다.
  - **객관적 완료 기준:** 지원 matrix의 각 조합에서 계약 실행 건수·성공/거부·rollback·재접속과 비밀정보 비노출을 기록하고 CI 실행/미실행을 구분한다. [기존 PostgreSQL 검증과 제외 범위](tasks/database-tls-reconnect-20261018.md), SQLx 전환 정책은 DEP-P3-01·02에서 별도 관리한다.
- [ ] **🟠 사용자 판단 대기** 여러 broker/account 포지션과 pending을 동시에 상주·대조하는 multi-scope runtime.
  - **현재 미완료:** 여러 broker/account의 포지션·pending을 동시에 상주시켜 scope별로 대조하는 runtime이다. 현재의 단일 활성 scope·owner 원장·계좌 전환 경합 보호는 구현했으며 동시 다계좌 운용과 다르다.
  - **진행할 수 없는 사유:** 동시 다계좌 운용의 제품 채택 여부와 scope별 실행·정지·리스크 한도가 정해지지 않았다. 현재 불변식을 임의로 바꾸면 주문·잔고·복원 경계가 달라진다.
  - **필요한 사용자 판단:** 동시 운용 채택 여부, 지원 broker/시장 조합, 계좌별·전체 리스크/비상정지 범위를 결정한다. 실제 credential·계좌 파일 제공은 이 결정에 필요하지 않다.
  - **재개 조건:** 제품 범위가 확정되면 `(broker, account, market, symbol)` tracker와 scope별 credential reconciliation 설계를 고정하고 mock부터 구현한다.
  - **객관적 완료 기준:** 동시 scope의 주문/체결/pending·캐시·소유 원가·재시작·부분 실패가 서로 침범하지 않으며 scope별/전체 정지·리스크를 실패 fixture와 UI에서 검증한다. 실제 계좌 운용은 별도 승인 검증으로 남긴다. [현재 소유·scope 경계](ARCHITECTURE.md), [기존 소유 피드백 검증](tasks/strategy-fill-ownership-20261018.md).

## P1 — 통합 검증과 실제 provider 확인

- [x] **P1-CONFIG-01 — 일반 전략의 설정 저장 중 보유 보호 상태를 보존했다.**
  - 13전략의 같은 ID/scope 재설정과 저장 후 공개, 저장 실패·불명확 저장 원본 복구/복구 실패 차단, params/targets 변경·빈 프로파일·held stop/trailing·취소·신호/주문 경합을 검증했다. 실제 provider·정상 fill 소유권 피드백까지 완료한 것은 아니다. [구현·검증](tasks/strategy-configuration-preservation-20261018.md).
  - 보유 또는 private 보호 중인 대상 제거/시장 변경을 거부하고, 다른 전략/계좌의 보유를 새 대상에 옮기거나 공유 tracker의 0수량으로 기존 보호를 지우지 않는다.

- [x] **P1-FILL-01 — 정상·부분 체결과 거부/취소를 주문 소유 전략에 반영했다.**
  - 계좌 snapshot과 exact owner(profile/account/모드/전략/시장) 원장을 분리하고 소유 수량·원가·LTH 최고가/관측을 보존한다. live 신호로 확정 보유를 바꾸지 않으며 자동매도/수동매도 가능 수량을 예약 직전 다시 검증한다.
  - provider snapshot 선반영 뒤 fill 중복 가감 방지, 누적 체결대금301의 잔액 보존, 고정 intent·0수량/증가수량 취소·저장 실패 재시도·원장 세대/재시작·타 owner/프로파일/모드 격리를 검증했다. KIS/Toss 엄격 잔고 변환과 Toss 수수료 비율100배 오류도 수정했다.
  - Rust455개(단위449+통합3+doc3), 브라우저89개, 독립 코드·심볼·helper·문서 리뷰 통과. 실제 운용/도입 전 보유 귀속은 별도 검증이다. [구현·검증과 한계](tasks/strategy-fill-ownership-20261018.md).
- [x] **P1-BAL-01 — 일반 잔고 조회의 늦은 응답과 계좌별 캐시 경합을 보호했다.**
  - 기존 게시 14곳을 공통 gate에 연결하고 profile/client/scope/모드·실행 epoch·설정 lease와 tracker별 조회/수량 version을 검증한다. A→B→A, 정지/재시작, 실제 0수량·삭제·두 시장 원자성·실제 partial fill·HTTP barrier를 검증했다. 네트워크 await 동안 전역 lock을 유지하지 않는다.
  - 잔고 이벤트는 현재 scope 재조회 힌트로 사용하고 이전 cache를 제거한다. 늦은 설정 응답·설정 재조회 실패·연속 프로파일 변경·backend 적용 후 mutation 오류도 검증했다. Rust365개·브라우저 신규13개와 기존26개, 독립 리뷰를 통과했다. [구현·검증과 한계](tasks/holdings-publication-fence-20261018.md).

- [x] **TLS verify-full 인증서, 서버 강제 단절·재접속 contract를 완료했다.**
  - 임시 CA/localhost 인증서의 PostgreSQL16 로컬 fixture에서 일반 계약3개와 TLS/재접속2개를 실제 실행했다. 신뢰 CA 성공, hostname/CA 불일치·비TLS·오인증 거부, backend socket 종료와 SIGKILL/재기동 뒤 동일 pool 복구, 단절 중 pending/소유/risk 오류 및 JSON/backup/backend 설정 보존을 검증했다.
  - cold pool single-flight·설정/pool snapshot, 파일/keychain 동시 원복·취소/불명확 복구 차단을 보강했다. 릴리스 gate에 같은 fixture를 연결했으며 실제 운영 DB·MariaDB matrix·Windows 실행은 별도 범위다. [구현·검증과 한계](tasks/database-tls-reconnect-20261018.md).
- [ ] **🟠 사용자 검증·자료 대기** KIS/Toss 기존 운영 원장과 실제 정정·취소 응답의 최종 검증.
  - **현재 미완료:** 실제 KIS/Toss 정정·취소의 old/new order ID·거부/만료 의미와 도입 이전 보유·기존 체결의 전략/프로파일 귀속·원가·투자모드를 실제 원본에 대조하는 검증이다. 공통 상태·직렬화·exact owner·로컬 합성 재시작/체결은 검증했으며 운영 원장 검증과 구분한다.
  - **진행할 수 없는 사유:** 실제 사례의 비식별 원본과 운영자 귀속 확인이 없고 계좌 조회·소액 주문의 별도 승인이 없다. 새 원장에 계좌 전체 보유를 자동 편입하거나 확인 불가 기록의 주문 차단을 해제할 수 없다.
  - **필요한 비식별 자료/판단:** 식별값을 일관되게 치환한 요청·응답·시간순 상태·누적 수량/대금/비용 사례와 도입 이전 보유 귀속 대조 결과를 제공한다. 운영 연결이 필요하면 read-only 범위를 먼저 정하고 소액 주문은 별도로 명시 승인한다. key·계좌 원문·개인 계정 파일은 받지 않는다.
  - **재개 조건:** 비식별 fixture로 mapping·중복/역순·부분체결/정정/취소/거부/만료를 먼저 재현한다. read-only 대조 후에도 불충분한 실제 주문 검증만 별도 승인 범위에서 진행한다.
  - **객관적 완료 기준:** provider별 원래/새 주문 연결·최종 상태·수량/원가/모드·소유 귀속을 원본 근거와 대사하고 불명확 사례의 fail-closed·재시도 멱등성을 입증한다. 운영자 대조/복구 UI 구현은 아래 자동 구현 항목에서 계속 진행한다. [로컬 검증과 운영 제외 범위](tasks/strategy-fill-ownership-20261018.md).

## P2 — 전략 시뮬레이션과 연구 워크플로

후속 완료: **P2-05·P2-17 live 완료 일봉과 페이지 준비 연결** — scope/config/run/시장 날짜를 고정한 bounded background 수집,252/512봉 요구·마지막 완료봉 보존·부족 중 신규 매수 차단/보유 손절, IPC/웹/UI 상태와 같은scope52주 설정/rollback 보유 보존을 구현했다. [검증/한계](tasks/live-completed-daily-history-20261018.md). 실제 provider 운용·전체 주문 lifecycle 검증으로 확대하지 않는다. 발견한 기존 주문·다른 전략 설정 위험은 위 P1 자동 과제로 먼저 해결한다.

전략별 항목의 최신 수치는 [2026-10-18 전수 연구](research/strategy-review-20261018/report.md)의 완전45창·기본 비용 결과다. 별도로 명시한 과거 SOXQ 수치는 당시 구간 관측이다. P1-01(v4) 초기화 결과는 [준비 상태 검증](research/soxq-3m-20261003/p1-01-validation.md), 강한 종가·변동성 확장은 [P1-03(v5) 검증](research/soxq-3m-20261003/p1-03-validation.md), generic LTH 거부는 [P1-04(v6) 검증](research/soxq-3m-20261003/p1-04-validation.md), 현재 일봉 진단/분봉 표본 경계는 [P1-05(v7) 검증](research/soxq-3m-20261003/p1-05-validation.md)을 먼저 확인한다. 일반 전략은 완료된 P2-14·15·18의 체결/회계 가정을 고정해 후보 비교를 이어가고, LTH의 별도 시간·자료 검증은 유지한다. 별도 [P2-18 연속 연구](research/continuous-replay-20261018/report.md)는 nextOpen·2023~2025 각752봉·최초 수량/PC기준가 고정의3년 장부와12개 독립 초기화 계좌를 구분한다. 기존45분기 통계와 합산하지 않는다.


- [x] **SOXQ-P2-01 — 이동평균 교차의 이전 완료 일봉 진입 필터와 단독 비교를 완료했다.**
  - 기존5/20 입력 관측 교차·Sell·매 관측 prevMA 갱신과 공통 guard를 유지하고 선택 추세/MA기울기/ATR%/밴드폭%를 Buy후보에만 적용했다. 새4조건은 strict 이전 완료 일봉만 사용하며 필터 거절 교차를 되살리지 않는다. 저장/두 preview·scoped 일봉 재조회/실패·raw UI/A-B·준비/조건미달·1분 거부·활성 잠금/모바일을 검증했다. [작업·검증](tasks/ma-cross-filters-20261020.md).
  - archived327739a 대비 기본1224개 전체 backtest/signals 호환과 unfiltered78개 원장 동일, 최종48연속/48분할/576별도분기(624원장)를 확인했다. 부모104,884개 기록 산술+3,240개 이전일봉 조건 대조·독립313,205개 검사 오류0. 기본1224는 생산 hash 대조 증거이며 독립 직접 기본원장 대조78개와 구분한다.
  - 추세/기울기는 연속6개 순평가수익 모두 하락했다. ATR0.5/폭2는 거부0으로 활성 효능 미평가다. 이를 본 뒤 사후 고정 ATR2/폭10 스트레스를 각각 한 번 추가했으며 원래36controls 입력/경제hash를 보존했다. 폭10 SOXQ20bp 수익7.4794→10.7262%/MDD11.8990→7.5717%이나 최저 완료손실 동일·노출 감소, SPY/QQQ 수익 하락이다. SPY 두 스트레스 완료1개·평균투자1.6268%를 우월성으로 표현하지 않는다.
  - 비용 때문에 SOXQ trend200의2025-01-21매수가20bp에서 비중한도로 차단돼10bp 손실청산−218,390.83원을 건너뛰는 경로 차이, 기말 청산비용의 분기3개 양→음 전환을 보존했다. 기본5/20·선택옵션0 유지, 추가격자/재조정·운용채택 없음. [사전 선언·사후 탐색·기각/표본·실현/잔여·비용/노출](research/ma-cross-filters-20261020/report.md).
- [x] **SOXQ-P2-02 — RSI 실제 소유·단독 진입/청산 후보 비교를 완료했다.**
  - v18은 단순RSI14/30/70·zero-loss100·기존once-cross를 유지하며 actualowned 수량/원가·잔여전량청산·age/metadata·재설정을 연결했다. 추세200·손절3%·보유63관측을 각각 단독 비교하고 새옵션0을 유지한다. strict 저장/두preview·current/A-B·legacy/손상/모바일·실제guard거절후cap재후보를fixture/mock으로검증했다.
  - 비RSI1122개 전체report/signals 호환, RSI102개 finalEquity/accounting 동일(100개raw/report변경)을확인했다. 30연속/30분할/360별도분기에서owned78개actual경제경로는legacy와같고flatSell386·heldBuy70 제거, raw648→192·blocked462→10이다. 신호감소를수익개선으로해석하지않는다.
  - trend200은연속4악화/2동일, stop3는6개전부수익악화·회전증가/SOXQ비용부호전환이었다. hold63은cap0·78개report/signals동일·최장54/57/46관측으로효능평가불가다. 사후튜닝없이옵션0을유지하며20개무신호와36개무완료(owned)를준비부족과구분한다. [작업](tasks/rsi-policy-20261022.md), [설계·실현/잔여·비용/노출·기각](research/rsi-policy-20261022/report.md).
  - P2-18 연속20bp 완료는3ETF 합23회(9/7/7), 차단116건이다. 작은 완료 표본과 무보유 매도·회전·노출을 먼저 분리한다.
- [x] **SOXQ-P2-03 — 모멘텀 실제 소유 피드백과 선택 진입 교차를 완료했다.**
  - 실제 owned_quantity를 관측·무장/미체결 후보와 분리하고 flat Buy/held Sell을 적용했다. 기존 inclusive >=T/<=−T, 보유 청산·부분체결·거절 재시도·live pending·같은0수량 피드백·strict warmup/재설정과 공통 guard/risk를 유지한다. 기본 level/0, 선택 entry-only crossing/간격%p를 제공한다.
  - 비모멘텀1122회 전체 report/signals 호환, 모멘텀102창 경제지표·회계 동일(65창에서 신호/차단수만 변경), 연속24/분할24/별도분기288과 parent46,772·독립136,960검사를 통과했다. corrected level은 legacy와78개 회계/자산곡선/실제체결 exact동일이다. 비용당 연속 raw231→42·차단195→6, 무보유Sell187/보유Buy2건을 제거했으며 수익·수수료 개선으로 표현하지 않는다.
  - crossing/1%p는 연속 장부 추가 효과가 없고 각비용36분기에서2개개선/3·4개악화여서 공통 기본 채택을 기각했다. 복수기간·두비용 순수익/MDD/회전·최저완료손익·실현/잔여·노출, 동결 v7과 구분한 새 엔진 기준은 [연구](research/momentum-feedback-20261020/report.md), 구현·실패/복구·전체검증은 [작업 기록](tasks/momentum-feedback-20261020.md)에 보관했다. 실제 provider 운용/OOS 우월성은 별도다.
- [x] **SOXQ-P2-04 — 이격도 실제 소유·하향 교차·단독 진입/청산 후보를 완료했다.**
  - MA20/−5/+5의 inclusive 기본 경계와 공통 guard를 유지하고 소유량·native 평균가·관측 나이를 분리했다. 선택 crossing/추세 관측/보유 관측/손절의 저장·두 preview·UI/A-B·실제 소유 metadata 복원과 live 추세 cadence 분리, 미체결/거절/partial/준비/설정 변경을 검증했다. 저장 중 새 초안도 일반/PC/LTH에서 보존한다. [작업 기록](tasks/deviation-policy-20261020.md).
  - archived bca6601 대비 비이격도1,122회 exact, 이격도102창 경제 동일(60창 raw/차단만 변화),36연속/36분할/432별도분기와 독립200,495검사 통과. 기본 level은78/78개 실제 filled/회계/자산곡선 동일이며 비용별 raw305→23·차단286→4를 비용절감으로 표현하지 않는다.
  - crossing은 연속 동일·분기 비용별1개 개선/35동일, trend200은3개 개선/4악화·SOXQ 낮은 노출, maxHold63은 분기 동일·SPY 비용별 청산 경로 차이, stop5는SOXQ20bp 수익4.5454→2.6040%/MDD9.1305→9.5692%로 악화했다. 기본 level/다른옵션0을 유지한다. [사전 고정·기각 근거](research/deviation-policy-20261020/report.md). 과거 동결 SOXQ v7 +1.3756%와 초기 v3 +2.4987%는 다른 구간/엔진 기록이며 이번3년 수치와 합치지 않는다.
  - P2-18 당시 연속20bp 차단286건에는 무보유 매도200·보유 중 매수82건이 포함됐다. 이번 기본 소유 교정의 재현 기준이며 실제 체결·손익 동일성을 별도 확인했다.
- [x] **SOXQ-P2-06 — 연속 상승/하락 진입 필터와 생산 A/B를 완료했다.**
  - 기본0의 누적 변화·직전 완료 일봉 ATR·장기 MA 진입 확인을 제공한다. live 가격 관측과 일봉 context를 분리하고 잘못된 설정/자료 부족/0 ATR은 신규 매수만 보류한다. 보유 청산·설정 재저장·관측열과 기존 guard/risk를 유지하며 현재 평가 OHLC는 판단 뒤에만 게시한다.
  - 직전 생산코드 단기1,224회·장기6회 전체 report/signals 기본 호환, 고정 후보24연속·24분할대조·288별도분기, parent55,986·독립148,099산술 대사를 통과했다. 누적1%는 SPY/QQQ 악화, ATR0.5는 SOXQ 제한 개선이나 MDD/최악손실 동일·SPY무효, MA200은6개모두수익하락이라 공통 기본 채택하지 않았다. 기본0을 유지한다.
  - SPY 비용10/20bp의 매도2023-08-04↔08-07/2025-02-25↔03-31과 후속 기대 순익 차단을 보존했다. MDD·회전·최악 완료 손실·실현/잔여·노출·필터 거절/후속 주문 차단 및 기각 근거는 [연구](research/consecutive-filters-20261018/report.md), UI/실패/복구·전체 검증은 [작업 기록](tasks/consecutive-entry-filters-20261018.md)에 기록했다. 실제 운용 효과/OOS 우월성이나 실거래 검증 완료를 뜻하지 않는다.
- [x] **SOXQ-P2-07 — 돌파 실패:** 실제 소유·후보 anchor·완료 OHLCV와 선택 고가/ATR/거래량·수익 보호/보유 한도를 구현하고 UI·엄격 저장/두 preview를 검증했다. 다음 시가 체결의 anchor 유실을 교정했으며 기존 sameClose102개 변화0·비대상1122개 정확히 일치, 42연속/42분할/504분기·부모85073/독립233251회 정산을 확인했다. [작업](tasks/failed-breakout-policy-20261022.md), [결과·한계](research/failed-breakout-policy-20261022/report.md), [독립 감사](research/failed-breakout-policy-20261022/independent-audit.json).
  - 실현/잔여/가정 기말 비용·노출을 분리했다. 거래량 조건은 연속6개 손실, 수익 보호 체결38/16(연속/분기), 한도28/0과 분기72개 owned동일을 구분하며 선택값 자동 승격을 기각했다. 완료 승률만으로 판단하지 않고 비중 제한으로 비교 보유42개가 미체결인 점을 보존한다.
  - 후속 연구 조건: 새 관측을 읽기 전에 구간·universe·수량·비용·단독 후보를 고정하고 자료 공급 범위·조정·달력·누락과 gross/net·노출·회전·실현/잔여/기말 비용을 대사한다. 이번 과거 진단을 독립 OOS나 실제 운용 완료로 승격하지 않는다.
  - P2-18 연속3ETF×두비용은 모두 완료청산0회다. 높은 평가이익을 청산 설계 완료로 해석하지 않고 최대보유/수익보호 후보를 사전 고정한다.
- [x] **SOXQ-P2-08 — 강한 종가의 실제 소유·완료 시장일·선택 청산 비교를 완료했다.**
  - v20은 기존3/3·고가 대비 종가 거리·stop0의 활성 의미를 유지하고 실제 수량/원가·잔량·보유 관측/최고가/무장·strict 복원을 연결했다. live 완료 일봉 갱신 누락은 유효 시장일 ticket으로 교정하며 같은 날 재조회·실패/지연·legacy가 소비 조건과 보유 보호를 지우지 않는다. 저장/두 preview·현재/A-B·모바일과 미체결 기원 보호를 fixture/mock으로 검증했다. [작업](tasks/strong-close-policy-20261022.md).
  - 비대상1122개 전체 report/signals·기존 StrongClose102창·새 owned78원장 모두 기본 경제 결과와 신호를 보존했다. 사전 고정30연속/30분할/360별도분기와 부모67393개/독립253595개 정산을 통과했다. 대체 분할 원장의 독립 재정산과 생산 동등성 assertion은 구분한다. [설계·결과·한계](research/strong-close-policy-20261022/report.md), [독립 감사](research/strong-close-policy-20261022/independent-audit.json).
  - 추세200·수익5/되밀림3·보유63은 각각 연속6개 수익이 모두 낮아 기본 승격하지 않았다. 보호 체결89/49·한도46/0(연속/분기), 마지막 분기 한도 신호10개의 미체결과72개 동일 경제 경로를 구분한다. 원본/owned 연속은 완료0·잔량51/2/3으로 보유 평가수익이며, 분기 비교 보유30개는 비중 제한으로0주다. 선택값0·공통 guard를 유지한다.
  - 후속 승격은 새 자료를 읽기 전 구간·수량·비용·단독 가설을 고정하고 실제 자료 범위/조정·달력·시점과 비용·노출·회전·실현/잔여/기말 비용을 대사해야 한다. 이 사후 연구와 날짜 fixture를 미관측 OOS·실제 운용 완료로 취급하지 않는다.
- [ ] **🟠 사용자 검증·자료 대기** **SOXQ-P2-09 — 변동성 확장의 실제 장중 근거 검증.** 테스트 가능한 구현·일봉 비교는 완료했으며 최초 장중 발동·실제 체결 가능성 조건은 열린 상태다. [작업](tasks/volatility-expansion-policy-20261022.md), [사전 설계·비교·한계](research/volatility-expansion-policy-20261022/report.md), [독립 감사](research/volatility-expansion-policy-20261022/independent-audit.json).
  - [x] v21 실제 수량/native 원가·잔량 청산·pending 기원·거절/부분/재설정/복원 보호, 이전N완료 범위와 당일OHLC·0범위/평균0·엄격 초과를 검증했다. live 유효 시장일의 관측 시가/고저·같은날 재조회/실패/latelegacy를 나누고 strict 저장·두 preview·전용 조건 빈도·현재/A-B·모바일을 동기화했다.
  - [x] 비대상1122개·기존102창·owned78원장의 기본 backtest/signals 호환, 고정36연속/36분할/432별도분기·부모70828개/독립249412개 정산과 현재390개 진단을 확인했다. 대체분할36개는 생산 경제 hash·입력연결이며 미저장 대체 원장 독립 재정산과 구분한다.
  - [x] 기간20은 연속4악화/2동일, 배수1.5는4개 개선/2동일이지만 모두 완료0·잔여평가다. 수익 보호/보유63은6개 모두 수익하락·회전증가, cap분기0·72개 동일이며 QQQ hold63은10/20bp +0.1099/−0.4609%다. 기본10/2/손절3·선택0·공통guard를 유지한다. 비교 보유0주36개·준비완료 무신호32/72개를 우수성/자료부족으로 오해하지 않는다.
  - **현재 미완료:** 최초 장중 조건 발동·주문 가능 시각·체결/차단·청산 창의 실제 자료 검증이다. 위 일봉/fixture·현재 UI 진단의 완료를 실제 장중 체결 가능성 확인으로 확대하지 않는다.
  - **진행할 수 없는 사유:** 관측 시가 대용인 live 첫 quote와 일봉 nextOpen은 실제 거래소 시가·봉 내부 순서·유동성을 입증하지 못한다. 해당 구간의 출처/공개 시각·세션·누락/coverage가 검증된 장중 자료가 없다.
  - **필요한 비식별 자료:** 장중 파일 경로 또는 공급원/이용 범위와 시장 timezone·봉 표기·발행/수신 시각·가격 조정·누락 metadata를 제공한다. 실제 체결 가능성까지 판단하려면 그 판단을 뒷받침하는 호가/체결 근거가 필요하며 OHLC만 있으면 그 한계를 남긴다.
  - **재개 조건:** 결과를 보기 전 동일 장중 창·자본·비용·단독 후보를 고정하고 coverage/공개 시각을 검증한 자료를 생산 엔진에 입력한다. 실제 계좌 연결·주문 없이 재현할 수 있는 검증은 그대로 자동 진행한다.
  - [ ] **객관적 완료 기준:** 최초 발동→주문 가능→체결/차단→청산의 시점·실현/잔여·비용/노출을 독립 대조하고, 미확인 장중 순서·유동성은 평가 불가로 분리한다. live 첫 quote/일봉 nextOpen 가정을 실제 장중 증거로 바꾸어 부르지 않는다.
  - 과거45창 중앙값0%·완료8·무완료38·무신호20은 별도 연구 관측으로 보존하며 이번 연속/분기 결과와 합산하지 않는다. P1-03의 완료봉·0범위/이전평균 계약도 유지한다.
- [x] **SOXQ-P2-10 — 평균회귀 실제 소유·단독 추세/청산/보유 후보 비교를 완료했다.**
  - 현재 포함20관측 모집단 밴드·strict 경계와 native stop5(0도 원가 이하 손절)를 유지하고 확정 소유·원가·관측 나이를 분리했다. 선택 middleBand/이전N추세/보유관측의 저장·두 preview·UI/A-B, live 약10초 관측과 일봉 준비 분리, ageunknown 복원·partial·거절·준비 부족 보호를 검증했다. 기존 QQQ1주252+64봉 비용/nextOpen4경로를 유지했다. [작업 기록](tasks/mean-reversion-policy-20261020.md).
  - 비평균회귀1122개 전체 report/signals와 평균회귀102개 backtest/signals가 동일했다. confirmedUpper는 legacy와78/78개 경제 경로가 같고 기존 원장 flatSell/heldBuy도0이므로 신호 감소·수익 개선으로 주장하지 않는다. 최종36연속/36분할/432별도분기와 부모74,968개·독립219,769개 기록 산술(오류0)을 통과했다.
  - 중심선은 연속6개 모두 순평가수익 하락·회전 증가, trend200은 SPY/QQQ 일부 손실 감소이나 SOXQ20bp6.9134→−3.7217%다.63관측은 발동0·최장55로 효능 평가 불가다. 이를 본 뒤 고정한21관측 사후 탐색은 QQQ 소폭 개선/SOXQ·SPY 악화, 최저 완료손실 동일과 guard 뒤22/24관측 재시도를 기록했다. upperBand/선택옵션0을 유지하며 종가 손절·보유 관측을 장중 손실/실제 보유 상한으로 부르지 않는다. [사전 선언·사후 탐색·실현/잔여·비용/노출·기각 근거](research/mean-reversion-policy-20261020/report.md).
  - 우선 회귀 근거: P2-18 SPY연속 +4.2519%→+0.0835%, 실제 매도2025-06-27↔04-09·동일경로 비용복원 +4.9703%→+1.5065%로 경로 자체가 달라진다. QQQ20bp −1.8039%/MDD4.5823%도 사전 고정 검증 구간으로 보관한다.
- [ ] **🟢 자동 구현** **SOXQ-P2-11 — 추세 필터:** 중앙값 +0.4886%, 최악 -4.1022%/MDD5.5534%, 완료22회 모두 손실·기말 보유39/45창. 최근248봉 SOXQ +9.6300%도 완료0회 평가손익이다. 완료 조건: 실제 청산이 있는 복수 구간에서 장기 추세 이탈·수익 보호·최대 보유 후보와 노출을 비교하고 실현/잔여 손익을 분리; 200봉 기준 유지.
  - P2-18 SOXQ연속20bp 총+9.5760%지만 실현−240553.37/잔여+1198152.86원·MDD16.2171%·평균투자21.2574%다. 실제 청산/수익보호·노출 비교를 함께 요구한다.
  - 사전 설계 준비: 원래200/5/20·현재 포함/strict·확인 기본1을 유지하고 legacy/owned/이탈확인2/수익5·되밀림3/보유63의 단독5종·30연속/30분할/360별도분기를 고정했다. [설계](research/trend-filter-policy-20261023/design.json), [연구 계약](research/trend-filter-policy-20261023/protocol.md), [작업 기록](tasks/trend-filter-policy-20261023.md).
  - [기존 원장 감사](research/trend-filter-policy-20261023/existing-ledger-audit.json)는 저장된v12 연속6/분기72에42,958검사·오류0, 두 재실행JSON동일·원본연결hash·변조4건 탐지를 확인했다. 각 비용 연속완료19회(이익5/손실14), 별도분기완료16회 모두손실·기말보유30/36이며 비용별비교0주3개·실현/잔여·노출을 구분한다. 현재81fa00d/v21 재실행이나 새 후보 결과가 아니며 미저장 대체분할 원장/benchmark guard결정/당시바이너리는 미검증이다. 구현·새 생산 비교·효능 판정은 계속 미완료다. [6연속 표·반올림 한계·우선순위·후속 조건](research/trend-filter-policy-20261023/report.md).
- [ ] **🟠 사용자 검증·자료 대기** **SOXQ-P2-12 — 레버리지 추세 보유(LTH):** 최근 실제 분봉2,318개/7날짜의 기본·볼린저 활성×비용2종 네 진단은 신호0이다. null OHLCV412개·정규장 밖1개는 제거했고 합성하지 않았다. 고정KST/auto 세션·봉 종료 표기의 시작·종가 공개 가능 시각 모델·10초 관측 차이·누락/기간 coverage 미검증이라 0% 성과/볼린저 효과 없음으로 해석하지 않는다. 완료 조건: P1-05 실제3개월 원본·DST/거래일/세션·gap·진입/마감 창 검증 후 옵션별 비교. generic 거부·일봉 평가 불가 유지.
  - **현재 미완료:** P1-05의 검증된 실제 3개월 분봉을 사용한 기본/볼린저 옵션별 생산 비교 결과다. 짧은 표본의 신호0과 시간 진단 fixture는 옵션 효과의 근거가 아니다.
  - **진행할 수 없는 사유:** 검증된 전체 기간 자료·시장 달력/세션·공개 시각과 실행 시계의 일치가 확보되지 않았다. P1-05의 자료 준비와 이 항목의 옵션별 비교는 서로 다른 결과물이다.
  - **필요한 비식별 자료:** P1-05의 원본 경로·출처/이용 범위·시각/조정/coverage metadata를 공유한다. 같은 자료를 중복 요청하거나 별도 계좌 credential을 요구하지 않는다.
  - **재개 조건:** P1-05의 자료·시간 검증을 마친 뒤 결과를 보기 전 옵션·창·수량·비용을 고정하고 실제 장중 경로에서 비교한다.
  - **객관적 완료 기준:** 기본/볼린저 각 결과의 신호·체결/차단·완료 표본·실현/잔여·MDD·회전·노출을 독립 대조하고 무신호/자료부족/평가불가를 구분한다. generic LTH 거부·일봉 notEvaluable를 유지하고 짧은 신호0을 효과 없음으로 단정하지 않는다. [표본 연구](research/strategy-review-20261018/report.md), [시장 시각 진단의 한계](tasks/lth-minute-timing-20261022.md).
- [x] **LTH 분봉의 거래소 시각·세션·미관측 슬롯 진단을 구현했다.**
  - `lth-minute-timing-v1`은 원본 실행 label의 UTC/시장 timezone·DST offset, 봉 종료/시작 구분, 중복/역순/분경계/asOf 뒤 종가와 관측일 내부 격자 공백을 bounded512행·64byte·날짜8/예시6개로 진단한다. current/A-B/복원·legacy/손상 형식·일봉/빈자료·모바일을 fixture/mock으로 검증했다. 기존 고정 KST 시계·신호·정산·입력hash와 sampleOnly/notEvaluable·lookAheadSafe=false는 유지한다. [구현·검증](tasks/lth-minute-timing-20261022.md).
  - 일반 평일 시간창과 실제 달력·발행/수신·공급원/요청기간 coverage는 다르다. P1-05/P2-12 실제3개월 자료와 운용 검증은 계속 미완료다.
- [ ] **🟢 자동 구현** **SOXQ-P2-13 — 가격조건:** 전일 종가 고정·익절5%·손절3% 연구 설정에서 중앙값 +0.2847%, 완료117회, 비용 부호 반전3창. 완료 조건: 평가 전 고정 기준가 유지, 손절 직후 재진입·공통 cooldown/최소 기대수익·회전율을 두 비용과 복수 구간에서 비교; guard 완화나 사후 최적화 금지.
  - **기존 원장 근거:** P2-18의 2023-01-03~2025-12-31 nextOpen 연속 평가는 비용별 3건·완료매도 3회·rawStopSignals 0·완료손절 0·guard 차단 0·기말보유 0이다. 별도 분기 초기화는 비용별 36건·완료매도 3회·33건 완료청산 없음·손절/차단 0이다. 원래 45창 sameClose 통계와 합산하지 않는다. [고정 설계](research/continuous-replay-20261018/design.json), [연속 원장](research/continuous-replay-20261018/continuous-results.jsonl.gz), [별도 분기 원장](research/continuous-replay-20261018/reset-results.jsonl.gz), [요약](research/continuous-replay-20261018/summary.csv).
  - **평가 범위:** 2022-12-30 기준가 SOXQ 19.91/SPY 382.43/QQQ 266.28, 수량 51/2/3, native 익절 5%·손절 3%, 수수료와 슬리피지 각각 10/20bp를 고정했다. 평균 초기자본 비중 0.0806/0.2430/0.1259%의 희소노출이며 손절 후 재진입·cooldown·최소 기대순익 gate의 효과를 이 표본으로 평가할 수 없다. 자료 준비 실패·코드 오류·전략 효능 0%라는 뜻이 아니다. 비교 도중 분기별 기준가 변경·사후 조정·guard 완화를 하지 않는다.
  - **구현 경계:** PC native 손절은 일반 TradeGuard 검사 대상이고 연구 overlay는 TradeGuard를 우회하는 별도 경로다. exact owner 제출수량 clamp가 있으므로 내부 bool 보유 모델만으로 초과매도 사고를 단정하지 않는다. 실제 소유 수량·native 평균 원가·부분보유/설정수량 차이·거절 피드백과 두 경로를 분리해 검증한다. [PC native 신호](../src-tauri/src/trading/strategy/price_condition.rs), [exact owner 제출수량](../src-tauri/src/trading/order/submission/owned.rs), [연구 정산·overlay 경로](../src-tauri/src/trading/simulation/session.rs).
  - **남은 완료 조건:** 위 고정 기준가·두 비용·공통 guard를 유지한 채 손절 접수 전후·차단된 손절·같은 날/다음 날·DST·계좌 분리 fixture를 추가하고 부분 체결/보유·거절·실제 native 평균가와 재진입/회전·순손익을 대조한다. 분 단위 cooldown/재진입 효능은 일봉만으로 입증하지 않으며 fixture 검증과 실제 장중 자료의 평가 범위를 구분한다.
  - **기존 검증 범위:** `cargo test --offline --workspace --lib price_condition --locked` 8개 통과·0개 실패·781개 제외는 기존 동작 검증이다. 위 새 조건의 구현·연구 결과나 P2-13 완료 증거가 아니며 이 항목은 자동 구현으로 열린 상태를 유지한다.
  - **해석 위험·검증 의존성:** 손절이 미발동한 일봉 표본을 재진입/손절 효능으로 확대하는 해석을 피한다. 생산 Rust fixture와 시각/시장일·체결 정보가 확인된 장중 자료가 검증 근거이며, fixture 완료와 실제 자료 검증의 범위를 구분한다.
- [x] **SOXQ-P2-14 — 체결 가능성과 장중 손절의 연구 모델을 분리했다.**
  - v10 공통 ReplaySession의 sameClose/nextOpen, 종가 신호→다음 관측 시가·이전 ATR→실제 원가 피드백, 마지막 미체결, signalAvailableAt/fillAt의 봉/단계를 구현했다. 연구 보호0 기본, 갭 시가·OHLC 양방향 접촉 순서 불명·손절 우선 가정을 UI와 A/B에 표시하고 실제 체결 시각/유동성 검증으로 표현하지 않는다.
  - 직전 ba74e03 생산코드1,224회와 기본 모드의 전체 회계 report/신호가 일치했다. QQQ2026Q3 평균회귀의 비용·체결가가 청산을 바꾸는 fixture와 강한 종가 포함 실자료8회/benchmark8회, 합성 갭·미체결·순서 불명, 일반/LTH 피드백·공통 UI를 검증했다. [작업·검증](tasks/replay-execution-model-20261018.md), [독립 분석](research/replay-execution-20261018/report.md).
- [x] **SOXQ-P2-15 — 비용·배당·환율·청산 가정과 비교 기준을 구현했다.**
  - v11의 방향별/최소 수수료·수동 출처, 평가봉에 정확히 매칭한 FX/배당·조정가격 중복계상 거절, native 진입평균/역사적 KRW 원가 보존을 구현했다. 실제 청산 전망을 최소fee 기대수익 guard와 정산이 공유하며 손절/보호/native손실 예외를 유지한다.
  - 실현+잔여 순평가+세후배당=자산증감, 가격/FX 귀속과 report-only 기말청산, 관측종가 평균/최대 자본비중·보유봉비율 구분을 제공한다. 같은 요청수량 단순보유의 실제 체결량/노출차이와 동일 실제경로 비용환급 산술참조를 구분하고 소표본·역부호·부분/겹침창 경고를 표시한다. UI/A/B/legacy/LTH 평가불가 gate를 함께 검증했다.
  - 직전1192673 생산코드1224회 전체report/신호 기본호환, QQQ 단일분기40개 합성 시나리오/120개 항등식과 기록fill 독립4540대사를 통과했다. 강한종가 nextOpen20bp는 평가+0.0039%지만 가정청산−0.0390%다. 실제 요율·배당·FX/장기 운용 검증으로 확대하지 않는다. [작업·검증](tasks/replay-accounting-20261018.md), [연구·한계](research/replay-accounting-20261018/report.md).
- [ ] **🟢 자동 구현** **SOXQ-P2-16 — ATR을 true range로 계산하고 표본 부족을 성과와 구분한다.**
  - v8 replay는 이전 종가 포함 TR14·동일 interval 사전 ATR warmup·준비 진단을 구현했다. 연구 UI는 기본 비활성 수량 산정과 위험bps/배수를 제공하고 부족·0범위·0bps 매수 차단을 표시한다. LTH는 실제 잔여수량/평균가 피드백으로 확대·축소·부분매도 보호를 검증했다. 최신45창의 작은 완료 표본과 자산/기간 상관·무거래 창은 여전히 구분해야 한다.
  - 완료 조건: 이전 종가 포함 TR·사전 ATR warmup·봉 간격 유지; 표본 수/신뢰 부족 표시; 70/30 구간을 기간 정규화 없이 과최적화 확정에 사용하지 않고 별도 walk-forward 평가.
  - **현재 판정 근거:** 같은 ReplaySession 상태가 이어지는 70/30은 분리 학습·독립 OOS가 아니다. `phase_summaries`는 완료매도가 체결된 phase에 전체 배분 원가의 손익을 귀속한다. `overfit_warning`은 전반 수익>0에서 후반<=0 또는 후반<전반×0.5만 검사하며 구간 길이·완료 수를 보지 않는다. session의 최종 판정식을 continuous 경로는 유지하고 문구만 완화한다. [구간 집계·판정식](../src-tauri/src/trading/simulation.rs), [최종 호출](../src-tauri/src/trading/simulation/session.rs), [연속 문구 치환](../src-tauri/src/commands/strategy_preview/continuous.rs).
  - **기존 원장 진단:** v12 연속 72원장은 526/226봉이며 후반 양수·전반 절반 미만 26건, 전반 양수·후반 음수 16건/정확히 0인 6건, 한쪽 완료청산 0인 32건이다. 별도 분기 864원장은 각각 52/132/53/786건이다. SOXQ PC 10bp의 2023Q1은 전반 +0.7526%·완료1/후반 0%·완료0인데 flat 경고는 후반을 “검증 구간은 손실”로 표시한다. 비용쌍·상관 ETF·겹치는 창은 독립 표본이 아니다. 앞선 감사와 동일 SHA의 기존 원장이며 새 v22 실행 결과가 아니다. [연속 원장](research/continuous-replay-20261018/continuous-results.jsonl.gz), [별도 분기 원장](research/continuous-replay-20261018/reset-results.jsonl.gz), [고정 설계](research/continuous-replay-20261018/design.json).
  - **남은 자동 구현 우선순위:** 누적수익 절반 조건으로 과최적화·학습/OOS를 단정하는 경고와 0%=손실 문구를 교정한다. 같은 finish를 쓰는 LTH 전용 preview를 포함해 flat/LTH/continuous·현재/A-B·legacy의 구간 길이·완료청산 표본·연속 상태 설명을 일치시킨다. 현재 flat/LTH UI의 “학습 구간(%) / 나머지는 out-of-sample 검증”·학습/In-sample/Out-of-sample 제목·표와 continuous의 앞/뒤·독립검증 아님 표현을 함께 정리한다. `BacktestPhaseView`/`BacktestPhase`에 관측수·노출·실현/잔여·경계보유 정보가 없으므로 필요한 설명/진단 계약을 보완하되 기존 회계·체결·원시 구간값은 보존한다. 길이가 다른 구간·후반0/손실·완료0/부분청산 fixture와 기본 거래/report·회계 불변을 검증하고 기존 TR·사전 ATR warmup·봉 간격 계약 및 LTH 평가불가/자료 gate를 유지한다. [현재 연구 UI](../src/pages/strategy/ui/strategyResearchPanel.tsx), [구간 TS 계약](../src/shared/api/strategyResearchTypes.ts).
  - **별도 walk-forward 완료 기준:** 생산 Rust API로 자료를 읽기 전 창·이전 정보·고정 후보·두 비용·설정 선택·warmup·수량·경계 보유 처리와 비중/노출·실현/잔여·MDD·차단·회전 기준을 선언한다. 과거 자료 재검토를 새 OOS로 부르지 않으며, 이어지는 장부의 사후 70/30과 분리한다.
  - **해석 위험·검증 의존성:** 경고 없음을 신뢰 충분으로, 청산0을 성과 우수로 해석하지 않는다. 생산 Rust 구현·fixture와 검증 가능한 자료·사전 선언에 의존하며, 로컬 자동 구현/검증에 새 사용자 승인 게이트를 추가하지 않는다.


- [x] **SOXQ-P2-18 — 연속 운용 검증의 포지션·guard·risk 상태를 보존했다.**
  - v12 공통driver에서 factory/warmup/ReplaySession/finish각1회, native원가·pending·ATR·쿨다운·시장날짜/연속손실 상태를 청크 사이에 유지한다. 기존flat합계500을 보존하고 새bounded입력(16청크/각500/평가4096/history512), strict시각검증·미래prefix/단일·분할회귀·IPC/REST/모드/A-B/UI자료수집을 연결했다.
  - 직전cc6f5d0 생산1224회 전체report/신호 호환,3ETF×12전략×비용2의3년72회/alternatepartition72회/별도분기864회와 root136717·독립143866 산술체크를 통과했다. 최초전일결정수량·PC기준가/params를 고정하고 실제 accepted 수량의 갭/장중 시장가격 경로를 비용환급과 대사했다. 분기계좌 수익률 합산/복리는 없다.
  - 강한종가·돌파실패의 높은 평가수익도 청산0회, SOXQ일부분기 단순보유72개는 비중한도로0주다. 같은요청수량을 동일실제노출/알파비교로 표현하지 않는다. UI의7일경계는 보수적자료gate이며 달력검증이 아니고 LTH시간/coverage·별도walk-forward는 미완료 유지. [작업·검증](tasks/continuous-replay-20261018.md), [설계·실자료·한계](research/continuous-replay-20261018/report.md).


## 자동매매 예산·볼린저 보강 검증 기록 (2026-09-11)

- [x] 계좌/통화별 전용 예산, 자동 소유수량, durable 예약·체결·완료 복원, 설정 UI와 E2E 추가.
- [x] 기존 LTH에 기본 비활성 볼린저 필터와 중심선/ATR 청산, 공통 live/replay 및 미리보기 설정 연결.
- [ ] **🟢 자동 구현** 실제 provider 응답 불명 주문의 운영자 대조/복구 UI: 현재는 자동 해제 없이 예산을 보류한다. 주문번호/clientOrderId 대조 및 멱등 승인 흐름으로 확장할 것.
기존 열린 제목 **“LTH 보강의 일봉 성과 미리보기”**는 별도 일봉 성과 기능이 아니다. 일봉 평가 불가·분봉만 성과 표본으로 허용하는 구현 근거와 실제 장중 자료/시장 시간 미완료 조건이 P1-05와 같아 **SOXQ-P1-05에 통합**했다. 미완료 조건을 닫거나 제거하지 않았으며, 활성 볼린저의 옵션별 비교 결과는 **P2-12**에 별도로 유지한다. [원래 일봉/분봉 검증 근거](research/soxq-3m-20261003/p1-05-validation.md).

## P3 — 유지보수·품질 게이트

- [ ] **🟢 자동 구현** 1,000라인 초과 파일을 책임 단위로 분리한다.
  - [x] `src-tauri/src/commands/trading.rs`: lifecycle469/positions329/balance96/risk_restore96/daemon527/history281행으로 분리. IPC4개·scope·복원/시작 순서 보존, lib212개와 독립 본문 감사 통과. [기록](tasks/trading-module-split-20261018.md).
  - `src/pages/trading/ui/Page.tsx`: broker별 orchestration과 공통 order form 분리.
  - [x] `src/api/hooks.ts`: 19행 facade와 account/market/order/strategy/settings/history/trading/log 모듈49~255행으로 분리, 공개export87개/선언74개 동일. [기록](tasks/frontend-hooks-split-20261018.md).
  - `src-tauri/src/commands/toss.rs`: diagnostic/preflight/orders surface 분리.
  - [x] `src-tauri/src/trading/order/submission.rs`: positions/prepare/provider/records와 run 회귀로 분리. 예약·scope·manual API 경계를 검증했다. [기록](tasks/execution-run-reservation-20261018.md).
  - `src/pages/strategy/ui/leveragedTrendHoldEditorPanel.tsx`: ticker picker/preview UI 분리.
  - [x] `src-tauri/src/trading/order.rs`: facade·health·pending·legacy·execution run으로 분리하고 1050행 fills도 적용/collector/reconciliation/provider mode로 나눴다. [기록](tasks/execution-run-reservation-20261018.md).
  - 검증: 변경 파일과 신규 파일은 1,000라인 아래, FSD/API public surface와 IPC 이름은 유지한다.

- [x] 핵심 거래 흐름의 자동화 테스트를 release gate로 승격한다.
  - broker mock 기반 제출→부분체결→완전체결→취소/거부→재시작 복구 테스트를 추가한다.
  - balance fail-closed, scope 전환, midnight rollover, manual/auto parity, 인증 없는 REST 거부를 포함한다.
  - `cargo check`, `cargo test`, `npx tsc --noEmit`, `npm run check:fsd`, `npm run test:e2e`, OpenAPI 검증을 CI에서 실행한다.

- [x] 의존성·릴리스 보안 점검을 자동화한다.
  - `cargo audit`와 npm audit 정책, Dependabot/Renovate, lockfile 검증을 CI에 추가한다.
  - release artifact 서명/해시와 updater 경로를 검증하고, 실패 시 사용자가 확인할 수 있게 한다.
  - `release-gate.yml`, `security.yml`, Dependabot, 추적 lockfile 검증, SHA-256/attestation 및 detached signature 준비 검증으로 반영했다. 실제 플랫폼 코드 서명/updater 서명 강제는 배포 키 준비 후 `REQUIRE_RELEASE_SIGNATURES=true`로 전환한다.

- [x] 사용자 가이드를 broker별 실제 지원 범위와 일치시킨다.
  - KIS 전용 소개를 KIS/Toss 공통 기능, broker별 주문·시세·자동매매 제한 표로 교체한다.
  - 실제 UI의 13개 전략 카드, 프로파일 설정, 세션/캔들 source, 소액 실거래 gate를 기준으로 오래된 절차를 정리한다.

- [ ] **🟠 사용자 판단 대기** 플랫폼 코드 서명·updater 서명 강제를 위한 배포 키 준비.
  - **현재 미완료:** 대상 플랫폼의 실제 코드 서명·updater 서명 생성과 서명 강제 release artifact 검증이다. checksum/attestation·sidecar 존재/비어 있지 않음·강제 옵션 검사는 구현했으며 암호학적 서명 검증이나 실제 서명 릴리스와 다르다.
  - **진행할 수 없는 사유:** 대상 플랫폼·키/인증서 관리 주체와 GitHub Environment의 서명 준비가 확정되지 않았다. 없는 키를 생성/등록한 것으로 가정하거나 서명 강제 변수만 먼저 켤 수 없다.
  - **필요한 사용자 판단:** 배포 플랫폼·서명 주체·키 보관/회전 정책과 승인할 release 환경을 정하고 관리자가 해당 환경에 키를 준비한다. 에이전트에는 비밀값 대신 준비 여부·키 식별용 비밀 아닌 metadata만 제공한다.
  - **재개 조건:** 환경의 서명 생성 단계와 필요한 키가 준비되면 별도 승인된 release 절차에서 `REQUIRE_RELEASE_SIGNATURES=true`를 적용한다. 로컬 verifier·mock의 자동 검증은 계속 가능하다.
  - **객관적 완료 기준:** 대상 artifact의 유효한 플랫폼/updater 서명을 확인하고 서명 누락·변조·불일치가 release를 차단함을 검증한다. checksum/attestation을 코드/updater 서명의 대체물로 취급하지 않는다. [현재 게이트와 제한](release-security.md).

### 보류한 호환 업데이트 후속 작업 (2026-10-24 검토 기준)

아래 현재→검토 버전은 2026-10-24 직전 읽기 검토값이며 최신 버전의 영구 고정값이 아니다. 각 작업 착수 시 공식 릴리스·레지스트리·보안 공지와 열린 Dependabot PR #13·#11의 현재 상태를 재확인하고 중복 변경을 피한다. [적용·보류 및 기존 검증 정본](security-mac-handoff-2026-10-06.md), [잔여 RSA·보안 정책 정본](release-security.md)을 따른다. 아래 작업은 모두 미완료이며 이번에는 문서만 기록했다.

권장 순서: **DEP-P3-01 → 02 → 03 → 04 → 05 → 06**, 기타 후보는 **07·08**에서 개별 판단한다. **DEP-P3-01의 MSRV·격리 DB 지원정책 결정은 DEP-P3-02 SQLx 전환의 실제 선행조건**이다. DEP-P3-03~08의 “다음/이후”는 검토 우선순위이며 앞 번호 전체 완료가 기술상 필수라는 뜻은 아니다. 각 항목의 실제 API·peer·toolchain 의존성이 확인되면 그 근거에 따라 순서를 조정한다. 우선순위는 이 P3 목록 내부 순서이며 기존 P1/P2를 대체하지 않는다. 검증은 금융 자격증명·실거래 연결 없이 합성/mock·격리 fixture로 한다. 운영 DB·거래 데이터·실행 중 서비스와 주문/취소/포지션·엔진 재시작·배포를 건드리지 않는다. Xcode 27.0→27.1은 사용자 보류 상태를 유지한다.

- [ ] **🟠 사용자 판단 대기** **DEP-P3-01 — 격리 DB 검증환경과 프로젝트 MSRV 기준을 결정한다.** 우선순위 1, SQLx 전환의 선행조건.
  - **확인:** 프로젝트 선언 MSRV 1.90.0→SQLx 0.9.0 요구 최소 1.94.0 결정이 필요하다. Mac Rust 1.99.1은 이미 요구를 충족한다. 기존 갱신 검증은 1.99.1 결과이며 1.90 실행 검증은 하지 않았다. DB 계약 5개는 환경 미설정으로 즉시 반환했으므로 실제 DB 통과가 아니다.
  - **현재 미완료:** SQLx 0.9 전환에 맞는 프로젝트 최소 Rust와 격리 PostgreSQL/MariaDB 지원 matrix·인증 정책의 확정, 해당 최소 버전/stable과 실제 DB 계약 실행이다. 기존 PostgreSQL16 로컬 검증과 2026-10-24 환경 미설정 DB 계약 5개의 즉시 반환을 구분한다.
  - **진행할 수 없는 사유:** 선언 MSRV 1.90과 SQLx 0.9 요구 1.94가 다르며 기존 MariaDB fixture 보류·비TLS 인증 지원의 정책을 임의로 변경할 수 없다. 개발 Mac 1.99.1의 성공만으로 최소 지원 버전을 검증한 것은 아니다.
  - **필요한 사용자 판단:** 최소 Rust를 1.94 이상으로 올릴지, 격리 DB 버전/인증 범위와 MariaDB 보류를 어떻게 다룰지 확정한다. 운영 계정/자료 없이 합성 전용 접속정보를 사용한다.
  - **재개 조건:** 결정된 지원 기준을 선언·CI에 반영하고 임시 DB fixture의 실제 실행·skip 감지를 연결한 후 DEP-P3-02 전환을 진행한다. 결정 전에도 로컬 검증 설계·호환 영향 조사·독립적인 다른 업데이트 검토는 가능하다.
  - **객관적 완료 기준:** 선택한 최소 Rust와 stable의 check/test, 지원 DB matrix별 실제 계약 실행 건수와 성공/거부를 기록한다. 즉시 반환·미실행을 통과로 세지 않고 schema/transaction·TLS·인증·복구를 확인한다. RSA 해소 판정은 별도 audit/의존성 근거로만 한다. [2026-10-24 검증·보류](security-mac-handoff-2026-10-06.md), [정책·RSA 잔여](release-security.md).

- [ ] **🟢 자동 구현** **DEP-P3-02 — SQLx 0.8.6→0.9.0의 SQL·DB 인증·잔여 RSA를 분리 검증한다.** 우선순위 2, DEP-P3-01 완료 후 전환.
  - **실제 코드 영향 확인:** `storage/database_schema.rs`·`database_projection.rs`의 동적 `query(&query)`/`query(&format!(…))`는 새 SqlSafeStr/AssertSqlSafe 경계 검토·수정 대상이다. 0.9의 비TLS MySQL RSA 인증은 `mysql-rsa` 기능을 별도 요구한다. PostgreSQL/MariaDB 저장·타입/문자열·transaction·실패 복구도 재검증한다. 이 경로는 모두 `src-tauri/src/` 아래다.
  - **완료 조건:** 허용된 테이블 식별자와 bind 경계를 보존하고 격리 DB의 schema/import/export·transaction rollback·TLS 및 지원 인증 계약을 통과한다. `cargo fmt --all -- --check` 및 `--locked` check/test/strict clippy/build와 최소 Rust/stable CI를 확인한다. RUSTSEC-2023-0071은 별도 dependency tree/audit로 재평가하고 SQLx 상승만으로 해소됐다고 처리하지 않는다. 기존 비TLS 지원 제거·인증 정책 변경은 별도 사용자 결정 없이 하지 않는다. [공식 0.9 변경](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/CHANGELOG.md).

- [ ] **🟢 자동 구현** **DEP-P3-03 — reqwest 0.12.28→0.13.5 및 tokio-tungstenite 0.24.0→0.30.0을 각각 전환한다.** 우선순위 3, DEP-P3-02 다음; 두 전환의 diff·검증은 분리한다.
  - **실제 코드 영향 확인:** Cargo의 `rustls-tls`→`rustls` 및 명시 `query` 기능, `api/rest/chart.rs`의 `.query()` 사용을 반영한다. `api/websocket.rs` 승인·구독의 `Message::Text(String)`는 Utf8Bytes 변환 대상이다. TLS 루트/crypto provider·수신/종료 payload와 버퍼 기본값은 별도 호환 확인 대상이며 실제 실패를 재현한 것은 아니다.
  - **완료 조건:** 합성 HTTP/TLS 서버에서 차트 query·헤더·JSON·쿠키·오류 처리를 확인하고 mock WebSocket에서 승인·구독·수신·ping/pong·종료·재연결을 검증한다. 실제 KIS/Toss 인증 없이 Rust format/strict clippy/check/test/build와 해당 CI를 통과한다. [reqwest 변경](https://github.com/seanmonstar/reqwest/releases/tag/v0.13.0), [WebSocket 변경](https://raw.githubusercontent.com/snapview/tungstenite-rs/v0.30.0/CHANGELOG.md).

- [ ] **🟢 자동 구현** **DEP-P3-04 — axum 0.7.9→0.8.9와 tower-http 0.5.2→0.7.1의 서버 호환성을 검증한다.** 우선순위 4, DEP-P3-03 다음.
  - **확인/미검증 구분:** `server/mod.rs`의 `/:symbol`·`/:id`는 새 `/{symbol}`·`/{id}` 문법으로 바꿔야 한다. tower-http는 현재 CORS 사용 경로의 별도 후보이며 프로젝트에서 구체적 API 실패를 확인하지 않았다. 이미 존재하는 간접 tower-http 0.6.11을 직접 의존성 전환 완료로 취급하지 않는다.
  - **완료 조건:** 주문·계정 provider를 mock으로 격리하고 기존 모든 route의 경로/상태·인증 거부·CORS·오류 응답·Tauri IPC와 REST view 계약을 회귀 검증한다. Rust format/strict clippy/check/test/build 및 CI 통과. [axum 변경](https://github.com/tokio-rs/axum/blob/axum-v0.8.0/axum/CHANGELOG.md).

- [ ] **🟢 자동 구현** **DEP-P3-05 — React 18.3.1→19.3.1·MUI 6.5.0→9.5.0을 단계적으로 이전한다.** 우선순위 5, DEP-P3-04 다음.
  - **실제 코드 영향 확인:** 대시보드·전략의 MUI `Grid item xs`는 `size` API로, 전략 입력의 `InputProps`는 `slotProps.input`으로 이전 대상이다. React는 이미 createRoot/새 JSX 변환을 사용한다. React DOM·타입/ref·오류 처리·주변 라이브러리 호환은 미검증 후보이며 이번에 React 교체 실패를 확인한 것은 아니다.
  - **완료 조건:** React/DOM/타입과 MUI 관련 패키지의 지원 조합을 공식 가이드로 정하고 분리된 변경으로 타입·FSD·production build를 통과한다. 금융 mock E2E에서 전략/설정/대시보드·좁은 화면·입력/메뉴/버튼·테마·차트를 검증하고 Mac/Linux 결과를 구분한다. [React 이전](https://react.dev/blog/2024/04/25/react-19-upgrade-guide), [MUI 이전](https://mui.com/material-ui/migration/upgrade-to-v9/).

- [ ] **🟢 자동 구현** **DEP-P3-06 — Vite 6.4.5→8.3.5·React plugin 4.7.0→6.1.2의 빌드 호환성을 검증한다.** 우선순위 6, DEP-P3-05 다음.
  - **확인/미검증 구분:** Rollup/esbuild→Rolldown/Oxc 전환에 맞춰 `vite.config.ts`의 `minify: 'esbuild'`, MUI/Emotion 등의 manualChunks·Tauri pre-bundling 제외 설정을 검토한다. 현재 설정이 모두 실패한다고 입증한 것은 아니다. 명시된 WebView target을 기본 browser target 변화와 혼동하지 않는다.
  - **완료 조건:** 공식 단계별 migration과 Node/plugin 지원 범위를 확인하고 production chunk 초기화·Tauri custom-protocol native build·WebView 호환을 금융 연결 없이 검증한다. 타입/FSD/build·production UI E2E와 source harness를 통과하며 경고 억제나 크기 제한 완화로 대체하지 않는다. 안전하게 앱을 기동할 수 없으면 해당 런타임 검증은 미완료로 남긴다. [Vite 이전](https://vite.dev/guide/migration).

- [ ] **🟢 자동 구현** **DEP-P3-07 — 나머지 Rust·UI 의존성 후보의 실제 필요와 영향부터 확인한다.** DEP-P3-06 이후 개별 우선순위를 정한다.
  - **미검증 후보:** keyring 3.6.3→4.2.0(DB 암호 저장·mock), sha2 0.10.9→0.11.1(체크섬/이벤트 식별값), thiserror 1.0.69→2.0.21(오류 derive), react-grid-layout 1.5.4→2.3.0(새 API/내장 타입 및 legacy 호환 경로). 모두 확정 대규모 수정으로 판단한 것은 아니다. 이미 간접 thiserror 2.0.21이 있어도 직접 전환 완료는 아니다.
  - **완료 조건:** manifest/실제 참조·공식 breaking change·지원 target을 대조해 각 후보를 전환/유지/제거 검토로 결정한다. react-grid-layout은 당시 `src` 직접 import가 없었으므로 사용·CSS·동적 참조를 먼저 확인하며 이번에는 삭제하지 않는다. 전환 시 keyring은 mock만, sha2는 기존 fixture hash 불변, 오류 타입은 변환/메시지 회귀, UI는 사용 경로의 타입/build/E2E를 검증한다.

- [ ] **🟢 자동 구현** **DEP-P3-08 — TypeScript·패키지매니저 major 후보를 개별 평가한다.** DEP-P3-06·07 검토 이후, 필요성이 확인된 항목만 전환한다.
  - **미검증 후보:** TypeScript 5.9.3→7.0.2, Mac 기본 npm 11.20.0→12.2.1, pnpm 10.30.3→12.15.0. 프로젝트 Node 24.22.1 LTS의 npm은 11.19.1이며 기본 Node의 npm과 구분한다. 프로젝트는 npm을 사용하고 pnpm은 이번에 채택하지 않았다. 개별 프로젝트 실패는 미확인이다.
  - **완료 조건:** 공식 compiler/Node engines·peer 지원과 설치/lockfile 변화를 확인한다. TypeScript는 strict 타입·FSD/build, npm은 금융 파일 없는 깨끗한 snapshot의 `npm ci --ignore-scripts`·lockfile 재현·감사·CI를 통과한다. pnpm은 프로젝트 사용 필요부터 확인하고 불필요하면 미채택 결정을 남기며, 전환·lockfile 병행을 자동 도입하지 않는다.

## 완료 기준

- 각 항목은 코드, 실패/복구 테스트, 사용자-visible 상태, 관련 문서가 함께 반영되어야 닫는다.
- 실계좌 검증은 모의/fixture → read-only 계좌 조회 → 명시 승인된 소액 주문 순서를 지킨다.
- 경고를 포함해 `cargo check`와 TypeScript 검증이 깨끗해야 하며, 주요 UI 배치 변경은 Playwright로 확인한다.
