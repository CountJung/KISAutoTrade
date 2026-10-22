# 작업 기록: P2-11 추세 필터의 사전 설계

## 1. 메타데이터

- 작업 ID: SOXQ-P2-11
- 담당자: 부모 Codex와 사전설계 연구 역할
- 상태: In Progress — 문서 준비 단계이며P2-11 구현·새 후보 실행은 미완료
- 작성/갱신일: 2026-10-23
- 대상 브랜치: fix/mac-security-storage-2026-10-06
- 관련 문서/PR: [TODO](../todo.md), [설계](../research/trend-filter-policy-20261023/design.json), [연구 계약](../research/trend-filter-policy-20261023/protocol.md), [보고서](../research/trend-filter-policy-20261023/report.md). PR 없음.
- 위험 등급: 이번 문서만의 변경은 낮음. 향후 전략·소유/보호 구현은 높음이며 별도 검증한다.

## 2. 목표와 비목표

### 목표

기존200/5/20·현재 포함·strict조건을 확인하고 실제 소유 대조/장기 이탈 확인2/수익5·되밀림3/보유63을 단독 비교할 사전 설계와 자료·비용·검증 한계를 고정한다.

### 비목표

이번 문서 작업에서 생산코드·전략 파라미터·계정 설정·기존 연구 원장을 바꾸지 않는다. 새후보 실행·주문·자동매매 시작·성과 채우기·P2-11 완료 표시·새 중복TODO를 수행하지 않는다.

### 사용자 완료 조건

- [x] 기준HEAD·입력전체/normalized hash와252이전/752평가 경계를 재확인한다.
- [x] 고정5종·30연속/30분할/360분기·비용/수량/시점/실제보유 계약을 문서화한다.
- [x] 새 후보 결과 미실행·이미 본 자료·비OOS·실제 운용 미검증을 명시한다.
- [ ] 실제 소유/선택청산·복원/엄격저장/두preview/UI구현과 모든 검증을 완료한다.
- [ ] 생산 비교와 독립 정산·문서/지도/리뷰 후 기존P2-11의 완료 여부를 판정한다.
- [ ] 부모가 검증된 변경을 현재 브랜치에 커밋·origin푸시하고 원격 상태를 확인한다.

## 3. 시작 상태

    git status --short: clean (사전설계 작업 시작 시 확인)
    HEAD: 81fa00d3aa50bc59606667568f36c4b686902412
    branch: fix/mac-security-storage-2026-10-06
    관련 기존 변경: P2-09 완료 커밋을 보존한다.

- [x] 사용자 변경과 할당 범위를 구분했다. 다른 작업자가 동시에 만드는 변경은 되돌리지 않는다.
- [x] .env, secure_config.json, profiles.json 등 민감파일을 읽지 않았다.

## 4. 조사 근거

| 확인한 파일/심볼 | 확인한 사실 | 영향 |
|---|---|---|
| mean_trend.rs::TrendFilterParams | 기본200/5/20 |200기준과 단기/중기 기본 유지 |
| TrendFilterStrategy::on_tick/moving_avg | 현재 포함 MA·price<longMA청산·strict진입 | 기존 장기 이탈을 중복 기능으로 만들지 않음 |
| on_live_tick/sync_position/reset | 실제수량/원가를bool로축약·가격버퍼유지 | 소유 교정/보호·관측단위를 분리 |
| strategy_preview/driver.rs::run_prepared_preview | 실제전략·단일회계·nextOpen공개순서 | 별도전략모델 없이 부모가 생산harness실행 |
| strategy-review-20261018/input.json | 전체hash·3normalizedhash·원본metadata·252/752 | 자료 승계와 재검증 범위를 구분 |
| docs/todo.md의P2-11·기존 전수/연속보고서 | 실현손실과 기말평가수익의 차이 | 청산/비용/노출과 잔여손익을 함께 비교 |

공식KIS/Toss 근거: N/A — endpoint/provider 변경·외부API조회를 수행하지 않는다.
미확인: 원본bytes이번재해시, 기업행동/조정·총수익·거래소달력/누락/coverage·feed통합성·장중최초발동/체결·운용안전성.

