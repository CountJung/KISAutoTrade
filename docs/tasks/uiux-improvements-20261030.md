# UI/UX 즉시 개선 9개 — 2026-10-30

## 1. 메타데이터

- 기준: `db87a1120fef3ea4153d27ef371c4f50b65bff99`, `fix/mac-security-storage-2026-10-06`, 시작 clean.
- 담당: 구현 담당·루트 검증/게시, `ui_ux_expert` 독립 검토, 프로젝트 지도 담당.
- 상태: In Progress. 날짜: 2026-10-30.
- 정본: [TODO](../todo.md), [완료 이력](../completed-history.md), [역할 사용](../agent-tooling.md#uiux-전문가-사용).
- 위험: 보통 — 금융 동작 정책을 유지하는 UI 상태·접근성 개선.

## 2. 목표와 비목표

TODO에 고정한 아래 순서대로 항목별 구현·격리 검증·독립 리뷰·이력 이관·일반 커밋/푸시를 완료한 뒤 다음 항목으로 간다. 금융 의미·query key/polling·계좌 경계·기존 실패 보호는 유지한다.
P2-UX-07의 확인 정책 및 기존 SQLx/사용자 판단/실자료 항목 11개는 범위 밖이다. 실거래·실계좌 조회·민감 파일·보안 설정·배포·강제 푸시는 실행하지 않는다.

## 3. 시작 상태와 범위 고정

| 순서 | 고정 ID | 작업 | 상태 | 게시 근거 |
| --- | --- | --- | --- | --- |
| 1 | P1-UX-01 | 실행 상태 미확인·실패·이전 자료 구분 | 구현·검증·리뷰 완료 | `d18dce3f9cbe858a29dd8c7e5cc2c9f8383e24a1` 원격 일치 |
| 2 | P1-UX-02 | KIS 잔고 로딩·실패·정상 빈 목록 구분 | 구현·검증·리뷰 완료 | `36ccd96618d1681cf9c2d5e68bda6815d596e60d` 원격 일치 |
| 3 | P1-UX-03 | History 통계 실패와 실제 0 구분 | 구현·검증·리뷰 완료 | `aaf94f274f4a5eb04f34a2d09df0665dc293a222` 원격 일치 |
| 4 | P2-UX-01 | 검색 결과·보유 종목의 키보드 선택 | 구현·검증·리뷰 완료 | `ee341d6ca6d6f3463d77fec2469e3ee986063e96` 원격 일치 |
| 5 | P2-UX-02 | 패널 크기 조절 키보드 대안 | 구현·검증·리뷰 완료 | 이번 회차 커밋; 푸시 확인 대기 |
| 6 | P2-UX-03 | 좁은 History 거래 상세 | 다음 작업 | — |
| 7 | P2-UX-04 | Log 로딩·실패·빈/필터0 구분 | 미착수 | — |
| 8 | P2-UX-05 | Settings 목차·heading·focus 이동 | 미착수 | — |
| 9 | P2-UX-06 | 차트 텍스트/키보드 자료 대안 | 미착수 | — |

원래 완료 조건과 제외 항목 본문은 착수 시 보존했고 완료 항목별로 이력에 이관한다. 사용자 기존 변경은 없었으며 이후 발견 시 이번 작업과 분리한다.

## 4. 조사 근거

현재 UI/UX 검토의 코드·화면 근거는 [UIUX-20261030](../completed-history.md#uiux-20261030)에 있다. 구현은 해당 소스·기존 스킬과 query/cache 계약을 먼저 확인한다. 표시 오류와 backend guard 위반을 혼동하지 않는다. provider 정책을 새로 구현하지 않아 외부 금융 API 조사는 N/A다.

## 5. 영향과 계약

React 페이지·공통 표시·접근성·mock UI 테스트가 대상이다. IPC/Rust/저장 schema/계정·동의·리스크 정책 변경은 N/A이며 변경0을 대조한다. 파일 추가·공개 API 이동은 지도/참조 검토를 수행한다. 같은 기능의 중복 helper는 의미를 확인해 가장 좁은 경계에서만 공유한다.

## 6. 금융 안전

실제·모의 주문이나 자동매매 시작/정지는 실행하지 않는다. 새 UI/UX 읽기 테스트는 모든 외부 페이지 요청·WebSocket과 금융 쓰기를 거부한 합성 fixture를 사용한다. 기존 주문/시작/설정 변이 회귀의 명시적 mock 응답은 브라우저 안에서만 끝나며 실제 backend를 사용하지 않는다. 읽기 POST 전송이 있으면 테스트 안의 정확한 합성 경로만 허용하고 금융 변이와 구분한다. 운영 앱·DB·OS 키체인을 열지 않는다. 캐시·query·guard의 동작 의미와 긴급 정지 즉시성은 보존한다.

## 7. 구현·롤백

각 회차의 완료 기준으로 검증하고 회차별 커밋을 남긴다. 롤백 단위는 해당 회차 커밋이며 자동 revert/배포는 실행하지 않는다. 선행 코드 변경으로 순서가 바뀌면 근거를 기록하며 현재 순서 변경은 없다.

## 8. 도구와 역할

현재 세션은 TOML 지침을 읽은 일반 서브 에이전트로 UI/UX 역할을 적용한다. 새 세션 자동 탐색을 확인할 전용 도구/역할 선택 매개변수가 없어 미검증이다. 전역 설정이나 별도 외부 에이전트를 실행하지 않는다.
Serena 전용 도구가 없으면 소스/rg 참조를 대조한다. 공통 컴포넌트·소스 추가 후 지도/Graphify freshness를 갱신하고 확인한다. 구조가 유지되는 부분은 불필요하게 이동하지 않는다.

## 9. 검증 방식

`.cache/uiux-implementation-20261030/`의 envDir=false Vite wrapper와 최소 비민감 환경 실행기를 사용한다. dev/production 서버는 loopback에만 바인딩하고 Playwright가 종료한다. 새 테스트는 기존 `tests/e2e/`에 남겨 최종 CI에서 같은 production UI를 검증한다. 실제 관측·집중 테스트·전체 회귀·미실행을 별도 집계한다.

회차별 앱/config 타입·FSD·격리 build·관련 mock 회귀·문서/지도/그래프/diff 검사를 수행하고, 마지막에 전체 UI·테스트/보안 CI의 정확한 코드 SHA를 확인한다. Rust 코드가 바뀌지 않으면 회차마다 같은 전체 Rust 검증을 반복하지 않는다.

## 10. 회차별 결과

각 결과는 실제 검증과 독립 리뷰 이후 기록한다.


### 1회차 — P1-UX-01

범용 QueryStatusSummary를 추가해 Sidebar와 Dashboard의 미확인·첫 실패·이전 실행/정지 자료·마지막 정상 확인 시각·읽기 재시도를 일치시켰다. query 옵션과 금융 handler/버튼 조건은 원문 동등성을 유지했다.

앱/config 타입·FSD·격리 production build와 최종 새 상태 테스트5개, 기존 dialog/Select/차트5개, 기존 합성 Sidebar 시작/정지1개의 고유11개 검증을 통과했다. 초회3·중간9·후속6 실행은 중복 합산하지 않는다. 160px Sidebar와375px 다크 메뉴 캡처를 확인했고 독립 리뷰의 문구 기대 정합 P2를 수정·재검증해 잔여 필수 P0/P1/P2는 없다.

실제 시작/정지·실계좌·native IPC·스크린리더는 실행하지 않았다. 기존 Sidebar 회귀의 시작/정지는 browser mock 응답만 사용한다. 최초 runner는 webServer 작업경로 오류로 서버 시작 전 실패했고 cwd만 교정해 통과했다. Graphify의 기존 JSON5개 zero-node 안내와5000노드 HTML 기본 제한을 제품 경고와 구분하며 로컬 HTML 재생성으로 기존 지도를 유지한다.

- 변경 핵심: src/shared/ui/QueryStatusSummary.tsx·index.ts, Sidebar.tsx, dashboard/ui/Page.tsx, uxReviewFixture.ts, uiux-status.spec.ts, 기존 strategy-scrollbar.spec.ts 정지 문구 기대값
- 다음: P1-UX-02

그래프 도구 보정: 초회 update의 핵심7069노드/15535간선은 유지했다. HTML 재생성을 위한 cluster-only가 이를7068개로 재해석해 핵심 덮어쓰기를 거부한 원본도 보존했다. 강제 덮어쓰기 없이 현재 graph.json과 byte가 같은 자동 백업의 보고서를 복원하고, 현재 모든 노드/간선을 직접 HTML exporter에 전달해 사용하던 지도를 유지했다. 핵심 graph.json 전후 SHA는 동일하며 의미 분석 LLM/외부 API는 사용하지 않았다.


### 2회차 — P1-UX-02

Trading의 KIS 국내·해외 잔고에 독립 조회 상태·마지막 정상 시각·읽기 재시도를 연결하고 미조회/첫 실패를 정상 빈 목록과 분리했다. 같은 scope의 query 자료만 이전 자료로 유지하며 기존 scope fence·Toss 분기·선택/주문 본문을 보존했다.

최종 새 holdings10개+기존 주문폼5개의 고유15개가 production mock UI에서 통과했다(27.8초). 국내/해외 각각 지연·첫 실패·빈/보유 성공·성공 후 실패·키보드 복구·연속2회 재조회·다른 시장 읽기 불변을 확인했다. 앱/config 타입·FSD·diff와 보호원문9항목 대조가 통과했고375px 다크 이전 해외잔고 캡처를 확인했다. 테스트 외부 guard P2를 보강·재검증해 최종 독립 리뷰에 잔여 필수 P0/P1/P2는 없다.

초회14/1은 Toss 로딩 검사의 전역 spinner 다중 매치였고 locator를 보유 패널로 좁혀 통과했다. 원본을 보존했다. Toss open-orders POST형 읽기는 loopback·정확 경로/POST만 합성 응답하며 새 검사 금융쓰기0이다. 기존5개는 합성 주문5건(성공4/거절1)으로 실제 금융 동작0과 구분한다. 전체UI·native·실계좌·스크린리더·profile 전환 재실행은 미실행이며 scope fence는 원문 대조로 보존했다. Graphify의 기존 JSON5개 zero-node 안내는 별도 한계다.

- 변경 핵심: trading/ui/Page.tsx·kisPanels.tsx·useTradingOrderModel.ts, tests/e2e/uiux-holdings.spec.ts; HARNESS의349개 과거 기준/새 회귀 집계 정본 안내
- 다음: P1-UX-03


### 3회차 — P1-UX-03

History의 통계와 체결 목록을 독립 조회 상태·마지막 정상 시각·읽기 재시도로 표시하고, 미조회/실패와 실제0을 분리했다. 날짜 입력의 미적용 변경과 실제 적용 기간을 명시하며 같은 query의 이전 자료만 보존한다. 원래 손익/승률·통화/trace·날짜 전달과 query 계약은 유지했다.

새 History6개+기존 live-daily13개의 고유19개가 첫 실행에서 통과했다(17.2초). 초기 조회 전/지연·통계 실패/체결 성공·실제0/빈 성공·양쪽 실패·각 이전 자료/시각·독립 키보드 재시도와 반복2회·미적용 입력/확정 기간을 검증했다. 앱/config 타입·FSD·보호원문11항목·독립 UI 리뷰를 통과했고 잔여 필수 P0/P1/P2는 없다.

375px 다크 캡처에서 적용 기간·상태·시각·재시도를 확인했다. 스크롤 화면의 모든 요약값이 한 프레임에 가림 없이 보였다는 의미는 아니다. 새6개는 금융쓰기/외부/unknown/pageerror0, 기존13개는 합성 상태 조회/시계/페이징 검증이다. native·스크린리더·실제 원장/실계좌·기간/승률 정책 개편은 미실행이다. Graphify 기존 JSON5개 zero-node 추출 한계는 남는다.

- 변경 핵심: src/pages/history/ui/Page.tsx, tests/e2e/uiux-history-status.spec.ts
- 다음: P2-UX-01


### 4회차 — P2-UX-01

Trading/Strategy 검색 결과와 KIS/Toss 양시장 보유 행에 이름·코드가 있는 native 선택 버튼과 focus 표시를 추가했다. 내부 Tab 이동 중 결과를 유지하고 Escape/검색 선택 후 입력으로 focus를 돌린다. 순수 RowSelectionButton을5곳에서 재사용하고 Strategy 검색 표만52행 모듈로 분리해 Page982행을 유지했다. 기존 금융/선택 state·모델·query는 보존했다.

새 키보드/포커스10개+기존 form5개+holdings10개의 고유25개가 최종 통과했다(27.8초). 검색 내부 blur·Escape/선택 후 닫힘·외부 blur·포인터 코드셀·KIS/Toss×KR/US 선택을 확인했다. 375px 다크 긴 이름의 full accessible name·focus ring·문서 overflow0 캡처를 확인했고 앱/config 타입·FSD·보호14항목/전략 함수15개 AST 동등성을 통과했다. 최종 독립 UI/helper·참조 검토에 잔여 필수 P0/P1/P2는 없다.

초회22/1은 전략 필드 로딩 전 테스트 baseline 비교여서 실제 준비 상태 대기를 추가했다. 추가 race 재현은 Trading 통과/Strategy 재개방 실패1건이었으며 Strategy의 programmatic focus 동안만 ref guard를 추가해 해소했다. 초기 clock 준비 오류·중간23통과·실제 재현1/1·최종25통과 원본을 구분했다. 원래 자동 검색을0으로 억제하지 않았다. 실제 금융 동작0이며 기존 form은 합성 주문 회귀다. native/스크린리더·새 API/등록·토글·위험 확인 정책은 검증/변경하지 않았다. Serena MCP가 없어 rg/소스 및 제한된 로컬 Graphify 탐색을 보조로 사용했다.

- 변경 핵심: Trading Page.tsx/kisPanels.tsx, Strategy Page.tsx/StockSearchResults.tsx, shared/ui/RowSelectionButton.tsx·index.ts, uiux-keyboard-selection.spec.ts, UI 스킬 원본
- 다음: P2-UX-02


### 5회차 — P2-UX-02

Sidebar 너비와 Log 높이에 명명된 키보드 separator를 추가했다. 방향키·Home/End·Enter 기본값 복원과 focus 표시를 제공하고 명시한 target을 즉시 저장한다. 기존 pointer delta/end·저장 키/범위·반응형 flex 계약은 보존했다.

신규 resizer5개와 기존 scroll5개의 고유10개가 최종 통과했다(4.7초). 반복키·경계·기본값·저장/route/reload·다른 pref 불변·pointer와 반응형 표시를 확인했다. 동일 Log 사례의 geometry 부분 재실행1개도 통과했으며 중복 합산하지 않는다. 앱/config 타입·FSD·보호12항목 및 AppShell12/Log4 callback AST 대조를 통과했고 최종 독립 리뷰에 필수 P0/P1/P2는 없다.

초회8통과/2실패는 Chrome의 기존 scroll Paper Tabstop을 건너뛴 테스트 가정이었다. 실제 Tab 경로로 교정한 원본을 보존했다. 1440×900에서 End 설정/ARIA/저장900에 실제 Log 높이는 기존 flex 제약으로751.984px이며 손잡이는 viewport 안에 남았다. 설정값이 항상 실제 렌더 픽셀과 일치한다고 주장하지 않는다. 새 검사 금융 쓰기/외부/unknown/pageerror0, 서버 정리 확인. native·스크린리더·touch·실계좌는 미검증이며 최종 전체UI/CI는 별도다.

- 변경 핵심: shared/ui/LayoutResizer.tsx, app-shell/ui/AppShell.tsx, log/ui/Page.tsx, uiux-resizer.spec.ts, UI 스킬 원본
- 다음: P2-UX-03
