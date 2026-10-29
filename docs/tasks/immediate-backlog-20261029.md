# 2026-10-29 즉시 작업 순차 실행 기록

- 시작 기준: `5437c07bc6c2bf0876c0ddc211ddeabb6cd632d7`, `fix/mac-security-storage-2026-10-06`, 깨끗한 작업 트리.
- 사용자 범위: [TODO](../todo.md)의 “1. 바로 진행할 코드·개선 작업” 전체를 문서 순서대로, 회차별 구현·검증·문서·커밋·푸시. 아래 순서는 시작 시 고정했다.
- 제외: 2의 DEP-P3-02, 3의 사용자 정책 결정, 4의 실자료 검증. 실주문/취소/자금이체·실계좌/키·유료자료·운영배포·보안 권한 확대는 수행하지 않는다.
- 실제 의존성 때문에 순서를 바꾸면 해당 회차에 근거를 기록한다. 앞 회차 차단이 독립 후속 작업까지 차단하지 않는다. 확인되지 않은 항목은 열린 상태로 유지한다.

| 회차 | 고정 ID·제목 | 상태 | 검증·커밋 근거 |
|---|---|---|---|
| 1 | P1-RECOVERY-01 — 응답 불명 주문의 운영자 대조·복구 UI를 구현한다. | 로컬 구현·검증 완료 | 아래 최종 게이트; `ac3eb564ea678fe99722bbc5f3f040b4c50b1773` 원격 SHA 일치 |
| 2 | SOXQ-P2-13 — 가격조건의 실제 소유·손절 후 재진입과 공통 guard를 검증한다. | 로컬 완료·실자료 조건 이관 | `00ea198ad021836862ad345d58f618660fae6d0b` 원격 SHA 일치; 전체 ID는 열린 상태 |
| 3 | SOXQ-P2-16 — 구간 경고·표본 해석을 교정하고 별도 walk-forward를 평가한다. | 구현·검증 완료 | `15b53286109f599242d27e5314ea2c0105ac453e` 원격 SHA·GitHub 문서 확인 |
| 4 | P3-REFACTOR-01 — 1,000라인 초과 파일을 책임 단위로 분리한다. | 구현·검증 완료 | `4a9e250236e8773f0b1fe3194339cc0978a8a5a8` 원격 SHA 일치 |
| 5 | DEP-P3-03 — reqwest 0.12.28→0.13.5 및 tokio-tungstenite 0.24.0→0.30.0을 각각 전환한다. | 5A HTTP 로컬 완료·5B WS 대기 | 아래 분리 검증; CI는 커밋별 후속 기록 |
| 6 | DEP-P3-04 — axum 0.7.9→0.8.9와 tower-http 0.5.2→0.7.1의 서버 호환성을 검증한다. | 대기 | — |
| 7 | DEP-P3-05 — React 18.3.1→19.3.1·MUI 6.5.0→9.5.0을 단계적으로 이전한다. | 대기 | — |
| 8 | DEP-P3-06 — Vite 6.4.5→8.3.5·React plugin 4.7.0→6.1.2의 빌드 호환성을 검증한다. | 대기 | — |
| 9 | DEP-P3-07 — 나머지 Rust·UI 의존성 후보의 실제 필요와 영향부터 확인한다. | 대기 | — |
| 10 | DEP-P3-08 — TypeScript·패키지매니저 major 후보를 개별 평가한다. | 대기 | — |

## 1회차 — P1-RECOVERY-01

- 조사: 기존 예약에는 owner/profile/mode/시각/clientOrderId가 없어 기존 unknown의 귀속을 추정하지 않는다. 신규 제출 전 비민감 snapshot을 영속화하고 operator 대조 승인은 pending 복원만 수행한다. 예산 직접 해제·체결 합성·재주문은 없다.
- 호출 경계: 현재 LAN 서버에 관리자 인증이 없으므로 새 복구 조회/승인은 desktop IPC로 한정한다. 계정/인증·보안 설정은 바꾸지 않는다. 불명확/legacy·계좌 또는 투자모드 불일치는 보류한다.
- 도구: rust-skills·frontend-fsd·ui-conventions·react-best-practices와 docs-canon 적용. Serena 도구가 없어 읽기 전용 symbol 감사는 rg/직접 읽기로 수행했다. 신규 모듈의 project_mapper와 별도 리뷰를 완료 게이트에 포함한다.
- 검증 계획: 중복 승인·다른 증거·저장 실패/재시작·부분체결·역순·terminal·scope/mode/legacy 및 웹 거부, mock UI 입력/승인/실패/모바일/늦은 응답. Rust/TS/FSD/build·관련 회귀와 문서 검사.
- 실제 provider 운용 검증: P1-PROVIDER-01에 미완료로 유지한다.


