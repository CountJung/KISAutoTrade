# KISAutoTrade — Todo

> 완료 이력은 `git log`와 릴리스 노트에서 관리한다. 이 문서에는 아직 끝나지 않은, 검증 가능한 작업만 둔다.
> 우선순위는 `P1 정확성/신뢰성 → P2 전략 연구 UX → P3 유지보수` 순이다.

## *마지막 비판적 점검: 2026-10-22*

[색상·필터가 있는 할일 보기](todo.html) · 원본은 이 문서다. 수정 후 `npm run todo:render`, 동기화 확인은 `npm run check:todo`를 실행한다.

- **🟢 자동 구현**: 추가 사용자 판단 없이 구현·fixture·연구 검증을 진행한다. 선행 과제가 있으면 순서대로 해결하며 어려움만으로 대기로 바꾸지 않는다.
- **🟠 사용자 판단 대기**: 제품 범위·기존 보류 해제·배포 키 등 명시적 선택이 필요하다. 항목의 필요한 결정을 먼저 확인한다.
- **🟠 사용자 검증·자료 대기**: 실제 자료 제공·계좌 접근·별도 승인된 검증이 필요하다. 로컬 fixture와 자동 구현 부분은 계속 진행한다.

주황 표시는 미완료 상태다. 테스트·문서·독립 검토가 완료된 항목만 체크하며, 실제 검증이 남은 항목을 자동 완료하지 않는다.

## P1 — 진행 가능한 잔여

### 현재 전략 전수 연구 후 권장 순서 (2026-10-18)

[전담 분석 보고서](research/strategy-review-20261018/report.md) · [실행별 근거](research/strategy-review-20261018/results.jsonl) · [요약](research/strategy-review-20261018/summary.json). 등록 13종 중 일봉 12종 1,224회, LTH 실제 분봉 표본 4회 진단을 실행했다. 주집계는 SOXQ/SPY/QQQ의 2023Q1~2026Q3 완전 45창이다. 고정된 전일 기준 15% 목표 수량, 수수료·슬리피지 각각 10/20bp, 기존 guard를 유지했다. 모두 기본 파라미터이며 가격조건은 전일 종가·익절5%·손절3% 연구 설정이다. 전략·계좌 설정을 변경한 작업이 아니다.

1. **정확성 선행:** **SOXQ-P2-05** generic 일봉 rolling252 고가와 **P2-16** TR14/준비 진단을 수정·회귀 검증했다. live 완료 일봉 공급은 후속 P2-05·17에서 연결했다. P2-16의 표본/walk-forward는 남아 열린 상태다. [v8 비교 근거](research/autonomous-corrections-20261018/report.md).
2. **비교 기반 완료:** **P2-14·15** 체결 시점과 실현/잔여 평가손익·기말 청산 비용, 가상 FX/배당·실제 투자비중 비교를 분리했다. 강한 종가 15창의 평가이익/실현손실과 QQQ 가정청산 비용의 부호 전환을 검증했다.
3. **불필요한 반복:** **P2-03·04·13**의 상태·중복 신호·손절 후 재진입을 공통 guard 유지하에 검증한다.
4. **자료/표본:** **P1-05·P2-12·16·17**의 실제 분봉·봉 공개 시각·DST·누락 자료·ATR·준비 자료 조회를 검증한다.
5. **연속 기반 완료·전략별 가설:** **P2-18**의3년 상태 보존/분기 초기화 비교를 완료했다. 아래 P2-01~13 후보는 비용 경로·반복 신호 근거를 우선하며 하나씩 사전 고정하고 기각 결과도 보관한다.