## 5. 영향 범위

### 예상 변경 파일

| 경로 | 역할 | 변경 이유 |
|---|---|---|
| docs/research/trend-filter-policy-20261023/design.json | 고정 기계판독 설계 | 실행 전5종·자료·가정·계약 |
| 같은 경로의protocol.md | 연구 계약 | 출처/hash·구간·상태·검증/한계 |
| 같은 경로의report.md | 연구 보고서 초안 | 기존근거/새결과미실행 분리 |
| docs/todo.md | 기존P2-11 아래 준비 상태 | 상위 체크·기능 설명은 보존하고 링크 추가 |
| AGENTS.md | 최근5건 요약 | 사전 설계/기존 감사·후보 미실행 표시 |
| docs/tasks/trend-filter-policy-20261023.md | 작업 기록 |10절과 미완료 검증 보존 |

### 계약 체크

- [ ] Frontend public API/FSD import: 이번 문서는N/A. 향후새설정UI구현·검증은미완료.
- [ ] Rust serde↔TypeScript타입미러: 이번 문서는N/A. 향후응답/설정변경시동기화필요.
- [ ] Tauri command↔lib.rs등록: 이번 문서는N/A. 신규IPC추가를결정하지않음.
- [ ] Web REST/transport: 이번 문서는N/A. 향후기존저장/두preview호환검증필요.
- [ ] query key/cache invalidation/backend event: 이번 문서는N/A. 신규요청/폴링추가없음.
- [ ] persisted JSON/DB schema및하위호환: 소유checkpoint정확한형식미결정. 확인값은복원0으로결정.
- [ ] docs/ipc-commands.md/프로젝트지도: 부모의 지도 역할이 이번 문서 추가를 반영하며 향후 구현 문서 게이트는 미완료다.

## 6. 금융·자동매매 안전 검토

실제주문가능성: 없음. 이번 범위는 할당된 연구/작업 문서와 후속 TODO·AGENTS 문서 갱신뿐이다.

- [x] 이번작업에서실전·모의주문/자동매매시작을실행하지않는다.
- [ ] BrokerScope보존: 이번문서는N/A. 향후구현에서broker/profile/account·exactowner회귀를확인한다.
- [ ] provider전수량/가격/통화/가능액/동의/risk검증: 이번문서는N/A. 기존경계를유지하며provider호출없음.
- [ ] 중복·반대·동시submission: 이번문서는N/A. 향후pending기원/실소유fixture미완료.
- [ ] timeout/rate limit/부분·불명확응답failclosed: 이번문서는N/A. 실제응답조회없음.
- [ ] pending/order/risk영속화실패차단: 이번문서는N/A. 향후기존경계보존검증필요.
- [ ] 비상정지/일일·연속손실/주문상태재시작우회방지: 이번문서는N/A. 생산코드변경없음.
- [x] credential/계좌/provider민감원문을문서/trace에남기지않는다.
- [x] preview/backtest한계·실현/잔여·노출·비OOS를명시한다.

잔여위험: 실제자료의장중체결/운용안전성과선택정책효능은아직평가하지않았다. 관측횟수를시장일·손실보장으로표현하지않는다.

## 7. 구현 계획

1. 기준HEAD와입력hash를재확인하고할당문서에새후보실행전설계를고정한다.
2. 부모가실소유/선택정책/엄격검증/두preview/UI를구현하고기준/비대상회귀를확인한다.
3. 부모가생산harness로30/30/360을실행하고독립정산·표본/비용/노출해석을감사한다.
4. 기본승격/기각/미평가를구분해기존P2-11에반영하고모든게이트뒤커밋·푸시한다.

롤백/비상중단: 이번문서변경만되돌릴수있으며운영설정·주문·외부상태를바꾸지않는다.

## 8. 도구 선택 기록

- Serena: 사용. 지침읽기/프로젝트활성화뒤mean_trend.rs정의/참조와run_prepared_preview를확인했다.
- Graphify: 미사용. 이번역할은문서사전설계이며구조리팩터링·helper이동없음.
- 별도역할: 부모가심볼/문서감사와기존원장감사를배정한다. 본역할은생산코드/기존결과를수정하지않는다.
- 한국어와필요한영어식별자만사용하고명령·문서·전송문장의언어혼입을점검한다.

