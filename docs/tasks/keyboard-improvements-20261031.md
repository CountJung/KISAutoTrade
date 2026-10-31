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
|3|P2-KBD-03|LTH/결과 목록 키보드|완료·원격 확인|[`a6c51fc`](https://github.com/CountJung/KISAutoTrade/commit/a6c51fc9dae70873f9c77d069e841ff42c069da1)|
|4|P2-KBD-04|수치 이름·단위·Enter commit|완료·원격 확인|[`73f1775`](https://github.com/CountJung/KISAutoTrade/commit/73f1775a8adc9d354e04fa6bde58818d00829c44)|
|5|P2-KBD-05|프로필 팝업 pending·복구|구현 게시·CI 보정 중|[`e27d7bd`](https://github.com/CountJung/KISAutoTrade/commit/e27d7bd51ac08e03c26a0cd408f3973af66573bb)|

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
- **원본·게시:** `.cache/kbd03-implementation-20261031/`의 red-initial-kis4, first-green-109, first-native-109, native-context-probe-34189, native-frame-109-13078 및 green-acceptance-109.log에 단계별 원본이 있다. 최종 same-context JSON과 focus/긴 목록 앱 PNG를 보존했다. 이번 회차는 `a6c51fc9dae70873f9c77d069e841ff42c069da1`로 일반 푸시하고 원격 SHA 일치와 clean을 확인했다. 최종 코드 CI는5회차 뒤 확인한다. 기존 대기11개와04/05는 이번 완료에 포함하지 않는다.

### P2-KBD-04

- **기준·최소 변경:**03 원격확인 a6c51fc에서 시작했다. Settings Page.tsx의 import·local SliderWithInput만16행 늘려937행이다. useId 파생 정적 label/설명/slider/input ID를 연결하고 현재 strong 값은 이름에서 제외했다. 단위·min/max/step은 실제 input의 description, Slider 수치는 기존 valueText로 전달한다. 별도 공용 helper/API는 추가하지 않았다.
- **입력·보존:** 실제 htmlInput의 typed isSearchEnter 판정 뒤 일반 Enter는 currentTarget.blur만 한 번 호출한다. 조합/native229는 blur하지 않는다. 원래 문자 정리·parseFloat·prevValue sync·round/clamp·NaN 복원·같은 유효값 blur commit과 native MUI Slider·11caller·범위/step/기본값/disabled·payload는 그대로다. 즉시5쌍과 Risk local dirty→명시 저장6쌍·비중/100·bps·dailyBuy0·volatility 의미를 유지했다.
- **red3:** 최소3개(session44293 exit1)는 이름 검사 전에11쌍22개를 전부 관측해 label 연결0을 확인했다. IME 두 case는 기존 일반 Enter의 blur1/mock 로그저장1 positive 뒤 native isComposing=true/keyCode13와 standalone229에서 각각 추가blur1/mock저장1·focus 이탈을 재현했다. unknown/external/API/pageerror/WS0이며 실제 쓰기는0이다.
- **최소 green·통합:** 수정 후 원래 최소3개의 고정 복사본은3/3(3.2초,session41762) 통과했다. canonical 확장 검사는 **151/151(1.5분,session35687)** 통과했고 신규31+앞선109+Settings목차6+UI의존성5다. 고유 관측/컨트롤22개·반복 실행을 검사 건수로 더하지 않는다.
- **증거 교정:** 최초151 통과 후 한글 제목의 ASCII 치환 파일명이 충돌해 JSON27개만 남은 기록 P2를 발견했다. 원본 first-integration-151/와 legacy-name-evidence를 보존하고 fixture의 import/capture만 수정해 제목 UTF-8 SHA25612자·원제목 JSON을 추가했다. 제품/검사 기대는 바꾸지 않았다. strict 타입 뒤 신규 **31/31(20.2초,session63819)**만 다시 실행했고 title고유31·passed31과 PNG를 확보했다. 이전109를 반복하거나151+31을 합산하지 않는다.
- **실제 범위:**11쌍22컨트롤의 정확 role/name·이름/ID 안정·단위/범위/step, 모든 쌍의 합성 composing/229와 일반 Enter를 확인했다. 네 저장 family 대표의 native Slider4방향키/Home/End·Tab/ShiftTab·기존 parser round/clamp/빈값 복원, 대표 pointer commit과 pending/실패/명시재시도를 대조했다. Risk 숨김/volatility2쌍잠금·held read/기존 retry/poll·dirty 초안 보존 및 명시 변환 저장도 검사했다. 모든11쌍에서 모든 Slider키를 각각 행사한 Cartesian 검증은 아니다.
- **mock/실제 경계:** 고유31개 JSON의 explicit mock 설정쓰기56회=성공52/의도실패4(Risk11·refresh13·archive14·log18)다. 의도 read실패3도 별도이며 HTTP 실패가 아닌 browser-only 합성 invoke 거절이다. unknown/external/API/WS/pageerror/console0, 실제 IPC/설정/파일/DB/계좌/주문0이다. 인접 주문폼5개의 명시 mock 주문은 따로 해석한다. refresh 미지원 REST를 제품에 추가하지 않았다.
- **검사·리뷰·구조:** Rust check --locked8.55초→앱/config 타입·FSD, 신규 검사 strict noEmit과 격리 production build가 통과했다. compiler 경고0, 독립 코드/검사·지도/문서 리뷰의 필수 P0/P1/P2 잔여가 없다. Graphify7368nodes/16356edges·최종 해시를 갱신했고 기존6개 비코드JSON0-node 안내는 extractor 한계로 분리했다.
- **시각·한계:**1280×900 light/375×812 dark에서 값·단위·native focus·폭을 대조하고 narrow Risk Slider 포커스 PNG를 직접 확인했다. querycache 재방문은 OS/file 설정 영속성 검증이 아니다. 합성 isTrusted=false IME는 실제 macOS 한글 IME·Tauri WebView·스크린리더 검증이 아니며 운영 파일 정리·위험엔진·계좌/API 검증은 수행하지 않았다.
- **원본·게시:** `.cache/kbd04-implementation-20261031/`의 red-initial-min3, green-minimum-3, first-integration-151, final-artifact-31와 final-31-evidence-index.json에 원본/집계를 보존했다. 이번 회차는 `73f1775a8adc9d354e04fa6bde58818d00829c44`로 일반 푸시하고 원격 SHA 일치와 clean을 확인했다. 최종 코드 CI는05 뒤 확인한다. 다음05 및 기존 대기11개는 이번 완료와 구분한다.

### P2-KBD-05

- **기준·source:**04 원격확인73f1775에서 시작했다. profileDialogs826행/accountProfiles581행과 새 useProfileDialogSession35행만 구현했다. local hook은 open/identity 참조의 epoch와 기존 useKeyedRequestGate React adapter의 독립 main/detect/lookup3개를 조립한다. closed/옛epoch begin·동기 close/aux 무효화만 소유하고 secret/form/payload를 key에 직렬화하지 않는다. Add/Edit/Delete3owner가 각각 소비하며 shared gate/API/accountHooks는 무수정이다.
- **main/aux 보존:** 기존 mutateAsync를 await해 hook-level publication/reconcile까지 main guard를 유지한다. 사용자 Cancel/Escape/backdrop과 현재 성공의 내부 닫기를 분리했고 native Cancel pending disabled를 유지했다. 유효 pending submit은 같은 Button/name/텍스트·spinner/aria-busy/aria-disabled와 동기 gate를 사용한다. 원래 submit은 isPending 외 invalid disabled가 없고 유효성 handler였으며 해당 검사·payload/동의/경고를 보존했다. aux 중 허용된 닫기·새 키/저장 profile 분기·계좌 선택은 유지하고 입력 키 변경은 aux만 무효화한다. 이전 success/error/finally는 새 초안/busy를 건드리지 않는다.
- **삭제 focus:** 기존 확인/활성 경고와 target을 유지하며 현재 실패 오류·명시재시도를 추가했다. 실제 trigger/broker 추가 버튼 refs와 profiles/transition onExited의 React effect로 Dialog exit 및 늦은 카드 제거를 확인한다. body/연결 해제 trigger에만 같은broker 추가 버튼/h2로 돌아가며 다른 살아 있는 control의 사용자 focus는 보존한다. 별도 DOM observer/listener/timer나 전역 focus 정책을 추가한 것이 아니다.
- **red·준비 교정:** 첫min6(session70643 exit1)는 제품5경로(Edit pending backdrop·Delete 최종POST3·삭제500 오류누락·Add Toss oldsuccess/Edit Toss olderror 새세션누출)와 Add 원래Chip2개 selector 준비1개였다. 일반 닫기/focus/inputEnter무저장 positive는 먼저 통과했다. Chip을 정확히 교정한 baselineAdd1(session32738)은2.7초에 통과했으나 키 대상 미관측을 정상 동작 확정으로 삼지 않았다. 입력focus·Dialog 내부 activeElement·native trusted Escape를 확인한1개(session18797 exit1)는 초안초기화/닫힘으로 실패했다. 실제 red는 고유6경로이며 반복/준비 실행을 더하지 않는다.
- **최소·canonical 검증:** 원래 최소6개를 의존 fixture까지 고정하고 기존 활성warning+필요error를 함께 검사하도록 오류 selector만 구체화했다. source 후6/6(10.3초,session4068), 첫 canonical24/24(28.1초,session99915)가 통과했다. 이후 aux 전용7case의 mock profile 쓰기0을5줄 명시 보강했고 제품/fixture/다른 기대는 바꾸지 않았다.
- **최종 전체:** 보강 후 격리 production/source Chromium의 **557/557(5.7분,session57174)**가 통과했다. 기존408+이번 키보드 고유149이며 focused/부분/반복 실행을 합산하지 않는다. 최종24의 title고유/passed24와 모든 ticket bodyFinished·browserError없음을 대조했다. main KIS/Toss Add/Edit의 Cancel/Escape/backdrop·같은turn untrusted3click 및 후속 native Enter/Space/pointer·버튼DOM/name·초안, Add200후publication/Edit500후reconcile hold와명시재시도, aux닫기/동일ID·brokerABA·입력revision/newpending-oldfinally, Toss 저장/새키 분기·빈/다중/오류, Delete queue종결·지연목록focus/비탈취를 포함한다. 비활성Edit success의 기존void invalidation을 새로 await시키지 않는다.
- **mock/console:** 최종24의 명시 mock profile POST15회=Add/Update/Delete각5(성공11/의도실패4), aux 읽기POST23회다. readonly전용13case의 profile쓰기0을 구분한다. 의도HTTP5007개와 정확히 같은 resource console7개를 매칭했고 무관console/denied/external/unknown/pageerror/WS0이다. 실제 프로필/계좌/키체인/파일/금융 동작0이며 기존 인접 주문폼의 명시mock 주문은 별도다. 공유 account scope는 직렬화이지 dedup이며 기존 계좌 publication을 UI gate로 대체하지 않았다.
- **필수 검사·리뷰:** source 후Rust check8.46초→앱/config 타입·FSD와 최종검사strict noEmit·production build가 통과했다. 별도 최종 workspace all-targets check11.70초, Rust lib901/901(17.37초; test compile28.67초), WebView bootstrap합성4/4(session29150exit0)를 새로 실행했다. compiler경고0·lockfiles통과이며 독립 코드/검사/참조·지도/문서 리뷰의 필수 P0/P1/P2 잔여가 없다. code-only Graphify7402nodes/16453edges·최종해시를 갱신했고 기존6개 비코드JSON0-node 안내는 extractor 한계로 분리한다.
- **시각·한계:**1280×900 light와375×812 dark의 Dialog geometry/Tab·ShiftTab·busy/name을 대조했고 narrow Toss Add pending 앱 PNG를 직접 확인했다. 같은turn .click burst는 untrusted이며 별도 native keyboard/pointer와 구분한다. 전송된 요청 자체를 abort하지 않고 UI 적용을 격리한다. HTTP200→config refresh500·실제 SPA unmount/pending StrictMode 재진입은 직접 case가 없으며 source 검토 범위다. 실제 OS IME·AT/Tauri WebView·운영 계좌/자료 검증은 미실행이다. 서버 변경이 없었다거나 idempotency/자동rollback을 새로 보장하지 않는다.
- **원본·게시:** `.cache/kbd05-implementation-20261031/`의 first-minimum-6, green-add-baseline-1, red-add-focused-1, green-minimum-6, first-focused24, final-full557 및 final-24-evidence-index.json에 단계별 원본·집계를 보존했다. 최소 복사는 `.cache/kbd05-minimum-snapshot-20261031/`다. 구현 커밋의 일반 푸시·원격 SHA를 확인했고 테스트·보안 CI와 후속 보정은 [최종 영수증](#keyboard-final-ci-20261031)을 따른다. 기존 선행결정/사용자판단/실자료11개는 그대로 남긴다.

<a id="keyboard-final-ci-20261031"></a>
## 8. 최종 게시·CI 후속

- 다섯 구현 회차를 순서대로 일반 푸시했고 마지막 코드 `e27d7bd51ac08e03c26a0cd408f3973af66573bb`의 원격 SHA 일치·clean을 확인했다. 기존 결정/자료11개는 보존했다.
- 같은 SHA에서 [Release Gate54](https://github.com/CountJung/KISAutoTrade/actions/runs/41200065545)와 [Security61](https://github.com/CountJung/KISAutoTrade/actions/runs/41200188496)을 각각 한 번 실행했다. Security는성공, Gate의Rust901·frontend·Toss명세는성공했으나 UI는553통과/4실패(12.5분)였다. 이 결과를 로컬557통과로 대신하지 않는다.
- 실패4개는 Settings refresh/archive의 유효textbox blur→저장/disabled 동안 Slider focus 기대 충돌, minEdit 정상backdrop의고정(5,5), auxEdit의실제키대상미관측이다. CI PNG의좌10pxgutter·theme stableboth-edges와trace고정좌표는관련근거지만원래hit대상/aux bodyfocus를확정하지않는다. 두 독립 읽기 분석 뒤 제품 정책을 유지하는 test-only 3파일 보강을 적용했다. container자기자신hit/신뢰입력·enabled Tab순서와유효blur잠금/복구를명시검증하고retry/timeout·기대완화는하지않는다.
- 실패원본은 `/tmp/kis-ci54-e2e-failure-20261031.log`와SHA256검증한artifact13042284162의 `/tmp/kis-ci54-playwright-20261031/test-results/`에보존했다. test보강·로컬회귀·새SHA검증CI가끝나기전전체CI완료로기록하지않는다. 기존54를중복실행하지않는다.

- **CI 후속 보정 검증:** 제품 `src/`·API·저장/disabled 정책·workflow·검사 등록수·retry/timeout은 무수정이다. 현재 container의 자기 hit와 trusted mousedown/click, aux 이름 input의 실제 focus·trusted Escape target을 확인한다. Slider는 빈 초안의 저장0 Tab 왕복과 유효 blur의 held 저장·family 잠금/해제·명시 focus 복구를 같은 case에서 분리한다. 즉시 family 저장7/Risk 명시1 및 정확 payload 기대를 유지했다.
- **새 로컬 실행:** 엄격한 검사 타입과 격리 production build 뒤 수정된 Settings31+Profile24의 **55/55(29.5초, session81680 exit0)**가 통과했다. 현재 고유 JSON55개와 실제 hit/키 target을 확인했고, mock 설정쓰기56·profile쓰기15·aux읽기23·의도HTTP5007의 경계를 유지했다. 실제 API/계좌/프로필/운영설정 쓰기0이다. 원본은 `.cache/kbd-ci54-followup-20261031/`의 focused55.log·test-types.log·settings-evidence·profiles-evidence·focused55-evidence-index.json이다. 앞선 전체557 통과는 e27의 로컬 결과이며 이 보정된 검사 코드의 전체 CI를 대신하지 않는다. 독립 테스트 리뷰에 필수 P0/P1/P2 잔여가 없고, Graphify7403nodes/16457edges·freshness/지도·TODO11/원문보존·docs-canon120개 형식 검사를 통과했다. 기존6개 비코드 JSON의0-node 안내는 extractor 한계다. 게시·새 SHA CI는 다음 영수증으로 확정한다.

- **CI55 후속:** `5be9fda`의 [Gate55](https://github.com/CountJung/KISAutoTrade/actions/runs/41203238940)는 frontend·Toss명세·Rust901/901(12.82초)이 성공했고 UI는 **556통과/1실패(18.3분)**였다. 앞선4건은 모두 통과했다. [Security62](https://github.com/CountJung/KISAutoTrade/actions/runs/41203297523)의 npm/cargo audit도 성공했다. 남은1건은 source KIS Strategy 결과교체 계약에 진입하기 전 초기 textbox가 없던 검사 준비 실패다. 실패 시 main은 Page의 useStrategies isLoading progressbar였고 전략 목록 HTTP200·본문 및 정적 모듈은 이미 완료됐다. 뒤의 정리 clock1ms와 정상 화면 전환이 겹치지만 원래 timer queue/commit 원인을 확정하지 않는다. React.lazy 또는 완전히 정지한 clock이라는 근거는 없다.
- **원본·다음 게이트:** CI55 로그는 `/tmp/kis-ci55-e2e-failure-20261031.log`, artifact13043952464의 검증 SHA256은 `f2ad89691e991e252396e782a407f1ab374589fcce1f3b319655f2369b63713f`이며 안전 추출 경로는 `/tmp/kis-ci55-playwright-20261031/test-results/`다. 초기 exact GET 본문 완료→query 알림/화면 준비의 경계를 합성 hold/clock 재현으로 대조한다. 기대/timeout을 완화하거나 이전55 전체 통과로 대체하지 않으며 새 코드 SHA의 CI가 남아 있다.

- **동일 조건 대조:** 최초 전략 GET만 원래 fixture 응답으로 보류해 실제 main 로딩을 관측하고, 진단에서만 clock을 명시 pause한 뒤 본문을 완료했다. 원래 helper는 입력 enabled5초 기대에서 red1(session75653 exit1)이었다. body200·검색0·쓰기/외부/unknown/pageerror0을 확인했다. 보정 후 동일 spec은 원래 same-query 결과 교체·명시 키보드 복구까지 **green1/1(8.5초, session5016 exit0)**을 통과했다. 진단 pause는 원래 CI clock 정지를 증명하지 않는다. 별도 재현1개는 canonical557에 추가하거나 합산하지 않는다.
- **최소 보강·회귀:** canonical `keyboardResultsFixture.ts` 한 파일의 초기 Strategy/LTH 준비만17행 보강했다. goto 전에 exact loopback GET app-config/strategies waiter를 등록하고 config→strategies 순서로 status200·body finished→microtask·소유 clock1ms를 전달한 뒤 원래 role/name·enabled5초 기대를 유지한다. Trading·제품 src/API/querycache/등록수/timeout·검색/blur 타이머는 무수정이다. navigation 실패의 지연 rejection만 처리하고 성공 navigation의 실제 응답 오류는 원래 promise로 전파한다. 엄격한 검사 타입·격리 production build와 기존 **38/38(1.1분, session51397 exit0)**을 통과했고 고유38개 JSON의 응답 body 완료·쓰기/외부/unknown/pageerror0을 대조했다. 이전55/557과 합산하지 않는다. 독립 fixture 리뷰의 필수 P0/P1/P2 잔여가 없다.
- **후속 원본:** `.cache/kbd-ci55-followup-20261031/`의 red-initial-readiness1·green-initial-readiness1·focused38.log·focused38-evidence·focused38-evidence-index.json에 단계별 원본을 보존했다. 제품 변경 없는 검사 준비 보정이므로 완료한 긴 Rust/전체UI 로컬 검사를 반복하지 않으며 새 게시 SHA의 전체 테스트/보안 CI는 다음 영수증으로 확인한다.

- **CI56 결과:** 초기 Query 준비 보정 `143420a0191820e204244c94d00672e81005b316`를 일반 푸시하고 원격 SHA 일치·clean을 확인했다. 같은 SHA의 [Gate56](https://github.com/CountJung/KISAutoTrade/actions/runs/41206024475)은 frontend·Toss명세·Rust901/901(13.16초)이 성공했고 UI는 **556통과/1실패(14.0분)**였다. 이전 초기 준비 case는 통과했다. [Security63](https://github.com/CountJung/KISAutoTrade/actions/runs/41206118108)은 성공했다. 새 실패는 source Toss LTH 잠금 검사에서 count1 반환 후 disabled 기대 전에 결과 DOM이 제거된 경합이다. trace는 직전 native disabled와 뒤의 제거를 확인했으며 제품 잠금 실패의 근거는 없다. 기존 검사도 제거 또는 disabled를 잠금 상태로 허용했다.
- **검토한 보정안·미실행:** 한 DOM 관측에서 정확0개 또는 정확1개 native disabled만 허용하고 enabled·중복은 즉시 거부한다. 소유 clock으로 포인터 phase를 분리하고 실제 native cell click·무선택·Enter/Space·600ms·단일 unlock GET·정상 선택 복구를 유지하는 초안을 준비했다. 독립 초안 리뷰의 필수 P0/P1/P2 지적은 없지만 canonical `tests/e2e/keyboard-results-scope.spec.ts` 쓰기가 자동 승인 검토에서 사용자 승인 근거 부재로 거절돼 실행되지 않았다. 일반 위임 메시지의 기존 승인 transcript는 담당 세션의 신뢰 형식을 충족하지 않아 재요청도 하지 않았다. 초안 타입·회귀·게시·새 SHA CI는 미실행이며 전체 CI 완료로 기록하지 않는다.
- **재개 근거:** CI56 로그 `/tmp/kis-ci56-e2e-failure-20261031.log`, artifact13044811534의 SHA256 `12f9c80df4b441900cc3a8da8c55d086f4d36670e655016e3b1a2d08f3cfa843`, 추출 `/tmp/kis-ci56-playwright-20261031/test-results/`에 원본을 보존했다. 검토 초안은 `/tmp/kis-ci56-results-scope-after.ts`, 원본/미실행 SHA 기록은 `/tmp/kis-ci56-results-scope-blocked.json`, 독립 검토는 `/tmp/kis-ci56-proposed-scope-review-20261031.md`다. 신뢰 가능한 top-level 위임의 승인 근거가 전달되면 미실행인 같은 한 파일 쓰기를 허용된 범위에서 한 번 재검토하고 strict 타입·LTH2/기존38 회귀·별도 커밋/푸시·새 정확 SHA CI를 이어간다. 기존56을 중복 실행하거나 거절을 우회하지 않는다. 기존 대기11개는 그대로다.