P1-RUN-01의 실행 epoch·설정 revision 예약 검증, provider 고정, 수동/체결 대조 계좌·모드 보호를 구현했다. [검증 기록](tasks/execution-run-reservation-20261018.md). P1-CONFIG-01은 저장 후 인스턴스 공개와 13전략 보유 보호 보존으로 해결했다. [검증 기록](tasks/strategy-configuration-preservation-20261018.md). P1-BAL-01은 공통 잔고 게시 gate와 계좌별 캐시·오류 복구로 해결했다. [검증 기록](tasks/holdings-publication-fence-20261018.md). P1-FILL-01은 exact owner 체결·원가·보호 복원으로 해결했다. [검증 기록](tasks/strategy-fill-ownership-20261018.md). 로컬 TLS verify-full·서버 단절/재접속 contract도 완료했다. P2-14 체결 모델과 P2-15 손익·비용·현금흐름 비교도 완료했다. P2-18의 연속 상태 경계·실제3년/별도분기 비교도 완료했다. P2-06의 기본 비용 경로와 단독 진입 필터 비교를 완료했다. P2-03 모멘텀의 실제 소유 피드백·교차 후보 비교도 완료했다. P2-04 이격도 실제 소유·단독 진입/청산 후보 비교도 완료했다. P2-10 평균회귀 소유 피드백·단독 후보 비교도 완료했다. P2-01 이동평균 교차의 이전 완료 일봉 필터와 단독·사후 스트레스 비교도 완료했다. P2-02 RSI 실제 소유·단독 후보 비교도 완료했다. P2-07 돌파 실패의 실제 소유·완료 자료·선택 보호 비교도 완료했다. P2-08 강한 종가의 실제 소유·완료 시장일·선택 청산 비교도 완료했다. 다음 자동 과제는 P2-09 변동성 확장의 조건 빈도·체결/청산 비교다.

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
  - [ ] **필요한 사용자 자료:** 보관 파일 경로 또는 사용 가능한 공급원/자료 이용 범위를 제공한다. 2026-07-06~10-02 SOXQ 실제 분봉을 확보·보관하고 거래소 시간대·DST·세션 및 기간 전체 coverage를 검증한 후 재실행한다. 현 표본으로 이 조건을 완료 처리하지 않는다.
  - 대상: `commands/strategy_preview.rs`, `trading/strategy/leveraged_trend_hold/{live,preview,indicators}.rs`, `trading/simulation/assessment.rs`. [작업 기록](tasks/soxq-p1-05.md), [v7 검증](research/soxq-3m-20261003/p1-05-validation.md).

## P1 — 보류 사항

- [ ] **🟠 사용자 판단 대기** MariaDB contract fixture와 지원 버전 matrix 검증.
  - **필요한 사용자 판단:** 기존 실제 서버 fixture 보류를 해제할지 결정하고 지원할 MariaDB 버전 범위를 선택한다. adapter·DDL은 유지한다.
- [ ] **🟠 사용자 판단 대기** 여러 broker/account 포지션과 pending을 동시에 상주·대조하는 multi-scope runtime.
  - **필요한 사용자 판단:** 동시 다계좌 운용을 제품 범위로 채택할지 결정한다. 현재 제품은 단일 활성 scope 불변식을 사용한다. 동시 다계좌 운용을 제품 범위로 채택할 때 `(broker, account, market, symbol)` tracker와 scope별 credential reconciliation으로 전환한다.

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
  - **필요한 사용자 검증:** 비식별 provider fixture 또는 명시 승인된 read-only/소액 검증 환경을 제공한다. 공통 상태와 정정/reconciliation 직렬화는 구현했으나 old/new order ID 및 거부·만료 필드 의미는 실제 근거 확인 후 고정한다. 주문 실행 승인은 이 목록에서 가정하지 않는다.
  - 도입 이전 보유·기존 체결의 전략/프로파일 귀속·원가·투자모드를 실제 원본과 대조한다. 새 원장에 계좌 전체 보유를 자동 편입하지 않으며 확인 불가 기록은 신규 주문 차단 사유로 유지한다. 로컬 합성 재시작·체결 검증을 운영 원장 검증으로 대신하지 않는다.

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
- [ ] **🟢 자동 구현** **SOXQ-P2-09 — 변동성 확장:** 중앙값0%, 완료8회·무완료38/45창·무신호20창. 전 창 준비 완료이므로 무신호를 자료 부족으로 오해하지 않는다. 완료 조건: 당일 범위·이전 N일 평균/0범위 포함 fixture 유지, 조건 발생 빈도·진입/청산 장중 창·실제 체결 가능성 검증 후 범위/배수 후보별 비교; 미청산 이익으로 우수성 판정 금지.
  - P2-18 연속20bp 완료청산은3ETF 합2회다. 희소 조건/청산 표본과 기말 보유를 분리해 후보 비교를 유지한다.
