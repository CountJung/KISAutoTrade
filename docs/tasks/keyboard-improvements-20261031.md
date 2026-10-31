# 키보드 편의성 5개 순차 구현 — 2026-10-31

## 1. 메타데이터와 승인 범위

- 기준: `8a9d1b54312721cdc38bf84abb8dbc801a2ad08b`, `fix/mac-security-storage-2026-10-06`, 시작 clean.
- 사용자 승인: 2026-10-31 08:19 UTC, 방금 등록한 할 일을 순서대로 구현. 이전 키보드 검토 전용 제한은 아래5개에 한해 변경됐다.
- 상태: In Progress. 루트는 검증/문서/게시를 맡고 회차별 source/test 생산과 독립 검토 역할을 분리한다. 역할이 바뀌면 해당 회차에 기록하며, 생산자가 자기 변경의 독립 리뷰를 맡지 않는다.
- 정본: [현재 TODO](../todo.md), [검토 근거](../completed-history.md#kbd-audit-20261031), [이전 UI9개](uiux-improvements-20261030.md), [역할 사용](../agent-tooling.md#uiux-전문가-사용).
- 제외: 기존 선행 결정/사용자 판단/실자료11개, P2-UX-07 확인 정책, 토스 실제 응답 원인 확정/실계좌·운영앱 확인. 금융 동작 정책·기본값·저장/조회 계약을 임의 변경하지 않는다.

## 2. 고정 순서와 회차 영수증

| 순서 | ID | 범위 | 상태 | 코드·원격 근거 |
|---|---|---|---|---|
|1|P2-KBD-01|조회 Enter·IME·버튼 이름|완료·원격 확인|[`a8cba69`](https://github.com/CountJung/KISAutoTrade/commit/a8cba69e1ac518d36ba5f5e173a4eff921098a6e)|
|2|P2-KBD-02|중복 조회·늦은 응답·scope|완료·원격 확인|[`5b45c46`](https://github.com/CountJung/KISAutoTrade/commit/5b45c463807cbd1385d7837b4daf3245e1659874)|
|3|P2-KBD-03|LTH/결과 목록 키보드|로컬 완료·일반 푸시 준비|검증 완료·원격 영수증 후속 기록|
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
- **원본·게시:** `.cache/kbd01-implementation-20261031/`의 red-initial-kis5, red-kr-kis.log, first-green-29, green-before-reopen, red-reopen-kis1 및 green-final-reopen-29.log에 단계별 원본을 보존했다. 이번 회차는 `a8cba69e1ac518d36ba5f5e173a4eff921098a6e`로 일반 푸시하고 원격 SHA 일치와 clean을 확인했다. 통합 테스트/보안 CI는 마지막 코드에서 별도 확인한다.

- **문서 마감:** docs-canon120개 문서의 링크/표/앵커/등록부 오류0, TODO15개·지도/그래프 freshness·diff 검사 통과. 문서 독립 리뷰의 범위 표현을 교정해 신규 읽기14개는 쓰기0, 인접 주문폼5개는 명시적 mock 제출·실제 주문0으로 구분했다. 최종 문서 리뷰의 필수 P0/P1/P2 잔여는 없다.

### P2-KBD-02

- **기준·구현:** 첫 회차 원격확인 `a8cba69`에서 시작했다. 신규 `keyedRequestGate.ts`는 순수 ticket/epoch controller와 React 수명/revision adapter다. Trading 모델·Strategy US local hook·LTH 모델3곳이 같은 동기 begin/current/finish를 소비하며, 입력/시장/선택/public scope·LTH 잠금의 key 조립과 조회/유효성·결과·오류·broker fallback은 caller에 남겼다.
- **보존·표시:** 현재 요청을 finish한 뒤 표시명·선택을 바꾸고, 이전 성공/오류/후속 거래소/fallback/finally가 새 요청을 덮지 않게 했다. Trading의 동일 IconButton과 Strategy 버튼은 유효 입력 pending에 aria-disabled/busy·설명을 제공하고 handler gate로 중복을 억제한다. 빈 입력·실제 잠금의 native disabled, 원래 query/cache/주기·금융 setter와 Toss Trading 즉시선택은 유지했다.
- **소유 분리:** Strategy US 조회/오류/busy·KIS/Toss 탐색만 `useStrategyUsSearch.ts`로 옮겼다. Page는 입력/선택·이름 cache/onSelect·전략 조립과 저장을 유지하며928행이다. 지도도 새 local hook의 실제 소유를 반영했다.
- **red5:** production4개(session54571 exit1)는 세 화면의 Space+Enter3+repeat가 첫 응답 전 논리GET5(기대1)를 만든 문제와 Trading oldA가 B 조회 중 A를 선택하는 문제를 재현했다. source1개(session10607 exit1)는 동일 rawA에서 account만 바꿔 oldA/newA ticket을 나눴고, 늦은 oldA 선택을 재현했다. 첫5개 모두 쓰기/외부/unknown/pageerror0이었다.
- **검사 보강·준비 교정:** Node route.fulfill만으로 완료를 판단하지 않고 정확 ticket header의 response.finished 이후 UI 알림을 대조했다. 첫68개는66통과/2준비실패(38.7초)였다. Strategy profile/broker 변경의 기존 입력 reset을 무시한 가정이 원인이며, 기준 커밋의 effect와 대조해 정상 reset을 검사하고 A를 명시 재입력하도록 교정했다. Trading same-raw profile/broker 단독2개를 추가해 독립 필드 판별도 보존했다. 후속70/70(44.4초)은 별도 원본으로 보존했다.
- **최종 경계:** LTH pending 중 합성 strategies GET의 enabled만 바꾼 잠금1개를 추가했다. 동일 main·input DOM/scope에서 old 성공/무효 본문 뒤 선택·경고·후속0과 unlock 후 새B 정상 복구를 확인했다. 실제 전략 toggle/save/start는 호출하지 않았다. 입력만 변경/비우기·raw/scope ABA·독립 broker/profile/account/paper·시장/SPA 이탈·stale 오류/invalid·명시 재시도·KIS/Toss/LTH의 원래 fallback도 대조했다.
- **최종 결과:** 신규42개(production34/source8)+인접29개의 **71/71(32.3초,session24996)**가 통과했다. 과거/부분 실행을 합산하지 않는다. 현재42개 제목과 JSON을 직접 대조한 `final-42-evidence-index.json`의 쓰기/외부/unknown/pageerror는0이다. 인접 주문폼5개의 명시적 mock 제출은 실제 주문과 분리한다.
- **필수 검사·리뷰:** Rust check --locked9.23초→앱/config 타입→FSD, 새 검사 noEmit 타입과 격리 production build를 통과했다. 컴파일 경고0이며 앱 타입 설정 완화는 없다. 독립 코드·검사·참조/지도 리뷰의 필수 P0/P1/P2 잔여는 없다. Graphify7304nodes/16188edges와 해시를 갱신했다. 기존6개 비코드 JSON의0-node 안내는 extractor 한계로 분리한다.
- **한계:** 이미 전송된 cmd Promise는 완료될 수 있고, 보호는 UI 적용·후속 탐색을 제한한다. source harness는 실제 router/Page/QueryClient를 유지하는 합성 공개 설정 재조회이며 실제 계좌/credential pinning이나 backend abort가 아니다. dev StrictMode 초기 수명과 정상조회·SPA 이탈은 관측했지만 pending-ticket dispose→activate나 중단된 concurrent render는 별도 재현하지 않았다. 실제 OS IME/WebView/AT·운영 계좌/자료·배포는 미실행이다.
- **원본·게시:** `.cache/kbd02-implementation-20261031/`의 red-initial-kis4, pre-test-expansion-20261031, first-green-68, green-before-lth-lock 및 green-final-71.log에 단계별 원본을 보존했다. 제품/source와 검사 생산을 분리하고 별도 담당이 독립 리뷰했다. 이번 회차는 `5b45c463807cbd1385d7837b4daf3245e1659874`로 일반 푸시하고 원격 SHA 일치와 clean을 확인했다. 통합 테스트/보안 CI는 마지막 코드에서 별도 확인한다.

- **문서 마감:** docs-canon120개 문서의 링크/표/앵커/등록부 오류0, TODO14개·지도/그래프 freshness·원래 열린14개/제외11개 보존·diff 검사 통과. 최종 독립 문서 리뷰에 필수 P0/P1/P2 잔여는 없다. 첫 회차의 원격 영수증은 기록했고 이번 회차의 게시 결과는 다음 영수증으로 이어 남긴다.

### P2-KBD-03

- **기준·구현:** 두 번째 회차 원격확인5b45c46에서 시작했다. 신규13행 `resultButtonNavigation.ts`는 opt-in 결과 TableBody3곳의 현재 enabled native 버튼 DOM만 찾아4키 focus를 clamp한다. 선택/query/index를 저장하지 않으며 modifier·event 취소는 caller에 둔다. RowSelectionButton optional disabled 기본false와 LTH KR 연결을 추가하고 기존 보유3곳은 그대로다.
- **LTH 표시 수명:** 명명된 버튼·KR region/input refs와 동기 반환 guard를 연결했다. 내부 Tab/Shift+Tab은 닫지 않고 외부180ms 닫기·Escape/선택 후 입력 복귀를 처리한다. 기존180ms의 owner가 입력에서 영역으로 바뀌었으며 모든 timer 무변경은 아니다. Trading400ms/LTH250ms 조회 debounce·모델·US fallback·02 gate·대상추가/저장 정책은 유지했다.
- **red와 제품 후속:** 최소KIS4개(session49819 exit1)는 기존 코드cell 포인터 positive4개 이후 LTH native 버튼 부재/Escape 미닫힘과 Trading/Strategy ArrowDown 미이동에서 실패했다. 첫109개는102통과/7실패였다. 그중 Trading 포인터 선택 뒤600ms 재개방은 실제 결함2case였고 동기 returningSearchFocus guard로 최소 교정했다. 이후 두 broker의 해당 경로가 모두 통과했다.
- **검사 조건 교정:** 나머지 실패에는 플랫폼 기본 Home/End에 숫자 caret0을 고정하거나 다른 문서/애니메이션 frame을 동일시한 비교가 있었다. 진단1개·동일부모 clone 대조와 설치 Playwright1.64.1의 Mac native command 구현을 확인했다. 첫 수정105/109, 다음103/109는 별도 원본이며 합산하지 않는다. 최종은 같은 부모·타입·서체·geometry의 React handler 없는 기본 입력을 대조한다. 앱 caret은 강제 설정하지 않았고 제품 기본 키 동작을 변경하지 않았다.
- **최종 계약·한계:** 결과 버튼4키의 focus/clamp와 입력4방향키의 caret/direction 비교는 strict다. 입력 Home/End는 같은 시작 selection·실제 key/trusted/defaultPrevented·값/focus 보존을 검사하고 후속 numeric caret·진행 중 scroll 좌표는 OS/브라우저 관측 한계로 분리한다. 이 한계를 통과한 실제 macOS IME/WebView/스크린리더 검증으로 확대하지 않는다. 이전 about:blank 진단 PNG는 최종 앱 PNG와 구분한다.
- **최종 결과:** 격리 production build 뒤 **109/109(1.3m)** 통과. 신규38개(production34/source4)+인접71개이며 반복 실행을 합산하지 않는다. 현재38개 제목의 JSON은 모두 passed이고 쓰기/외부/unknown/pageerror0이다. 인접 주문폼5개의 명시적 browser mock 제출은 실제 주문0으로 분리한다. 내부 callback spy 대신 native click1과 최종 선택/닫힘·값 보존을 대조했다.
- **범위 검증:** KIS/Toss×Trading/Strategy/LTH의 Enter/Space·Tab/ShiftTab·4키 경계·입력 기본 동작·Escape·내부/외부blur·코드cell 포인터·RACE01 복귀600ms, loading/empty/error/복구를 검사했다. 40행 내부 scroll/bounds, 좁은375×812 dark와1280×900 light, 현재 query 결과 교체와 합성 GET enabled 잠금/해제를 포함한다. modifier 조합 직접 행사는 미실행이며 caller의 early-return 소스로만 대조했다. 실제 거래·설정 변이·전략toggle/save/start는 없다.
- **필수 검사·리뷰:** Trading 후속 교정 후 Rust check --locked8.11초→앱/config 타입→FSD, 검사 strict noEmit 및 production build 통과. 컴파일 경고0이다. 독립 코드/검사·참조/지도 리뷰에 필수 P0/P1/P2 잔여가 없다. code-only Graphify7340nodes/16291edges와 해시를 갱신했고 기존6개 비코드 JSON0-node 안내는 extractor 한계로 분리한다.
- **원본·게시:** `.cache/kbd03-implementation-20261031/`의 red-initial-kis4, first-green-109, first-native-109, native-context-probe-34189, native-frame-109-13078 및 green-acceptance-109.log에 단계별 원본이 있다. 최종 same-context JSON과 focus/긴 목록 앱 PNG를 보존했다. 일반 커밋·푸시 및 원격 SHA 영수증은 게시 후 기록하며 최종 코드 CI는5회차 뒤 확인한다. 기존 대기11개와04/05는 이번 완료에 포함하지 않는다.