### 1회차 구현·감사 경과

- 신규 제출 전 snapshot, desktop 조회/대조 승인, durable receipt→원래 pending 복원을 구현했다. 승인 자체는 provider 확인이나 체결/현금/소유량/예산 해제 완료가 아니다.
- 독립 감사의 정상 완료 provider ID 재사용, intent 저장 후 재시작 근거 유실, pending 저장 실패 후 재조회 재시도 소멸을 수정했다. retained primary 이력 전체의 보수적 충돌 검사와 backup/격리본만 남은 경우 보류, 저장된 근거 복원 및 같은 승인 저장 재시도를 추가했다.
- 정상 pending/terminal(복구 receipt 없음)은 복구 목록에서 제외한다. 미해결→연결대기→종결 순으로 최대100개를 표시하고 전체개수를 따로 반환한다. receipt256개와 이력 category별 2,048문서·16MiB·50,000행 상한을 넘으면 보류한다. 보관기간 밖/별도 archive의 사실은 자동 검증 범위가 아니다.
- 중간 검증: 예산/복구16개, submission15개, TypeScript/FSD·웹 빌드 통과. UI11개는 dev 합성검증이며 직전 UI10개+Settings4개 production14개도 통과했다. 최종 소스 재검증은 아래 최종 게이트에 별도 기록한다. 초기 E2E에서 source 경로까지 API mock이 차단한 fixture 설정 오류를 수정했다. Chromium의 NO_COLOR/FORCE_COLOR 환경 안내는 소스 빌드 경고와 구분한다.
- 추가 감사에서 unknown 제출의 guard 누락을 확인하고 원래 제출시각·사유·예약 ID를 이용한 멱등 guard 복원을 추가했다. 당일 손절·다음날60분 경계·최신기록 보존·반복승인·terminal/fullfill/flat/restart 뒤 실제 shared 제출 차단을 검증했다. 승인시각으로 새 거래를 기록하거나 실제 체결 수치를 합성하지 않는다.
- 실제 provider/실계좌/자격증명/주문·취소·자금이체/운영배포는 실행하지 않았다. 로컬 완료와 원격 게시 확인은 아래 검증 기록 및 Git 커밋으로 구분한다.


### 1회차 최종 게이트

| 검사 | 결과·범위 |
|---|---|
| Rust | `cargo fmt --all -- --check`, `cargo check --offline --workspace --locked`, `cargo test --offline --workspace --lib --locked` **807/807**, `cargo clippy --offline --all-targets --all-features --locked -- -D warnings` 통과·경고0 |
| focused | budget16개와 submission15개; 최종 guard 포함 `cargo test --offline --lib recovery` **19/19**. 서로 겹치는 테스트 수를 합산하지 않음 |
| 프런트 | `npx tsc --noEmit`, `check:fsd`, `build:web` 통과·빌드 경고0 |
| UI | 최종 `CI=1 npx playwright test tests/e2e/budget-recovery.spec.ts tests/e2e/settings-database.spec.ts` **15/15**. production 번들+mock native/REST, 실제 Tauri/provider 운용 아님 |
| 감사 | 읽기 전용 재감사에서 새 P0/P1/P2 지적 없음. 사용자 기존 변경 없음. 실제 계좌·민감파일·금융거래 미접근 |
| 문서 | docs-canon 검사: 상대링크·표·도달성·앵커·완료TODO 경고 모두0; 내용 감사 별도 수행 |
| 지도·생성물 | project_mapper 책임표/인벤토리 pass, Graphify source freshness 갱신. JSON 설정 예제4개 zero-node 안내는 실행코드 추출 실패가 아니며 Graphify 한계로 기록 |