- [x] **SOXQ-P2-10 — 평균회귀 실제 소유·단독 추세/청산/보유 후보 비교를 완료했다.**
  - 현재 포함20관측 모집단 밴드·strict 경계와 native stop5(0도 원가 이하 손절)를 유지하고 확정 소유·원가·관측 나이를 분리했다. 선택 middleBand/이전N추세/보유관측의 저장·두 preview·UI/A-B, live 약10초 관측과 일봉 준비 분리, ageunknown 복원·partial·거절·준비 부족 보호를 검증했다. 기존 QQQ1주252+64봉 비용/nextOpen4경로를 유지했다. [작업 기록](tasks/mean-reversion-policy-20261020.md).
  - 비평균회귀1122개 전체 report/signals와 평균회귀102개 backtest/signals가 동일했다. confirmedUpper는 legacy와78/78개 경제 경로가 같고 기존 원장 flatSell/heldBuy도0이므로 신호 감소·수익 개선으로 주장하지 않는다. 최종36연속/36분할/432별도분기와 부모74,968개·독립219,769개 기록 산술(오류0)을 통과했다.
  - 중심선은 연속6개 모두 순평가수익 하락·회전 증가, trend200은 SPY/QQQ 일부 손실 감소이나 SOXQ20bp6.9134→−3.7217%다.63관측은 발동0·최장55로 효능 평가 불가다. 이를 본 뒤 고정한21관측 사후 탐색은 QQQ 소폭 개선/SOXQ·SPY 악화, 최저 완료손실 동일과 guard 뒤22/24관측 재시도를 기록했다. upperBand/선택옵션0을 유지하며 종가 손절·보유 관측을 장중 손실/실제 보유 상한으로 부르지 않는다. [사전 선언·사후 탐색·실현/잔여·비용/노출·기각 근거](research/mean-reversion-policy-20261020/report.md).
  - 우선 회귀 근거: P2-18 SPY연속 +4.2519%→+0.0835%, 실제 매도2025-06-27↔04-09·동일경로 비용복원 +4.9703%→+1.5065%로 경로 자체가 달라진다. QQQ20bp −1.8039%/MDD4.5823%도 사전 고정 검증 구간으로 보관한다.
- [ ] **🟢 자동 구현** **SOXQ-P2-11 — 추세 필터:** 중앙값 +0.4886%, 최악 -4.1022%/MDD5.5534%, 완료22회 모두 손실·기말 보유39/45창. 최근248봉 SOXQ +9.6300%도 완료0회 평가손익이다. 완료 조건: 실제 청산이 있는 복수 구간에서 장기 추세 이탈·수익 보호·최대 보유 후보와 노출을 비교하고 실현/잔여 손익을 분리; 200봉 기준 유지.
  - P2-18 SOXQ연속20bp 총+9.5760%지만 실현−240553.37/잔여+1198152.86원·MDD16.2171%·평균투자21.2574%다. 실제 청산/수익보호·노출 비교를 함께 요구한다.
- [ ] **🟠 사용자 검증·자료 대기** **SOXQ-P2-12 — 레버리지 추세 보유(LTH):** 최근 실제 분봉2,318개/7날짜의 기본·볼린저 활성×비용2종 네 진단은 신호0이다. null OHLCV412개·정규장 밖1개는 제거했고 합성하지 않았다. 고정KST/auto 세션·봉 종료 표기의 시작·종가 공개 가능 시각 모델·10초 관측 차이·누락/기간 coverage 미검증이라 0% 성과/볼린저 효과 없음으로 해석하지 않는다. 완료 조건: P1-05 실제3개월 원본·DST/거래일/세션·gap·진입/마감 창 검증 후 옵션별 비교. generic 거부·일봉 평가 불가 유지.
- [x] **LTH 분봉의 거래소 시각·세션·미관측 슬롯 진단을 구현했다.**
  - `lth-minute-timing-v1`은 원본 실행 label의 UTC/시장 timezone·DST offset, 봉 종료/시작 구분, 중복/역순/분경계/asOf 뒤 종가와 관측일 내부 격자 공백을 bounded512행·64byte·날짜8/예시6개로 진단한다. current/A-B/복원·legacy/손상 형식·일봉/빈자료·모바일을 fixture/mock으로 검증했다. 기존 고정 KST 시계·신호·정산·입력hash와 sampleOnly/notEvaluable·lookAheadSafe=false는 유지한다. [구현·검증](tasks/lth-minute-timing-20261022.md).
  - 일반 평일 시간창과 실제 달력·발행/수신·공급원/요청기간 coverage는 다르다. P1-05/P2-12 실제3개월 자료와 운용 검증은 계속 미완료다.