## 9. 검증 계획과 결과

[검증 지도](../HARNESS_MAP.md)에따라준비단계와새구현검증을구분한다.

| 상태 | 명령/확인 | 실행결과 | 비고 |
|---|---|---|---|
| PASS | git rev-parse HEAD·git status --short |81fa00d전체hash·시작clean |이번설계기준 |
| PASS | Python입력/normalizedSHA·날짜경계 |전체입력/3종목normalized일치·252/752·정규화중복/역순0 |rawbytes재해시는미실행 |
| PASS | cargo test --offline -p kis-auto-trade trend_filter |부모보고: 실제3개통과·다른target0건합산제외 |기존구현집중검증이며새보호검증아님 |
| PASS | cargo check --offline --workspace --all-targets --locked |부모보고: 통과·경고0 |현재기준구현 |
| PASS | npx tsc --noEmit |부모보고: 통과 |현재기준구현 |
| NOT RUN | npm run check:fsd |이번문서범위N/A |향후UI변경시검증 |
| NOT RUN | cargo test --workspace --lib --locked |새구현전체회귀미실행 |기존3개집중과구분 |
| NOT RUN | 새보호focused test |미구현 |현재3개를신규검증으로쓰지않음 |
| NOT RUN | focused/전체Playwright |이번문서범위N/A |향후설정/current-A-B/모바일검증필요 |
| PASS | npm run check:project-map | 부모의 두 지도 pass 뒤 문서 역할이 최종 직접 재실행·통과 | 문서/감사 파일6개 반영 |
| NOT RUN | 생산연구harness30/30/360 |새후보결과없음 |부모가실제Rust경로실행 |
| PASS | 기존v12원장 독립 정산 | 연속6·분기72·42,958검사/오류0, 두 재실행 JSON동일·sourceFiles8개hash·변조4건 탐지 | 새 생산 실행/후보 검증과 구분 |
| NOT RUN | 새후보 독립원장정산·통계해석 | 새후보원장 없음 | 기존v12감사는 새후보 검증이 아님 |
| NOT RUN | provider/구조검사 |이번문서범위N/A |provider/구조변경없음 |
| PASS | npm run check:graphify | 부모 보고: source 최신·통과 | 소스 미변경 기준 |
| PASS | node --test scripts/render-todo.test.mjs | 부모 보고: 현재 할일 원본 기준3/3 | 이후 TODO 문서 추가의 최종 렌더는 별도 확인 |

실전주문: NOT RUN. 모의주문: NOT RUN. 후보성과/체결가능성/실제운용: 미평가·미검증.

## 10. 완료 보고

### 변경 요약

사전 설계 문서4개와 추가 할당된 TODO·AGENTS 문서를 작성하고 입력/기준/후보/상태·시점·비용·기말 보유·검증 한계를 고정했다. 문서준비는P2-11구현·연구완료가아니다. 새후보손익은아직없다.

### 변경 파일(절대경로)

- /Users/jsjmac/Workspace/MacWorking/KISAutoTrade/docs/research/trend-filter-policy-20261023/design.json
- /Users/jsjmac/Workspace/MacWorking/KISAutoTrade/docs/research/trend-filter-policy-20261023/protocol.md
- /Users/jsjmac/Workspace/MacWorking/KISAutoTrade/docs/research/trend-filter-policy-20261023/report.md
- /Users/jsjmac/Workspace/MacWorking/KISAutoTrade/docs/tasks/trend-filter-policy-20261023.md
- /Users/jsjmac/Workspace/MacWorking/KISAutoTrade/docs/todo.md
- /Users/jsjmac/Workspace/MacWorking/KISAutoTrade/AGENTS.md

### 기존 사용자 변경 보존 결과

시작clean을확인했고할당된 연구/작업 출력4개와 후속으로 추가 할당된 TODO·AGENTS 문서만 작성했다. 생산코드·기존자료/결과·다른역할변경을수정하거나되돌리지않았다.

