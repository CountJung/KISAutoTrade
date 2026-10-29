# 2026-10-29 즉시 작업 순차 실행 기록

- 시작 기준: `5437c07bc6c2bf0876c0ddc211ddeabb6cd632d7`, `fix/mac-security-storage-2026-10-06`, 깨끗한 작업 트리.
- 사용자 범위: [TODO](../todo.md)의 “1. 바로 진행할 코드·개선 작업” 전체를 문서 순서대로, 회차별 구현·검증·문서·커밋·푸시. 아래 순서는 시작 시 고정했다.
- 제외: 2의 DEP-P3-02, 3의 사용자 정책 결정, 4의 실자료 검증. 실주문/취소/자금이체·실계좌/키·유료자료·운영배포·보안 권한 확대는 수행하지 않는다.
- 실제 의존성 때문에 순서를 바꾸면 해당 회차에 근거를 기록한다. 앞 회차 차단이 독립 후속 작업까지 차단하지 않는다. 확인되지 않은 항목은 열린 상태로 유지한다.

| 회차 | 고정 ID·제목 | 상태 | 검증·커밋 근거 |
|---|---|---|---|
| 1 | P1-RECOVERY-01 — 응답 불명 주문의 운영자 대조·복구 UI를 구현한다. | 로컬 구현·검증 완료 | 아래 최종 게이트; `ac3eb564ea678fe99722bbc5f3f040b4c50b1773` 원격 SHA 일치 |
| 2 | SOXQ-P2-13 — 가격조건의 실제 소유·손절 후 재진입과 공통 guard를 검증한다. | 로컬 완료·실자료 조건 이관 | live6/replay9, 보관78일치·정산10,536; 전체 ID는 열린 상태 |
| 3 | SOXQ-P2-16 — 구간 경고·표본 해석을 교정하고 별도 walk-forward를 평가한다. | 대기 | — |
| 4 | P3-REFACTOR-01 — 1,000라인 초과 파일을 책임 단위로 분리한다. | 대기 | — |
| 5 | DEP-P3-03 — reqwest 0.12.28→0.13.5 및 tokio-tungstenite 0.24.0→0.30.0을 각각 전환한다. | 대기 | — |
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
