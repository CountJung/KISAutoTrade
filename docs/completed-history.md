# KISAutoTrade — 완료·경과 이력

> 현재 남은 작업과 완료 조건의 정본은 [todo.md](todo.md)다. 이 문서는 완료된 범위·과거 관측·기각 이유를 요약한다. 아래 수치는 당시 검증이며 문서 압축 중 재실행한 결과가 아니다.
> 원문은 [고정 커밋 4021e38의 이력](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md)에 보존한다. 각 절의 불변 링크에서 원래 TODO 인용·중간 실패·수치·근거를 복원할 수 있다. 2026-10-29 이전 이관 원본은 `6eeb0ea718138438296b832bcbbe3e218fc49564`의 TODO다.

당시 짧은 `P2-13` 표기는 `SOXQ-P2-13`을 뜻한다. 현재 `DEP-P3-01`의 MSRV/DB 결정과 나머지 열린 조건은 [TODO](todo.md)에서 관리한다.

<a id="p2-ux-07"></a>
## P2-UX-07 — 위험 행동 확인·설정 가시성 보강·원격 CI 통과 (2026-11-07)

- 기준 `80c81c86794507262bde5f248279722d5ea28a46` 위의 공용 확인 구현은 `586004310ef6c4c60dd43b9727ba21985a1c96a6`으로 푸시했다. 아래 원격 frame·추가 지연 레이아웃 및 테스트 입력 전제의 후속 보강은 로컬 전체 회귀와 최종 독립 코드·문서 검토를 통과했다. 후속 구현은 `dff35f4651e683923969ecfbed0629c2bdea5127`로 정상 커밋·푸시했고 해당 SHA 원격 CI 성공까지 확인해 P2의 코드/mock 검증 범위를 완료했다. 실제 native·계좌와 P1 잔여 조건은 아래 경계대로 유지한다.
- 실전 시작·비상정지 해제·매수 재개·KIS 국내/미국·Toss 수동 주문은 `features/financial-confirmation`의 공용 Dialog·동기 소비/공유 잠금을 사용한다. KIS 모의 시작 면제는 요청 시 캡처한 동일 신뢰 config로 판단한다. 확인에는 broker/account/모드·실행 영향과 수동 주문의 종목·매수/매도·수량·유형/가격을 표시하며 순수 모의 계산은 대상이 아니다.
- 정지·비상정지는 즉시 요청하며 미전송 확인만 무효화한다. 기접수 주문 취소·포지션 청산·이미 전송한 요청 취소나 모든 정지 mutation의 전역 잠금을 뜻하지 않는다.
- 비상정지 해제는 daemon 시작/손실 카운터 초기화가 아니며 매수 재개는 입금/잔고 검증이 아니다. 기존 backend/IPC·동의/preflight·scope/risk·예산/예약/멱등·원장/수수료 정책은 유지한다. **P1 원장 모드·legacy 검증 이관·server ABA는 미해결**이다.
- scope·AppConfig 공개세대·fetch/error/invalidated/publication pending·초안/입력·unmount와 경로 ABA가 기존 확인을 무효화한다. opener는 busy 전에 캡처하고 닫기 transition 후 새 확인이 없고 연결된 경우에만 복귀한다.
- 부모는 Escape/backdrop 포커스2건과 same-task 모의→실전/경로 ABA의 RED→GREEN을 관측했다. RED에는 최종 확인 전 mock start POST1과 되살아난 승인이 있었으며, 수정 후 core 독립 리뷰와 SHA256 일치를 확인했다.
- exact origin/method/path/noquery/body·전체 요청열·취소0/승인1·중복 방지·6종403 negative control을 검사했다. unknown API/외부/앱 WS 차단을 유지한다. 캐시/Dialog scope 대조는 무본문 요청이 서버 identity를 전달한다는 증거가 아니다.
- 부모의 envDir:false source HTTP mock focused 실행은 확인 흐름32개·기존 주문폼5개의 **37/37 통과, 실패·skip0**다. synthetic IME/229·브라우저 modifier/repeat·정상 Sell·해제 비대상 상태 보존·375px/dark DOM reflow·색상 대비를 관측했으며 테스트-only focus 강제는 없다.
- CI와 같은 source 프로젝트 대상을 로컬에서 실행해 **88/88 통과, 실패·skip·flaky0(1.6분)**를 확인했다. 당시 CI 정본 등록은 dist UI541개+source88개=629개이며 등록은 실행 통과가 아니다. source88에는 focused 확인32개가 포함되므로 중복 합산하지 않는다.
- 기존 source-harness 진단34/34와 기존2spec25/25도 별도 결과다. 예산 복구 exact 기대값의 profile/mode를 추가하고, frozen clock의 실제 응답/body·초기 config 공개 barrier와 held mock IPC 진입 확인을 보강했다. 기존 debounce/IME/횟수·timeout·금융 guard는 완화하지 않았다.
- `5860043`에 반영한 부모 코드의 `npm run build:web -- --config .cache/p2-ux-07/offline-final-build.config.mts`는 **exit0**이다. `envDir:false`·별도 `.cache/p2-ux-07/final-dist`를 사용했고 실제 preview의 index/router와 빌드 파일 SHA256 일치를 확인했다. 이후 제품 소스7개는 같은 해시를 유지했다. 앱/Node/변경 spec 타입·lockfiles·FSD·TODO·project-map·Graphify freshness·diff도 통과했다.
- 로컬에서 CI 정본 dist541개를 처음 실행한 결과는 **534pass/7fail, skip·flaky0, 전역 오류0(9.0분)**다. 6건은 기존 Settings navigation의 exact GET 기대값에 profile/mode가 빠진 것으로 trace에서 확인해 두 필드를 보강했다. 나머지 차트 검사에서는 네이티브 스크롤 부분 이동 뒤 다음 키를 보내고 한 픽셀 일찍 종료하는 경계를 관측했다.
- 키 전에 실제 scroller의 trusted `scrollend`를 준비하고 완료 후 다음 키를 보내며, 기존 오른쪽 끝·마지막 열 가시성 조건을 함께 만족할 때만 반복을 끝내도록 보강했다. 실제 이동·헤더/값 가시성·focus·overflow·기존 허용 오차와 timeout을 유지했다. 수정 원본2spec은 **14/14**, 진단 observer 없이 원본 KR/US 각10회는 **20/20**이다. 별도 차단 대조군2건은 네 번째 키를 막자 중간120에서 의도대로 실패해 무이동을 정상 끝으로 오인하지 않음을 확인했다. 마지막2파일의 독립 리뷰는 새 P1/P2 없음이며 부모도 같은 SHA256을 확인했다.
- `5860043`을 준비한 로컬 production 번들의 CI 정본 dist 프로젝트는 **541/541 통과, 실패·skip·flaky0, 전역 오류0(7.1분)**다. 부모가 JSON으로 unique541건과 앞선 source88건의 비중복 합계629건을 확인했다. focused·진단·반복 시도를 여기에 더하지 않는다. 완료 뒤에도 제품7파일·마지막 리뷰2파일·빌드 index/router 해시는 동일했다. 상세 명령·JSON/trace는 `~/.hermes/cache/kisautotrade-e2e-audit/`, 압축 전 경과는 `~/.hermes/cache/kisautotrade-sequential/`에 보존한다.
- `5860043`의 실제 PR16 원격 CI는 보안·frontend·Rust·Toss·빌드가 성공했으나 E2E는 **628/629 통과·실패1, skip·flaky·전역 오류0**였다. 실패는 `uiux-settings-navigation.spec.ts`의 지연/실패 risk 첫 목적지 geometry 검사다. 공개 artifact13570048150의 ZIP 크기·SHA256을 GitHub 원 metadata와 대조한 뒤 HTML을 실행하지 않고 report JSON/trace를 해석해 정확한 실패를 확인했다. 로그403을 반복하거나 새 인증·자격증명을 요청하지 않았다.
- 원격 Enter 전후 snapshot은 main3553과 숨은 셸 부모325의 동시 스크롤을 기록했다. 로컬 observer도 부모324·main rect−324…396을 관측했지만 기존 상대좌표 검사는 통과했다. main 자체의 viewport 포함까지 요구하자 기존 불변 번들에서 **RED1/1**을 재현했다. 제품의 Settings local helper를 main만 이동하도록 바꾸되 notify·preventScroll focus·기존 margin·자연 clamp·경로/user-intent 복원을 보존했다. main 밖의 기존 fallback은 유지하며 전역 스크롤 초기화나 테스트-only 보정은 없다.
- 후속 `npm run build:web -- --config .cache/p2-ux-07/offline-navigation-build.config.mts`는 **exit0**, outDir은 `.cache/p2-ux-07/navigation-final-dist`다. 강화 원본 Settings/Chart는 **14/14**, Settings 반복은 **30/30**이다. 새 코드의 CI source 프로젝트는 개발 클라이언트/HMR을 유지하고 `.env`만 제외한 별도 포트에서 **88/88 통과·실패/skip/flaky/전역 오류0(1.7분)**였다. 소스·spec 두 파일의 독립 리뷰는 새 P1/P2 없음이며 부모가 동일 해시를 확인했다. 새 preview의 index/router도 빌드 파일과 일치했다.
- 후속 새 번들의 정본 dist는 **541/541 통과·실패/skip/flaky/전역 오류0(9.0분)**다. 부모가 새 두 JSON의 모든 결과와 물리 case를 projectName 없이 대조해 합계629·파일 교집합0을 확인했다. 반복·focused·진단은 합산하지 않았다. 실행 뒤에도 제품8파일·spec2파일·번들 index/router 해시는 빌드 영수증과 같았다. **이 시점에는 후속 새 SHA의 원격 CI 대기 상태였으며** `5860043`의 원격 실패를 성공으로 바꾸어 기록하지 않는다.
- 추가 격리 진단은 risk 응답을 보류한 채 budget 응답을 첫 제목 이동 뒤에 공개하고 실제 응답 완료·예산 입력 렌더를 기다렸다. **5회 모두** heading focus는 유지됐지만 가시성 검사가 실패했다. 스크린샷의 Sidebar/main frame은 정상 높이였고 risk 제목은 아래로 밀려 보이지 않았다. 정본 통과는 이 지연 순서의 해결 근거가 아니다. 후속 커밋을 보류하고 로컬 포커스 가시성의 레이아웃 추적·사용자 의도 취소·cleanup 및 정본 회귀의 추가 보강으로 넘겼다. 원격 실패에서 지연 레이아웃의 정확한 기여는 아직 단정하지 않는다.
- 지연 보정 후보는 `settingsNavigation.tsx`의 단일 소유 추적·DOM/크기 관측·RAF·입력/경로/unmount 정리와 risk 정본의 응답 순서·실제 wheel/PageUp 취소 대조를 보강했다. 별도 `.cache/p2-ux-07/layout-anchor-final-dist`의 `envDir:false` production build는 **exit0**, 실제 preview index/router 바이트·해시를 확인했다. 같은 정본 risk는 이전 불변 main-only 번들에서 **RED1/1**, 새 번들의 Settings/Chart는 **16/16**, 원본 Settings 반복은 **40/40(1.1분)** 통과했다. passive capture wheel/key 취소만 누락시킨 negative control은 **2건 모두 예상 실패**하여 취소 검사의 반증 가능성을 확인했다. app/spec/Node TS·FSD·lockfiles와 실행 뒤 제품8/spec2/번들2 해시는 통과/일치했다. 이때 CI dist **543개는 등록 목록만 확인했다**. 새 코드의 같은 raw Vite/HMR·envDir:false source 실행은 **88/88 통과·실패/skip/flaky/전역 오류0(1.6분)**였으며 부모가 JSON의 물리88·모든 결과와 실행 뒤 입력 해시를 확인했다. 당시 전체 dist 재검증·최종 독립 리뷰·새 SHA 원격 CI는 후속 게이트로 남겼다.
- 독립 리뷰는 제품 소스의 새 P1/P2는 없었지만 취소 spec이 native 스크롤의 첫 이동만 기다린다는 P2를 지적했다. 진행 중인 전체 dist는 의도적으로 중단하고 종료를 확인한 뒤, 입력 전 정확한 main의 trusted `scrollend` 설치·완료 대기와 `finally` 정리를 보강했다. 중단 실행을 전체 통과/실패나 완료 건수로 기록하지 않는다. 제품8·번들 해시는 그대로라 검증된 build를 재사용하고 spec-only 영수증으로 변경을 구분했다. 강화 후 원본 Settings/Chart **16/16(22.2초)**·원본 Settings 반복 **40/40(1.1분)**·취소 누락 대조 **2건 예상 실패**와 spec TS를 재확인했다. source의 실행 대상에는 수정한 UI spec이 없고 runtime도 바뀌지 않아 앞선 source 근거를 유지한다. 이후 새 전체 dist·test-only 재리뷰·최종 문서/원격 CI를 진행했다.
- 다음 전체 dist는 **542통과/1timeout·skip/flaky/전역 오류0(9.3분)**였다. 실패는 `candle-pagination.spec.ts`의 repeatedCursor case에서 collector 실행 전 미리보기 버튼을 기다린 것이다. trace의 구간 선택은 클릭 완료 로그 뒤에도50봉·menu open이었고 candle-page 요청은0이었다. 원본 격리 **10/10** 재실행은 원인 해결 근거로 삼지 않았다. popup 이동을 native down/up 사이에 놓은 별도 timing fault에서 원래 helper는 같은30초 대기로 실패하고 준비 완료를 기다리는 helper는 통과했다. 이는 moving-popup lost-click 기전 대조이며 원래 native packet의 정확 인과를 단정하지 않는다. 영구 spec에는 CSS/delay fault 없이 Grow 완료→원래 클릭1회→선택값/listbox 해제만 보강했고 collector·fixture·기존 oracle은 수정하지 않았다. 확장 spec TS와 원본 Candle/Settings/Chart **38/38(49초)**·수정한 repeatedCursor 반복 **10/10(16.9초)**을 확인했다. 제품8·번들·Settings spec은 불변이며 새로 바뀐 두 UI spec이 source 실행 대상 밖임을 확인했다. 다음 전체 dist와 추가 test-only 리뷰로 게이트를 넘겼다.
- 메뉴 보강 후 같은 production 번들의 정본 dist는 **543/543 통과·실패/skip/flaky/전역 오류0(9.1분)**다. 부모가 새 JSON의 모든 단일 결과와 등록 목록의 case 일치, 앞선 동일 runtime source88 결과를 대조해 **물리631개·파일 교집합0**을 확인했다. source 실행 뒤 변경한 두 UI spec은 source lane 대상 밖이며 제품8·번들2는 불변이라 source 근거를 유지한 것이지 새 source 실행을 주장하는 것은 아니다. focused·반복·진단·의도적 실패·중복 완료 알림은 더하지 않았다. 실행 후 제품8/spec3/번들2 해시는 영수증과 같았다. Settings spec의 독립 재리뷰는 이전 native 완료 대기 P2 해결·새 P1/P2 없음이며 부모가 제품/spec 동일 SHA256을 확인했다. Candle helper 독립 리뷰도 새 P1/P2 없음이며 부모가 동일 SHA256을 확인했다. 최종 문서 delta pass도 새 actionable 없음이고 report JSON을 독립 파싱한 물리631·파일 교집합0이 부모 결과와 일치했다. 이 시점에는 후속 새 SHA 원격 CI 게이트가 남아 있었다.
- 후속 `dff35f4651e683923969ecfbed0629c2bdea5127`의 실제 PR16 **Release Gate42374152326·Dependency Security42374152224 모두 success**를 확인했다. 공개 job/step metadata에서 e2e·frontend·rust·toss-openapi 및 npm/cargo audit가 모두 성공이며, production build와 `npm run test:e2e`, 기본/research-only Rust check·library test도 실제 실행 성공이다. 실제 commit blob은 검증 제품8/spec3/payload12 해시와 같고 원격 SHA·clean도 일치했다. 성공 시 실패 전용 보고서 업로드 step이 생략돼 artifact는0이며 **원격 테스트 건수는 별도 확정하지 않았다**. 로컬 물리631을 원격 실행 건수로 복사하지 않는다. 근거는 `~/.hermes/cache/kisautotrade-e2e-audit/p2-ci-dff35f4-success-receipt.json`에 보존한다. P2는 이 코드/mock·CI 범위에서 완료하고 실제 OS IME/native·실계좌/API/주문/운영 DB와 P1 원장·legacy/server ABA는 미완료로 유지한다.
- all629를 dev에 강제한 추가 진단은3회 조기 종료했고 마지막은234pass/3fail/1interrupt/391not-run이다. 실패3건은 LiveDailyHistory UI이며, 별도 격리7건은6pass/1fail이다. 개발 HMR을 strict0 가드가 잡은 설정 불일치는 개발 entry만 제거했으며 앱 WS 가드는 유지했다.
- 원래 assertion을 유지한 진단에서 cache success/idle·512행과 실패 직후 기대 DOM 요약을 관측했지만5초 assertion은 실패했다. 개발 초기화/렌더 타이밍 추론일 뿐 production 회귀 해결 증거가 아니며 해당 UI/spec은 수정하지 않았다. 현재 검증은 CI의 dist/source 분리를 따른다.
- 이전 `80c81c8` CI의 보안/Rust/frontend/Toss/build는 성공, E2E는 실패했다. 원문 로그는 권한403으로 미확보해 원격 실패 테스트명은 확정하지 않았다. 실제 OS IME·native WebView/Rust IPC·provider/계좌/API·주문·daemon/운영 DB·서명/배포는 실행하지 않았으며 mock 통과로 승격하지 않는다.

<a id="p1-data-01"></a>
## P1-DATA-01 — LTH bounded 수집·요청 수명·성공 요약 부분 수용 (2026-11-08 KST, 코드/offline 범위)

기준 HEAD `67f8d9ca2618aabbe17533ba4d8faaf80f78f276` 위의 A2/A3/A4/A5.1/A5.2/A5.3 변경을 부모가 검증한 기록이다. **전체 P1-DATA-01은 미완료**이며 실제 provider/자료/native 조건은 [TODO](todo.md)의 열린 ID에 유지한다. 구현 SHA의 커밋/푸시·원격 CI는 아래 근거대로 완료했다. 이 마감 문서 갱신은 새 제품 실행이 아니며 문서-only 후속 SHA의 CI와 구현 SHA의 결과를 구분한다.