- 코드 근거: [복구 경로](../../src-tauri/src/trading/order/budget/recovery.rs), [이력 충돌](../../src-tauri/src/trading/order/budget/recovery/history.rs), [guard 복원](../../src-tauri/src/trading/guard/recovery.rs), [UI](../../src/pages/settings/ui/budgetRecoverySection.tsx).
- 1회차 종료 시 다음 ID는 SOXQ-P2-13이었다. 이후 회차 결과는 아래 절을 따른다.

- Graphify 생성기 한계: 기본 HTML 5,000노드 한도로 시각화가 생략됐고 cluster-only 재로딩이 노드를 축소했다. 최초 갱신된 6,669노드·14,678edge·567community JSON을 보존한 뒤 동일 Graphify HTML/report 렌더러에 직접 전달해 세 산출물을 일치시켰다. source freshness 통과, LLM/API 라벨링 미사용. 다음 갱신에서는 기존 규모에 맞는 HTML 한도를 먼저 지정한다.


## 2회차 — SOXQ-P2-13

- 기존 PC 매도는 설정수량이고 exact owner 제출 경로가 실제 수량으로 제한한다. 이 계약과 live/replay 평균가 반올림, native 일반 guard/연구 overlay 별도 경계를 유지한다. 수량 정책·기본 guard·실거래 설정을 바꾸지 않는다.
- 실거래 없이 live 제출/거절/pending/부분체결/확정 소유 피드백, production preview의 실제 native 원가·손절/재진입·비용·분할불변 fixture를 먼저 보강한다.
- [재현 설계](../research/price-condition-contracts-20261029/design.json)는 기존2023~2025 일봉 연구의 PC3종목·원래 기준가/수량·익절5%/손절3%·두비용·guard를 그대로 고정한다. 현재 생산API6연속+6대체분할+72독립분기의 코드 호환성 재현이며 새로운 OOS나 장중 효능 평가가 아니다. 과거 원본/결과는 덮어쓰지 않는다.
- 실제 장중 자료의 시각/시장일/DST/세션/coverage와 실제 접수·체결 사례는 여전히 별도 미완료 조건이다.


### 2회차 결과·검증

- 생산 정책 변경0, 테스트 등록6줄과 live/replay 신규 leaf만 추가했다. live6개·replay9개 통과; replay34합성호출/12분할쌍은 실제 시장 표본 수가 아니다.
- 초기 live fixture의 mut 누락과 지정가보다 높은 매수 fill 입력을 수정했다. 기존 예산 보호를 완화하지 않았다. 실제 잔량 후속 매도는 기본 일일매도 제한이 먼저 막고 일반 guard도 별도 적용함을 확인했다.
- `cargo fmt --all -- --check`, `cargo check --offline --workspace --locked`, 전체lib **822/822**, strict clippy 경고0. 리뷰 후 FX실적용 단정 추가→해당1개 재실행 통과·fmt/strict clippy 재확인. 전체822 실행과 후속1 실행을 하나의 새전체실행으로 합치지 않는다. `npx tsc --noEmit`, FSD통과. UI/IPC 변경없음으로 UI E2E는 다시 실행하지 않았다.
- 현재 생산API **6연속+6대체분할+72독립분기**, 기존 보관78개 전체 report/signals/preparation일치와 독립 기록체결 산술**10,536건** 통과. [원장/재현/해석 정본](../research/price-condition-contracts-20261029/report.md). 이전commit별도빌드대조나 새OOS/실제장중평가가 아니다.
- stock_analyst가 압축SHA·78고유키·전체대조·현금/비용/손익/회전·CSV/aggregate를 읽기전용으로 독립 확인했다. 코드리뷰 새P0/P1/P2없음. FX시나리오가 실제정산에 적용되는지 추가단정을 반영했다.
- SOXQ-P2-13 전체는 **미완료**다. 로컬 구현 가능한 검증은 이력에 보존하고 실제 장중 자료/효능 조건을 같은ID의 TODO4구역으로 이관했다. 현재 남은18개는 자동9+판단4+자료5이며 다음 즉시항목은 **SOXQ-P2-16**이다.

