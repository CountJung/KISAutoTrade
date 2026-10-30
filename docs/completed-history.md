# KISAutoTrade — 완료·경과 이력

> 2026-10-29 문서 재편. 현재 남은 작업의 정본은 [todo.md](todo.md)이며, 이 문서는 완료·과거 경과·기각 근거를 보존한다.
> 아래 인용은 재편 전 `6eeb0ea718138438296b832bcbbe3e218fc49564`의 `docs/todo.md`에서 이관했다. 원래 ID·날짜·검증 수치·한계를 유지하며 이번에 다시 실행한 검증으로 주장하지 않는다.

## 이력 사용과 추적

- 상세 구현/실험 기록은 기존 `docs/tasks/`·`docs/research/` 링크를 정본으로 유지한다. 이 문서는 TODO에서 옮긴 기록의 색인과 원문 보존소다.
- 완료 표시는 당시 명시한 범위에 한정된다. 실제 provider 운용, 독립 OOS, 새 전략 채택이나 미확인 자료까지 완료했다는 뜻이 아니다.
- 부분 완료 항목은 아래 별도 절에 보관하고, 원래 열린 상위 ID는 현재 TODO에 유지한다.
- 원문 줄 번호는 위 고정 커밋 기준이다. 기존 ID가 없는 열린 항목에는 이번에 추적 ID만 부여했다.

## 2026-10-29 즉시 작업 완료

<a id="p1-recovery-01"></a>
### P1-RECOVERY-01 — 운영자 대조·복구 UI

- **완료 범위:** desktop IPC/UI, 신규 제출 전 원래 주문 snapshot, 대조 승인 receipt→기존 pending 복원, 원래 시각의 멱등 TradeGuard 복원. 승인 자체는 재주문·체결 합성·예약 해제를 하지 않는다.
- **검증:** Rust 전체807개, 복구 focused19개, 제출 focused15개, format/check/strict clippy 경고0. TypeScript/FSD·production 웹빌드, 최종 production 복구11+Settings4=15개 mock UI 통과. 독립 안전감사에서 지적한 중복 provider ID·재시작/재조회 재시도·유실 primary·손절 guard 누락 수정 후 새 P0/P1/P2 지적 없음.
- **한계:** legacy 원래 소유 근거가 없거나 부분체결 pending intent가 유실된 경우 보류한다. 실제 provider·운영자 원본 대조, 보관기간 밖 이력, 실제 계좌/주문 검증은 수행하지 않았으며 P1-PROVIDER-01이 남는다. [회차별 범위·검증 기록](tasks/immediate-backlog-20261029.md).
- **원래 조건 보존:** `5437c07bc6c2bf0876c0ddc211ddeabb6cd632d7`의 TODO P1-RECOVERY-01에서 이관했다. 대조 근거·scope·주문 식별값·누적 체결/보류 예산 표시와 상태 전이, 중복 승인·역순/부분 체결·불일치·저장 실패·재시작 fixture, mock 대조→승인→영속화→재시작 일치와 확인 불가 보류가 완료 기준이었다. [기존 소유·체결 경계](tasks/strategy-fill-ownership-20261018.md), [예산 원장 구조](ARCHITECTURE.md) 의존성을 보존한다.

## 기존 기록·운영 참고 색인

아래는 기존 문서의 진입 링크다. 기록의 원래 날짜·검증 범위와 미완료 조건은 각 문서를 따른다.

| 역할 | 기존 정본 |
|---|---|
| 과거 전체 설계 아카이브 | [MasterPlan.md](MasterPlan.md) |
| 저장·보안 인계 기록 | [2026-10-06 인계](security-storage-handoff-2026-10-06.md) |
| 초기 프로젝트 점검·SOXQ P1-01 기록 | [프로젝트 점검](tasks/project-health-and-soxq-p1-01.md) |
| 자동 진행 백로그 기록 | [2026-10-18 작업](tasks/autonomous-backlog-20261018.md) |
| 문서 배치 변경 기록 | [2026-10-18 배치](tasks/documentation-layout-20261018.md) · [2026-10-20 루트 정리](tasks/root-documents-20261020.md) |
| 현재 에이전트 도구·코딩 규칙 | [도구 운영](agent-tooling.md) · [코딩 가이드](coding-guide.md) |
| 현재 LTH 파라미터 안내 | [파라미터 가이드](leveraged-trend-hold-parameter-guide.md) |
| 별도 승인 범위의 운영 검증 절차 | [모의 검증 체크리스트](mock-trading-e2e-checklist.md) · [Toss read-only·소액 검증](toss-readonly-small-order-checklist.md) |

## 문서 재편 당시 열린 항목의 ID 대응