- **수집·실행 연결:** A2 bounded 페이지/cursor·누락/중복·단일 cutoff/absolute deadline, A3 요청 소유 실행, A4 공통 IPC/REST 작업 수명·취소와 UI 세대 무효화를 연결했다. US1m은 `lth_fetch`→`lth_collection`→`lth_regular`, KR/일봉은 기존 조회 호환 경로다. EMA20/60은 실제 분봉 stream을 사용하므로 일봉200개를 필수 warmup으로 삼는 초기 가설은 철회했다. 수집된 typed 봉·중복 제거·admission 제외·평가 채택 수를 구분한다.
- **출처와 성공 wire:** A5.1은 완전히 수신한 identity HTTP body의 UTF-8/JSON 변환 전 SHA/길이·수신 시각을 같은 성공 retry 결과에 결합한다. A5.2의 버전 있는 native typed hash는 raw body 및 기존 engine hash와 별개다. A5.3은 기존 단일 Summary를 성공 `collectionSummary`로만 운반하고 UI는 구조 검증·표시만 수행한다. 실패는 기존 일반 오류이며 내부 partial Summary의 상세 횟수/사유를 wire/UI로 공개하지 않는다. 정확 계약은 [IPC](ipc-commands.md)·[사용법](user-guide.md)이 소유한다.
- **단계 수용:** A2/A3/A4/A5.1 이후 A5.2는 12:01 KST, A5.3 성공-only는 13:13 KST에 코드/offline 범위로 수용했다. 단계별 `P1-DATA-01-A{2,3,4}-accepted.json`, `A5-transport-accepted.json`, `A5-collection-accepted.json`, `A5-wire-accepted.json`은 `~/.hermes/cache/kisautotrade-sequential/`에 보존한다. worker의 loopback PermissionDenied/browser EPERM은 원래 미실행이고 부모의 후속 실제 실행으로 별도 귀속한다.
- **최종 부모 검사:** `P1-DATA-01-A5-wire-review-final-parent-verification.json`의 실제8게이트가 모두 exit0·Rust warning0이다. `cargo test --workspace --lib --locked --offline` **1035통과**, focused strategy_preview **181통과**(전체 lib에 포함), default/research-only all-targets check·app TS·영향43spec TS·FSD·개인 환경 제외 새 web build가 통과했다. HTTP response20·default adapter2·production library 통합6은 앞선 부모 실행에서 통과했고 최종 delta가 cfg-test뿐인 생산 동일성으로 **보존**한 것이지 새 실행이 아니다. 집중·부분 결과를 lib 총수에 합산하지 않는다.
- **DB 경계:** `KISAT_PG_HOST/PORT/USER/PASSWORD`는 존재 여부만 확인해 모두 false였고 값은 읽지 않았다. lib 총수에는 fixture 미설정으로 조기 반환하는 DB 계약 함수가 포함될 수 있다. 위 Rust 통과를 PostgreSQL 접속·TLS·인증·운영 DB 검증으로 표현하지 않는다.
- **전체 정본 UI:** 공유 fixture 직접/전이 영향43spec의 **601통과(dist539/source62)**와 정확한 잔여9spec **76통과(dist50/source26)**를 실행했다. 부모가 등록 inventory와 두 실행의 물리 case ID·파일 교집합0을 대조해 **677/677 고유 통과(dist589/source88)**를 확인했다. 각 case 단일 passed attempt이며 재시도/skip/flaky/전역 오류0이다. `P1-DATA-01-A5-canonical-reconciliation.json`이 합집합 증거를 소유한다. 서로 다른 두 실행을 한 번의 full-suite 호출로 부르지 않으며 focused·반복·의도적 실패·중복 완료 알림은 합산하지 않는다. 소유 서버 수명 내 실제 HTTP13자산 바이트와 새 build가 일치했고 실행 뒤 입력/dist 및 동결 제품·보호 파일은 유지됐다. 이는 offline dist renderer/source-harness HTTP mock이며 actual Rust/native/provider 연결 E2E가 아니다.
- **반증 가능한 보강:** raw metadata 누락, 401→token→최종200의 오래된 source, source 빈 순회/잘못된 stream, 진입 잔여예산 누락, deadline 재충전 등은 별도 통제 fault의 실제 assertion RED와 복원 GREEN으로 확인했다. 준비/컴파일/권한 오류는 behavioral RED로 세지 않았다. 비협조 Ready deadline oracle는 실제 adapter 진입 뒤 paused clock을 같은 poll에서 이동하며 고정 Tokio1.54.1의 advance-before-yield 구현에 의존하므로 업그레이드 시 재검토한다.
- **기존 결과 호환:** 현재 구현 두 번 비교는 결정성만 입증하므로 보존된 before 함수에 fake adapter만 기계 주입해 1m/1d engine input hash·backtest JSON SHA 고정 oracle을 얻었다. 신규 테스트는 실제 production 실행을 호출하고 expected를 현재 코드에서 계산하지 않는다. 동일 hash fault에서 기존 선택1개 PASS/신규 고정 oracle1개 assertion RED, 복원 후 GREEN을 관측했다. 전체 기존180개가 fault 아래 통과했다는 뜻은 아니다. 임시 include/reference/probe는 최종 소스에서 제거했고 독립 delta 리뷰는 조치할 결함 없음을 확인했다.
- **stale UI 통제:** 정상 선택3개 PASS 후 보호 hook 소스는 바꾸지 않고 별도 Vite transform artifact에 오래된 결과 게시 fault를 넣었다. 3개 모두 body 완료 뒤 latest-summary assertion에서 예상대로 실패했고 원본 소스·정상 dist는 유지됐다. 이6실행은 정본677에 포함하지 않는다. 특정 fault 검출력이며 모든 앱 continuation ordering의 보편적 인증은 아니다.
- **정본·구현 SHA 마감:** 문서/HTML·project-map·Graphify 갱신 및 신선도·lockfile/diff의 부모9게이트와 별도 Node 회귀7개가 통과했다. 제품/계약·보호64 해시는 유지했고 내용4문서 리뷰는 새 actionable 없음, 지도/그래프 리뷰의 protocol 표기1건은 실제 `reserve → run(await) / cancel`에 맞춰 두 곳을 수정했다. 경미한 수치/표기 delta는 역제거 SHA 일치와 직접 근거·후속 검사로 확인했으며 전체 제품 검증을 새 실행으로 집계하지 않았다. Graphify zero-node6 경고와 대형 HTML 시각화 생략은 Rust warning0와 별개다.
- **원격 실측:** 구현 `25d4d32448a966b15db995ee668db4b0c1361834`를 현재 브랜치에 정상 커밋·푸시했다. 정확44경로의 staged/committed blob은 검증/리뷰 해시와 같고 원격 SHA·clean 상태를 확인했다. 같은 head SHA의 Release Gate **42448719636**·Dependency Security **42448719642**가 모두 success이며 E2E **128633122224**, Rust **128633122260**, frontend **128633122304**, Toss **128633122121** 및 cargo/npm audit가 성공했다. `npm run build:web`·`npm run test:e2e`와 기본/research-only Rust check·lib test의 실제 step 성공을 확인했다. 원격 정확 case 수는 별도 확정하지 않았으며 로컬677을 원격 통과 수로 복사하지 않는다. 원문 metadata·검증은 `P1-DATA-01-exact-ci-verification.json`, blob/원격 대조는 `P1-DATA-01-commit-receipt.json`에 보존한다.
- **미검증 유지:** 공개 규격/소스 snapshot 대조는 실제 broker 성공이 아니다. raw/typed hash와 수신 시각은 발행/PIT·정규장 전용 일봉 OHLC·조정·특별휴장 완전성·30일/3개월 coverage 인증이 아니다. `lookAheadSafe=false`·표본/평가불가 경계를 유지한다. 실제 native/OS IME·계좌/API·새 인증·주문·자동매매/위험 해제·운영 DB·서명/배포는 실행하지 않았다. 실패 상세 공개, P1-SCOPE 원장 모드/검증된 legacy 이관/server ABA와 SECTOR 후속도 완료로 바꾸지 않는다.

<a id="p1-session-01"></a>
## P1-SESSION-01 — US LTH 연구 정규장·실행 근거 보강 완료 (2026-11-08 KST, 코드/mock·구현 SHA CI 범위)