- [ ] **🟢 자동 구현** **SOXQ-P2-13 — 가격조건:** 전일 종가 고정·익절5%·손절3% 연구 설정에서 중앙값 +0.2847%, 완료117회, 비용 부호 반전3창. 완료 조건: 평가 전 고정 기준가 유지, 손절 직후 재진입·공통 cooldown/최소 기대수익·회전율을 두 비용과 복수 구간에서 비교; guard 완화나 사후 최적화 금지.
  - P2-18 최초2022기준가 고정 모델은20bp 각ETF 청산1회·평균초기자본비중0.0806/0.2430/0.1259%다. 희소노출로 해석하고 비교 도중 분기마다 기준가를 바꾸지 않는다.
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


- [x] **SOXQ-P2-18 — 연속 운용 검증의 포지션·guard·risk 상태를 보존했다.**
  - v12 공통driver에서 factory/warmup/ReplaySession/finish각1회, native원가·pending·ATR·쿨다운·시장날짜/연속손실 상태를 청크 사이에 유지한다. 기존flat합계500을 보존하고 새bounded입력(16청크/각500/평가4096/history512), strict시각검증·미래prefix/단일·분할회귀·IPC/REST/모드/A-B/UI자료수집을 연결했다.
  - 직전cc6f5d0 생산1224회 전체report/신호 호환,3ETF×12전략×비용2의3년72회/alternatepartition72회/별도분기864회와 root136717·독립143866 산술체크를 통과했다. 최초전일결정수량·PC기준가/params를 고정하고 실제 accepted 수량의 갭/장중 시장가격 경로를 비용환급과 대사했다. 분기계좌 수익률 합산/복리는 없다.
  - 강한종가·돌파실패의 높은 평가수익도 청산0회, SOXQ일부분기 단순보유72개는 비중한도로0주다. 같은요청수량을 동일실제노출/알파비교로 표현하지 않는다. UI의7일경계는 보수적자료gate이며 달력검증이 아니고 LTH시간/coverage·별도walk-forward는 미완료 유지. [작업·검증](tasks/continuous-replay-20261018.md), [설계·실자료·한계](research/continuous-replay-20261018/report.md).


## 자동매매 예산·볼린저 보강 검증 기록 (2026-09-11)

- [x] 계좌/통화별 전용 예산, 자동 소유수량, durable 예약·체결·완료 복원, 설정 UI와 E2E 추가.
- [x] 기존 LTH에 기본 비활성 볼린저 필터와 중심선/ATR 청산, 공통 live/replay 및 미리보기 설정 연결.
- [ ] **🟢 자동 구현** 실제 provider 응답 불명 주문의 운영자 대조/복구 UI: 현재는 자동 해제 없이 예산을 보류한다. 주문번호/clientOrderId 대조 및 멱등 승인 흐름으로 확장할 것.
- [ ] **🟠 사용자 검증·자료 대기** LTH 보강의 일봉 성과 미리보기: 일봉 context와 분봉 지표는 분리했지만 장중 경로를 일봉에서 복원하지 않는다. 현재 일봉은 평가 불가 진단이며 활성 볼린저 옵션의 성과 표본은 1분봉만 허용한다. 실제 장중 자료·시장 시간 검증은 P1-05 잔여 조건이다.

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
  - **필요한 사용자 판단:** 배포 대상 플랫폼과 키/인증서 관리 주체를 정하고 GitHub Environment에 키를 준비한다. 자동화된 checksum/attestation과 signature 검사 구현은 완료되어 있다. 키 준비 후 `REQUIRE_RELEASE_SIGNATURES=true`를 적용하고 실제 서명 artifact를 검증한다. [보안·서명 게이트](release-security.md).

## 완료 기준

- 각 항목은 코드, 실패/복구 테스트, 사용자-visible 상태, 관련 문서가 함께 반영되어야 닫는다.
- 실계좌 검증은 모의/fixture → read-only 계좌 조회 → 명시 승인된 소액 주문 순서를 지킨다.
- 경고를 포함해 `cargo check`와 TypeScript 검증이 깨끗해야 하며, 주요 UI 배치 변경은 Playwright로 확인한다.