| 기존 제목 | 현재 ID | 처리 |
|---|---|---|
| MariaDB contract fixture와 지원 버전 matrix 검증. | P1-DB-01 | 미완료 유지·분류 이동 |
| 여러 broker/account 포지션과 pending을 동시에 상주·대조하는 multi-scope runtime. | P1-SCOPE-01 | 미완료 유지·분류 이동 |
| KIS/Toss 기존 운영 원장과 실제 정정·취소 응답의 최종 검증. | P1-PROVIDER-01 | 미완료 유지·분류 이동 |
| 실제 provider 응답 불명 주문의 운영자 대조/복구 UI: 현재는 자동 해제 없이 예산을 보류한다. 주문번호/clientOrderId 대조 및 멱등 승인 흐름으로 확장할 것. | P1-RECOVERY-01 | 재편 당시 미완료 → [2026-10-29 로컬 완료](#p1-recovery-01) |
| 1,000라인 초과 파일을 책임 단위로 분리한다. | P3-REFACTOR-01 | 미완료 유지·분류 이동 |
| 플랫폼 코드 서명·updater 서명 강제를 위한 배포 키 준비. | P3-SIGN-01 | 미완료 유지·분류 이동 |

기존 “LTH 보강의 일봉 성과 미리보기”는 SOXQ-P1-05에 통합되어 있으며 SOXQ-P2-12 옵션 비교는 별도 결과물로 유지한다. MariaDB(P1-DB-01)와 MSRV/DB 정책(DEP-P3-01)은 정책 결정을 공유하지만 실행 검증의 완료 조건이 달라 합치지 않았다.

## 과거 연구·선행 작업 경과

### 원문 18–47행

### 현재 전략 전수 연구 후 권장 순서 (2026-10-18)

[전담 분석 보고서](research/strategy-review-20261018/report.md) · [실행별 근거](research/strategy-review-20261018/results.jsonl) · [요약](research/strategy-review-20261018/summary.json). 등록 13종 중 일봉 12종 1,224회, LTH 실제 분봉 표본 4회 진단을 실행했다. 주집계는 SOXQ/SPY/QQQ의 2023Q1~2026Q3 완전 45창이다. 고정된 전일 기준 15% 목표 수량, 수수료·슬리피지 각각 10/20bp, 기존 guard를 유지했다. 모두 기본 파라미터이며 가격조건은 전일 종가·익절5%·손절3% 연구 설정이다. 전략·계좌 설정을 변경한 작업이 아니다.

1. **정확성 선행:** **SOXQ-P2-05** generic 일봉 rolling252 고가와 **P2-16** TR14/준비 진단을 수정·회귀 검증했다. live 완료 일봉 공급은 후속 P2-05·17에서 연결했다. 당시 P2-16의 표본/walk-forward는 남아 있었다(후속 완료는 아래 해당 ID 참조). [v8 비교 근거](research/autonomous-corrections-20261018/report.md).
2. **비교 기반 완료:** **P2-14·15** 체결 시점과 실현/잔여 평가손익·기말 청산 비용, 가상 FX/배당·실제 투자비중 비교를 분리했다. 강한 종가 15창의 평가이익/실현손실과 QQQ 가정청산 비용의 부호 전환을 검증했다.
3. **불필요한 반복:** **P2-03·04·13**의 상태·중복 신호·손절 후 재진입을 공통 guard 유지하에 검증한다.
4. **자료/표본:** **P1-05·P2-12·16·17**의 실제 분봉·봉 공개 시각·DST·누락 자료·ATR·준비 자료 조회를 검증한다.
5. **연속 기반 완료·전략별 가설:** **P2-18**의3년 상태 보존/분기 초기화 비교를 완료했다. 아래 P2-01~13 후보는 비용 경로·반복 신호 근거를 우선하며 하나씩 사전 고정하고 기각 결과도 보관한다.

P1-RUN-01의 실행 epoch·설정 revision 예약 검증, provider 고정, 수동/체결 대조 계좌·모드 보호를 구현했다. [검증 기록](tasks/execution-run-reservation-20261018.md). P1-CONFIG-01은 저장 후 인스턴스 공개와 13전략 보유 보호 보존으로 해결했다. [검증 기록](tasks/strategy-configuration-preservation-20261018.md). P1-BAL-01은 공통 잔고 게시 gate와 계좌별 캐시·오류 복구로 해결했다. [검증 기록](tasks/holdings-publication-fence-20261018.md). P1-FILL-01은 exact owner 체결·원가·보호 복원으로 해결했다. [검증 기록](tasks/strategy-fill-ownership-20261018.md). 로컬 TLS verify-full·서버 단절/재접속 contract도 완료했다. P2-14 체결 모델과 P2-15 손익·비용·현금흐름 비교도 완료했다. P2-18의 연속 상태 경계·실제3년/별도분기 비교도 완료했다. P2-06의 기본 비용 경로와 단독 진입 필터 비교를 완료했다. P2-03 모멘텀의 실제 소유 피드백·교차 후보 비교도 완료했다. P2-04 이격도 실제 소유·단독 진입/청산 후보 비교도 완료했다. P2-10 평균회귀 소유 피드백·단독 후보 비교도 완료했다. P2-01 이동평균 교차의 이전 완료 일봉 필터와 단독·사후 스트레스 비교도 완료했다. P2-02 RSI 실제 소유·단독 후보 비교도 완료했다. P2-07 돌파 실패의 실제 소유·완료 자료·선택 보호 비교도 완료했다. P2-08 강한 종가의 실제 소유·완료 시장일·선택 청산 비교도 완료했다. P2-09 변동성 확장의 테스트 가능한 구현·일봉 비교도 완료했고 실제 장중 근거는 열린 자료 조건으로 유지한다. P2-11 추세 필터의 실제 소유·청산·노출 비교도 완료했다. 당시 다음 자동 과제는 P2-13 가격조건과 P2-16 구간 해석이었다(후속 기록은 아래 해당 ID 참조).

모든 12전략의 동일 수량 보유 대비 초과수익 중앙값이 0 이하이며 우수 전략을 확정하지 않았다. 분기 포트폴리오는 매번 초기화하며, 부분 Q4와 겹치는 최근248봉은 주집계에서 제외했다. 작은 완료 표본·서로 상관된 ETF·same-close 체결·고정 환율/무배당의 한계를 유지한다.

SOXQ 연구에서 확인한 시뮬레이션 정확성 과제는 아래에 기록한다.


## SOXQ 3개월 연구에서 확인한 P1 과제 (2026-10-03)

근거: [평가 보고서](research/soxq-3m-20261003/report.md), [차트](research/soxq-3m-20261003/charts.html), [원시 결과](research/soxq-3m-20261003/results.json).
평가 구간은 2026-07-06~10-02, 64거래일이며 사전자료 252봉을 별도 제공했다. 런타임 수정 없이 현재 엔진을 실행한 연구다.

후속 완료: **SOXQ-P1-02 고가/종가 warmup 분리** — 기본 OHLC 훅은 종가, 52주 신고가는 고가를 선택하도록 수정했다. 기본 200/252봉, 호출 1회, 첫 신호/미래 봉/분봉 혼입·보유 상태 회귀를 검증했다. [검증·동일 입력 재평가](research/soxq-3m-20261003/p1-02-validation.md): 평균회귀 +1.4402%→+2.2083%, 완료 거래 2→1. 나머지 정확성 과제는 아래에 유지한다.

후속 완료: **SOXQ-P1-01 평가 구간·사전자료·준비 상태 분리** — 모든 전략이 같은 평가 기간을 유지한다. 6개 누락 initializer와 이전 MA/RSI 기준선을 복원하고, 200/252봉 기준을 유지하며 실제 준비 상태와 조건 평가 봉 수를 반환한다. live 신고가 최신 봉 제외와 자료 부족 재초기화 뒤 보유 손절을 검증했다. 단일 provider 응답의 이전 봉만 사전자료로 사용하고 자료 부족·준비 중·조건 불충족·신호 무체결을 구분한다. [동일 입력 재평가·검증](research/soxq-3m-20261003/p1-01-validation.md).

후속 완료: **SOXQ-P1-03 일봉 완료봉·날짜 전환 이벤트** — 시가-only 시작 → 완성 OHLC 종가 평가 → 실제 체결/차단 포지션 피드백 → 완료봉 적재 순서를 도입했다. 강한 종가는 매일 전일 조건을 다음 평가일에 사용하고, 변동성 확장은 당일 OHLC와 이전 N일 범위를 비교한다. 0변동폭도 일수에 포함하며 날짜 전환 시 보유 상태를 유지한다. 두 전략은 일봉만 허용한다. 동일 종가 체결 근사이며 익일 시가·장중 체결은 P2-14로 유지한다. [v5 재평가·검증](research/soxq-3m-20261003/p1-03-validation.md).

후속 완료: **SOXQ-P1-04 LTH 과거 실행의 현재시각 의존 차단** — generic LTH는 factory 전 `UNSUPPORTED_GENERIC_REPLAY`로 명시 거부하며 접미 ID·모든 간격에 적용한다. 기존 입력 시각 전용 경로를 유지하고, production-library fixture에서 세션/blackout/장마감·신호/체결·미래 봉 불변을 검증했다. 서로 다른 시스템 분·시간대의 오프라인 재실행에서 지원 24실행과 거부 결과·전용 일봉 진단이 일치한다. 합성 일봉/고정 세션의 시장 정확성은 P1-05에 유지한다. [v6 검증](research/soxq-3m-20261003/p1-04-validation.md).


### 원문 99–105행

## P2 — 전략 시뮬레이션과 연구 워크플로

후속 완료: **P2-05·P2-17 live 완료 일봉과 페이지 준비 연결** — scope/config/run/시장 날짜를 고정한 bounded background 수집,252/512봉 요구·마지막 완료봉 보존·부족 중 신규 매수 차단/보유 손절, IPC/웹/UI 상태와 같은scope52주 설정/rollback 보유 보존을 구현했다. [검증/한계](tasks/live-completed-daily-history-20261018.md). 실제 provider 운용·전체 주문 lifecycle 검증으로 확대하지 않는다. 발견한 기존 주문·다른 전략 설정 위험은 위 P1 자동 과제로 먼저 해결한다.

전략별 항목의 최신 수치는 [2026-10-18 전수 연구](research/strategy-review-20261018/report.md)의 완전45창·기본 비용 결과다. 별도로 명시한 과거 SOXQ 수치는 당시 구간 관측이다. P1-01(v4) 초기화 결과는 [준비 상태 검증](research/soxq-3m-20261003/p1-01-validation.md), 강한 종가·변동성 확장은 [P1-03(v5) 검증](research/soxq-3m-20261003/p1-03-validation.md), generic LTH 거부는 [P1-04(v6) 검증](research/soxq-3m-20261003/p1-04-validation.md), 현재 일봉 진단/분봉 표본 경계는 [P1-05(v7) 검증](research/soxq-3m-20261003/p1-05-validation.md)을 먼저 확인한다. 일반 전략은 완료된 P2-14·15·18의 체결/회계 가정을 고정해 후보 비교를 이어가고, LTH의 별도 시간·자료 검증은 유지한다. 별도 [P2-18 연속 연구](research/continuous-replay-20261018/report.md)는 nextOpen·2023~2025 각752봉·최초 수량/PC기준가 고정의3년 장부와12개 독립 초기화 계좌를 구분한다. 기존45분기 통계와 합산하지 않는다.



### 원문 198–201행

## 자동매매 예산·볼린저 보강 검증 기록 (2026-09-11)


### 원문 203–204행

기존 열린 제목 **“LTH 보강의 일봉 성과 미리보기”**는 별도 일봉 성과 기능이 아니다. 일봉 평가 불가·분봉만 성과 표본으로 허용하는 구현 근거와 실제 장중 자료/시장 시간 미완료 조건이 P1-05와 같아 **SOXQ-P1-05에 통합**했다. 미완료 조건을 닫거나 제거하지 않았으며, 활성 볼린저의 옵션별 비교 결과는 **P2-12**에 별도로 유지한다. [원래 일봉/분봉 검증 근거](research/soxq-3m-20261003/p1-05-validation.md).


## 완료 항목 원문

### 원문 77–80행

- [x] **P1-CONFIG-01 — 일반 전략의 설정 저장 중 보유 보호 상태를 보존했다.**
  - 13전략의 같은 ID/scope 재설정과 저장 후 공개, 저장 실패·불명확 저장 원본 복구/복구 실패 차단, params/targets 변경·빈 프로파일·held stop/trailing·취소·신호/주문 경합을 검증했다. 실제 provider·정상 fill 소유권 피드백까지 완료한 것은 아니다. [구현·검증](tasks/strategy-configuration-preservation-20261018.md).
  - 보유 또는 private 보호 중인 대상 제거/시장 변경을 거부하고, 다른 전략/계좌의 보유를 새 대상에 옮기거나 공유 tracker의 0수량으로 기존 보호를 지우지 않는다.


### 원문 81–84행

- [x] **P1-FILL-01 — 정상·부분 체결과 거부/취소를 주문 소유 전략에 반영했다.**
  - 계좌 snapshot과 exact owner(profile/account/모드/전략/시장) 원장을 분리하고 소유 수량·원가·LTH 최고가/관측을 보존한다. live 신호로 확정 보유를 바꾸지 않으며 자동매도/수동매도 가능 수량을 예약 직전 다시 검증한다.
  - provider snapshot 선반영 뒤 fill 중복 가감 방지, 누적 체결대금301의 잔액 보존, 고정 intent·0수량/증가수량 취소·저장 실패 재시도·원장 세대/재시작·타 owner/프로파일/모드 격리를 검증했다. KIS/Toss 엄격 잔고 변환과 Toss 수수료 비율100배 오류도 수정했다.
  - Rust455개(단위449+통합3+doc3), 브라우저89개, 독립 코드·심볼·helper·문서 리뷰 통과. 실제 운용/도입 전 보유 귀속은 별도 검증이다. [구현·검증과 한계](tasks/strategy-fill-ownership-20261018.md).

### 원문 85–88행

- [x] **P1-BAL-01 — 일반 잔고 조회의 늦은 응답과 계좌별 캐시 경합을 보호했다.**
  - 기존 게시 14곳을 공통 gate에 연결하고 profile/client/scope/모드·실행 epoch·설정 lease와 tracker별 조회/수량 version을 검증한다. A→B→A, 정지/재시작, 실제 0수량·삭제·두 시장 원자성·실제 partial fill·HTTP barrier를 검증했다. 네트워크 await 동안 전역 lock을 유지하지 않는다.
  - 잔고 이벤트는 현재 scope 재조회 힌트로 사용하고 이전 cache를 제거한다. 늦은 설정 응답·설정 재조회 실패·연속 프로파일 변경·backend 적용 후 mutation 오류도 검증했다. Rust365개·브라우저 신규13개와 기존26개, 독립 리뷰를 통과했다. [구현·검증과 한계](tasks/holdings-publication-fence-20261018.md).


### 원문 89–91행

- [x] **TLS verify-full 인증서, 서버 강제 단절·재접속 contract를 완료했다.**
  - 임시 CA/localhost 인증서의 PostgreSQL16 로컬 fixture에서 일반 계약3개와 TLS/재접속2개를 실제 실행했다. 신뢰 CA 성공, hostname/CA 불일치·비TLS·오인증 거부, backend socket 종료와 SIGKILL/재기동 뒤 동일 pool 복구, 단절 중 pending/소유/risk 오류 및 JSON/backup/backend 설정 보존을 검증했다.
  - cold pool single-flight·설정/pool snapshot, 파일/keychain 동시 원복·취소/불명확 복구 차단을 보강했다. 릴리스 gate에 같은 fixture를 연결했으며 실제 운영 DB·MariaDB matrix·Windows 실행은 별도 범위다. [구현·검증과 한계](tasks/database-tls-reconnect-20261018.md).

### 원문 106–110행

- [x] **SOXQ-P2-01 — 이동평균 교차의 이전 완료 일봉 진입 필터와 단독 비교를 완료했다.**
  - 기존5/20 입력 관측 교차·Sell·매 관측 prevMA 갱신과 공통 guard를 유지하고 선택 추세/MA기울기/ATR%/밴드폭%를 Buy후보에만 적용했다. 새4조건은 strict 이전 완료 일봉만 사용하며 필터 거절 교차를 되살리지 않는다. 저장/두 preview·scoped 일봉 재조회/실패·raw UI/A-B·준비/조건미달·1분 거부·활성 잠금/모바일을 검증했다. [작업·검증](tasks/ma-cross-filters-20261020.md).
  - archived327739a 대비 기본1224개 전체 backtest/signals 호환과 unfiltered78개 원장 동일, 최종48연속/48분할/576별도분기(624원장)를 확인했다. 부모104,884개 기록 산술+3,240개 이전일봉 조건 대조·독립313,205개 검사 오류0. 기본1224는 생산 hash 대조 증거이며 독립 직접 기본원장 대조78개와 구분한다.
  - 추세/기울기는 연속6개 순평가수익 모두 하락했다. ATR0.5/폭2는 거부0으로 활성 효능 미평가다. 이를 본 뒤 사후 고정 ATR2/폭10 스트레스를 각각 한 번 추가했으며 원래36controls 입력/경제hash를 보존했다. 폭10 SOXQ20bp 수익7.4794→10.7262%/MDD11.8990→7.5717%이나 최저 완료손실 동일·노출 감소, SPY/QQQ 수익 하락이다. SPY 두 스트레스 완료1개·평균투자1.6268%를 우월성으로 표현하지 않는다.
  - 비용 때문에 SOXQ trend200의2025-01-21매수가20bp에서 비중한도로 차단돼10bp 손실청산−218,390.83원을 건너뛰는 경로 차이, 기말 청산비용의 분기3개 양→음 전환을 보존했다. 기본5/20·선택옵션0 유지, 추가격자/재조정·운용채택 없음. [사전 선언·사후 탐색·기각/표본·실현/잔여·비용/노출](research/ma-cross-filters-20261020/report.md).

### 원문 111–115행

- [x] **SOXQ-P2-02 — RSI 실제 소유·단독 진입/청산 후보 비교를 완료했다.**
  - v18은 단순RSI14/30/70·zero-loss100·기존once-cross를 유지하며 actualowned 수량/원가·잔여전량청산·age/metadata·재설정을 연결했다. 추세200·손절3%·보유63관측을 각각 단독 비교하고 새옵션0을 유지한다. strict 저장/두preview·current/A-B·legacy/손상/모바일·실제guard거절후cap재후보를fixture/mock으로검증했다.
  - 비RSI1122개 전체report/signals 호환, RSI102개 finalEquity/accounting 동일(100개raw/report변경)을확인했다. 30연속/30분할/360별도분기에서owned78개actual경제경로는legacy와같고flatSell386·heldBuy70 제거, raw648→192·blocked462→10이다. 신호감소를수익개선으로해석하지않는다.
  - trend200은연속4악화/2동일, stop3는6개전부수익악화·회전증가/SOXQ비용부호전환이었다. hold63은cap0·78개report/signals동일·최장54/57/46관측으로효능평가불가다. 사후튜닝없이옵션0을유지하며20개무신호와36개무완료(owned)를준비부족과구분한다. [작업](tasks/rsi-policy-20261022.md), [설계·실현/잔여·비용/노출·기각](research/rsi-policy-20261022/report.md).
  - P2-18 연속20bp 완료는3ETF 합23회(9/7/7), 차단116건이다. 작은 완료 표본과 무보유 매도·회전·노출을 먼저 분리한다.

### 원문 116–119행

- [x] **SOXQ-P2-03 — 모멘텀 실제 소유 피드백과 선택 진입 교차를 완료했다.**
  - 실제 owned_quantity를 관측·무장/미체결 후보와 분리하고 flat Buy/held Sell을 적용했다. 기존 inclusive >=T/<=−T, 보유 청산·부분체결·거절 재시도·live pending·같은0수량 피드백·strict warmup/재설정과 공통 guard/risk를 유지한다. 기본 level/0, 선택 entry-only crossing/간격%p를 제공한다.
  - 비모멘텀1122회 전체 report/signals 호환, 모멘텀102창 경제지표·회계 동일(65창에서 신호/차단수만 변경), 연속24/분할24/별도분기288과 parent46,772·독립136,960검사를 통과했다. corrected level은 legacy와78개 회계/자산곡선/실제체결 exact동일이다. 비용당 연속 raw231→42·차단195→6, 무보유Sell187/보유Buy2건을 제거했으며 수익·수수료 개선으로 표현하지 않는다.
  - crossing/1%p는 연속 장부 추가 효과가 없고 각비용36분기에서2개개선/3·4개악화여서 공통 기본 채택을 기각했다. 복수기간·두비용 순수익/MDD/회전·최저완료손익·실현/잔여·노출, 동결 v7과 구분한 새 엔진 기준은 [연구](research/momentum-feedback-20261020/report.md), 구현·실패/복구·전체검증은 [작업 기록](tasks/momentum-feedback-20261020.md)에 보관했다. 실제 provider 운용/OOS 우월성은 별도다.

### 원문 120–124행

- [x] **SOXQ-P2-04 — 이격도 실제 소유·하향 교차·단독 진입/청산 후보를 완료했다.**
  - MA20/−5/+5의 inclusive 기본 경계와 공통 guard를 유지하고 소유량·native 평균가·관측 나이를 분리했다. 선택 crossing/추세 관측/보유 관측/손절의 저장·두 preview·UI/A-B·실제 소유 metadata 복원과 live 추세 cadence 분리, 미체결/거절/partial/준비/설정 변경을 검증했다. 저장 중 새 초안도 일반/PC/LTH에서 보존한다. [작업 기록](tasks/deviation-policy-20261020.md).
  - archived bca6601 대비 비이격도1,122회 exact, 이격도102창 경제 동일(60창 raw/차단만 변화),36연속/36분할/432별도분기와 독립200,495검사 통과. 기본 level은78/78개 실제 filled/회계/자산곡선 동일이며 비용별 raw305→23·차단286→4를 비용절감으로 표현하지 않는다.
  - crossing은 연속 동일·분기 비용별1개 개선/35동일, trend200은3개 개선/4악화·SOXQ 낮은 노출, maxHold63은 분기 동일·SPY 비용별 청산 경로 차이, stop5는SOXQ20bp 수익4.5454→2.6040%/MDD9.1305→9.5692%로 악화했다. 기본 level/다른옵션0을 유지한다. [사전 고정·기각 근거](research/deviation-policy-20261020/report.md). 과거 동결 SOXQ v7 +1.3756%와 초기 v3 +2.4987%는 다른 구간/엔진 기록이며 이번3년 수치와 합치지 않는다.
  - P2-18 당시 연속20bp 차단286건에는 무보유 매도200·보유 중 매수82건이 포함됐다. 이번 기본 소유 교정의 재현 기준이며 실제 체결·손익 동일성을 별도 확인했다.

### 원문 125–128행

- [x] **SOXQ-P2-06 — 연속 상승/하락 진입 필터와 생산 A/B를 완료했다.**
  - 기본0의 누적 변화·직전 완료 일봉 ATR·장기 MA 진입 확인을 제공한다. live 가격 관측과 일봉 context를 분리하고 잘못된 설정/자료 부족/0 ATR은 신규 매수만 보류한다. 보유 청산·설정 재저장·관측열과 기존 guard/risk를 유지하며 현재 평가 OHLC는 판단 뒤에만 게시한다.
  - 직전 생산코드 단기1,224회·장기6회 전체 report/signals 기본 호환, 고정 후보24연속·24분할대조·288별도분기, parent55,986·독립148,099산술 대사를 통과했다. 누적1%는 SPY/QQQ 악화, ATR0.5는 SOXQ 제한 개선이나 MDD/최악손실 동일·SPY무효, MA200은6개모두수익하락이라 공통 기본 채택하지 않았다. 기본0을 유지한다.
  - SPY 비용10/20bp의 매도2023-08-04↔08-07/2025-02-25↔03-31과 후속 기대 순익 차단을 보존했다. MDD·회전·최악 완료 손실·실현/잔여·노출·필터 거절/후속 주문 차단 및 기각 근거는 [연구](research/consecutive-filters-20261018/report.md), UI/실패/복구·전체 검증은 [작업 기록](tasks/consecutive-entry-filters-20261018.md)에 기록했다. 실제 운용 효과/OOS 우월성이나 실거래 검증 완료를 뜻하지 않는다.

### 원문 129–132행

- [x] **SOXQ-P2-07 — 돌파 실패:** 실제 소유·후보 anchor·완료 OHLCV와 선택 고가/ATR/거래량·수익 보호/보유 한도를 구현하고 UI·엄격 저장/두 preview를 검증했다. 다음 시가 체결의 anchor 유실을 교정했으며 기존 sameClose102개 변화0·비대상1122개 정확히 일치, 42연속/42분할/504분기·부모85073/독립233251회 정산을 확인했다. [작업](tasks/failed-breakout-policy-20261022.md), [결과·한계](research/failed-breakout-policy-20261022/report.md), [독립 감사](research/failed-breakout-policy-20261022/independent-audit.json).
  - 실현/잔여/가정 기말 비용·노출을 분리했다. 거래량 조건은 연속6개 손실, 수익 보호 체결38/16(연속/분기), 한도28/0과 분기72개 owned동일을 구분하며 선택값 자동 승격을 기각했다. 완료 승률만으로 판단하지 않고 비중 제한으로 비교 보유42개가 미체결인 점을 보존한다.
  - 후속 연구 조건: 새 관측을 읽기 전에 구간·universe·수량·비용·단독 후보를 고정하고 자료 공급 범위·조정·달력·누락과 gross/net·노출·회전·실현/잔여/기말 비용을 대사한다. 이번 과거 진단을 독립 OOS나 실제 운용 완료로 승격하지 않는다.
  - P2-18 연속3ETF×두비용은 모두 완료청산0회다. 높은 평가이익을 청산 설계 완료로 해석하지 않고 최대보유/수익보호 후보를 사전 고정한다.

### 원문 133–137행

- [x] **SOXQ-P2-08 — 강한 종가의 실제 소유·완료 시장일·선택 청산 비교를 완료했다.**
  - v20은 기존3/3·고가 대비 종가 거리·stop0의 활성 의미를 유지하고 실제 수량/원가·잔량·보유 관측/최고가/무장·strict 복원을 연결했다. live 완료 일봉 갱신 누락은 유효 시장일 ticket으로 교정하며 같은 날 재조회·실패/지연·legacy가 소비 조건과 보유 보호를 지우지 않는다. 저장/두 preview·현재/A-B·모바일과 미체결 기원 보호를 fixture/mock으로 검증했다. [작업](tasks/strong-close-policy-20261022.md).
  - 비대상1122개 전체 report/signals·기존 StrongClose102창·새 owned78원장 모두 기본 경제 결과와 신호를 보존했다. 사전 고정30연속/30분할/360별도분기와 부모67393개/독립253595개 정산을 통과했다. 대체 분할 원장의 독립 재정산과 생산 동등성 assertion은 구분한다. [설계·결과·한계](research/strong-close-policy-20261022/report.md), [독립 감사](research/strong-close-policy-20261022/independent-audit.json).
  - 추세200·수익5/되밀림3·보유63은 각각 연속6개 수익이 모두 낮아 기본 승격하지 않았다. 보호 체결89/49·한도46/0(연속/분기), 마지막 분기 한도 신호10개의 미체결과72개 동일 경제 경로를 구분한다. 원본/owned 연속은 완료0·잔량51/2/3으로 보유 평가수익이며, 분기 비교 보유30개는 비중 제한으로0주다. 선택값0·공통 guard를 유지한다.
  - 후속 승격은 새 자료를 읽기 전 구간·수량·비용·단독 가설을 고정하고 실제 자료 범위/조정·달력·시점과 비용·노출·회전·실현/잔여/기말 비용을 대사해야 한다. 이 사후 연구와 날짜 fixture를 미관측 OOS·실제 운용 완료로 취급하지 않는다.

### 원문 148–152행

- [x] **SOXQ-P2-10 — 평균회귀 실제 소유·단독 추세/청산/보유 후보 비교를 완료했다.**
  - 현재 포함20관측 모집단 밴드·strict 경계와 native stop5(0도 원가 이하 손절)를 유지하고 확정 소유·원가·관측 나이를 분리했다. 선택 middleBand/이전N추세/보유관측의 저장·두 preview·UI/A-B, live 약10초 관측과 일봉 준비 분리, ageunknown 복원·partial·거절·준비 부족 보호를 검증했다. 기존 QQQ1주252+64봉 비용/nextOpen4경로를 유지했다. [작업 기록](tasks/mean-reversion-policy-20261020.md).
  - 비평균회귀1122개 전체 report/signals와 평균회귀102개 backtest/signals가 동일했다. confirmedUpper는 legacy와78/78개 경제 경로가 같고 기존 원장 flatSell/heldBuy도0이므로 신호 감소·수익 개선으로 주장하지 않는다. 최종36연속/36분할/432별도분기와 부모74,968개·독립219,769개 기록 산술(오류0)을 통과했다.
  - 중심선은 연속6개 모두 순평가수익 하락·회전 증가, trend200은 SPY/QQQ 일부 손실 감소이나 SOXQ20bp6.9134→−3.7217%다.63관측은 발동0·최장55로 효능 평가 불가다. 이를 본 뒤 고정한21관측 사후 탐색은 QQQ 소폭 개선/SOXQ·SPY 악화, 최저 완료손실 동일과 guard 뒤22/24관측 재시도를 기록했다. upperBand/선택옵션0을 유지하며 종가 손절·보유 관측을 장중 손실/실제 보유 상한으로 부르지 않는다. [사전 선언·사후 탐색·실현/잔여·비용/노출·기각 근거](research/mean-reversion-policy-20261020/report.md).
  - 우선 회귀 근거: P2-18 SPY연속 +4.2519%→+0.0835%, 실제 매도2025-06-27↔04-09·동일경로 비용복원 +4.9703%→+1.5065%로 경로 자체가 달라진다. QQQ20bp −1.8039%/MDD4.5823%도 사전 고정 검증 구간으로 보관한다.

### 원문 153–158행

- [x] **SOXQ-P2-11 — 추세 필터 실제 소유·청산 비교를 완료했다.**
  - v22는200/5/20·현재포함산술MA·strict경계·세MA준비후원래장기이탈을유지하고실수량/native평균·잔량Sell·pending기원·보유관측나이/peak/무장·strict7필드checkpoint를연결했다. 양수unknown도metadataSome이며실제caller의coldrestore/동량/부분에서확인streak를보존한다. 재시작/과거지표재적재/core변경은streak0재관측, warmup은보호를만들지않고reset가격버퍼를보존한다. strict저장·두preview·raw초안/current-A-B/legacy미확인·모바일/실패/지연검증을동기화했다. [작업](tasks/trend-filter-policy-20261023.md).
  - 사전고정5종·30연속/30대체분할/360별도분기를생산RustAPI로실행했다. 기준81fa00d/v21과새owned의78원장/기존Trend102창은전체backtest/signals동일, 비대상1122호환·부모63559/독립171604검사·오류0이다. 실제소유교정의수익개선으로주장하지않는다. [고정설계·결과/한계](research/trend-filter-policy-20261023/report.md), [독립감사](research/trend-filter-policy-20261023/independent-audit.json).
  - 이탈확인2는일부개선/작은완료표본·SPY악화이고수익5/되밀림3·보유63은SOXQ20bp−3.6376/−3.5244%다. 기본확인1/선택0·공통guard를유지하고기본승격을기각한다. 연속수익보호58/보유한도45체결과분기26/0을구분하며보유63분기72개실제경로동일/16개마지막pending만추가라분기효능은미평가다. SOXQowned20bp+9.5760%는실현−240553.37/잔여+1198152.86원이며기말가상비용·비교보유0주·노출/회전을분리했다.
  - 후속승격은새자료를읽기전창·universe·수량·비용·단독가설을고정하고자료범위/조정/달력·공개시각과실제청산/차단·실현/잔여·기말·노출/회전을대사해야한다. 상관3ETF/겹침/분할을독립OOS나운용안전성으로승격하지않는다. 기존v12감사42958건은과거준비이력으로새실행과구분한다.


### 원문 165–167행

- [x] **LTH 분봉의 거래소 시각·세션·미관측 슬롯 진단을 구현했다.**
  - `lth-minute-timing-v1`은 원본 실행 label의 UTC/시장 timezone·DST offset, 봉 종료/시작 구분, 중복/역순/분경계/asOf 뒤 종가와 관측일 내부 격자 공백을 bounded512행·64byte·날짜8/예시6개로 진단한다. current/A-B/복원·legacy/손상 형식·일봉/빈자료·모바일을 fixture/mock으로 검증했다. 기존 고정 KST 시계·신호·정산·입력hash와 sampleOnly/notEvaluable·lookAheadSafe=false는 유지한다. [구현·검증](tasks/lth-minute-timing-20261022.md).
  - 일반 평일 시간창과 실제 달력·발행/수신·공급원/요청기간 coverage는 다르다. P1-05/P2-12 실제3개월 자료와 운용 검증은 계속 미완료다.

### 원문 175–177행

- [x] **SOXQ-P2-14 — 체결 가능성과 장중 손절의 연구 모델을 분리했다.**
  - v10 공통 ReplaySession의 sameClose/nextOpen, 종가 신호→다음 관측 시가·이전 ATR→실제 원가 피드백, 마지막 미체결, signalAvailableAt/fillAt의 봉/단계를 구현했다. 연구 보호0 기본, 갭 시가·OHLC 양방향 접촉 순서 불명·손절 우선 가정을 UI와 A/B에 표시하고 실제 체결 시각/유동성 검증으로 표현하지 않는다.
  - 직전 ba74e03 생산코드1,224회와 기본 모드의 전체 회계 report/신호가 일치했다. QQQ2026Q3 평균회귀의 비용·체결가가 청산을 바꾸는 fixture와 강한 종가 포함 실자료8회/benchmark8회, 합성 갭·미체결·순서 불명, 일반/LTH 피드백·공통 UI를 검증했다. [작업·검증](tasks/replay-execution-model-20261018.md), [독립 분석](research/replay-execution-20261018/report.md).

### 원문 178–181행

- [x] **SOXQ-P2-15 — 비용·배당·환율·청산 가정과 비교 기준을 구현했다.**
  - v11의 방향별/최소 수수료·수동 출처, 평가봉에 정확히 매칭한 FX/배당·조정가격 중복계상 거절, native 진입평균/역사적 KRW 원가 보존을 구현했다. 실제 청산 전망을 최소fee 기대수익 guard와 정산이 공유하며 손절/보호/native손실 예외를 유지한다.
  - 실현+잔여 순평가+세후배당=자산증감, 가격/FX 귀속과 report-only 기말청산, 관측종가 평균/최대 자본비중·보유봉비율 구분을 제공한다. 같은 요청수량 단순보유의 실제 체결량/노출차이와 동일 실제경로 비용환급 산술참조를 구분하고 소표본·역부호·부분/겹침창 경고를 표시한다. UI/A/B/legacy/LTH 평가불가 gate를 함께 검증했다.
  - 직전1192673 생산코드1224회 전체report/신호 기본호환, QQQ 단일분기40개 합성 시나리오/120개 항등식과 기록fill 독립4540대사를 통과했다. 강한종가 nextOpen20bp는 평가+0.0039%지만 가정청산−0.0390%다. 실제 요율·배당·FX/장기 운용 검증으로 확대하지 않는다. [작업·검증](tasks/replay-accounting-20261018.md), [연구·한계](research/replay-accounting-20261018/report.md).

### 원문 192–197행

- [x] **SOXQ-P2-18 — 연속 운용 검증의 포지션·guard·risk 상태를 보존했다.**
  - v12 공통driver에서 factory/warmup/ReplaySession/finish각1회, native원가·pending·ATR·쿨다운·시장날짜/연속손실 상태를 청크 사이에 유지한다. 기존flat합계500을 보존하고 새bounded입력(16청크/각500/평가4096/history512), strict시각검증·미래prefix/단일·분할회귀·IPC/REST/모드/A-B/UI자료수집을 연결했다.
  - 직전cc6f5d0 생산1224회 전체report/신호 호환,3ETF×12전략×비용2의3년72회/alternatepartition72회/별도분기864회와 root136717·독립143866 산술체크를 통과했다. 최초전일결정수량·PC기준가/params를 고정하고 실제 accepted 수량의 갭/장중 시장가격 경로를 비용환급과 대사했다. 분기계좌 수익률 합산/복리는 없다.
  - 강한종가·돌파실패의 높은 평가수익도 청산0회, SOXQ일부분기 단순보유72개는 비중한도로0주다. 같은요청수량을 동일실제노출/알파비교로 표현하지 않는다. UI의7일경계는 보수적자료gate이며 달력검증이 아니고 LTH시간/coverage·별도walk-forward는 미완료 유지. [작업·검증](tasks/continuous-replay-20261018.md), [설계·실자료·한계](research/continuous-replay-20261018/report.md).



### 원문 200–200행

- [x] 계좌/통화별 전용 예산, 자동 소유수량, durable 예약·체결·완료 복원, 설정 UI와 E2E 추가.

### 원문 201–201행

- [x] 기존 LTH에 기본 비활성 볼린저 필터와 중심선/ATR 청산, 공통 live/replay 및 미리보기 설정 연결.

### 원문 217–221행

- [x] 핵심 거래 흐름의 자동화 테스트를 release gate로 승격한다.
  - broker mock 기반 제출→부분체결→완전체결→취소/거부→재시작 복구 테스트를 추가한다.
  - balance fail-closed, scope 전환, midnight rollover, manual/auto parity, 인증 없는 REST 거부를 포함한다.
  - `cargo check`, `cargo test`, `npx tsc --noEmit`, `npm run check:fsd`, `npm run test:e2e`, OpenAPI 검증을 CI에서 실행한다.


### 원문 222–226행

- [x] 의존성·릴리스 보안 점검을 자동화한다.
  - `cargo audit`와 npm audit 정책, Dependabot/Renovate, lockfile 검증을 CI에 추가한다.
  - release artifact 서명/해시와 updater 경로를 검증하고, 실패 시 사용자가 확인할 수 있게 한다.
  - `release-gate.yml`, `security.yml`, Dependabot, 추적 lockfile 검증, SHA-256/attestation 및 detached signature 준비 검증으로 반영했다. 실제 플랫폼 코드 서명/updater 서명 강제는 배포 키 준비 후 `REQUIRE_RELEASE_SIGNATURES=true`로 전환한다.


### 원문 227–230행

- [x] 사용자 가이드를 broker별 실제 지원 범위와 일치시킨다.
  - KIS 전용 소개를 KIS/Toss 공통 기능, broker별 주문·시세·자동매매 제한 표로 교체한다.
  - 실제 UI의 13개 전략 카드, 프로파일 설정, 세션/캔들 source, 소액 실거래 gate를 기준으로 오래된 절차를 정리한다.


## 미완료 항목 안의 부분 완료·과거 진단

### SOXQ-P1-05

원문 48–59행의 부분 완료/과거 관측이다. 상위 작업은 [현재 TODO](todo.md)에서 미완료로 유지한다.

  - [x] v7/LTH v3에서 일봉 context를 bounded 별도 버퍼로 분리했다. 분봉 EMA/RSI/ADX·반동·볼린저에 일봉을 넣지 않으며, 분봉 OHLC 재초기화는 OHLC·반동·볼린저 snapshot, 가격 재초기화는 반동 snapshot만 교체한다. 일봉 재초기화는 분봉·보유 상태를 유지한다.
  - [x] 전용 일봉 경로는 시가를 장중 진입 가격으로 합성하지 않고 `dailyDiagnostic/notEvaluable` 자료 진단만 반환한다. UI는 수익률·거래 성과·A/B 저장을 숨기며 assessment 없는 기존 LTH 응답도 평가 불가로 표시한다.
  - [x] 현재 최대 200봉의 실제 분봉은 `intradaySample/sampleOnly`, 유효 분봉 0개는 `notEvaluable`로 표시한다. 일봉 context 수와 실제 장중 관측 수를 분리하고 시간대·DST·세션 미검증, 고정 KST 근사와 3개월 대표성 부족을 명시한다.

### SOXQ-P2-09

원문 138–147행의 부분 완료/과거 관측이다. 상위 작업은 [현재 TODO](todo.md)에서 미완료로 유지한다.

  - [x] v21 실제 수량/native 원가·잔량 청산·pending 기원·거절/부분/재설정/복원 보호, 이전N완료 범위와 당일OHLC·0범위/평균0·엄격 초과를 검증했다. live 유효 시장일의 관측 시가/고저·같은날 재조회/실패/latelegacy를 나누고 strict 저장·두 preview·전용 조건 빈도·현재/A-B·모바일을 동기화했다.
  - [x] 비대상1122개·기존102창·owned78원장의 기본 backtest/signals 호환, 고정36연속/36분할/432별도분기·부모70828개/독립249412개 정산과 현재390개 진단을 확인했다. 대체분할36개는 생산 경제 hash·입력연결이며 미저장 대체 원장 독립 재정산과 구분한다.
  - [x] 기간20은 연속4악화/2동일, 배수1.5는4개 개선/2동일이지만 모두 완료0·잔여평가다. 수익 보호/보유63은6개 모두 수익하락·회전증가, cap분기0·72개 동일이며 QQQ hold63은10/20bp +0.1099/−0.4609%다. 기본10/2/손절3·선택0·공통guard를 유지한다. 비교 보유0주36개·준비완료 무신호32/72개를 우수성/자료부족으로 오해하지 않는다.
  - 과거45창 중앙값0%·완료8·무완료38·무신호20은 별도 연구 관측으로 보존하며 이번 연속/분기 결과와 합산하지 않는다. P1-03의 완료봉·0범위/이전평균 계약도 유지한다.

### SOXQ-P2-12

원문 159–164행의 부분 완료/과거 관측이다. 상위 작업은 [현재 TODO](todo.md)에서 미완료로 유지한다.

- [ ] **🟠 사용자 검증·자료 대기** **SOXQ-P2-12 — 레버리지 추세 보유(LTH):** 최근 실제 분봉2,318개/7날짜의 기본·볼린저 활성×비용2종 네 진단은 신호0이다. null OHLCV412개·정규장 밖1개는 제거했고 합성하지 않았다. 고정KST/auto 세션·봉 종료 표기의 시작·종가 공개 가능 시각 모델·10초 관측 차이·누락/기간 coverage 미검증이라 0% 성과/볼린저 효과 없음으로 해석하지 않는다. 완료 조건: P1-05 실제3개월 원본·DST/거래일/세션·gap·진입/마감 창 검증 후 옵션별 비교. generic 거부·일봉 평가 불가 유지.

### SOXQ-P2-13

**2026-10-29 로컬 부분 완료:** live6/replay9 생산 경로 fixture, 현재 v22의6연속+6대체분할+72별도분기와 기존 보관78 PC원장 report/signals/preparation 일치, 기록체결 독립산술10,536건을 확인했다. 원래 기준가·수량·native5%/3%·두비용·기본 guard를 유지했고 생산 정책 변경은 없다. [이번 설계·결과·한계](research/price-condition-contracts-20261029/report.md), [회차 검증](tasks/immediate-backlog-20261029.md).

회차 시작 `ac3eb564ea678fe99722bbc5f3f040b4c50b1773`의 자동 구현 조건인 실제 exact owner/native 원가·부분/설정수량 차이·거절/미체결·손절 접수 전후·같은날/다음날/DST·계좌분리·native/overlay·두비용·분할불변을 합성 생산 경로에서 검증했다. live 잔량은 일일매도 제한·guard에 막힐 수 있으며 상태 보존을 청산 성공으로 해석하지 않는다. 실제 장중 자료와 실제 체결 가능성 조건은 같은 ID의 TODO 자료 구역으로 이관했고 전체 완료 체크는 하지 않았다. 아래는 이전 원문 이력이다.

원문 168–174행의 부분 완료/과거 관측이다. 상위 작업은 [현재 TODO](todo.md)에서 미완료로 유지한다.

- [ ] **🟢 자동 구현** **SOXQ-P2-13 — 가격조건:** 전일 종가 고정·익절5%·손절3% 연구 설정에서 중앙값 +0.2847%, 완료117회, 비용 부호 반전3창. 완료 조건: 평가 전 고정 기준가 유지, 손절 직후 재진입·공통 cooldown/최소 기대수익·회전율을 두 비용과 복수 구간에서 비교; guard 완화나 사후 최적화 금지.
  - **기존 원장 근거:** P2-18의 2023-01-03~2025-12-31 nextOpen 연속 평가는 비용별 3건·완료매도 3회·rawStopSignals 0·완료손절 0·guard 차단 0·기말보유 0이다. 별도 분기 초기화는 비용별 36건·완료매도 3회·33건 완료청산 없음·손절/차단 0이다. 원래 45창 sameClose 통계와 합산하지 않는다. [고정 설계](research/continuous-replay-20261018/design.json), [연속 원장](research/continuous-replay-20261018/continuous-results.jsonl.gz), [별도 분기 원장](research/continuous-replay-20261018/reset-results.jsonl.gz), [요약](research/continuous-replay-20261018/summary.csv).
  - **평가 범위:** 2022-12-30 기준가 SOXQ 19.91/SPY 382.43/QQQ 266.28, 수량 51/2/3, native 익절 5%·손절 3%, 수수료와 슬리피지 각각 10/20bp를 고정했다. 평균 초기자본 비중 0.0806/0.2430/0.1259%의 희소노출이며 손절 후 재진입·cooldown·최소 기대순익 gate의 효과를 이 표본으로 평가할 수 없다. 자료 준비 실패·코드 오류·전략 효능 0%라는 뜻이 아니다. 비교 도중 분기별 기준가 변경·사후 조정·guard 완화를 하지 않는다.
  - **기존 검증 범위:** `cargo test --offline --workspace --lib price_condition --locked` 8개 통과·0개 실패·781개 제외는 기존 동작 검증이다. 위 새 조건의 구현·연구 결과나 P2-13 완료 증거가 아니며 이 항목은 자동 구현으로 열린 상태를 유지한다.

### SOXQ-P2-16

원문 182–191행의 부분 완료/과거 관측을 보존한다. 아래 2026-10-29 구현·검증으로 로컬 작업을 완료했다.

  - v8 replay는 이전 종가 포함 TR14·동일 interval 사전 ATR warmup·준비 진단을 구현했다. 연구 UI는 기본 비활성 수량 산정과 위험bps/배수를 제공하고 부족·0범위·0bps 매수 차단을 표시한다. LTH는 실제 잔여수량/평균가 피드백으로 확대·축소·부분매도 보호를 검증했다. 최신45창의 작은 완료 표본과 자산/기간 상관·무거래 창은 여전히 구분해야 한다.
  - **기존 원장 진단:** v12 연속 72원장은 526/226봉이며 후반 양수·전반 절반 미만 26건, 전반 양수·후반 음수 16건/정확히 0인 6건, 한쪽 완료청산 0인 32건이다. 별도 분기 864원장은 각각 52/132/53/786건이다. SOXQ PC 10bp의 2023Q1은 전반 +0.7526%·완료1/후반 0%·완료0인데 flat 경고는 후반을 “검증 구간은 손실”로 표시한다. 비용쌍·상관 ETF·겹치는 창은 독립 표본이 아니다. 앞선 감사와 동일 SHA의 기존 원장이며 새 v22 실행 결과가 아니다. [연속 원장](research/continuous-replay-20261018/continuous-results.jsonl.gz), [별도 분기 원장](research/continuous-replay-20261018/reset-results.jsonl.gz), [고정 설계](research/continuous-replay-20261018/design.json).


**2026-10-29 완료:** v23 동일 장부 구간 진단·0보합/음수손실 경고 교정·작은 표본 설명을 flat/LTH/continuous와 현재/A-B에 동기화했다. 기존 경제값·체결·guard·TR14/사전warmup/봉간격과 LTH 평가불가 gate를 유지했다. [계약](ipc-commands.md#v23-동일-장부-구간-관측-진단), [회차 검증](tasks/immediate-backlog-20261029.md).

사전 고정12전략×3ETF×11fold×2비용의 context/evaluation각792개, 총1,584계좌를 별도 reset해 회고 walk-forward로 평가했다. PC보관78경제값/6분할 일치, phase3,324개·독립산술358,749건 검증. 별도 이전commit빌드는 아니며 이미 관측한 자료를 새OOS로 주장하지 않는다. 비용별396평가 계좌의 완료매도0은228/227, 모두20미만으로 신뢰 충분·우수전략·장중효능을 확정하지 않는다. [설계·결과·한계](research/phase-walk-forward-20261029/report.md).

<details>
<summary>이관한 원래 잔여 조건·판정 근거 (완료 전 상태)</summary>

- [x] **로컬 완료** **SOXQ-P2-16 — 구간 경고·표본 해석을 교정하고 별도 walk-forward를 평가한다.**
  - **의존성:** 기존 TR14·사전 ATR warmup·LTH 평가불가 gate는 보존한다. 경고/설명·fixture 교정은 즉시 가능하며 walk-forward는 생산 API·검증 자료·사전 선언에 의존한다. 과거 수치와 구현 근거는 [부분 완료 이력](completed-history.md#soxq-p2-16)에 보존했다.
  - 완료 조건: 이전 종가 포함 TR·사전 ATR warmup·봉 간격 유지; 표본 수/신뢰 부족 표시; 70/30 구간을 기간 정규화 없이 과최적화 확정에 사용하지 않고 별도 walk-forward 평가.
  - **현재 판정 근거:** 같은 ReplaySession 상태가 이어지는 70/30은 분리 학습·독립 OOS가 아니다. `phase_summaries`는 완료매도가 체결된 phase에 전체 배분 원가의 손익을 귀속한다. `overfit_warning`은 전반 수익>0에서 후반<=0 또는 후반<전반×0.5만 검사하며 구간 길이·완료 수를 보지 않는다. session의 최종 판정식을 continuous 경로는 유지하고 문구만 완화한다. [구간 집계·판정식](../src-tauri/src/trading/simulation.rs), [최종 호출](../src-tauri/src/trading/simulation/session.rs), [연속 문구 치환](../src-tauri/src/commands/strategy_preview/continuous.rs).
  - **남은 자동 구현 우선순위:** 누적수익 절반 조건으로 과최적화·학습/OOS를 단정하는 경고와 0%=손실 문구를 교정한다. 같은 finish를 쓰는 LTH 전용 preview를 포함해 flat/LTH/continuous·현재/A-B·legacy의 구간 길이·완료청산 표본·연속 상태 설명을 일치시킨다. 현재 flat/LTH UI의 “학습 구간(%) / 나머지는 out-of-sample 검증”·학습/In-sample/Out-of-sample 제목·표와 continuous의 앞/뒤·독립검증 아님 표현을 함께 정리한다. `BacktestPhaseView`/`BacktestPhase`에 관측수·노출·실현/잔여·경계보유 정보가 없으므로 필요한 설명/진단 계약을 보완하되 기존 회계·체결·원시 구간값은 보존한다. 길이가 다른 구간·후반0/손실·완료0/부분청산 fixture와 기본 거래/report·회계 불변을 검증하고 기존 TR·사전 ATR warmup·봉 간격 계약 및 LTH 평가불가/자료 gate를 유지한다. [현재 연구 UI](../src/pages/strategy/ui/strategyResearchPanel.tsx), [구간 TS 계약](../src/shared/api/strategyResearchTypes.ts).
  - **별도 walk-forward 완료 기준:** 생산 Rust API로 자료를 읽기 전 창·이전 정보·고정 후보·두 비용·설정 선택·warmup·수량·경계 보유 처리와 비중/노출·실현/잔여·MDD·차단·회전 기준을 선언한다. 과거 자료 재검토를 새 OOS로 부르지 않으며, 이어지는 장부의 사후 70/30과 분리한다.
  - **해석 위험·검증 의존성:** 경고 없음을 신뢰 충분으로, 청산0을 성과 우수로 해석하지 않는다. 생산 Rust 구현·fixture와 검증 가능한 자료·사전 선언에 의존하며, 로컬 자동 구현/검증에 새 사용자 승인 게이트를 추가하지 않는다.



</details>

### P3-REFACTOR-01

원문 207–216행의 부분 완료/과거 관측을 보존한다. 남은 세 대상도 아래 2026-10-29 회차에서 분리·검증했다.

  - [x] `src-tauri/src/commands/trading.rs`: lifecycle469/positions329/balance96/risk_restore96/daemon527/history281행으로 분리. IPC4개·scope·복원/시작 순서 보존, lib212개와 독립 본문 감사 통과. [기록](tasks/trading-module-split-20261018.md).
  - [x] `src/api/hooks.ts`: 19행 facade와 account/market/order/strategy/settings/history/trading/log 모듈49~255행으로 분리, 공개export87개/선언74개 동일. [기록](tasks/frontend-hooks-split-20261018.md).
  - [x] `src-tauri/src/trading/order/submission.rs`: positions/prepare/provider/records와 run 회귀로 분리. 예약·scope·manual API 경계를 검증했다. [기록](tasks/execution-run-reservation-20261018.md).
  - [x] `src-tauri/src/trading/order.rs`: facade·health·pending·legacy·execution run으로 분리하고 1050행 fills도 적용/collector/reconciliation/provider mode로 나눴다. [기록](tasks/execution-run-reservation-20261018.md).

**2026-10-29 완료:** Trading Page1056→474행·주문 model414/form234, Toss facade1093→13행·accounts111/diagnostics379/preflight236/orders371, LTHeditor993→413행·model343/picker177/preview161로 책임을 분리했다. LTH는 시작시 이미1000미만이었지만 요청된 책임 분리를 수행했다. 모든 변경/신규 코드파일1000미만, IPC/public API·query·state/effect/closure·JSX 동작을 보존했다.

Toss32개item의속성/시그니처/본문byte동일, Trading56문장/7helper·LTH68문장/5helper 및 재확장JSX AST동등을 확인했다. Rust831/strictclippy·TS/FSD/build와 분리전 합성거래5개, 분리후47개 UI검증이 통과했다. 실제 주문/계좌/키 접근은 없다. [정본 회차 기록](tasks/immediate-backlog-20261029.md).

## 2026-10-29 문서 재편 검증

- 대상: `6eeb0ea718138438296b832bcbbe3e218fc49564`에서 시작한 문서 변경. macOS 로컬 작업 트리는 시작 시 깨끗했으며 기존 다른 작업 폴더는 변경하지 않았다.
- 범위: TODO와 이력·생성 HTML·문서 포인터만 변경했다. 구현/전략 기본값·계정/API·거래·배포·의존성 파일은 변경하지 않았다.
- 이관 대조: 열린 최상위 19개(자동 구현11·판단4·실자료4), 유일 ID19개, 완료 최상위22개와 기존 완료 하위19개, 기존 링크74개 및 항목 상세의 보존을 확인했다. 번호 없는 열린 제목6개에는 대응표의 ID를 부여했다.
- 형식 검사: `docs-canon`의 `check_docs.py --changed` 1회에서 상대 링크·표·앵커·TODO 완료 체크 경고는 0이었다. 기존 문서11개의 등록부 도달성 누락으로 최초 종료 코드1이었으며, 위 참고 색인을 추가한 뒤 별도 Markdown 링크 그래프 대조에서 미도달0·변경 문서 링크 유효를 확인했다. 검사기를 반복 실행한 결과로 표기하지 않는다.
- 저장소 검사: `npm run check:todo`는 열린19개 원본/HTML 동기화 통과, `npm run check:project-map`은 새 이력 인벤토리 일치, `git diff --check`는 통과했다.
- HTML 검사: `node --test scripts/render-todo.test.mjs` 3개 통과. 초기 sandbox에서는 Chromium MachPort 권한 제한으로 브라우저가 시작되지 않았고, 허용된 로컬 환경에서 재실행해 전체 통과했다. 375px 화면의 전체 항목·키보드 검색/상태 필터·무스크립트 읽기·오버플로·브라우저 오류를 검사했다.
- 내용 검토: 별도 문서 감사로 집계·의존성·원래 ID/근거·실자료 미완료 경계의 보존을 확인했다. 이동 전 “아래 자동 구현 항목” 표현은 P1-RECOVERY-01 참조로 수정했다.
- 미실행 범위: 문서 harness에 따라 거래 앱/Rust 전체·전략 replay·실자료/실계좌 검증은 실행하지 않았다. 기존 기록의 테스트 수치를 이번 실행으로 주장하지 않는다. Git 커밋/원격 반영은 저장소 이력에서 확인한다.


## DEP-P3-03 — 5A HTTP 부분 완료

2026-10-29 시작 `4a9e250236e8773f0b1fe3194339cc0978a8a5a8`에서 reqwest0.12.28→0.13.5를 분리 전환했다. 8파일14생성점의 기존 TLS trust/provider와 호출부 옵션을 공통 builder로 보존했고 합성 HTTP/TLS10·전체lib841이 통과했다. RustSec 기존RSA1·Linux경고2는 남는다. 당시 WS0.24→0.30과 각 SHA CI 조건은 미완료였으며 아래 최종 기록에서 후속 완료를 확인한다. [구현·검사·제약 정본](tasks/immediate-backlog-20261029.md).


## DEP-P3-03 — 5B WebSocket 로컬 부분 완료

2026-10-29 HTTP `0dead386`의 Release Gate43(Rust841/UI344)·Security50 성공을 확인한 뒤 WS0.24→0.30을 별도 전환했다. 기존 wire/native roots/ring과4KiB read chunk를 유지하고, 교체 전 재현한 Close 응답 누락·송신 실패 연결 플래그 잔류를 수정했다. 합성WS/WSS14개가 통과했으며 자동 재연결·실계좌 연결은 추가하지 않았다. 이 기록 시점에는 WS 커밋 CI 확인 전이었으며 최종 결과는 아래와 같다. [최종 검사·게시 정본](tasks/immediate-backlog-20261029.md).


## DEP-P3-03 — HTTP·WebSocket 전환 완료

2026-10-29 두 전환을 별도 커밋과 검사로 완료했다. 5A `0dead386aa0e1270cff4092009b253dc0f37e8e5`는 Release Gate43·Security50 성공, 5B `270296db196ee7191e5c4848b1346095b1061dc7`는 [Release Gate44](https://github.com/CountJung/KISAutoTrade/actions/runs/40837592217)·[Security51](https://github.com/CountJung/KISAutoTrade/actions/runs/40839169041) 성공이다. 5B에서 Linux Rust855개·UI344개와 frontend/Toss OpenAPI 검사를 통과했다. 기존 RSA 예외와 Linux RustSec 경고2개는 유지한다.

원래 완료 조건인 합성 HTTP/TLS의 query·헤더·JSON·쿠키·오류와 mock WS의 승인·구독·수신·ping/pong·종료·명시 재연결/재구독, Rust fmt/strict clippy/check/test/build 및 해당 CI를 확인했다. 실제 KIS/Toss 인증·실계좌·자동 재연결·OS 신뢰 변경은 수행하지 않았다. 완료 근거와 미실행 세부 경계는 [회차 기록](tasks/immediate-backlog-20261029.md)을 따른다. 다음 즉시 항목은 DEP-P3-04이며 전체 요청 완료는 아니다.


## DEP-P3-04 — 서버 전환 로컬 부분 완료

2026-10-29 `270296db` 이후 axum0.7.9→0.8.9, tower-http0.5.2→0.7.1을 별도 단계로 전환했다. production 라우터의 70 URL/74 binding과 보안·handler 계약을 유지하며 13 capture 문법을 변경했다. 각 단계 서버24개와 최종 전체 Rust873개, strict clippy/check/fmt/custom-protocol build·타입/FSD/웹 빌드를 통과했다. 실제 provider와 모든 handler 성공 경로의 실행은 아니며 [회차 기록의 범위](tasks/immediate-backlog-20261029.md)를 따른다. 같은 SHA의 CI 확인 전까지 TODO 상위 ID를 유지한다.


## DEP-P3-04 — 서버 전환 완료

2026-10-29 커밋 `653377ce4ba49bc679d1762fe0f5d6ec2409fb74`를 정상 푸시·원격 SHA 대조하고 같은 SHA의 Release Gate45·Security52 성공을 확인했다. 위 로컬 계약 검증과 Linux Rust873·frontend·Toss OpenAPI·UI CI를 완료해 TODO 상위 ID를 닫았다. 실제 provider/계좌·운영 배포를 수행한 결과는 아니다. [최종 CI 링크와 범위](tasks/immediate-backlog-20261029.md).


## DEP-P3-05 — React·MUI 전환 로컬 부분 완료

2026-10-29 `653377ce` 뒤 React19.3.1/types19.3.0→MUI7.3.11→MUI9.5.0을 단계적으로 전환했다. Grid/slot/system sx 변환 밖 금융 AST·콜백·scope/preview 정책을 보존했다. 타입/FSD/clean lock 설치·경고 없는 production build와 전체UI349개, 별도 개발UI5개·Tauri custom-protocol build를 통과했다. 당시 같은 SHA의 원격 CI 확인을 남겨 상위ID를 TODO에 유지했다. 최종 완료는 아래 절을 따른다. [단계별 근거·실행 경계](tasks/immediate-backlog-20261029.md).


## DEP-P3-05 — React·MUI 전환 완료

2026-10-29 UI 전환 `f15f192fee105600d79468291334fb10f1495e9f`와 공개 Toss 명세 CI 보정 `99b85eb3cdc63deceb7ec60c325559607f98f8ce`를 정상 푸시·원격 SHA 대조했다. 최종 SHA의 Release Gate47·Security54가 모두 성공해 TODO 상위 ID를 닫았다. React19→MUI7→9 단계별 합성UI·최종349개와 개발QA5개, Rust 통합 빌드 및 Linux CI 검증을 구분해 기록했다. 원래 앱 startup·계좌·실거래·Windows/native 실동작 전체를 검증한 결과는 아니다. [CI 링크와 정확 범위](tasks/immediate-backlog-20261029.md).


## DEP-P3-06 — Vite·React plugin 전환 완료

2026-10-29 Vite6→7→8.3.5/plugin-react6.1.2·Oxc/Rolldown 전환을 `06a84fe32fa37dd0f31a13c21f13efb6dc4e8ac1`로 정상 푸시·원격 SHA 대조했다. clean 설치·target/debug4조합·전체UI349와 Tauri custom-protocol 빌드, 승인된 별도 WKWebView 차트·테마·입력·종료/재실행을 검증했다. 같은 코드 SHA의 Gate48·Security55도 성공했다. 알려진 calendar fixture 차단1건, 원래 Tauri startup/IPC·Windows·macOS12·CSP negative control 등의 제외 범위와 캡처 provenance는 [회차 기록](tasks/immediate-backlog-20261029.md)에 보존한다. 실제 계정·운영 DB·거래 검증이 아니다.


## DEP-P3-07 — 나머지 의존성 평가·선별 전환 완료

2026-10-29 `fe3a3f8a8836c7c20745b3a7c7ea7e09e68ccda4`로 sha2.11.1/thiserror2.0.21과 기존 hex0.4.3 재사용을 정상 푸시·원격 SHA 대조했다. keyring3/RGL1은 실제 초기화/backend·사용 이익을 검토해 유지·삭제 없음으로 판정했다. 구버전 및 호환 baseline과 새 버전의 같은 literal14개, 로컬 Rust882(실DB5제외)·전체UI349·strict clippy/build를 통과했다. 같은 코드 SHA의 Gate49·Security56도 성공했다. 최초 LowerHex 실패, 입력/생산 본문 동등성, 실제 DB/최소Rust/타플랫폼 제외는 [9회차 기록](tasks/immediate-backlog-20261029.md)에 보존한다.


## DEP-P3-08 — TypeScript·패키지매니저 평가 완료

2026-10-29 `adf0e5a4e4635e1e3cea9540df3016e6f60b070d`로 TypeScript7.0.2를 정상 푸시·원격 SHA 대조했다. TS5.9→6.0.3→7을 격리 비교해 원문130개·node1개 검사 범위와 strict·동일dist를 유지했다. npm11 유지·pnpm 미채택을 독립 판정했고 global 도구는 바꾸지 않았다. clean 설치/타입·UI349와 후속 짧은 loader 검증을 구분하고 Gate50·Security57 성공을 확인했다. 같은 프로젝트의 warm5회 앱 검사 중앙값은 TS6 2.1884→7 0.5722초이며 전체CI·순수CPU·설치크기 개선으로 확대하지 않는다. 초기 도구 안내·수정·지원환경 미검증은 [10회차 기록](tasks/immediate-backlog-20261029.md)에 보존한다.


## UIUX-20261030

2026-10-30 UI/UX 전문가 역할 구성·현재 흐름 검토와 개선 백로그 작성을 수행했다. [역할 TOML](../.codex/agents/ui_ux_expert.toml)·[사용법](agent-tooling.md#uiux-전문가-사용)은 기존 구성을 따르며 모델/권한 재정의는 없다. 이번 세션은 일반 서브에이전트가 TOML을 읽고 같은 지침으로 감사했다. 새 Codex 세션의 자동 역할 탐색은 별도 미검증이다.

코드 기준은 `7f2ec166a47d88508f2e72e10ab6355f9e9a585e`다. 현재 dist 10개 파일 SHA를 직전 TypeScript7 검증의 `dist-final-hashes.json`과 전부 대조했다. Rust/Tauri를 실행하지 않고 정적 파일만 별도 `127.0.0.1` 임시 서버에서 서브했다. Vite/프로젝트 env 로딩 없이 새 headless Chromium context·service worker 차단·외부 HTTP/WebSocket 거부·모든 API 쓰기 403·미지정 API 503으로 격리했다. `strategyResearchFixture.ts`의 합성 KIS 모의 자료를 사용하고 시작/정지·주문/취소·실전동의를 누르지 않았다.

초회 02:25:51~02:26:08 UTC 8개와 후속 02:28:23~02:28:29 UTC 4개는 **화면 관측 시나리오 12개**이며 개선 구현 또는 통과 테스트12개라는 뜻이 아니다. 실제 화면 PNG와 요청/DOM을 부모가 열어 확인했고, 전문가도 P1·Log 주요 증거를 독립 대조했다.

| 관측 | 화면·주입 상태 | 확인과 한계 |
| --- | --- | --- |
| P1-UX-01 | `/` 1280×900 light, trading/status503×3·별도 응답 보류 | ‘대기 중/대기’·시작 버튼과 상태 확인 중 문구 공존; 데몬 실제 상태·시작 결과를 검증한 것은 아님 |
| P1-UX-02 | `/trading` 같은 크기/light, 국내 balance503×3·응답 보류 | ‘보유 종목 없음’; 해외 실패/기존 자료 후 재조회는 소스 추론 |
| P1-UX-03 | `/history` 같은 크기/light, 조회 GET·stats503×3·trades 정상빈 배열 | 오류 안내 없이 +0원/0건/0.0%; 손익 산식·실제 원장 정합성 판정은 아님 |
| P2-UX-04 | `/log` 같은 크기/light, 빈 성공·후속 recent-logs503×3 | 둘 다 0건/빈 패널; 필터0·기존 자료 후 오류는 소스 추론 |
| 반응형·정보구조 | `/settings`, `/strategy` 375×812 dark | 문서 가로 overflow0, Settings 장문 섹션·목차 없음; 일반 모바일 오버플로 결함으로 확대하지 않음 |

초회 Settings의 Tab 샘플이 log-config POST1회를 유발했으나 403으로 차단했다. auto-budget GET fixture 누락503도 같이 남긴다. 이를 운영 설정 변경이나 제품 결함으로 세지 않는다. 누락 mock을 보완한 후속 Settings는 입력/Tab 없는 관찰용 자동 캡처로 분리해 쓰기 시도/미지정/외부 요청/페이지 예외0을 확인했다. 전체12관측의 외부 페이지 요청·페이지 예외는0이며 주입 503/403의 브라우저 resource 오류는 의도된 결과로 보존한다. 모든 context/브라우저/임시 서버는 종료했다.

원본은 Git 제외 `.cache/uiux-review-20261030/`의 `audit-first-pass.mjs`, `audit.mjs`, 두 로그·`report-first-pass.json`/`report-followup.json`·동명 PNG다. 역할 검토는 `/tmp/kis-uiux-expert-20261030.md`에도 남겼다. 실제 Tauri IPC·Toss 상태의 실행·스크린리더·전반 명도대비·200% 확대·Windows/macOS 최소버전은 미검증이다. 기존 UI349 통과를 이번 UX 검증 성공으로 재사용하지 않았다.

새 작업은 [TODO](todo.md)의 즉시9개(P1-UX-01~03, P2-UX-01~06)와 판단1개(P2-UX-07)로 정리했다. 역할 구성과 검토만 완료했으며 후보 구현은 전부 미착수다. 기존 결정/실자료/SQLx 10개·지난 즉시 10회차 완료는 보존했다.

검증: 역할 TOML 5개 파싱·신규 역할 필드/권한 불변, 기존 TODO10개 본문 보존·고유ID20개, HTML freshness·지도·그래프·diff가 통과했다. 기존 TODO 렌더러 테스트 3개(Chromium 좁은 폭·키보드 검색/필터·JS 없는 읽기 포함)를 통과했고 독립 리뷰에 필수 P0/P1/P2 지적은 없었다. 제품 소스·의존성·workflow는 변경하지 않아 Rust/전체UI349/배포 CI를 새로 실행하지 않았다.


## P1-UX-01

2026-10-30 범용 QueryStatusSummary를 추가해 Sidebar와 Dashboard의 미확인·첫 실패·이전 실행/정지 자료·마지막 정상 확인 시각·읽기 재시도를 일치시켰다. query 옵션과 금융 handler/버튼 조건은 원문 동등성을 유지했다. 앱/config 타입·FSD·격리 production build와 최종 새 상태 테스트5개, 기존 dialog/Select/차트5개, 기존 합성 Sidebar 시작/정지1개의 고유11개 검증을 통과했다. 초회3·중간9·후속6 실행은 중복 합산하지 않는다. 160px Sidebar와375px 다크 메뉴 캡처를 확인했고 독립 리뷰의 문구 기대 정합 P2를 수정·재검증해 잔여 필수 P0/P1/P2는 없다. 사용자 판단·실자료 조건은 대신 닫지 않는다. [회차 기록](tasks/uiux-improvements-20261030.md)에 검증/미검증·게시 상태를 보존한다.

### 원래 TODO 완료 조건·근거 보존

P1-UX-01 — 실행 상태 미확인·조회 실패·이전 상태를 ‘대기’와 구분한다.
- 근거/대상: `src/widgets/sidebar/ui/Sidebar.tsx:56-68,118-131`은 `useTradingStatus`의 data만 받아 `undefined ?? false`를 ‘대기 중’과 시작 아이콘으로 렌더한다. `src/api/tradingHooks.ts:14-23`은 주기 조회 query이지만 Sidebar는 loading/error/dataUpdatedAt을 소비하지 않는다. Dashboard의 상태 chip·시작 버튼도 `src/pages/dashboard/ui/Page.tsx:460-488`에서 같은 boolean 해석이다. 실제 건강 패널이 없는 것이 아니라 **상태 조회 자체가 아직 성립하지 않았거나 실패했을 때 전역 요약을 구분하지 않는 문제**다.
- 현재 화면 보강: 위 `sidebar-status-error` mock과 PNG, 후속 `sidebar-status-loading`에서 같은 표현을 확인했다. 503 노출 또는 응답 보류 초기 프레임의 관측이며 실제 데몬 상태나 실제 시작 성공의 증거가 아니다.
- 영향/우선순위: P1. 사용자가 실제 실행 중단 사실과 단순 통신 미확인을 혼동할 수 있다. 현재 backend guard가 우회되거나 주문이 안전하지 않다고 입증한 것은 아니다.
- 개선 방향: 확인 중/확인 실패/마지막 정상 상태를 텍스트·아이콘으로 구분하고 마지막 정상 확인 시각 및 읽기 재시도 경로를 표시한다. 기존 health 상세와 연결해 중복 상태를 만들지 않는다. 실제 시작·정지 동작/권한·guard 정책은 보존한다.
- 의존성: 기존 tradingStatus query key·polling/변이 캐시 계약 재사용. 미확인 때 시작 버튼의 금융 동작 정책을 새로 바꾸려면 P2-UX-07에서 별도 판단한다.
- 완료/검증: 지연·첫 실패·정상 실행/정지·성공 후 재조회 실패의 synthetic fixture에서 Sidebar와 Dashboard가 같은 상태를 표시한다. 미확인이 ‘대기/정상’으로 단정되지 않고 이전 데이터임이 보인다. 문구/재조회 키보드 경로를 확인하며 start/stop·provider 호출0을 기록한다.


## P1-UX-02

2026-10-30 Trading의 KIS 국내·해외 잔고에 독립 조회 상태·마지막 정상 시각·읽기 재시도를 연결하고 미조회/첫 실패를 정상 빈 목록과 분리했다. 같은 scope의 query 자료만 이전 자료로 유지하며 기존 scope fence·Toss 분기·선택/주문 본문을 보존했다. 최종 새 holdings10개+기존 주문폼5개의 고유15개가 production mock UI에서 통과했다(27.8초). 국내/해외 각각 지연·첫 실패·빈/보유 성공·성공 후 실패·키보드 복구·연속2회 재조회·다른 시장 읽기 불변을 확인했다. 앱/config 타입·FSD·diff와 보호원문9항목 대조가 통과했고375px 다크 이전 해외잔고 캡처를 확인했다. 테스트 외부 guard P2를 보강·재검증해 최종 독립 리뷰에 잔여 필수 P0/P1/P2는 없다. 사용자 판단·실자료 조건은 대신 닫지 않는다. [회차 기록](tasks/uiux-improvements-20261030.md)에 검증/미검증·게시 상태를 보존한다.

### 원래 TODO 완료 조건·근거 보존

P1-UX-02 — Trading의 KIS 잔고 로딩·실패를 ‘보유 없음’과 분리한다.
- 근거/대상: `src/pages/trading/ui/useTradingOrderModel.ts:148-155`는 KIS 국내/해외 잔고에서 data만 꺼내지만 Toss 보유는 loading/error를 꺼낸다. `src/pages/trading/ui/Page.tsx:151-166`은 KIS data가 없으면 `[]`를 넘기고, `kisPanels.tsx:26-31,97-102`는 이를 ‘보유 종목 없음/해외 보유 종목 없음’으로 렌더한다. Toss의 별도 상태 분기(`Page.tsx:88-100`)는 이미 있으므로 유지한다.
- 현재 화면 보강: 위 `trading-balance-error`에서 국내503인데 ‘보유 종목 없음’을 확인했다. 후속 `trading-balance-loading` 초기 프레임에서도 같은 빈 표시를 확인했다. 동시 appConfig 초기 로딩·프로파일 미선택을 별도 문제로 세지 않는다. 해외 실패와 stale-data 재조회 상태는 소스 기반 후보로 남긴다.
- 영향/우선순위: P1. 미조회·인증/연결 실패를 정상 잔고0으로 해석하게 할 수 있다.
- 개선 방향: KIS 두 query의 loading/error/마지막 성공 data를 보유 패널까지 전달하고 로딩·조회 불가·정상 빈 목록·이전 자료를 구분한다. 기존 데이터를 무조건 버리거나 잔고 수량을0으로 치환하지 않는다.
- 의존성: query key, broker gate, polling, 보유 선택이 채우는 주문 폼 의미는 그대로 둔다. 자격증명 UI 확대 없이 재조회/기존 설정 안내를 활용한다.
- 완료/검증: 국내/해외 각각 지연·실패·빈 성공·보유 성공·보유 성공 후 실패를 mock으로 확인한다. 실패 시 ‘보유 없음’ 단정이 없고 국내와 해외의 상태를 독립적으로 읽을 수 있다. Toss 분기 회귀와 실제 외부 요청0을 확인한다.


## P1-UX-03

2026-10-30 History의 통계와 체결 목록을 독립 조회 상태·마지막 정상 시각·읽기 재시도로 표시하고, 미조회/실패와 실제0을 분리했다. 날짜 입력의 미적용 변경과 실제 적용 기간을 명시하며 같은 query의 이전 자료만 보존한다. 원래 손익/승률·통화/trace·날짜 전달과 query 계약은 유지했다. 새 History6개+기존 live-daily13개의 고유19개가 첫 실행에서 통과했다(17.2초). 초기 조회 전/지연·통계 실패/체결 성공·실제0/빈 성공·양쪽 실패·각 이전 자료/시각·독립 키보드 재시도와 반복2회·미적용 입력/확정 기간을 검증했다. 앱/config 타입·FSD·보호원문11항목·독립 UI 리뷰를 통과했고 잔여 필수 P0/P1/P2는 없다. 사용자 판단·실자료 조건은 대신 닫지 않는다. [회차 기록](tasks/uiux-improvements-20261030.md)에 검증/미검증·게시 상태를 보존한다.

### 원래 TODO 완료 조건·근거 보존

P1-UX-03 — History 통계 조회 실패를 0원·0건·0%로 표시하지 않는다.
- 근거/대상: `src/pages/history/ui/Page.tsx:53-65`는 stats query의 data/isLoading만 사용하며 미존재 값을0으로 요약한다. `143-159`는 로딩이 끝나면 그 값을 보여준다. `171-176`의 tradesError는 별도 체결 query에만 적용되므로 통계 단독 실패는 표시되지 않는다.
- 현재 화면 보강: 위 `history-stats-error`에서 통계503과 +0원/0건/0.0%를 JSON·PNG로 확인했다. 실제 원장/손익 값의 정확성이나 다른 통계 경로를 검증한 것은 아니다.
- 영향/우선순위: P1. 기록을 확인하려는 사용자가 조회 실패를 실제 손익/거래0으로 오독할 수 있다. 기존 손익 산식 자체의 맞고 틀림은 이 검토 범위가 아니다.
- 개선 방향: 통계 query의 실패·미확인/이전 자료·성공0을 명시적으로 구분한다. 요약과 체결 목록 각각의 상태 및 실제 적용 조회기간을 표시하고 해당 실패만 재시도 가능하게 한다. 원장/손익·승률 계산 정책을 바꾸지 않는다.
- 의존성: 기존 `useStatsByRange`/`useTradesByRange` query와 날짜 범위 전달 계약.
- 완료/검증: stats500+trades성공, stats성공0+trades빈, 둘다실패, 초기조회·지연의 mock matrix를 확인한다. 실패 요약은 0수익/0승률로 단정되지 않고 성공0과 식별되며 재시도 후 실제 적용기간과 결과가 일치한다.


## P2-UX-01

2026-10-30 Trading/Strategy 검색 결과와 KIS/Toss 양시장 보유 행에 이름·코드가 있는 native 선택 버튼과 focus 표시를 추가했다. 내부 Tab 이동 중 결과를 유지하고 Escape/검색 선택 후 입력으로 focus를 돌린다. 순수 RowSelectionButton을5곳에서 재사용하고 Strategy 검색 표만52행 모듈로 분리해 Page982행을 유지했다. 기존 금융/선택 state·모델·query는 보존했다. 새 키보드/포커스10개+기존 form5개+holdings10개의 고유25개가 최종 통과했다(27.8초). 검색 내부 blur·Escape/선택 후 닫힘·외부 blur·포인터 코드셀·KIS/Toss×KR/US 선택을 확인했다. 375px 다크 긴 이름의 full accessible name·focus ring·문서 overflow0 캡처를 확인했고 앱/config 타입·FSD·보호14항목/전략 함수15개 AST 동등성을 통과했다. 최종 독립 UI/helper·참조 검토에 잔여 필수 P0/P1/P2는 없다. 사용자 판단·실자료 조건은 대신 닫지 않는다. [회차 기록](tasks/uiux-improvements-20261030.md)에 검증/미검증·게시 상태를 보존한다.

### 원래 TODO 완료 조건·근거 보존

P2-UX-01 — 종목 검색 결과·보유 종목 선택을 키보드로 수행한다.
- 근거/대상: `src/pages/trading/ui/kisPanels.tsx:50-55,121-126`, `trading/ui/Page.tsx:118-123,295-308`, `strategy/ui/Page.tsx:577-589`의 선택 행은 클릭 handler만 있는 TableRow다. 해당 행에 tabIndex/키 handler/실제 button이 없다. 국내 입력의 blur180ms 닫힘과 mouseDown preventDefault(`Trading:217-226,264-270`, `Strategy:535-538,564-567`)도 포인터 흐름 중심이다. 해외 티커 Enter 검색과 기존 MUI dialog/Select 키보드 지원은 이미 있으므로 전체 키보드가 없다고 주장하지 않는다.
- 영향/우선순위: P2. 마우스를 쓰기 어려운 사용자는 이미 나온 결과 선택이나 보유 클릭을 통한 폼 채우기 경로를 이용하기 어렵다.
- 개선 방향: 행 안 명명된 선택 button 또는 일관된 combobox/listbox 패턴을 정하고, 검색 결과 이동·선택·닫기와 포커스 복귀를 지원한다. 표시명과 종목코드를 함께 알리고 기존 선택 후 폼값/시장/scope 유지.
- 의존성: 현재 검색 훅·선택 closure 유지. 공용화는 실제 입력/선택 의미가 같은 부분만 검토한다.
- 완료/검증: 키보드만으로 KR/Toss 보유 및 국내 검색 결과를 선택하고 폼의 정확한 종목/시장/수량을 확인한다. Arrow/Enter/Escape 또는 명시 button의 Tab/Enter 동작, blur 중 결과 소실 방지, focus visible, 긴 이름·좁은 폭 및 포인터 회귀를 mock으로 검사한다. 주문 제출0.


## P2-UX-02

2026-10-30 Sidebar 너비와 Log 높이에 명명된 키보드 separator를 추가했다. 방향키·Home/End·Enter 기본값 복원과 focus 표시를 제공하고 명시한 target을 즉시 저장한다. 기존 pointer delta/end·저장 키/범위·반응형 flex 계약은 보존했다. 신규 resizer5개와 기존 scroll5개의 고유10개가 최종 통과했다(4.7초). 반복키·경계·기본값·저장/route/reload·다른 pref 불변·pointer와 반응형 표시를 확인했다. 동일 Log 사례의 geometry 부분 재실행1개도 통과했으며 중복 합산하지 않는다. 앱/config 타입·FSD·보호12항목 및 AppShell12/Log4 callback AST 대조를 통과했고 최종 독립 리뷰에 필수 P0/P1/P2는 없다. 사용자 판단·실자료 조건은 대신 닫지 않는다. [회차 기록](tasks/uiux-improvements-20261030.md)에 검증/미검증·게시 상태를 보존한다.

### 원래 TODO 완료 조건·근거 보존

P2-UX-02 — 패널 크기 조절에 키보드 대안과 현재값을 제공한다.
- 근거/대상: `src/shared/ui/LayoutResizer.tsx:27-67,71-78`은 mouse down/move/up에만 연결된 Box이며 focus/keyboard/명명·현재값 API가 없다. `widgets/app-shell/ui/AppShell.tsx:328-334` Sidebar, `pages/log/ui/Page.tsx:178-184` 로그 높이 등에 사용된다. 앱 main의 native scroll과 custom rail은 별도이므로 ‘스크롤 전체가 키보드 불가’로 확대하지 않는다.
- 영향/우선순위: P2. 포인터 없이 밀도/읽기 폭을 조절하기 어렵고 현재 크기·방향을 알 수 없다.
- 개선 방향: 명명된 separator의 focus/화살표/Home/End 지원 또는 명시 크기 입력·기본값 복원 대안을 제공한다. 기존 min/max·저장 키·드래그 callback 보존.
- 의존성: 각 부모의 clamp·persistentLayout 계약을 먼저 대조. touch 필요 여부는 pointer 추가 전 별도 runtime 확인.
- 완료/검증: Sidebar 폭과 Log 높이를 Tab으로 찾아 keyboard로 조절, min/max·기본값 복원·재방문 persistence·포인터 드래그를 확인한다. 접근 가능한 이름/방향/현재값과 focus visible를 확인하며 금융 호출0.


## P2-UX-03

2026-10-30 History 거래별 상세 보기와 page-local Dialog를 추가해 같은 체결의 전체 기록 시각·ID·통화·가격/금액/수수료/슬리피지·전략·trace를 좁은 화면에서도 읽게 했다. 기존 formatter 결과를 전달하며 선택은 ID+적용 기간만 보관한다. polling에서 대상이 사라지면 선택도 폐기해 자동 재개방을 막는다. 신규 상세4개+기존 History 상태6개+일봉13개의 고유23개가 최종 통과했다(23.3초). 375/1280폭에서 KRW/USD fallback·환산/부호·같은 종목 다중 체결·긴 ID·null/빈 trace·Tab/Enter/Escape/닫기/focus 복귀·키보드 스크롤과 overflow를 검사했다. 격리 Chromium의 실제 tab zoom2에서 CSS1280→640/높이1000→500·DPR1→2·visualScale1과 상세 접근을 확인했다. 앱/config 타입·FSD·원래 함수13개 및 보호15항목 대조를 통과했다. 최종 루트·UI 전문가 독립 리뷰에 필수 P0/P1/P2는 없다. 사용자 판단·실자료 조건은 대신 닫지 않는다. [회차 기록](tasks/uiux-improvements-20261030.md)에 검증/미검증·게시 상태를 보존한다.

### 원래 TODO 완료 조건·근거 보존

P2-UX-03 — 좁은 History에서 숨긴 거래 상세를 다시 열 수 있게 한다.
- 근거/대상: `src/pages/history/ui/Page.tsx:183-192,198-199,218-242`는 xs에서 시간·가격·전략을, md 미만에서 금액·수수료·슬리피지를 숨긴다. xs ProviderTraceChips 대안은 `206-208`에 있지만 숨긴 경제값/시간을 여는 상세 컨트롤은 없다.
- 영향/우선순위: P2. 좁은 화면에서 같은 종목/수량의 여러 체결을 구분하거나 비용·시각을 대조하기 위해 화면 폭을 바꿔야 한다.
- 개선 방향: 거래별 명명된 ‘상세 보기’ disclosure/dialog 또는 카드 보조행으로 숨긴 값·통화·시간·전략·trace를 제공한다. 테이블의 현재 간결한 요약과 기존 표시 산식은 유지한다.
- 의존성: 기존 trade 데이터만 사용. P1-PROVIDER-01의 실제 원장 검증을 완료 처리하지 않는다.
- 완료/검증: 375폭과 1280폭, 200% 확대를 기준으로 같은 mock 체결의 모든 숨긴 항목에 접근 가능하고 KRW/USD·수수료·슬리피지·같은 종목 다중 체결을 정확히 식별한다. Tab/Enter/Escape·focus 복귀·가로 넘침을 검사한다. 이것은 목표 검증이며 현재의 화면 실패를 실측한 주장은 아니다.


## P2-UX-04

2026-10-30 Log에서 placeholder와 실제 응답을 구분해 최초 로딩/실패·정상 무기록·필터 결과0·이전 자료와 마지막 정상 시각을 표시했다. 읽기 재시도·검색 label·필터 지우기를 추가하고 5초 polling의 건수/시각/fetching은 live 영역 밖에 뒀다. 기존 query/tail300·logger·행/trace·필터·scroll/follow·resize를 보존했다. 신규 Log5개+기존 resizer5개의 고유10개가 최종 통과했다(13.3초). 지연/500/성공[]/필터0/이전0·150행/반복 키보드 복구와 필터 초기화, 원래5초 poll2회의 정확한 요청 증가·위쪽scrollTop0·하단follow 복귀·status DOM mutation0을 확인했다. 앱/config 타입·FSD·원래6개 함수/표현과 보호18항목 AST/원문 대조를 통과했고 루트·UI 전문가 독립 리뷰에 필수 P0/P1/P2는 없다. 375px 다크 실패와 desktop 이전자료 PNG를 직접 열었다. 사용자 판단·실자료 조건은 대신 닫지 않는다. [회차 기록](tasks/uiux-improvements-20261030.md)에 검증/미검증·게시 상태를 보존한다.

### 원래 TODO 완료 조건·근거 보존

P2-UX-04 — Log의 조회 실패·기록0·필터 결과0을 구분한다.
- 근거/대상: `src/pages/log/ui/Page.tsx:60,76-80,102-110,130-175`는 query error를 읽지 않고 기본[]·결과 건수와 map만 표시한다. `src/api/logHooks.ts:43-50`은 placeholderData[]를 쓰며 실패 상태의 부모 분기나 빈 결과 안내가 없다. 검색 필드는 placeholder만 있어 명시 label도 보완 대상이다. 현재 auto-follow는 사용자가 위로 스크롤하면 멈추는 동작(`Log:62-74`)이 이미 있으므로 없는 기능으로 다시 제안하지 않는다.
- 현재 화면 보강: 초회 `log-empty`와 후속 `log-error-final`의 JSON/PNG를 대조했다. 후속 GET503 3회 뒤에도0건/빈 패널만 보이며 실패 안내가 없었다. 필터0·기존 자료 후 오류는 아직 소스 기반 후보다.
- 영향/우선순위: P2. 비어 있는 로그가 정상 무기록인지 필터나 통신 실패인지 구분하기 어렵고 장애 조사 다음 행동이 불명확하다.
- 개선 방향: 최초 로딩·불러오기 실패/이전 자료·기록 없음·검색/level 불일치 메시지와 읽기 재시도·필터 지우기를 제공한다. 검색 label, 결과 건수의 과도하지 않은 상태 안내를 추가한다. tail/poll 주기·logger 자체는 변경하지 않는다.
- 완료/검증: 지연·500·성공[]·비어 있지 않은 원본에 필터0·실패 후 재시도 matrix를 mock으로 확인한다. 오류가 0건으로만 보이지 않고 검색명 접근·필터초기화·사용자 스크롤 고정이 유지된다. 로그 내용·민감 metadata를 늘리지 않는다.