- 최종 문서/생성물: docs-canon 링크·표·도달성·앵커·완료TODO 경고0, 내용 감사 이상없음. project_mapper 책임/인벤토리 pass, TODO18개·project-map·Graphify freshness·diff 검사 통과. Graphify6727노드/14821edge, JSON설정4개 zero-node 안내는 기존 도구 한계로 구분했다.


## 3회차 — SOXQ-P2-16

- v23은 동일 장부 앞/뒤 관측 진단을 추가하고 후반0 보합/음수 손실 문구를 구분한다. 기간 정규화 없는 양수 절반 조건으로 과최적화를 판정하지 않는다. 원래 수익/MDD/완료 매도·회계·체결·guard와 TR14/사전warmup/간격·LTH 평가불가 gate를 유지한다.
- 현재/A-B의 flat/LTH/continuous 공통 UI에 관측·보유·경계 잔량/pending·실현/잔여/배당/자산 증감을 표시한다. 구진단·손상 응답은 미확인으로, 완료 매도0과 작은 표본은 한계로 설명한다. 저장시 축약된 거래/곡선에서 진단을 재구성하지 않는다.
- 사전 설계한 [별도 walk-forward](../research/phase-walk-forward-20261029/design.json)는 이미 관측한 동결 일봉의 회고 절차검증이다. 동일 장부 70/30과 분리하고 새 OOS·성과 우수·실제 장중 효능으로 주장하지 않는다.
- 독립 코드리뷰의 부분매도 원가 문구 지적을 수정했다. 실제 매도 수량에 배분된 원가·매수비용을 뜻하며 원 포지션 전체 원가를 매번 차감하지 않는다. 후속 읽기 전용 감사에서 새 P0/P1/P2 지적 없음.


### 3회차 최종 게이트

| 검사 | 실제 결과·범위 |
|---|---|
| Rust | alltargets check·strict clippy 경고0, simulation76/preview122 집중 검사, 최종 전체lib **831/831**. 신규9개 포함. 기존 TR/ATR/warmup/간격·LTHgate 유지 |
| 프런트 | TypeScript·FSD·production 웹 빌드 통과·소스 경고0 |
| UI | 구간/연속/LTH3spec **20/20**, 회계/ATR/scrollbar 인접3spec **37/37**, 별도 두 실행. production 번들·합성API이며 실제금융호출0 |
| 연구 | 사전고정 **1,584회**, PC보관78경제값 일치·대체분할6, phase3,324·독립산술358,749검사 통과. 최대금액차이약0.01원. [연구 정본](../research/phase-walk-forward-20261029/report.md) |
| 감사 | 코드 읽기전용 리뷰의 부분매도 문구 수정 후 새P0/P1/P2없음. 루트독립1,584고유키·비용별표본·잔량·manifest SHA 확인 |
| 경계 | 실제broker·계좌·키·주문·취소·자금이체·운영배포 미실행. 회고자료를 새OOS로 주장하지 않음 |

- 단말의 `NO_COLOR`/`FORCE_COLOR` 안내와 Graphify 기존JSON설정4개 zero-node 안내는 소스 컴파일 경고와 구분한다. Graphify6,766노드/14,944edge의 AST산출물·freshness를 갱신했으며 LLM/API라벨링을 사용하지 않았다.
- P2-16 로컬 수용조건을 완료 이력으로 이관했다. 남은17개는 자동8+판단4+자료5, 다음 즉시ID는 **P3-REFACTOR-01**이다. 고정10회차 중3회까지 진행했으며 전체 작업 완료는 아니다.

- 문서 최종 형식 검사: docs-canon `--changed` 117문서, 상대링크·표·도달성·앵커·완료TODO 경고0. 별도 내용 검토에서 현재 계약·회고 연구 한계·남은 작업 포인터를 확인했다. project-map·TODO17개·Graphify freshness·diff 검사 통과.


## 4회차 — P3-REFACTOR-01

