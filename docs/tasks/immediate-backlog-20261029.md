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
| 5 | DEP-P3-03 — reqwest 0.12.28→0.13.5 및 tokio-tungstenite 0.24.0→0.30.0을 각각 전환한다. | HTTP·WS 구현/검증·CI 완료 | 5A `0dead386`·5B `270296db` 원격 일치; 각 Gate/Security 성공 |
| 6 | DEP-P3-04 — axum 0.7.9→0.8.9와 tower-http 0.5.2→0.7.1의 서버 호환성을 검증한다. | 구현·검증·CI 완료 | `653377ce` 원격 일치, Gate45·Security52 성공 |
| 7 | DEP-P3-05 — React 18.3.1→19.3.1·MUI 6.5.0→9.5.0을 단계적으로 이전한다. | 구현·검증·CI 완료 | UI `f15f192`·명세보정 `99b85eb`, Gate47·Security54 성공 |
| 8 | DEP-P3-06 — Vite 6.4.5→8.3.5·React plugin 4.7.0→6.1.2의 빌드 호환성을 검증한다. | 로컬 구현·검증 완료/게시·CI 대기 | 시작 `99b85eb`; 전체UI349·격리WK차트/종료·재실행 확인 |
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


### 5A 원격 게시·CI

- 커밋·원격 SHA: `0dead386aa0e1270cff4092009b253dc0f37e8e5`. 정상 push 후 ls-remote 일치·clean을 확인했고 GitHub의 해당 SHA 문서도 읽어 게시를 확인했다.
- [Release Gate #43](https://github.com/CountJung/KISAutoTrade/actions/runs/40835993472): 이 SHA로 2026-10-29T07:31Z 수동 실행. **성공(6분46초)**. frontend·toss-openapi, Linux stable1.99.1의 check·Rust841/841, UI344/344 통과. Actions v4 Node20 안내4개·runner 이전 notice4개는 별도이며 소스 검사 경고0과 구분한다.
- [Dependency Security #50](https://github.com/CountJung/KISAutoTrade/actions/runs/40836234313): 같은 SHA로 07:33Z 실행, **성공(24초)**. npm-audit·cargo-audit 둘 다 통과. Rust stable1.99.1/Ubuntu24.04.5. 기존 RSA 예외와 Linux RustSec2경고는 유지하며 Actions v4의 Node20 폐기예정 안내2개·Ubuntu runner 이전 notice2개는 별도 인프라 안내다. 경고를 숨기거나 CI 설정을 완화하지 않았다.


### 5B — tokio-tungstenite 0.30.0

- 5A `0dead386aa0e1270cff4092009b253dc0f37e8e5` 이후 별도 diff. 공개 subscribe/이벤트 DTO·기존 token/URL·승인/구독 JSON·parser·broadcast256을 유지하고 AppHandle adapter 밖의 private transport helper를 합성 테스트에 재사용한다. Text만 Utf8Bytes로 변환했다.
- [공식 변경](https://raw.githubusercontent.com/snapview/tungstenite-rs/v0.30.0/CHANGELOG.md)의 128KiB eager read buffer 대신 기존0.24의 `READ_BUFFER_CHUNK_SIZE=4096`에 맞춰4KiB를 명시했다. 메시지64MiB/프레임16MiB/write128KiB와 나머지 보안 기본값은 유지하며20KiB 정상 수신으로 read buffer와 메시지 크기 상한을 구분했다.
- 버전 교체 전0.24의 동일 생산 helper에서 **2개 실패를 재현**했다: peer Close 응답이 flush되지 않아 ResetWithoutClosingHandshake, 승인 send BrokenPipe 뒤 connected=true 잔류. Close 응답은2초 상한으로 flush하고 송신 실패는 false를 발행하도록 최소 수정했다. 자동 재시도/재연결·새 운용 endpoint는 추가하지 않았다.
- 합성 WS/WSS **14/14**: exact 승인/두 구독 JSON·순서/가격 필드, JSON/잘못된TR/부족필드/Binary/Pong 무시, 자동Pong중복 없음, UTF8 Close echo, EOF/잘못된UTF8·send 실패·HTTP403, 명시 두 번째 구독,4KiB/20KiB, TLS1.2·1.3 신뢰/미신뢰 거부. WSS는5A 공개 DER와 테스트 전용 root connector이며 OS trust를 변경하지 않았다. 실제 KIS 승인·OS native-root 접속을 검증한 것은 아니다.
- 0.24와0.30 upstream connect와 rustls native-root 경로를 대조했고 기존 native roots/ring을 유지했다. AWS-LC·전역 provider 설치 없음. 추가/갱신12crate의 선언 MSRV1.85는 앱1.90 이하이나 실제1.90 실행 검증은 아니다. 5A HTTP helper·DER 변경0, 기존 sha2 직접 의존성은 유지한다.
- check alltargets·strict clippy alltargets/allfeatures·fmt 통과·소스 경고0. 전체lib **855/855** 및 `cargo build --offline --workspace --locked --features custom-protocol` 통과했다. 프런트 변경이 없어5A의 타입/FSD와 원격UI344 결과를 근거로 유지하며 WS SHA의 CI는 별도 실행한다. RustSec는 기존 RSA 예외만 적용한 감사 exit0, 기존 Linux2경고 유지다.
- 독립 리뷰에서 신규 P0/P1/P2 없음. Close flush2초 상한은 코드로 확인했고 stalled-write timeout 전용 fixture는 별도 실행하지 않았다. 실거래·계좌·API키·provider·OS trust·운영배포 접근 없음.

- 5B 최종 구조 검사: mapper 인벤토리/책임표 통과, Graphify6,911노드/15,287edge 갱신. 기존 JSON설정4개의zero-node 안내는 소스 경고와 구분한다. lock에는 platform-verifier의 windows-sys 허용 범위 참조가0.52→0.61로 재해석된 변화도 있으며 Windows 실동작을 검증한 것으로 주장하지 않는다.

- 5B docs-canon `--changed`117문서의 상대링크·표·도달성·앵커·완료TODO 경고0, TODO16·project-map·graph freshness·diff 검사 통과.

- 별도 내용 감사: HTTP SHA CI성공과 WS 로컬855/CI대기·TODO16·합성/실환경 한계가 정합, 차단할 drift 없음. 완료 조건의 재연결은 기존 명시 재연결/재구독을 뜻하도록 명확히 했으며 새 자동재연결을 추가하지 않았다.


### 5B 원격 게시·CI 및 5회차 종료

- 커밋·원격 SHA: `270296db196ee7191e5c4848b1346095b1061dc7`. 정상 push·ls-remote 일치·clean 및 GitHub 해당 SHA 문서 게시를 확인했다.
- [Release Gate #44](https://github.com/CountJung/KISAutoTrade/actions/runs/40837592217): 같은 SHA의 rust·frontend·toss-openapi·e2e 전부 성공. Linux Rust **855/855**, UI **344/344**(9.4분, 07:56:55 UTC 로그). 이전 실행을 재사용하지 않았고 중복 dispatch하지 않았다.
- [Dependency Security #51](https://github.com/CountJung/KISAutoTrade/actions/runs/40839169041): 08:01 UTC 같은 브랜치·SHA로 실행했고 cargo-audit·npm-audit 모두 성공(22초). 기존 RSA 예외·Linux RustSec 경고2개는 별도 잔여로 유지하며 감사 정책을 완화하지 않았다. Actions v4 Node20 안내2개와 Ubuntu runner 이전 notice2개는 소스 검사 경고와 구분한다.
- 07:47~07:54 UTC에 Mac mini 연결이 일시 중단됐다. 기존 환경에서 상태 확인이 다시 성공했고, 08:01:42 UTC에도 `git status --short` 출력 없음·HEAD 일치를 확인했다. 다른 환경/새 작업으로 전환하지 않았다. 이후 브라우저의 Page.getFrameTree 10초 지연은 다음 상태 읽기에서 복구됐으며 이미 제출한 Security51을 재실행하지 않았다.
- DEP-P3-03의 로컬/CI 완료 조건을 충족해 이력으로 이관했다. 남은 최상위15개는 자동6+판단4+자료5이며 즉시 구역에는 DEP-P3-04~08 다섯 개가 남는다. 실제 provider·금융거래·배포 검증 완료를 뜻하지 않는다.

## 6회차 — DEP-P3-04

시작 `270296db196ee7191e5c4848b1346095b1061dc7`의 clean 상태와 5B CI 성공을 확인했다. 기존 production router의 70 URL/74 명시 method-handler와 자체 origin/token 정책을 기준으로 axum0.7 baseline→0.8.9→tower-http0.7.1 순서로 별도 영향을 검증한다. 선언된 tower-http cors feature에는 실제 CorsLayer 호출이 없으므로 새 CORS 정책을 추가하지 않는다. 격리 fixture는 keychain·운영 config·실계좌·외부 provider·운영 앱을 시작하지 않는다. 구현·검증 결과는 후속 기록한다.


### 6회차 구현·검증 결과

- production 조립을 `server/router.rs`로 추출하고 13개 capture를 `:name`→`{name}`으로 변경했다. 이전/현재 전체식 기계 대조와 독립 리뷰에서 70 URL·74 명시 method-handler, fallback·보안 layer·state의 동등성을 확인했다. 실제 handler6파일과 HTTP/WS 코드의 hash는 동일하다. 새 IPC·보안 정책·CORS layer는 추가하지 않았다.
- axum0.7.9/tower-http0.5.2 기준 → axum0.8.9/tower-http0.5.2 → axum0.8.9/tower-http0.7.1을 순서대로 각각 **서버24/24** 검사했다. 신규18개와 기존6개이며 세 실행 수를 합쳐 별개 테스트 수로 세지 않는다. 초기 fixture의 응답 예상값 교정은 별도 실패 로그를 보존했고 생산 로직은 바꾸지 않았다.
- 실제 production Router로 74 binding·13 capture, POST35개×token거부4종=140건, 70 URL의405/Allow, HEAD·Origin·OPTIONS·fallback·body/query/path 거부를 확인했다. profile/pending/risk/strategy/preview의 IPC view, KIS loopback4경로의 query/header/두 capture·오류 전달을 확인했다. 최종 0.8은 trailing JSON을 framework에서400으로 거부하며 0.7의 semantic guard 도달과 구분해 회귀를 고정했다.
- cfg(test) ServerState와 JSON-only DatabaseManager는 합성 profile/token·localhost·UUID 임시 경로만 사용한다. 운영 config·keychain·전역 DB 설치·실계좌·앱 시작은 없다. Toss는 미설정/없는 profile의 early error 범위이며 모든 성공 handler를 실행했다고 주장하지 않는다. 운영 설정 저장·외부 종목 갱신·거래 시작은 실제 허용 동작을 실행하지 않았다.
- lock 변화는 axum0.8.9/core0.5.6/matchit0.8.4/tower-http0.7.1 네 패키지 교체다. reqwest의 간접 tower-http0.6.11은 별도 유지된다. metadata상 MSRV1.90 초과 선언은 없으며 matchit은 미선언이다. 실제 검증 도구는 **Rust1.99.1**이고 Rust1.90 실행 성공을 주장하지 않는다.

| 검사 | 실제 결과 |
|---|---|
| Rust 전체 | workspace/alltargets check, 전체lib **873/873**, alltargets/allfeatures strict clippy, custom-protocol build, fmt 모두 통과·소스 경고0 |
| 프런트 | TypeScript·FSD·production 웹 빌드 통과. 현재 목록은344개/26파일(312production·32source harness)이며 이번 로컬 회차에서 전체 UI를 재실행한 것은 아니다. 같은 SHA CI에서 실행한다 |
| 보안 | 저장소의 기존 `.cache/audit-tools/bin/cargo-audit`로 기존 RSA 예외만 적용해 exit0. DB revision `40f5be6fb12ed93fc591bf4ab23b7d793b3f2239`, Linux RustSec 경고2개 유지. PATH의 bare cargo audit는 미설치 오류였고 기존 도구의 명시 경로로 해결했다 |
| 리뷰 | 최종 코드 읽기 감사에서 P0/P1/P2 없음. 전체식·lock·fixture 안전 경계와 trailing 거부 강화 확인 |
| 구조 | mapper 인벤토리/책임표 통과. Graphify6,976노드/15,390edge·source freshness 갱신, 기존 JSON설정4개 zero-node 안내는 도구 한계로 분리 |

DEP-P3-04는 로컬 구현·검증 완료이며 커밋·푸시 후 해당 SHA의 CI 확인 전까지 TODO에 열린 상태로 유지한다. 다음 순서는 DEP-P3-05다. 실제 계좌/금융거래·운영배포·OS 보안설정 변경은 수행하지 않았다.


- 6회차 문서 게이트: docs-canon `--changed`117문서의 상대링크·표·도달성·앵커·완료TODO 경고0. TODO15·project-map·Graphify freshness·lockfile·diff 검사 통과. 별도 내용 리뷰에서 TODO의 이전 파일 위치/미래형 한 문장을 현재 router13개 변경 완료로 교정했다. 문서 수정으로 테스트 실행 수치를 새로 만들지 않았다.


### 6회차 게시 차단 — 2026-10-29 08:28 UTC

검증된 27개 파일을 명시하여 git add→commit→`git push origin HEAD:fix/mac-security-storage-2026-10-06`→원격 SHA/clean 확인을 요청했으나 명령 생성 전에 자동 승인 검토가 거절했다. 사유는 이번 구체적인 코드 변경과 대상 브랜치에 대한 명확한 사용자 승인 부족이었다. 우회나 반복 실행은 하지 않았다. 후속 읽기 확인에서 HEAD는 `270296db196ee7191e5c4848b1346095b1061dc7`, staged diff는 비어 있고 6회차 변경은 unstaged/untracked로 보존돼 있다. 따라서 6회차 커밋·푸시·해당 SHA CI는 모두 미실행이다.

원래 순차 요청에 따라 다음 구현을 섞지 않고 게시 단계의 재승인 근거를 기다린다. 독립적인 7·8회차 읽기 전용 준비는 완료했으며 실제 구현/설치는 시작하지 않았다. 5B의 Release Gate44·Security51 성공은 확정된 별도 결과다. 이 거절은 환경 연결 장애나 테스트 실패가 아니다.


### 6회차 게시 재개

부모 대화가 사용자의 원래 순차 커밋·푸시 요청과 사용자 제공 AGENTS 지침 원문을 이번27개 파일·대상 브랜치에 연결한 추가 승인 근거를 전달했다. 거절된 명령이 미실행임을 HEAD/staged diff로 확인하고 같은 게시 작업을 승인 검토에 한 번 재요청했으며 승인됐다. 커밋 `653377ce4ba49bc679d1762fe0f5d6ec2409fb74`를 정상 푸시했고 ls-remote 일치·clean 및 GitHub 해당 SHA 문서 게시를 확인했다. 기존 거절을 우회하지 않았다.

같은 SHA의 Release Gate45 `40842772362`를 한 번 실행했으며 결과 확인 중이다. Dependency Security도 같은 브랜치로 한 번 제출했다. 새 금융/API/자격증명·서명·운영 배포 범위는 추가하지 않았다.


- 6회차 CI 중간 확인: Release Gate45 `40842772362`는 frontend·Toss OpenAPI와 Linux Rust1.99.1의 전체873/873 성공을 확인했다(08:36:53 UTC). UI는 계속 실행 중이다. Dependency Security52 `40842954656`은 같은 SHA의 cargo-audit·npm-audit 모두 성공했다. 최종 UI 결과 전에는 전체 Gate 성공으로 표시하지 않는다.


### 6회차 최종 CI 완료

커밋 `653377ce4ba49bc679d1762fe0f5d6ec2409fb74`의 [Release Gate45](https://github.com/CountJung/KISAutoTrade/actions/runs/40842772362)와 [Dependency Security52](https://github.com/CountJung/KISAutoTrade/actions/runs/40842954656)가 모두 성공했다. Linux Rust873·frontend·Toss OpenAPI·UI344/344(7.3분, 08:39:55 UTC)를 완료했다. 게시 승인 차단은 추가 근거에 대한 정상 재검토로 해소됐으며 새 차단은 없다. DEP-P3-04를 이력으로 이관하고 남은14개=자동5+판단4+자료5로 갱신했다. 다음 고정ID는 DEP-P3-05다.

## 7회차 — DEP-P3-05

6회차 SHA·CI 완료를 확인한 뒤 React19 단독→MUI7→MUI9 순서로 전환한다. 프로젝트 Node24.22.1/npm11.19.1을 사용하며 각 단계 peer/타입/FSD/build와 금융 mock UI를 확인한다. 금융 모델·IPC/파라미터·scope/guard 정책은 유지한다. 실제 운영 앱은 시작하지 않고 WebView 미실행 경계를 따로 기록한다.


### 7회차 단계별 구현·검증

프로젝트 Node24.22.1/npm11.19.1과 빈 임시 npm user/global 설정·독립 cache·공개 registry를 사용했다. React/DOM19.3.1·types19.3.0 → MUI7.3.11 → MUI9.5.0을 따로 전환하고 각 단계 manifest/lock/증가 diff를 보존했다. Vite6.4.5·React plugin4.7.0·TS5.9.3·Emotion11.14.0/11.14.1·chart5.2.1은 그대로다. 실제 lock 버전 변경은 React/DOM/types/scheduler·MUI 계열13개이며 패키지 제거는 없다.

| 단계 | 실제 확인한 결과 |
|---|---|
| React18/MUI6 기준 | 기존 거래/스크롤bar27개와 별도 신규UI5개 통과. 기준 dist·합성 화면6장을 보존 |
| React19/MUI6 | UI 소스 변경0, 최종 peer 오류0, 타입/FSD/build 및 기존27개 통과. 설치 도중 임시 peer 안내와 최종해석을 구분 |
| React19/MUI7 | 타입/FSD/build·기존47개와 별도 신규5개 통과. Switch의 공식 역할 변경에 맞춰 checked/요청 값과 정확한 switch 역할을 검증 |
| React19/MUI9 | focused64개·독립UI5개 통과. clean npm ci 뒤 peer/타입/FSD/lock/build와 확대집중100개 통과. 전체349/349(2.7분, production317+source harness32, 실패/skip0)도 통과 |

- UI52개 TSX의2,297개 JSX를 대조했다. slot65·Grid68·system 스타일432 전환을 정규화하면 나머지 AST가 같다. native input min/step/aria-label, 기존 sx의 배열/함수 우선순위와 Typography.color189개를 보존했다. query/API/model/theme·chart effect·scope/손절/preview generation은 바꾸지 않았다. 독립 리뷰에서도 순서·JSX spread 충돌·비속성 구조를 확인했고 소스는 모두1,000행 미만이다.
- MUI9에서 제거된 DeleteOutline import는 DeleteOutlined로 바꿨다. 독립 리뷰가 공식 이전 SVG와 현재 SVG를 좌표 정규화해 같은 도형임을 확인했다. [이전 공식 소스](https://raw.githubusercontent.com/mui/material-ui/v6.5.0/packages/mui-icons-material/lib/DeleteOutline.js).
- baseline 빌드는 경고가 없었으나 React19에서 entry635.20kB 경고가 새로 생겼다. React·DOM·client·JSX runtime을 react chunk로 분리해 최종entry424.75/mui362.82/react209.67kB, 빌드·순환 경고0을 확인했다. threshold·target·minifier·Tauri optimizer 제외는 바꾸지 않았다. 전체 전송량/성능 개선을 측정한 결과는 아니다.
- MUI7 Switch 역할과 MUI9 Alert 복합 CSS class 제거는 공식 출력 변화다. 실제 Checkbox는 유지하고 Alert의 variant/severity를 각각 정확한 class 토큰으로 검사했다. 기존 실패 로그를 보존했다. 새QA 초안의 inputmode/닫힌 preview 가정은 제품 기준과 달랐으며 제거 후 독립 spec을 채택했다.
- `ui-dependency-contract.spec.ts`5개는 dialog focus/Escape 복귀·Select 키보드/저장1회·좁은 Dashboard pagination·차트 paint/theme/resize/unmount/re-entry를 검사한다. 같은 최종 spec이 기준/MUI7/MUI9에서 각각5/5이며 브라우저 경고·오류·외부 origin0이다. Drawer 닫힘 중 aria-hidden을 데스크톱으로 오판한 테스트 경합은 CSS visibility와 정상 접근 가능·route/닫힘 대기로 교정했고 강제click·sleep·기대 완화는 없다. 최초 baseline 초안 raw로그 유실은 별도 QA 기록에 명시했다.
- Rust alltargets check와 최종 번들을 포함한 custom-protocol build 통과·경고0. Rust 소스/manifest/lock 변경은 없다. 이 회차에서 Rust 전체 테스트를 로컬 재실행한 결과로 주장하지 않으며 해당 SHA CI에서 확인한다.
- Node24/npm11의 운영 high/critical 및 전체 critical npm audit 모두 exit0, 양쪽 취약점0이다. Cargo 보안 잔여는 기존 RSA 예외·Linux2경고로 유지된다.
- Mac Chromium 합성 화면과 build는 실제 WKWebView·Windows WebView2·macOS12 실행 성공이 아니다. MUI9 기본 지원과 현재 제품 OS/번들 target을 구분하고 실제 native runtime 경계는 다음 빌드 회차에서도 따로 검증한다. 운영 앱·계좌·keychain·실제 provider·주문·배포를 시작하지 않았다.


- 7회차 최종 로컬 결과: clean install 뒤 전체349/349, 개발모드 독립UI5/5도 통과했다. 독립코드리뷰의 신규P0/P1/P2는 없으며 query/API/model/금융정책·sx우선순위·peer·icon도형·chunk초기화를 확인했다. Mac 실제 native app은 실행하지 않았다. 커밋 후 같은 SHA의 Linux CI는 후속 확인한다.


- 7회차 구조 게이트: mapper 인벤토리/책임표와 Graphify6,981노드/15,397edge를 갱신했다. 기존 JSON설정4개 zero-node 안내와 Node프로세스의 NO_COLOR/FORCE_COLOR 안내는 소스 빌드·브라우저 console 경고와 구분한다. 실제 전체349개 결과를 이 안내 때문에 소스 경고0/브라우저0보다 넓게 해석하지 않는다.


- 7회차 docs-canon `--changed`117문서의 상대링크·표·도달성·앵커·완료TODO 경고0. TODO14·mapper·Graphify freshness·lock·diff 검사 통과. 별도 내용 감사에서 상단표의 로컬완료/CI대기 상태를 본문349개 결과에 맞췄다.


- 최종 문서 내용 감사: 기존 UI스킬의 제거된 Grid/system props 예제와 MUI6 설명을 MUI9로 교정했고, HARNESS의789/322는 당시 수치로 명시했다. 읽기 재감사에서 P2가 해소됐으며 남은 문서 차단은 없다. 7회차 변경은 최종 커밋·푸시 뒤 같은 SHA CI로 확인한다.


### 7회차 원격 게시·CI 시작

커밋 `f15f192fee105600d79468291334fb10f1495e9f`의77개 파일을 정상 푸시하고 ls-remote SHA 일치·clean 및 GitHub 해당 SHA 문서 게시를 확인했다. 같은 SHA의 Release Gate46 `40848005302`와 Dependency Security53 `40848005873`을 각1회 실행했다. 결과 확인 전까지 DEP-P3-05는 열린 상태다. 다음 고정 항목은 DEP-P3-06이다.


### 7회차 CI 공개 명세 변경 보정

첫 Gate46의 frontend·Linux Rust·UI는 성공했으나 Toss OpenAPI가 v1.2.28의39 paths와 기존고정38개 불일치로 실패했다. Security53은 성공했다. 독립 공개 명세 감사에서 기존38개 삭제0·새 `GET /api/v1/rankings/investor-trading`1개 추가, 나머지 기존 검사 predicate 충족을 확인했다. 과거 명세 JSON이 없어 전체 schema byte불변으로 해석하지 않는다.

검증기 변경은 정확한 expected 목록1줄 추가뿐이며 나머지 byte는 같다. 기존 title/base·전체경로수·누락경로·계좌/오류/rate-limit 검사를 유지하고 공개 CLI 검증39개가 통과했다. API 정본/원본스킬의 현재표·신규GET 설명을 갱신하고 과거33/38 이력은 보존했다. REST 명세의 별도AsyncAPI 안내를 실제 Toss WS 구현으로 확대하지 않았다. 실제 token/계좌/시세/주문 endpoint는 호출하지 않았다. 이 보정은 별도 CI수정 커밋·푸시와 같은 SHA 재검증으로 마무리한다.


- 7회차 첫 SHA의 실제 CI 결과: Linux Rust873/873·UI349/349(9.8분, 09:24:36 UTC)·frontend 성공, 공개 명세 목록만 실패했다. 보정된 검증기와 문서의 독립 감사에서 검사 완화/앱 변경0을 확인했고 새 행의 공식 Ranking 태그도 맞췄다. 보정 후 공개 검증 exit0, docs-canon117 경고0·TODO14/지도/그래프/diff 검사 통과다. 09:28:59 UTC 동일 환경 상태 확인도 성공했으며 다른 환경으로 이동하지 않았다.


- 공개 명세 보정 커밋 `99b85eb3cdc63deceb7ec60c325559607f98f8ce`를 정상 푸시·원격 SHA/clean·GitHub 문서로 확인했다. 같은 SHA의 Release Gate47 `40850458276`와 Dependency Security54 `40850527418`을 각1회 실행했다. Toss OpenAPI·frontend·Security54는 성공했으며 전체Gate 결과는 계속 확인한다. 원래 실패 SHA를 무작정 재실행하지 않았다.


### 7회차 최종 완료·다음 재개 지점

보정 SHA `99b85eb3cdc63deceb7ec60c325559607f98f8ce`의 [Release Gate47](https://github.com/CountJung/KISAutoTrade/actions/runs/40850458276)·[Security54](https://github.com/CountJung/KISAutoTrade/actions/runs/40850527418)가 모두 성공했다. Linux Rust873/873·frontend·공개 Toss39경로·전체UI 검증을 확인했다. 7회차는 주 변경 `f15f192`와 공개명세 보정 `99b85eb` 두 커밋으로 완료했다. 승인/연결 차단은 현재 없으며 원격 CI 중복 실행도 없다.

7회차 종료 시점의 재개 메모: DEP-P3-05를 완료 이력으로 이관했다. 당시 남은13개는 자동4+판단4+자료5이며 즉시 구역은 DEP-P3-06~08 세 개다. 다음 구현은 **8회차 DEP-P3-06**이다. Vite6→7→8/plugin6 단계·Node24·5개vendor그룹·cold/warm source harness 및 안전한 WebView 보강 계획을 읽기 전용으로 준비했으며 아직8회차 설치/구현은 시작하지 않았다. 9·10회차도 읽기 계획만 준비됐다. 같은 저장소·브랜치에서 이어가며 현재 완료 기록 문서는 다음 회차 커밋에 포함한다.


## 8회차 — DEP-P3-06

### 구현·프런트 검증

7회차 최종 `99b85eb3cdc63deceb7ec60c325559607f98f8ce`와 Gate47/Security54 성공을 기준으로 시작했다. Vite6.4.5→7.3.6→8.3.5/plugin-react6.1.2로 단계 전환했다. React19/MUI9·TS5.9·금융 src와 기존 source harness는 수정하지 않았다. `rolldownOptions.output.codeSplitting.groups`의 React/MUI/Emotion/TanStack/charts 다섯 경계와 Oxc를 사용하며 target·debug·sourcemap·Tauri optimizer/server/HMR 의미를 보존했다. Node engine은 공식 `^20.19.0 || >=22.12.0`로 맞췄고 실제 실행은 Node24.22.1/npm11.19.1이다. 기존 config 별도 타입 오류를 재현한 뒤 @types/node24.19.2·명시 Promise<UserConfig>·node types만 보강했다.

- baseline64, Vite7 cold64/warm source32, Vite8 cold64/warm source32를 각 실행에서 통과했다. 최종 clean 설치 뒤 전체349/349(생산317+source32,2.9분), 타입/config 타입/FSD/lock/build·audit0을 확인했다. 실행 건수를 합쳐 하나의 실행으로 세지 않는다.
- Safari13/Chrome105 × production/debug 네 조합 모두 빌드 경고0이다. production vendor는 각각500kB 미만이고 debug는 minify=false여서 더 크다. 경고 threshold·target·검증 기대값을 완화하지 않았다. 최종 lock SHA256은 `1c3cecf4a35ea56bfe32d7e880e8d8ccb8dd9d798d388ef5614c40a2098c5d90`이다.
- 중간 Vite7 audit high와 임시 matrix probe의 개발 NODE_ENV·deprecated envFile 옵션 오류는 최종 제품 실패와 구분해 보존했다. probe를 교정한 네 조합과 정상 CLI dist 해시 일치를 확인했다. 최종 Vite8 audit0이며 esbuild/Rollup은 lock에서 제거됐다.
- 8회차 `cargo check --workspace --all-targets --locked` 및 Tauri `custom-protocol` 빌드를 실제 실행해 경고0으로 통과했다. 원래 운영 앱 startup을 실행한 결과는 아니다.

프런트 원본 로그·단계 patch·matrix chunk 소유권·동결 dist10개 해시는 Git 제외 `.cache/vite-migration-20261029/`에 보존한다. frontend 독립 리뷰에서 신규 P0/P1/P2는 없었다.

### 승인된 격리 WebView 실제 확인

원래 앱은 DB/계정 초기화에 들어가므로 기동하지 않았다. `scripts/webview-smoke/`의 build-only driver가 동일 dist10파일(1,338,474bytes)을 별도 Wry0.57.1/Tao0.37.2 앱에 동결한다. 생산 crate·Tauri IPC·DB/keyring/reqwest 의존성은 없다. 비영속 WK store를 확인한 뒤 private tmp 규칙을 설치하고 자기 asset protocol만 연다. 합성 GET/순수 preview 외 변경·외부 요청은 차단한다. 실제 운영 자료나 자격증명은 사용하지 않았다.

초기 후보를 여는 CUA 호출이 약2,157초 지연된 뒤 반환했다. F8 캡처가 동작하지 않아 성공으로 처리하지 않았고, 소스 리뷰 후 화면 상태 변화에 따른 읽기 전용 최대12회 자기 WebView 캡처로 바꿨다. 사용자에게 수정 앱 실행·종료·재실행을 확인했고 “검증을 마치고 이어서 진행해줘” 승인을 전달받아 재개했다. 이전 실행 증거는 별도 보존했다.

수정 후보 `/private/tmp/kis-webview-smoke-8-UKLMbZ/`의 source·실행 파일·asset 해시를 독립 감사와 루트에서 대조했다. executable SHA256은 `0aa7ec947e20a0ecac873491eb3fa2535136e3302e61a5cad36b1f37ced18a26`이다. offline build/strict clippy/rustfmt/Node 구문 및 순수VM4개를 통과했다.

macOS27.0.1/arm64·Rust1.99.1 환경에서 2026-10-29 11:15 UTC부터 CUA로 다음을 실제 확인했다.

1. Dashboard·Strategy 화면과 순수 합성 preview를 열어 캔들/선/매수·매도 표식의 paint를 확인했다. body 테마·캔버스 크기/색상 관측과 실제 PNG를 대조했다.
2. Settings 라이트 전환, 계좌 추가 dialog의 합성 이름 입력, Tab의 빈 APP KEY 포커스, Escape 닫기와 추가 버튼 포커스 복귀를 확인했다. 계좌·키를 입력하거나 제출하지 않았다.
3. native snapshot6개는 sidecar success=true와 PNG를 확인했다. console error/warning·CSP ledger0, overflow=false다. `GET /api/toss-market-calendar`는 고정 fixture가 없어1건 차단됐으며 해당 조회 성공은 검증하지 않았다. 주문/거래 시작·저장 경로는 실행하지 않았다.
4. 창 종료 후 정확한 테스트 실행 파일 process가 없음을 확인했다. 11:17:21 UTC 같은 앱을 명시 재실행해 새 PID20090·Dashboard와 임시 marker initiallyAbsent/readbackMatches=true를 확인했다. 라이트 설정도 남지 않았다. 다시 종료한 뒤 process 부재를 확인했다.

첫 실행과 재실행의 report/PNG는 후보의 `verified-first-run/`, `verified-restart/`에 보존한다. 재실행 캡처는 실제 생성된01번 한 장만 유효하다. 독립 리뷰가 같은 output 재사용으로 섞인 이전02~06번을 발견해 `stale-previous-run/`으로 보존·분리했고 `run-artifacts.json`에 현재 report와 유효 파일 해시/시각을 명시했다. 이전 파일을 재실행 성공 증거로 중복 집계하지 않는다. 과거 장시간 idle run의 ledger overflow는 성공으로 재사용하지 않았다. bare WKWebView 결과를 원래 Tauri startup/IPC·실제 broker·DB·Windows·macOS12 실행으로 확대하지 않는다. CSP negative control과 한글 IME는 별도 미검증이다. 2초 관측 사이 화면 변화·실패 캡처 자동 재시도는 지원하지 않는다.

### 게시 전 남은 게이트

신규 harness 지도·문서와 그래프를 동기화하고 final 리뷰를 마친 뒤 8회차를 커밋·정상 푸시한다. Release Gate frontend에는 앱과 별도의 config 타입 검사 및 순수VM4개를 추가해 재현한다. 같은 SHA의 Gate/Security 결과를 확인할 때까지 DEP-P3-06은 열린 상태다. 9·10회차는 현재 입력으로 읽기 계획만 재점검했으며 구현하지 않았다.


- 게시 전 정적 게이트: 별도 config 타입 검사, 순수VM4개, lock/TODO13/지도/diff 검사를 통과했다. Graphify는7034노드/15465간선으로 갱신했다. 새 fixture를 포함한 데이터 JSON5개는 AST 노드를 만들지 않는 도구 안내가 남으며 소스 컴파일 경고와 구분한다. 보관 검증 근거는 `.cache/vite-migration-20261029/native/`로도 복사했다.
