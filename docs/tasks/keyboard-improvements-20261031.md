# 키보드 편의성 5개 순차 구현 — 2026-10-31

## 1. 메타데이터와 승인 범위

- 기준: `8a9d1b54312721cdc38bf84abb8dbc801a2ad08b`, `fix/mac-security-storage-2026-10-06`, 시작 clean.
- 사용자 승인: 2026-10-31 08:19 UTC, 방금 등록한 할 일을 순서대로 구현. 이전 키보드 검토 전용 제한은 아래5개에 한해 변경됐다.
- 상태: In Progress. 루트는 검증/문서/게시, 구현 담당은 해당 회차 source/test, UI 전문가·참조/지도 담당은 독립 읽기 검토를 맡는다.
- 정본: [현재 TODO](../todo.md), [검토 근거](../completed-history.md#kbd-audit-20261031), [이전 UI9개](uiux-improvements-20261030.md), [역할 사용](../agent-tooling.md#uiux-전문가-사용).
- 제외: 기존 선행 결정/사용자 판단/실자료11개, P2-UX-07 확인 정책, 토스 실제 응답 원인 확정/실계좌·운영앱 확인. 금융 동작 정책·기본값·저장/조회 계약을 임의 변경하지 않는다.

## 2. 고정 순서와 회차 영수증

| 순서 | ID | 범위 | 상태 | 코드·원격 근거 |
|---|---|---|---|---|
|1|P2-KBD-01|조회 Enter·IME·버튼 이름|로컬 완료·일반 푸시 준비|검증 완료·원격 영수증 후속 기록|
|2|P2-KBD-02|중복 조회·늦은 응답·scope|대기|미착수|
|3|P2-KBD-03|LTH/결과 목록 키보드|대기|미착수|
|4|P2-KBD-04|수치 이름·단위·Enter commit|대기|미착수|
|5|P2-KBD-05|프로필 팝업 pending·복구|대기|미착수|

각 항목의 원래 근거·기대 동작·완료 조건은 착수 TODO에서 고정했다. 완료할 때 원래 ID/조건을 이력에 보존하고 열린 목록에서 걷어 낸다. 회차별 red→최소 수정→green/인접 회귀→타입/빌드·독립 리뷰→문서/지도/Graphify→일반 커밋·푸시·원격 SHA 확인 뒤 다음 항목으로 진행한다. 순서를 바꾸면 의존 근거를 남기며 현재 변경은 없다.

## 3. 구현 경계와 수용 기준

- P2-KBD-01: Trading/Strategy/LTH×KR/US×KIS/Toss의 현재 validity/조회 callback은 caller가 소유한다. 일반 Enter·검색 버튼·조합 중 isComposing/native229·종료 후 Enter, 빈칸/유효한1글자US ticker·6자리직접선택을 대조한다. TextField의 adornment 버튼 native Enter가 상위 handler와 중복 조회되지 않아야 한다.
- P2-KBD-02: 동기 단일 진행 guard·입력/시장/broker/profile/요청 세대/unmount의 이전 결과 무효화를 검증한다. 원래 거래소 fallback 단계와 Toss 전용 경로·캐시/조회 주기는 유지하고 pending 중 명명 컨트롤·focus를 보존한다.
- P2-KBD-03: LTH의 별도 국내 결과와 상단 결과의 Tab/Shift+Tab·Arrow/Home/End·Enter/Space·Escape/닫기/복귀·내부/외부 blur·pointer·stratEnabled를 대조한다. 선택이 자동 대상추가/전략저장으로 이어지면 안 된다.
- P2-KBD-04: 보이는 label/unit를 해당 입력/Slider에 연결하고 조합 Enter는 commit하지 않는다. 기존 일반 Enter/Tab blur·범위/step·요청·disabled/오류 의미는 보존한다.
- P2-KBD-05: 현재 pending 정책과 Cancel/Escape/backdrop을 일치시키고 이전 Dialog callback·재진입·중복 활성화·실패/재시도·focus 복귀를 검증한다. 원래 삭제 확인·활성프로필 경고·계좌 mutation 직렬화는 유지한다.

## 4. 안전한 실행과 증거 등급

fresh Chromium context의 합성 mock/offline만 사용한다. envDir:false와 비민감 환경 allowlist로 프로젝트 .env/계좌/키/운영 config를 로드하지 않는다. 모든 외부/WS·unknown·금융/설정 변이는 기본 거부하고, 기존 의미상 읽기 POST가 필요하면 정확 loopback/path/method의 합성 응답으로만 분리한다. 프로필 저장/삭제와 기존 주문폼 회귀의 명시적 제출은 explicit browser mock이며 실제 운영 변경·실제 주문0으로 구분한다. 신규 키보드 읽기 검사의 쓰기0과 기존 합성 주문 제출 검사의 mock 쓰기 경계를 합치지 않는다.

실거래·실계좌 API·운영 DB/앱·자격증명/키체인·보안 설정·운영배포·강제푸시·사용자 앱 강제종료는 수행하지 않는다. 실제 OS 한글 IME·WebView·스크린리더는 합성 isTrusted=false 이벤트/DOM/focus 검사와 구분한다. 이전7개 관측 및 408UI/901Rust CI를 이번 구현의 새 통과로 주장하지 않는다.

## 5. 조사·참조와 설계

기존 AGENTS·원본 UI/React/FSD 스킬과 docs-canon을 적용한다. Serena/Graphify 전용 MCP가 없으면 rg/소스/AST로 대조하고 Graphify는 기존 로컬 code-only 갱신·freshness 용도로만 사용한다. 전역 설정/모델/role config를 변경하지 않는다.

사전 KBD01 분석은 `/tmp/kis-kbd01-impact-20261031.md`다. 공유 후보는 순수 Enter/IME·229 판정뿐이며 callback/validity/금융 의미는 공유하지 않는다. Strategy Page는 시작982행으로1000행 규칙을 지킨다. 파일/공개 helper 추가는 참조·지도 pass를 거친다.

## 6. 검증·롤백

회차별 meaningful red를 먼저 실제 실행하고 원본 실패·fixture 준비 오류·green 결과를 분리한다. 기능이 정상이라는 positive control도 유지한다. 각 회차는 영향 범위의 타입/production build·focused와 인접 UI 회귀, 필요 Rust check·FSD·문서/지도/Graphify·독립 리뷰를 통과한다. 마지막 코드에서 전체 회귀·필수 검사·테스트/보안 CI를 확인한다. 문서 후속 커밋과 CI 코드 SHA를 구분한다.

롤백 단위는 회차 커밋이다. 자동 revert·운영 설치/교체·배포는 실행하지 않는다. 캐시·민감 파일·사용자 기존 변경은 커밋하지 않는다. 승인 거절은 미실행/불확실 결과부터 확인하고 새 관련 사용자 근거가 있을 때만 해당 동작을한번 재검토하며 우회하지 않는다.

## 7. 회차 기록

### P2-KBD-01

- **구현 범위:** `searchKeyboard.ts::isSearchEnter`는 직접/native 조합 상태와229만 판정한다. Trading/Strategy/LTH의 실제 입력이 지역 유효성·기존 읽기 callback을 호출하며, 버튼·결과 행의 native Enter를 중복 해석하지 않는다. KR6자리 직접/자동 경로, US1자 ticker, KIS 거래소 순서와 Toss/LTH fallback은 유지했다.
- **표시 분리:** 미국 검색 표시만 `StrategyUsSearch.tsx`로 추출했다. 조회·선택·이름 캐시는 Page에 남고 본문은966행이다. 버튼 이름과 조회 중 설명을 보완했다.
- **최초 red:** KIS5개(session21114)와 추가 KR2개(session62985)가 제품 계약에서 실패했다. Trading KR Enter GET0, 세 US 조합 Enter 논리GET1, 미국 검색 이름 누락 및 Strategy/LTH KR Enter 누락을 분리했다. adornment의 정상 논리GET1은 중복 조회의 증거가 아니다. 자동250/400ms와 명시 Enter는 정지된 clock으로 구분했다.
- **독립 리뷰의 추가 재현:** 같은 이름 조회→Escape→값 변경 없는 Enter 재개방은 KIS1개(session32453)에서 결과 소실로 실패했다. 기존 이름 분기는 명시한 현재 query만 유지하고 다른 입력·닫기·6자리 경로는 보존하도록 최소 교정했다. KIS/Toss 모두1ms와600ms 뒤 결과·입력 focus 유지, 캐시 GET1→1을 검증했다. 도움말도 이름2자 조회와 기존6자리 자동 경로를 구분했다.
- **검사 준비 교정:** 첫 타입 오류5곳은 명시적 React 이벤트 타입으로 해결했다. 첫29개 실행의27통과/2실패는 LTH의 기존 동명 버튼2개에 걸린 selector 준비 오류다. 실제 수량 행·선택 chip·유일한 대상 버튼의 enabled 조건을 유지해 범위만 좁혔다. 무시 주석·any·기대 완화는 없다.
- **최종 검증:** Rust check --locked10.72초→앱/config 타입→FSD 통과, 경고0. 격리 production build 후 최종29/29(신규14+인접15,18.2초,session52851) 통과. 앞선17.2초 green과 부분/실패 실행을 합산하지 않는다. 신규14개 JSON 모두 쓰기/외부/unknown/pageerror0이다.
- **리뷰·지도:** 독립 UI 및 참조/지도 리뷰에 필수 P0/P1/P2 지적이 없다. 공유 경계는4파일·5handler이며 모델 callback·금융 조립은 유지했다. code-only Graphify7267nodes/16095edges와 최종 해시를 갱신했다.
- **실제 관측/한계:** fresh Chromium1280×720 light의 Trading/LTH 미국 선택·focus 캡처를 확인했다. 합성 isTrusted=false 조합 이벤트는 실제 macOS IME·Tauri WebView·AT 검증이 아니다. 실제 계좌 조회·운영 데이터 변형·실제 금융 동작·배포는 실행하지 않았다. P2-KBD-02~05와 기존 대기11개는 이 완료에 포함되지 않는다.
- **원본·게시:** `.cache/kbd01-implementation-20261031/`의 red-initial-kis5, red-kr-kis.log, first-green-29, green-before-reopen, red-reopen-kis1 및 green-final-reopen-29.log에 단계별 원본을 보존했다. 이번 회차는 검증 완료·커밋/일반 푸시 준비 상태이며 원격 SHA는 후속 영수증에 기록한다.

- **문서 마감:** docs-canon120개 문서의 링크/표/앵커/등록부 오류0, TODO15개·지도/그래프 freshness·diff 검사 통과. 문서 독립 리뷰의 범위 표현을 교정해 신규 읽기14개는 쓰기0, 인접 주문폼5개는 명시적 mock 제출·실제 주문0으로 구분했다. 최종 문서 리뷰의 필수 P0/P1/P2 잔여는 없다.