시작 `15b53286109f599242d27e5314ea2c0105ac453e`의 깨끗한 작업 트리. 실제 LTHeditor는993행이었지만 요청된 ticker picker/preview 책임분리를 유지했다. 동작개선·API변경·정책변경을 섞지 않았다. Serena사용불가로 symbol 역할의 rg참조추적을 위임했다.

| 대상 | 이전→현재 | 분리 책임·현재 행수 |
|---|---|---|
| Trading Page | 1056→474 | `useTradingOrderModel.ts`414: broker별query gate·state/effect/주문closure, `tradingOrderForm.tsx`234: 공통 주문 표시 |
| Toss commands | 1093→13 | `accounts.rs`111, `diagnostics.rs`379, `preflight.rs`236, `orders.rs`371; 기존small_order·pub re-export유지 |
| LTH editor | 993→413 | `useLeveragedTrendHoldEditorModel.ts`343, `leveragedTrendHoldTickerPicker.tsx`177, `leveragedTrendHoldPreviewPanel.tsx`161 |

### 동등성·검증 근거

- Toss 원래32fn/struct를 attrs/signature/visibility/body포함 byte비교해32/32동일. commands.rs/lib.rs/서버3caller/small_order본체·테스트7파일 무변경이며 기존commands::*와Tauri7등록을 check로확인했다. 독립리뷰도별도추출대조했다.
- 프런트 원래 Trading56statement/7helper, LTH68statement/5helper AST동일. 추출패널 재확장JSX도 공백/투명fragment정규화후동일. 상태는 parent의customhook1회에남겨effect순서/비동기closure/key/query/IPC를보존했다. 공개page entry와hasInvalidLthEntries도유지했다.
- `cargo check --locked`, 전체lib **831/831**, strictclippy alltargets/allfeatures 경고0, fmt통과. 기존Toss합성2개도통과. TypeScript/FSD/production build통과.
- 신규거래폼mock5개는 분리전production번들에서5/5통과했고 분리후전체선택47개에도포함된다. 최초해외차트mock이잘못된{}를반환해발생한fixture실패를[]로교정했다. 실제API실패나제품로직수정이아니다.
- 분리후 `trading-form-contract`, `strategy-scrollbar`, `lth-minute-timing`, `strategy-configuration-errors`, `strategy-replay-assessment`, `toss-commission-ratio` **47/47**.46개production+1개기존sourceharness, 모두금융mock. 입력검증/거절후초안/성공정리/시장가0/해외거래소/모의매도제한/Toss사전·닫힌세션·auto payload/모바일·LTH지연응답/설정/평가gate검증.
- 원본/변경파일만읽었으며 실주문·취소·계좌·키·DB·운영배포를실행하지않았다. 현재남은16개=자동7+판단4+자료5. 다음고정ID는 **DEP-P3-03**이며 전체10회차중4회차까지다.

- 최종 독립리뷰: 별도Rust item/TS AST·재확장JSX 대조까지 통과, 새P0/P1/P2 없음. docs-canon `--changed`117문서의 링크/표/도달성/앵커/완료TODO경고0, 별도내용검토에서 책임포인터·진행수·미검증경계확인. mapper·TODO16·Graphifyfreshness·diff통과. Graphify6,802노드/14,973edge, 기존JSON설정4개zero-node안내만있으며소스경고와구분한다.


## 5회차 — DEP-P3-03

시작 `4a9e250236e8773f0b1fe3194339cc0978a8a5a8`. TODO의 별도 diff·검증 조건에 따라 **5A HTTP → 5B WebSocket**을 각 커밋으로 나눈다. 고정10개 순서는 유지한다. 공식 reqwest0.13.5/tokio-tungstenite0.30.0을 재확인했고 열린 Dependabot #13(Rust minor/patch), #11(frontend minor/patch)은 읽기만 했으며 변경하지 않았다.

### 5A — reqwest 0.13.5