기준 HEAD `de6d5eb0d6473bfca554d0c3c16aed25682585d7` 위의 동결 제품/검사16파일을 대상으로 한 부모 관측이다. A2 admission+execute 앞176행과 보호6파일은 유지했다. US `1m` 연구의 NY/DST 계획 정규장·단일 cutoff·동일 채택 cohort를 Open→지표/LTH→Replay와 차트에 연결하고, B는 선택 executionClock·현재/저장 A/B·legacy/unknown 표시를 추가했다. KR/일봉 v4·live·legacy KST blackout은 보존한다. 정확 필드/시계 버전은 [IPC](ipc-commands.md#lth-분봉-시각-진단-v1-2026-10-22)가 소유한다.

부모 영수증 `P1-SESSION-01-B-focused-parent-receipt.json`의 최종 Rust는 `cargo test --workspace --lib --locked --offline` **976통과/실패0/ignored0**, `cargo test --workspace --test lth_replay_clock --locked --offline` **6통과**, `cargo check --workspace --all-targets --locked --offline` **exit0·warning0**다. focused strategy_preview129/LTH38은 lib976과 중복되므로 합산하지 않는다. focused dist/mock UI는 **29/29·skip/flaky/전역 오류0, 약56초**이며 반복29를 다시 더하지 않는다. 테스트 webServer가 실제 제공한 index/router 바이트·SHA256이 빌드와 일치했다. 이는 native IPC/실제 provider 검증이 아니다.

부모는 분 경계가 아닌 초 단위 라벨의 마감 투영이 일반장15:59·단축장12:59로 잘리는 문제를 **RED exit101 → 계획 마감16:00/13:00 정확 투영 GREEN1**로 수정했고 최종 포맷 소스를 위 전체 Rust에서 재확인했다. 별도 기존 UI 대조의 RED1은 최종29 통과에 더하지 않는다. 동결 snapshot 영수증의16파일 SHA256은 문서 작업 시작 시 실제 파일과 일치했다.

이 기록은 부모의 관측을 정리한 것이며 문서 writer의 신규 실행으로 집계하지 않는다. **전체 정본 E2E는645/645(dist557/source88), 약11분 통과**했고 skip/flaky/전역 오류는0이다. 등록 inventory와 실제 단일 passed 결과·물리 case/file 교집합0을 JSON으로 대조했으며 동결 소스16·빌드 입력·실제 제공 index/router 해시가 유지됐다. 독립 코드/UI 리뷰와 문서 내용 리뷰도 각 동결 해시에서 actionable 문제 없이 수용했다. focused29·진단 RED·반복 실행은 전체645에 다시 더하지 않는다. TODO/지도·Graphify 생성/신선도 검사와 별도 Node 회귀7개도 통과했다. Node·focused 결과는 앱 E2E645에 합산하지 않는다. Graphify의 기존 metadata/fixture zero-node6 경고와 대형 HTML 시각화 생략은 Rust warning0와 별개다.

구현 `6f38a79559717ad81f4c62dcc538c1a7c336afda`를 현재 브랜치에 정상 커밋·푸시했고 검증한28경로의 staged/committed blob과 원격 SHA·clean 상태를 대조했다. 정확 구현 SHA의 Release Gate `42388449522`와 Dependency Security `42388449513`은 모두 성공이다. E2E `128450732239`, frontend `128450732414`, Toss 규격 `128450732578`, Rust `128450732601` 및 npm/cargo 감사 job의 성공도 확인했다. 원격 E2E 실행 step은 성공했지만 failure-only 보고 업로드는 skipped·artifact0이므로 원격 정확 case 수는 미확정이며 로컬645와 혼동하지 않는다. P1-SESSION-01은 이 코드/mock·구현 SHA CI 범위로 완료 이관했고, 실제3개월 원본/기간·전략 검증은 SOXQ-P1-05·SOXQ-P2-12에 유지한다.

계획 달력2023–2028+알려진 Carter 예외는 모든 특별휴장 완전성이나 실제 원본/offset·발행 지연·정규장 전용 일봉 OHLC·기간 coverage를 인증하지 않는다. 검증 flag는 false, lookAheadSafe=false·sampleOnly/notEvaluable를 유지한다. 실제 native/live·계좌·운영 DB는 미검증이며 P1 원장 모드/legacy/server ABA 및 DATA/SECTOR 후속은 별도 미완료다. 실자료·후속 조건은 [TODO의 SOXQ-P1-05·SOXQ-P2-12·P1-DATA-01 등 기존 ID](todo.md)에서 유지한다.

<a id="p1-scope-01"></a>
## P1-SCOPE-01 — 단일 활성 계좌 격리 보강 (2026-11-07)

기준 `779f52f48bbac8d3ec7b530bcbe5755b51fde42e` 위의 단계적 보강이며 구현 커밋은 이 절을 추가한 Git 이력으로 추적한다. 기존 holdings ticket/epoch·exact owner·원 client 예약/pending 후처리를 보존하고 KIS→Toss→KIS 늦은 ticket, 부분체결→risk 저장 실패→재시작→원 scope 누적 재시도/원가·손익 멱등 회귀를 보강했다. Toss는 공통 parse/publication 합성 fixture이며 외부 HTTP/auth 성공 증거가 아니다.

독립 리뷰의 P1 두 건을 수정했다. 저장 입력의 `expectedProfileId`·`expectedIsPaper`를 필수화하고 IPC/REST 공통 storage→selection lock 아래 broker/account/profile/mode를 검증한다. 실제 handler를 storage lock 앞에서 대기시킨 뒤 동일 계좌 profile/mode를 바꾸는 거부 회귀와 누락 payload/정상 저장을 검증했다. 기존 원장에는 모드를 입증할 정보가 없어 안전한 명시 이관을 제공할 수 없으므로 **새 `is_paper` 영속화와 legacy 유자금 차단만 철회**했다. 기존 유자금 schema 재시작·소유수량/매도 예약 상한, loopback 수동 매도의 전략소유 제외를 검증했다. 현재 선택으로 모드를 추정하거나 원장을 삭제/초기화하지 않는다. **원장 모드 격리·검증된 이관은 미해결**이며 서버 발급 ABA generation도 없다.

frontend는 AppConfig 성공 공개 횟수를 query key/완료 검증에 포함해 같은 scope 설정 복구도 새 예산/복구 GET 성공 전 금액/editor를 복원하지 않는다. 보이는 설정 재점검은 AppConfig+진단과 금액을 재검증한다. 저장은 onMutate 공개 횟수 캡처와 동일 공개의 후속 GET 성공 뒤에만 성공/초안 초기화를 허용한다. GET 지연은 pending 유지, 실패는 저장 반영 가능성 안내/초안 보존, 이전 공개의 응답은 현재 query를 무효화하지 않는다. 복구는 무폴링·명시 검토를 유지한다. 신규 mock은 exact origin/method/path·예산 broker/account/profile/mode와 저장 횟수를 검사하며 잘못된 endpoint/method/account negative control을 추가했다.

revision 검증: `cargo check --workspace --all-targets --locked`, `cargo test --workspace --lib --locked` **969개 통과·실패0/ignored0**, compiler warning0. focused `budget_save` 2개·`legacy_funded` 2개도 통과했다(전체와 중복 합산하지 않음). 최종 전체 실행은 DB 환경4개를 자식에서 제거했으며 PostgreSQL/TLS 계약5개는 즉시 반환으로 실제 DB 검증이 아니다. `npx tsc --noEmit`, `npm run check:fsd`, envDir:false의 `npx vite build --config .cache/p1-scope-01/offline-vite.config.mts`, fmt check·project-map·TODO render/check(열린14개)·diff check를 통과했다. Graphify freshness는 소스 변경으로 stale 실패했으며 요청대로 산출물을 재생성하지 않았다. 소스 독립 감사에서 추가 blocking 결함은 발견되지 않았고 Serena 도구가 없어 참조를 rg로 대체했다.

부모의 이전 Chromium은 **30개 중29통과/1실패**(같은 scope 설정 복구 후 재조회 공백)였고 revision 통과 증거가 아니다. revision 브라우저1개 시도는 `MachPortRendezvousServer ... Permission denied (1100)`/SIGTRAP으로 시작 실패해 추가 launch를 중단했다. 현재4파일 **40개 --list**는 실행 성공 수가 아니다. 부모 재실행: `./node_modules/.bin/playwright test --config .cache/p1-scope-01/playwright.config.ts tests/e2e/balance-events.spec.ts tests/e2e/uiux-holdings.spec.ts tests/e2e/settings-database.spec.ts tests/e2e/simulation-native.spec.ts`. native는 injected Tauri browser mock이며 Rust IPC E2E가 아니다. 실제 native·계좌·provider API/auth·실자료·운영 DB·주문·시작/재개는 실행하지 않았다. P1-SCOPE-01은 미완료이며 일반 AppShell 운영 query/daemon 중단을 주장하지 않는다. 커밋/푸시·브랜치 변경·stash/reset/checkout/clean은 실행하지 않았다.

부모 재검증에서는 잔여 query-key assertion을 계좌 suffix와 공개 세대 증가 검사로 함께 보정한 뒤 같은4파일을 별도 포트의 headless Chromium에서 **40/40 통과**했다. 설정 복구 뒤 금액 재조회 실패·저장 후 GET 지연/실패·late save ABA·잘못된 endpoint/method/account negative control을 포함하며 실제 native IPC는 아니다. 부모가 fmt check·Cargo check·전체 Rust lib **969/969**(실패0/ignored0)·FSD·TODO/지도·diff 검사와 웹 production build도 실행해 통과했다. Graphify는 LLM 없이 AST 일반 update를 실제 수행해 **8023 nodes/13821 edges**로 갱신하고 freshness 검사를 통과했다. 설정/fixture6개 zero-node 및5000node 시각화 상한 안내는 도구 한계이며 compiler warning과 구분한다. 이 확인 시점에는 수정 후 최종 독립 리뷰가 진행 중이며 원장 모드 격리·검증 이관·서버 발급 ABA generation은 여전히 미해결이다.

마지막 P2 리뷰에서 `invalidateQueries`가 paused 재조회 완료를 보장하지 않는 문제가 추가됐다. 부모가 POST 성공 후 onlineManager를 offline으로 바꾸는 합성 회귀를 실행해 수정 전 `pending` 대신 `success`가 나오는 실패를 재현했고, 현재 공개 세대의 정확한 query를 취소/명시 fetch한 뒤 실제 GET 완료를 기다리도록 수정해 통과했다. 해당 회귀를 포함한 같은4파일은 **41/41**, TypeScript/FSD·TODO/지도·diff·Graphify freshness도 통과했다(AST8023nodes/13822edges). 수정 후 마지막 웹 재빌드는 도구의 lifecycle 스크립트 검사1MiB 한도에 막혀 실행되지 않았으며 앞선 build 통과를 이 마지막 수정의 build 결과로 바꾸지 않는다. 최종 소규모 P2 재리뷰는 추가 actionable 결함 없음으로 끝났으며 원격 CI는 이 시점 대기다. 원래 P1 미완료 경계는 유지한다.

<a id="simulation-plan-20261107"></a>
## 모의투자 후속 계획 반영 — 2026-11-07

기준 코드는 `ef24ccf5fe76d85c54f5788faad5bec0771d632b`, 환경은 맥미니 외장 KISAutoTrade의 기존 작업 브랜치다. 11월6일의 KIS/Toss 차트 한정·미국 정규장 한정 요청과 11월7일 제안 반영 요청을 대조해 TODO의 작업 순서·상태·완료 조건을 갱신했다. 새 구현·설정 변경·실자료 수집·계좌/주문 실행은 없다. 기존 12개 ID는 보존하고 시간/수집 경계 2개를 추가했으며, 상세 잔여 조건과 현재 집계는 [TODO](todo.md)가 소유한다.

P1-SCOPE-01은 당분간 단일 활성 계좌 유지와 전환/모의·실전 격리 검증으로 좁혔다. 원래 **여러 broker/account 포지션과 pending을 동시에 상주·대조하는 multi-scope runtime**은 보류이며 구현 완료가 아니다. 재개 시 broker/시장 조합·계좌별/전체 리스크·정지 범위를 먼저 결정하고 `(broker, account, market, symbol)` tracker·scope별 credential reconciliation을 설계한다. 동시 주문/체결/pending·캐시·소유 원가·재시작·부분 실패의 독립성과 scope별/전체 정지를 실패 fixture/UI에서 검증한다는 기존 완료 조건은 유지한다. 실제 계좌 운용은 별도 승인 범위다.

P2-UX-07은 정지/비상정지 즉시성, 실전 시작·비상정지 해제·매수 재개·수동 주문 확인으로 방향을 확정해 자동 구현으로 이동했다. 11월6일 [현행 영향 조사](#p2-ux-07-action-audit)의 정적 근거는 유지하고 새 정책 구현/검증은 TODO에 남긴다. DB/SQLx·MSRV와 배포 키 준비는 시뮬레이션과 분리하며 서명된 수동 배포 우선 검토·자동 업데이트 별도 단계로 정리했다. 기존 보류를 해제하거나 보안/인증 설정을 바꾸지 않았다.

문서 전용 검증은 TODO renderer **3/3**, 생성 HTML 동기화 **14개**, 상대 링크/앵커 **147개·오류0**, 기존12개 ID·실자료 대기5개 원문 보존, project-map·Graphify freshness·diff check를 통과했다. renderer의 최초2/3 결과는 macOS Chromium의 Mach bootstrap 권한으로 브라우저1개가 시작 전에 실패한 것이며, 승인된 동일 명령 재실행에서3/3 통과했다. npm은 주입된 `http-proxy` 환경 설정의 향후 지원 중단 안내를 출력했고 제품 코드 경고는 아니다. 전용 문서 검사 스크립트를 찾거나 읽을 수 없어 실행하지 못했으며 대체 링크/앵커·보존 검사와 별도의 내용 리뷰를 수행했다. 독립 문서 리뷰의 서명 게이트 유지·과거 집계 시제 권고를 반영했다. 생성 HTML의 모바일 폭·키보드 필터/검색·dark mode·JavaScript 없는 전체 항목 접근까지 확인했으며 거래 코드·Rust/TypeScript 빌드·실제 native/API·CI는 실행하지 않았다. 현재 실행의 GitHub 네트워크 허용 변경은 확인되지 않아 push/원격 CI는 미실행이다. 이 문서 변경의 로컬 커밋은 해당 Git 이력으로 추적한다.

## 이력 사용과 추적

- **최신 전체 기준:** 키보드 최종 코드의 Release Gate58·Security65 성공, UI **557/557(14.0분)**·Rust lib **901/901(12.81초)**·TODO renderer3·bootstrap4. 기존408+키보드149의 고유557이며 focused/진단/재실행을 더하지 않는다. compiler 경고0, Actions/Node deprecation과 기존 RSA 예외는 별도다. [최종 영수증](#keyboard-final-ci-20261031).
- 실제 macOS 한글 IME·Tauri WebView·스크린리더/AT·실계좌·시장자료·운영 DB/배포는 이 결과로 검증 완료하지 않는다. 아래 과거349/408 UI·887 Rust는 당시 기준이다.
- 연구는 실제 자료/합성 가정·생산 실행/독립 재정산·실현/잔여 평가·연속/별도 초기화 계좌를 구분한다. 과거 사후 연구를 독립 OOS나 전략 채택 근거로 승격하지 않는다. 실제 provider·실자료·정책 결정은 TODO에 남긴다.
- 완료 작업은 아래 통합 영수증과 불변 Git 원문, 연구 판단은 기존 research 정본을 따른다. 이력 압축은 구현·수치·설정·거래 정책 변경이 아니다.

<a id="p2-sector-01"></a>
## P2-SECTOR-01 — 섹터 ETF 3전략 구현 가능성 검토 · 2026-11-05

Mac mini 원본 checkout은 clean `dd4e498`/`fix/mac-security-storage-2026-10-06`였고 원격은 `a94e1ab`로 앞서 있어 원본을 보존한 격리 사본에서 fast-forward 후 검토했다. Rust의 종목별 Momentum·단일 symbol preview/ReplaySession·KIS 완료 일봉 page·통화별 live 예산을 읽고, 구현 가능하되 다종목 순위/공통 현금·목표비중·USD 장부가 추가로 필요하다고 판정했다. 기존 max_position_ratio=30%와 guard의 매도 차단도 공격/방어 리밸런싱의 검증 대상으로 남겼다. 공식 KIS dailyprice의 수정주가 입력을 확인했으며 실제 ETF 자료/조정 의미/coverage는 조회하지 않았다. [TODO P2-SECTOR-01](todo.md)에 원장 대조 확정 규칙(RS 백분위/동률·초기 보통·5%p·편도10bp)·30거래일/달력30일·SPY22/섹터21 선행종가와 상태 복원·09:50표시가 대비 nextOpen 근사 차이·인과/기업행위·독립 정산 및 실거래 전 완료 조건을 합쳤다.

최초 회차의 범위는 검토와 문서 반영이었다. 제품 코드/가상 실험/계좌/주문·운영 설정/배포 변경 및 30일 시뮬레이션 실행은 없으며 수익률을 산출하지 않았다. TODO renderer3/3(모바일 검색·필터/전체항목/escaping 포함), TODO freshness12개·project-map·diff check·기존11개 제목/본문 보존 검사와 독립 문서 감사의 상대 링크/앵커133개 검사를 통과했다. renderer의 첫 sandbox 실행은 macOS Chromium 시작 권한으로 브라우저1개가 실패했고 승인된 sandbox 밖 재실행에서3/3 통과했다. stock_analyst의 코드·표본 감사와 별도 documentation pass에 필수 수정 지적이 없었다. 문서 전용이므로 Rust/TS 빌드·전체 거래 테스트·CI는 이번에 실행하지 않았다. 기존 11개 대기 작업은 유지하고 새 구현 과제1개만 추가했으며 커밋은 이 절을 추가한 동일 Git 이력에서 추적한다.


### 후속 구현·합성 검증

사용자의 코드 구현 요청 후 같은 ID를 이어 `sector_portfolio-v1` 순수 USD 엔진과 공통 순위/장전 planner·계좌 독립 연구 UI·IPC/REST·오프라인 example을 구현했다. 초기Normal·history는 지표만 준비, 이전종가 예산→공통관측시가 정수수량·매도후순위매수·10/20bp 총비용·독립5계정·전체입력실패를 명시한다. 모의 원장 요약은 같은 시작일/초기자본/비용과 동일 평가일만 대조하며 기존 experimentStore 축약 경로를 쓰지 않는다. 운영 OrderManager/guard/risk·provider/계좌·파일저장/배포에는 연결하지 않았다. 기업행위 사건목록 또는 평가창 사건/미수 부재 미확인 입력은 전체 거부하며 실제 처리 구현은 TODO에 남겼다.

로컬 전체 Rust **920/920(최종18.83초)**, 전체 브라우저 **561/561(3.6분)**(기존557+신규4), 집중 UI4개(28.7초), TypeScript·Vite config타입·FSD·웹production빌드·lockfiles·fmt·TODO3·bootstrap4·지도·Graphify freshness를 통과했다. 초기 추가 회귀1개는 첫양수종가 당일을 장전에 미리 쓰는 fixture 기대 오류였고 엔진을 바꾸지 않고 혼합쌍 유지/다음날 양수2회복귀를 명시해 통과했다. 서버 내부 작업실패500/입력422를 분리한 후에도 전체920개가 통과했으며 최종 로그는 검증 캐시에 보존한다. 합성 POST 인증/동일origin·대형입력413·IPC/REST동일JSON·부작용0을 검증했다. compiler 경고0이며 기존 NO_COLOR/FORCE_COLOR 안내와 Graphify의6개 설정/fixture zero-node 경고는 도구 관측으로 구분한다. Graphify 실제 재생성은7574nodes/17692edges이며5000node시각화상한으로 오래된graph.html이 제거됐다. node/graph source를 수동 조작하지 않았다.

실제 생산 CLI로 합성22+30봉·비용10/20bp의 **10장부/300일별 기록·36매도/4부분매도**를 만들고 두 독립 감사가 순위/적격·상태·장전예산·rebalance사유·매도선행·정수수량/현금한도/status·원가/비용·실현/미실현·NAV/MDD/최대일손실/표본변동성/노출/회전율을 대사했다. 별도 Decimal40자리 장부의 최대 절대차는3e-12다. 결과 raw SHA256은 기본 `c6686fc144bc4aa7c7865ffaaf663d56b9b4a533626ebb0cf52c3207e799cd14`, 스트레스 `0e76ec742203f5977babeb6418fe2f999563550086a01dea23e6ce81ea0219e6`이다. 원본/정규화hash·입력/결과·부모audit·독립Decimal/sizing감사 영수증은 격리 사본의 Git제외 `.cache/sector-portfolio/`에 보존했다. 다른 체크아웃에서 재현 가능한 단위fixture는 `sector_portfolio/tests.rs`, UI 예제생성기는 `sectorPortfolioInput.ts`에 있다. 실제 시장 수익률을 산출하거나 합성 수익을 실측으로 승격하지 않았다.

독립 코드/전략·구조/문서·UI/UX pass에 필수 잔여 지적이 없다. 실제1440/375px light 합성화면은 `.cache/sector-frontend/`, 전체 UI로그/보고서는 `.cache/sector-regression/`에 있다. 계좌 로그인·자격증명·실주문·자금이동·실제DB·nativeWebView·실제시장자료/09:50체결은 미검증이다. UI keyboard/AT 세부 제외는 TODO에 남겼다. 원본 Mac checkout은 clean dd4e498 그대로 보존했고 main1b6779c/작업73912e9의 공통 코드 기준을 확인한 격리 사본에서 작업했다. 구현 커밋은 `b2211569a8a3d5e6bf3ec22dd03d76e79fefe103`이며 기존 작업 브랜치에 푸시했다. 최초 원격 확인 때 해당 SHA의 workflow_runs/statuses는 비어 있었고 초안PR 생성은 자동 승인 검토의 명시 승인 부족으로 거부됐다. 이후 2026-11-06 사용자가 CI 확인을 명시 승인해 같은 PR 생성만 한 번 재시도했고 [초안PR #16](https://github.com/CountJung/KISAutoTrade/pull/16)을 생성했다. 최초 [Release Gate64](https://github.com/CountJung/KISAutoTrade/actions/runs/42139403940)의 head_sha는 `16d2db140b8f07def755a936dc40760581b3c7ae`이며 frontend/rust/e2e/toss-openapi 실행을 확인했다. 이 승인 이력 문서 반영 후에는 PR의 최종 HEAD에 연결된 검사를 다시 대조한다. 검사별 완료 결과와 정확한 SHA는 PR Checks 및 최종 보고를 정본으로 삼으며, 과거 SHA의 통과를 최종 HEAD의 통과로 바꾸어 기록하지 않는다. PR 승인·병합·main변경·운영배포·계좌/거래 동작은 수행하지 않는다.

### 연구 전용 실행·시세 자료 준비 보강 — 2026-11-06

사용자가 승인한 후속 보강에서 연구 전용 `research-only` native와 별도 identifier/config·운영 router 이전의 동적 연구 화면 진입을 추가했다. AppState/profile/config/store/plugin/server/daemon 없이 파일 검증·3전략 계산만 제공하고 로컬 서명 없는 `.app` 빌드를 운영 설치·배포와 구분한다. 일반 native의 최소 opaque scope 확인과 명시 동의 시세 수집은 별도 경로다. 실제 API 호출을 자동 실행하거나 계좌 연결 진술을 인증 성공 근거로 쓰지 않는다.

자료 경로는 고정12종목의 MODP0 raw/MODP1 adjusted 열과 페이지별 보존필드 hash를 분리하고 순수 검증에서 달력·매핑·조정/기업행위 수동 근거를 요구한다. 공식 chk_dailyprice.py의 MODP0/1만 확인했으며 NYSE Arca→AMS·배당총수익·100봉 보장을 추론하지 않았다. 22사전봉+30평가 거래일·특별휴장·nextOpen 대09:50 차이를 현재 사용법/IPC와 같은 TODO에 반영한다. 이번 단계에서 실제 인증/API·개인 profile/계정 파일에 접근하지 않았으며 실제 시장 성과·운영 연결은 미완료다. 앞 절의 Rust920/UI561 수치는 이전 구현 회차의 검증이며 이 보강의 새 검사 결과를 뜻하지 않는다. `cargo test --workspace --lib sector_ --locked`의 보강 후 집중 검증은32/32(신규 자료13·기존 포트폴리오19, 실행0.18초/컴파일17.91초·경고0) 통과했다. 최초 sandbox에서 localhost fixture bind가 거부된4개는 허용 환경에서 같은 합성 검사를 다시 실행해 통과했고 실제 외부/KIS 호출은0이다. scope ID/client Arc helper와 mock source 범위오류는 검증했지만 실제 AppState를 provider await 중 A→B→A로 전환한 통합 검증은 아니다. 전체 Rust/UI·실제 연구 native 빌드/실행 결과는 이 집중 검사로 대체하지 않는다. docs-canon 첫 문서 pass에서 기존 정본6곳을 갱신했고 상대 링크/앵커186개·TODO12개 동기화·기존11개 제목/본문 보존·AGENTS 최근5개·diff 검사를 통과했다. 스킬의 check_docs.py resource를 읽을 수 없어 전용 검사기는 실행하지 못했으며 이 형식 검사와 별도의 코드/계약 내용 대조를 수행했다.

보강 UI는 집중9개(23.4초, 기존4+연구/자료준비5)와 production preview 연구4개(6.0초)를 통과했다. production 연구 시작에서 자동 계좌/설정/시세 API·localStorage 읽기·WebSocket 시도는0이었다. dev의 Vite 전용 loopback WebSocket 시도는 식별·차단했고 앱 WebSocket과 구분한다. 첫 dev2회는 이를 구분하지 않은 assertion으로 실패했으며 허용된 금융/API 경계를 바꾸지 않고 검사 분류를 고쳤다. `.cache/sector-frontend/verification.log`와 research-mobile/collector-desktop PNG가 근거다. collector-desktop은 mock native 응답의 브라우저 화면이며 실제 native/KIS 성공 증거가 아니다. 초기 non-CI 개발 서버의 전체 실행은484통과/82실패였으며 전체 통과로 보고하지 않는다. Vite 개발 WS와 fixture 준비 상태 문제가 관측됐고 대표 live-clock 검사는 이전 e7e9742의 정적 진입과 새 동적 진입 서버에서 모두 동일하게 실패했다. 이 A/B로 bootstrap 단독 회귀를 입증하지 못했으며82건 모두의 원인을 확정한 것도 아니다. 정적 production preview에서 같은 대표4개 spec은55/55(45.5초) 통과했다(`.cache/sector-frontend/prod-regression-report.json`). 최종 전체 게이트는 CI와 같은 정적 preview+별도 source-harness 구성으로 재실행했으며 아래 최종 결과와 구분한다. CI 구성의 native mock 수집 검사는 별도 `sector-data-native.spec.ts`의 source-harness로 배정했다. 기존 holdings/Toss 검사의 초기 profile 준비 중 KIS placeholder를 조회하던 경합은 제품 코드를 바꾸지 않고 Toss 표식의 표시를 기다리도록 fixture를 보강해 반복 검사했다. 상세 실패 목록·중간 실행을 합산하지 않고 최종 전체 결과와 구분한다.

`cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`가 통과했다. 연구 전용 debug bundle `target/debug/bundle/macos/KISAutoTrade Research.app`을 서명 없이 빌드했고 identifier `com.countjung.kisautotrade.research`를 확인했다. 해당 bundle 실행파일을 직접 시작해 수분간 프로세스 생존을 확인한 뒤 종료한 smoke이며 native 화면·IPC 클릭·실계좌 연결을 검증하지 않았다. 빌드의 `--no-sign` 안내는 서명 생략 관측이며 compiler 경고와 구분한다. 운영 앱 설치·교체·배포는 하지 않았다.

오프라인 생산 Rust CLI에서12종목의 합성22사전봉+30평가봉 bundle→순수 검증→5계정 replay를 연결했고 생성 입력으로 계산한 portfolios가 직접 준비한 기준 입력의 계산 결과와 정확히 같았다. 미확인(false) 선언·빈 확인 근거/누락 가격쌍은 `ready=false/input=null`로 차단했다. `.cache/sector-account-review/sector-data-cli-receipt.json`·`sector-data-*-report.json`·`sector-data-native-build.log`·`sector-data-clippy.log`에 보존했다. 실제 API/인증은0이며 이 검증은 실제 시세 진위·조정 의미·거래소 매핑·기업행위 부재나 투자 성과를 입증하지 않는다.

보강의 최종 로컬 게이트는 **Rust934/934(18.16초)**와 `CI=1 npx playwright test --workers=6`의 **UI566/566(2.7분, retry0/skip0)**로 통과했다. 이전 개발 서버484/82·중간 fixture 구성/준비 경합과 최종 결과를 합산하거나 서로 대체하지 않는다. default/research-only의 all-targets Cargo check, 엄격 Clippy 경고0, 앱/Node config TypeScript와 위 연구 native debug build·프로세스 smoke·오프라인5계정 CLI 동등성도 통과했다. `.cache/sector-account-review/sector-data-rust.log`·`sector-data-ci-mode-e2e.log`와 앞의 검사별 원본이 근거다. 이는 로컬 합성 검증이며 실제 KIS 인증/시장자료·운영 계정/주문/DB·native 화면/IPC 조작·투자 성과 검증을 뜻하지 않는다. 최종 변경 문서의 상대 링크/앵커334개·TODO12개 동기화·기존11개 본문 보존·최근 이력5개·project-map·diff 검사도 통과했다. 최신 변경 SHA의 원격 CI는 커밋·푸시 후 [PR #16 Checks](https://github.com/CountJung/KISAutoTrade/pull/16/checks)에서 별도로 확인한다.

### Simulation 메뉴 통합 — 2026-11-06

사용자가 승인한 메뉴 통합을 `eb0d972` 기반에서 구현했다. 일반 앱 `/simulation`의 자료 준비/세 전략 비교/모의 결과 대조3탭과 데스크톱·모바일 진입을 연결하고 Strategy의 기존 패널은 바로가기로 바꾼다. 공유 UI/model을 widgets로 옮기고 이전 import shim을 남기며 기존 backend API·순수 엔진을 재사용한다. 연구 전용 앱은 선택적인 내부 오프라인 도구로 유지하고 일반 사용에 별도 앱을 요구하지 않는다.

일반 AppShell의 기존 계좌/운영 query와 daemon은 계속된다. 최근 정상·이전·오류/미확인 상태와 별도 운영 제어를 표시하며 메뉴·계산은 시작/정지/주문 변이를 요청하지 않는다. 실제 잔고를 USD 가상자본에 연결하지 않고 profile publication/epoch는 인증 수집 scope/동의/늦은 응답에만 적용해 순수 입력/결과를 보존한다. 모의 대조는 수동 JSON 요약이며 자동 원장 연동이 아니다. 이 회차에서 실제 KIS/계좌·운영 native 앱을 실행하지 않았으며 앞선 연구 전용 앱의 자동API0·Rust934/UI566 수치를 새 통합 검증 결과로 쓰지 않는다.

메뉴 통합의 로컬 최종 게이트는 `CI=1 npx playwright test --workers=6 --output=.cache/simulation-menu/full-ui-results`의 **UI575/575(2.9분, retry0/skip0)**와 **Rust934/934(19.36초)**로 통과했다. 집중 검사는 새production5개(11.6초)·실제 Router/AppShell을 사용하는 브라우저 native모사4개(4.7초)·기존focused9개(6.9초)의18개이며 전체575개와 합산하지 않는다. 실제 OS native IPC·KIS 인증·계좌/잔고·주문·자동매매 시작/정지 호출은 실행하지 않았고 native모사는 `__TAURI_INTERNALS__` 합성 응답이다. 데스크톱/375px 화면의 직접 관측에 필수 잔여 문제가 없었으며 `.cache/simulation-menu/{desktop-workspace,mobile-workspace}.png`가 근거다. 일반 `KISAutoTrade.app`의 서명 없는 로컬 debug bundle 빌드(106.16MiB)는 성공했고 실행·설치·배포하지 않았다. 연구 전용 앱 빌드/프로세스 smoke와도 다른 관측이다.

`cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`가 경고0으로 통과했다(`.cache/simulation-menu/clippy.log`). 앱/Node config TypeScript·FSD·lockfiles·지도와 Graphify force-refresh(7803nodes/18158edges)가 통과했고 신규15파일·기존6shim을 대조했다. 일반 AppShell의 기존 읽기는 유지하되 메뉴·검증·계산에서 금융 변이가 없고 프로필 A→B→A/동일 ID·동일 시각 재게시의 인증 수집 무효화와 USD 입력/결과·수동 모의 초안 보존을 합성 검사했다. KIS 이외 프로필은 인증 수집을 보류하며 파일 계산은 독립적이다. backend API·엔진은 변경하지 않았다. 현재 실행·코드 근거는 `.cache/simulation-menu/{full-ui,rust,native-build,menu-focused,graphify}.log`와 `.cache/simulation-native/results`에 보존한다. 최종 문서/원본 스킬 변경 Markdown12개의 상대 링크/앵커356개·TODO12개 동기화·기존11개 제목/본문 보존·최근 이력5개·project-map·diff 검사도 통과했다. 현재 변경 SHA의 원격 CI는 커밋·푸시 후 [PR #16 Checks](https://github.com/CountJung/KISAutoTrade/pull/16/checks)에서 별도로 확인하며 로컬 통과를 원격 통과로 기록하지 않는다. 실제 시장자료/모의 원장 자동 연동·운영 native 화면/IPC·실계좌 준비 검증은 미완료다.

<a id="p2-sector-01-20d"></a>
### 20거래일 표시자료·신호 진단

코드 `5ca8c4e3e4ed4e46ecb76c861d8773d0987a87f8`에서 2026-10-09 시가 근사~11-05 종가의20거래일과09-09~10-08 사전22거래일을 검토했다. StockAnalysis의 SPY+11개 섹터는 각각50개 공통 날짜(08-27~11-05)를 확보했고 OHLC 범위·가격3000셀·누락0을 검사했다. 사용 구간은22+20회이며 수집50회를 평가50일로 부르지 않는다. 직접 HTTP는403이어서 `.cache/sector-20d/raw/*.web.txt`의 web 도구 추출문서 고정본과 hash를 보관했으며 원 HTTP bytes·페이지별 정확한 조회시각·당시 이용 가능 자료(PIT)는 확인하지 못했다. 개별 가격은 센트 표시값이고 조정 정의도 미확인이다. 근거는 `.cache/sector-20d/evidence/data-quality.json`이다.

`corporateActionsVerified=false` 등 미확인 선언을 유지한 실제 생산 Rust CLI는 `UNVERIFIED_SECTOR_SOURCE`로 exit1·성과 출력0bytes를 반환했다. 확인 flag를 올리거나 gate를 우회하지 않았다. 성과 원장·체결/현금흐름·순손익·MDD·회전율·10/20bp 비용 비교를 계산하지 않았으며0% 성과로 표시하지 않는다. 별도 임시 Cargo/Rust harness는 공개 `rank_sectors`/`plan_sector_portfolio`만 사용해 전일 완료자료의 순위·선정·방어상태를 출력했다. 보유는 비우고 현금은 고정한 가정이므로 목표비중 합계는 실제 투자 노출이 아니다.

표시자료의 선정 결과는 기본형1개2일/2개1일/3개17일, 공격형1개2일/2개18일이다. 방어형은 보통11일·방어9일이며 실행일10-21 각15%,10-29 각30%,11-03 각15%로 전환했다(각각 전일 완료 신호). 첫10-09는XLK 하나로 기본/방어30%·공격47.5% 목표, 마지막11-05는 기본XLP/XLRE/XLV 각30%·방어 같은3개 각15%·공격XLP/XLRE 각47.5% 목표다. 독립 감사는20×11=220순위·60전략신호·1600수치(최대차0)·중복 점수43개와 원표→입력1512가격셀을 대사했다. 사전22회/30회 prefix의20행 및 평가 first1/first10 prefix가 같았다. rawClose 감도에서20일 선정/상태 차이0도 관측했으나 조정 정의나 다른 기간의 동등성을 입증하지 않는다. 첫날XLY의 표시 R20은 정확0이라 원 가격 정밀도의 부호를 알기 전 경계 판정을 확대하지 않는다. `signals.json`·`stock-signal-audit.json`·`raw-adjusted-sensitivity.json`에 보존했다.

공식 [SSGA 배당 XLSX](https://www.ssga.com/library-content/products/fund-data/etfs/us/spdr-etf-historical-distributions.xlsx)는HTTP200/584557bytes·SHA256 `4add18439e505ba6c114bea5d352ac68b1ea5420bc99abdf2e21500dd6f1faa2`를 확인했다. 대상1193개 과거 배당행의 파싱과 독립 XML 감사에서 준비기간12배당락·평가기간 배당락/자본이득 행0을 확인했다. SPY의10-30 지급은09-18 권리에 해당해10-09 신규 빈 보유 장부의 새 현금으로 넣지 않는다. 이 배당 파일은 분할/병합 부재·공급자 가격 조정 정의를 증명하지 않는다. Yahoo는 검색요약 대신 직접 캐시 원문을 비교해11-05 종가733.00은 일치했으나 조정열9일이0.005달러보다 차이 났다(최대0.0078101, 실제 준비창3일 포함). 공급원은 혼합하지 않았고 이를 독립 feed 인증으로 부르지 않는다. `ssga-window-check.json`·`stock-distribution-audit.json`과 원본 파일/영수증이 근거다.

운영 시세/계정 없이 오류 표시를 확인하려던 격리 bare Wry harness는 생산 crate·IPC·keychain·reqwest 없이 incognito·HTTP/WS/file 차단으로 빌드했다. 45초 제한으로1회 실행을 시도했으나 exit134·`sysmon request failed with error: sysmond service not found`로 종료돼 host report/PNG는0개다. native 시도는 권한 상승·재시도 없이 중단했고 native 화면·실제 Tauri IPC·일반 운영 앱은 검증하지 않았다. 브라우저 첫 sandbox 시도도 page 생성 전 Mach bootstrap Permission denied1100/SIGTRAP로 종료됐으나, 보안 검토를 거친 격리 브라우저에서는 공유 production Research 화면을 확인했다. 실제22+20 입력의 flag/내용 보존·기록된 실제 CLI 거절 응답의422 fixture 표시·성과영역/결과 다운로드 없음·preview fixture1회·외부/계좌/금융 요청 시도0·pageerror0·375px 가로 넘침0과 desktop/mobile PNG2개를 확인했다. 이는 오류 UI 렌더 관측이며 브라우저에서 실제 backend나 native IPC를 실행한 검증은 아니다. `.cache/sector-20d/evidence/error-ui-browser-receipt.json`과 `unverified-source-{desktop,mobile}.png`에 보존했다.

최종 캐시 요약은 `.cache/sector-20d/evidence/research-summary.json`(수치·제한·근거파일 hash), 독립 감사는 같은 `evidence/stock-{data,signal,distribution}-audit.json`이다. 원래 P2의 검증된 실제30일 비교는 열린 상태로 유지하고11/6~24 forward 실험의 규칙·성과 원장·schedule은 바꾸지 않았다. 제안은2회 확인 상태·전일/당일09:50 기록·목표/실현 현금·적격 부족 시 현금·비용/센트 경계를 추가로 관찰하는 shadow 범위이며 사후 최적화·실제 주문·방어 효능/우월성 추천으로 승격하지 않는다. 제품 코드는 변경하지 않았다. 문서3곳과 파생 TODO HTML의 링크/앵커145개·TODO12개 동기화·기존11개 제목/본문 보존·project-map·diff 검사도 통과했다.

<a id="p2-sector-01-broker-chart"></a>
### 브로커 차트가격 근사·KIS20일 실측

승인된 후속 구현에서 Simulation의 기본 **검증형**을 유지하고 **차트 가격수익률 근사**를 추가했다. 별도 `BrokerChartInput`과 명시 한계 확인·순수 IPC/보호 REST·오프라인 `broker_chart_price_replay`는 생산 rank/planner/정수주 장부를 공유하며 strict source 확인을 완화하지 않는다. 비수정 요청 종가 신호·평가, 다음 관측시가 가상체결·매도선행·현금한도·10/20bp 총비용을 사용하고 알려진 사건은 전체 거부한다. 빈 `knownCorporateActions`는 사건 부재 증명이 아니며 coverage/corporateActions flag는 false, economicReturnVerified/dividendReturnIncluded도 false다. [계약 정본](ipc-commands.md#브로커-차트-가격수익률-근사)에 스키마·상한·hash와 수집 권한 경계를 둔다.

기존 KIS 연결 profile을 내부 quote collector로 사용해12종목 시세12회를 성공했다. 잔고·계좌목록·주문 호출은0이며 설정/키를 변경하지 않았다. 문서 담당은 민감 profile/자격증명을 읽지 않고 비민감 캐시 영수증만 대조했다. Toss 실제 인증은 실행하지 않았다. 공식 KIS AMS 마스터의2026-11-06 dated snapshot에서 대상12개 US/USD/type3/AMS와 매핑 파일을 독립 확인했다. 이는 조회 코드의 날짜 고정 근거이며 상장시장·최신성·NYSE Arca 변환 규칙 인증이 아니다. 현재 코드는 이 근거의 seed와 전체12개 수동 override를 제공하고 오류/빈 자료에서 다른 코드로 fallback하지 않는다.

실자료는 같은 KIS의 SPY 관측일42개와12종목이며 사전2026-09-09~10-08의22회, 평가10-09 시가 근사~11-05 종가의20회다. USD10000의 독립5장부에 같은 창/가격/정수주/비용을 적용했다. SPY 공통관측일은 공식 달력 완전성·PIT·당시 유니버스·기업행위 부재를 인증하지 않는다. KIS MODP0/Toss adjusted=false는 비수정 **요청**이며 경제적으로 완전한 원시가격·과거 조정 부재를 자동 보증하지 않는다. 배당 현금/재투자·분할 권리·실제09:50 호가/체결·유동성/부분체결/거절·운영 guard는 반영하지 않았다. 기말은 마지막 종가 평가로 최종 청산비용을 추가하지 않는다.

| 장부 | 10bp 가격장부 순수익률 | 10bp MDD | 10bp 비용(USD) | 20bp 가격장부 순수익률 |
|---|---:|---:|---:|---:|
| 기본 | −0.546545% | 2.611848% | 102.296504 | −1.569826% |
| 방어 | −0.672030% | 2.611848% | 86.048980 | −1.568592% |
| 공격 | −4.168766% | 5.402578% | 172.895569 | −5.887444% |
| SPY 단순보유 | −5.281949% | 5.458223% | 9.314880 | −5.375098% |
| 현금 | 0% | 0% | 0 | 0% |

이 표는 배당 포함 총수익이나 미래 전략 효과가 아닌 비용 차감 **차트 가격 장부**다. 기본/방어의 같은10bp MDD와 세 전략의 비용 민감도는 별도 국면·미관측 표본을 요구하며 급락 방어·우월성 추천으로 확대하지 않는다. 회전율은(매수+매도대금)/초기자본, 비용 제거 수익은 같은 체결경로의 비용 가산값으로 비용0 재실행이 아니다. 비용/회전율·20bp 민감도를 별도 shadow에서 관찰하는 정도만 제안하며 기존 전략 파라미터·forward3주 규칙/원장/schedule·주문을 바꾸지 않았다.

독립 감사는10/20bp×5장부×20일의200행·11539수치를 대사해 최대 절대차5e-12, 미래 평가 prefix4개 일치와259가상체결을 확인했다. 원수집 입력의 acknowledgedPriceOnly=false를 보존하고 계산 사본에서만 확인/비용을 바꿨으며 strict flags를 승격하지 않았다. `.cache/broker-chart-live/run-20261106/{stock-summary.csv,stock-daily-ledger.csv,stock-live-audit-receipt.json,stock-source-warnings.json}`이 근거다.10bp 보고서 SHA256은 `8475acc06e3920334296accaeda3ba9eb3640ed96d4fabcb49860b2f3af27a07`,20bp는 `b36a1a8d101cbdffc17edc8a38f79a2eb91af4530d5952968ced87ecae9ca807`이다.100행 페이지의 정규화 rowsHash는 기록됐지만 collection에는 선택42행만 보존돼 독립 페이지hash 재계산은 불가능하며 HTTP 원문hash도 아니다. 개인 캐시의 독립 재현 근거를 외부 공개/게시하지 않았다.

seed/REST 보호 route/Toss raw wire 보강 후 로컬 Rust965/965(16.66초), server29/29와 normal/research all-targets check·엄격 all-features Clippy 경고0·앱/Node TypeScript·연구 웹 build·FSD·lockfile 검사를 통과했다. 전체 UI580개 통과는 마지막 경미 보강 **이전** 실행이며 이후 focused6/6을 확인했다. 예상 전체581을 실행 성공으로 기록하지 않는다. native 수집 집중1개와 native 화면 PNG는 브라우저 mock이며 실제 OS native IPC 화면은 미검증이다. KIS quote/auth·Toss auth의 합성307/308 redirect는 두 번째 서버 수신0을 확인했다. Toss raw wire 회귀16/16은 캐시 토큰 재사용·quote GET1·OAuth POST0·계좌 header0을 검증했으며 실제 Toss 인증 성공이 아니다. Graphify는8014nodes/18661edges/357communities로 재생성했고5000상한 HTML 생략은 도구 안내다. 최종 원격 CI는 커밋·푸시 뒤 [PR #16의 해당 HEAD Checks](https://github.com/CountJung/KISAutoTrade/pull/16/checks)에서 별도로 확인한다. 원래 **검증된 실제30일 비교**·기업행위 정산·운영 연결은 계속 미완료다.

**2026-11-08 원문 출처 운반 후속 — 코드/offline·정본·구현 SHA CI 완료.** 기존 30일 UI·사전22봉·수집기·엔진을 재구현하지 않고 Toss `get_candles_with_source(1d)`의 동일 성공 응답 값/출처를 page→receipt→기존 collection/export에 연결했다. optional `responseSource`는 `sha256`·`byteLength`·UTC `receivedAt`을 담으며 정규화 `rowsHash`와 다르다. KIS는 미제공 None이고 key 생략과 before 고정 JSON/collectionHash를 유지한다. Some은 전체 receipts projection을 통해 collectionHash에 반영되지만 경제 입력·rowsHash·계산 결과는 바꾸지 않는다. 스키마·원문 표현 경계는 [IPC 정본](ipc-commands.md#브로커-차트-가격수익률-근사)이 소유한다. 원문 bytes 자체를 collection에 복제한 것이 아니며 과거 KIS 실측의 미보존 원문을 소급 복구하지 않는다.

부모는 실제 생산 Toss client를 합성 loopback에 연결해 12종목×2페이지의 서로 다른 raw body를 각 receipt의 종목/페이지 키·고정 SHA/길이·UTC 수신 범위와 대조했다. 두 wire 변형의 48개 SHA/길이를 Python으로 독립 검산했고 두 21봉을 합친 정규화 자료가 기존 42봉 raw fixture와 정확히 같음을 확인했다. 기존 동일 body 반복 검사는 첫 source 재사용 오류 아래 집중31개가 모두 통과하는 공백이 있었다. 강화 후 같은 오류는 새 관통 검사에서 `source binding: SPY page 2` assertion으로 실패했고, provider에서 source만 None으로 버린 오류도 메타데이터 존재 검사에서 실패했다. 둘 다 컴파일/설정/권한 오류가 아닌 **post-implementation negative control**이며 실제 제품 결함이나 원래 TDD RED의 증거로 부르지 않는다. 각 오류를 제거해 검토된 전체 소스 hash를 복구한 후 최종 전체 Rust lib **1038/1038**, normal/research all-targets check·diff 검사가 모두 exit0·경고0이었다. 집중31개는 전체 lib에 중복 가산하지 않는다. 독립 최종 provider 감사와 나머지 제품/TS·문서 감사는 지적 없음으로 수용했다. worker의 loopback 권한 거부는 여전히 NOTRUN이며 부모 성공으로 소급 변경하지 않는다.

UI는 새 source-harness **88/88**이 각1회 통과했고 skip/retry/flaky/global errors는0이었다. 초기70통과/18실패는 별도 실패 기록으로 보존했다. 부모 custom Vite cacheDir가 dependency URL을 바꿔 strict asset guard에 걸린 하네스 불일치였으며 실제 CI의 cache 경로를 복원했을 뿐 앱/fixture/미허용 API·WebSocket 차단을 완화하지 않았다. 이전 DATA의 dist **589개는 새 실행이 아니라 유지한 근거**다. 현재 root CI 등록과 새88+유지589의 고유 case 집합이 정확히 일치해 canonical **677개**, source/dist의 case·파일 교집합0을 확인했다. 관련 test/dependency71파일과 새 production build의13개 assets·입력 hash 동일성으로 유지 범위를 확인했고 집중 재실행·고의 실패는 합산하지 않았다. 이는 offline renderer/합성 native bridge이며 실제 Rust IPC/WebView나 외부 broker 실행 증거가 아니다.

코드/offline 수용은 2026-11-08 15:49 KST다. 기존 소유 정본·파생 HTML/지도/그래프의 갱신·내용 검토와 정확한17파일 정상 커밋/푸시를 마쳤다. 구현 `e991f5589e64582ca650df2f55b7516bce2a7dec`의 원격 SHA 일치와 Release Gate `42458130674`·Dependency Security `42458130749` 성공을 확인했다. Rust/frontend/E2E/Toss 규격·cargo-audit/npm-audit의 실제 job 및 필수 실행 step 성공이며 원격 개별 case 수·skip/flaky를 인증하거나 로컬677개를 원격 통과 수로 복사하지 않는다. 이번 작업에서 계좌/자격증명·실제 API·운영 앱/DB·주문·자동매매·배포에 접근하지 않았다. 실제 Toss 인증, provider 발행/PIT·정규장 전용 일봉 OHLC·기업행위/조정·30일/3개월 coverage·native·실제DB/계좌·원장 mode/legacy/server ABA를 검증한 것이 아니며 원래 실제30일·기업행위 정산·운영 연결의 열린 조건을 유지한다.

<a id="p2-sector-01-local-followup"></a>
### P2-SECTOR-01 로컬 후속 — 수집30봉·키보드 관측 · 2026-11-06

외장 원본은 clean `e7e9742`/`fix/mac-security-storage-2026-10-06`이며 원격보다4커밋 뒤였다. 원본을 바꾸지 않고 현재 작업 폴더의 격리 사본을 승인된 같은 원격 브랜치 `50c65f2`까지 fast-forward했다. TODO12개를 코드와 대조해 독립 로컬 진행이 가능한 P2-SECTOR-01을 먼저 보강했다. DB/MSRV·scope·확인정책·서명 결정과 나머지 실제 장중자료/운영검증의 보류는 유지한다.

차트가격 근사 UI의 고정20봉 요청을20/30거래일 선택(기본30)+사전22봉으로 바꿨다. 기간 변경은 요청 revision을 올리고 동의를 해제해30→20→30 뒤 이전 응답도 버린다. 이미 입력한 독립 장부는 옵션만으로 지우지 않는다. 합성 native 브리지에서 정확한 요청/늦은응답 폐기·재동의·22+30 입력과 원본 수집JSON 내보내기를 확인했다. 실제 native 프로세스/IPC·시세 인증이 아닌 브라우저 합성이다.

집중 Playwright13개가 통과했고 내보내기 기대 추가 후 native 합성2개도 다시 통과했다(재실행을 고유 검사 수에 더하지 않음).375px light/dark에서 성과·계획보유·순위의3개 표 실제 방향키 scrollLeft/마지막열 geometry와 상세전략 Home/End/Enter/Escape를 관측했다. 외부/미허용/실제금융 요청·pageerror0이며 native fixture는 운영 shell 없이 실행했다. PNG와 원본은 Git제외 `.cache/sector-frontend/{broker-chart-30-mobile,keyboard-results-light-mobile,keyboard-results-dark-mobile}.png`에 있다. 파일선택키보드·모든select·확대·스크린리더·실제WebView는 미검증이다.

Rust `cargo check --workspace --all-targets --locked --offline`, 전체lib965/965(16.34초), 앱/Node TypeScript·환경파일을 읽지 않는 Vite 웹빌드·FSD·lockfile·project-map·TODO HTML 동기화를 확인했다. 컴파일러 경고0이다. 처음 sandbox의 loopback 금지로 실패한 Rust/브라우저는 제한된 환경과 별도 승인된 실행에서 통과했으며 실제 provider/자격증명은 사용하지 않았다. Graphify 실제 update/check를 수행했으며 기존 zero-node 설정/fixture6개·5000노드 HTML 생략 안내는 코드 경고와 구분한다.

내보낸22+30 합성 입력의 독립 사본에서만 가격근사 확인을true로 해 생산 `broker_chart_price_replay` CLI를 실행했다. source.synthetic=true 및 검증flag=false를 유지한5장부/150일별 행의 데이터흐름 관측이며 합성 수익을 실제 성과로 보고하지 않는다. 원수집본의 한계확인false는 보존했다. 독립 감사는52공통일·12종목·5장부/150행·가상체결9개의10bp 비용을 확인했으며 전체 금융 모델 재구현 대사는 아니다. 입력 raw SHA256 `2902c7672ce6687c2056243a8544a3fa0e87e185691cf143632469e481b04439`, 보고서 raw SHA256 `ec26f34bc4b545c7408fae4abbceafe343a6a98e788e67f8b4330e0006303ffe`이며 `.cache/sector-frontend/broker-chart-30-replay-{input,report}.json`과 실행log가 근거다. 한국투자·토스 차트범위와 미국 정규장만의 모의 의도를 유지하며, 일봉 DTO의 관측일/달력값은 정규장·시간외 포함범위의 인증이 아니므로 이를 미검증으로 남긴다. 실제30일 검증·기업행위 정산·운영연결은 계속 열려 있다. read-only 독립 코드/문서 감사에 차단할 회귀는 없었다. 배포·실계좌조회·주문·자금이동·운영설정 변경은 없다.

후속 순차 pass에서 파일선택기3곳(검증형·자료준비·차트가격)의 실제 focus→Enter/Space→filechooser 이벤트→같은 이름/내용 파일 재선택과 입력 반영·계산 요청0을 확인했다.1280×900의200% CSS zoom에서는 비교모드 Home/End/Enter/Escape·비용/상세전략 선택·한계확인/계산 버튼을 실제 키보드로 조작하고 선택 영역/문서 가로 넘침0을 확인했다. 렌더 확대이며 실제 OS 파일 대화상자 외형·브라우저 도구창 확대·스크린리더/WebView를 검증한 것은 아니다. `.cache/sector-frontend/price-results-css-zoom-200.png`가 production preview의 관측 PNG다.

이 후속은 제품 소스 변경 없이 회귀검사·문서만 보강했다. 관련 dev16/16(18.7초), 동일 제품 소스의 빌드 산출물 production14/14(9.9초)가 통과했고 두 환경 검사를 고유30개로 합산하지 않는다. native 브리지2개는 dev전용이며14개와 별도 플랫폼 인증이 아니다. TypeScript·FSD·lockfile·project-map·TODO 동기화·Graphify 실제 update/check·diff 검사와 독립 리뷰를 통과했다. Rust965와 웹빌드는 앞의 같은 제품 소스 결과이며 후속에서 재실행한 것으로 기록하지 않는다. 갱신한 TODO HTML의 기존 renderer3개도 실제 Chromium의375px 검색/상태필터·JavaScript 없는12개 읽기를 포함해 통과했다.

선행 코드 커밋 **`dbf45a409b2a44b72d366676bb0d47b2ee726b74`**를 지정해 원격 CI를 확인했다. [Release Gate70](https://github.com/CountJung/KISAutoTrade/actions/runs/42208552559)의 frontend·rust(normal/research check와 lib test)·e2e·toss-openapi와 [Dependency Security76](https://github.com/CountJung/KISAutoTrade/actions/runs/42208552487)의 cargo-audit·npm-audit가 모두 성공했다. E2E job127861200239의 실제 로그는 **584/584(18.5분)**이며2026-11-06T11:32:17Z 완료를 확인했다. 후속 푸시로 이 실행을 취소하지 않고 결과를 먼저 보존했다. 이 영수증은 지정한 선행 커밋의 결과이며 후속 테스트/문서 커밋의 CI 성공을 뜻하지 않는다. 기존 보안 예외·도구 안내와 실제 운영 검증의 미완료 경계도 유지한다.

<a id="p2-ux-07-action-audit"></a>
## P2-UX-07 — 확인 정책 결정 전 현행 영향 조사 · 2026-11-06

TODO에 허용된 결정 전 준비를 진행해 [아키텍처의8행 영향표](ARCHITECTURE.md#p2-ux-07-action-matrix)에 Sidebar/Dashboard 시작·정지, 비상정지 발동/해제, 매수 재개, KIS 국내/미국·Toss 수동주문의 진입점/현재 확인/pending과 backend scope/risk/예약을 코드로 추적했다. Serena가 노출되지 않아 rg와 파일 읽기로 참조를 확인했으며 독립 UI 전문 감사와 주문 준비/예약·execution run 종료를 대조했다. 실제 행동·계좌 조회·자격증명 사용은 없다.

정지와 기접수 주문 취소/청산, 비상정지 해제와 daemon 시작/손실 카운터 초기화, 매수 재개와 입금/잔고 재확인의 차이를 구분했다. 최종 확인창이 없는 현행 UI와 backend guard를 구분하고, UI pending이 컴포넌트별이라는 한계와 수동주문에 자동예산 예약이 일괄 적용되지 않는 분기를 기록했다. 새 확인 정책·단축키·금융 권한은 바꾸지 않았다.

당시 이 표는 현행 정적 조사 완료였고 P2-UX-07 상위는 사용자 판단 대기였다. 11월7일의 후속 결정은 [계획 반영 이력](#simulation-plan-20261107)을 따른다. 행동별 확인 수준·scope/모드/영향 요약·입력 변경 시 무효화·모의/실전 차이 결정 후 취소0/승인1/pending 중복0/범위 변경 무효화 검사를 진행한다. 당시 나머지11개 제목/차단 조건도 보존했고 상위 TODO는12개였다.

<a id="toss-inventory-20261105"></a>
## Toss REST inventory 보정 — 2026-11-05

PR #15의 Release Gate 41941404293에서 공식40개/기대39개 차이를 보고받아 개발 HEAD `dd4e4989ca4f5dd40cdb7a55d05f4e9fb2b40175`의 검증 목록에 `/api/v1/transactions` 한 줄을 추가했다. 공식 공개 v1.2.28 문서의 GET·OAuth2·계좌 헤더 계약을 재확인하고 [정본](toss-openapi.md#2026-11-05-공개-rest-inventory-보정)·원본 스킬을 동기화했다. 기존 목록·검사·런타임 거래 코드는 유지한다.

합성22개 통과: 기존39개 순서·목록 보존 및 한 줄 외 동일, 원래39개 성공/40개 실패와 수정40개 성공/39개 실패, 추가·동수 대체·제목·서버·버전·계좌 헤더·rate-limit3종·오류 DTO·HTTP/JSON/전송 실패를 대조했다. syntax·diff check, 지도·TODO·Graphify freshness와 문서 링크/앵커95개를 통과했고 독립 코드/문서 감사의 필수 지적은 없다. TODO renderer는2개 통과/브라우저1개 환경 차단이다(벤더 다운로드403, 시스템 Chromium의 file URL 관리자 차단). 로컬 `npm run verify:toss-openapi`는 네트워크 정책의 CONNECT403/fetch failed로 차단돼 공식 JSON 원문 SHA256·로컬 실제 명세 대조는 미완료다. 같은 명령의 새 HEAD GitHub CI 결과는 병합 담당자가 확인한다.

Graphify0.9.26 정상 재생성에서 나온 전역 ID·edge·community 변경은 최종 tree에서 제외했다. 원래7403nodes/16457edges·ID·community·HTML·report를 보존하고, 실제 생성 결과 중 검증기1개에 속한 source location12곳·manifest1개·source hash1개만 반영했다. 이 파일의8개 심볼과7개 관계는 줄 위치 외 동일함을 대조했으며, 해시를 수동 계산해 검사를 우회하지 않았다. 단일파일 도구 갱신의7402node shrink guard는 강제하지 않았고, 전체 재생성의 zero-node 설정/fixture6개 경고는 도구 관측으로만 남긴다. 검증 원본과 전체 생성 결과는 Git 제외 `.cache/toss-inventory-20261105/`에 보관한다. 계좌·주문 API, 앱 실행·배포·병합은 수행하지 않았다.

## 키보드 개선 완료 — 2026-10-31

[원래 완료 조건·관측 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L13-L112). 아래 줄 범위로 각 ID를 찾을 수 있다.

공통 Rust/앱·검사 타입/FSD/격리 production build·독립 UI/코드 리뷰를 통과했다. 읽기 fixture의 쓰기/외부/unknown/pageerror0과 기존 주문폼의 브라우저 mock 제출을 구분하며 실제 주문은0이다.

<a id="p2-kbd-01"></a>
### P2-KBD-01

2026-10-31 Trading/Strategy/LTH의 KIS·Toss KR/US 입력에 비조합 Enter 조회와 IME/229 방어를 연결했다. 검색 버튼 이름·조회 중 설명을 보완하고, Strategy 이름 조회를 Escape로 닫은 뒤 같은 입력의 Enter 재개방이 유지되게 교정했다. 최종 신규14+기존 선택10/주문폼5의 고유29개가 production mock에서 통과했다(18.2초). 신규14개는 읽기, 기존 주문폼5개는 mock 제출이다. 실제 OS IME·WebView·스크린리더는 미검증이다. 중복/늦은 응답은 후속 P2-KBD-02에서 처리했다.

[회차 기록](#p2-kbd-01) · 원문 L13–31

<a id="p2-kbd-02"></a>
### P2-KBD-02

2026-10-31 US 조회에 입력·시장·선택·공개 broker/profile/account/모드의 ticket/epoch 보호를 연결해 반복 요청과 늦은 결과·오류·후속 fallback·이전 finally를 차단했다. 검색 중 명명 버튼과 focus를 유지하고 기존 KIS 거래소 순서·Toss 동기 선택·Strategy safety·LTH 직접선택 fallback을 보존했다. 최종71개가 통과했다(32.3초): 신규42개(production34/source8)+인접29개다. 신규42개는 읽기, 기존 주문폼5개는 mock 제출이다. 실제 backend 요청 취소·계좌 pinning·운영 검증을 완료한 것은 아니다.

[회차 기록](#p2-kbd-02) · 원문 L33–51

<a id="p2-kbd-03"></a>
### P2-KBD-03

2026-10-31 완료. LTH 국내 결과를 명명된 native 선택 버튼으로 연결하고 내부 Tab/Shift+Tab 유지·Escape/선택 뒤 입력 복귀·활성전략 잠금을 보강했다. 세 국내 결과 목록만 ArrowUp/Down·Home/End focus 이동에 opt-in한다. 조회/모델·미국 경로·대상추가/저장은 유지했다. red4와 별도 Trading 포인터 복귀 결함을 재현·교정하고 최종109/109를 통과했다.

[회차 기록](#p2-kbd-03) · 원문 L53–71

<a id="p2-kbd-04"></a>
### P2-KBD-04

2026-10-31 완료. Settings 수치11쌍의 정적 이름·단위/범위/step 설명과 안정 ID를 실제 Slider/textbox에 연결했다. IME Enter/229는 기존 blur commit을 막고 일반 Enter·Tab/포인터·native Slider·즉시5/Risk 명시저장6쌍의 기존 정책을 유지했다. red3→고정 최소green3→통합151 통과를 확인했다. 증거 파일명 충돌 교정 뒤 신규31개를 별도 재검증해 고유31개 기록을 보존했다. 실제 운영 설정·주문은 호출하지 않았다.

[회차 기록](#p2-kbd-04) · 원문 L73–91

<a id="p2-kbd-05"></a>
### P2-KBD-05

2026-10-31 구현 뒤 최종 원격 CI까지 완료. 프로필 Add/Edit/Delete의 현재 세션에만 main/aux 응답을 적용하고 main 처리 중 Cancel/Escape/backdrop·반복 활성화를 동기 guard로 일치시켰다. 기존 계좌 hook publication/reconcile·payload·동의/확인/활성 경고·aux 중 닫기 허용을 유지했다. 삭제 오류/명시재시도와 Dialog exit·늦은 목록 제거 뒤 조건부 focus 복귀를 보강했다. 제품 red6→최소green6→canonical24→최종전체UI557 및 Rust901·bootstrap4를 통과했다.

[회차 기록](#p2-kbd-05) · 원문 L93–111

<a id="toss-parse-20261031"></a>
## TOSS-PARSE-01 — 시뮬레이션 응답 협상·파싱 진단

2026-10-31 공개 명세·합성 wire에서 헤더 생략→raw gzip UTF-8 실패, 명시 identity→정상 캔들을 대조해 Toss 전용 협상을 교정했다. 캔들 오류는 안전한 단계/status/위치만 남기며 잘못된 본문·미지원 coding을 빈자료 성공으로 바꾸지 않는다. 실제 인증 응답의 gzip이나 사용자가 기억한 장애 원인은 미확정이다. 직접 client/DTO와 생산 adapter의 시장/scope 검증을 구분한다. [작업 기록](#task-toss-simulation-response-parsing-20261031) · [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L113-L117).

<a id="kbd-audit-20261031"></a>
## 2026-10-31 키보드 UX 후속 검토

감사 기준 `74c303a901040d2922b2c41ea58cbd8c798d2c3d`, 당시 clean. 기능·역할·추적 테스트 변경 없이 전문가 소스 검토와 root의 격리 mock 관측을 수행해 P2-KBD-01~05를 제안했다. **제안은 이후 위 절에서 완료됐으며 당시 ‘구현 대기’ 표현을 현재 상태로 읽지 않는다.**

07:22~07:27 UTC, fresh Chromium1280×720/light·production build·envDir:false·환경 allowlist·합성 KIS 모의 fixture, loopback GET만 허용하고 외부/WS·API 변이를 차단했다. 최초6개는5관측+LTH 기본US를KR로 가정한 준비실패1(9.2초); KR전환 명시 LTH1+Settings1의 후속2개(3.5초)로 **고유7관측**을 확보했다. 단일7/7 기능검증이 아니며 완료 관측 쓰기/외부/unknown/pageerror0이다.

- Trading 국내 이름 Enter 전후600ms GET0→0, 버튼 GET1·직접6자리 선택 정상. 미국 일반Enter는 이미 지원하며 선택 후 GET2는 검색+정상 price hook이다. 합성 composing Enter(`isTrusted=false/isComposing=true/keyCode229`)는 각 GET+1, 응답 보류 Enter3회는 각 GET3이었다. 600ms 관측을 영구 미발생이나 실제 OS IME 재현으로 부르지 않는다.
- LTH KR 결과 focus 자식0·row tabIndex 없음, Tab→검색버튼/250ms 후 닫힘·Escape 후 목록 유지. Settings 수치11쌍/22컨트롤 ARIA 이름이 없었으며 값/focus/blur/저장은 조작하지 않았다. 프로필 pending/닫기 경합은 당시 소스 근거였다.
- 이미 지원하던 상단 결과/보유 native 선택, Settings15목차·Select, DB/예산/복구 busy/승인 guard, History 명시 조회/상세, Log retry/리사이저, 차트 표·MUI 메뉴·명시 주문을 미구현으로 재분류하지 않았다. 전역 Enter 주문/저장은 추가하지 않으며 P2-UX-07 판단은 별도다.

원본은 `.cache/keyboard-audit-20261031/first-run-32193/`와 `followup-evidence/`, `followup-test-results/`, `followup-run.log`; 전문가 보고는 `/tmp/kis-keyboard-ux-audit-expert-20261031.md`. 실제 IME·AT·터치·운영 프로필/계좌·주문 미실행, 당시408UI/887Rust를 재실행으로 재사용하지 않았다. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L118-L147).

<a id="p1-recovery-01"></a>
## P1-RECOVERY-01 — 운영자 대조·복구 UI

2026-10-29 desktop IPC/UI·제출 전 원래 주문 snapshot·승인 receipt→기존 pending 복원·원래 시각 멱등 TradeGuard를 완료했다. 승인 자체는 재주문·체결 합성·예약 해제가 아니다. Rust807/복구focused19/제출15·fmt/check/strict clippy 경고0, TS/FSD/build와 production 복구11+Settings4=15 mock UI를 통과했다. 중복 provider ID·재시작/재조회 재시도·유실 primary·손절 guard 누락을 수정한 뒤 독립 필수 지적 없음.

legacy 소유 근거 또는 부분체결 pending intent가 없으면 보류한다. 실제 provider·운영자 원본·보관기간 밖 이력·실계좌/주문은 P1-PROVIDER-01에 남는다. [회차](#task-immediate-backlog-20261029), [기존 소유·체결](#task-strategy-fill-ownership-20261018), [예산 구조](ARCHITECTURE.md) · [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L148-L157).

## 기존 기록·운영 참고 색인

아래는 기존 문서의 진입 링크다. 기록의 원래 날짜·검증 범위와 미완료 조건은 각 문서를 따른다.

| 역할 | 기존 정본 |
|---|---|
| 과거 전체 설계 아카이브 | [MasterPlan.md](#legacy-master-plan) |
| 저장·보안 인계 기록 | [2026-10-06 인계](#security-storage-handoff) |
| 초기 프로젝트 점검·SOXQ P1-01 기록 | [프로젝트 점검](#task-project-health-and-soxq-p1-01) |
| 자동 진행 백로그 기록 | [2026-10-18 작업](#task-autonomous-backlog-20261018) |
| 문서 배치 변경 기록 | [2026-10-18 배치](#task-documentation-layout-20261018) · [2026-10-20 루트 정리](#task-root-documents-20261020) |
| 현재 에이전트 도구·코딩 규칙 | [도구 운영](agent-tooling.md) · [코딩 가이드](coding-guide.md) |
| 현재 LTH 파라미터 안내 | [파라미터 가이드](leveraged-trend-hold-parameter-guide.md) |
| 별도 승인 범위의 운영 검증 절차 | [모의 검증 체크리스트](mock-trading-e2e-checklist.md) · [Toss read-only·소액 검증](toss-readonly-small-order-checklist.md) |

## 문서 재편 당시 열린 항목의 ID 대응

P1-DB-01=MariaDB matrix, P1-SCOPE-01=multi-scope runtime, P1-PROVIDER-01=실제 원장·정정/취소 대조, P3-SIGN-01=서명 키 준비는 현재 TODO에 남는다. P1-RECOVERY-01과 P3-REFACTOR-01은 2026-10-29 로컬 범위를 완료했다. “LTH 일봉 성과 미리보기”는 일봉 평가불가/분봉 자료 조건이 같은 SOXQ-P1-05에 통합했으며 SOXQ-P2-12 옵션 비교는 별도다. P1-DB-01과 DEP-P3-01은 정책을 공유해도 완료 조건이 달라 합치지 않는다. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L173-L185).

## 과거 연구의 공통 기준

옛 축약 ID P1-03·P1-04와 P2-01·P2-02·P2-03·P2-04·P2-06·P2-07·P2-08·P2-09·P2-10·P2-11·P2-13은 아래 SOXQ 접두 ID의 같은 작업을 뜻한다.

2026-10-18 등록13종 중 일봉12종1,224회·LTH 분봉4진단. SOXQ/SPY/QQQ의2023Q1~2026Q3 완전45창, 전일 기준15% 목표수량·수수료/슬리피지 각10/20bp·공통guard·기본파라미터(PC는 전일종가/익절5/손절3)를 고정했다. 12전략 모두 동일수량 보유 대비 초과수익 중앙값≤0으로 우수전략을 확정하지 않았다. 부분Q4/최근248 겹침창은 주집계 제외, 분기 초기화·상관 ETF·작은 완료표본·sameClose·고정FX/무배당 한계를 보존한다. [보고/결과](research/strategy-review-20261018/report.md) · [실행](research/strategy-review-20261018/results.jsonl) · [요약](research/strategy-review-20261018/summary.json).

후속 정확성(SOXQ-P2-05/P2-16), 완료일봉(P2-05/P2-17), 체결/회계(P2-14/P2-15), 연속장부(P2-18)와 전략별 교정을 완료했다. 당시 권장 순서는 현재 작업 순서가 아니다. v12의 nextOpen·2023~2025 각752봉·최초수량/PC기준가 고정3년 장부와12개 독립 분기계좌는45창 통계와 합산하지 않는다. [v8 교정](research/autonomous-corrections-20261018/report.md), [연속 연구](research/continuous-replay-20261018/report.md) · [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L186-L230).

## SOXQ 3개월 연구의 P1 정확성

기준2026-07-06~10-02·64거래일+별도252사전봉, 당시 runtime 수정 없이 생산 엔진을 실행했다. [보고](research/soxq-3m-20261003/report.md) · [차트](research/soxq-3m-20261003/charts.html) · [결과](research/soxq-3m-20261003/results.json).

| ID | 최종 변화·검증·한계 |
|---|---|
| SOXQ-P1-01 | 평가 기간/사전자료/준비 상태 분리, 누락initializer6개·이전MA/RSI 복원·200/252봉 유지. live 신고가 최신봉 제외·재초기화 후 보유 손절, 자료부족/준비/조건불충족/무체결 구분. [v4](research/soxq-3m-20261003/report.md#validation-p1-01) |
| SOXQ-P1-02 | 기본 warmup 종가/52주신고가 고가 분리·호출1회·첫신호/미래봉/분봉혼입/보유 회귀. 동일입력 평균회귀+1.4402→+2.2083%, 완료2→1. [검증](research/soxq-3m-20261003/report.md#validation-p1-02) |
| SOXQ-P1-03 | 시가-only→완성OHLC 종가판단→실제체결/차단 피드백→완료봉 적재. StrongClose 전일조건·Volatility 당일/이전N범위·0범위포함·날짜전환 보유유지, 일봉 전용. 장중 실제체결 아님. [v5](research/soxq-3m-20261003/report.md#validation-p1-03) |
| SOXQ-P1-04 | generic LTH는 factory 전 UNSUPPORTED_GENERIC_REPLAY, 전용 입력시각 경로 유지. 서로 다른 시스템 분/시간대의 지원24실행·거부·일봉진단 일치·미래불변. 고정세션 시장정확성은 P1-05에 남음. [v6](research/soxq-3m-20261003/report.md#validation-p1-04) |

[이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L207-L220).

## 주문·상태·저장 경계의 완료 기록

**P1-RUN-01:** 실행 epoch/config revision·provider 고정과 예약, 수동/체결 대조의 계좌·모드 보호를 구현했다. [기록](#task-execution-run-reservation-20261018) · [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L201-L205).

### P1-CONFIG-01

- **P1-CONFIG-01 — 일반 전략의 설정 저장 중 보유 보호 상태를 보존했다.**
  - 13전략의 같은 ID/scope 재설정과 저장 후 공개, 저장 실패·불명확 저장 원본 복구/복구 실패 차단, params/targets 변경·빈 프로파일·held stop/trailing·취소·신호/주문 경합을 검증했다. 실제 provider·정상 fill 소유권 피드백까지 완료한 것은 아니다. [구현·검증](#task-strategy-configuration-preservation-20261018).
  - 보유 또는 private 보호 중인 대상 제거/시장 변경을 거부하고, 다른 전략/계좌의 보유를 새 대상에 옮기거나 공유 tracker의 0수량으로 기존 보호를 지우지 않는다.

[이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L245-L248)

### P1-FILL-01

- **P1-FILL-01 — 정상·부분 체결과 거부/취소를 주문 소유 전략에 반영했다.**
  - 계좌 snapshot과 exact owner(profile/account/모드/전략/시장) 원장을 분리하고 소유 수량·원가·LTH 최고가/관측을 보존한다. live 신호로 확정 보유를 바꾸지 않으며 자동매도/수동매도 가능 수량을 예약 직전 다시 검증한다.
  - provider snapshot 선반영 뒤 fill 중복 가감 방지, 누적 체결대금301의 잔액 보존, 고정 intent·0수량/증가수량 취소·저장 실패 재시도·원장 세대/재시작·타 owner/프로파일/모드 격리를 검증했다. KIS/Toss 엄격 잔고 변환과 Toss 수수료 비율100배 오류도 수정했다.
  - Rust455개(단위449+통합3+doc3), 브라우저89개, 독립 코드·심볼·helper·문서 리뷰 통과. 실제 운용/도입 전 보유 귀속은 별도 검증이다. [구현·검증과 한계](#task-strategy-fill-ownership-20261018).

[이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L252-L255)

### P1-BAL-01

- **P1-BAL-01 — 일반 잔고 조회의 늦은 응답과 계좌별 캐시 경합을 보호했다.**
  - 기존 게시 14곳을 공통 gate에 연결하고 profile/client/scope/모드·실행 epoch·설정 lease와 tracker별 조회/수량 version을 검증한다. A→B→A, 정지/재시작, 실제 0수량·삭제·두 시장 원자성·실제 partial fill·HTTP barrier를 검증했다. 네트워크 await 동안 전역 lock을 유지하지 않는다.
  - 잔고 이벤트는 현재 scope 재조회 힌트로 사용하고 이전 cache를 제거한다. 늦은 설정 응답·설정 재조회 실패·연속 프로파일 변경·backend 적용 후 mutation 오류도 검증했다. Rust365개·브라우저 신규13개와 기존26개, 독립 리뷰를 통과했다. [구현·검증과 한계](#task-holdings-publication-fence-20261018).

[이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L259-L262)

### TLS verify-full·재접속

- **TLS verify-full 인증서, 서버 강제 단절·재접속 contract를 완료했다.**
  - 임시 CA/localhost 인증서의 PostgreSQL16 로컬 fixture에서 일반 계약3개와 TLS/재접속2개를 실제 실행했다. 신뢰 CA 성공, hostname/CA 불일치·비TLS·오인증 거부, backend socket 종료와 SIGKILL/재기동 뒤 동일 pool 복구, 단절 중 pending/소유/risk 오류 및 JSON/backup/backend 설정 보존을 검증했다.
  - cold pool single-flight·설정/pool snapshot, 파일/keychain 동시 원복·취소/불명확 복구 차단을 보강했다. 릴리스 gate에 같은 fixture를 연결했으며 실제 운영 DB·MariaDB matrix·Windows 실행은 별도 범위다. [구현·검증과 한계](#task-database-tls-reconnect-20261018).

[이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L266-L268)

**P2-05·P2-17:** scope/config/run/시장 날짜를 고정한 bounded 완료일봉 수집·252/512봉 준비·마지막 완료봉 보존, 부족 중 신규매수 차단/보유손절, IPC/REST/UI와 같은scope52주 재설정/rollback 보호를 연결했다. 실제 provider 전수 운용 완료가 아니다. [기록](#task-live-completed-daily-history-20261018) · [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L223-L229).

**2026-09-11 전용예산/LTH:** 계좌·통화별 예산·자동 소유량·durable 예약/체결/terminal 재시작과 Settings/E2E, 기본off 볼린저/중심선·ATR청산·live/replay/preview 설정 연결을 완료했다. 일봉 성과 기능으로 해석하지 않는다. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L368-L375).

## 전략별 연구 완료와 기각 판단

구현·당시 검사는 아래 작업 영수증, 고정 설계·비용/노출별 원장과 해석은 다음 연구 정본이 소유한다. 수익률 표·같은 방법·원문 TODO를 이력에 다시 복제하지 않는다. 당시 선택과 기각 이유는 다음과 같다.

| ID | 남긴 판단·경계 | 연구 정본 |
|---|---|---|
| SOXQ-P2-01 | MA 기본5/20·선택 필터0 유지. 사전 고정 비교와 사후 스트레스 선택을 분리하며 모든 구간의 우수 전략으로 승격하지 않았다. | [MA 교차](research/ma-cross-filters-20261020/report.md) |
| SOXQ-P2-02 | RSI14/30/70·선택0 유지. 확정 소유 피드백을 교정했지만 추세·손절·보유 후보의 구간별 결과를 기본 채택으로 확대하지 않았다. | [RSI](research/rsi-policy-20261022/report.md) |
| SOXQ-P2-03 | 소유를 보정한 level/간격0 유지. crossing/1%p는 연속 장부 추가 효과가 없고 일부 분기 악화로 기본 채택하지 않았다. QQQ2024Q3 개선은 청산 감소·낮은 노출·잔여 평가를 동반했다. | [Momentum](research/momentum-feedback-20261020/report.md) |
| SOXQ-P2-04 | 현재 포함 MA20의 −5/+5 경계와 level·선택0 유지. 실제 소유·미체결을 구분했고 사전 후보를 단독 비교했다. 비대상 경제 동등성과 대상 변화는 별도다. | [Deviation](research/deviation-policy-20261020/report.md) |
| SOXQ-P2-06 | 누적 변화·이전 완료 ATR·장기 MA 확인은 기본0. 단독 필터의 거래 감소·미발동·노출 변화와 손익을 구분해 공통 기본값으로 승격하지 않았다. | [Consecutive](research/consecutive-filters-20261018/report.md) |
| SOXQ-P2-07 | 기존 lookback20/buffer0.5·선택0 유지. raw 후보·완료 거래량의0/결측·실제 소유 보호를 분리했고 통과를 체결로 표현하지 않았다. | [FailedBreakout](research/failed-breakout-policy-20261022/report.md) |
| SOXQ-P2-08 | 이전 완료 고가·종가 거리3%·stop0 원가 이하 의미를 보존했다. D−1조건→D종가 신호→다음 관측 시가와 live 첫 quote 대용을 구분했고 높은 잔여 평가를 동일 노출·청산 성과로 표현하지 않았다. | [StrongClose](research/strong-close-policy-20261022/report.md) |
| SOXQ-P2-10 | upperBand 기본·선택0 유지. 사전63관측의 미발동과 사후21관측 탐색을 분리하고 midBand/추세/보유 후보를 새 OOS나 기본 승격 근거로 삼지 않았다. | [MeanReversion](research/mean-reversion-policy-20261020/report.md) |
| SOXQ-P2-11 | MA200/5/20·선택0 유지. v21→v22 실제 소유 교정과 고정5후보·390원장 독립 대사를 분리하고 후보 미발동/구간 차이·기말 보유로 기본 채택하지 않았다. | [TrendFilter](research/trend-filter-policy-20261023/report.md) |
| SOXQ-P2-12 / SOXQ-P1-05 | UTC/시장 시각·DST·세션·격자 공백 진단만 구현. 기존 fixed KST·선택순서·신호/정산/hash·lookAheadSafe=false를 유지했고 평일창 진단을 실제 달력/coverage 검증으로 보지 않았다. 실제3개월 조건은 TODO에 남는다. | [LTH 근거](research/soxq-3m-20261003/report.md) |
| SOXQ-P2-14 | v10 sameClose/nextOpen·이전 ATR→실제 원가·마지막 미체결과 갭/손절 우선 가정을 분리했다. baseline1224 동등성·합성/실가격 비교는 실제 체결 시각·유동성 검증이 아니다. | [체결 연구](research/replay-execution-20261018/report.md) |
| SOXQ-P2-15 | v11 수수료·FX/배당·실현/잔여·가정 기말청산·실제 자본 노출을 구분했다. baseline1224·원장4540 대사는 가상 가정 정산이며 실제 요율/배당/FX 검증이 아니다. | [회계 연구](research/replay-accounting-20261018/report.md) |
| SOXQ-P2-18 | v12 단일 장부와 청크 상태 보존: 연속72/분할72/별도 분기864를 분리했다. 최초 요청 수량 고정은 동일 실제 노출이 아니고 분기 수익률 합산/복리를 하지 않았다. 청산0·잔여 평가·0주 기준을 우수성으로 표현하지 않았다. | [연속 연구](research/continuous-replay-20261018/report.md) |

세부 수치·기각 과정의 [통합 전 이력 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L280-L365)도 보존한다. 공개 자료의 동결 입력·수량 결정·사전/사후 후보·독립 대사 범위를 연구 정본에서 확인하며, 이미 관측한 과거를 독립 OOS로 부르지 않는다.

## release gate·안내 정합성

broker mock 제출→부분/완전체결→취소/거부→재시작, 잔고fail-closed·scope·자정·manual/auto·미인증REST 거부와 Rust/TS/FSD/UI/OpenAPI를 release gate에 연결했다. audit·Dependabot·lockfile·SHA256/attestation/detached signature 준비를 자동화했으나 실제 플랫폼/updater 서명 강제는 키 준비 후 `REQUIRE_RELEASE_SIGNATURES=true` 조건이다. 가이드는 KIS/Toss 지원표·13전략·프로파일·세션/candle source·소액 gate에 맞췄다. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L376-L398).

## 미완료 상위 항목의 완료된 부분

### SOXQ-P1-05

v7/LTHv3는 bounded 일봉 context와 분봉 EMA/RSI/ADX·반동·볼린저를 분리했다. 분봉OHLC 재초기화는 OHLC/반동/볼린저, 가격 재초기화는 반동만 교체하고 일봉 갱신은 분봉·보유를 유지한다. 일봉은 시가 진입 합성 없이 dailyDiagnostic/notEvaluable이며 성과·A/B저장을 숨긴다(assessment 없는 legacy도 포함). 최대200 실제분봉은 sampleOnly, 유효0개는 notEvaluable. 고정KST·timezone/DST/세션·3개월 대표성 한계는 미완료다. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L401-L408).

### SOXQ-P2-09

v21의 실제수량/원가·잔량·pending/부분/거절/재설정 보호, 이전N완료범위/당일OHLC·0범위·엄격초과·유효시장일/latelegacy, 저장/두preview/준비진단/UI를 완료했다. 비대상1122·기존102창·owned78 호환,36연속/36분할/432분기·부모70828/독립249412정산·390진단을 확인했다(미저장 대체원장 독립정산 아님).

기간20은4악화/2동일, 배수1.5는4개선/2동일이나 모두 완료0·잔여평가. 수익보호/보유63은6개모두하락·회전증가, 분기cap0/72동일, QQQ hold63은10/20bp+0.1099/−0.4609%. 기본10/2/손절3·선택0·guard 유지. 비교보유0주36개·준비완료 무신호32/72를 우월성/자료부족으로 오해하지 않는다. 과거45창 중앙값0%·완료8·무완료38·무신호20은 별도이며 장중효능 미완료. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L409-L417).

### SOXQ-P2-12

실제분봉2318개/7날짜의 기본·볼린저×비용2 네 진단은 신호0. null OHLCV412개·장외1개를 제거했고 합성하지 않았다. 고정KST/auto세션·봉종료/공개가능시각·10초 관측 차이·누락/coverage 미검증이므로0%성과/효과없음이 아니다. P1-05의 실제3개월/DST/달력/진입·마감창 검증 뒤 옵션 비교가 남는다. generic 거부·일봉 평가불가 유지. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L418-L423).

### SOXQ-P2-13

2026-10-29 로컬 계약만 완료: live6/replay9 fixture, v22의6연속+6분할+72분기와 보관78PC report/signals/preparation 일치·독립산술10536건. exact owner/native원가·partial/설정수량·거절/미체결·손절 전후·날짜/DST/scope·native/overlay·두비용·분할불변을 검증하되 정책은 바꾸지 않았다. live 잔량은 일일매도/guard에 막힐 수 있어 상태보존≠청산성공이다. [연구](research/price-condition-contracts-20261029/report.md), [회차](#task-immediate-backlog-20261029).

과거45창 sameClose 중앙값+0.2847%/완료117/비용부호반전3창과 v12 nextOpen3년은 별도다. 후자는 비용별3원장/완료매도3·rawStop/손절/guard차단/기말보유0, 분기36원장 중완료3·무완료33·손절/차단0. 2022-12-30 기준 SOXQ19.91/SPY382.43/QQQ266.28·수량51/2/3·native5%/3%·각10/20bp, 평균 초기비중0.0806/0.2430/0.1259%의 희소노출이다. 손절재진입/cooldown/기대순익 효능은 이 표본으로 평가불가. 기존price_condition8통과/781제외는 이전 동작 검사였으며 새 완료 증거가 아니었다. 실제장중자료/체결은 현재TODO에 남고 사후 기준가조정·guard완화는 없다. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L424-L436).

### SOXQ-P2-16

2026-10-29 v23 로컬 완료: 동일장부 구간 진단·0보합/음수손실 문구·소표본 설명을 flat/LTH/continuous·현재/A-B에 동기화했다. 경제값/체결/guard·v8 TR14(이전종가 포함)/동일간격warmup·기본off수량산정·LTH평가불가/실잔량보호를 유지한다. 70/30은 같은 상태가 이어진 사후 관측이며 학습/OOS가 아니다. 완료매도phase에 전체 배분원가 손익을 귀속하고 원래 전반양수·후반≤0/절반미만 판정식은 경제값과 함께 보존했다.

기존 v12 72연속(526/226봉)에서 후반양수/절반미만26·양→음16·양→0 6·한쪽완료0 32,864분기는52/132/53/786건이다. SOXQ PC10bp2023Q1 +0.7526%/완료1→0%/완료0을 ‘손실’로 부르던 사례를 교정했다. 새 v22 실행으로 오인하지 않는다.

사전고정12전략×3ETF×11fold×2비용의 context/evaluation 각792,총1584 reset계좌 회고walk-forward·PC78경제값/6분할 일치·phase3324/독립산술358749를 확인했다. 별도 이전commit빌드나 미관측OOS가 아니며 비용별396평가의 완료0은228/227, 모두20미만이다. 경고없음·청산0을 신뢰/우수/장중효능으로 승격하지 않는다. [계약](ipc-commands.md#v23-동일-장부-구간-관측-진단), [회차](#task-immediate-backlog-20261029), [연구](research/phase-walk-forward-20261029/report.md) · [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L437-L463).

## P3-REFACTOR-01

2026-10-18 trading.rs를 lifecycle469/positions329/balance96/risk_restore96/daemon527/history281행으로 분리해 IPC4·시작/복원순서와lib212를 확인했다. hooks는19행 facade+49~255행 모듈·export87/선언74동일. submission과order/fills는 준비/provider/기록·health/pending/run/collector/reconciliation/mode 책임으로 나눴다. [trading](#task-trading-module-split-20261018), [hooks](#task-frontend-hooks-split-20261018), [run](#task-execution-run-reservation-20261018).

2026-10-29 나머지 Trading1056→474(model414/form234), Toss1093→13(accounts111/diagnostics379/preflight236/orders371), LTH993→413(model343/picker177/preview161)을 완료했다. LTH는 시작부터1000미만이나 요청된 분리였다. 변경코드1000미만·IPC/query/state/effect/closure/JSX 보존, Toss32item 본문byte동일·Trading56문장/7helper·LTH68문장/5helper·재확장JSX AST동등, Rust831/clippy·TS/FSD/build·분리전mock5/후47UI를 통과했다. 실계좌/주문/키 접근0. [회차](#task-immediate-backlog-20261029) · [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L464-L476).

## 2026-10-29 문서 재편 검증

기준6eeb0ea clean·문서만 변경. 당시 열린19(자동11/판단4/자료4)·고유ID19·완료상위22/하위19·링크74를 보존하고 무ID제목6개에 대응ID를 부여했다. docs-canon1회는 링크/표/앵커/완료체크경고0이나 기존11문서미도달로exit1; 색인 보완 뒤 별도링크그래프로미도달0을 확인했으며 검사 재실행으로 표현하지 않는다. TODO/map/diff와 renderer3개를 통과했다. 최초 sandbox Chromium MachPort실패 뒤 허용환경375px의 키보드필터/JS없는읽기/overflow·오류를 확인했다. 거래앱/Rust/replay/실자료 미실행, 당시수치를 새검증으로 재사용하지 않는다. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L477-L488).

## 의존성 선별 전환 완료 — 2026-10-29

[아래 ID의 고정 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L489-L540). 각 절 끝에 원래 줄 범위를 표시한다.

각 전환의 부분완료는 아래 최종 상태에 합쳤다. 실제계좌/인증·배포/운영DB·OS신뢰 변경 완료가 아니다. [단계별 정본](#task-immediate-backlog-20261029).

### DEP-P3-03 — HTTP·WebSocket

reqwest0.12.28→0.13.5의8파일14생성점 TLStrust/provider·옵션을 보존했다. HTTP/TLS10·lib841, 5A `0dead386aa0e1270cff4092009b253dc0f37e8e5` Gate43/Security50. WS0.24→0.30은wire/native roots/ring/4KiB chunk 유지·교체전 재현 Close응답누락/실패연결flag 교정·WS/WSS14, 5B `270296db196ee7191e5c4848b1346095b1061dc7` Gate44/Security51·LinuxRust855/UI344. query/header/JSON/cookie/error·승인/구독/수신/pingpong/종료/명시재연결을검증했고 자동재연결은 추가하지 않았다. 기존RSA예외/Linux경고2유지. 원문 L489–505.

### DEP-P3-04 — 서버

axum0.7.9→0.8.9/tower-http0.5.2→0.7.1,70URL/74binding·보안handler계약/13capture문법 변경. 단계서버24·Rust873·strictclippy/check/fmt/custom-protocol/타입/FSD/build. `653377ce4ba49bc679d1762fe0f5d6ec2409fb74` 원격일치·Gate45/Security52·Linux/frontend/OpenAPI/UI 성공. 모든handler성공/실제provider/배포 실행은 아니다. 원문 L506–515.

### DEP-P3-05 — React·MUI

React19.3.1/types19.3.0→MUI7.3.11→9.5.0, Grid/slot/sx 외 금융AST·callback/scope/preview 보존. clean설치/타입/FSD/무경고build·UI349·별도개발5·Tauri custom-protocol. `f15f192fee105600d79468291334fb10f1495e9f` 뒤 공개명세CI보정 `99b85eb3cdc63deceb7ec60c325559607f98f8ce` 원격일치·Gate47/Security54. native startup/실계좌/Windows전수 검증은 아니다. 원문 L516–525.

### DEP-P3-06 — Vite·React plugin

Vite6→7→8.3.5/plugin6.1.2·Oxc/Rolldown, `06a84fe32fa37dd0f31a13c21f13efb6dc4e8ac1` Gate48/Security55. clean설치·target/debug4조합·UI349/Tauri빌드·별도승인 WKWebView 차트/테마/입력/종료재실행. calendar fixture차단1과원래Tauri startup/IPC·Windows/macOS12/CSP negative control 제외를 보존한다. 원문 L526–530.

### DEP-P3-07 — 나머지 선별

sha2.11.1/thiserror2.0.21·기존hex0.4.3 재사용, keyring3/RGL1은 초기화/backend·이익 평가 뒤유지·삭제없음. `fe3a3f8a8836c7c20745b3a7c7ea7e09e68ccda4` Gate49/Security56·동일literal14·Rust882(실DB5제외)/UI349/clippy/build. 최초LowerHex실패와호환baseline·본문동등, 최소Rust/다른OS/실DB미검증을구분한다. 원문 L531–535.

### DEP-P3-08 — TypeScript·패키지매니저

TS5.9→6.0.3→7.0.2 격리 비교·원문130/node1·strict/동일dist, npm11유지/pnpm미채택/global무변경. `adf0e5a4e4635e1e3cea9540df3016e6f60b070d` Gate50/Security57·clean설치/타입/UI349와후속loader를구분한다. 동일프로젝트warm5중앙값TS6 2.1884→7 0.5722초이며 전체CI/순수CPU/설치크기 개선으로확대하지않는다. 초기도구수정/지원환경미검증유지. 원문 L536–540.

## UIUX-20261030

`ui_ux_expert`는 기존TOML형식·부모모델/권한상속으로 구성했다. 현재세션 일반subagent가TOML을읽은것과 새세션autoload미검증을구분한다. [역할](../.codex/agents/ui_ux_expert.toml), [사용법](agent-tooling.md#uiux-전문가-사용).

`7f2ec166a47d88508f2e72e10ab6355f9e9a585e` dist10SHA를TS7검증과대조한 정적loopback·freshChromium·serviceworker/외부HTTP/WS차단·쓰기403/미지정503·합성KIS관측이다. 초회02:25:51~02:26:08UTC8+후속02:28:23~02:28:29UTC4는12관측이며기능PASS12가아니다. 1280×900light의status/balance/stats503×3와별도지연에서대기/보유없음/0원표시,Log빈성공/후속503×3의0건을확인했다. Settings/Strategy375×812dark문서overflow0, 해외실패/staledata는소스추론이었다.

초회Settings Tab은log-config POST1을유발했으나403차단,누락budget GET503도보존하고제품결함으로세지않았다. 후속입력/Tab없는캡처는쓰기/unknown/외부/pageerror0,전체외부/pageerror0. 모든프로세스종료. `.cache/uiux-review-20261030/`의초회/후속JSON·PNG·script와`/tmp/kis-uiux-expert-20261030.md`가원본이다. nativeIPC/Toss실행/AT/전반contrast/200%확대/최소OS는당시미검증.

역할5TOML·신규권한불변·기존TODO10본문/고유20·HTML/map/graph/diff·renderer3(좁은폭/keyboard/JS없는읽기)·독립리뷰를통과했다. 제품/의존성/workflow무변경이므로Rust/전체UI349를새실행하지않았다. 제안9개는이후아래에서구현했고판단P2-UX-07과기존대기는유지한다. [이전 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L541-L565).

## UI/UX 개선 완료 — 2026-10-30

[아래 ID의 고정 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/completed-history.md#L566-L691). 각 절 끝에 원래 줄 범위를 표시한다.

아래는 production mock의 고유검사 수이며 인접/후속재실행을 합산하지 않는다. 공통 앱/config타입·FSD·보호본문/AST 대조와 독립리뷰를 통과했고 실거래/운영설정·사용자판단/실자료를 대신 완료하지 않았다. [회차 기록](#task-uiux-improvements-20261030).

### P1-UX-01

QueryStatusSummary로Sidebar/Dashboard 미확인·첫실패·이전실행/정지자료·마지막정상시각·읽기재시도를일치시켰다. query/금융handler/버튼조건보존. 신규5+기존UI5+합성Sidebar시작정지1=고유11,160pxSidebar/375pxdark캡처,독립문구기대P2교정. 초회3/중간9/후속6중복합산없음. 원문 L566–580.

### P1-UX-02

KIS국내/해외 독립상태·시각/retry·같은scope이전자료,실패와정상빈잔고분리·Toss/선택/주문보존. 신규10+기존폼5=15(27.8초),각시장지연/빈/보유/이전자료·키보드연속2회재조회·다른시장불변·보호9항목/375dark. 외부guard검사P2보강. 원문 L581–595.

### P1-UX-03

History통계/체결독립상태·시각/retry·날짜draft/실제기간구분·같은query이전자료. 손익/승률/통화/trace보존. 신규6+일봉13=19(17.2초),통계실패/체결성공·정상0/양쪽실패·각stale/retry2회·미적용기간·보호11항목. 원문 L596–610.

### P2-UX-01

Trading/Strategy검색과KIS/Toss양시장보유 native명명선택·focus·내부Tab유지/Escape·입력복귀. RowSelectionButton5곳,검색표52행분리/Page982. 신규10+폼5+holdings10=25(27.8초),blur/외부닫힘/포인터/375dark긴이름/overflow0,보호14·전략함수15AST동등. 원문 L611–624.

### P2-UX-02

Sidebar/Log명명separator·Arrow/Home/End/Enter기본값·명시target즉시저장. pointer delta/end·범위/키/flex보존. 신규5+scroll5=10(4.7초),repeat/경계/route/reload/다른pref/반응형·동일geometry1재검사,보호12/AppShell12·Log4callback동등. 원문 L625–638.

### P2-UX-03

History상세Dialog에동일체결의시각/ID/통화/경제값/전략/trace. ID+적용기간선택·대상소실시폐기·기존formatter보존. 신규4+상태6+일봉13=23(23.3초),375/1280·다중체결/긴ID/nulltrace/keyboard/focus/scroll. 실제Chromium tabzoom2의CSS1280→640/1000→500,DPR1→2,visualScale1을확인했다(AT/native아님). 함수13/보호15. 원문 L639–652.

### P2-UX-04

Log placeholder/실응답·로딩/실패/무기록/필터0/stale·시각/retry/검색label/필터초기화. 5초poll의빈번한값은live영역밖,tail300/행/trace/filter/scroll/follow/resize보존. 신규5+resizer5=10(13.3초),이전0/150행·poll2회정확증가·위scrollTop0/하단follow·statusDOMmutation0,함수6/보호18·375dark/desktop캡처. 원문 L653–666.

### P2-UX-05

Settings15목차/h1·h2·복구h3·focus/복귀, Risk지연/실패제목유지. 지연routeback이명시이동을덮는문제는동기알림+같은route pending만취소해교정했다. 저장/DB/금융/원래복원보존. 신규6+DB예산4/복구11/UI4/resizer5/Strategy3=33(17.1초);375dark/1280·draft무저장·지연back반복. 동일1case재검증2.5초와test-only anchoring차단에서wrong-route top4824복원/same-route3929유지(895px차이). 보호40/원래6파일대조. 원문 L667–680.

### P2-UX-06

두차트동일변환결과를canvas와CandleDataSummary가공유,OHLCV/시점/조회조건/최근20봉표·keyboard선택/scroll·scope/새자료선택해제. KIS KRW/USD·Toss통화미확인구분/명명toggle,새조회·환산·시각변환없음. 신규8+폼5+LTHpaint1=14(15.0초),유색픽셀/빈격자0·US1분/단위/20봉/상태/theme/unmount. 동일lifecycle2후속4.8초에서backing/CSS570→256·584→270,scroll0→201/199,우측거래량18/3확인. 보호44/converter/query/effects/model대조. 원문 L681–691.

<a id="keyboard-final-ci-20261031"></a>
## 최신 키보드 게시·CI 영수증

P2-KBD-01~05를 순서대로 구현·검증·푸시했다. 마지막 검증 코드 **`7b8b9978a182c8dc138264e5b61cdb9a8c775a70`**, 그 결과를 기록한 문서 **`4021e38bc5caaf4751eacaf96c20dc54dcf0f708`**는 서로 다른 커밋이다. 같은 코드의 [Release Gate58](https://github.com/CountJung/KISAutoTrade/actions/runs/41253655588)과 [Security65](https://github.com/CountJung/KISAutoTrade/actions/runs/41253708366)가 성공했다. UI557/557(14.0분; 기존408+키보드149), Rust901/901(12.81초), renderer3·bootstrap4·compiler 경고0을 로그와 대조했다. 기존 RSA 예외와 외부 도구 deprecation은 별도다.

CI54(553/4)는 실제 입력·backdrop hit와 저장 중 focus를 구분하는 준비 보강, CI55(556/1)는 최초 query body→화면 준비, CI56(556/1)은 잠금 DOM의 원자적 제거/disabled 관측, CI57(556/1)은 첫 정상 preview 결과 대기로 교정했다. 제품 정책·검사 등록수·timeout을 완화하지 않았고, 같은 실패 run을 중복 실행하지 않았다. 이전 파일 쓰기 승인 거절은 미실행 확인 후 새 사용자 지시의 정상 재검토로 한 번 해소했다. 현재 승인/게시 차단은 없다. [회차·red/green·정확 SHA 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/keyboard-improvements-20261031.md#keyboard-final-ci-20261031).

실제 macOS 한글 IME·Tauri WebView·스크린리더·운영 계좌/시장자료는 미검증이다. 공개 scope를 바꾼 합성 검사와 실제 계좌 pinning, UI 응답 격리와 backend abort/idempotency는 구분한다. 기존 TODO11개는 이 완료에 포함하지 않는다. 아래/이전의 작은 검사 수와 반복 실행은557에 더하지 않는다.

**2026-11-05 병합 검증 후속:** HEAD `8cd29b8a7bf5b363b85fe245c5e5cec10963cdb7`의 [Release Gate61](https://github.com/CountJung/KISAutoTrade/actions/runs/41943810202) 최초·재실행은557중556개 성공, source KIS Strategy 결과 교체 후 명시 키보드 선택 복구1개에서 전체30초 timeout이었다. 호출자의 독립 trace 감사에서 초기 `clock.runFor(1)`에16.660/18.340초가 소요됐고 이후 선택 흐름은 진전했다. GET/200/body 완료·microtask·clock·input readiness는 이미 확인하므로 fixture나 제품 코드를 추측해서 바꾸지 않고 CI 전용 source project의 전체 예산만60초로 분리했다. production/local30초·assertion5초·557개 시나리오와 fixture는 유지한다. 초기 지연의 근본 원인은 미확정이며60초 재실패 시 추가 연장하지 않고 trace로 재진단한다. 새 HEAD의 전체 CI와 병합은 호출자가 확인한다.

로컬 Node24.19.0·Playwright1.61.1·시스템 Chromium151에서 envDir:false·비민감 환경 allowlist·합성 localhost fixture로 실패 시나리오3회가26.5/23.5/24.1초에 통과했고, 별도 results-scope4개+search-scope8개의12개도 통과했다. 반복3회는 고유12개와 중복되며15개 고유 검사로 합산하지 않는다. 실제 CI/비CI config 로딩 대조에서 차이는 source CI timeout 한 항목뿐이고 기본30초·expect5초를 보존했다. Playwright 등록도 CI513+44=557·로컬557·오류0이다. 지도·TODO·Graphify freshness·diff 및 독립 코드/문서 감사를 통과했다. Graphify는 대상 config의 baseline/current AST1node/1edge 동일을 확인하고 지원 subset manifest와 기존 snapshot 생성 로직만 사용했으며 graph/HTML/report는 원본 그대로다. 원본 결과는 Git 제외 `.cache/source-harness-timeout-20261105/`에 보관한다. 로컬 시스템 브라우저 결과는 CI 번들 Chromium의 새 HEAD 전체 검증을 대신하지 않으며 계좌·주문 API/운영 앱/배포는 실행하지 않았다.

## 통합한 작업 기록의 검증·결정 색인

기존 중복 구현 설명은 위 ID 절과 현재 계약/연구 정본을 따른다. 아래35개는 당시 코드·환경의 별도 검증 영수증과 고유 판단을 보강한 것이다. 시작/baseline SHA를 게시 SHA로 바꾸지 않으며, 원문의 게시 예정·미착수는 당시 상태다. 원래 완료 조건·명령 출력·중간 실패는 각 불변 Git 원문에서 복원한다. 실제 사용자 변경·미추적 원본을 삭제한 것이 아니다.

| 날짜·작업 | 당시 코드·검증 | 판단·미검증 경계 | 고정 원문 |
|---|---|---|---|
|<a id="task-appledouble-and-soxq-p1-03"></a>2026-10-03 · SOXQ-P1-03|Rust lib180, all-target check/Clippy 경고0; Playwright22, TS/FSD/build/map/graph 통과; 같은 입력2회+각20bp 지원24실행 일치; 다른 전략20실행 v4 결과 유지|현재일 범위를 이전 N일 평균에 섞지 않고 flat range0 보존; 공용 기본 훅은 live tick 경계를 유지; 기존 미커밋 사용자 작업 보존; 커밋/PR 생성 없음; 실전·모의주문/자동시작 미실행; SOXQ +1.7179%/+0.7807%는 미청산 수량 포함 엔진 진단; 성과 우수성 아님; 당시 P1-04/05·P2-14/16/17 잔여는 후속기록과 구분|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/appledouble-and-soxq-p1-03.md)|
|<a id="task-project-health-and-soxq-p1-01"></a>2026-10-03 · SOXQ-P1-01, 프로젝트 캐시 경계|main; 커밋없음; 캐시이동후초기Rust142, P1-01최종Rust167; UI21/39.5초·TS/FSD/build/Clippy 경고0; 13전략×2모드64봉, 지원12×2 생성시각외재현/20bp스트레스|registry/toolchain공통cache미이동·연구원본보존; 부족자료에기간단축금지·가상warmup주문금지; 이전p1-02 fmt실패는당시기록보존; 외부DB없는contractskip은실서버검증아님; 당시날짜이벤트/LTH/nextOpen/TRATR는후속범위; 실주문/자동시작없고커밋생성없음|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/project-health-and-soxq-p1-01.md)|
|<a id="task-soxq-p1-04"></a>2026-10-03 · SOXQ-P1-04|baseline65e465e에P1-01~03보존·origin/main게시확인; Rust181lib+3productionintegration+3doc=187·UI22; generatedAt외전체JSON반복동일·지원24 v5결과동일|provider조회자동fallback금지; generic차단대안을선택·엔진v6/deterministic지원범위표시; LTH시장세션/DST·실분봉/P1-05·nextOpen은당시미완료; 실주문/모의운용없음|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/soxq-p1-04.md)|
|<a id="task-soxq-p1-05"></a>2026-10-05 · SOXQ-P1-05|시작main cbc4553; Rust190lib+3integration+3doc=196·UI26; 지원24+구조적거부2 v6동일반복·TS/FSD/build/Clippy경고0|일봉재초기화가분봉/보유/highwater/age지우지않음; warmupCount0·dailyContext/intraday별도·LTHv3/enginev7; 일봉→분봉보간금지; 2026-07-06~10-02 원본/공급원/시간대·start/end·정규연장·캘린더·조정/누락 metadata대기; 실전체3개월 replay NOT RUN·운용/모의소액별도승인; 전체Done으로압축금지|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/soxq-p1-05.md)|
|<a id="task-autonomous-backlog-20261018"></a>2026-10-18 · SOXQ-P2-05, SOXQ-P2-16, LTH 실제 체결 feedback, P3 사용자 가이드|시작694fa14; 최종 Rust212lib+3integration+3doc=218, Clippy 경고0; focusedUI10, 앱36+TODO3 E2E; v8 두 실행1432행 동일; 1402행 v7동등/52주30행 차이|ATR 기본 비활성·신규매수 차단/보유매도 유지; 동결 bool 연구 API 보존; replay 전용 module 유지; live 일봉/providerpaging/walk-forward/연속replay/체결모델 등 당시 잔여; 외부 DB fixture 미설정 early return은 실DB 검증 아님; 실계좌조회·실주문·자동시작 없음|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/autonomous-backlog-20261018.md)|
|<a id="task-consecutive-entry-filters-20261018"></a>2026-10-18 · SOXQ-P2-06|기준6f9ca35; 구현d39d15f 원격동일/clean(문서 최종감사 후속기록 별도); Rust537lib+4integration+3doc=544; E2E125+production33+TODO3; 기본flat1224+long6, 후보연속24/분할24/분기288; 최종312원장148099체크 오류0; map/graph·253링크|기존 필드명 저장 호환; 숫자+관측 단위 문구만 reason정규화; legacy warmup이 context 무효화하면 ready표도 지워 재요청; 차트 chunk 분리로 build경고 해소, warning limit 유지; 관측한2023~2025 상관ETF3개 사후진단; 2026 미사용은 독립OOS증거 아님; 분기는 별도1천만원 계좌; 수익률합산/복리 금지; 실자료coverage·장중체결·운용안전 미검증|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/consecutive-entry-filters-20261018.md)|
|<a id="task-continuous-replay-20261018"></a>2026-10-18 · SOXQ-P2-18|시작cc6f5d0; 게시커밋은원문Git이력참조; Rust517lib+4integration+3doc; UI114; 기본flat1224 full report/signals 일치; 연속72/alternate72/초기화864; root136717/독립143866 산술체크|16청크/각500·평가4096/history512, UI48페이지/4608보관; 7달력일 초과 공백은 보수적 계산보류; 누락봉 합성 금지; 모드는 경제hash 포함·청크분할은 제외; typed신호키순서로 baseline대조; 원장30MB→gzip4.2MB 무손실·SHA보존; LTH시간대·실분봉coverage·실거래·서버세션저장·walk-forward 미완료; 완료청산0 평가수익/실제0주비중제한은 동일노출 성과아님; 고정FX/무배당/추정비용·기업행위/달력 미검증|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/continuous-replay-20261018.md)|
|<a id="task-database-tls-reconnect-20261018"></a>2026-10-18 · DB TLS·단절·복구|시작f7d3a24; 게시Git이력참조; Docker run5f656179: 일반3+TLS재접속1+불신CA1 실제통과·정리확인; Rust464lib+3integration+3doc(외부5 earlyreturn 별도), focusedDB29; UI DB4·subprocess timeoutcleanup2·정적게이트/Clippy 경고0|정확한UUID/소유label·loopback·임시CA/합성credential만 사용; retention fixture를오늘/91일전으로 교정, 제품90일규칙유지; 취소/post-rename/marker sync 실패는 복구확인 전 차단; Windows 실제실행/전원차단/운영keychain·DB/MariaDB 미검증; 원격tag release미실행; marker만삭제해복구우회금지|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/database-tls-reconnect-20261018.md)|
|<a id="task-documentation-layout-20261018"></a>2026-10-18 · 루트 문서 정리(1차)|시작cd35389; 게시Git이력참조; 상대링크110/독립107 broken0·연구SHA20일치; cargo check/TS·map/graph freshness·독립리뷰 통과|기존연구 validationhash 재작성금지; index.html/LICENSE·도구진입예외유지; 동작변경없어 Rust단위/E2E미재실행; 운영배포/모의소액범위아님|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/documentation-layout-20261018.md)|
|<a id="task-execution-run-reservation-20261018"></a>2026-10-18 · P1-RUN-01|시작2c593de; 신규race22 포함 Rust297lib+3integration+3doc=303; Clippy/TS/FSD/build 경고0·함수71본문분리전동일성; TODO3·152링크·map/graph|manager→order lock순서·networkawait에bool/strategyguard유지금지; KIS pending TR-ID 모드불명/반대면조회·종결보류; 비활성profile삭제로활성runtime재생성금지; 실provider·실모의주문·자동시작미실행; async close stop은프로세스종료내구성보장아님; 당시일반설정 보유보존 P1-CONFIG-01은후속범위|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/execution-run-reservation-20261018.md)|
|<a id="task-frontend-hooks-split-20261018"></a>2026-10-18 · P3 frontend Query hook 분리|기준eb8b63d; TS AST함수72/전체선언74 exact·export87동일; TS/FSD/build 경고0·E2E36·TODO3·map/graph|새module은facade역참조금지·전면FSD이전미포함; 의미동일본문복사테스트추가안함; Toss38경로명세검사는runtime섹터구현아님; 실주문·계좌API·자동시작미실행|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/frontend-hooks-split-20261018.md)|
|<a id="task-holdings-publication-fence-20261018"></a>2026-10-18 · P1-BAL-01|시작323727c; Rust359lib+3integration+3doc=365; 계좌전환/이벤트UI13+기존26·TODO3; A→B→A/partialfill/두시장원자성·159링크·정적게이트 경고0|새조회실패해도old응답복귀금지·priceupdate는수량version유지; 진짜empty/0은삭제·무효수량/가격은전체오류; profile5mutation직렬화·late listener해제/unmount보호; provider잔고자체지연·조회전fill+reconcile이중반영미해결; 공유총수량을전략소유로추정금지; 당시 P1-FILL-01후속; 실provider인증/실모의주문/자동시작미실행|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/holdings-publication-fence-20261018.md)|
|<a id="task-live-completed-daily-history-20261018"></a>2026-10-18 · SOXQ-P2-05, SOXQ-P2-17|Rust275lib+3integration+3doc=281·UI35; manager19+collector/scheduler scope/날짜/공유수집회귀; 133링크·map/graph/정적게이트 경고0|동시2·요청8·512봉·대상511+초과진단1; 최종완료일1~10일과거만ready; 실제휴장달력증거아님; 52주전용legacyseed제외; 완료봉재차제외금지; 실provider·계좌주문미실행·긴휴장도보수적보류; 당시quote→예약 pinning/P1-CONFIG보유재생성위험은별도후속; LTH초기화미호출|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/live-completed-daily-history-20261018.md)|
|<a id="task-provider-candle-contract-20261018"></a>2026-10-18 · SOXQ-P2-17, SOXQ-P2-05 선행|Rust243lib+3integration+3doc=249·UI54; localhost실adaptercursor/+URL·KIS요청/파서·8MiB상한; UTC/Seoul/NY독립프로세스 hash/trades일치·DST반복시각회귀|해외dailyprice HHDFS76240000·xymd; 현재가TR유지; Toss200/KIS100, UI8요청/500보관·cur_scope/asOf고정; NAS→NYS→AMS는첫provider오류만·성공뒤거래소유지; 주/월/당일 보수적제외와provider확정flag를구분; 공개스펙/합성fixture와실providercoverage구분; LTH세션/DST/3개월운용미검증; live연결은후속live완료일봉기록참조|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/provider-candle-contract-20261018.md)|
|<a id="task-replay-accounting-20261018"></a>2026-10-18 · SOXQ-P2-15|시작1192673; Rust504lib+4integration+3doc·UI104, focused47/formatter후31별도; 기본1224 metadata외 report/signals exact·합성시나리오40/항등120; 독립fill4540/경제640항목·최대0.01원차|FX배당각64개·정확평가봉시각·이중배당거부; 최소fee native/방향별fee·수동확인은사용자선언; 실제경로비용환급은산술참조,무비용재실행아님; fmtRoundedKrw만공유·음수half-tie다른함수유지; QQQ실가격+합성FX/배당/요율40은독립시장40표본아님; 실브로커요율인증/USD현금장부/실배당자료아님; PG환경없는5earlyreturn은실DB검증아님|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/replay-accounting-20261018.md)|
|<a id="task-replay-execution-model-20261018"></a>2026-10-18 · SOXQ-P2-14|시작ba74e03; Rust480lib+4integration+3doc·UI95·focused38; baseline1224전체report/signals exact·실가격8+benchmark8; simulation25·기존runner2offlinecompile|sameClose신규진입에과거고저가적용금지; LTHOpen에미래closehighwater혼입금지; sameClose lookAheadSafe=false/LTHfalse 유지; 고정FX/일봉소표본·미청산평가이익/완료손실혼합; 실유동성·LTH시장시각·배당/기말청산 미검증; 당시P2-15/18은후속완료항목과구분|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/replay-execution-model-20261018.md)|
|<a id="task-strategy-configuration-preservation-20261018"></a>2026-10-18 · P1-CONFIG-01|시작3040e8c; 문서이전694fa14; Rust342lib+3integration+3doc=348(45추가); UI22+설정오류4·TODO3·156링크; 13전략reconfigure·owned취소/실제저장뒤오류복구·정적게이트경고0|보유제거/시장변경사전거절·PC빈symbols실효대상반영; 다른profile에private인스턴스이전금지·공유0잔고로private보유삭제금지; same_identity기계predicate만13caller공유; 당시P1-FILL/P1-BAL은별도후속완료기록; 거래소세션/손익안전·실계좌운용인증아님|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/strategy-configuration-preservation-20261018.md)|
|<a id="task-strategy-fill-ownership-20261018"></a>2026-10-18 · P1-FILL-01|시작886dd5b; Rust449lib+3integration+3doc=455·UI89; Tossv1.2.24/38paths·162링크·정적게이트 경고0; legacy wholeaccount원가/자정손실/terminalintent/즉시fillID 등독립finding 수정재검증|fill은tracker delta아닌조회무효화·재조회전예약보류; 소유ledger/generationmanifest 손상·불일치는failclosed·낡은backup자동복구금지; 0.00015=0.015%·비정상비율0수수료통과금지; ledger+manifest모두삭제coldstart를신규설치와구별못함; 계좌총잔고로잃은전략귀속복원금지; 실provider지연/운영원장/실제분봉미검증|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/strategy-fill-ownership-20261018.md)|
|<a id="task-strategy-review-20261018"></a>2026-10-18 · 전략 전수 연구, stock_analyst 역할|시작2cd04db; Rust197lib+3integration+3doc=203; 1432행전체반복generatedAt외일치·harnessClippy경고0; 기존SOXQ24/거부2/진단/benchmark동일·TOML모델상속검증|전략Python재구현금지·생산Rust실행; 분기252준비/최근248평가·두비용/동일요청수량; 재다운로드수정가능하므로보관input기준재현; sameClose/고정FX/무배당/낮은노출/미청산/상관표본; LTH0신호=0%성과아님·P1-05실3개월/시장시각미완료; Playwright미실행(제품/UI변경없음)|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/strategy-review-20261018.md)|
|<a id="task-trading-module-split-20261018"></a>2026-10-18 · P3 자동매매커맨드 분리|기준191fa93; Rustlib212·check/Clippy/TS/FSD 경고0; fmt/map/graph/TODO HTML 통과·독립본문동일감사|privatehelper하위module한정·외부재수출만기존visibility; 순수이동본문복제테스트추가없음; 분리만으로live52주고가갱신구현아님; 실계좌/API/주문/자동시작미실행|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/trading-module-split-20261018.md)|
|<a id="task-deviation-policy-20261020"></a>2026-10-20 · SOXQ-P2-04|baseline bca6601/v15; 게시예정문구만있음; Rust583lib+4integration+3doc=590; E2E155 및 별도추가case/최종spec18; 비대상1122exact·대상102delta, 연속36/분할36/분기432; 468원장 root68421/독립200495검사 오류0|약10초관측·warmup일봉/분봉을분리; isFiniteNumberInRange/removeDraftIfUnchanged만 공유승격, domain정책local; ABA callback 실제재전달source검사로우연한 observer폐기성공 방지; raw305→23/차단286→4는 수수료절감아님; 노출·미청산수익·비중제한0주는우수성아님; 상관3ETF 사후진단; 추가case를156단일실행으로합산금지; 실제provider/운영DB/장중미실행|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/deviation-policy-20261020.md)|
|<a id="task-ma-cross-filters-20261020"></a>2026-10-20 · SOXQ-P2-01|기준327739a/v16→v17; Rust635lib+4integration+3doc=642·MA27; UI191/focused20·baseline1224 full exact; 최종48연속/48분할/576분기, root104884+3240/독립313205검사|ATR0/폭0은자료충분시ready·양수문턱미달; 손상legacy기간 slope활성 검증우회차단; StageA36→최종48 후보추가는사후탐색으로구분; 전체사전등록/OOS·SOXQ일부개선의기본승격아님; 독립current1224재실행아닌runner/hash감사; 실provider/주문/DB미검증|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/ma-cross-filters-20261020.md)|
|<a id="task-mean-reversion-policy-20261020"></a>2026-10-20 · SOXQ-P2-10|baseline474aa06/v15→v16; Rust611lib+4integration+3doc=618·UI171 후문구focused15별도; 최종36연속/36분할/432분기468원장·flat102; root74968/독립219769검사·TrendFilter실행bytesSHA동일|기존ddof0/strictband·기간버퍼/reset·1분warmup no-op유지; 나이불명 활성maxHold은보수적매도후보; 완료자료부족중에도stop/maxHold평가; 1분준비분봉을기존경로가사용하지않는한계보존; 63관측미발동으로효능평가불가; 사후21/상관과거자료OOS아님; QQQ1주기존4경로와새3주연구혼합금지|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/mean-reversion-policy-20261020.md)|
|<a id="task-momentum-feedback-20261020"></a>2026-10-20 · SOXQ-P2-03|기준86a9829/v14; Rust553lib+4integration+3doc=560·UI138; 비모멘텀1122exact·Momentum102경제동일·연속24/분할24/분기288; 312원장root46772/독립136960체크 오류0|같은0수량/nextOpenpending은체결아님·양수feedback에만후보소모; gap1에서4<m<5는후보취소만,재무장아님; warmup가상후보/포지션생성금지; raw차단감소는수수료/수익개선아님; partition감사는독립대체재실행아님·상관3ETF/OOS아님; QQQ부분개선은낮은노출/잔여평가동반·실운용미검증|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/momentum-feedback-20261020.md)|
|<a id="task-root-documents-20261020"></a>2026-10-20 · 루트 LICENSE 문서정리(2차)|선행Momentum be732e7; 문서게시예정; LICENSE바이트/SHA55d027a578fedf8881e20abd62556611978529201c87471b4e94c9f8e3a120d9동일; 링크48·map/graph/TODO/diff통과|도구요구숨김진입/Graphify산출물예외유지; 선행Momentum be732e7 별도커밋; 기능테스트추가없음·제품동작변경없음|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/root-documents-20261020.md)|
|<a id="task-failed-breakout-policy-20261022"></a>2026-10-22 · SOXQ-P2-07|baseline13c5fd3/v18→v19; commitpush예정문구; Rust718+integration4+doc3·focused30; UI21/전체238; 비대상1122exact/대상102sameClose변화0, 연속42/분할42/분기504; 546원장63168봉·root85073/독립233251검사 오류0|raw정상0/결측분리·현재누적거래량을과거완료량으로사용금지; 체결반영후pending해제·실제잔량매도; v1미확인/v2엄격복원; nextOpen 소유교정은경제경로변경, sameClose와구분; 비중제한SOXQ분기42계좌0주·독립OOS아님; P1-05/P2-12 실자료운용게이트미완료·최종지연심볼호출결과미사용|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/failed-breakout-policy-20261022.md)|
|<a id="task-lth-minute-timing-20261022"></a>2026-10-22 · SOXQ-P2-12(자동진단 하위항목), SOXQ-P1-05|시작ec57c02; Rust660lib+4integration+3doc=667·minute25/preview79; 양수체결fixture signals/backtest/inputhash exact·엔진v17/LTHv4유지; UI최종15/전체202·188링크/anchor2|TossbarClose label/label−60초, explicitbarStart label/+60초; RFC3339실제순간변환·compact12/14는legacyKST가정; 512입력/64byte/진단목록상한·gap산술계산; 실제3개월자료·휴장조기마감·coverage·실수신발행시각 미검증; 상위P1-05/P2-12 미완료유지, 자동하위1개만완료; 초기UI11/12 후fixturefalse교정/손상컬렉션보강을단일15성공과구분|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/lth-minute-timing-20261022.md)|
|<a id="task-rsi-policy-20261022"></a>2026-10-22 · SOXQ-P2-02|baseline978640c; Rust687lib+4integration+3doc=694·UI217/focused15; 비RSI1122exact/대상102delta·연속30/분할30/분기360; 390원장root61782/독립185202체크 오류0|zero-loss100·매관측기준갱신유지; runtime경제SHA와후속testmanifest갱신분리·audit순환hash금지; hold63 전체78경로owned동일로효능미평가; 비RSI/partition은부모증거확인이지독립재실행아님; P1-05/P2-12 실제자료/운용미완료|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/rsi-policy-20261022.md)|
|<a id="task-strong-close-policy-20261022"></a>2026-10-22 · SOXQ-P2-08|baseline ac9d7d0e432b2e992aba58fce68187de18777fa5/v20; Rust745lib+4integration+3doc·focused34·UI262/focused24; 기본1224/비대상1122·대상102/owned78경제신호동일; 연속30/분할30/분기360·root67393/독립253595검사390원장|D-1조건→D종가신호→D+1관측시가와익일시가매수오해구분; 기본3/3·inclusive(H-C)/H·stop0원가이하보존; 상태/peak/metadata무리한공유없음; 원본owned연속6 청산0·잔량평가수익을청산성과로표현금지; hold63 마지막신호10/실한도청산0·효능미평가; 대체partition원장미저장/독립재정산아님·P1-05/P2-12대기|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/strong-close-policy-20261022.md)|
|<a id="task-volatility-expansion-policy-20261022"></a>2026-10-22/2026-10-23 · SOXQ-P2-09|baseline47fc10ba744223ec069edc71d77a4d05f425ff19/v21; Rust775lib+4integration+3doc=782·focused41; UI전체306후문구수정focused46, 전체308단일통과주장금지; 36연속/36분할/432분기·468원장root70828/독립249412|새시장일첫quote는거래소시가아님·관측고저는전체장중범위아님; 평균H-L을ATR/종가평균helper와합치지않음; 진단512상한초과는ready아닌미확인; 실장중최초발동·체결·세션/누락/coverage미검증으로상위P2-09열림; 대체분할원장미저장·hash동등성36만; 실주문/모의운용미실행|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/volatility-expansion-policy-20261022.md)|
|<a id="task-trend-filter-policy-20261023"></a>2026-10-23/2026-10-25 · SOXQ-P2-11|시작70df3b3·문서b3c441c보존·연구baseline81fa00d/v21→v22; Rustlib789·UI322(production290/source32), focused84별도; 30연속/30분할/360분기·비대상1122/대상102/owned78호환; 390원장root63559/독립171604검사 오류0|positiveunknown metadataSome·원자복원뒤streak0; StrongClose전체state공유안함·age기계동작만재사용; baselineharness는별도offline lock해석,원Cargo동일빌드주장금지; hold63분기72경제동일·16lastpending추가로분기효능미평가; SOXQowned+9.5760%=실현손실+잔여평가혼합; 실운영DB/주문/장중체결미검증|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/trend-filter-policy-20261023.md)|
|<a id="task-immediate-backlog-20261029"></a>2026-10-29 · P1-RECOVERY-01, SOXQ-P2-13, SOXQ-P2-16, P3-REFACTOR-01, DEP-P3-03, DEP-P3-04, DEP-P3-05, DEP-P3-06, DEP-P3-07, DEP-P3-08|시작5437c07bc6c2bf0876c0ddc211ddeabb6cd632d7; 아래 회차별 게시 추적 참조; 회차별별도영수증·반복실행수합산금지; 마지막Gate50/Security57 동일adf0e5a에서Rust887/UI349; 격리WK실제paint/Tab/Escape/종료재실행·stale재시작PNG분리/provenancemanifest; TS6→7 동일source파일명검사warm앱3.82배/node2.10배, executor/ambient차이·전체앱성능아님|복구승인은desktopIPC한정·pending복원이며재주문/체결합성/예산직접해제아님; PC상위미완료·70/30같은장부진단과회고walkforward/OOS분리; 기존RSA예외와LinuxRustSec2경고유지·최소Rust1.90실행성공아님; 자동승인거절은미실행확인+새원문근거1회정상재검토로해소·우회없음; SOXQ-P2-13 실제장중자료/효능·실접수체결미완료; 실DB미설정5earlyreturn을887실DB통과로세지않음; WK는bare Wry/Tao·CSPnegative/IME/원TauriIPC/Windows/macOS12미검증; SQLx/MariaDB/서명/실자료·사용자판단미완료유지|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/immediate-backlog-20261029.md)|
|<a id="task-uiux-improvements-20261030"></a>2026-10-30/2026-10-31 · P1-UX-01, P1-UX-02, P1-UX-03, P2-UX-01, P2-UX-02, P2-UX-03, P2-UX-04, P2-UX-05, P2-UX-06|구현최종d143824ffd606d4cda4f3eed1d3a87eeabbb70ed; 검사보정5b3db3c06ce5d36b7271ac6e99cddb5679bc5d39; 로컬전체408/3.7m; 첫Gate51 406/2 후준비보정고유Log5·반복20별도; 최종5b3db3c Gate52UI408/12.7m·Rust887/Security59success; 신규59+기존349; 신규읽기writing0/기존mock주문별도|QueryStatusSummary/RowSelectionButton·명시focus동기알림 등기계동작만공유; History선택은ID+적용기간·poll에서대상사라지면폐기; 표는기존chart정규화자료공유·새조회/환산/시각변환없음; Log첫정상조회완료후clockpause로초기Query통지준비만교정; 제외TODO11개유지·실provider/계좌/DB/AT/native미실행; resizer설정900≠실제flex픽셀·스크롤표요약이모두한frame에보임을보장안함; 키보드5미착수footer는당시검토상태로현재완료를덮지않음|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/uiux-improvements-20261030.md)|
|<a id="task-toss-simulation-response-parsing-20261031"></a>2026-10-31 · TOSS-PARSE-01|코드0e69fb7a26eb30d3fe0c73f7cc910a8d2a60d53d 원격동일/clean; 키보드검토문서7703f38636f78e25f015a6f7841d7e599a60bb6c; 최종focused37·신규14 포함Rustlib901/replayintegration4; 로컬productionmockUI30·compiler경고0·OpenAPIv1.2.28/39paths; Gate53=Rust901/UI408 7.7m·Security60success|decoder/dependency/TLS/DTO허용범위/15초·4MiB유지; 캔들GET만안전metadata·token/POST/401/GET retry계약보존; 최종gzip거절stage=content_encoding,옛UTF8재현과구분; 실계좌API/운영로그/앱재실행/배포미실행; publicdocs gzip제공은인증캔들gzip증거아님; 원문keyboard미착수는이작업당시상태로현재KBD완료를덮으면안됨|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/toss-simulation-response-parsing-20261031.md)|
|<a id="task-keyboard-improvements-20261031"></a>2026-10-31/2026-11-01 · P2-KBD-01, P2-KBD-02, P2-KBD-03, P2-KBD-04, P2-KBD-05|01 a8cba69e1ac518d36ba5f5e173a4eff921098a6e; 02 5b45c463807cbd1385d7837b4daf3245e1659874; 03 a6c51fc9dae70873f9c77d069e841ff42c069da1; 04 73f1775a8adc9d354e04fa6bde58818d00829c44; 05 e27d7bd51ac08e03c26a0cd408f3973af66573bb; 최종검사코드7b8b9978a182c8dc138264e5b61cdb9a8c775a70 원격동일/clean; 문서commit와분리; 최종Gate58 UI557/557(14.0m)=기존408+신규149·Rust901/901(12.81s)·TODO3/bootstrap4; Security65npm/cargo성공·기존RSA예외/도구안내유지; 고유04 JSON31 mocksettings56(52성공/4의도거절),05 JSON24 profile15/auxread23/HTTP5007console매칭; 실제쓰기0|shared helper는EnterIME·opaque key/epoch·focus-only4키같은기계동작만; LTH외부180ms owner가input→region으로변경됨,모든timer불변주장금지; Home/End숫자caret OS가정대신동일부모기본input대조·기본동작재설계없음; main pending사용자닫기와성공내부닫기분리·aux닫기허용·serialization≠dedup; CI후속은실제hit/trustedkey·enabledTab/held저장분리·초기GETbody→clock통지·atomicdisabled/removed/nativepointer·Trendbaseline결과await; 04IMEs합성이벤트isTrusted=false·모든11쌍×모든Slider키Cartesian검증아님; 02 pendingStrictMode dispose→activate/concurrent중단·05HTTP200→config500/실SPAunmount미재현; 전송Promise자체abort/계좌pinning/idempotency/자동rollback보장아님; CI54~57실패원본과최종성공/로컬focused실행중복합산금지|[원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/tasks/keyboard-improvements-20261031.md)|

### 기존 이력에 부족했던 구현 요약

- **SOXQ-P1-03:** 등록 작업 루트 두 곳의 AppleDouble 후보·제거·오류가 모두 0임을 확인했다. StrongClose/VolatilityExpansion에 날짜 시작→종가 조건→실제 포지션 피드백→완료봉 적재 순서를 도입하고 replay v5 및 일봉 제한 UI를 맞췄다. live tick·next-open·장중 체결은 이 작업의 완료 범위가 아니다.
- **SOXQ-P1-01, 프로젝트 캐시 경계:** 외장파일시스템 시절 절대 Cargo target을 프로젝트 내부 상대 target/으로 이전하고 setup-local이 공통 설정을 덮지 않도록 했다. 모든 전략의 동일 평가기간과 엄격한 과거 warmup·준비 상태를 분리해 고정200/252 기본을 보존했다. 기존 사용자 미커밋 연구/코드를 유지하며 13전략 동일64봉을 재평가했다.
- **SOXQ-P2-05, SOXQ-P2-16, LTH 실제 체결 feedback, P3 사용자 가이드:** 52주 rolling 고가와 TR14 ATR 준비·차단을 보강하고 LTH에 실제 잔여수량·평균체결가 snapshot을 반영했다. 0bps fallback과 bool feedback 결함을 독립 감사 후 수정했으며 live/LTH/replay helper는 의미가 달라 공용화하지 않았다. 사용자 가이드와 TODO 생성·대기 표시도 맞췄다.
- **루트 문서 정리(1차):** 루트 일반 문서를 docs로 옮기고 README·AGENTS·CLAUDE만 진입 문서로 남겼다. PROJECT_MAP과 자동 project-map의 역할·파일명을 구분하고 상대 링크·도구 참조를 동기화했다. 연구 artifact20개 SHA를 유지했고 생산 동작은 변경하지 않았다.
- **SOXQ-P2-17, SOXQ-P2-05 선행:** Toss inclusive nextBefore·봉 종료시각과 KIS 공식 TR-ID/날짜 xymd를 확인해 공통 scoped candle page를 연결했다. UI는 평가봉을 우선 보존하고 별도252 사전자료를 bounded 수집하며 완료/무효/부족·cursor오류를 표시한다. RFC3339 실제순간과 시장날짜를 분리해 host timezone 의존을 제거했으나 LTH 세션 인증으로 확대하지 않았다.
- **루트 LICENSE 문서정리(2차):** 기존 docs 배치를 확인한 뒤 남은 루트 LICENSE를 docs/LICENSE로 원문 변경 없이 옮겼다. README 배지/링크와 AGENTS 배치 규칙을 맞추고 실행 코드·전략 설정은 건드리지 않았다.

### 묶음 회차별 게시 추적

| 작업 | 당시 검증·게시 근거 |
|---|---|
|P1-RECOVERY-01|ac3eb564ea678fe99722bbc5f3f040b4c50b1773; Rust807/UI15·recovery19(부분집합); 실provider운용P1-PROVIDER-01열림|
|SOXQ-P2-13|00ea198ad021836862ad345d58f618660fae6d0b; Rust822+후속FX1별도; live6/replay9;6연속+6분할+72분기·10536정산; 상위ID미완료·실자료조건TODO이관|
|SOXQ-P2-16|15b53286109f599242d27e5314ea2c0105ac453e; Rust831·UI20+37별도·1584실행/독립358749정산; 새OOS·성과우수아님|
|P3-REFACTOR-01|4a9e250236e8773f0b1fe3194339cc0978a8a5a8; Rust831·분리전폼5/분리후47·정적경고0|
|DEP-P3-03|HTTP10/Rust841/Gate43UI344·WS14/Rust855/Gate44UI344·Security50/51; 실OSroot/KIS인증·stalledwrite전용fixture·MSRV실행미검증|
|DEP-P3-04|653377ce4ba49bc679d1762fe0f5d6ec2409fb74; 서버24단계별·Rust873/UI344/Gate45·Security52|
|DEP-P3-05|전체UI349·Rust873; 첫Gate46명세실패/Sec53후Gate47/Sec54성공; MacChromium이nativeWebView검증아님|
|DEP-P3-06|06a84fe32fa37dd0f31a13c21f13efb6dc4e8ac1; cold64/warm32단계별·전체349·네build조합·Rust873/Gate48/Sec55; 실앱startup/계정미실행·재시작유효PNG1/stale분리|
|DEP-P3-07|fe3a3f8a8836c7c20745b3a7c7ea7e09e68ccda4; 고유focused104⊂localRust882(DB5제외),CI887(DB5earlyreturn)/UI349 Gate49/Sec56; DBscanner checksumtest는productionimport/exportroundtrip아님|
|DEP-P3-08|adf0e5a4e4635e1e3cea9540df3016e6f60b070d; 전체UI349와loader후속무경고짧은실행분리·dist10동일·Gate50Rust887/UI349/Sec57; TSspeed는executor/ambient차이포함·Windows/editorLSP/Mac12실행미검증|
|P1-UX-01|d18dce3f9cbe858a29dd8c7e5cc2c9f8383e24a1; 고유11·160px/375dark|
|P1-UX-02|36ccd96618d1681cf9c2d5e68bda6815d596e60d; 신규10+mock주문5=15|
|P1-UX-03|aaf94f274f4a5eb04f34a2d09df0665dc293a222; 신규6+일봉13=19|
|P2-UX-01|ee341d6ca6d6f3463d77fec2469e3ee986063e96; 키보드10+form5+holdings10=25|
|P2-UX-02|44c1ae5fa0e3e7dc675eb9b17132146b824d3422; resizer5+scroll5=10·설정값/실제pixel구분|
|P2-UX-03|9d0a8d290fa6f9f81d33c79dfab68763da902801; 상세4+상태6+일봉13=23·실제tabzoom2|
|P2-UX-04|8d89417e7b1df76e9b5318cc6a81b7dfd3ac823c; Log5+resizer5=10·최종CI준비보정별도|
|P2-UX-05|6fa2e4f32feb41578ff2eb164330025398e28440; 33+같은case부분1별도·wrongroute895px판별|
|P2-UX-06|d143824ffd606d4cda4f3eed1d3a87eeabbb70ed; 새8+form5+paint1=14·같은lifecycle2부분별도|
|P2-KBD-01|신규14+인접15=29|
|P2-KBD-02|신규42+인접29=71|
|P2-KBD-03|신규38+인접71=109|
|P2-KBD-04|신규31+기존109+인접11=151후artifact31별도; 제목ASCII충돌27JSON→UTF8SHA12+원title로31증거복구|
|P2-KBD-05|min6/canonical24/최종557별도|

키보드 순차 커밋: `01 a8cba69e1ac518d36ba5f5e173a4eff921098a6e`; `02 5b45c463807cbd1385d7837b4daf3245e1659874`; `03 a6c51fc9dae70873f9c77d069e841ff42c069da1`; `04 73f1775a8adc9d354e04fa6bde58818d00829c44`; `05 e27d7bd51ac08e03c26a0cd408f3973af66573bb`; `최종검사코드7b8b9978a182c8dc138264e5b61cdb9a8c775a70 원격동일/clean; 문서commit와분리`.

## 초기 설계·보안 인수인계의 통합 기록

<a id="legacy-master-plan"></a>
### 초기 설계 아카이브

초기 `#legacy-master-plan`는 개인용 Desktop/Web·JSON 거래/통계 보관·Discord 알림·자유형 화면·Theme·민감정보 비노출의 요구사항을 기록했다. React18/MUI6·JSON-only·옛 파일 트리/스키마는 당시 구상이며 현재 계약이 아니다. 현재 사용법은 [사용 가이드](user-guide.md), 저장/IPC 계약은 [코딩 가이드](coding-guide.md)·[IPC](ipc-commands.md), 구조는 [아키텍처](ARCHITECTURE.md), 운영 단계는 [사람용 계획](MASTER_PLAN.html)이 소유한다. 재사용 지침과 Discord 설정은 각각 AGENTS와 [Discord 가이드](discord-setup-guide.md)를 따른다. [초기 요구·스키마·설계 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/MasterPlan.md)은 고정 Git에서 복원한다.

<a id="security-storage-handoff"></a>
### 2026-10-06 Windows 저장·보안

기준 `8db7bf8`에서 Windows 보안 패치 `0812c71`과 저장 변경을 분리했다. Lenovo Windows x64/Rust1.92·Node22 환경에서 같은 디렉터리 temp flush→handle 종료→Windows MoveFileExW(REPLACE_EXISTING/WRITE_THROUGH), Unix rename→parent fsync의 차이를 보존했다. backup 실패 시 정상본 보존·긴 한글 경로·Windows 잠금/재시도를 검사했으며 copy/delete fallback으로 오류를 숨기지 않았다. Unix rename 후 directory fsync 실패는 이미 새 정상본이 반영됐을 수 있고, 임의 backup copy 실패의 기존 backup 보존이나 실제 정전 내구성은 보장하지 않는다.

당시 lib181통과/9실패의 저장·원장 문제를 교정한 뒤 **193통과 중 실제190+DB 즉시반환3**, LTH clock3·Chromium mock30, check/clippy/fmt·타입/build를 통과했다. Unix 실행·실제 DB·브로커 TLS/주문·desktop artifact는 미검증이었다. 당시 RSA/XML 예외3·RustSec경고14·개발 Vite/esbuild 잔여는 뒤 Mac 회차와 구분한다. [Windows 환경·패키지 버전·수정·검사 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/security-storage-handoff-2026-10-06.md).

<a id="security-mac-handoff"></a>
### 2026-10-06 / 10-24 Mac 보안·호환 검증

**10/06:** Windows `015a28a8b38d9cc9665511ccdae96afe06100c32`를 보존한 Apple Silicon/Rust1.95·Node26 작업본에서 private0600·read-only 원본 교체·backup/corrupt 복구·동시 쓰기를 합성 검증했다. lib **195통과 중 실제192+DB 즉시반환3**, clock3·mock UI30 및 check/clippy/fmt/타입/build가 통과했다. plist/quick-xml·anyhow/event-listener/rand·Tauri의 옛 XML/UNIC/selectors 경로와 Vite/esbuild를 교정해 예외3→1, 경고14→2를 기록했다. compiler 경고0과 Linux 보안 경고2는 다르다. 실제 keychain은 mock이며 실DB·브로커·desktop 실행·정전 내구성은 미검증이다.

**10/24:** 시작 `04f3b04`, Mac stable Rust1.99.1·프로젝트 Node24.22.1/npm11.19.1로 안정 호환 갱신을 검증했다. 선언 MSRV1.90 자체 실행은 하지 않았고 SQLx0.9의 최소1.94·MariaDB 인증/TLS 정책은 사용자 결정으로 남겼다. lib **775통과 중 실제 오프라인770+DB 즉시반환5**, clock4·mock UI308, Linux/Mac production60·source32, renderer3·빌드/타입·Clippy/fmt의 당시 결과를 구분했다. npm audit0→0, Cargo 취약점12→1·RSA 예외 유지와 Linux경고2를 기록했다. Xcode27.0→27.1 설치는 사용자 보류이며 이 문서 통합으로 해제하지 않는다.

당시 `64ed2b6` CI는279/29, `beebe01` CI는293/15였고 모바일 overflow와 dev/source harness 전제를 교정했다. Linux 로컬 전체308 통과와 당시 최종 harness 게시 대기는 각각 과거 상태다. 이를 현재 실패/게시 대기로 표시하지 않으며 이후 구현·최종 코드 CI는 이 문서의 최신 영수증을 따른다. 설치/worktree·패키지 준비 명령은 실행하지 않았고 현재 명령은 [HARNESS](HARNESS_MAP.md), 남은 RSA/Linux 정책은 [release-security](release-security.md), DB/MSRV·Xcode·실운영 조건은 [TODO](todo.md)를 따른다. [회차별 전체 버전표·검사·CI·근거 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/security-mac-handoff-2026-10-06.md).

## 문서 압축·통합 — 2026-11-01

현재 정본/열린 작업/가이드/결정/완료 로그를 구분해 작업35·템플릿1·구 설계1·인수인계2·연구 검증/감사6의45파일을 기존 이력·기록 규칙·연구 부록으로 통합했다. 새 문서를 만들지 않았으며 원본은 고정4021e38 Git 링크로 복원한다. 문서 MD/HTML은83→38개, 원자료·설계·실행기213개와 TODO11개의 조건·기존 이력8앵커를 보존한다. 코드·의존성·계정/보안·운영배포는 변경하지 않았다. 문서 형식75개 오류0·지도/TODO/그래프 freshness·HTML 회귀3개·원자료/ID/조건 보존 검사를 통과했고 독립 내용 리뷰의 필수 P0/P1/P2 잔여는 없다. UI557/Rust901은 앞선 정확 코드의 결과를 보존한 것이다. 상세 문서 수/분량은 실제 최종 diff로 집계하며 별도 날짜 보고서를 생성하지 않는다.
