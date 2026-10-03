# 프로젝트 상태·캐시 정리와 SOXQ-P1-01

- 상태: Completed / 2026-10-03 / main
- 목표: 프로젝트 내부 캐시 경계를 복구하고 검증한 뒤 모든 전략의 평가 기간·사전자료·준비 상태를 분리한다.
- 위험: 보통(전략 초기화·preview 계약). 실전·모의 주문과 자동매매 시작은 실행하지 않는다.
- 시작 상태: P1-02와 SOXQ 연구의 미커밋 코드·문서·Graphify 생성물 변경이 있다. 모두 보존한다. 민감 파일은 읽지 않는다.

## 프로젝트 상태 점검 완료

- 원인: 2026-04-04 외장 exFAT/NTFS에서 AppleDouble 문제를 우회하기 위해 setup-local.sh가 머신별 절대 Cargo target 경로를 생성했다. 현재 프로젝트와 캐시는 같은 내장 APFS 볼륨에 있다.
- 외부 Cargo 캐시(약 40GB)를 프로젝트 루트 target/으로 이동했다. 기존 내부 target은 .cache/에 보존한 뒤 검증 후 cargo clean으로 정리했다.
- 이전 절대 경로를 포함한 Tauri build-script 산출물은 tauri/plugin/main 패키지만 cargo clean해 재생성했다. 도구의 사용자 공통 registry/toolchain 다운로드 캐시는 이동하지 않는다.
- .cargo/config.toml은 상대 target-dir="target"인 공통 설정으로 Git 추적한다. target/·.cache/는 Git 제외한다. setup-local.sh는 외부 경로 생성이나 공통 설정 덮어쓰기를 하지 않는다.
- 기존 5개 파일은 rustfmt로 서식만 교정했다: broker/rate_limit, commands/toss, server/profiles, storage/database_projection, storage/trade_store.
- PASS: Cargo metadata의 target_directory가 프로젝트 root/target, setup-local.sh 실행/구문 검사, 전체 cargo fmt, Rust all-target check, Rust lib 142 tests, Clippy all-target/all-features warning 0, TS noEmit, FSD, lockfile.
- 과거 p1-02-validation.md의 fmt 미통과 기록은 당시 결과로 보존한다. 이번 점검에서 해당 차이를 해결했다.

## P1-01 완료 조건

- [x] 모든 전략에서 입력 평가 기간을 같게 유지한다. 암묵적으로 절반을 준비 구간으로 소비하지 않는다.
- [x] MA/RSI/모멘텀/이격도/연속/돌파 실패를 실제 과거 관측치로 초기화하고 이전 지표를 복원한다. 과거 자료에 주문이나 가상 포지션을 만들지 않는다.
- [x] 추세 200봉·신고가 252봉 기본값을 유지한다.
- [x] 사전자료 부족·지표 준비 중·조건 불충족·신호 있지만 무체결을 UI와 결과에서 구분한다.
- [x] 동일 범위·부족 자료·첫 신호·미래 봉 불변·scope/분봉 경계 fixtures, Rust 전체 검사, TS/FSD, focused Playwright를 통과한다.
- [x] SOXQ 같은 입력 재실행과 관련 TODO/스킬/IPC/프로젝트 맵/그래프를 갱신한다.

## 도구와 계약

- Serena symbol navigator에게 초기화·preview/TS/UI 호출자 영향 분석을 위임한다.
- 파일 추가/분리 후 project mapper에게 지도 및 문서 drift pass를 위임한다.
- 기존 IPC 이름·주문/guard/risk/provider 경계를 유지한다. 준비 상태와 사전자료/평가 경계의 추가 계약만 동기화한다.
- Graphify는 최종 구조/소스 변경 뒤 갱신한다.

## 검증·잔여 기록

- 내부 `target/` 약 11GB, `.cache/` 약 2MB. 외부 project Cargo target 경로는 없어졌고 이동 전 내부 중복 산출물도 정리했다. `.cache/research/soxq-replay/`에 임시 Cargo manifest/lock와 repeat 검증 자료를 보관한다.
- PASS: `cargo fmt --all --check`, `cargo check --workspace --all-targets --locked`, `cargo test --workspace --lib --locked` **167 passed**, `cargo clippy --workspace --all-targets --all-features --locked` 코드 경고 0.
- PASS: `npx tsc --noEmit`, `npm run check:fsd`, `npm run check:lockfiles`, `npm run build:web`.
- PASS: `npx playwright test tests/e2e/strategy-scrollbar.spec.ts` **21 passed / 39.5초**, mock transport. 신규 케이스는 동일 평가 기간/252봉 사전자료, provider 부족과 좁은 화면, Toss 200봉 경계/무체결 표시를 검증한다. 외부 DB 환경 없는 contract 테스트는 내부 skip 경로이므로 실제 DB contract 검증 완료로 간주하지 않는다.
- PASS: Graphify refresh + `npm run check:graphify`, project mapper의 map/document drift pass + `npm run check:project-map`, `git diff --check`.
- 도구 진단: Playwright Node 색상 환경변수 충돌(NO_COLOR/FORCE_COLOR) 2줄, Graphify의 설정 JSON 4개 AST 노드 없음 메시지가 남는다. 코드 컴파일/타입/Clippy 경고와 구분하며 제거했다고 보고하지 않는다.
- Serena 교차 심볼 참조 및 독립 리뷰 완료. seed-only 조건 판정, 날짜 granularity 혼합, 비정렬 추세 기간 cap, live 신고가 미확정 최신 봉 경계를 보완했다. 자료 부족 후에도 보유 손절이 작동하는 251봉/0봉 재초기화 회귀를 포함했다.
- [P1-01 재평가](../docs/research/soxq-3m-20261003/p1-01-validation.md): 13전략×2모드 모두 2026-07-06~10-02의 64봉 평가. 지원 12전략×2모드 generatedAt 제외 결과 재현, 각20bp 비용 스트레스 보존. 초기 v2/v3 보고서·차트는 해당 시점 기록으로 유지한다.
- provider 페이지 조회와 완료 일봉 보장은 `SOXQ-P2-17`에 별도 기록했다. 단일 응답 자료가 부족하면 기간/200·252 기본값을 줄이지 않고 평가 불가/부족 상태를 표시한다.
- 실제 주문/자동매매 시작·위험 설정 완화는 실행하지 않았다. 기존 미커밋 변경을 보존했으며 커밋은 만들지 않았다.

날짜 이벤트(P1-03), LTH 역사 시각/장중 모델(P1-04/05), next-open 체결(P2-14), true-range ATR(P2-16)은 별도 과제다.