- [공식 0.13 변경](https://github.com/seanmonstar/reqwest/releases/tag/v0.13.0)의 기본 platform verifier/aws-lc 전환을 그대로 적용하면 기존 신뢰 정책과 달라진다. 따라서 `default-features=false`, `json/cookies/query/form/rustls-no-provider`, 명시 ring과 **기존과 같은 webpki-roots1.0.11**로 만든 preconfigured TLS를 사용한다. 시스템 CA를 읽거나 전역 crypto provider를 설치하지 않으며 인증서/호스트명 검증을 유지한다. rustls0.23.46·TLS1.2/1.3·HTTP1.1 ALPN도 보존했다.
- `http.rs`의 공통 builder를 기존8파일14생성점이 사용한다. 호출부 timeout·cookie·proxy·header·redirect 설정을 보존했다. Toss OAuth의 `.form()`과 차트 `.query()` 기능을 명시했다. 기존 WS0.24의 lock entry는 동일하다. reqwest0.12 중복은 제거됐다.
- 새 HTTP/TLS **10/10**: query/header/JSON/form, cookie/redirect, HTTP503 metadata/count1, timeout/명시proxy, TLS12/13·SNI·ALPN, 미신뢰CA/잘못된hostname 거부, WS provider 공존, same/cross-origin 민감헤더, auth/order POST본문 완전 수신 후 무응답 단절 시 각각 전송1회·transport error를 검증했다. 합성 loopback만 사용하고 fixture DER/key는 공개 테스트 전용이며 OS trust에 설치하지 않았다.
- upstream retry classifier의 행동 코드는 기존과 같고 HTTP2/3는 활성화하지 않았다. reqwest0.13의 scheme 변경 redirect 민감헤더 제거는 upstream 보안 변경으로 유지한다. 동일 host/port HTTPS→HTTP fixture는 미실행이며 다른 port origin 제거 테스트와 혼동하지 않는다.
- 전체lib **841/841**, check·strict clippy alltargets/allfeatures·fmt 경고0, TypeScript·FSD 통과. `cargo build --offline --workspace --locked --features custom-protocol`도 통과했다. docs-canon `--changed`117문서의 링크·표·도달성·앵커·완료TODO 경고0, mapper·TODO16·graph freshness·diff 검사도 통과했다. 앱 실행·실제 provider/계좌/키/주문·배포는 하지 않았다.
- all-target dependency metadata에서 새 crate 선언 MSRV는 최대1.85이며 프로젝트1.90을 넘지 않는다. 실제 실행은 설치된 Rust1.95이며 Rust1.90·다른 플랫폼·모바일 실동작 검증이 아니다. Tauri의 직접 reqwest builder는 dev+mobile 조건의 경로이며 desktop 활성 경로와 구분했다.
- RustSec DB `40f5be6fb12ed93fc591bf4ab23b7d793b3f2239`(2026-10-28)에서 raw audit는 기존 RSA 예외 `RUSTSEC-2023-0071` 때문에 exit1, 기존 예외만 적용하면 exit0. Linux 경고 `RUSTSEC-2024-0429`·`RUSTSEC-2024-0370` 2개가 남는다. 추가 ignore·경고 억제 없음. [보안 정본](../release-security.md).
- 독립 코드 감사: 생성점14·root/provider·TLS·feature graph·POST단절 fixture·metadata를 별도 확인했고 새 P0/P1/P2 없음. 기존 사용자 변경은 없었다.
- CI 경과: 초기 브라우저 로그아웃/dispatch 도구 미지원으로 실행하지 못했다. 사용자가 직접 로그인한 뒤 CountJung 로그인과 Run workflow 버튼을 확인했다. `release-gate.yml`·`security.yml`은 read-only contents, mock 테스트/빌드·감사이며 배포/릴리스 게시/서명키/계좌 호출이 없다. 커밋·푸시 후 해당 SHA로 실행하고 결과를 후속 기록한다. `release.yml`은 실행하지 않는다.

- 5A 지도·그래프: mapper 책임표/인벤토리와 check 통과. Graphify6,849노드/15,113edge, 기존 JSON설정4개 zero-node 안내만 남으며 소스 경고와 구분한다.

- 별도 문서 내용 감사에서 보안 정본의 이전 점검일을 과거 기록으로 바로잡았다. 현재 검증 수치·부분 완료·CI 대기와 신뢰 경계가 정합하며 추가 문서 차단 없음.