### 알려진 제한/잔여 위험

소유checkpoint형식·raw검증상한·실제engine/input/sourcehash는구현/실행때확정한다. 확인연속값은영속복원0·새MA문맥재관측으로결정했다. 원본hash·조회일은기존기록승계이며raw재조회/재해시는없다. 기업행동·달력·자료범위·장중체결·운용안전성은검증하지않았다.

### 운영자 확인 필요

- [ ] N/A — 이번단계에서배포/설정변경/자동매매시작을수행하지않는다.
- [ ] N/A — 실전전모의·소액검증은별도승인절차이며이번단계에포함하지않는다.
- [ ] N/A — 기존비상정지/rollback/로그·provider대조운영게이트를유지한다.

기존 집중3개는 설정7주/실보유2주 매도 제안 등 현재 동작의 회귀이며 실잔량 교정 완료가 아니다. 소스 표현의 한계와 실제 경제 경로·운용 사고를 구분한다. 원래 세 MA 준비 gate를 유지하고 새 수익/나이 보호만 gate 밖에서 평가하도록 설계를 확정했다.

최종 기존 원장 감사는 [감사 JSON](../research/trend-filter-policy-20261023/existing-ledger-audit.json)과 [스크립트](../research/trend-filter-policy-20261023/existing-ledger-audit.py)에 고정했다. 저장된v12연속6/분기72에42,958검사·오류0이며 현재81fa00d/v21 재실행이나 새 후보 결과가 아니다. 핵심6연속 결과·분기 분포와 비교 노출·반올림 한계는 [보고서](../research/trend-filter-policy-20261023/report.md)에 기록했다. 각 비용 연속 완료19회(이익5/손실14), 별도분기 완료16회 모두손실·기말30/36·무완료25/36이며 합산/복리하지 않는다.

- 고정 설계 SHA-256: f2a3f801da05d77544580b3fabc265197bdc26cd7072859df9f2f4428262040f
- 감사 스크립트 SHA-256: 24f1f4b9f612b38f77b6e16c5d026776490154d357be308f5757dc92ab02d5af
- 감사 JSON SHA-256: d465feaf42af0fddd1382298aa16f8fb3a71e07cc501fa667a4a511f7e9cc3f2

직접 정밀 산술 허용은0.0051원/0.000051%p다. 공개 파생값의 피연산자 반올림은 비용 환급0.0151원·노출차이0.000151%p로 별도 검사하며 정밀 원장 허용치를 늘리지 않았다. 기록 inputHash전체재생성·미저장 대체분할 원장·benchmark guard결정·당시v12바이너리/의존성동일성은 미검증이다.

최종 문서 검증: npm run todo:render와npm run check:todo를 직접 실행해 열린13개·동기화를 확인했다. npm run check:project-map와git diff --check를 통과했고, 상대 링크90개·문서/감사 언어 검사·TASK_TEMPLATE10절·P2-11상위미완료·고정설계hash·감사sourceFiles8개/원장압축2개hash를 확인했다. 이 단계에서는 생산코드/README/기존연구를 수정하지 않았으며 커밋·푸시는 부모가 최종 검증 뒤 진행한다.

감사 실행 주체: trend_helpers가 두 번 재실행의 JSON bytes 동일성과 메모리 변조4건 탐지를 수행했다. 부모는 최종 JSON/검사수·출처8개 SHA·설계 SHA·39체결경로/부호반전을 직접 대조했다. 별도 심볼 역할은 정적 스크립트 검토와8개 SHA·집계/최종 표를 대조했으며 재실행/변조 시험을 직접 수행한 것은 아니다.

부모의 원기록 차단 사유 대조: 기존 v12 추세 필터78개에서 연속 SOXQ 비용별 비중 초과8건, 별도 분기 SOXQ 비용별 비중 초과137건·기대 순익0이하1건을 집계했다. SPY/QQQ는0건이며 guard 재실행이나 독립 감사42,958개에 추가된 검사가 아니다. 보고서에 원장 경로와 해석 한계를 보완했으며 guard 완화 근거로 사용하지 않는다.
